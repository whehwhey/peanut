//! Deterministic finite automata over product alphabets of base-k digits.
//!
//! A `Dfa` with variables [v0..v_{n-1}] over base k reads words whose letters are
//! n-tuples of digits in {0..k-1}. Letter (d0,..,d_{n-1}) is encoded as
//! sum_j d_j * k^j.  Words are read most-significant-digit first, and every
//! automaton we build is *leading-zero robust*: prepending a tuple of zeros to a
//! word never changes acceptance.  State 0 is always the start state.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use crate::det_par;

/// FxHash — the rustc hasher. The default SipHash is cryptographic and costs more
/// than the automata operations it guards; this is 2-4x faster on our key shapes.
#[derive(Default)]
pub struct FxHasher { hash: u64 }
const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;
impl FxHasher {
    #[inline] fn add(&mut self, w: u64) { self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(SEED); }
}
impl Hasher for FxHasher {
    #[inline] fn write(&mut self, bytes: &[u8]) {
        let mut c = bytes;
        while c.len() >= 8 { self.add(u64::from_le_bytes(c[..8].try_into().unwrap())); c = &c[8..]; }
        if !c.is_empty() { let mut b = [0u8; 8]; b[..c.len()].copy_from_slice(c); self.add(u64::from_le_bytes(b)); }
    }
    #[inline] fn write_u32(&mut self, i: u32) { self.add(i as u64); }
    #[inline] fn write_u64(&mut self, i: u64) { self.add(i); }
    #[inline] fn write_usize(&mut self, i: usize) { self.add(i as u64); }
    #[inline] fn finish(&self) -> u64 { self.hash }
}
/// A `HashMap` keyed/hashed with [`FxHasher`] instead of the default SipHash.
pub type FxMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

/// A DFA state index. `u32` is plenty (automata this engine builds top out well
/// below 2^32 states) and halves the memory of transition tables vs `usize`.
pub type State = u32;

// ---------------------------------------------------------------- digit order
// Two representations are supported and must agree on every closed formula:
//   msd  words are read most-significant-digit first; padding = leading zeros
//   lsd  words are read least-significant-digit first; padding = trailing zeros
// lsd makes addition deterministic in the forward direction, so projecting a sum
// away does not blow up the subset construction. msd is kept as an independent
// oracle: any disagreement between the two is a bug.
use std::sync::atomic::{AtomicBool, Ordering};
/// Process-global digit-order flag: `false` = msd (default), `true` = lsd.
/// Set once at startup via `mode msd|lsd`; every automaton constructor consults
/// [`is_lsd`] to decide which convention it is building under.
pub static LSD: AtomicBool = AtomicBool::new(false);

/// Largest intermediate automaton built while compiling the current formula.
/// This is the size of the proof: a short sentence whose verification needs a
/// huge automaton is, by any reasonable reading, a deep theorem.
use std::sync::atomic::AtomicUsize;
pub static PEAK: AtomicUsize = AtomicUsize::new(0);
/// Zero the peak-size counter (call before compiling a new top-level formula).
pub fn peak_reset() { PEAK.store(0, Ordering::Relaxed) }
/// Read the peak-size counter accumulated since the last [`peak_reset`].
pub fn peak_get() -> usize { PEAK.load(Ordering::Relaxed) }
/// Record `n` as a new intermediate-automaton size if it exceeds the current peak.
#[inline] pub fn peak_bump(n: usize) {
    if n > PEAK.load(Ordering::Relaxed) { PEAK.store(n, Ordering::Relaxed) }
}
/// Is the engine currently operating in least-significant-digit-first mode?
#[inline] pub fn is_lsd() -> bool { LSD.load(Ordering::Relaxed) }
/// Set the process-wide digit order (`true` = lsd, `false` = msd).
pub fn set_lsd(b: bool) { LSD.store(b, Ordering::Relaxed) }

/// A deterministic finite automaton over the product alphabet of `vars.len()`
/// base-`k` digit coordinates. `trans` is a flattened `nstates x alpha` table
/// (row `s`, column `a` at index `s * alpha + a`); state `0` is always the
/// start state. Every automaton the engine constructs is leading/trailing-zero
/// robust in the active digit order (see the module header), so acceptance is
/// well-defined for numbers of any padded width.
#[derive(Clone, Debug)]
pub struct Dfa {
    pub k: usize,
    pub vars: Vec<String>,
    pub alpha: usize,
    pub nstates: usize,
    pub trans: Vec<State>, // nstates * alpha
    pub accept: Vec<bool>,
}

/// A nondeterministic finite automaton with (possibly) multiple start states
/// and multiple successors per (state, symbol). Used as the intermediate form
/// during determinization (see [`Dfa::determinize`] and
/// [`Dfa::reverse_determinize`]) and for the Brzozowski-style construction.
pub struct Nfa {
    pub k: usize,
    pub vars: Vec<String>,
    pub alpha: usize,
    pub nstates: usize,
    pub trans: Vec<Vec<State>>, // nstates * alpha
    pub init: Vec<State>,
    pub accept: Vec<bool>,
}

