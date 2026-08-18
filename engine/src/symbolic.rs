//! Symbolic (MONA-style) existential projection and determinization.
//!
//! The explicit engine (`dfa.rs`) determinizes by enumerating, for every subset and
//! every one of the `k^tracks` letters, the union of the members' targets.  The cost
//! of one subset is therefore `alpha * |subset| * k` set operations, and `alpha` is
//! exponential in the number of tracks: a five-variable formula over base 4 already
//! has 1024 letters, most of which behave identically.
//!
//! MONA's answer is to keep the transition function of every state as a decision
//! diagram over the *bits of the letter*, so letters that behave the same share
//! structure and are handled once.  This module is the same idea specialised to this
//! engine's alphabet, which is a tuple of base-`k` digits rather than bits: the
//! diagram is a reduced, hash-consed **multi-terminal k-ary decision diagram**
//! (`Dd`), one level per track, `k` children per node, terminals carrying an interned
//! set of source states (`Sets`).  Then
//!
//! * **projection** of one track is done while the diagram is built (each new letter's
//!   terminal is the union over the `k` digits of the projected track);
//! * **subset construction** computes a subset's whole transition function as one
//!   union of diagrams -- cost proportional to the number of *distinct behaviours*
//!   (leaves) rather than to `alpha` -- with the union memoised on node pairs, so
//!   subsets that share structure share work;
//! * **minimisation** is Moore refinement done on the diagrams: recolouring the
//!   terminals of a state's diagram and hash-consing the result yields a canonical
//!   signature for the whole row in one memoised walk, again independent of `alpha`;
//! * the result is expanded back to an explicit, minimal [`Dfa`] at the very end, when
//!   it is small.
//!
//! Nothing here is on by default.  `AM_STRATEGY=bdd` makes [`exists`] the first thing
//! `Dfa::exists` tries (falling back to the explicit ladder if a cap is hit);
//! `AM_STRATEGY=auto` tries it only when the alphabet is big enough to pay for it.
//!
//! Env knobs (all optional):
//! ```text
//! AM_STRATEGY=bdd|auto|off   select the strategy (default off = explicit ladder)
//! AM_BDD_CAP=N               subset cap before giving up (default AM_CAP0 = 50_000)
//! AM_BDD_NODES=N             diagram node cap before giving up (default 30_000_000)
//! AM_BDD_MINALPHA=N          `auto` only fires at alphabets this big (default 16)
//! AM_BDD_PROBE=N             `auto`'s explicit probe cap (default 200_000/alpha)
//! AM_BDD_DEBUG=1             one stderr line per projection
//! ```
use crate::dfa::{Dfa, FxMap, peak_bump};
use std::sync::OnceLock;

// ---------------------------------------------------------------- node encoding
// A diagram reference is either an internal node index (< TERM) or a terminal
// TERM|payload, where the payload is an index into `Sets` (during construction) or a
// colour (during minimisation).
const TERM: u32 = 0x8000_0000;
#[inline] fn is_term(x: u32) -> bool { x & TERM != 0 }
#[inline] fn term(v: u32) -> u32 { v | TERM }
#[inline] fn payload(x: u32) -> u32 { x & !TERM }

#[inline]
fn mix(mut h: u64, w: u64) -> u64 {
    h ^= w;
    h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 29)
}

/// Reduced, hash-consed multi-terminal k-ary decision diagram over the tracks of a
/// letter: level `l` branches on the digit of track `nlev-1-l`, `k` children each.
/// Reduction rule: a node whose `k` children are all equal is replaced by that child
/// (so a level a function does not depend on is simply skipped).
struct Dd {
    k: usize,
    nlev: u32,
    lev: Vec<u32>,
    kid: Vec<u32>,   // stride k
    tab: Vec<u32>,   // open-addressed unique table: 0 empty, else node+1
    mask: usize,
    used: usize,
}

