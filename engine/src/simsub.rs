//! Simulation-based subsumption in the subset construction (`AM_SIMSUB=1`,
//! **default off**).
//!
//! ## Credit
//!
//! The idea that a subset may be pruned to its simulation-maximal states is not
//! ours.  It is the "antichain" line of work:
//!
//! * M. De Wulf, L. Doyen, T. A. Henzinger, J.-F. Raskin, *Antichains: a new
//!   algorithm for checking universality of finite automata*, CAV 2006 -- subset
//!   inclusion as a subsumption order on the sets a subset construction explores.
//! * P. A. Abdulla, Y.-F. Chen, L. Holik, R. Mayr, T. Vojnar, *When simulation
//!   meets antichains (on checking language inclusion of nondeterministic finite
//!   (tree) automata)*, TACAS 2010 -- refining that order by a forward simulation
//!   preorder, which is exactly the reduction implemented here.
//! * John Nicol's on-the-fly determinization library
//!   (<https://github.com/jn1z/OTF>), shipped in Walnut 7.0+ as the `CCLS` and
//!   `BRZ-CCLS` strategies ("Convexity Closure Lattice **with** simulation").
//!   `bench/WALNUT-STRATEGIES.md` records that `CCLS` answers the `tail-c` `FE`
//!   query in 10.6 s where Peanut's direct construction needs 191 s; that gap is
//!   what this module exists to close, and the technique is Nicol's, used here
//!   with attribution.
//!
//! ## What it does
//!
//! Let `N = (Q, Sigma, delta, I, F)` be the NFA an existential projection
//! produces (`det_par::FlatNfa::from_exists`).  Write `q >= p` ("`q` simulates
//! `p`") for the greatest relation such that
//!
//! * `p in F` implies `q in F` (accepting-compatible), and
//! * for every `a` and every `p' in delta(p,a)` there is `q' in delta(q,a)` with
//!   `q' >= p'`.
//!
//! Simulation implies language containment: `q >= p` gives `L(p) subset L(q)`.
//! Hence for any set `S subset Q`, writing `L(S) = union_{s in S} L(s)`, dropping
//! from `S` a state that some *other retained* state of `S` simulates leaves
//! `L(S)` unchanged.  [`reduce`] does exactly that: it keeps the `>=`-maximal
//! elements of `S`, and of each `>=`-equivalence class present in `S` keeps the
//! member of least index (so the result is canonical and never empty).
//!
//! The subset construction then runs on reduced sets:
//!
//! ```text
//!     S0    = reduce(I)
//!     d(S,a) = reduce( union_{s in S} delta(s,a) )
//!     S in F' iff S meets F
//! ```
//!
//! ## Why the minimal DFA is unchanged (correctness argument)
//!
//! **Claim.** For every reduced set `S`, the reduced construction accepts from
//! `S` exactly `L(S)`.
//!
//! *Proof* by induction on the length of `w`.
//! For `w = eps`: the construction accepts at `S` iff `S` meets `F`, and
//! `eps in L(S)` iff `S` meets `F`.  Reduction cannot destroy this: if `x in S`
//! is dropped, some retained `z in S` has `z >= x`, and `x in F` implies
//! `z in F`, so `S` meets `F` before reduction iff it does after.
//! For `w = a u`: the construction accepts `a u` at `S` iff it accepts `u` at
//! `d(S,a) = reduce(union delta(s,a))`, which by the induction hypothesis is
//! `u in L(reduce(union delta(s,a)))`.  Reduction preserves the union of the
//! languages of a set's members (each dropped `x` has a retained `z >= x`, and
//! `L(x) subset L(z)`), so that is `u in L(union_s delta(s,a))`, i.e.
//! `a u in L(S)`.  QED.
//!
//! With `S0 = reduce(I)` the constructed DFA therefore accepts `L(I) = L(N)` --
//! the *same language* as the ordinary subset construction.  It is a different
//! DFA in general (different, usually far fewer, reachable states; different
//! numbering), so unlike `det_par` this path is **not** state-for-state identical
//! to the default one.  What is identical is what the engine reports: the answer
//! of `Dfa::exists` is `det.zero_closure().minimize()`, both of those operations
//! are language-determined, and the minimal DFA of a language is unique up to
//! isomorphism -- so the minimal state count, the accepted language and every
//! downstream verdict are unchanged.  `peak=` (the largest intermediate
//! automaton) is deliberately *not* preserved: shrinking it is the point.
//!
//! Termination and non-emptiness: `reduce` never empties a non-empty set (the
//! least-index member of a maximal equivalence class is always kept), and there
//! are finitely many subsets, so the construction terminates exactly as the
//! ordinary one does.
//!
//! ## Ladder placement and flags
//!
//! `Dfa::exists` calls [`exists`] first when `AM_SIMSUB` is set; it returns
//! `None` -- and the caller then runs the ordinary ladder unchanged -- whenever
//! this module declines or every one of its rungs exceeds its cap, so it can
//! only ever answer or abstain.
//!
//! [`exists`] runs the same four-rung ladder `Dfa::exists_fast` does --
//! forward(`AM_CAP0`), Brzozowski(small), forward(`AM_CAP`), Brzozowski(big) --
//! with every subset construction replaced by [`det_stage`]: trim, simulation,
//! reduced construction.  A stage whose preorder turns out to be trivial (no
//! state dominated by another) or whose NFA is above `AM_SIMSUB_MAX` runs the
//! ordinary flat core instead, so the reduction is never paid for where it
//! cannot pay back.
//!
//! * `AM_SIMSUB=1`      eager: the reduced construction is the first rung.
//! * `AM_SIMSUB=lazy`   lazy: try the ordinary forward construction with
//!                      `AM_CAP0` first, and only pay for the simulation if that
//!                      rung misses.
//! * `AM_SIMSUB=0`/unset  off (default).
//! * `AM_SIMSUB_MAX`    (default 20_000) NFA state ceiling; above it a stage
//!                      falls back to the ordinary flat determinizer.
//! * `AM_SIMSUB_WORK`   (default 400_000_000) simulation refinement budget in
//!                      elementary checks; on overrun, abstain.
//! * `AM_SIMSUB_CAP`    (default = `AM_CAP`, 3_000_000) reduced-subset ceiling.
//! * `AM_SIMSUB_DEBUG`  one line per projection and per stage on stderr.
//! * `AM_FAST_VERIFY=1` also checks this module: every projection it answers is
//!                      compared, after `zero_closure().minimize()`, against the
//!                      ordinary subset construction of the same NFA.  (Equality
//!                      of the *minimized* automata, not of the intermediates --
//!                      that is the whole claim, see above.)
//!
//! ## Measured outcome (see `bench/SIMSUB-RESULTS.md`)
//!
//! Answer-identical (0 disagreements over 228 pairs x 4 configurations, 0
//! `AM_FAST_VERIFY` assertion failures), and it does **not** win the case it was
//! built for: `tail-c` is 192.068 s against the default's 190.359 s, same 1382
//! states, same 8 207 234 peak.  `tail-c`'s hard projection has 298 dominating
//! pairs over 578 trimmed states and the reversed automaton the Brzozowski rung
//! actually answers on has none at all, so there is nothing to subsume where it
//! matters.  Peak intermediate size does fall on 8 of 16 panel rows (up to
//! 20.3x) and 3 rows get faster, but 5 rows and Tribonacci (8.8x) regress,
//! because the reduced construction is serial where `det_par` is
//! frontier-parallel.  The flag stays off by default.

