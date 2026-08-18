//! Flat, allocation-free determinization and minimization (Builder A core).
//!
//! The engine's original subset construction (`Nfa::determinize_capped` in
//! `dfa.rs`) is correct but pays for three things on every step:
//!
//!   * `Nfa::trans` is a `Vec<Vec<State>>` -- one heap allocation per
//!     (state, symbol), so the inner loop chases a cold pointer for every
//!     source state of every subset.  A 20 s `sample` of the `tail-a` panel
//!     case spends 99 % of its time in exactly that loop.
//!   * every discovered subset is stored twice as an owned `Vec<u64>` (once as
//!     the `HashMap` key, once in the `order` vector), so a construction with
//!     N subsets does 2N allocations and keeps 2N bitsets live.
//!   * the successor bitsets are recomputed per symbol, re-scanning the set
//!     bits of the current subset `alpha` times.
//!
//! This module replaces all three: transitions live in one flat `u32` array
//! (`FlatNfa`), subsets live in one flat `u64` arena addressed by an
//! open-addressing index, and the successor loop is transposed so the bits of
//! a subset are scanned once for all `alpha` symbols at a time.
//!
//! Everything here is **bit-for-bit identical** to the old path by
//! construction: subsets are discovered in the same BFS order (queue order,
//! symbols ascending), so state `i` of the produced DFA is the same subset as
//! state `i` of the old one, and the transition table and accept vector are
//! equal element by element.  `AM_FAST_VERIFY=1` asserts this at run time by
//! building both and comparing.
//!
//! Flags:
//!   `AM_PAR=<threads>`  frontier-parallel subset construction (implies AM_FAST).
//!                       **Default since 2026-08-19: `min(8, cores-2)`, i.e. on.**
//!                       `AM_PAR=1` restores the pre-2026-08-19 reference path;
//!                       `AM_PAR=1 AM_FAST=1` the serial flat core.
//!   `AM_FAST=1`         use this module for determinization + minimization
//!                       (implied by the default `AM_PAR`; only meaningful with
//!                       `AM_PAR=1`)
//!   `AM_LAZY_CLOSED=1`  default OFF.  When the last variable is projected away,
//!                       answer the resulting closed sentence by NFA reachability
//!                       instead of determinizing (see [`closed_verdict`])
//!   `AM_FAST_VERIFY=1`  default OFF.  Run old and new side by side and assert
//!                       equality

use crate::dfa::{Dfa, State, SUBSET_TICK, peak_bump};
use std::sync::OnceLock;

// ------------------------------------------------------------------ flags

fn env_flag(name: &'static str) -> bool {
    std::env::var(name).map(|v| v != "0" && !v.is_empty()).unwrap_or(false)
}

/// `AM_FAST=1`: route determinization/minimization through this module.
pub fn fast_enabled() -> bool {
    static F: OnceLock<bool> = OnceLock::new();
    *F.get_or_init(|| env_flag("AM_FAST") || par_threads() > 1)
}

/// `AM_PAR=<n>`: number of worker threads for the frontier-parallel subset
/// construction. `1` means the serial path.
///
/// **Default (no `AM_PAR` in the environment): `min(8, cores - 2)`, clamped to at
/// least 1** — the setting `bench/SPEED-ROUND6.md` ("Final defaults") measured at
/// 2.5x-12.6x faster than serial on every hard panel case, at lower peak memory,
/// with zero verdict or state-count disagreements over 1120 + 280 + 100 fuzz-diff
/// pairs.  Two threads are left to the rest of the machine (the Python runner's
/// watchdog, the GUI server); 8 is the cap because the frontier stops widening.
///
/// Known cost, measured and unfixed: on a *closed* sentence whose projections are
/// all small the flat core this flag implies is slower than `dfa.rs`'s reference
/// core — prism-1's `? E i,n. n>=1 & $FE(i,i+n,2*n)` is 0.128 s at `AM_PAR=1` and
/// 0.33 s here.  See `docs/KNOWN-ISSUES.md` §7.
pub fn par_threads() -> usize {
    static F: OnceLock<usize> = OnceLock::new();
    *F.get_or_init(|| match std::env::var("AM_PAR").ok().and_then(|v| v.parse::<usize>().ok()) {
        Some(n) => n.max(1),
        None => default_threads(),
    })
}