impl Dd {
    fn new(k: usize, nlev: u32) -> Dd {
        Dd { k, nlev, lev: Vec::new(), kid: Vec::new(), tab: vec![0; 4096], mask: 4095, used: 0 }
    }
    #[inline] fn nnodes(&self) -> usize { self.lev.len() }
    #[inline] fn level_of(&self, x: u32) -> u32 { if is_term(x) { self.nlev } else { self.lev[x as usize] } }
    /// Child `i` of `x` *as seen from level `l`*: a node below level `l` (or a
    /// terminal) does not depend on that track, so it is its own child.
    #[inline] fn child_at(&self, x: u32, l: u32, i: usize) -> u32 {
        if !is_term(x) && self.lev[x as usize] == l { self.kid[x as usize * self.k + i] } else { x }
    }
    fn hash(&self, l: u32, kids: &[u32]) -> usize {
        let mut h = mix(0xcbf2_9ce4_8422_2325, l as u64);
        for &c in kids { h = mix(h, c as u64); }
        h as usize
    }
    fn mk(&mut self, l: u32, kids: &[u32]) -> u32 {
        let first = kids[0];
        if kids.iter().all(|&x| x == first) { return first; }
        let mut i = self.hash(l, kids) & self.mask;
        loop {
            let e = self.tab[i];
            if e == 0 { break; }
            let n = (e - 1) as usize;
            if self.lev[n] == l && &self.kid[n * self.k..(n + 1) * self.k] == kids { return n as u32; }
            i = (i + 1) & self.mask;
        }
        let n = self.lev.len() as u32;
        assert!(n < TERM, "symbolic: diagram node index overflow");
        self.lev.push(l);
        self.kid.extend_from_slice(kids);
        self.tab[i] = n + 1;
        self.used += 1;
        if self.used * 10 > self.tab.len() * 7 { self.grow(); }
        n
    }
    fn grow(&mut self) {
        let ntab = vec![0u32; self.tab.len() * 2];
        let mask = ntab.len() - 1;
        self.tab = ntab;
        self.mask = mask;
        for n in 0..self.lev.len() {
            let l = self.lev[n];
            let h = self.hash(l, &self.kid[n * self.k..(n + 1) * self.k]);
            let mut i = h & self.mask;
            while self.tab[i] != 0 { i = (i + 1) & self.mask; }
            self.tab[i] = n as u32 + 1;
        }
    }
    fn clear(&mut self) {
        self.lev.clear(); self.kid.clear();
        for x in self.tab.iter_mut() { *x = 0; }
        self.used = 0;
    }
}

/// Interned sets of source states, stored as fixed-width bitsets in one arena with an
/// open-addressed content-hash table, so equal sets are the same `u32` id everywhere.
struct Sets {
    w: usize,
    arena: Vec<u64>,
    tab: Vec<u32>,
    mask: usize,
    n: usize,
}

impl Sets {
    fn new(nstates: usize) -> Sets {
        Sets { w: (nstates + 63) / 64, arena: Vec::new(), tab: vec![0; 4096], mask: 4095, n: 0 }
    }
    #[inline] fn get(&self, id: u32) -> &[u64] { &self.arena[id as usize * self.w..][..self.w] }
    fn hash(&self, bits: &[u64]) -> usize {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for &b in bits { h = mix(h, b); }
        h as usize
    }
    fn intern(&mut self, bits: &[u64]) -> u32 {
        let mut i = self.hash(bits) & self.mask;
        loop {
            let e = self.tab[i];
            if e == 0 { break; }
            let s = (e - 1) as usize;
            if &self.arena[s * self.w..][..self.w] == bits { return s as u32; }
            i = (i + 1) & self.mask;
        }
        let id = self.n as u32;
        self.arena.extend_from_slice(bits);
        self.tab[i] = id + 1;
        self.n += 1;
        if self.n * 10 > self.tab.len() * 7 { self.grow(); }
        id
    }
    fn grow(&mut self) {
        let ntab = vec![0u32; self.tab.len() * 2];
        self.tab = ntab;
        self.mask = self.tab.len() - 1;
        for s in 0..self.n {
            let h = self.hash(&self.arena[s * self.w..][..self.w]);
            let mut i = h & self.mask;
            while self.tab[i] != 0 { i = (i + 1) & self.mask; }
            self.tab[i] = s as u32 + 1;
        }
    }
}

// ---------------------------------------------------------------- strategy flag
/// Which determinization strategy `Dfa::exists` should use.
#[derive(Clone, Copy, PartialEq)]
pub enum Strategy { Off, Bdd, Auto }