use crate::dfa::{Dfa, SUBSET_TICK, peak_bump};
use crate::det_par::FlatNfa;
use std::sync::OnceLock;

// ------------------------------------------------------------------ flags

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode { Off, Eager, Lazy }

fn mode() -> Mode {
    static M: OnceLock<Mode> = OnceLock::new();
    *M.get_or_init(|| match std::env::var("AM_SIMSUB").ok().as_deref() {
        None | Some("") | Some("0") | Some("off") => Mode::Off,
        Some("lazy") | Some("2") => Mode::Lazy,
        _ => Mode::Eager,
    })
}

/// Is simulation-based subsumption enabled at all? (`AM_SIMSUB=1|lazy`)
pub fn enabled() -> bool { mode() != Mode::Off }

fn envnum(k: &str, d: usize) -> usize {
    std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}
fn dbg_on() -> bool { std::env::var("AM_SIMSUB_DEBUG").is_ok() }

// ------------------------------------------------------------------ bitsets

#[inline(always)] fn words(n: usize) -> usize { (n + 63) / 64 }
#[inline(always)] fn bs_get(b: &[u64], i: usize) -> bool { b[i >> 6] >> (i & 63) & 1 == 1 }
#[inline(always)] fn bs_set(b: &mut [u64], i: usize) { b[i >> 6] |= 1u64 << (i & 63) }
#[inline(always)] fn bs_clear(b: &mut [u64], i: usize) { b[i >> 6] &= !(1u64 << (i & 63)) }
#[inline(always)] fn bs_meets(a: &[u64], b: &[u64]) -> bool {
    a.iter().zip(b.iter()).any(|(x, y)| x & y != 0)
}