/// The default worker count: `min(8, cores - 2)`, at least 1.
fn default_threads() -> usize {
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    cores.saturating_sub(2).clamp(1, 8)
}

/// `AM_LAZY_CLOSED=1`: skip determinization + minimization for the closed
/// sentence produced by projecting away the last variable.
pub fn lazy_closed() -> bool {
    static F: OnceLock<bool> = OnceLock::new();
    *F.get_or_init(|| env_flag("AM_LAZY_CLOSED"))
}
/// `AM_FAST_VERIFY=1`: build every determinization/minimization both ways and
/// assert the two results are identical (development gate; roughly 2x slower).
pub fn verify() -> bool {
    static F: OnceLock<bool> = OnceLock::new();
    *F.get_or_init(|| env_flag("AM_FAST_VERIFY"))
}

/// Words of successor-bitset scratch the transposed inner loop may use before
/// falling back to one symbol at a time (32 MB).
const TRANSPOSE_WORDS: usize = 4 << 20;
/// Words of candidate-subset scratch one parallel block may use (8 MB). The
/// buffer starts small and doubles, so a construction that finishes in a few
/// hundred subsets never pays for the full block.
const PAR_BLOCK_WORDS: usize = 1 << 20;

// ------------------------------------------------------------------ FlatNfa

/// An NFA whose transition relation is one flat `u32` array.
///
/// `arity != 0` means every (state, symbol) has exactly `arity` (not
/// necessarily distinct) successors at `dsts[(s*alpha+a)*arity ..]`; this is
/// the shape an existential projection produces (`arity = k`) and the shape a
/// DFA has (`arity = 1`), and it needs no offset table at all.  `arity == 0`
/// means CSR: `offs` has `nstates*alpha+1` entries into `dsts`.
pub struct FlatNfa {
    pub k: usize,
    pub vars: Vec<String>,
    pub alpha: usize,
    pub nstates: usize,
    pub arity: usize,
    pub offs: Vec<u32>,
    pub dsts: Vec<u32>,
    pub init: Vec<u32>,
    pub accept: Vec<bool>,
}

impl FlatNfa {
    /// Successors of edge slot `e = s*alpha + a`.
    #[inline(always)]
    pub fn succ(&self, e: usize) -> &[u32] {
        if self.arity != 0 {
            let b = e * self.arity;
            unsafe { self.dsts.get_unchecked(b..b + self.arity) }
        } else {
            let (l, r) = (self.offs[e] as usize, self.offs[e + 1] as usize);
            unsafe { self.dsts.get_unchecked(l..r) }
        }
    }

    /// The NFA `Dfa::exists` builds: drop coordinate `pos` from the alphabet,
    /// so symbol `s` of the new alphabet has the `k` old symbols that insert a
    /// digit at `pos` as successors.  Laid out so those `k` targets, and the
    /// whole row of `alpha` symbols for one state, are contiguous.
    pub fn from_exists(d: &Dfa, pos: usize, newvars: Vec<String>, nalpha: usize) -> FlatNfa {
        let k = d.k;
        let lo = k.pow(pos as u32);
        let hi = lo * k;
        let mut dsts = vec![0u32; d.nstates * nalpha * k];
        for st in 0..d.nstates {
            let row = st * d.alpha;
            let orow = st * nalpha * k;
            for s in 0..nalpha {
                let low = s % lo;
                let high = (s / lo) * hi;
                let base = orow + s * k;
                for dg in 0..k {
                    dsts[base + dg] = d.trans[row + low + dg * lo + high];
                }
            }
        }
        FlatNfa { k, vars: newvars, alpha: nalpha, nstates: d.nstates, arity: k,
                  offs: Vec::new(), dsts, init: vec![0], accept: d.accept.clone() }
    }