/// Upper bound on `k^n` (the product alphabet size) any automaton may use.
/// Guards against runaway memory from formulas with too many free/bound vars.
pub const MAX_ALPHA: usize = 1 << 22;

/// How many new subsets between `{"ev":"subsets"}` progress events.
pub const SUBSET_TICK: usize = 50_000;

fn pow(k: usize, n: usize) -> usize {
    let mut r: usize = 1;
    for _ in 0..n {
        r = r.checked_mul(k).expect("alphabet overflow");
        assert!(r <= MAX_ALPHA, "alphabet too large ({} vars, base {})", n, k);
    }
    r
}

/// digit of `sym` in coordinate `i`
#[inline]
pub fn digit(sym: usize, i: usize, k: usize) -> usize {
    (sym / k.pow(i as u32)) % k
}

impl Dfa {
    #[inline]
    pub fn t(&self, s: usize, a: usize) -> usize {
        self.trans[s * self.alpha + a] as usize
    }

    /// Build a `Dfa` from raw parts. `trans.len()` must equal `nstates * k^vars.len()`
    /// and `accept.len()` must equal `nstates`; both are checked with `assert_eq!`.
    pub fn new(k: usize, vars: Vec<String>, nstates: usize, trans: Vec<State>, accept: Vec<bool>) -> Dfa {
        let alpha = pow(k, vars.len());
        assert_eq!(trans.len(), nstates * alpha);
        assert_eq!(accept.len(), nstates);
        Dfa { k, vars, alpha, nstates, trans, accept }
    }

    /// Language over the given vars that is everything (true) or nothing (false).
    ///
    /// Under a numeration system "everything" means "every track holds a valid
    /// representation" -- this is the cylindrification leaf, and leaving it
    /// unrestricted would let an unconstrained variable range over junk words.
    pub fn constant(k: usize, vars: Vec<String>, val: bool) -> Dfa {
        let alpha = pow(k, vars.len());
        let d = Dfa { k, vars, alpha, nstates: 1, trans: vec![0; alpha], accept: vec![val] };
        if val { crate::numsys::restrict(&d) } else { d }
    }

    /// Does the automaton accept the empty word (state 0 accepting)?
    pub fn accepts_epsilon(&self) -> bool { self.accept[0] }

    /// Does this automaton accept at least one word?
    pub fn is_nonempty(&self) -> bool {
        let mut seen = vec![false; self.nstates];
        let mut stack = vec![0usize];
        seen[0] = true;
        while let Some(s) = stack.pop() {
            if self.accept[s] { return true; }
            for a in 0..self.alpha {
                let d = self.t(s, a);
                if !seen[d] { seen[d] = true; stack.push(d); }
            }
        }
        false
    }

    /// Run on an explicit word (sequence of symbol indices).
    pub fn run(&self, word: &[usize]) -> bool {
        let mut s = 0usize;
        for &a in word { s = self.t(s, a); }
        self.accept[s]
    }

    /// Enumerate up to `limit` accepted words as tuples of integers (values of each var).
    /// Words are explored by increasing length; padding is stripped by dedup on value.
    ///
    /// The word itself is kept (as a parent-pointer arena, 8 bytes per node) rather
    /// than an incrementally accumulated value, because under a numeration system a
    /// prefix has no value until the total length is known -- the value of a valid
    /// word is its rank in the radix ordering of the validity language.
    pub fn enumerate(&self, limit: usize, maxlen: usize) -> Vec<Vec<u64>> {
        let n = self.vars.len();
        let mut out = Vec::new();
        if n == 0 { return out; }
        let mut seen: std::collections::HashSet<Vec<u64>> = std::collections::HashSet::new();
        // arena of word nodes: (parent index, symbol); index 0 is the empty word
        let mut par: Vec<u32> = vec![0];
        let mut sym: Vec<u32> = vec![0];
        let word_of = |par: &Vec<u32>, sym: &Vec<u32>, mut i: usize| -> Vec<usize> {
            let mut w = Vec::new();
            while i != 0 { w.push(sym[i] as usize); i = par[i] as usize; }
            w.reverse();
            w
        };
        let mut frontier: Vec<(usize, u32)> = vec![(0, 0)];
        for _depth in 0..maxlen {
            let mut next: Vec<(usize, u32)> = Vec::new();
            for (s, node) in frontier.drain(..) {
                for a in 0..self.alpha {
                    let d = self.t(s, a);
                    let idx = par.len() as u32;
                    par.push(node); sym.push(a as u32);
                    if self.accept[d] {
                        let w = word_of(&par, &sym, idx as usize);
                        if let Some(v) = crate::numsys::decode_word(self.k, n, &w) {
                            if seen.insert(v.clone()) {
                                out.push(v);
                                if out.len() >= limit { return out; }
                            }
                        }
                    }
                    next.push((d, idx));
                }
            }
            if next.is_empty() { break; }
            // prune: cap frontier to avoid blowup
            if next.len() > 400_000 { next.truncate(400_000); }
            // compact the word arena onto the surviving frontier's ancestors, so its
            // size stays proportional to the frontier and not to everything explored
            if par.len() > 1 << 20 {
                let mut keep = vec![false; par.len()];
                keep[0] = true;
                for &(_, n) in next.iter() {
                    let mut i = n as usize;
                    while !keep[i] { keep[i] = true; i = par[i] as usize; }
                }
                let mut map = vec![0u32; par.len()];
                let (mut np, mut nsym) = (Vec::new(), Vec::new());
                for i in 0..par.len() {
                    if !keep[i] { continue; }
                    map[i] = np.len() as u32;
                    np.push(map[par[i] as usize]);
                    nsym.push(sym[i]);
                }
                for e in next.iter_mut() { e.1 = map[e.1 as usize]; }
                par = np; sym = nsym;
            }
            frontier = next;
        }
        out
    }