// --------------------------------------------------------------- simulation

/// The greatest forward simulation on `nfa`, as `nfa.nstates` bitset rows:
/// bit `t` of row `s` means "`t` simulates `s`", hence `L(s) subset L(t)`.
///
/// Naive greatest-fixpoint refinement (start from the accepting-compatible
/// relation, delete a pair as soon as one move of `s` cannot be matched), with
/// pairs already deleted never re-examined.  `budget` counts elementary
/// (pair, symbol) checks; `None` means the budget ran out and the caller must
/// abstain.
fn simulation(nfa: &FlatNfa, budget: u64) -> Option<Vec<u64>> {
    let q = nfa.nstates;
    let w = words(q);
    let alpha = nfa.alpha;
    let mut sim = vec![0u64; q * w];
    for s in 0..q {
        let row = &mut sim[s * w..(s + 1) * w];
        if nfa.accept[s] {
            for t in 0..q { if nfa.accept[t] { bs_set(row, t) } }
        } else {
            for t in 0..q { bs_set(row, t) }
        }
    }
    let mut spent: u64 = 0;
    loop {
        let mut changed = false;
        for s in 0..q {
            for wi in 0..w {
                let mut bits = sim[s * w + wi];
                while bits != 0 {
                    let t = wi * 64 + bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    let mut ok = true;
                    'a: for a in 0..alpha {
                        spent += 1;
                        let sa = nfa.succ(s * alpha + a);
                        let ta = nfa.succ(t * alpha + a);
                        for &s2 in sa {
                            let row = &sim[s2 as usize * w..(s2 as usize + 1) * w];
                            if !ta.iter().any(|&t2| bs_get(row, t2 as usize)) { ok = false; break 'a; }
                        }
                    }
                    if !ok { bs_clear(&mut sim[s * w..(s + 1) * w], t); changed = true; }
                }
            }
            if spent > budget { return None; }
        }
        if !changed { break; }
    }
    Some(sim)
}

/// The per-state removal masks derived from a simulation preorder.
///
/// `mask[x]` is the set of `y != x` whose presence in a subset licenses deleting
/// `x`: either `y` strictly dominates `x` (`x <= y` and `y </= x`), or `y` is
/// simulation-equivalent to `x` and has the smaller index.  `cand` is the set of
/// `x` with a non-empty mask -- the only states `reduce` has to look at.
struct Prune { w: usize, mask: Vec<u64>, cand: Vec<u64> }