    /// A DFA seen as an NFA with the given start states (arity 1).
    pub fn from_dfa(d: &Dfa, init: Vec<u32>) -> FlatNfa {
        FlatNfa { k: d.k, vars: d.vars.clone(), alpha: d.alpha, nstates: d.nstates,
                  arity: 1, offs: Vec::new(), dsts: d.trans.clone(), init,
                  accept: d.accept.clone() }
    }

    /// Reverse: flip every arrow, swap initial and accepting sets. Built with a
    /// counting sort into CSR, so no per-edge allocation.
    pub fn reversed(&self) -> Option<FlatNfa> {
        let ne = self.nstates.checked_mul(self.alpha)?;
        let total: usize = if self.arity != 0 { ne * self.arity } else { self.dsts.len() };
        if total > u32::MAX as usize || ne.checked_add(1).is_none() { return None; }
        let mut offs = vec![0u32; ne + 1];
        for s in 0..self.nstates {
            for a in 0..self.alpha {
                for &d in self.succ(s * self.alpha + a) {
                    offs[d as usize * self.alpha + a + 1] += 1;
                }
            }
        }
        for i in 0..ne { offs[i + 1] += offs[i]; }
        let mut fill: Vec<u32> = offs[..ne].to_vec();
        let mut dsts = vec![0u32; total];
        for s in 0..self.nstates {
            for a in 0..self.alpha {
                for &d in self.succ(s * self.alpha + a) {
                    let slot = d as usize * self.alpha + a;
                    dsts[fill[slot] as usize] = s as u32;
                    fill[slot] += 1;
                }
            }
        }
        let init: Vec<u32> = (0..self.nstates).filter(|&s| self.accept[s]).map(|s| s as u32).collect();
        let mut accept = vec![false; self.nstates];
        for &s in &self.init { accept[s as usize] = true; }
        Some(FlatNfa { k: self.k, vars: self.vars.clone(), alpha: self.alpha,
                       nstates: self.nstates, arity: 0, offs, dsts, init, accept })
    }

    /// Is an accepting state reachable from an initial state? This is the whole
    /// answer for a projection that leaves no free variables -- see
    /// [`closed_verdict`].
    pub fn reaches_accept(&self) -> bool {
        let mut seen = vec![false; self.nstates];
        let mut stack: Vec<u32> = Vec::new();
        for &s in &self.init {
            if !seen[s as usize] { seen[s as usize] = true; stack.push(s); }
        }
        for &s in &self.init { if self.accept[s as usize] { return true; } }
        while let Some(s) = stack.pop() {
            let base = s as usize * self.alpha;
            for a in 0..self.alpha {
                for &d in self.succ(base + a) {
                    if !seen[d as usize] {
                        if self.accept[d as usize] { return true; }
                        seen[d as usize] = true;
                        stack.push(d);
                    }
                }
            }
        }
        false
    }
}

// --------------------------------------------------------------- interner

#[inline(always)]
fn mix(h: u64, w: u64) -> u64 {
    (h.rotate_left(5) ^ w).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95)
}
#[inline(always)]
fn hash_bits(bits: &[u64]) -> u64 {
    let mut h = 0u64;
    for &w in bits { h = mix(h, w); }
    // final avalanche: the low bits index the table, and rotate/multiply alone
    // leaves them weak for the very sparse bitsets a subset construction sees
    h ^= h >> 32;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^ (h >> 29)
}

/// A set of fixed-width bitsets stored in one flat arena, indexed by an
/// open-addressing hash table. `id` is the insertion order, so it matches the
/// BFS numbering the old `HashMap`+`Vec` pair produced.
struct Interner {
    words: usize,
    arena: Vec<u64>,
    hashes: Vec<u64>,
    slots: Vec<u32>, // 0 = empty, else id+1
    mask: usize,
    n: usize,
}

