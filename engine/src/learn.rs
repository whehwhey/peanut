//! `learn` -- Khodier's guess-and-verify ("self-verifying predicate") construction,
//! generalised from equality-of-factors to a family of predicate classes.
//!
//!     learn NAME fe                        FE(i,j,l)     A t<l.  T[i+t] = T[j+t]
//!     learn NAME rev                       REV(i,j,l)    A t<l.  T[i+t] = T[j+l-1-t]
//!     learn NAME period                    PER(i,l,p)    A t. t+p<l => T[i+t] = T[i+t+p]
//!     learn NAME border                    BOR(i,l,b)    b<=l & A t<b. T[i+t] = T[i+l-b+t]
//!     learn NAME (v1,..,vn) [on:v] init:PHI0 step:PHI1        user-supplied recurrence
//!
//! `learnfe NAME` is `learn NAME fe`.
//!
//! # The pattern
//!
//! Each class is pinned down by a recurrence that (a) has a UNIQUE solution and (b)
//! can be written as finitely many *open* formulas which must hold at every point.
//! A candidate automaton H is guessed by active learning against a concrete oracle
//! and then checked against those formulas: if every one of them compiles to an
//! automaton all of whose reachable states accept, then H is the predicate --
//! whatever the learner did to get there.  No universal quantifier over the inner
//! index t is ever compiled, which is the whole point: that quantifier is what forces
//! the subset construction that blows up (Khodier 2026, Open Problem 1, aspect B).
//!
//! ## Why each recurrence has a unique solution
//!
//! Write T for the sequence.  In each case the argument is the same: induction on one
//! coordinate, uniformly in the others, with the recurrence read as a definition of
//! the value at that coordinate's successor.
//!
//! **FE.**  (C1) H(i,j,0);  (C2) H(i,j,l+1) <=> (T[i]=T[j] & H(i+1,j+1,l)).
//! Base l=0: (C1) says H is true, and FE(i,j,0) is vacuously true.  Step: assume
//! H(.,.,l) = FE(.,.,l) everywhere; then H(i,j,l+1) = T[i]=T[j] & FE(i+1,j+1,l) =
//! FE(i,j,l+1) by splitting off t=0.
//!
//! **REV.**  (C1) H(i,j,0);  (C2) H(i,j,l+1) <=> (T[i]=T[j+l] & H(i+1,j,l)).
//! REV(i,j,l+1) is "A t<=l. T[i+t] = T[j+l-t]".  Splitting off t=0 gives T[i]=T[j+l];
//! the remaining conditions, at t' = t-1, are "A t'<l. T[(i+1)+t'] = T[j+(l-1)-t']",
//! i.e. REV(i+1,j,l).  Note the second index does NOT advance: the window at j is
//! being consumed from its right end.  Base l=0 vacuous, as before.
//!
//! **PERIOD.**  (C1) H(i,0,p);  (C2) H(i,l+1,p) <=> ( l+1<=p | (T[i]=T[i+p] & H(i+1,l,p)) ).
//! PER(i,l+1,p) is "A t. t+p<l+1 => T[i+t]=T[i+t+p]".  If l+1<=p the condition set is
//! empty, so it is true.  Otherwise t=0 contributes T[i]=T[i+p] and t'=t-1 gives
//! "A t'. t'+p<l => T[(i+1)+t']=T[(i+1)+t'+p]", i.e. PER(i+1,l,p).  Induction on l,
//! uniformly in (i,p).
//!
//! **BORDER.**  BOR(i,l,b) := b<=l & A t<b. T[i+t] = T[i+l-b+t] -- the length-b prefix
//! and the length-b suffix of the factor of length l at i agree.  Here the recurrence
//! moves along the diagonal l,b -> l+1,b+1 (which keeps the offset l-b fixed, and the
//! offset is what the comparison actually depends on):
//!     (C1) H(i,l,0);  (C1') H(i,0,b) <=> b=0;
//!     (C2) H(i,l+1,b+1) <=> (T[i+b]=T[i+l] & H(i,l,b)).
//! Splitting off the LAST index t=b-1 of BOR(i,l+1,b+1) gives T[i+b]=T[i+l], and the
//! rest is BOR(i,l,b) (same offset, one shorter).  Induction on min(l,b): if
//! min(l,b)=0 one of the two base sentences applies; otherwise (l,b) = (l'+1,b'+1)
//! with min(l',b') one smaller.  Every (l,b) is reached: from (l-b,0) if b<=l, from
//! (0,b-l) otherwise.
//!
//! **User-supplied.**  `learn NAME (v1..vn) on:v init:PHI0 step:PHI1` checks
//!     (C1) H(v1,..,0,..,vn) <=> PHI0        (v replaced by 0)
//!     (C2) H(v1,..,v+1,..,vn) <=> PHI1      (v replaced by v+1)
//! and is sound -- i.e. (C1)&(C2) has exactly one solution -- exactly when
//!   * PHI0 does not mention $H at all, and
//!   * every occurrence of $H in PHI1 has the recursion coordinate equal to the bare
//!     variable `v` (or the constant 0).
//! Then the value of H on the slice v = L+1 is a function of its values on the slice
//! v = L (and of T and arithmetic), so induction on v determines H everywhere; and the
//! predicate the user has in mind satisfies (C1)&(C2) iff it is that unique solution.
//! Both conditions are checked before anything is learned and refused if violated.
//! Quantifiers are refused in PHI0/PHI1 as well: not for soundness (the verifier would
//! compile them fine) but because the membership oracle for a user-supplied class is
//! the recurrence itself, unrolled, and unrolling cannot evaluate a quantifier.
//!
//! # Membership oracles
//!
//! Direct evaluation on the sequence, never through an automaton.  All four built-in
//! classes reduce to one of two longest-common-extension walks, both of which read the
//! DFAO with counters that carry their state along the digit path, so stepping a
//! position costs O(1) amortised and memory is O(log n) rather than O(n):
//!
//!     LCP(i,j)   = max m with T[i+t] = T[j+t]   for all t<m      (forward/forward)
//!     RLCE(i,e)  = max m with T[i+t] = T[e-t]   for all t<m      (forward/backward)
//!
//!     FE(i,j,l)     <=>  l <= LCP(i,j)
//!     REV(i,j,l)    <=>  l = 0  or  l <= RLCE(i, j+l-1)
//!     PER(i,l,p)    <=>  l <= p  or  l-p <= LCP(i, i+p)
//!     BOR(i,l,b)    <=>  b <= l  and  b <= LCP(i, i+l-b)
//!
//! Both walks are memoised per pair and capped at `AM_LEARN_LCP` steps (default 2^22);
//! a pair that survives the cap is treated as matching forever.  That is wrong for
//! eventually periodic T and cannot make a reported automaton wrong -- the recurrence
//! check is the judge -- but it can stall the learner, which is detected and answered
//! by raising the cap and relearning.  For a user-supplied class the oracle is the
//! recurrence unrolled recursively with memoisation, capped at `AM_LEARN_UNROLL` levels.
//!
//! # Learner
//!
//! Kearns-Vazirani discrimination tree over the k^n-letter track alphabet, with
//! Rivest-Schapire binary-search counterexample decomposition (O(log |w|) membership
//! queries per counterexample) and *incremental* hypothesis maintenance: splitting a
//! leaf re-sifts only the transitions that pointed at it, one query each, instead of
//! rebuilding the whole hypothesis.  Equivalence queries are expensive (they build
//! automata), so before each one we (a) run a cheap randomised counterexample search
//! concentrated on the language boundary of the class at hand, (b) crawl the
//! neighbourhood of every counterexample found, and (c) harvest one shortest
//! counterexample per *rejecting state* of the verification automaton rather than one
//! per round.