fn prunes(sim: &[u64], q: usize) -> Option<Prune> {
    let w = words(q);
    let mut mask = vec![0u64; q * w];
    let mut cand = vec![0u64; w];
    let mut any = false;
    for x in 0..q {
        let mut nonempty = false;
        for wi in 0..w {
            let mut bits = sim[x * w + wi];
            while bits != 0 {
                let y = wi * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if y == x { continue; }
                // y simulates x; keep the pair if y is strictly above x, or if
                // the two are equivalent and y is the canonical (smaller) index.
                let back = bs_get(&sim[y * w..(y + 1) * w], x);
                if !back || y < x {
                    bs_set(&mut mask[x * w..(x + 1) * w], y);
                    nonempty = true;
                }
            }
        }
        if nonempty { bs_set(&mut cand, x); any = true; }
    }
    if !any { return None; }           // preorder is trivial: nothing to gain
    if dbg_on() {
        let pairs: u64 = mask.iter().map(|x| x.count_ones() as u64).sum();
        let nc: u64 = cand.iter().map(|x| x.count_ones() as u64).sum();
        eprintln!("      [simsub] preorder: {} prunable states of {}, {} dominating pairs", nc, q, pairs);
    }
    Some(Prune { w, mask, cand })
}

impl Prune {
    /// Prune `set` in place to its simulation-maximal states (least index per
    /// equivalence class).  `orig` must be the unpruned copy of `set`: the test
    /// is made against the original members, which keeps the result independent
    /// of iteration order and (see the module header) non-empty.
    #[inline]
    fn reduce(&self, set: &mut [u64], orig: &[u64]) {
        let w = self.w;
        for wi in 0..w {
            let mut bits = orig[wi] & self.cand[wi];
            while bits != 0 {
                let x = wi * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if bs_meets(&self.mask[x * w..(x + 1) * w], orig) { bs_clear(set, x) }
            }
        }
    }
}

// ------------------------------------------------------------------- trim

/// Keep only the states that are both reachable from `init` and co-reachable to
/// an accepting state, and drop every edge that left the surviving set.
///
/// This is not an optimisation of the subset construction (the states it removes
/// are exactly the ones whose presence in a subset changes nothing); it is what
/// makes the *simulation* worth computing.  `FlatNfa::from_exists` produces a
/// **complete** NFA -- every (state, symbol) has `k` successors, several of them
/// the projected DFA's sink -- and on a complete automaton `q` can only simulate
/// `p` by matching every move of `p`, including its moves into dead ends.  After
/// trimming the automaton is partial, a missing move is matched vacuously, and
/// the preorder gets sharply coarser.  Walnut's OTF library does the same thing
/// (its `OTF.NFATrim`) before running `CCLS`.
///
/// Returns the trimmed NFA in CSR form, or `None` if nothing was removed (then
/// the caller may as well use the original) or if the language is empty.
fn trim(nfa: &FlatNfa) -> Option<FlatNfa> {
    let q = nfa.nstates;
    let alpha = nfa.alpha;
    let ne = q.checked_mul(alpha)?;
    let total: usize = if nfa.arity != 0 { ne.checked_mul(nfa.arity)? } else { nfa.dsts.len() };
    if total > u32::MAX as usize { return None }     // CSR offsets are u32
    // backward reachability from the accepting states, over a CSR predecessor
    // index built by counting sort (no per-state Vec: this runs on automata with
    // millions of states)
    let mut roff = vec![0u32; q + 1];
    for s in 0..q {
        for a in 0..alpha { for &d in nfa.succ(s * alpha + a) { roff[d as usize + 1] += 1 } }
    }
    for i in 0..q { roff[i + 1] += roff[i] }
    let mut fill = roff[..q].to_vec();
    let mut rsrc = vec![0u32; roff[q] as usize];
    for s in 0..q {
        for a in 0..alpha {
            for &d in nfa.succ(s * alpha + a) {
                rsrc[fill[d as usize] as usize] = s as u32;
                fill[d as usize] += 1;
            }
        }
    }
    let mut co = vec![false; q];
    let mut stack: Vec<u32> = Vec::new();
    for s in 0..q { if nfa.accept[s] { co[s] = true; stack.push(s as u32) } }
    while let Some(s) = stack.pop() {
        for i in roff[s as usize]..roff[s as usize + 1] {
            let p = rsrc[i as usize];
            if !co[p as usize] { co[p as usize] = true; stack.push(p) }
        }
    }
    // forward reachability from the initial states, staying inside `co`
    let mut live = vec![false; q];
    for &s in &nfa.init { if co[s as usize] && !live[s as usize] { live[s as usize] = true; stack.push(s) } }
    while let Some(s) = stack.pop() {
        let base = s as usize * alpha;
        for a in 0..alpha {
            for &d in nfa.succ(base + a) {
                if co[d as usize] && !live[d as usize] { live[d as usize] = true; stack.push(d) }
            }
        }
    }
    let nlive = live.iter().filter(|&&b| b).count();
    if nlive == 0 || nlive == q { return None }
    let mut map = vec![u32::MAX; q];
    let mut n = 0u32;
    for s in 0..q { if live[s] { map[s] = n; n += 1 } }
    let nn = n as usize;
    let mut offs: Vec<u32> = Vec::with_capacity(nn * alpha + 1);
    let mut dsts: Vec<u32> = Vec::new();
    offs.push(0);
    let mut seen: Vec<u32> = Vec::with_capacity(8);
    for s in 0..q {
        if !live[s] { continue }
        for a in 0..alpha {
            seen.clear();
            for &d in nfa.succ(s * alpha + a) {
                let m = map[d as usize];
                if m != u32::MAX && !seen.contains(&m) { seen.push(m) }
            }
            seen.sort_unstable();
            dsts.extend_from_slice(&seen);
            offs.push(dsts.len() as u32);
        }
    }
    let init: Vec<u32> = nfa.init.iter().filter(|&&s| live[s as usize]).map(|&s| map[s as usize]).collect();
    let accept: Vec<bool> = (0..q).filter(|&s| live[s]).map(|s| nfa.accept[s]).collect();
    Some(FlatNfa { k: nfa.k, vars: nfa.vars.clone(), alpha, nstates: nn, arity: 0,
                   offs, dsts, init, accept })
}