impl Interner {
    fn new(words: usize, hint: usize) -> Interner {
        let mut cap = 1024usize;
        while cap < hint * 2 { cap <<= 1; }
        Interner { words, arena: Vec::new(), hashes: Vec::new(),
                   slots: vec![0u32; cap], mask: cap - 1, n: 0 }
    }
    #[inline] fn row(&self, id: usize) -> &[u64] {
        &self.arena[id * self.words..(id + 1) * self.words]
    }
    #[inline] fn find(&self, key: &[u64], h: u64) -> Option<u32> {
        let mut i = h as usize & self.mask;
        loop {
            let s = unsafe { *self.slots.get_unchecked(i) };
            if s == 0 { return None; }
            let id = (s - 1) as usize;
            if self.hashes[id] == h && self.row(id) == key { return Some(id as u32); }
            i = (i + 1) & self.mask;
        }
    }
    fn grow(&mut self) {
        let cap = self.slots.len() * 2;
        let mask = cap - 1;
        let mut slots = vec![0u32; cap];
        for id in 0..self.n {
            let mut i = self.hashes[id] as usize & mask;
            while slots[i] != 0 { i = (i + 1) & mask; }
            slots[i] = (id + 1) as u32;
        }
        self.slots = slots;
        self.mask = mask;
    }
    /// Look up `key`, inserting it with the next id if absent.
    /// Returns (id, inserted).
    #[inline] fn intern(&mut self, key: &[u64], h: u64) -> (u32, bool) {
        if let Some(id) = self.find(key, h) { return (id, false); }
        if (self.n + 1) * 10 >= self.slots.len() * 7 { self.grow(); }
        let id = self.n as u32;
        self.arena.extend_from_slice(key);
        self.hashes.push(h);
        let mut i = h as usize & self.mask;
        while self.slots[i] != 0 { i = (i + 1) & self.mask; }
        self.slots[i] = id + 1;
        self.n += 1;
        (id, true)
    }
}

// ---------------------------------------------------------- determinization

#[inline(always)]
fn or_succ(buf: &mut [u64], succ: &[u32]) {
    for &d in succ {
        let d = d as usize;
        unsafe { *buf.get_unchecked_mut(d >> 6) |= 1u64 << (d & 63) };
    }
}

/// Subset construction on a [`FlatNfa`], capped at `cap` subsets.
///
/// Identical output to `dfa::Nfa::determinize_capped` on the equivalent NFA:
/// same state numbering, same transition table, same accept vector.
pub fn determinize_capped(nfa: &FlatNfa, cap: usize) -> Option<Dfa> {
    let threads = par_threads();
    if threads > 1 { return determinize_par(nfa, cap, threads); }
    let alpha = nfa.alpha;
    let words = (nfa.nstates + 63) / 64;
    let mut it = Interner::new(words, 4096);
    let mut init = vec![0u64; words];
    for &s in &nfa.init { init[s as usize >> 6] |= 1u64 << (s & 63); }
    it.intern(&init, hash_bits(&init));

    let transposed = alpha > 1 && alpha.saturating_mul(words) <= TRANSPOSE_WORDS;
    let mut buf = vec![0u64; if transposed { alpha * words } else { words }];
    let mut cur = vec![0u64; words];
    let mut trans: Vec<u32> = Vec::new();
    let mut i = 0usize;
    let mut next_tick = SUBSET_TICK;
    while i < it.n {
        cur.copy_from_slice(it.row(i));
        if transposed {
            for w in buf.iter_mut() { *w = 0; }
            for wi in 0..words {
                let mut bits = cur[wi];
                while bits != 0 {
                    let s = wi * 64 + bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    let base = s * alpha;
                    for a in 0..alpha {
                        or_succ(&mut buf[a * words..(a + 1) * words], nfa.succ(base + a));
                    }
                }
            }
            for a in 0..alpha {
                let key = &buf[a * words..(a + 1) * words];
                let h = hash_bits(key);
                let id = match it.find(key, h) { Some(id) => id, None => it.intern(key, h).0 };
                trans.push(id);
            }
        } else {
            for a in 0..alpha {
                for w in buf.iter_mut() { *w = 0; }
                for wi in 0..words {
                    let mut bits = cur[wi];
                    while bits != 0 {
                        let s = wi * 64 + bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        or_succ(&mut buf, nfa.succ(s * alpha + a));
                    }
                }
                let h = hash_bits(&buf);
                let id = match it.find(&buf, h) { Some(id) => id, None => it.intern(&buf, h).0 };
                trans.push(id);
            }
        }
        i += 1;
        peak_bump(it.n);
        if it.n >= next_tick { next_tick = it.n + SUBSET_TICK; crate::progress::subsets(it.n); }
        if it.n >= cap { return None; }
    }
    Some(finish(nfa, it, trans))
}

