//! `learnfe` -- Khodier's guess-and-verify ("self-verifying predicate") construction
//! of the equality-of-factors automaton.
//!
//!     FE(i,j,l)  :=  A t. t < l => T[i+t] = T[j+t]
//!
//! The direct construction quantifies universally over `t`, which forces a subset
//! construction over a product whose intermediate size is the documented pathology
//! (Khodier 2026, Open Problem 1; Walnut peaks at 3.2e8 states on Tribonacci).  Here
//! we never build that intermediate at all.  Instead we *guess* the automaton by
//! active learning from a cheap concrete oracle, and then *verify* the guess against
//! a recurrence whose solution is unique.
//!
//! # The recurrence and why it pins FE down
//!
//! Claim.  Let H : N^3 -> {true,false} be any predicate satisfying, for all i,j,l,
//!
//!     (R)   H(i,j,l)  <=>  ( l = 0  or  ( T[i] = T[j]  and  H(i+1, j+1, l-1) ) ).
//!
//! Then H = FE.
//!
//! Proof.  Induction on l, uniformly in (i,j).
//!   Base l = 0.  The right-hand side of (R) is true, so H(i,j,0) = true for every
//!   i,j; and FE(i,j,0) = "A t. t < 0 => ..." is vacuously true.  So they agree.
//!   Step.  Let l >= 1 and assume H(i',j',l-1) = FE(i',j',l-1) for all i',j'.  The
//!   disjunct `l = 0` of (R) is false, so
//!       H(i,j,l) = ( T[i]=T[j] and H(i+1,j+1,l-1) )
//!                = ( T[i]=T[j] and FE(i+1,j+1,l-1) )        (induction hypothesis)
//!                = ( T[i]=T[j] and A t. t < l-1 => T[i+1+t] = T[j+1+t] )
//!                = ( A t. t < l => T[i+t] = T[j+t] )        (split off t = 0)
//!                = FE(i,j,l).
//!   Since every l is reached, H = FE.  QED
//!
//! FE itself satisfies (R) (same split-off-t=0 computation read backwards), so (R)
//! has exactly one solution, namely FE.  Consequently: *any* candidate automaton that
//! passes the (R) check is FE, no matter where it came from.  The learner may be
//! heuristic, its membership oracle may even lie -- a wrong guess can only cost extra
//! iterations, never correctness.  That is the whole point of "self-verifying".
//!
//! To keep the check inside the engine's Presburger fragment we avoid the subtraction
//! `l-1` by reindexing, and split (R) into the two sentences actually tested:
//!
//!     (C1)  A i,j.    H(i,j,0)
//!     (C2)  A i,j,l.  H(i,j,l+1) <=> ( T[i]=T[j] & H(i+1,j+1,l) )
//!
//! (C1)&(C2) is equivalent to (R): (C2) is (R) at l+1 for every l >= 0, i.e. (R) at
//! every l >= 1, and (C1) is (R) at l = 0.
//!
//! Neither sentence is checked with `forall`, which would run a complement/subset/
//! complement sandwich.  Both are of the shape "A vars. Phi", so we compile only the
//! open formula Phi and ask whether its (already trimmed and minimised) DFA has a
//! non-accepting state.  If it does, breadth-first search from the start state yields
//! a SHORTEST word reaching each such state -- a concrete counterexample tuple, and
//! short counterexamples are exactly what the learner wants.
//!
//! # Counterexamples from a violated recurrence
//!
//! A witness (i,j,l) of ~(C2) is a point where the *recurrence* fails, not directly a
//! point where H differs from FE.  But FE satisfies the recurrence, so if both
//! H(i,j,l+1) = FE(i,j,l+1) and H(i+1,j+1,l) = FE(i+1,j+1,l) held, the recurrence
//! would hold at (i,j,l).  Hence at least one of the two tuples (i,j,l+1),
//! (i+1,j+1,l) is a genuine counterexample; we test both with the membership oracle
//! and feed whichever differs.  A witness (i,j) of ~(C1) gives the counterexample
//! (i,j,0) directly, since FE(i,j,0) is true.
//!
//! # Membership oracle
//!
//! FE(i,j,l) <=> l <= LCP(i,j), where LCP(i,j) is the length of the longest common
//! prefix of the suffixes of T at i and at j.  LCP is computed by walking two base-k
//! counters through the DFAO in lockstep (amortised O(1) per step, no prefix array,
//! O(log n) memory), stopping at the first mismatch, and is memoised per pair.  A hard
//! step cap (AM_LEARN_LCP, default 2^22) bounds the work; a pair that survives the cap
//! is treated as having LCP = infinity.  That can be wrong -- for eventually periodic T
//! it certainly is -- and it cannot break correctness, by the self-verification argument
//! above.  It can only cost convergence, and it does: the capped language is not the
//! language of a small automaton, so a cap that is too tight makes the hypothesis grow
//! straight past the true state count (measured on k3m3-artefact-b: true answer 71
//! states, still climbing through 178 at a 2^18 cap, instant at 2^22).  A stall is
//! detected and answered by raising the cap 16x and relearning from scratch, up to
//! AM_LEARN_LCP_MAX (default 2^26).
//!
//! # Learner
//!
//! Kearns-Vazirani discrimination tree over the k^3-letter track alphabet, with
//! Rivest-Schapire binary-search counterexample decomposition (O(log |w|) membership
//! queries per counterexample) and *incremental* hypothesis maintenance: splitting a
//! leaf re-sifts only the transitions that pointed at it, one query each, instead of
//! rebuilding the whole hypothesis.  Equivalence queries are expensive (they build
//! automata), so before each one we (a) run a cheap randomised counterexample search
//! concentrated on the language boundary l = LCP(i,j), (b) crawl the neighbourhood of
//! every counterexample found, and (c) harvest one shortest counterexample per
//! *rejecting state* of the verification automaton rather than one per round.  Without
//! these, Kearns-Vazirani needs one equivalence query per state; with them the count
//! runs at roughly one per 13 states (1877 states in 146 equivalence queries).