// --------------------------------------------------------------- interner

/// Flat arena of fixed-width bitsets with an open-addressing index; ids are
/// insertion order.  (A private copy of `det_par`'s interner: this module owns
/// its own file and does not widen that one's interface.)
struct Interner { words: usize, arena: Vec<u64>, hashes: Vec<u64>, slots: Vec<u32>, mask: usize, n: usize }

#[inline(always)] fn hash_bits(bits: &[u64]) -> u64 {
    let mut h = 0u64;
    for &x in bits { h = (h.rotate_left(5) ^ x).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95); }
    h ^= h >> 32;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^ (h >> 29)
}

impl Interner {
    fn new(words: usize) -> Interner {
        Interner { words, arena: Vec::new(), hashes: Vec::new(), slots: vec![0u32; 1024], mask: 1023, n: 0 }
    }
    #[inline] fn row(&self, id: usize) -> &[u64] { &self.arena[id * self.words..(id + 1) * self.words] }
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
    #[inline] fn intern(&mut self, key: &[u64], h: u64) -> u32 {
        let mut i = h as usize & self.mask;
        loop {
            let s = unsafe { *self.slots.get_unchecked(i) };
            if s == 0 { break }
            let id = (s - 1) as usize;
            if self.hashes[id] == h && self.row(id) == key { return id as u32 }
            i = (i + 1) & self.mask;
        }
        if (self.n + 1) * 10 >= self.slots.len() * 7 {
            self.grow();
            i = h as usize & self.mask;
            while self.slots[i] != 0 { i = (i + 1) & self.mask; }
        }
        let id = self.n as u32;
        self.arena.extend_from_slice(key);
        self.hashes.push(h);
        self.slots[i] = id + 1;
        self.n += 1;
        id
    }
}

// ------------------------------------------------- reduced subset construction