/// Turn the interned subsets into a `Dfa` (accept = subset meets NFA accept).
fn finish(nfa: &FlatNfa, it: Interner, trans: Vec<u32>) -> Dfa {
    let words = it.words;
    let mut acc = vec![0u64; words];
    for s in 0..nfa.nstates { if nfa.accept[s] { acc[s >> 6] |= 1u64 << (s & 63); } }
    let mut accept = Vec::with_capacity(it.n);
    for id in 0..it.n {
        let row = &it.arena[id * words..(id + 1) * words];
        accept.push((0..words).any(|w| row[w] & acc[w] != 0));
    }
    Dfa { k: nfa.k, vars: nfa.vars.clone(), alpha: nfa.alpha, nstates: it.n, trans, accept }
}

/// Frontier-parallel subset construction.
///
/// The frontier is walked in blocks; a block's `(state, symbol)` successor
/// bitsets are computed and probed against the (immutable during the phase)
/// interner in parallel, then the misses are interned serially **in the same
/// order the serial construction would have visited them**.  That keeps the
/// state numbering, and therefore the whole output DFA, identical to the
/// serial path no matter how many threads run.
fn determinize_par(nfa: &FlatNfa, cap: usize, threads: usize) -> Option<Dfa> {
    let alpha = nfa.alpha;
    let words = (nfa.nstates + 63) / 64;
    let mut it = Interner::new(words, 4096);
    let mut init = vec![0u64; words];
    for &s in &nfa.init { init[s as usize >> 6] |= 1u64 << (s & 63); }
    it.intern(&init, hash_bits(&init));

    let per_state = alpha * words;
    let block_max = (PAR_BLOCK_WORDS / per_state.max(1)).clamp(1, 4096);
    let mut block = block_max.min(64);
    let mut out = vec![0u64; block * per_state];
    let mut ids = vec![u32::MAX; block * alpha];
    let mut hs = vec![0u64; block * alpha];
    let mut trans: Vec<u32> = Vec::new();
    let mut i = 0usize;
    let mut next_tick = SUBSET_TICK;

    let pool = pool(threads);
    while i < it.n {
        if block < block_max && it.n - i > block {
            block = (block * 4).min(block_max);
            out.resize(block * per_state, 0);
            ids.resize(block * alpha, u32::MAX);
            hs.resize(block * alpha, 0);
        }
        let end = (i + block).min(it.n);
        let m = end - i;
        {
            let itr = &it;
            let base_id = i;
            pool.install(|| {
                use rayon::prelude::*;
                out[..m * per_state]
                    .par_chunks_mut(per_state)
                    .zip(ids[..m * alpha].par_chunks_mut(alpha))
                    .zip(hs[..m * alpha].par_chunks_mut(alpha))
                    .enumerate()
                    .for_each(|(t, ((obuf, oid), oh))| {
                        for w in obuf.iter_mut() { *w = 0; }
                        let cur = itr.row(base_id + t);
                        for wi in 0..words {
                            let mut bits = cur[wi];
                            while bits != 0 {
                                let s = wi * 64 + bits.trailing_zeros() as usize;
                                bits &= bits - 1;
                                let sb = s * alpha;
                                for a in 0..alpha {
                                    or_succ(&mut obuf[a * words..(a + 1) * words], nfa.succ(sb + a));
                                }
                            }
                        }
                        for a in 0..alpha {
                            let key = &obuf[a * words..(a + 1) * words];
                            let h = hash_bits(key);
                            oh[a] = h;
                            oid[a] = match itr.find(key, h) { Some(id) => id, None => u32::MAX };
                        }
                    });
            });
        }
        for t in 0..m {
            for a in 0..alpha {
                let e = t * alpha + a;
                let id = if ids[e] != u32::MAX { ids[e] } else {
                    let (lo, hi) = ((t * per_state) + a * words, (t * per_state) + (a + 1) * words);
                    it.intern(&out[lo..hi], hs[e]).0
                };
                trans.push(id);
            }
            peak_bump(it.n);
            if it.n >= next_tick { next_tick = it.n + SUBSET_TICK; crate::progress::subsets(it.n); }
            if it.n >= cap { return None; }
        }
        i = end;
    }
    Some(finish(nfa, it, trans))
}