use crate::dfa::{self, Dfa, FxMap};
use crate::dfao::Dfao;
use crate::numsys::{self, NumSys};
use std::sync::Arc;
use crate::logic::{compile_str, Defs};
use std::time::Instant;

// ------------------------------------------------------------------ encoding

/// Number of base-k digits of v (at least 1).
fn ndigits(mut v: u64, k: u64) -> usize {
    let mut n = 1;
    while v >= k { v /= k; n += 1; }
    n
}

/// Encode a tuple of values as a word over the product alphabet, in the ACTIVE
/// digit order and numeration system (see [`crate::numsys::encode_word`]).
pub fn encode(k: usize, vals: &[u64]) -> Vec<usize> { numsys::encode_word(k, vals) }

/// Inverse of `encode`.  `None` if the word is not a tuple of valid
/// representations, or denotes a value beyond u64.
pub fn decode(k: usize, n: usize, w: &[usize]) -> Option<Vec<u64>> { numsys::decode_word(k, n, w) }

// ------------------------------------------------------------------ oracle

/// A counter that carries the DFAO state along its msd digit path, so that
/// stepping `n -> n+1` costs O(1) amortised instead of O(log n).
///
/// Two modes.  Without a numeration system the digits are plain base-k and the
/// successor is the schoolbook carry (`inc_base`).  With one, the digits are the
/// canonical representation and the successor is "the next word of this width in
/// the validity language" (`inc_ns`), found from the counting table -- the same
/// rank/unrank machinery that defines the value of a word in the first place.
/// The two are separate methods, and `walk` picks one *outside* its loop: this is
/// the hottest loop in the engine and a per-step branch on the numeration system
/// costs it ~1.8x.
struct Counter<'a> {
    d: &'a Dfao,
    digits: Vec<u8>,     // msd-first, fixed width, leading zeros allowed
    st: Vec<u32>,        // st[p] = DFAO state after digits[0..p]; st[0] = 0
    vst: Vec<u32>,       // validity states, only used in numeration-system mode
    dg: Vec<usize>,      // scratch for inc_ns, kept allocated
}