    /// Breadth-first spanning tree from the start state.  Returns (prev, sym) with
    /// prev[s] = u32::MAX for unreachable states and prev[0] = 0.  Used to extract a
    /// SHORTEST word reaching a given state -- i.e. a smallest witness.
    pub fn bfs_tree(&self) -> (Vec<u32>, Vec<u32>) {
        let mut prev = vec![u32::MAX; self.nstates];
        let mut psym = vec![0u32; self.nstates];
        let mut q = std::collections::VecDeque::new();
        prev[0] = 0;
        q.push_back(0usize);
        while let Some(s) = q.pop_front() {
            for a in 0..self.alpha {
                let d = self.t(s, a);
                if prev[d] == u32::MAX {
                    prev[d] = s as u32;
                    psym[d] = a as u32;
                    q.push_back(d);
                }
            }
        }
        (prev, psym)
    }

    /// Reconstruct the shortest word reaching `s` from a `bfs_tree` result.
    pub fn word_to(&self, prev: &[u32], psym: &[u32], s: usize) -> Option<Vec<usize>> {
        if prev[s] == u32::MAX { return None; }
        let mut w = Vec::new();
        let mut t = s;
        while t != 0 {
            w.push(psym[t] as usize);
            t = prev[t] as usize;
        }
        w.reverse();
        Some(w)
    }

    /// A shortest accepted word, or None if the language is empty.
    pub fn shortest_word(&self) -> Option<Vec<usize>> {
        let (prev, psym) = self.bfs_tree();
        let mut best: Option<Vec<usize>> = None;
        for s in 0..self.nstates {
            if !self.accept[s] || prev[s] == u32::MAX { continue; }
            let w = self.word_to(&prev, &psym, s)?;
            if best.as_ref().map_or(true, |b| w.len() < b.len()) { best = Some(w); }
        }
        best
    }

    /// Flip every accept bit; language complement over the same alphabet.
    ///
    /// Under a numeration system the flip would also accept every *invalid*
    /// word (the original rejects them all), so the result is re-restricted to
    /// valid representations -- the same thing Walnut's `not()` does with
    /// `applyAllRepresentations`.  Without it `A i. phi(i)` is vacuously false
    /// for any phi, since `E i. ~phi(i)` is witnessed by a junk word.
    pub fn complement(&self) -> Dfa {
        let c = Dfa { accept: self.accept.iter().map(|b| !b).collect(), ..self.clone() };
        crate::numsys::restrict(&c)
    }

    /// Lift to a larger (superset) ordered variable list.
    pub fn extend_vars(&self, newvars: &[String]) -> Dfa {
        if self.vars == newvars { return self.clone(); }
        for v in &self.vars { assert!(newvars.contains(v), "extend_vars: {} missing", v); }
        let k = self.k;
        let nalpha = pow(k, newvars.len());
        // position of each of self.vars inside newvars
        let pos: Vec<usize> = self.vars.iter().map(|v| newvars.iter().position(|w| w == v).unwrap()).collect();
        let mut map = vec![0usize; nalpha];
        for s in 0..nalpha {
            let mut old = 0usize;
            let mut mult = 1usize;
            for j in 0..self.vars.len() {
                old += digit(s, pos[j], k) * mult;
                mult *= k;
            }
            map[s] = old;
        }
        let mut trans = vec![0u32; self.nstates * nalpha];
        for st in 0..self.nstates {
            for s in 0..nalpha {
                trans[st * nalpha + s] = self.trans[st * self.alpha + map[s]];
            }
        }
        Dfa { k, vars: newvars.to_vec(), alpha: nalpha, nstates: self.nstates, trans, accept: self.accept.clone() }
    }