fn pool(threads: usize) -> &'static rayon::ThreadPool {
    static P: OnceLock<rayon::ThreadPool> = OnceLock::new();
    P.get_or_init(|| rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap())
}

// ------------------------------------------------------------ minimization

/// Trim + Moore partition refinement, with the refinement driven by an LSD
/// radix sort over `(color, color of each successor)` instead of a `HashMap`
/// keyed by an owned `Vec<u32>`. Same partition, same numbering, same output as
/// `Dfa::minimize`; no allocation inside the refinement loop.
pub fn minimize(d: &Dfa) -> Dfa {
    if d.nstates >= SUBSET_TICK { crate::progress::phase("minimize", ""); }
    let alpha = d.alpha;
    // --- trim (BFS from state 0, symbols ascending) ---
    let mut map = vec![u32::MAX; d.nstates];
    let mut order: Vec<u32> = Vec::with_capacity(d.nstates);
    map[0] = 0;
    order.push(0);
    let mut i = 0;
    while i < order.len() {
        let s = order[i] as usize;
        for a in 0..alpha {
            let t = d.trans[s * alpha + a] as usize;
            if map[t] == u32::MAX { map[t] = order.len() as u32; order.push(t as u32); }
        }
        i += 1;
    }
    let n = order.len();
    let mut trans: Vec<u32> = Vec::with_capacity(n * alpha);
    for &s in &order {
        let row = s as usize * alpha;
        for a in 0..alpha { trans.push(map[d.trans[row + a] as usize]); }
    }
    let accept: Vec<bool> = order.iter().map(|&s| d.accept[s as usize]).collect();

    // --- Moore refinement by radix sort ---
    let mut color: Vec<u32> = accept.iter().map(|&b| b as u32).collect();
    let mut ncolors: u32 = if accept.iter().any(|&b| b) && accept.iter().any(|&b| !b) { 2 } else { 1 };
    if ncolors == 1 {
        // one class: normalise the colours (they are the accept bits) to 0, which is
        // what the reference minimizer's first refinement round would have done.
        for c in color.iter_mut() { *c = 0; }
    } else {
        let mut perm: Vec<u32> = (0..n as u32).collect();
        let mut tmp: Vec<u32> = vec![0; n];
        let mut counts: Vec<u32> = Vec::new();
        let mut newcolor: Vec<u32> = vec![0; n];
        loop {
            // LSD radix: least significant key first = successor under alpha-1,
            // ... , successor under 0, then the state's own colour.
            for (j, x) in perm.iter_mut().enumerate() { *x = j as u32; }
            let nc = ncolors as usize;
            counts.clear();
            counts.resize(nc + 1, 0);
            for pass in 0..=alpha {
                let keyof = |s: usize| -> usize {
                    if pass == alpha { color[s] as usize }
                    else { color[trans[s * alpha + (alpha - 1 - pass)] as usize] as usize }
                };
                for c in counts.iter_mut() { *c = 0; }
                for &s in perm.iter() { counts[keyof(s as usize) + 1] += 1; }
                for j in 0..nc { counts[j + 1] += counts[j]; }
                for &s in perm.iter() {
                    let key = keyof(s as usize);
                    tmp[counts[key] as usize] = s;
                    counts[key] += 1;
                }
                std::mem::swap(&mut perm, &mut tmp);
            }
            // assign new colours by scanning the sorted order
            let same = |x: usize, y: usize| -> bool {
                if color[x] != color[y] { return false; }
                let (rx, ry) = (x * alpha, y * alpha);
                for a in 0..alpha {
                    if color[trans[rx + a] as usize] != color[trans[ry + a] as usize] { return false; }
                }
                true
            };
            let mut c: u32 = 0;
            newcolor[perm[0] as usize] = 0;
            for j in 1..n {
                let (p, q) = (perm[j - 1] as usize, perm[j] as usize);
                if !same(p, q) { c += 1; }
                newcolor[q] = c;
            }
            let nnew = c + 1;
            color.copy_from_slice(&newcolor);
            if nnew == ncolors { break; }
            ncolors = nnew;
        }
    }
    // --- rebuild, start state relabelled to 0, classes in first-appearance order ---
    let m = ncolors as usize;
    let mut relabel = vec![u32::MAX; m];
    relabel[color[0] as usize] = 0;
    let mut cnt = 1u32;
    for s in 0..n {
        let c = color[s] as usize;
        if relabel[c] == u32::MAX { relabel[c] = cnt; cnt += 1; }
    }
    let mut ntrans = vec![0u32; m * alpha];
    let mut naccept = vec![false; m];
    for s in 0..n {
        let c = relabel[color[s] as usize] as usize;
        naccept[c] = accept[s];
        for a in 0..alpha {
            ntrans[c * alpha + a] = relabel[color[trans[s * alpha + a] as usize] as usize];
        }
    }
    Dfa { k: d.k, vars: d.vars.clone(), alpha, nstates: m, trans: ntrans, accept: naccept }
}