impl<'a> Counter<'a> {
    fn new(d: &'a Dfao, n: u64, width: usize, ns: Option<&NumSys>) -> Counter<'a> {
        let mut digits = vec![0u8; width];
        match ns {
            None => {
                let k = d.k as u64;
                let mut v = n;
                for p in (0..width).rev() { digits[p] = (v % k) as u8; v /= k; }
            }
            Some(nsy) => {
                let r = nsy.rep(n);
                let off = width - r.len();
                for (i, &dg) in r.iter().enumerate() { digits[off + i] = dg as u8; }
            }
        }
        let mut st = vec![0u32; width + 1];
        for p in 0..width { st[p + 1] = d.t(st[p] as usize, digits[p] as usize) as u32; }
        let dg: Vec<usize> = digits.iter().map(|&x| x as usize).collect();
        let vst = match ns { None => Vec::new(), Some(nsy) => nsy.vstates(&dg) };
        Counter { d, digits, st, vst, dg }
    }
    #[inline(always)] fn out(&self) -> u8 { self.d.out[*self.st.last().unwrap() as usize] }
    #[inline(always)] fn resync(&mut self, p: usize) {
        let w = self.digits.len();
        for q in p..w { self.st[q + 1] = self.d.t(self.st[q] as usize, self.digits[q] as usize) as u32; }
    }
    #[inline(always)] fn inc_base(&mut self) {
        let k = self.d.k as u8;
        let w = self.digits.len();
        let mut p = w - 1;
        loop {
            if self.digits[p] + 1 < k { self.digits[p] += 1; break; }
            self.digits[p] = 0;
            if p == 0 { break; }          // width was chosen so this cannot matter
            p -= 1;
        }
        self.resync(p);
    }
    #[inline(always)] fn inc_ns(&mut self, ns: &NumSys) {
        let w = self.digits.len();
        let p = ns.succ(&mut self.dg, &mut self.vst).unwrap_or(0);
        for i in p..w { self.digits[i] = self.dg[i] as u8; }
        self.resync(p);
    }
}

/// Membership-query oracle over the sequence `d`, computing (capped) longest
/// common prefix lengths between suffixes as the primitive query the learner's
/// equivalence/counterexample search is built from. Caches results by `(i,j)`
/// pair so repeated queries during learning are O(1) after the first.
pub struct Oracle<'a> {
    d: &'a Dfao,
    k: usize,
    lcp: FxMap<(u64, u64), (u64, u64)>,   // (i<=j) -> (cap used, min(LCP,cap))
    pub mqs: u64,
    pub steps: u64,
    pub hardcap: u64,
    pub assumed_inf: u64,
    pub overflow: u64,
}