/// Subset construction on `nfa` with every discovered set pruned by [`Prune::reduce`],
/// capped at `cap` reduced subsets (`None` on overrun).
fn determinize_sim(nfa: &FlatNfa, dr: &Prune, cap: usize) -> Option<Dfa> {
    let alpha = nfa.alpha;
    let w = words(nfa.nstates);
    let mut it = Interner::new(w);
    let mut init = vec![0u64; w];
    for &s in &nfa.init { bs_set(&mut init, s as usize) }
    let orig = init.clone();
    dr.reduce(&mut init, &orig);
    it.intern(&init, hash_bits(&init));

    let mut buf = vec![0u64; alpha * w];
    let mut red = vec![0u64; w];
    let mut cur = vec![0u64; w];
    let mut trans: Vec<u32> = Vec::new();
    let mut i = 0usize;
    let mut next_tick = SUBSET_TICK;
    while i < it.n {
        cur.copy_from_slice(it.row(i));
        for x in buf.iter_mut() { *x = 0 }
        for wi in 0..w {
            let mut bits = cur[wi];
            while bits != 0 {
                let s = wi * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                let base = s * alpha;
                for a in 0..alpha {
                    let dst = &mut buf[a * w..(a + 1) * w];
                    for &d in nfa.succ(base + a) {
                        let d = d as usize;
                        unsafe { *dst.get_unchecked_mut(d >> 6) |= 1u64 << (d & 63) };
                    }
                }
            }
        }
        for a in 0..alpha {
            let key = &buf[a * w..(a + 1) * w];
            red.copy_from_slice(key);
            dr.reduce(&mut red, key);
            let h = hash_bits(&red);
            trans.push(it.intern(&red, h));
        }
        i += 1;
        peak_bump(it.n);
        if it.n >= next_tick { next_tick = it.n + SUBSET_TICK; crate::progress::subsets(it.n) }
        if it.n >= cap { return None }
    }
    let mut acc = vec![0u64; w];
    for s in 0..nfa.nstates { if nfa.accept[s] { bs_set(&mut acc, s) } }
    let accept: Vec<bool> = (0..it.n).map(|id| bs_meets(it.row(id), &acc)).collect();
    Some(Dfa { k: nfa.k, vars: nfa.vars.clone(), alpha, nstates: it.n, trans, accept })
}

// ------------------------------------------------------------------ entry

/// One determinization stage: trim, compute the simulation, and run the reduced
/// subset construction -- falling back to the ordinary flat core for this stage
/// when the preorder turns out to be trivial (nothing to prune, so nothing to
/// gain and a per-subset cost to pay).  `None` = cap or simulation budget blown.
fn det_stage(n: &FlatNfa, cap: usize, tag: &str) -> Option<Dfa> {
    let dbg = dbg_on();
    let t0 = std::time::Instant::now();
    // The simulation is a |Q|^2 bit matrix and a |Q|^2-pair fixpoint: above the
    // ceiling it costs more than the construction it is meant to save (Walnut's
    // own help text says the same about CCLS above ~50 000 NFA states).
    if n.nstates > envnum("AM_SIMSUB_MAX", 20_000) {
        let d = crate::det_par::determinize_capped(n, cap)?;
        if dbg { eprintln!("      [simsub] {}: {} states, above AM_SIMSUB_MAX -> plain det {}",
                           tag, n.nstates, d.nstates) }
        return Some(d);
    }
    let trimmed = trim(n);
    let nn = trimmed.as_ref().unwrap_or(n);
    let sim = simulation(nn, envnum("AM_SIMSUB_WORK", 400_000_000) as u64);
    let dr = sim.as_ref().and_then(|sm| prunes(sm, nn.nstates));
    let tsim = t0.elapsed().as_secs_f64();
    match dr {
        Some(dr) => {
            let d = determinize_sim(nn, &dr, cap)?;
            if dbg { eprintln!("      [simsub] {}: {} states (trim {}) x {} -> sim {:.3}s -> det {}",
                               tag, n.nstates, nn.nstates, n.alpha, tsim, d.nstates) }
            Some(d)
        }
        None => {
            let d = crate::det_par::determinize_capped(nn, cap)?;
            if dbg { eprintln!("      [simsub] {}: {} states (trim {}) x {} -> no usable preorder ({:.3}s) -> plain det {}",
                               tag, n.nstates, nn.nstates, n.alpha, tsim, d.nstates) }
            Some(d)
        }
    }
}