use crate::dfa::{self, Dfa, FxMap};
use crate::dfao::Dfao;
use crate::numsys::{self, NumSys};
use crate::logic::{self, Ast, Compiler, Defs, Lin};
use std::collections::HashMap;
use std::time::Instant;

/// Name the hypothesis is registered under while the recurrence is being checked.
/// User templates write `$H`; it is renamed to this so that a user `let H` cannot be
/// captured by accident.
const HOLE: &str = "__H";

// ------------------------------------------------------------------ encoding

/// Number of base-k digits of v (at least 1).
fn ndigits(mut v: u64, k: u64) -> usize {
    let mut n = 1;
    while v >= k { v /= k; n += 1; }
    n
}

/// Encode a tuple of values as a word over the product alphabet, in the ACTIVE
/// digit order and numeration system (see [`crate::numsys::encode_word`]).  The
/// values must be in the automaton's canonical (sorted-variable-name) order.
pub fn encode(k: usize, vals: &[u64]) -> Vec<usize> { numsys::encode_word(k, vals) }

/// Inverse of `encode`.  `None` if the word is not a tuple of valid
/// representations, or denotes a value beyond u64.
pub fn decode(k: usize, n: usize, w: &[usize]) -> Option<Vec<u64>> { numsys::decode_word(k, n, w) }

// ------------------------------------------------------------------ counters

/// A counter that carries the DFAO state along its msd digit path, so that
/// stepping `n -> n+1` (or `n -> n-1`) costs O(1) amortised instead of O(log n).
///
/// Two modes.  Without a numeration system the digits are plain base-k and the
/// successor is the schoolbook carry (`inc_base`).  With one, the digits are the
/// canonical representation and the successor is "the next word of this width in
/// the validity language" (`inc_ns`), found from the counting table -- the same
/// rank/unrank machinery that defines the value of a word in the first place.
/// The two are separate methods, and the walks pick one *outside* their loop: this is
/// the hottest loop in the engine and a per-step branch on the numeration system
/// costs it ~1.8x.
///
/// `val` is maintained only by the decrementing methods (`dec_base`/`dec_ns`), which
/// the reversed-factor walk uses; the incrementing ones leave it alone.
struct Counter<'a> {
    d: &'a Dfao,
    digits: Vec<u8>,     // msd-first, fixed width, leading zeros allowed
    st: Vec<u32>,        // st[p] = DFAO state after digits[0..p]; st[0] = 0
    vst: Vec<u32>,       // validity states, only used in numeration-system mode
    dg: Vec<usize>,      // scratch for inc_ns, kept allocated
    val: u64,            // current value (kept up to date by dec_* only)
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
        Counter { d, digits, st, vst, dg, val: n }
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
    /// `n -> n-1` in base k: schoolbook borrow.  Caller guarantees `val > 0`.
    #[inline(always)] fn dec_base(&mut self) {
        let k = self.d.k as u8;
        let w = self.digits.len();
        let mut p = w - 1;
        while self.digits[p] == 0 { self.digits[p] = k - 1; p -= 1; }
        self.digits[p] -= 1;
        self.val -= 1;
        self.resync(p);
    }
    /// `n -> n-1` under a numeration system.  There is no in-place predecessor in
    /// `numsys` (only `succ`), so this re-derives the representation from the value:
    /// O(width) per step rather than O(1) amortised, which is why the reversed-factor
    /// walk is slower under `numsys` than in base k.  The DFAO state path is still
    /// only resynced from the leftmost digit that actually changed.
    #[inline(always)] fn dec_ns(&mut self, ns: &NumSys) {
        let w = self.digits.len();
        self.val -= 1;
        let r = ns.rep(self.val);
        let off = w - r.len();
        let mut p = w;
        for i in 0..w {
            let nd = if i < off { 0u8 } else { r[i - off] as u8 };
            if nd != self.digits[i] { if p == w { p = i; } self.digits[i] = nd; }
        }
        if p < w { self.resync(p); }
    }
}

// ------------------------------------------------------------------ predicate classes

/// Which self-verifying predicate class is being learned.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Fe, Rev, Period, Border, Custom }

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Fe => "fe", Kind::Rev => "rev", Kind::Period => "period",
            Kind::Border => "border", Kind::Custom => "custom",
        }
    }
    /// Parse the kind word of a `learn NAME <kind>` command.
    pub fn parse(s: &str) -> Option<Kind> {
        match s {
            "fe" | "FE" | "eq" => Some(Kind::Fe),
            "rev" | "REV" | "reverse" | "reversed" => Some(Kind::Rev),
            "period" | "PERIOD" | "per" => Some(Kind::Period),
            "border" | "BORDER" | "bor" => Some(Kind::Border),
            _ => None,
        }
    }
}

/// Everything the driver needs to learn and verify one predicate: its parameter list
/// (in call order), the sentences that must hold identically, and -- for a
/// user-supplied class -- the two halves of the recurrence, kept as ASTs so the
/// membership oracle can unroll them.
pub struct Spec {
    pub kind: Kind,
    /// Parameter names in call order, e.g. `["i","l","b"]`.
    pub params: Vec<String>,
    /// The same names sorted: the automaton's canonical track order.
    pub sorted: Vec<String>,
    /// `pos[d]` is the track (sorted) index of parameter `d`.
    pub pos: Vec<usize>,
    /// Open formulas that must be true at every point, with `$H` renamed to `HOLE`.
    pub sentences: Vec<(String, Ast)>,
    /// Index (in `params`) of the coordinate the induction runs on.
    pub rec: usize,
    /// User-supplied classes only: the base and step right-hand sides.
    pub init: Option<Ast>,
    pub step: Option<Ast>,
}

fn parse_formula(src: &str, word: &str) -> Result<Ast, String> {
    let toks = logic::lex(src)?;
    let mut a = logic::Parser::new(toks, word).parse()?;
    rename_calls(&mut a, "H", HOLE);
    Ok(a)
}