// ------------------------------------------------------------- lazy closed

/// The verdict of a closed sentence `E var. phi`, where projecting `var` away
/// leaves no free variables.
///
/// The projected NFA is over a one-letter alphabet, so the value the engine
/// eventually reads -- `det.zero_closure().accept[0]` -- is
/// "some `0^n`, `n >= 0`, is accepted", which is plain reachability from the
/// initial to an accepting state.  No subset construction, no zero closure, no
/// minimization; the 1-state answer automaton is exactly what `minimize` would
/// have returned, because a zero-closed language over one letter is either
/// empty or all of `0*`.
pub fn closed_verdict(nfa: &FlatNfa) -> Dfa {
    let v = nfa.reaches_accept();
    Dfa { k: nfa.k, vars: Vec::new(), alpha: 1, nstates: 1, trans: vec![0], accept: vec![v] }
}

// ------------------------------------------------------------------ checks

/// Assert two DFAs are equal element by element (used by `AM_FAST_VERIFY=1`).
pub fn assert_same(what: &str, a: &Dfa, b: &Dfa) {
    assert_eq!(a.nstates, b.nstates, "{}: nstates {} vs {}", what, a.nstates, b.nstates);
    assert_eq!(a.alpha, b.alpha, "{}: alpha", what);
    assert_eq!(a.vars, b.vars, "{}: vars", what);
    assert_eq!(a.accept, b.accept, "{}: accept", what);
    assert_eq!(a.trans, b.trans, "{}: trans", what);
}

/// Bridge for the old `dfa::Nfa` (used only by `AM_FAST_VERIFY`): build the
/// flat form of an arbitrary `Vec<Vec<State>>` NFA.
pub fn flatten(k: usize, vars: Vec<String>, alpha: usize, nstates: usize,
               trans: &[Vec<State>], init: Vec<u32>, accept: Vec<bool>) -> Option<FlatNfa> {
    let ne = nstates.checked_mul(alpha)?;
    let total: usize = trans.iter().map(|v| v.len()).sum();
    if total > u32::MAX as usize { return None; }
    let mut offs = vec![0u32; ne + 1];
    for e in 0..ne { offs[e + 1] = offs[e] + trans[e].len() as u32; }
    let mut dsts = Vec::with_capacity(total);
    for e in 0..ne { dsts.extend_from_slice(&trans[e]); }
    Some(FlatNfa { k, vars, alpha, nstates, arity: 0, offs, dsts, init, accept })
}