fn strategy() -> Strategy {
    static S: OnceLock<u8> = OnceLock::new();
    match *S.get_or_init(|| match std::env::var("AM_STRATEGY").unwrap_or_default().as_str() {
        "bdd" | "symbolic" | "mona" => 1,
        "auto" => 2,
        _ => 0,
    }) { 1 => Strategy::Bdd, 2 => Strategy::Auto, _ => Strategy::Off }
}

fn envn(name: &str, dflt: usize) -> usize {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(dflt)
}

/// Is the symbolic strategy selected at all?  (`Dfa::exists`'s 3-line hook asks this
/// first so the default path pays one atomic load and nothing else.)
pub fn enabled() -> bool { strategy() != Strategy::Off }

/// Symbolic replacement for the body of `Dfa::exists`: project track `pos` away and
/// determinize+minimize symbolically, returning the same minimal DFA the explicit
/// ladder would return, or `None` if a cap was hit (the caller then falls back).
pub fn exists(d: &Dfa, var: &str) -> Option<Dfa> {
    let pos = d.vars.iter().position(|v| v == var)?;
    let nalpha = d.alpha / d.k;
    let strat = strategy();
    // Same philosophy as the explicit ladder (dfa.rs): a SMALL cap that fails fast is
    // worth more than a big one, because the cases that need millions of subsets are
    // exactly the ones a reverse-first construction settles in seconds.  So the
    // symbolic pass is a first rung, capped at AM_CAP0, and the ladder takes over.
    let cap0 = envn("AM_CAP0", 50_000);
    if strat == Strategy::Auto {
        // `auto` = the ladder with a symbolic rung inserted.  Two gates keep the rung
        // from costing anything on the easy projections that make up the bulk of a
        // compilation: a minimum alphabet (below it the explicit inner loop is simply
        // cheaper than hashing diagram nodes), and a *probe* -- a short explicit
        // forward construction whose cap is scaled so that it never does more than
        // ~200k subset-by-letter cells of work.  If the probe finishes, that IS the
        // answer and the symbolic pass is never entered; if it overflows, the
        // projection is one of the expensive ones and the symbolic pass takes over.
        if nalpha < envn("AM_BDD_MINALPHA", 16) { return None; }
        let probe = envn("AM_BDD_PROBE", (200_000 / nalpha).clamp(500, cap0));
        if let Some(det) = probe_forward(d, pos, nalpha, probe) {
            let res = det.zero_closure().minimize();
            crate::progress::states(res.nstates, var);
            return Some(res);
        }
    }
    let cap = envn("AM_BDD_CAP", cap0);
    let dbg = std::env::var("AM_BDD_DEBUG").is_ok();
    let t0 = std::time::Instant::now();
    crate::progress::phase("symbolic", &d.vars[pos]);
    let r = Ctx::run(d, pos, cap, envn("AM_BDD_NODES", 30_000_000));
    if dbg {
        match &r {
            Some(x) => eprintln!("    symbolic exists({}): nfa {} states x alpha {} -> min {} [{} ms]",
                                 d.vars[pos], d.nstates, nalpha, x.nstates, t0.elapsed().as_millis()),
            None => eprintln!("    symbolic exists({}): gave up (cap {}) after {} ms",
                              d.vars[pos], cap, t0.elapsed().as_millis()),
        }
    }
    r
}

/// A short explicit forward subset construction of the projected NFA, capped at
/// `cap` subsets.  This is the same construction `Dfa::exists` runs (same `Nfa`, same
/// `Nfa::determinize_capped`), used by `AM_STRATEGY=auto` as a cheap probe: only when
/// it overflows is the symbolic machinery worth building.
fn probe_forward(d: &Dfa, pos: usize, nalpha: usize, cap: usize) -> Option<Dfa> {
    let k = d.k;
    let mut newvars = d.vars.clone();
    newvars.remove(pos);
    let lo = k.pow(pos as u32);
    let hi = lo * k;
    let mut trans: Vec<Vec<u32>> = vec![Vec::new(); d.nstates * nalpha];
    for st in 0..d.nstates {
        for a in 0..nalpha {
            let (low, high) = (a % lo, a / lo);
            let at = st * d.alpha + low + high * hi;
            let mut set: Vec<u32> = (0..k).map(|dg| d.trans[at + dg * lo]).collect();
            set.sort_unstable();
            set.dedup();
            trans[st * nalpha + a] = set;
        }
    }
    crate::dfa::Nfa {
        k, vars: newvars, alpha: nalpha, nstates: d.nstates,
        trans, init: vec![0], accept: d.accept.clone(),
    }.determinize_capped(cap)
}