fn mk_spec_ast(kind: Kind, params: Vec<String>, rec: usize, sentences: Vec<(String, Ast)>) -> Spec {
    let mut sorted = params.clone();
    sorted.sort();
    let pos: Vec<usize> = params.iter()
        .map(|p| sorted.iter().position(|q| q == p).unwrap()).collect();
    Spec { kind, params, sorted, pos, sentences, rec, init: None, step: None }
}

fn mk_spec(kind: Kind, params: &[&str], rec: usize, sents: &[&str]) -> Result<Spec, String> {
    let params: Vec<String> = params.iter().map(|s| s.to_string()).collect();
    let mut sentences = Vec::new();
    for s in sents { sentences.push((s.to_string(), parse_formula(s, "T")?)); }
    Ok(mk_spec_ast(kind, params, rec, sentences))
}

impl Spec {
    /// The recurrence for one of the built-in classes.  The formulas here are exactly
    /// the (C1)/(C2) sentences proved unique in the module header.
    pub fn builtin(kind: Kind) -> Result<Spec, String> {
        match kind {
            Kind::Fe => mk_spec(Kind::Fe, &["i", "j", "l"], 2, &[
                "$H(i,j,0)",
                "($H(i,j,l+1)) <=> ((T[i] = T[j]) & ($H(i+1,j+1,l)))"]),
            Kind::Rev => mk_spec(Kind::Rev, &["i", "j", "l"], 2, &[
                "$H(i,j,0)",
                "($H(i,j,l+1)) <=> ((T[i] = T[j+l]) & ($H(i+1,j,l)))"]),
            Kind::Period => mk_spec(Kind::Period, &["i", "l", "p"], 1, &[
                "$H(i,0,p)",
                "($H(i,l+1,p)) <=> ((l+1 <= p) | ((T[i] = T[i+p]) & ($H(i+1,l,p))))"]),
            Kind::Border => mk_spec(Kind::Border, &["i", "l", "b"], 1, &[
                "$H(i,l,0)",
                "($H(i,0,b)) <=> (b = 0)",
                "($H(i,l+1,b+1)) <=> ((T[i+b] = T[i+l]) & ($H(i,l,b)))"]),
            Kind::Custom => Err("custom kind needs init:/step:".into()),
        }
    }

    /// A user-supplied recurrence.  `recname` is the coordinate the induction runs on;
    /// `init` is the right-hand side at `recname = 0` and `step` the right-hand side at
    /// `recname + 1`.  Refuses anything the uniqueness argument in the module header
    /// does not cover.
    pub fn custom(params: Vec<String>, recname: &str, init_src: &str, step_src: &str, word: &str)
        -> Result<Spec, String> {
        if params.len() < 1 { return Err("need at least one parameter".into()); }
        if params.len() > 4 { return Err("at most 4 parameters (k^n alphabet)".into()); }
        let rec = params.iter().position(|p| p == recname)
            .ok_or_else(|| format!("recursion variable {:?} is not a parameter", recname))?;
        {
            let mut seen = params.clone(); seen.sort(); seen.dedup();
            if seen.len() != params.len() { return Err("duplicate parameter name".into()); }
        }
        let init = parse_formula(init_src, word)?;
        let step = parse_formula(step_src, word)?;
        // Soundness side conditions (module header, "User-supplied").
        if has_quantifier(&init) || has_quantifier(&step) {
            return Err("quantifiers are not allowed in init:/step: -- the oracle unrolls \
                        the recurrence and cannot evaluate one".into());
        }
        let mut calls = Vec::new();
        collect_calls(&init, HOLE, &mut calls);
        if !calls.is_empty() {
            return Err("init: must not mention $H (it is the base case)".into());
        }
        collect_calls(&step, HOLE, &mut calls);
        for args in &calls {
            if args.len() != params.len() {
                return Err(format!("$H takes {} arguments, got {}", params.len(), args.len()));
            }
            let a = &args[rec];
            let ok = a.is_plain_var().as_deref() == Some(recname)
                || (a.c == 0 && a.coef.is_empty());
            if !ok {
                return Err(format!("every $H in step: must use {0} (or 0) as its {1}-th \
                    argument, so that H at {0}+1 depends only on H at {0}; \
                    otherwise the recurrence need not have a unique solution",
                    recname, rec + 1));
            }
        }
        // Assemble the two sentences.  `init` is read at the base point, so any
        // mention of the recursion variable in it is replaced by 0 -- "init: l=0" and
        // "init: true" then mean the same thing, which is what the phrase "the formula
        // for l = 0" says.
        let mut init = init;
        subst_zero(&mut init, recname);
        let args = |plus1: bool| -> Vec<Lin> {
            params.iter().enumerate().map(|(d, p)| {
                if d != rec { Lin::var(p) }
                else if plus1 { Lin::var(p).add(&Lin::num(1), 1) }
                else { Lin::num(0) }
            }).collect()
        };
        let disp = |v: &str| -> String {
            params.iter().enumerate()
                .map(|(d, p)| if d == rec { v.to_string() } else { p.clone() })
                .collect::<Vec<_>>().join(",")
        };
        let s0 = Ast::Iff(Box::new(Ast::Call(HOLE.to_string(), args(false))),
                          Box::new(init.clone()));
        let s1 = Ast::Iff(Box::new(Ast::Call(HOLE.to_string(), args(true))),
                          Box::new(step.clone()));
        let sents = vec![
            (format!("($H({})) <=> ({})", disp("0"), init_src), s0),
            (format!("($H({})) <=> ({})", disp(&format!("{}+1", recname)), step_src), s1)];
        let mut sp = mk_spec_ast(Kind::Custom, params, rec, sents);
        sp.init = Some(init);
        sp.step = Some(step);
        Ok(sp)
    }

    fn n(&self) -> usize { self.params.len() }
}

// ------------------------------------------------------------------ AST utilities

fn walk<'a>(a: &'a Ast, f: &mut dyn FnMut(&'a Ast)) {
    f(a);
    match a {
        Ast::Not(x) => walk(x, f),
        Ast::And(x, y) | Ast::Or(x, y) | Ast::Imp(x, y) | Ast::Iff(x, y) => { walk(x, f); walk(y, f) }
        Ast::Forall(_, x) | Ast::Exists(_, x) => walk(x, f),
        _ => {}
    }
}

fn walk_mut(a: &mut Ast, f: &mut dyn FnMut(&mut Ast)) {
    f(a);
    match a {
        Ast::Not(x) => walk_mut(x, f),
        Ast::And(x, y) | Ast::Or(x, y) | Ast::Imp(x, y) | Ast::Iff(x, y) => { walk_mut(x, f); walk_mut(y, f) }
        Ast::Forall(_, x) | Ast::Exists(_, x) => walk_mut(x, f),
        _ => {}
    }
}

fn rename_calls(a: &mut Ast, from: &str, to: &str) {
    walk_mut(a, &mut |n| { if let Ast::Call(nm, _) = n { if nm == from { *nm = to.to_string(); } } });
}