impl<'a> Oracle<'a> {
    /// Build an oracle over `d`; `hardcap` bounds any single LCP walk so a
    /// runaway comparison cannot make the learner hang.
    pub fn new(d: &'a Dfao, hardcap: u64) -> Oracle<'a> {
        Oracle { d, k: d.k, lcp: FxMap::default(), mqs: 0, steps: 0, hardcap,
                 assumed_inf: 0, overflow: 0 }
    }

    /// min(LCP(i,j), cap), by lockstep walk with early exit.
    fn walk(&mut self, i: u64, j: u64, cap: u64) -> u64 {
        if i == j || cap == 0 { return cap; }
        let hi = i.max(j).saturating_add(cap);
        let ns = numsys::active();
        let width = match &ns {
            Some(n) => n.replen(hi),
            None => ndigits(hi, self.k as u64),
        };
        let mut a = Counter::new(self.d, i, width, ns.as_deref());
        let mut b = Counter::new(self.d, j, width, ns.as_deref());
        let mut t = 0u64;
        match &ns {
            None => {
                while t < cap {
                    if a.out() != b.out() { self.steps += t; return t; }
                    a.inc_base(); b.inc_base(); t += 1;
                }
            }
            Some(n) => {
                while t < cap {
                    if a.out() != b.out() { self.steps += t; return t; }
                    a.inc_ns(n); b.inc_ns(n); t += 1;
                }
            }
        }
        self.steps += cap;
        cap
    }

    /// min(LCP(i,j), cap), memoised.  `cap` is silently clamped to the hard cap.
    pub fn lcp_upto(&mut self, i: u64, j: u64, cap: u64) -> u64 {
        if i == j { return cap; }
        let cap = cap.min(self.hardcap);
        let key = if i < j { (i, j) } else { (j, i) };
        if let Some(&(cu, v)) = self.lcp.get(&key) {
            if v < cu { return v.min(cap); }        // exact LCP known
            if cap <= cu { return cap; }            // LCP >= cu >= cap
        }
        let v = self.walk(i, j, cap);
        let e = self.lcp.entry(key).or_insert((0, 0));
        if cap > e.0 || v < e.1 { *e = (cap, v); }
        v
    }

    /// FE(i,j,l) = (l <= LCP(i,j)).
    pub fn fe(&mut self, i: u64, j: u64, l: u64) -> bool {
        self.mqs += 1;
        if l == 0 || i == j { return true; }
        if l > self.hardcap {
            let v = self.lcp_upto(i, j, self.hardcap);
            if v < self.hardcap { return false; }
            self.assumed_inf += 1;                  // LCP survived the cap: assume infinite
            return true;
        }
        self.lcp_upto(i, j, l) >= l
    }

    /// Membership query on a 3-track word.
    pub fn mq(&mut self, w: &[usize]) -> bool {
        match decode(self.k, 3, w) {
            Some(v) => self.fe(v[0], v[1], v[2]),
            None => { self.overflow += 1; self.mqs += 1; false }
        }
    }
}

// ------------------------------------------------------------------ learner

enum Node {
    Leaf { access: Vec<usize>, state: usize },
    Inner { suffix: Vec<usize>, yes: usize, no: usize },
}

struct Learner<'a> {
    or: Oracle<'a>,
    k: usize,
    alpha: usize,
    nodes: Vec<Node>,
    incoming: Vec<Vec<u32>>,      // per node: transition slots currently sifting into it
    root: usize,
    access: Vec<Vec<usize>>,      // per state
    accept: Vec<bool>,            // per state
    leaf_of: Vec<usize>,          // per state: its leaf node
    trans: Vec<u32>,              // nstates * alpha
}