    /// Rename variables (must stay a bijection), then re-sort coordinates so the
    /// variable list is in canonical (sorted) order.
    pub fn rename(&self, map: &dyn Fn(&str) -> String) -> Dfa {
        let newnames: Vec<String> = self.vars.iter().map(|v| map(v)).collect();
        let mut sorted = newnames.clone();
        sorted.sort();
        assert!(sorted.windows(2).all(|w| w[0] != w[1]), "rename produced duplicate variables");
        let tmp = Dfa { vars: newnames, ..self.clone() };
        tmp.reorder(&sorted)
    }

    /// Permute coordinates so that `self.vars` becomes `target` (a permutation of it).
    pub fn reorder(&self, target: &[String]) -> Dfa {
        if self.vars == target { return self.clone(); }
        let k = self.k;
        let n = self.vars.len();
        // coordinate j of the target corresponds to coordinate src[j] of self
        let src: Vec<usize> = target.iter().map(|v| self.vars.iter().position(|w| w == v).expect("reorder: missing var")).collect();
        let mut map = vec![0usize; self.alpha];
        for s in 0..self.alpha {
            let mut old = 0usize;
            let mut mult = 1usize;
            for j in 0..n {
                old += digit(s, src[j], k) * k.pow(0) * 0; // placeholder, replaced below
                let _ = mult;
                mult = mult;
            }
            // build: target symbol s has digit d_j at coordinate j; that digit belongs
            // to self's coordinate src[j].
            let mut o = 0usize;
            for j in 0..n { o += digit(s, j, k) * k.pow(src[j] as u32); }
            old = o;
            map[s] = old;
        }
        let mut trans = vec![0u32; self.nstates * self.alpha];
        for st in 0..self.nstates {
            for s in 0..self.alpha { trans[st * self.alpha + s] = self.trans[st * self.alpha + map[s]]; }
        }
        Dfa { k, vars: target.to_vec(), alpha: self.alpha, nstates: self.nstates, trans, accept: self.accept.clone() }
    }

    /// Boolean product. `op` combines the two acceptance bits.
    pub fn product(&self, other: &Dfa, op: impl Fn(bool, bool) -> bool) -> Dfa {
        assert_eq!(self.k, other.k);
        let mut vars: Vec<String> = self.vars.clone();
        for v in &other.vars { if !vars.contains(v) { vars.push(v.clone()); } }
        vars.sort();
        let a = self.extend_vars(&vars);
        let b = other.extend_vars(&vars);
        let alpha = a.alpha;
        let mut order: Vec<(u32, u32)> = Vec::new();
        let mut trans: Vec<u32> = Vec::new();
        // AM_FAST: when |A|*|B| fits a direct-indexed table (<= 64M pairs, 256 MB),
        // the pair -> id map is an array lookup instead of a hash probe.  Pairs are
        // still discovered in the same BFS order, so the numbering is unchanged.
        let flat = det_par::fast_enabled()
            && a.nstates.checked_mul(b.nstates).map_or(false, |n| n <= (1 << 26));
        if flat {
            let bn = b.nstates;
            let mut index: Vec<u32> = vec![u32::MAX; a.nstates * bn];
            index[0] = 0;
            order.push((0, 0));
            let mut i = 0;
            while i < order.len() {
                let (p, q) = order[i];
                let (pr, qr) = (p as usize * alpha, q as usize * alpha);
                for s in 0..alpha {
                    let np = a.trans[pr + s];
                    let nq = b.trans[qr + s];
                    let key = np as usize * bn + nq as usize;
                    let mut id = index[key];
                    if id == u32::MAX {
                        id = order.len() as u32;
                        index[key] = id;
                        order.push((np, nq));
                    }
                    trans.push(id);
                }
                i += 1;
            }
        } else {
        let mut index: FxMap<(u32, u32), u32> = FxMap::default();
        index.insert((0, 0), 0);
        order.push((0, 0));
        let mut i = 0;
        while i < order.len() {
            let (p, q) = order[i];
            for s in 0..alpha {
                let np = a.trans[p as usize * alpha + s];
                let nq = b.trans[q as usize * alpha + s];
                let id = *index.entry((np, nq)).or_insert_with(|| {
                    order.push((np, nq));
                    (order.len() - 1) as u32
                });
                trans.push(id);
            }
            i += 1;
        }
        }
        peak_bump(order.len());
        let accept: Vec<bool> = order.iter().map(|&(p, q)| op(a.accept[p as usize], b.accept[q as usize])).collect();
        let res = Dfa { k: a.k, vars, alpha, nstates: order.len(), trans, accept }.minimize();
        // Both operands reject every invalid word, so the product accepts an
        // invalid word exactly when op(false,false) does -- true for `=>` and
        // `<=>`, false for `&` and `|`.  Only then is a re-restriction needed.
        if op(false, false) { crate::numsys::restrict(&res) } else { res }
    }