/// Argument lists of every `$name(...)` in `a`, in syntax order.
fn collect_calls(a: &Ast, name: &str, out: &mut Vec<Vec<Lin>>) {
    walk(a, &mut |n| { if let Ast::Call(nm, args) = n { if nm == name { out.push(args.clone()); } } });
}

/// Replace every occurrence of variable `v` by the constant 0 in every linear term.
fn subst_zero(a: &mut Ast, v: &str) {
    fn lin(t: &mut Lin, v: &str) { t.coef.remove(v); }
    walk_mut(a, &mut |n| match n {
        Ast::Cmp(x, _, y) | Ast::SeqSeq(x, _, y) => { lin(x, v); lin(y, v) }
        Ast::SeqLetter(x, _, _) | Ast::IsPow(x) => lin(x, v),
        Ast::Call(_, args) => { for x in args { lin(x, v) } }
        _ => {}
    });
}

fn has_quantifier(a: &Ast) -> bool {
    let mut found = false;
    walk(a, &mut |n| { if matches!(n, Ast::Forall(..) | Ast::Exists(..)) { found = true; } });
    found
}

fn lin_eval(t: &Lin, env: &HashMap<String, i64>) -> Option<i64> {
    let mut v = t.c;
    for (k, c) in &t.coef { v += c * env.get(k).copied()?; }
    Some(v)
}

// ------------------------------------------------------------------ oracle

/// Membership-query oracle: direct evaluation of the predicate on the sequence `d`,
/// never through an automaton.  The two longest-common-extension walks it is built
/// from (forward/forward and forward/backward) are memoised per pair, so repeated
/// queries during learning are O(1) after the first.
pub struct Oracle<'a> {
    d: &'a Dfao,
    k: usize,
    kind: Kind,
    n: usize,
    /// `pos[d]` = track index of display parameter `d` (see [`Spec`]).
    pos: Vec<usize>,
    spec: Option<&'a Spec>,
    defs: Option<&'a Defs>,
    lcp: FxMap<(u64, u64), (u64, u64)>,   // (i<=j) -> (cap used, min(LCP,cap))
    rlce: FxMap<(u64, u64), (u64, u64)>,  // (i,e)  -> (cap used, min(RLCE,cap))
    memo: HashMap<Vec<u64>, bool>,        // user-supplied classes: unrolled values
    pub mqs: u64,
    pub steps: u64,
    pub hardcap: u64,
    pub unroll: u32,
    pub assumed_inf: u64,
    pub overflow: u64,
}