/// Everything one symbolic projection needs: the diagram, the set pool and the memo
/// tables, so the recursive helpers can be methods and borrow fields disjointly.
struct Ctx<'a> {
    src: &'a Dfa,
    /// index of the track being projected away (its position in `src.vars`)
    projected: usize,
    dd: Dd,
    sets: Sets,
    /// per source state: its transition diagram after the projection (terminals = set
    /// ids), built lazily -- a state whose row is never needed is never expanded
    rows: Vec<u32>,
    /// interned singleton `{s}` per source state, also lazy
    sing: Vec<u32>,
    /// scratch letter->set-id array while one row is being built
    arr: Vec<u32>,
    /// cache: sorted target tuple of one letter -> interned set id
    small: FxMap<Vec<u32>, u32>,
    /// `k^pos` and `k^(pos+1)`: where the projected track sits in a letter index
    lo: usize,
    hi: usize,
    umemo: FxMap<(u32, u32), u32>,   // diagram union
    smemo: FxMap<(u32, u32), u32>,   // set union
    cmemo: FxMap<(u32, u64), u32>,   // union of one 64-state chunk of a subset
    buf: Vec<u32>,                   // scratch child stack
    tmp: Vec<u64>,                   // scratch bitset
    stamp: Vec<u32>,                 // per-node visit stamp, for leaf collection
    mark: u32,
    nodecap: usize,
}

impl<'a> Ctx<'a> {
    fn run(src: &'a Dfa, pos: usize, cap: usize, nodecap: usize) -> Option<Dfa> {
        let k = src.k;
        let nn = src.vars.len() - 1;
        let nalpha = src.alpha / k;
        let mut c = Ctx {
            src,
            projected: pos,
            dd: Dd::new(k, nn as u32),
            sets: Sets::new(src.nstates),
            rows: vec![u32::MAX; src.nstates],
            sing: vec![u32::MAX; src.nstates],
            arr: vec![0; nalpha],
            small: FxMap::default(),
            lo: k.pow(pos as u32),
            hi: k.pow(pos as u32) * k,
            umemo: FxMap::default(), smemo: FxMap::default(), cmemo: FxMap::default(),
            buf: Vec::new(), tmp: vec![0u64; (src.nstates + 63) / 64],
            stamp: Vec::new(), mark: 0, nodecap,
        };
        c.determinize(cap, nalpha)
    }

    // ---------------------------------------------------------- build the NFA rows
    /// The interned singleton `{t}`.
    fn singleton(&mut self, t: u32) -> u32 {
        if self.sing[t as usize] != u32::MAX { return self.sing[t as usize]; }
        for x in self.tmp.iter_mut() { *x = 0; }
        self.tmp[t as usize / 64] = 1u64 << (t % 64);
        let i = self.sets.intern(&self.tmp);
        self.sing[t as usize] = i;
        i
    }

    /// Source state `s`'s transition diagram over the `n-1` surviving tracks: the
    /// terminal for letter `a` is the interned set `{ delta(s, a with digit d at the
    /// projected track) : d < k }`, so the projection happens here, once per letter,
    /// and the diagram then shares every letter that behaves the same.
    fn row_for(&mut self, s: usize) -> u32 {
        if self.rows[s] != u32::MAX { return self.rows[s]; }
        let k = self.src.k;
        let (lo, hi) = (self.lo, self.hi);
        let nalpha = self.arr.len();
        let base = s * self.src.alpha;
        let mut key: Vec<u32> = Vec::with_capacity(k);
        for a in 0..nalpha {
            let (low, high) = (a % lo, a / lo);
            let at = base + low + high * hi;
            let t0 = self.src.trans[at];
            // fast path: every digit of the projected track leads to the same state
            if (1..k).all(|dg| self.src.trans[at + dg * lo] == t0) {
                self.arr[a] = self.singleton(t0);
                continue;
            }
            key.clear();
            for dg in 0..k { key.push(self.src.trans[at + dg * lo]); }
            key.sort_unstable();
            key.dedup();
            let id = match self.small.get(&key) {
                Some(&i) => i,
                None => {
                    for x in self.tmp.iter_mut() { *x = 0; }
                    for &t in key.iter() { self.tmp[t as usize / 64] |= 1u64 << (t % 64); }
                    let i = self.sets.intern(&self.tmp);
                    self.small.insert(key.clone(), i);
                    i
                }
            };
            self.arr[a] = id;
        }
        let r = self.build(0, 0, nalpha);
        self.rows[s] = r;
        r
    }