    /// Conjunction (`&`): product automaton, accept iff both accept.
    pub fn and(&self, o: &Dfa) -> Dfa { self.product(o, |x, y| x && y) }
    /// Disjunction (`|`): product automaton, accept iff either accepts.
    pub fn or(&self, o: &Dfa) -> Dfa { self.product(o, |x, y| x || y) }
    /// Implication (`=>`): product automaton, accept iff `!self | o`.
    pub fn implies(&self, o: &Dfa) -> Dfa { self.product(o, |x, y| !x || y) }
    /// Biconditional: product automaton, accept iff both sides agree.
    pub fn iff(&self, o: &Dfa) -> Dfa { self.product(o, |x, y| x == y) }

    /// Existentially project out `var`, then re-close under leading zeros.
    ///
    /// Dispatches to the flat/parallel core ([`crate::det_par`]) when `AM_FAST`
    /// or `AM_PAR` is set; `AM_FAST_VERIFY=1` runs both and asserts the results
    /// are identical element by element.
    pub fn exists(&self, var: &str) -> Dfa {
        // AM_STRATEGY=bdd|auto: symbolic (MONA-style) projection, default off;
        // `None` means a cap was hit, so fall through to the explicit ladder.
        if crate::symbolic::enabled() {
            if let Some(d) = crate::symbolic::exists(self, var) { return d; }
        }
        if det_par::fast_enabled() {
            let fast = self.exists_fast(var);
            if det_par::verify() { det_par::assert_same("exists", &self.exists_ref(var), &fast); }
            return fast;
        }
        self.exists_ref(var)
    }