impl<'a> Oracle<'a> {
    /// Build an FE oracle over `d`; `hardcap` bounds any single LCP walk so a
    /// runaway comparison cannot make the learner hang.  (`fe_map` in `main.rs`
    /// uses this directly as the ground truth for the heatmap.)
    pub fn new(d: &'a Dfao, hardcap: u64) -> Oracle<'a> {
        Oracle { d, k: d.k, kind: Kind::Fe, n: 3, pos: vec![0, 1, 2], spec: None, defs: None,
                 lcp: FxMap::default(), rlce: FxMap::default(), memo: HashMap::new(),
                 mqs: 0, steps: 0, hardcap, unroll: 5000, assumed_inf: 0, overflow: 0 }
    }

    fn for_spec(d: &'a Dfao, hardcap: u64, spec: &'a Spec, defs: &'a Defs, unroll: u32) -> Oracle<'a> {
        Oracle { d, k: d.k, kind: spec.kind, n: spec.n(), pos: spec.pos.clone(),
                 spec: Some(spec), defs: Some(defs),
                 lcp: FxMap::default(), rlce: FxMap::default(), memo: HashMap::new(),
                 mqs: 0, steps: 0, hardcap, unroll, assumed_inf: 0, overflow: 0 }
    }

    // -------- forward/forward: longest common prefix of two suffixes

    /// min(LCP(i,j), cap), by lockstep walk with early exit.
    fn walk_lcp(&mut self, i: u64, j: u64, cap: u64) -> u64 {
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
        let v = self.walk_lcp(i, j, cap);
        let e = self.lcp.entry(key).or_insert((0, 0));
        if cap > e.0 || v < e.1 { *e = (cap, v); }
        v
    }

    // -------- forward/backward: longest common extension of a suffix and a reversed prefix

    /// min(RLCE(i,e), cap, e+1), where RLCE(i,e) = max m with T[i+t] = T[e-t] for all
    /// t < m.  One counter walks up from i, the other down from e.
    fn walk_rlce(&mut self, i: u64, e: u64, cap: u64) -> u64 {
        let cap = cap.min(e + 1);
        if cap == 0 { return 0; }
        let ns = numsys::active();
        let hi = i.saturating_add(cap);
        let (wa, wb) = match &ns {
            Some(n) => (n.replen(hi), n.replen(e)),
            None => (ndigits(hi, self.k as u64), ndigits(e, self.k as u64)),
        };
        let mut a = Counter::new(self.d, i, wa, ns.as_deref());
        let mut b = Counter::new(self.d, e, wb, ns.as_deref());
        let mut t = 0u64;
        match &ns {
            None => {
                while t < cap {
                    if a.out() != b.out() { self.steps += t; return t; }
                    t += 1;
                    if t == cap { break; }
                    a.inc_base(); b.dec_base();
                }
            }
            Some(n) => {
                while t < cap {
                    if a.out() != b.out() { self.steps += t; return t; }
                    t += 1;
                    if t == cap { break; }
                    a.inc_ns(n); b.dec_ns(n);
                }
            }
        }
        self.steps += cap;
        cap
    }

    /// min(RLCE(i,e), cap, e+1), memoised.
    pub fn rlce_upto(&mut self, i: u64, e: u64, cap: u64) -> u64 {
        let cap = cap.min(self.hardcap).min(e + 1);
        if cap == 0 { return 0; }
        if let Some(&(cu, v)) = self.rlce.get(&(i, e)) {
            if v < cu { return v.min(cap); }
            if cap <= cu { return cap; }
        }
        let v = self.walk_rlce(i, e, cap);
        let ent = self.rlce.entry((i, e)).or_insert((0, 0));
        if cap > ent.0 || v < ent.1 { *ent = (cap, v); }
        v
    }

    // -------- the four built-in predicates

    /// `FE(i,j,l) = (l <= LCP(i,j))`.
    pub fn fe(&mut self, i: u64, j: u64, l: u64) -> bool {
        if l == 0 || i == j { return true; }
        if l > self.hardcap {
            let v = self.lcp_upto(i, j, self.hardcap);
            if v < self.hardcap { return false; }
            self.assumed_inf += 1;                  // LCP survived the cap: assume infinite
            return true;
        }
        self.lcp_upto(i, j, l) >= l
    }

    /// `REV(i,j,l) = (l = 0 or l <= RLCE(i, j+l-1))`.
    pub fn rev(&mut self, i: u64, j: u64, l: u64) -> bool {
        if l == 0 { return true; }
        let e = match j.checked_add(l - 1) { Some(e) => e, None => return false };
        if l > self.hardcap {
            let v = self.rlce_upto(i, e, self.hardcap);
            if v < self.hardcap { return false; }
            self.assumed_inf += 1;
            return true;
        }
        self.rlce_upto(i, e, l) >= l
    }

    /// `PER(i,l,p) = (l <= p or l-p <= LCP(i,i+p))`.
    pub fn period(&mut self, i: u64, l: u64, p: u64) -> bool {
        if l <= p { return true; }
        let need = l - p;
        let j = match i.checked_add(p) { Some(j) => j, None => return false };
        if need > self.hardcap {
            let v = self.lcp_upto(i, j, self.hardcap);
            if v < self.hardcap { return false; }
            self.assumed_inf += 1;
            return true;
        }
        self.lcp_upto(i, j, need) >= need
    }

    /// `BOR(i,l,b) = (b <= l and b <= LCP(i, i+l-b))`.
    pub fn border(&mut self, i: u64, l: u64, b: u64) -> bool {
        if b > l { return false; }
        if b == 0 { return true; }
        let j = match i.checked_add(l - b) { Some(j) => j, None => return false };
        if b > self.hardcap {
            let v = self.lcp_upto(i, j, self.hardcap);
            if v < self.hardcap { return false; }
            self.assumed_inf += 1;
            return true;
        }
        self.lcp_upto(i, j, b) >= b
    }

    // -------- user-supplied classes: unroll the recurrence

    /// Evaluate a user-supplied H at `v` by unrolling its own recurrence, memoised.
    /// Beyond `unroll` levels the answer is `true` by fiat and counted in
    /// `assumed_inf` -- exactly like an LCP that survives its cap, and harmless for
    /// the same reason (the recurrence check, not the oracle, decides correctness).
    fn custom(&mut self, v: &[u64], depth: u32) -> bool {
        if let Some(&b) = self.memo.get(v) { return b; }
        if depth >= self.unroll { self.assumed_inf += 1; return true; }
        let sp = self.spec.expect("custom oracle without a spec");
        let mut env: HashMap<String, i64> = HashMap::new();
        for (d, p) in sp.params.iter().enumerate() { env.insert(p.clone(), v[d] as i64); }
        let r = if v[sp.rec] == 0 {
            self.eval_ast(sp.init.as_ref().unwrap(), &env, depth)
        } else {
            env.insert(sp.params[sp.rec].clone(), v[sp.rec] as i64 - 1);
            self.eval_ast(sp.step.as_ref().unwrap(), &env, depth)
        };
        if self.memo.len() < (1 << 22) { self.memo.insert(v.to_vec(), r); }
        r
    }

    fn eval_ast(&mut self, a: &Ast, env: &HashMap<String, i64>, depth: u32) -> bool {
        let idx = |t: &Lin| -> Option<u64> {
            lin_eval(t, env).and_then(|v| if v >= 0 { Some(v as u64) } else { None })
        };
        match a {
            Ast::Bool(b) => *b,
            Ast::Not(x) => !self.eval_ast(x, env, depth),
            Ast::And(x, y) => self.eval_ast(x, env, depth) && self.eval_ast(y, env, depth),
            Ast::Or(x, y) => self.eval_ast(x, env, depth) || self.eval_ast(y, env, depth),
            Ast::Imp(x, y) => !self.eval_ast(x, env, depth) || self.eval_ast(y, env, depth),
            Ast::Iff(x, y) => self.eval_ast(x, env, depth) == self.eval_ast(y, env, depth),
            Ast::Cmp(x, r, y) => {
                match (lin_eval(x, env), lin_eval(y, env)) {
                    (Some(a), Some(b)) => match r {
                        logic::Rel::Eq => a == b, logic::Rel::Ne => a != b,
                        logic::Rel::Lt => a < b, logic::Rel::Le => a <= b,
                        logic::Rel::Gt => a > b, logic::Rel::Ge => a >= b,
                    },
                    _ => false,
                }
            }
            Ast::SeqLetter(t, r, c) => {
                let eq = idx(t).map(|x| self.d.at(x) == *c).unwrap_or(false);
                if matches!(r, logic::Rel::Ne) { !eq } else { eq }
            }
            Ast::SeqSeq(t1, r, t2) => {
                let eq = match (idx(t1), idx(t2)) {
                    (Some(x), Some(y)) => self.d.at(x) == self.d.at(y),
                    _ => false,
                };
                if matches!(r, logic::Rel::Ne) { !eq } else { eq }
            }
            Ast::IsPow(t) => {
                match idx(t) {
                    None => false,
                    Some(mut x) => {
                        if x == 0 { return false; }
                        let k = self.k as u64;
                        while x % k == 0 { x /= k; }
                        x == 1
                    }
                }
            }
            Ast::Call(nm, args) => {
                if nm == HOLE {
                    let mut v = Vec::with_capacity(args.len());
                    for t in args { match idx(t) { Some(x) => v.push(x), None => return false } }
                    return self.custom(&v, depth + 1);
                }
                // Any other $NAME is an already-built automaton: run it on the values.
                let Some(defs) = self.defs else { return false };
                let Some((params, aut)) = defs.get(nm) else { return false };
                if params.len() != args.len() { return false; }
                let mut byname: HashMap<&str, u64> = HashMap::new();
                for (p, t) in params.iter().zip(args) {
                    match idx(t) { Some(x) => { byname.insert(p.as_str(), x); }, None => return false }
                }
                let mut vals = Vec::with_capacity(aut.vars.len());
                for v in &aut.vars {
                    match byname.get(v.as_str()) { Some(&x) => vals.push(x), None => return false }
                }
                aut.run(&encode(self.k, &vals))
            }
            Ast::Forall(..) | Ast::Exists(..) => false,   // refused at spec-build time
        }
    }

    /// The predicate itself, on values in parameter (display) order.
    pub fn eval(&mut self, v: &[u64]) -> bool {
        self.mqs += 1;
        match self.kind {
            Kind::Fe => self.fe(v[0], v[1], v[2]),
            Kind::Rev => self.rev(v[0], v[1], v[2]),
            Kind::Period => self.period(v[0], v[1], v[2]),
            Kind::Border => self.border(v[0], v[1], v[2]),
            Kind::Custom => self.custom(v, 0),
        }
    }

    /// Membership query on an n-track word (canonical track order).
    pub fn mq(&mut self, w: &[usize]) -> bool {
        match decode(self.k, self.n, w) {
            Some(v) => {
                let d: Vec<u64> = self.pos.iter().map(|&p| v[p]).collect();
                self.eval(&d)
            }
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
    n: usize,
    kind: Kind,
    pos: Vec<usize>,
    vars: Vec<String>,
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
    fn new(d: &'a Dfao, hardcap: u64, spec: &'a Spec, defs: &'a Defs, unroll: u32) -> Learner<'a> {
        let k = d.k;
        let n = spec.n();
        let alpha = k.pow(n as u32);
        let mut or = Oracle::for_spec(d, hardcap, spec, defs, unroll);
        let acc0 = or.mq(&[]);
        let mut l = Learner {
            or, k, n, kind: spec.kind, pos: spec.pos.clone(), vars: spec.sorted.clone(), alpha,
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

    /// Encode a tuple given in parameter (display) order.
    fn enc(&self, v: &[u64]) -> Vec<usize> {
        let mut s = vec![0u64; self.n];
        for (d, &p) in self.pos.iter().enumerate() { s[p] = v[d]; }
        encode(self.k, &s)
    }

    /// Decode a word back to parameter (display) order.
    fn dec(&self, w: &[usize]) -> Option<Vec<u64>> {
        decode(self.k, self.n, w).map(|v| self.pos.iter().map(|&p| v[p]).collect())
    }

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
        // The oracle answers "not a member" for any word that is not a tuple of
        // valid representations, so the learned automaton already rejects them;
        // the restriction is a cheap belt-and-braces on states the learner never
        // had to separate.
        let d = Dfa::new(self.k, self.vars.clone(),
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

/// A second position "near" `i`, drawn from the four shapes that produce interesting
/// factor comparisons: uniform, adjacent, digit-shifted, and one-digit-perturbed.
fn partner(rng: &mut Rng, i: u64, k: u64, nd: u32, range: u64) -> u64 {
    match rng.below(4) {
        0 => rng.below(range),
        1 => i + 1 + rng.below(16),
        2 => i + k.saturating_pow(rng.below(nd as u64) as u32) * (1 + rng.below(k)),
        _ => i ^ (1 << rng.below(nd as u64)),
    }
}

/// Errors cluster: if the hypothesis is wrong at a tuple it is usually wrong at the
/// digit-tree neighbours of that tuple too.  Given seed tuples, do a bounded
/// breadth-first crawl of the neighbourhood, keeping every tuple on which the
/// hypothesis and the oracle disagree.  Pure membership queries, so this is nearly
/// free compared with an equivalence query -- it is what keeps the number of
/// equivalence queries from growing like the number of states.
///
/// Neighbours are magnitude-preserving on purpose: enlarging the coordinates lengthens
/// the words, and long words mean long distinguishing suffixes, which make every later
/// sift query an expensive long walk.  Cheap locality only.  (Measured on FE
/// `[s2 != 0 mod 5]`, 1877 states: 205 equivalence queries / 29.3 s with no probe,
/// 146 / 19.9 s with this one, no completion in 150 s with a magnitude-growing one.)
fn local_probe(l: &mut Learner, seeds: &[Vec<u64>], budget: usize) -> Vec<Vec<usize>> {
    let k = l.k as u64;
    let kind = l.kind;
    let n = l.n;
    let mut seen: std::collections::HashSet<Vec<u64>> = std::collections::HashSet::new();
    let mut queue: Vec<Vec<u64>> = Vec::new();
    let mut out: Vec<Vec<usize>> = Vec::new();
    for s in seeds { if seen.insert(s.clone()) { queue.push(s.clone()); } }
    let mut head = 0usize;
    while head < queue.len() && out.len() < budget {
        let v = queue[head].clone();
        head += 1;
        let w = l.enc(&v);
        let hit = l.run(&w) != l.or.mq(&w);
        if hit { out.push(w); }
        if !hit && head > seeds.len() { continue; }   // only expand around real errors
        let mut nb: Vec<Vec<u64>> = Vec::new();
        match kind {
            // The FE neighbourhood, unchanged from the FE-only learner.
            Kind::Fe => {
                let (i, j, ll) = (v[0], v[1], v[2]);
                for d in 0..7u64 { nb.push(vec![i, j, (ll + d).saturating_sub(3)]); }
                nb.push(vec![i + 1, j + 1, ll]);
                nb.push(vec![i + 1, j + 1, ll.saturating_sub(1)]);
                nb.push(vec![i.saturating_sub(1), j.saturating_sub(1), ll + 1]);
                nb.push(vec![i / k, j / k, ll / k]);
                nb.push(vec![i / k, j / k, ll]);
            }
            _ => {
                // Generic: perturb the length-like coordinate hardest, then take the
                // step the recurrence itself takes, then divide everything by k.
                let r = match kind { Kind::Rev => 2, Kind::Period | Kind::Border => 1, _ => n - 1 };
                for d in 0..7u64 {
                    let mut u = v.clone();
                    u[r] = (u[r] + d).saturating_sub(3);
                    nb.push(u);
                }
                for c in 0..n {
                    let mut u = v.clone(); u[c] += 1; nb.push(u);
                    let mut u = v.clone(); u[c] = u[c].saturating_sub(1); nb.push(u);
                }
                match kind {
                    Kind::Rev => {
                        nb.push(vec![v[0] + 1, v[1], v[2].saturating_sub(1)]);
                        nb.push(vec![v[0].saturating_sub(1), v[1], v[2] + 1]);
                    }
                    Kind::Period => {
                        nb.push(vec![v[0] + 1, v[1].saturating_sub(1), v[2]]);
                        nb.push(vec![v[0].saturating_sub(1), v[1] + 1, v[2]]);
                    }
                    Kind::Border => {
                        nb.push(vec![v[0], v[1].saturating_sub(1), v[2].saturating_sub(1)]);
                        nb.push(vec![v[0], v[1] + 1, v[2] + 1]);
                    }
                    _ => {}
                }
                nb.push(v.iter().map(|x| x / k).collect());
            }
        }
        for t in nb { if queue.len() < budget * 4 && seen.insert(t.clone()) { queue.push(t); } }
    }
    out
}

/// Randomised counterexample search concentrated on the language boundary of the class
/// being learned -- the surface where one more matching position flips the answer.
/// Pure membership queries: no automaton is built, so this is orders of magnitude
/// cheaper than an exact equivalence query and does most of the learning work.
fn sample_ces(l: &mut Learner, rng: &mut Rng, tries: usize, maxdig: u32, want: usize) -> Vec<Vec<usize>> {
    let k = l.k as u64;
    let mut out: Vec<Vec<usize>> = Vec::new();
    macro_rules! try_tuple {
        ($v:expr) => {{
            let w = l.enc(&$v);
            if l.run(&w) != l.or.mq(&w) {
                out.push(w);
                if out.len() >= want { return out; }
            }
        }};
    }
    for _ in 0..tries {
        let nd = 1 + rng.below(maxdig as u64) as u32;
        let range = k.saturating_pow(nd);
        let i = rng.below(range);
        let cap = range.saturating_mul(2).min(l.or.hardcap).min(1 << 20);
        match l.kind {
            Kind::Fe => {
                let j = partner(rng, i, k, nd, range);
                let lc = l.or.lcp_upto(i, j, cap);
                let cands = [0u64, 1, lc, lc + 1, lc.saturating_sub(1),
                             rng.below(lc + 2), rng.below(range)];
                for &ll in cands.iter() { try_tuple!(vec![i, j, ll]); }
            }
            Kind::Rev => {
                // The boundary is l = RLCE(i,e) with e the last position of the second
                // window, so draw (i,e) and read j = e+1-l back off.
                let e = partner(rng, i, k, nd, range);
                let m = l.or.rlce_upto(i, e, cap);
                let cands = [0u64, 1, m, m + 1, m.saturating_sub(1),
                             rng.below(m + 2), rng.below(range)];
                for &ll in cands.iter() {
                    if ll == 0 { try_tuple!(vec![i, rng.below(range), 0]); continue; }
                    if ll > e + 1 { continue; }
                    try_tuple!(vec![i, e + 1 - ll, ll]);
                }
            }
            Kind::Period => {
                // Boundary: l = p + LCP(i,i+p).
                let p = match rng.below(4) {
                    0 => rng.below(range), 1 => 1 + rng.below(16),
                    2 => k.saturating_pow(rng.below(nd as u64) as u32), _ => 1 + rng.below(range),
                };
                let m = l.or.lcp_upto(i, i.saturating_add(p), cap);
                let b = m.saturating_add(p);
                let cands = [0u64, 1, p, p + 1, b, b + 1, b.saturating_sub(1), rng.below(range)];
                for &ll in cands.iter() { try_tuple!(vec![i, ll, p]); }
            }
            Kind::Border => {
                // Boundary: b = LCP(i, i+d) at fixed offset d = l-b.
                let d = match rng.below(4) {
                    0 => rng.below(range), 1 => 1 + rng.below(16),
                    2 => k.saturating_pow(rng.below(nd as u64) as u32), _ => 1 + rng.below(range),
                };
                let m = l.or.lcp_upto(i, i.saturating_add(d), cap);
                let cands = [0u64, 1, m, m + 1, m.saturating_sub(1), rng.below(m + 2)];
                for &b in cands.iter() {
                    try_tuple!(vec![i, b + d, b]);
                    if b > 0 { try_tuple!(vec![i, b - 1, b]); }   // the b>l false region
                }
            }
            Kind::Custom => {
                let mut v = vec![0u64; l.n];
                for c in 0..l.n {
                    v[c] = match rng.below(4) {
                        0 => rng.below(range), 1 => rng.below(8),
                        2 => i, _ => i + rng.below(16),
                    };
                }
                let cap = (l.or.unroll as u64).saturating_sub(1);
                v[l.spec_rec()] = v[l.spec_rec()].min(cap);
                try_tuple!(v);
            }
        }
    }
    out
}

impl<'a> Learner<'a> {
    fn spec_rec(&self) -> usize {
        self.or.spec.map(|s| s.rec).unwrap_or(self.n - 1)
    }
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

/// Check every sentence of the recurrence.  `Ok(())` means the hypothesis IS the
/// predicate.  `Err(v)` returns candidate counterexample tuples (in parameter order)
/// derived from shortest witnesses of the violations.
///
/// A witness of a violated sentence is a point where the *recurrence* fails, not
/// directly a point where H differs from the predicate.  But the predicate satisfies
/// the recurrence, so if H agreed with it at every H-argument tuple occurring in that
/// sentence, the sentence would hold there.  Hence at least one of those tuples is a
/// genuine counterexample; all of them are handed to the learner, which tests each
/// against the membership oracle and keeps the ones that differ.
fn verify(seq: &Dfao, defs: &Defs, spec: &Spec, hyp: &Dfa, k: usize, maxw: usize)
    -> Result<(), Vec<Vec<u64>>> {
    // The sentences go through the ordinary compiler.  Hand-rolling them out of
    // base::adder was tried and is 2-100x SLOWER: the compiler's `$H(i+1,...)` path
    // binds each argument to a fresh variable with `equal` before projecting, and that
    // ordering keeps the intermediate determinisations far smaller than projecting a
    // successor relation straight out of a product with H.
    let mut d2: Defs = defs.clone();
    d2.insert(HOLE.to_string(), (spec.params.clone(), hyp.clone()));
    let mut bad: Vec<Vec<u64>> = Vec::new();
    for (_src, ast) in &spec.sentences {
        let a = Compiler::new(k, seq, &d2).compile(ast).map_err(|_| Vec::<Vec<u64>>::new())?;
        let mut calls = Vec::new();
        collect_calls(ast, HOLE, &mut calls);
        for w in rejecting_witnesses(&a, maxw) {
            let Some(vals) = decode(k, a.vars.len(), &w) else { continue };
            let mut env: HashMap<String, i64> = HashMap::new();
            for (p, v) in a.vars.iter().zip(vals.iter()) { env.insert(p.clone(), *v as i64); }
            for p in &spec.params { env.entry(p.clone()).or_insert(0); }
            for args in &calls {
                let mut t = Vec::with_capacity(args.len());
                let mut ok = true;
                for x in args {
                    match lin_eval(x, &env) {
                        Some(v) if v >= 0 => t.push(v as u64),
                        _ => { ok = false; break }
                    }
                }
                if ok { bad.push(t); }
            }
        }
    }
    if bad.is_empty() { Ok(()) } else { Err(bad) }
}

// ------------------------------------------------------------------ driver

/// Summary counters returned by the `learn` / `learnfe` drivers: automaton size
/// reached and how much oracle work it cost, for the `AM_PROGRESS` telemetry and
/// end-of-run reporting.
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

/// Learn FE.  Kept as its own entry point because `learnfe` is a documented command.
pub fn learn_fe(seq: &Dfao, defs: &Defs) -> Result<(Dfa, LearnStats), String> {
    let spec = Spec::builtin(Kind::Fe)?;
    learn_pred(seq, defs, &spec)
}

/// Learn the predicate described by `spec`, escalating the oracle's cap if it turns
/// out to be too coarse.
///
/// A capped oracle can answer "matches forever" wrongly, which cannot break
/// correctness (the recurrence check is the judge) but can stall the learner: the
/// verifier keeps producing witnesses that the oracle does not recognise as
/// counterexamples.  When that happens we raise the cap and relearn from scratch --
/// restarting rather than patching, because the discrimination tree's stored answers
/// were taken under the old oracle and would now be inconsistent.
pub fn learn_pred(seq: &Dfao, defs: &Defs, spec: &Spec) -> Result<(Dfa, LearnStats), String> {
    let start: u64 = std::env::var("AM_LEARN_LCP").ok().and_then(|v| v.parse().ok())
        .unwrap_or(1 << 22);
    let ceiling: u64 = std::env::var("AM_LEARN_LCP_MAX").ok().and_then(|v| v.parse().ok())
        .unwrap_or(1 << 26);
    let mut cap = start.max(1);
    loop {
        match learn_once(seq, defs, spec, cap) {
            Ok(r) => return Ok(r),
            Err((msg, retry)) => {
                if !retry || cap >= ceiling { return Err(msg); }
                cap = (cap * 16).min(ceiling);
                if std::env::var("AM_LEARN_DEBUG").is_ok() {
                    eprintln!("  [learn] stalled ({}); raising AM_LEARN_LCP to {} and restarting", msg, cap);
                }
            }
        }
    }
}

fn learn_once(seq: &Dfao, defs: &Defs, spec: &Spec, hardcap: u64)
    -> Result<(Dfa, LearnStats), (String, bool)> {
    let t0 = Instant::now();
    let k = seq.k;
    let n = spec.n();
    if k.pow(n as u32) > crate::dfa::MAX_ALPHA {
        return Err((format!("base {} too large for a {}-track alphabet", k, n), false));
    }
    let unroll: u32 = std::env::var("AM_LEARN_UNROLL").ok().and_then(|v| v.parse().ok())
        .unwrap_or(5000);
    let mut maxdig: u32 = std::env::var("AM_LEARN_DIGITS").ok().and_then(|v| v.parse().ok())
        .unwrap_or(match k { 2 => 22, 3 => 14, 4 => 11, _ => 9 });
    if spec.kind == Kind::Custom {
        // The unrolled oracle costs one level per unit of the recursion coordinate, so
        // do not sample values it cannot afford to evaluate.
        let mut d = 1u32;
        while (k as u64).saturating_pow(d + 1) < unroll as u64 { d += 1; }
        maxdig = maxdig.min(d);
    }
    let tries: usize = std::env::var("AM_LEARN_SAMPLES").ok().and_then(|v| v.parse().ok())
        .unwrap_or(4000);
    let maxw: usize = std::env::var("AM_LEARN_WITNESS").ok().and_then(|v| v.parse().ok())
        .unwrap_or(256);
    let maxiter: usize = std::env::var("AM_LEARN_ITERS").ok().and_then(|v| v.parse().ok())
        .unwrap_or(20_000);
    let probe: usize = std::env::var("AM_LEARN_PROBE").ok().and_then(|v| v.parse().ok())
        .unwrap_or(2000);
    let dbg = std::env::var("AM_LEARN_DEBUG").is_ok();

    crate::progress::phase("learn", &format!("{} cap {}", spec.kind.name(), hardcap));
    let mut l = Learner::new(seq, hardcap, spec, defs, unroll);
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let mut iters = 0usize;
    let mut eqs = 0usize;
    let mut ces = 0usize;

    loop {
        // ---- cheap phase: randomised search on the class's language boundary
        loop {
            let want = 64 + l.nstates() / 2;
            let batch = sample_ces(&mut l, &mut rng, tries, maxdig, want);
            if batch.is_empty() { break; }
            let mut progress = false;
            let seeds: Vec<Vec<u64>> = batch.iter().filter_map(|w| l.dec(w)).collect();
            let mut all = local_probe(&mut l, &seeds, probe);
            all.extend(batch.iter().cloned());
            for w in &all { if l.process_ce(w) { ces += 1; progress = true; } }
            iters += 1;
            if dbg { eprintln!("  [learn] sample round: {} ces, {} states, {} mqs",
                               batch.len(), l.nstates(), l.or.mqs); }
            if !progress || iters > maxiter { break; }
        }
        // ---- exact phase: the self-verifying recurrence
        eqs += 1;
        let hyp = l.to_dfa();
        crate::progress::learn(eqs, hyp.nstates, l.or.mqs);
        if dbg { eprintln!("  [learn] EQ #{} on {} states", eqs, hyp.nstates); }
        crate::progress::phase("verify", "recurrence");
        let vr = verify(seq, defs, spec, &hyp, k, maxw);
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
                for t in &bad { all.push(l.enc(t)); }
                for w in &all { if l.process_ce(w) { ces += 1; progress = true; } }
                iters += 1;
                if dbg { eprintln!("  [learn] EQ #{} gave {} candidate ces -> {} states",
                                   eqs, bad.len(), l.nstates()); }
                if !progress {
                    return Err((format!("no progress: {} recurrence witnesses, none is a \
                        counterexample to the hypothesis at cap {}", bad.len(), hardcap), true));
                }
                if iters > maxiter {
                    return Err((format!("gave up after {} iterations", iters), false));
                }
            }
        }
    }
}

// ------------------------------------------------------------------ command

/// Parse and run a `learn` command line (everything after the word `learn`).
/// Returns the name to bind, its parameter list, the automaton, and the reply line.
pub fn cmd_learn(seq: &Dfao, defs: &Defs, rest: &str)
    -> Result<(String, Vec<String>, Dfa, String), String> {
    const USAGE: &str = "usage: learn NAME fe|rev|period|border  |  \
                         learn NAME (v1,..,vn) [on:v] init:FORMULA step:FORMULA";
    let rest = rest.trim();
    let cut = rest.find(char::is_whitespace).ok_or(USAGE)?;
    let (name, tail) = (rest[..cut].to_string(), rest[cut..].trim());
    let spec = if tail.starts_with('(') {
        let close = tail.find(')').ok_or("unclosed parameter list")?;
        let params: Vec<String> = tail[1..close].split(',')
            .map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if params.is_empty() { return Err("empty parameter list".into()); }
        for p in &params {
            if !p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                || !p.chars().next().unwrap().is_ascii_alphabetic() {
                return Err(format!("bad parameter name {:?}", p));
            }
        }
        let body = tail[close + 1..].trim();
        let ip = body.find("init:").ok_or("expected init:")?;
        let sp = body.find("step:").ok_or("expected step:")?;
        if sp < ip { return Err("init: must come before step:".into()); }
        let on = body[..ip].trim();
        let recname = if on.is_empty() { params.last().unwrap().clone() }
            else if let Some(v) = on.strip_prefix("on:") { v.trim().to_string() }
            else { return Err(format!("unexpected {:?} before init:", on)) };
        let init = body[ip + 5..sp].trim();
        let step = body[sp + 5..].trim();
        if init.is_empty() || step.is_empty() { return Err("empty init: or step:".into()); }
        Spec::custom(params, &recname, init, step, &seq.name)?
    } else {
        let word = tail.split_whitespace().next().unwrap_or("");
        let kind = Kind::parse(word).ok_or_else(|| format!("unknown kind {:?}; {}", word, USAGE))?;
        Spec::builtin(kind)?
    };
    dfa::peak_reset();
    let (a, st) = learn_pred(seq, defs, &spec)?;
    let msg = format!("OK learn {}({}) kind={} states={} iters={} eqs={} ces={} mqs={} \
steps={} peak={} ms={}{}",
        name, spec.params.join(","), spec.kind.name(), st.states, st.iters, st.eqs, st.ces,
        st.mqs, st.steps, dfa::peak_get(), st.ms,
        if st.assumed_inf > 0 { format!(" capped={}", st.assumed_inf) } else { String::new() });
    Ok((name, spec.params.clone(), a, msg))
}