    fn build(&mut self, lev: u32, base: usize, len: usize) -> u32 {
        if lev == self.dd.nlev { return term(self.arr[base]); }
        let k = self.dd.k;
        let sub = len / k;
        let mark = self.buf.len();
        for i in 0..k {
            let c = self.build(lev + 1, base + i * sub, sub);
            self.buf.push(c);
        }
        let node = self.dd.mk(lev, &self.buf[mark..]);
        self.buf.truncate(mark);
        node
    }

    // ---------------------------------------------------------------- union
    fn set_union(&mut self, x: u32, y: u32) -> u32 {
        if x == y { return x; }
        let key = if x < y { (x, y) } else { (y, x) };
        if let Some(&r) = self.smemo.get(&key) { return r; }
        let w = self.sets.w;
        let (px, py) = (key.0 as usize * w, key.1 as usize * w);
        for i in 0..w { self.tmp[i] = self.sets.arena[px + i] | self.sets.arena[py + i]; }
        let r = self.sets.intern(&self.tmp);
        self.smemo.insert(key, r);
        r
    }

    fn union(&mut self, a: u32, b: u32) -> u32 {
        if a == b { return a; }
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&r) = self.umemo.get(&key) { return r; }
        let (a, b) = key;
        let r = if is_term(a) && is_term(b) {
            let s = self.set_union(payload(a), payload(b));
            term(s)
        } else {
            let l = self.dd.level_of(a).min(self.dd.level_of(b));
            let k = self.dd.k;
            let mark = self.buf.len();
            for i in 0..k {
                let ca = self.dd.child_at(a, l, i);
                let cb = self.dd.child_at(b, l, i);
                let c = self.union(ca, cb);
                self.buf.push(c);
            }
            let node = self.dd.mk(l, &self.buf[mark..]);
            self.buf.truncate(mark);
            node
        };
        self.umemo.insert(key, r);
        r
    }

    /// Union of the rows of the states in one 64-state chunk of a subset, memoised on
    /// the raw bit pattern: different subsets that agree on a chunk share the work.
    fn chunk(&mut self, wi: u32, bits: u64) -> u32 {
        if let Some(&r) = self.cmemo.get(&(wi, bits)) { return r; }
        let lowbit = bits & bits.wrapping_neg();
        let s = wi as usize * 64 + lowbit.trailing_zeros() as usize;
        let rest = bits ^ lowbit;
        let r = if rest == 0 { self.row_for(s) } else {
            let a = self.row_for(s);
            let b = self.chunk(wi, rest);
            self.union(a, b)
        };
        self.cmemo.insert((wi, bits), r);
        r
    }

    /// Transition diagram of a whole subset.
    fn row_of(&mut self, setid: u32) -> u32 {
        let w = self.sets.w;
        let base = setid as usize * w;
        let mut acc: Option<u32> = None;
        for wi in 0..w {
            let bits = self.sets.arena[base + wi];
            if bits == 0 { continue; }
            let c = self.chunk(wi as u32, bits);
            acc = Some(match acc { None => c, Some(a) => self.union(a, c) });
        }
        match acc {
            Some(a) => a,
            None => {
                // the empty subset: every letter leads back to the empty subset
                for x in self.tmp.iter_mut() { *x = 0; }
                let e = self.sets.intern(&self.tmp);
                term(e)
            }
        }
    }

    /// The set reached from `setid` on the all-zero letter (follow child 0 down).
    fn zero_succ(&mut self, setid: u32) -> u32 {
        let mut n = self.row_of(setid);
        while !is_term(n) { n = self.dd.kid[n as usize * self.dd.k]; }
        payload(n)
    }

    // ---------------------------------------------------- leaves in letter order
    /// Push every terminal of `node` into `out`, first-occurrence order, which -- with
    /// children visited in digit order -- is exactly the order the explicit forward
    /// subset construction discovers successors in (letters 0,1,2,... in index order).
    fn leaves(&mut self, node: u32, out: &mut Vec<u32>) {
        if is_term(node) { out.push(payload(node)); return; }
        let n = node as usize;
        if self.stamp.len() <= n { self.stamp.resize(self.dd.nnodes().max(n + 1), 0); }
        if self.stamp[n] == self.mark { return; }
        self.stamp[n] = self.mark;
        let k = self.dd.k;
        for i in 0..k {
            let c = self.dd.kid[n * k + i];
            self.leaves(c, out);
        }
    }

    // ---------------------------------------------------------- subset construction
    fn determinize(&mut self, cap: usize, nalpha: usize) -> Option<Dfa> {
        // initial subset: {0}, closed under the all-zero letter in msd mode (that is
        // what `Dfa::zero_closure` does after the explicit construction -- leading
        // zeros must not change acceptance).
        for x in self.tmp.iter_mut() { *x = 0; }
        self.tmp[0] |= 1;
        let mut init = self.sets.intern(&self.tmp);
        if !crate::dfa::is_lsd() {
            loop {
                let nxt = self.zero_succ(init);
                let u = self.set_union(init, nxt);
                if u == init { break; }
                init = u;
            }
        }
        let mut states: Vec<u32> = vec![init];
        let mut sid: FxMap<u32, u32> = FxMap::default();
        sid.insert(init, 0);
        let mut srows: Vec<u32> = Vec::new();
        let mut leaves: Vec<u32> = Vec::new();
        let mut qi = 0usize;
        let mut tick = crate::dfa::SUBSET_TICK;
        while qi < states.len() {
            let node = self.row_of(states[qi]);
            srows.push(node);
            self.mark += 1;
            leaves.clear();
            self.leaves(node, &mut leaves);
            for &l in leaves.iter() {
                // NB: `insert` would overwrite an existing id -- only new subsets get one
                if !sid.contains_key(&l) { sid.insert(l, states.len() as u32); states.push(l); }
            }
            qi += 1;
            if states.len() >= cap || self.dd.nnodes() > self.nodecap { return None; }
            if states.len() >= tick { tick = states.len() + crate::dfa::SUBSET_TICK; crate::progress::subsets(states.len()); }
        }
        peak_bump(states.len());
        let m = states.len();

        // acceptance: a subset accepts iff it holds an accepting source state
        let w = self.sets.w;
        let mut accset = vec![0u64; w];
        for s in 0..self.src.nstates { if self.src.accept[s] { accset[s / 64] |= 1u64 << (s % 64); } }
        let mut accept: Vec<bool> = (0..m).map(|q| {
            let b = states[q] as usize * w;
            (0..w).any(|i| self.sets.arena[b + i] & accset[i] != 0)
        }).collect();

        // lsd padding is trailing zeros, so acceptance must be closed under them
        if crate::dfa::is_lsd() {
            let zsucc: Vec<u32> = (0..m).map(|q| {
                let mut n = srows[q];
                while !is_term(n) { n = self.dd.kid[n as usize * self.dd.k]; }
                sid[&payload(n)]
            }).collect();
            loop {
                let mut changed = false;
                for q in 0..m {
                    if !accept[q] && accept[zsucc[q] as usize] { accept[q] = true; changed = true; }
                }
                if !changed { break; }
            }
        }

        // relabel terminals from subset ids to state ids
        let mut memo: FxMap<u32, u32> = FxMap::default();
        let rows: Vec<u32> = (0..m).map(|q| {
            let r = srows[q];
            self.relabel(r, &sid, &mut memo)
        }).collect();

        Some(self.minimize(&rows, &accept, nalpha))
    }

    fn relabel(&mut self, node: u32, sid: &FxMap<u32, u32>, memo: &mut FxMap<u32, u32>) -> u32 {
        if is_term(node) { return term(sid[&payload(node)]); }
        if let Some(&r) = memo.get(&node) { return r; }
        let k = self.dd.k;
        let l = self.dd.lev[node as usize];
        let mark = self.buf.len();
        for i in 0..k {
            let c = self.dd.kid[node as usize * k + i];
            let c = self.relabel(c, sid, memo);
            self.buf.push(c);
        }
        let r = self.dd.mk(l, &self.buf[mark..]);
        self.buf.truncate(mark);
        memo.insert(node, r);
        r
    }

    // ---------------------------------------------------------------- minimisation
    /// Moore refinement on the diagrams.  Recolouring a state's whole transition row
    /// and hash-consing it gives a canonical signature for "where does this state go,
    /// as a function of the letter, up to the current colouring" in one memoised walk
    /// over the row's nodes -- no `alpha`-sized signature vector is ever built.
    fn minimize(&mut self, rows: &[u32], accept: &[bool], nalpha: usize) -> Dfa {
        let m = rows.len();
        let mut color: Vec<u32> = accept.iter().map(|&b| b as u32).collect();
        let mut ncolors = if accept.iter().any(|&b| b) && accept.iter().any(|&b| !b) { 2 } else { 1 };
        let mut sd = Dd::new(self.dd.k, self.dd.nlev);
        let mut sig: Vec<u32> = vec![0; m];
        loop {
            sd.clear();
            let mut memo: FxMap<u32, u32> = FxMap::default();
            let mut buf: Vec<u32> = Vec::new();
            for q in 0..m {
                sig[q] = recolor(&self.dd, &mut sd, rows[q], &color, &mut memo, &mut buf);
            }
            let mut fresh: FxMap<(u32, u32), u32> = FxMap::default();
            let mut newcolor = vec![0u32; m];
            for q in 0..m {
                let n = fresh.len() as u32;
                newcolor[q] = *fresh.entry((color[q], sig[q])).or_insert(n);
            }
            let nc = fresh.len() as u32;
            color = newcolor;
            if nc == ncolors { break; }
            ncolors = nc;
        }
        // relabel colours: start state first, then in state order (matches dfa.rs)
        let nc = ncolors as usize;
        let mut id = vec![u32::MAX; nc];
        id[color[0] as usize] = 0;
        let mut cnt = 1u32;
        let mut rep = vec![usize::MAX; nc];
        for q in 0..m {
            let c = color[q] as usize;
            if id[c] == u32::MAX { id[c] = cnt; cnt += 1; }
            if rep[c] == usize::MAX { rep[c] = q; }
        }
        let mut trans = vec![0u32; nc * nalpha];
        let mut acc = vec![false; nc];
        for c in 0..nc {
            let q = rep[c];
            let f = id[c] as usize;
            acc[f] = accept[q];
            let node = rows[q];
            expand(&self.dd, node, 0, 0, nalpha, &color, &id, &mut trans[f * nalpha..(f + 1) * nalpha]);
        }
        let mut vars = self.src.vars.clone();
        vars.remove(self.projected);
        Dfa { k: self.src.k, vars, alpha: nalpha, nstates: nc, trans, accept: acc }
    }
}