impl<'a> Learner<'a> {
    fn new(d: &'a Dfao, hardcap: u64) -> Learner<'a> {
        let k = d.k;
        let alpha = k * k * k;
        let mut or = Oracle::new(d, hardcap);
        let acc0 = or.mq(&[]);
        let mut l = Learner {
            or, k, alpha,
            nodes: vec![Node::Leaf { access: vec![], state: 0 }],
            incoming: vec![Vec::new()],
            root: 0,
            access: vec![vec![]],
            accept: vec![acc0],
            leaf_of: vec![0],
            trans: vec![0u32; alpha],
        };
        for a in 0..alpha { l.incoming[0].push(a as u32); }
        l
    }

    fn nstates(&self) -> usize { self.access.len() }

    fn sift(&mut self, w: &[usize]) -> usize {
        let mut n = self.root;
        loop {
            match &self.nodes[n] {
                Node::Leaf { .. } => return n,
                Node::Inner { suffix, yes, no } => {
                    let (y, nn) = (*yes, *no);
                    let mut q = Vec::with_capacity(w.len() + suffix.len());
                    q.extend_from_slice(w);
                    q.extend_from_slice(suffix);
                    n = if self.or.mq(&q) { y } else { nn };
                }
            }
        }
    }

    fn leaf_state(&self, n: usize) -> usize {
        match &self.nodes[n] { Node::Leaf { state, .. } => *state, _ => unreachable!() }
    }

    fn run(&self, w: &[usize]) -> bool {
        let mut s = 0usize;
        for &a in w { s = self.trans[s * self.alpha + a] as usize; }
        self.accept[s]
    }

    /// Split `leaf` with distinguishing suffix `suffix`, adding a state with access
    /// string `newaccess`.  Only the transitions that pointed at `leaf` are re-sifted,
    /// and each needs exactly one membership query (the new suffix).
    fn split(&mut self, leaf: usize, newaccess: Vec<usize>, suffix: Vec<usize>) {
        let (oldaccess, oldstate) = match &self.nodes[leaf] {
            Node::Leaf { access, state } => (access.clone(), *state),
            _ => unreachable!(),
        };
        let mut q = oldaccess.clone(); q.extend_from_slice(&suffix);
        let bold = self.or.mq(&q);
        let mut q = newaccess.clone(); q.extend_from_slice(&suffix);
        let bnew = self.or.mq(&q);
        debug_assert!(bold != bnew, "split with a non-distinguishing suffix");

        let newstate = self.nstates();
        let idx_old = self.nodes.len();
        self.nodes.push(Node::Leaf { access: oldaccess, state: oldstate });
        self.incoming.push(Vec::new());
        let idx_new = self.nodes.len();
        self.nodes.push(Node::Leaf { access: newaccess.clone(), state: newstate });
        self.incoming.push(Vec::new());
        let (yes, no) = if bold { (idx_old, idx_new) } else { (idx_new, idx_old) };
        let inc = std::mem::take(&mut self.incoming[leaf]);
        self.nodes[leaf] = Node::Inner { suffix: suffix.clone(), yes, no };
        self.leaf_of[oldstate] = idx_old;

        self.access.push(newaccess.clone());
        let acc = self.or.mq(&newaccess);
        self.accept.push(acc);
        self.leaf_of.push(idx_new);
        self.trans.resize((newstate + 1) * self.alpha, 0);

        // redistribute the transitions that used to land in `leaf`
        for slot in inc {
            let s = slot as usize / self.alpha;
            let a = slot as usize % self.alpha;
            let mut q = self.access[s].clone();
            q.push(a);
            q.extend_from_slice(&suffix);
            let b = self.or.mq(&q);
            let (node, st) = if b == bold { (idx_old, oldstate) } else { (idx_new, newstate) };
            self.trans[slot as usize] = st as u32;
            self.incoming[node].push(slot);
        }
        // outgoing transitions of the new state
        for a in 0..self.alpha {
            let mut q = newaccess.clone();
            q.push(a);
            let lf = self.sift(&q);
            let slot = newstate * self.alpha + a;
            self.trans[slot] = self.leaf_state(lf) as u32;
            self.incoming[lf].push(slot as u32);
        }
    }

    /// Rivest-Schapire counterexample decomposition.  Returns false if `w` is not (or
    /// is no longer) a counterexample.
    fn process_ce(&mut self, w: &[usize]) -> bool {
        let target = self.or.mq(w);
        if self.run(w) == target { return false; }
        let n = w.len();
        if n == 0 { return false; }        // would mean mq(eps) disagrees with itself
        let mut path = Vec::with_capacity(n + 1);
        let mut s = 0usize;
        path.push(0usize);
        for &a in w { s = self.trans[s * self.alpha + a] as usize; path.push(s); }
        // alpha(p) = mq(access(path[p]) . w[p..]);  alpha(0) = target, alpha(n) = !target
        let (mut lo, mut hi) = (0usize, n);
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            let mut q = self.access[path[mid]].clone();
            q.extend_from_slice(&w[mid..]);
            if self.or.mq(&q) == target { lo = mid; } else { hi = mid; }
        }
        let p = lo;
        let mut newaccess = self.access[path[p]].clone();
        newaccess.push(w[p]);
        let suffix = w[p + 1..].to_vec();
        let leaf = self.leaf_of[path[p + 1]];
        self.split(leaf, newaccess, suffix);
        true
    }

    fn to_dfa(&self) -> Dfa {
        // The oracle answers "not a member" for any word that is not a triple of
        // valid representations, so the learned automaton already rejects them;
        // the restriction is a cheap belt-and-braces on states the learner never
        // had to separate.
        let d = Dfa::new(self.k, vec!["i".into(), "j".into(), "l".into()],
                 self.nstates(), self.trans.clone(), self.accept.clone()).minimize();
        numsys::restrict(&d).minimize()
    }
}

// ------------------------------------------------------------------ sampling

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 { if n == 0 { 0 } else { self.next() % n } }
}