/// Brzozowski with simulation subsumption in both subset constructions
/// (Walnut calls the same combination `BRZ-CCLS`).  Reversal is language-exact
/// for a DFA and each stage is language-exact by the module header's claim, so
/// the composite still recognises `L(nfa)`.
fn brz_stage(nfa: &FlatNfa, cap: usize) -> Option<Dfa> {
    let r = nfa.reversed()?;
    let d1 = det_stage(&r, cap, "brz/rev")?;
    let f2 = FlatNfa::from_dfa(&d1, vec![0]).reversed()?;
    det_stage(&f2, cap, "brz/fwd")
}

/// Existential projection of `var` out of `d` via the simulation-subsumed subset
/// construction.  `None` = "declined": the caller must run its ordinary ladder.
///
/// Declines on: a variable that is not there; a projection that leaves no free
/// variable (the closed case, already handled better by `antichain`/`AM_LAZY_CLOSED`);
/// an NFA above `AM_SIMSUB_MAX`; and a ladder every rung of which exceeded its cap.
///
/// The ladder is the same shape as `Dfa::exists_fast`'s -- forward(`AM_CAP0`),
/// Brzozowski(small), forward(`AM_CAP`), Brzozowski(big) -- with every subset
/// construction replaced by [`det_stage`].
pub fn exists(d: &Dfa, var: &str) -> Option<Dfa> {
    let m = mode();
    if m == Mode::Off { return None }
    let pos = d.vars.iter().position(|v| v == var)?;
    if d.vars.len() <= 1 { return None }
    let mut newvars = d.vars.clone();
    newvars.remove(pos);
    let nalpha = d.alpha / d.k;
    let nfa = FlatNfa::from_exists(d, pos, newvars, nalpha);
    let cap0 = envnum("AM_CAP0", 50_000);
    let cap = envnum("AM_SIMSUB_CAP", envnum("AM_CAP", 3_000_000));
    let dbg = dbg_on();

    if m == Mode::Lazy {
        crate::progress::phase("forward", var);
        if let Some(det) = crate::det_par::determinize_capped(&nfa, cap0) {
            let res = det.zero_closure().minimize();
            crate::progress::states(res.nstates, var);
            if dbg { eprintln!("    simsub({}): forward({}) ok, det {} -> min {}", var, cap0, det.nstates, res.nstates) }
            return Some(res);
        }
    }
    if nfa.nstates > envnum("AM_SIMSUB_MAX", 20_000) {
        if dbg { eprintln!("    simsub({}): declined, nfa {} states", var, nfa.nstates) }
        return None;
    }
    crate::progress::phase("forward-sim", var);
    let det = if let Some(x) = det_stage(&nfa, cap0, "fwd") { x }
        else if let Some(x) = { crate::progress::phase("brzozowski-sim", var);
                                brz_stage(&nfa, cap0.saturating_mul(4).max(200_000)) } { x }
        else if let Some(x) = { crate::progress::phase("forward-sim", var);
                                det_stage(&nfa, cap, "fwd/big") } { x }
        else if let Some(x) = { crate::progress::phase("brzozowski-sim", var);
                                brz_stage(&nfa, cap.saturating_mul(4).max(8_000_000)) } { x }
        else {
            if dbg { eprintln!("    simsub({}): every rung exceeded its cap, declining", var) }
            return None;
        };
    let res = det.zero_closure().minimize();
    // AM_FAST_VERIFY=1: the reduced construction is only claimed to agree with the
    // ordinary one *after* minimization, so that is what is asserted -- the plain
    // subset construction of the same projected NFA, zero-closed and minimized, must
    // be equal to `res` element by element.  (Skipped when the plain construction
    // would itself blow past `AM_CAP`, which is exactly when there is nothing to
    // compare against.)
    if crate::det_par::verify() {
        if let Some(plain) = crate::det_par::determinize_capped(&nfa, cap) {
            crate::det_par::assert_same("simsub/exists", &plain.zero_closure().minimize(), &res);
        }
    }
    crate::progress::states(res.nstates, var);
    if dbg {
        eprintln!("    simsub({}): nfa {} states x alpha {} -> det {} -> min {}",
                  var, nfa.nstates, nalpha, det.nstates, res.nstates);
    }
    Some(res)
}