/// Recolour every terminal of `node` (a state id) to its current colour, rebuilding in
/// the signature diagram `sd`; equal results are the same id, which is the signature.
fn recolor(dd: &Dd, sd: &mut Dd, node: u32, color: &[u32], memo: &mut FxMap<u32, u32>, buf: &mut Vec<u32>) -> u32 {
    if is_term(node) { return term(color[payload(node) as usize]); }
    if let Some(&r) = memo.get(&node) { return r; }
    let k = dd.k;
    let l = dd.lev[node as usize];
    let mark = buf.len();
    for i in 0..k {
        let c = dd.kid[node as usize * k + i];
        let c = recolor(dd, sd, c, color, memo, buf);
        buf.push(c);
    }
    let r = sd.mk(l, &buf[mark..]);
    buf.truncate(mark);
    memo.insert(node, r);
    r
}

/// Expand one row of the final (small) automaton back to an explicit transition row.
fn expand(dd: &Dd, node: u32, lev: u32, base: usize, len: usize, color: &[u32], id: &[u32], out: &mut [u32]) {
    if is_term(node) {
        let t = id[color[payload(node) as usize] as usize];
        for i in base..base + len { out[i] = t; }
        return;
    }
    let k = dd.k;
    let sub = len / k;
    if dd.lev[node as usize] > lev {
        for i in 0..k { expand(dd, node, lev + 1, base + i * sub, sub, color, id, out); }
        return;
    }
    for i in 0..k {
        let c = dd.kid[node as usize * k + i];
        expand(dd, c, lev + 1, base + i * sub, sub, color, id, out);
    }
}