/// Errors cluster: if the hypothesis is wrong at (i,j,l) it is usually wrong at the
/// digit-tree neighbours of that triple too.  Given seed triples, do a bounded
/// breadth-first crawl of the neighbourhood, keeping every triple on which the
/// hypothesis and the oracle disagree.  Pure membership queries, so this is nearly
/// free compared with an equivalence query -- it is what keeps the number of
/// equivalence queries from growing like the number of states.
fn local_probe(l: &mut Learner, seeds: &[[u64; 3]], budget: usize) -> Vec<Vec<usize>> {
    let k = l.k as u64;
    let mut seen: std::collections::HashSet<[u64; 3]> = std::collections::HashSet::new();
    let mut queue: Vec<[u64; 3]> = Vec::new();
    let mut out: Vec<Vec<usize>> = Vec::new();
    for &s in seeds { if seen.insert(s) { queue.push(s); } }
    let mut head = 0usize;
    while head < queue.len() && out.len() < budget {
        let [i, j, ll] = queue[head];
        head += 1;
        let w = encode(l.k, &[i, j, ll]);
        let hit = l.run(&w) != l.or.mq(&w);
        if hit { out.push(w); }
        if !hit && head > seeds.len() { continue; }   // only expand around real errors
        // Neighbours are magnitude-preserving on purpose: enlarging i,j,l lengthens the
        // words, and long words mean long distinguishing suffixes, which make every
        // later sift query an expensive long LCP walk.  Cheap locality only.
        let mut nb: Vec<[u64; 3]> = Vec::new();
        for d in 0..7u64 { nb.push([i, j, (ll + d).saturating_sub(3)]); }
        nb.push([i + 1, j + 1, ll]);
        nb.push([i + 1, j + 1, ll.saturating_sub(1)]);
        nb.push([i.saturating_sub(1), j.saturating_sub(1), ll + 1]);
        nb.push([i / k, j / k, ll / k]);
        nb.push([i / k, j / k, ll]);
        for t in nb { if queue.len() < budget * 4 && seen.insert(t) { queue.push(t); } }
    }
    out
}

/// Randomised counterexample search concentrated on the language boundary l = LCP(i,j).
/// Pure membership queries -- no automaton is built -- so this is orders of magnitude
/// cheaper than an exact equivalence query and does most of the learning work.
fn sample_ces(l: &mut Learner, rng: &mut Rng, tries: usize, maxdig: u32, want: usize) -> Vec<Vec<usize>> {
    let k = l.k as u64;
    let mut out: Vec<Vec<usize>> = Vec::new();
    for _ in 0..tries {
        let nd = 1 + rng.below(maxdig as u64) as u32;
        let range = k.saturating_pow(nd);
        let i = rng.below(range);
        let j = match rng.below(4) {
            0 => rng.below(range),
            1 => i + 1 + rng.below(16),
            2 => i + k.saturating_pow(rng.below(nd as u64) as u32) * (1 + rng.below(k)),
            _ => i ^ (1 << rng.below(nd as u64)),
        };
        let cap = range.saturating_mul(2).min(l.or.hardcap).min(1 << 20);
        let lc = l.or.lcp_upto(i, j, cap);
        let cands = [0u64, 1, lc, lc + 1, lc.saturating_sub(1),
                     rng.below(lc + 2), rng.below(range)];
        for &ll in cands.iter() {
            let w = encode(l.k, &[i, j, ll]);
            if l.run(&w) != l.or.mq(&w) {
                out.push(w);
                if out.len() >= want { return out; }
            }
        }
    }
    out
}

// ------------------------------------------------------------------ verification

/// Reachable non-accepting states of `a`, each with a shortest word reaching it.
fn rejecting_witnesses(a: &Dfa, maxw: usize) -> Vec<Vec<usize>> {
    let (prev, psym) = a.bfs_tree();
    let mut order: Vec<usize> = (0..a.nstates).filter(|&s| prev[s] != u32::MAX).collect();
    order.sort_by_key(|&s| {
        let mut n = 0usize; let mut t = s;
        while t != 0 { t = prev[t] as usize; n += 1; }
        n
    });
    let mut out = Vec::new();
    for s in order {
        if a.accept[s] { continue; }
        if let Some(w) = a.word_to(&prev, &psym, s) { out.push(w); }
        if out.len() >= maxw { break; }
    }
    out
}