    /// The flat-core existential projection: no `Vec<Vec<State>>` NFA, no
    /// per-subset allocation, the same determinization ladder and the same
    /// output as [`Dfa::exists_ref`].
    fn exists_fast(&self, var: &str) -> Dfa {
        let Some(pos) = self.vars.iter().position(|v| v == var) else { return self.clone() };
        let k = self.k;
        let mut newvars = self.vars.clone();
        newvars.remove(pos);
        let nalpha = pow(k, newvars.len());
        let closed = newvars.is_empty();
        let nfa = det_par::FlatNfa::from_exists(self, pos, newvars, nalpha);
        let cap0: usize = std::env::var("AM_CAP0").ok().and_then(|v| v.parse().ok()).unwrap_or(50_000);
        let cap: usize = std::env::var("AM_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(3_000_000);
        let dbg = std::env::var("AM_DEBUG2").is_ok();
        // AM_LAZY_CLOSED: projecting the last variable leaves a one-letter
        // alphabet, and the only thing ever read off the result is
        // `zero_closure().accept[0]` -- i.e. plain reachability in the NFA.
        if closed && det_par::lazy_closed() {
            let res = det_par::closed_verdict(&nfa);
            crate::progress::states(res.nstates, var);
            if dbg { eprintln!("    exists({}): lazy closed verdict, {} states", var, res.nstates); }
            return res;
        }
        let brz = |c: usize| -> Option<Dfa> {
            let r = nfa.reversed()?;
            let r1 = det_par::determinize_capped(&r, c)?;
            let f2 = det_par::FlatNfa::from_dfa(&r1, vec![0]).reversed()?;
            det_par::determinize_capped(&f2, c)
        };
        crate::progress::phase("forward", var);
        let det = if let Some(d) = det_par::determinize_capped(&nfa, cap0) { d }
            else if let Some(d) = { crate::progress::phase("brzozowski", var);
                                    brz(cap0.saturating_mul(4).max(200_000)) } {
                if dbg { eprintln!("    exists({}): forward > {} subsets; Brzozowski(small) ok, {} states", var, cap0, d.nstates); }
                d }
            else if let Some(d) = { crate::progress::phase("forward", var);
                                    det_par::determinize_capped(&nfa, cap) } {
                if dbg { eprintln!("    exists({}): Brzozowski(small) failed; forward(big) ok", var); }
                d }
            else {
                if dbg { eprintln!("    exists({}): forward exceeded {} subsets, trying Brzozowski(big)", var, cap); }
                let big = cap.saturating_mul(4).max(8_000_000);
                crate::progress::phase("brzozowski", var);
                brz(big).expect("forward and reverse determinization both blew up")
            };
        let res = det.zero_closure().minimize();
        crate::progress::states(res.nstates, var);
        if dbg {
            eprintln!("    exists({}): nfa {} states x alpha {} -> det {} -> min {}",
                      var, self.nstates, nalpha, det.nstates, res.nstates);
        }
        res
    }

    /// Reference existential projection (the pre-2026-08-18 code path): builds a
    /// `Vec<Vec<State>>` NFA and runs the same ladder over `Nfa::determinize_capped`.
    fn exists_ref(&self, var: &str) -> Dfa {
        let Some(pos) = self.vars.iter().position(|v| v == var) else { return self.clone() };
        let k = self.k;
        let mut newvars = self.vars.clone();
        newvars.remove(pos);
        let nalpha = pow(k, newvars.len());
        // For each new symbol, the set of old symbols projecting onto it.
        // old symbol = insert digit d at coordinate `pos`.
        let lo = k.pow(pos as u32);
        let hi = lo * k;
        let mut trans: Vec<Vec<State>> = vec![Vec::new(); self.nstates * nalpha];
        for st in 0..self.nstates {
            for s in 0..nalpha {
                let low = s % lo;
                let high = s / lo;
                let mut set: Vec<State> = Vec::with_capacity(k);
                for d in 0..k {
                    let old = low + d * lo + high * hi;
                    set.push(self.trans[st * self.alpha + old]);
                }
                set.sort_unstable();
                set.dedup();
                trans[st * nalpha + s] = set;
            }
        }
        let nfa = Nfa {
            k, vars: newvars, alpha: nalpha, nstates: self.nstates,
            trans, init: vec![0], accept: self.accept.clone(),
        };
        // Adaptive determinization ladder (2026-08-16).  Measured on the FE sweep: a SMALL
        // forward cap that fails fast into Brzozowski (reverse-first) is dramatically
        // better than a big forward cap -- 482 states in 0 s where cap=3M blew 6 GB.
        // Ladder: forward(cap0) -> Brzozowski(cap0*4) -> forward(cap) -> Brzozowski(cap*4).
        //   AM_CAP0 (default 50_000)   first, cheap forward attempt
        //   AM_CAP  (default 3_000_000) last-resort forward attempt
        let cap0: usize = std::env::var("AM_CAP0").ok().and_then(|v| v.parse().ok()).unwrap_or(50_000);
        let cap: usize = std::env::var("AM_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(3_000_000);
        let dbg = std::env::var("AM_DEBUG2").is_ok();
        let brz = |c: usize| -> Option<Dfa> {
            let r1 = nfa.reversed().determinize_capped(c)?;
            let r2 = r1.as_nfa().reversed().determinize_capped(c)?;
            Some(r2)
        };
        crate::progress::phase("forward", var);
        let det = if let Some(d) = nfa.determinize_capped(cap0) { d }
            else if let Some(d) = { crate::progress::phase("brzozowski", var);
                                    brz(cap0.saturating_mul(4).max(200_000)) } {
                if dbg { eprintln!("    exists({}): forward > {} subsets; Brzozowski(small) ok, {} states", var, cap0, d.nstates); }
                d }
            else if let Some(d) = { crate::progress::phase("forward", var);
                                    nfa.determinize_capped(cap) } {
                if dbg { eprintln!("    exists({}): Brzozowski(small) failed; forward(big) ok", var); }
                d }
            else {
                if dbg { eprintln!("    exists({}): forward exceeded {} subsets, trying Brzozowski(big)", var, cap); }
                let big = cap.saturating_mul(4).max(8_000_000);
                crate::progress::phase("brzozowski", var);
                brz(big).expect("forward and reverse determinization both blew up")
            };
        let res = det.zero_closure().minimize();
        crate::progress::states(res.nstates, var);
        if dbg {
            eprintln!("    exists({}): nfa {} states x alpha {} -> det {} -> min {}",
                      var, self.nstates, nalpha, det.nstates, res.nstates);
        }
        res
    }

    /// Universal quantification `A var`, via De Morgan: `!exists var. !self`.
    pub fn forall(&self, var: &str) -> Dfa {
        self.complement().exists(var).complement().minimize()
    }

    /// Re-establish closure under zero padding after an existential projection.
    ///
    /// msd: padding is *leading* zeros, so we must close under removing them,
    ///      L'' = { w : 0^m w in L for some m >= 0 }.  Needs a subset construction.
    /// lsd: padding is *trailing* zeros, so we close under removing those,
    ///      L'' = { w : w 0^m in L for some m >= 0 }.  This only changes the
    ///      accepting set -- no determinization at all.
    pub fn zero_closure(&self) -> Dfa {
        if is_lsd() {
            let mut accept = self.accept.clone();
            // q accepts if delta*(q, 0^m) accepts for some m >= 0
            loop {
                let mut changed = false;
                for q in 0..self.nstates {
                    if !accept[q] && accept[self.t(q, 0)] { accept[q] = true; changed = true; }
                }
                if !changed { break; }
            }
            return Dfa { accept, ..self.clone() };
        }
        let mut init = vec![0u32];
        let mut s = 0usize;
        let mut seen = vec![false; self.nstates];
        seen[0] = true;
        loop {
            let n = self.t(s, 0);
            if seen[n] { break; }
            seen[n] = true;
            init.push(n as u32);
            s = n;
        }
        if init.len() == 1 { return self.clone(); }
        if det_par::fast_enabled() {
            let f = det_par::FlatNfa::from_dfa(self, init.clone());
            let fast = det_par::determinize_capped(&f, usize::MAX).unwrap();
            if det_par::verify() {
                let nfa = Nfa {
                    k: self.k, vars: self.vars.clone(), alpha: self.alpha, nstates: self.nstates,
                    trans: (0..self.nstates * self.alpha).map(|i| vec![self.trans[i]]).collect(),
                    init, accept: self.accept.clone(),
                };
                det_par::assert_same("zero_closure", &nfa.determinize(), &fast);
            }
            return fast;
        }
        let nfa = Nfa {
            k: self.k, vars: self.vars.clone(), alpha: self.alpha, nstates: self.nstates,
            trans: (0..self.nstates * self.alpha).map(|i| vec![self.trans[i]]).collect(),
            init, accept: self.accept.clone(),
        };
        nfa.determinize()
    }

    /// Brzozowski-style determinization: reverse the automaton (as an NFA),
    /// determinize, reverse again, determinize again. Yields a DFA no larger
    /// than any other DFA for the language (often minimal), at the cost of two
    /// subset constructions instead of one; used as a fallback when the direct
    /// forward determinization's subset count exceeds its cap.
    pub fn reverse_determinize(&self) -> Dfa {
        if det_par::fast_enabled() {
            if let Some(r) = det_par::FlatNfa::from_dfa(self, vec![0]).reversed() {
                return det_par::determinize_capped(&r, usize::MAX).unwrap().minimize();
            }
        }
        let mut trans: Vec<Vec<State>> = vec![Vec::new(); self.nstates * self.alpha];
        for st in 0..self.nstates {
            for a in 0..self.alpha {
                let d = self.t(st, a);
                trans[d * self.alpha + a].push(st as u32);
            }
        }
        let init: Vec<State> = (0..self.nstates).filter(|&s| self.accept[s]).map(|s| s as u32).collect();
        let mut accept = vec![false; self.nstates];
        accept[0] = true;
        Nfa { k: self.k, vars: self.vars.clone(), alpha: self.alpha, nstates: self.nstates, trans, init, accept }
            .determinize().minimize()
    }

    /// Remove unreachable states and merge Nerode-equivalent ones (Moore refinement).
    ///
    /// `AM_FAST` swaps the refinement's `HashMap<Vec<u32>>` signature table for a
    /// radix sort ([`crate::det_par::minimize`]); the partition, the numbering and
    /// therefore the output automaton are the same.
    pub fn minimize(&self) -> Dfa {
        if det_par::fast_enabled() {
            let fast = det_par::minimize(self);
            if det_par::verify() { det_par::assert_same("minimize", &self.minimize_ref(), &fast); }
            return fast;
        }
        self.minimize_ref()
    }

    /// Reference minimizer (trim + Moore refinement over a hashed signature).
    pub fn minimize_ref(&self) -> Dfa {
        if self.nstates >= SUBSET_TICK { crate::progress::phase("minimize", ""); }
        // --- trim ---
        let mut map = vec![usize::MAX; self.nstates];
        let mut order = vec![0usize];
        map[0] = 0;
        let mut i = 0;
        while i < order.len() {
            let s = order[i];
            for a in 0..self.alpha {
                let d = self.t(s, a);
                if map[d] == usize::MAX { map[d] = order.len(); order.push(d); }
            }
            i += 1;
        }
        let n = order.len();
        let alpha = self.alpha;
        let mut trans: Vec<u32> = Vec::with_capacity(n * alpha);
        for &s in &order {
            for a in 0..alpha { trans.push(map[self.t(s, a)] as u32); }
        }
        let accept: Vec<bool> = order.iter().map(|&s| self.accept[s]).collect();

        // --- Moore partition refinement ---
        let mut color: Vec<u32> = accept.iter().map(|&b| b as u32).collect();
        let mut ncolors = if accept.iter().any(|&b| b) && accept.iter().any(|&b| !b) { 2 } else { 1 };
        loop {
            let mut sig: FxMap<Vec<u32>, u32> = FxMap::default();
            let mut newcolor = vec![0u32; n];
            let mut key: Vec<u32> = Vec::with_capacity(alpha + 1);
            for s in 0..n {
                key.clear();
                key.push(color[s]);
                for a in 0..alpha { key.push(color[trans[s * alpha + a] as usize]); }
                let next = sig.len() as u32;
                newcolor[s] = match sig.get(&key) { Some(&c) => c, None => { sig.insert(key.clone(), next); next } };
            }
            let nc = sig.len() as u32;
            color = newcolor;
            if nc == ncolors { break; }
            ncolors = nc;
        }
        // rebuild with start = color[0] relabelled to 0
        let m = ncolors as usize;
        let mut relabel = vec![usize::MAX; m];
        relabel[color[0] as usize] = 0;
        let mut cnt = 1;
        for s in 0..n {
            let c = color[s] as usize;
            if relabel[c] == usize::MAX { relabel[c] = cnt; cnt += 1; }
        }
        let mut ntrans = vec![0u32; m * alpha];
        let mut naccept = vec![false; m];
        for s in 0..n {
            let c = relabel[color[s] as usize];
            naccept[c] = accept[s];
            for a in 0..alpha { ntrans[c * alpha + a] = relabel[color[trans[s * alpha + a] as usize] as usize] as u32; }
        }
        Dfa { k: self.k, vars: self.vars.clone(), alpha, nstates: m, trans: ntrans, accept: naccept }
    }
}

impl Nfa {
    /// Subset construction with bitset-encoded state sets.
    ///
    /// A sorted Vec<u32> costs ~4 bytes per member; for the wide NFAs produced by
    /// universal quantification the average subset holds hundreds of states, and
    /// the memory, not the time, is what kills the construction.  A bitset costs
    /// nstates/8 bytes regardless of how full the subset is, which is a 10x saving
    /// at the sizes that matter here.
    pub fn determinize_capped(&self, cap: usize) -> Option<Dfa> {
        let alpha = self.alpha;
        let words = (self.nstates + 63) / 64;
        let mut init = vec![0u64; words];
        for &s in &self.init { init[s as usize / 64] |= 1u64 << (s % 64); }
        let mut index: FxMap<Vec<u64>, u32> = FxMap::default();
        let mut order: Vec<Vec<u64>> = Vec::new();
        index.insert(init.clone(), 0);
        order.push(init);
        let mut trans: Vec<u32> = Vec::new();
        let mut buf = vec![0u64; words];
        let mut i = 0;
        // progress tick: one usize compare per subset when AM_PROGRESS is off
        let mut next_tick = SUBSET_TICK;
        while i < order.len() {
            for a in 0..alpha {
                for w in buf.iter_mut() { *w = 0; }
                // iterate set bits of order[i]
                for wi in 0..words {
                    let mut bits = order[i][wi];
                    while bits != 0 {
                        let b = bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        let s = wi * 64 + b;
                        for &d in &self.trans[s * alpha + a] {
                            buf[d as usize / 64] |= 1u64 << (d % 64);
                        }
                    }
                }
                let id = match index.get(&buf) {
                    Some(&id) => id,
                    None => {
                        let id = order.len() as u32;
                        index.insert(buf.clone(), id);
                        order.push(buf.clone());
                        id
                    }
                };
                trans.push(id);
            }
            i += 1;
            peak_bump(order.len());
            if order.len() >= next_tick {
                next_tick = order.len() + SUBSET_TICK;
                crate::progress::subsets(order.len());
            }
            if order.len() >= cap { return None; }
        }
        let accept = order.iter().map(|set| {
            (0..words).any(|wi| {
                let mut bits = set[wi];
                let mut hit = false;
                while bits != 0 {
                    let b = bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    if self.accept[wi * 64 + b] { hit = true; break; }
                }
                hit
            })
        }).collect();
        Some(Dfa { k: self.k, vars: self.vars.clone(), alpha, nstates: order.len(), trans, accept })
    }

    /// Subset-construct an equivalent DFA from this NFA, with no cap on the
    /// number of subsets explored (see [`Nfa::determinize_capped`]).
    pub fn determinize(&self) -> Dfa {
        self.determinize_capped(usize::MAX).unwrap()
    }

    /// Reverse this NFA (swap initial/accepting, flip arrows).
    pub fn reversed(&self) -> Nfa {
        let mut trans: Vec<Vec<State>> = vec![Vec::new(); self.nstates * self.alpha];
        for s in 0..self.nstates {
            for a in 0..self.alpha {
                for &d in &self.trans[s * self.alpha + a] {
                    trans[d as usize * self.alpha + a].push(s as u32);
                }
            }
        }
        let init: Vec<State> = (0..self.nstates).filter(|&s| self.accept[s]).map(|s| s as u32).collect();
        let mut accept = vec![false; self.nstates];
        for &s in &self.init { accept[s as usize] = true; }
        Nfa { k: self.k, vars: self.vars.clone(), alpha: self.alpha, nstates: self.nstates, trans, init, accept }
    }
}

impl Dfa {
    /// View this DFA as a (trivially deterministic) NFA, for feeding into
    /// NFA-only operations such as [`Nfa::reversed`].
    pub fn as_nfa(&self) -> Nfa {
        Nfa {
            k: self.k, vars: self.vars.clone(), alpha: self.alpha, nstates: self.nstates,
            trans: (0..self.nstates * self.alpha).map(|i| vec![self.trans[i]]).collect(),
            init: vec![0], accept: self.accept.clone(),
        }
    }
}