/// Pull the values of i,j,l out of an accepted word of an automaton whose variable
/// list may be any superset/subset of {i,j,l}.
fn triple_from(a: &Dfa, k: usize, w: &[usize]) -> Option<[u64; 3]> {
    let v = decode(k, a.vars.len(), w)?;
    let mut r = [0u64; 3];
    for (c, name) in ["i", "j", "l"].iter().enumerate() {
        if let Some(p) = a.vars.iter().position(|x| x == name) { r[c] = v[p]; }
    }
    Some(r)
}

/// Check (C1) and (C2).  Ok(()) means the hypothesis IS FE.  Err(v) returns candidate
/// counterexample triples derived from shortest witnesses of the violations.
fn verify(seq: &Dfao, defs: &Defs, hyp: &Dfa, k: usize, maxw: usize)
    -> Result<(), Vec<[u64; 3]>> {
    // Build the two open formulas through the ordinary compiler.  Hand-rolling them
    // out of base::adder was tried and is 2-100x SLOWER: the compiler's `$H(i+1,...)`
    // path binds each argument to a fresh variable with `equal` before projecting, and
    // that ordering keeps the intermediate determinisations far smaller than projecting
    // a successor relation straight out of a product with H.
    let mut d2: Defs = defs.clone();
    d2.insert("__H".to_string(), (vec!["i".into(), "j".into(), "l".into()], hyp.clone()));
    let c1 = compile_str(k, seq, &d2, "$__H(i,j,0)").map_err(|_| Vec::new())?;
    let c2 = compile_str(k, seq, &d2,
        "($__H(i,j,l+1)) <=> ((T[i] = T[j]) & ($__H(i+1,j+1,l)))").map_err(|_| Vec::new())?;
    let mut bad: Vec<[u64; 3]> = Vec::new();
    // (C1)  H(i,j,0)
    for w in rejecting_witnesses(&c1, maxw) {
        if let Some(t) = triple_from(&c1, k, &w) { bad.push([t[0], t[1], 0]); }
    }
    // (C2)  H(i,j,l+1) <=> (T[i]=T[j] & H(i+1,j+1,l))
    for w in rejecting_witnesses(&c2, maxw) {
        if let Some(t) = triple_from(&c2, k, &w) {
            bad.push([t[0], t[1], t[2] + 1]);
            bad.push([t[0] + 1, t[1] + 1, t[2]]);
        }
    }
    if bad.is_empty() { Ok(()) } else { Err(bad) }
}

// ------------------------------------------------------------------ driver

/// Summary counters returned by [`learn`] (or the top-level `learnfe` driver):
/// automaton size reached and how much oracle work it cost, for the `AM_PROGRESS`
/// telemetry and end-of-run reporting.
pub struct LearnStats {
    pub states: usize,
    pub iters: usize,
    pub eqs: usize,
    pub ces: usize,
    pub mqs: u64,
    pub steps: u64,
    pub assumed_inf: u64,
    pub ms: u128,
}

/// Learn FE, escalating the LCP cap if the oracle turns out to be too coarse.
///
/// A capped oracle can answer "LCP = infinity" wrongly, which cannot break correctness
/// (the recurrence check is the judge) but can stall the learner: the verifier keeps
/// producing witnesses that the oracle does not recognise as counterexamples.  When
/// that happens we raise the cap and relearn from scratch -- restarting rather than
/// patching, because the discrimination tree's stored answers were taken under the old
/// oracle and would now be inconsistent.
pub fn learn_fe(seq: &Dfao, defs: &Defs) -> Result<(Dfa, LearnStats), String> {
    let start: u64 = std::env::var("AM_LEARN_LCP").ok().and_then(|v| v.parse().ok())
        .unwrap_or(1 << 22);
    let ceiling: u64 = std::env::var("AM_LEARN_LCP_MAX").ok().and_then(|v| v.parse().ok())
        .unwrap_or(1 << 26);
    let mut cap = start.max(1);
    loop {
        match learn_once(seq, defs, cap) {
            Ok(r) => return Ok(r),
            Err((msg, retry)) => {
                if !retry || cap >= ceiling { return Err(msg); }
                cap = (cap * 16).min(ceiling);
                if std::env::var("AM_LEARN_DEBUG").is_ok() {
                    eprintln!("  [learnfe] stalled ({}); raising AM_LEARN_LCP to {} and restarting", msg, cap);
                }
            }
        }
    }
}

fn learn_once(seq: &Dfao, defs: &Defs, hardcap: u64) -> Result<(Dfa, LearnStats), (String, bool)> {
    let t0 = Instant::now();
    let k = seq.k;
    if k * k * k > crate::dfa::MAX_ALPHA { return Err(("base too large for a 3-track alphabet".into(), false)); }
    let maxdig: u32 = std::env::var("AM_LEARN_DIGITS").ok().and_then(|v| v.parse().ok())
        .unwrap_or(match k { 2 => 22, 3 => 14, 4 => 11, _ => 9 });
    let tries: usize = std::env::var("AM_LEARN_SAMPLES").ok().and_then(|v| v.parse().ok())
        .unwrap_or(4000);
    let maxw: usize = std::env::var("AM_LEARN_WITNESS").ok().and_then(|v| v.parse().ok())
        .unwrap_or(256);
    let maxiter: usize = std::env::var("AM_LEARN_ITERS").ok().and_then(|v| v.parse().ok())
        .unwrap_or(20_000);
    let probe: usize = std::env::var("AM_LEARN_PROBE").ok().and_then(|v| v.parse().ok())
        .unwrap_or(2000);
    let dbg = std::env::var("AM_LEARN_DEBUG").is_ok();

    crate::progress::phase("learn", &format!("lcp cap {}", hardcap));
    let mut l = Learner::new(seq, hardcap);
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let mut iters = 0usize;
    let mut eqs = 0usize;
    let mut ces = 0usize;

    loop {
        // ---- cheap phase: randomised search on the boundary l ~ LCP(i,j)
        loop {
            let want = 64 + l.nstates() / 2;
            let batch = sample_ces(&mut l, &mut rng, tries, maxdig, want);
            if batch.is_empty() { break; }
            let mut progress = false;
            let seeds: Vec<[u64; 3]> = batch.iter()
                .filter_map(|w| decode(k, 3, w).map(|v| [v[0], v[1], v[2]])).collect();
            let mut all = local_probe(&mut l, &seeds, probe);
            all.extend(batch.iter().cloned());
            for w in &all { if l.process_ce(w) { ces += 1; progress = true; } }
            iters += 1;
            if dbg { eprintln!("  [learnfe] sample round: {} ces, {} states, {} mqs",
                               batch.len(), l.nstates(), l.or.mqs); }
            if !progress || iters > maxiter { break; }
        }
        // ---- exact phase: the self-verifying recurrence
        eqs += 1;
        let hyp = l.to_dfa();
        crate::progress::learn(eqs, hyp.nstates, l.or.mqs);
        if dbg { eprintln!("  [learnfe] EQ #{} on {} states", eqs, hyp.nstates); }
        crate::progress::phase("verify", "recurrence C1 & C2");
        let vr = verify(seq, defs, &hyp, k, maxw);
        crate::progress::phase("learn", "counterexamples");
        match vr {
            Ok(()) => {
                let st = LearnStats {
                    states: hyp.nstates, iters, eqs, ces, mqs: l.or.mqs, steps: l.or.steps,
                    assumed_inf: l.or.assumed_inf, ms: t0.elapsed().as_millis(),
                };
                return Ok((hyp, st));
            }
            Err(bad) => {
                if bad.is_empty() { return Err(("verification formula failed to compile".into(), false)); }
                let mut progress = false;
                let mut all = local_probe(&mut l, &bad, probe);
                for t in &bad { all.push(encode(k, t)); }
                for w in &all { if l.process_ce(w) { ces += 1; progress = true; } }
                iters += 1;
                if dbg { eprintln!("  [learnfe] EQ #{} gave {} candidate ces -> {} states",
                                   eqs, bad.len(), l.nstates()); }
                if !progress {
                    return Err((format!("no progress: {} recurrence witnesses, none is a \
                        counterexample to the hypothesis at LCP cap {}", bad.len(), hardcap), true));
                }
                if iters > maxiter {
                    return Err((format!("gave up after {} iterations", iters), false));
                }
            }
        }
    }
}
