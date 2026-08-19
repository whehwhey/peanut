//! Auto-`learnfe`: make an ordinary `let NAME(args) <body>` take the guess-and-verify
//! (`learnfe` / `learn`) construction automatically when
//!
//!   1. `<body>` is *syntactically* one of the self-verifying predicate shapes the
//!      learner already knows (FE, rev, period, border -- up to variable renaming and
//!      argument order), and
//!   2. the ordinary determinization ladder cannot build it *cheaply*.
//!
//! The user then gets the fast `learnfe` construction on hard cases (tail-c: 448 s of
//! direct determinization -> ~15 s of guess-and-verify) without knowing the `learnfe`
//! command exists.
//!
//! # Why "cheap probe, then hand off" rather than a thread race
//!
//! Measured (see `docs/LEARNFE.md`, `bench/RIG-BENCH-32GB.md`): every panel / trib FE
//! case the direct construction can do at all finishes on the *cheap* rungs of the
//! adaptive ladder -- forward(`AM_CAP0`=50k) or Brzozowski(4*`AM_CAP0`=200k subsets),
//! peaking under ~120k subsets in milliseconds to a couple of seconds.  The cases the
//! direct construction is bad at (the "tail" family) blow straight past the cheap rungs
//! into millions of subsets and gigabytes over hundreds of seconds.  So the cheap rungs
//! are a clean, low-cost classifier: if they succeed the ladder answer is returned
//! unchanged (byte-for-byte identical to `AM_AUTOLEARN=0`); if they fail we hand off to
//! `learn_pred`, whose result is *proved* language-equal to the predicate by the
//! recurrence check and is the same minimal DFA the ladder would have produced.
//!
//! This is sequential -- no second thread, no shared-memory-budget race, no cancellation
//! of an in-flight subset construction (which the process's `panic = "abort"` profile
//! would make unsafe).  It is exactly the driver `docs/LEARNFE.md` §8 recommends.
//!
//! The probe is implemented by a process-global flag read inside the determinization
//! ladder (`Dfa::exists`): while it is set, the ladder stops after the two cheap rungs
//! and, if they both fail, records `gave_up` and returns a throwaway automaton instead
//! of grinding the expensive last-resort rungs.  The throwaway result is discarded by
//! the caller, which then reruns `learn_pred` with the probe off.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::learn::Kind;
use crate::logic::{Ast, Lin, Rel};

// ------------------------------------------------------------------ switch + probe

/// `AM_AUTOLEARN` (default ON).  `AM_AUTOLEARN=0` forces the pure ladder on every
/// `let`, which is what the benchmarks use to time the two paths separately.
pub fn enabled() -> bool {
    std::env::var("AM_AUTOLEARN").map(|v| v != "0").unwrap_or(true)
}

static PROBE: AtomicBool = AtomicBool::new(false);
static GAVE_UP: AtomicBool = AtomicBool::new(false);

/// Start a cheap-ladder probe: the determinization ladder will not escalate past its
/// two cheap rungs.  Clears the `gave_up` flag.
pub fn probe_begin() {
    GAVE_UP.store(false, Ordering::Relaxed);
    PROBE.store(true, Ordering::Relaxed);
}
/// End the probe (must run before any `learn_pred`, whose verifier determinizes too).
pub fn probe_end() { PROBE.store(false, Ordering::Relaxed); }
/// Is a probe active?  Read in the hot ladder in `engine/src/dfa.rs`.
#[inline]
pub fn probe_active() -> bool { PROBE.load(Ordering::Relaxed) }
/// The ladder hit the end of its cheap rungs during a probe; caller should hand off.
pub fn set_gave_up() { GAVE_UP.store(true, Ordering::Relaxed); }
/// Did the most recent probe give up?
pub fn gave_up() -> bool { GAVE_UP.load(Ordering::Relaxed) }

// ------------------------------------------------------------------ shape detection

/// A matched predicate shape: which learner class, and how the user's parameter names
/// map onto the class's canonical coordinate names (`i`/`j`/`l`, `i`/`l`/`p`, ...).
pub struct Shape {
    pub kind: Kind,
    /// `(canonical name, user parameter name)` for every canonical coordinate.
    pub roles: Vec<(&'static str, String)>,
}

impl Shape {
    /// Rename map from a canonical coordinate name to the user's parameter name.
    pub fn user_name(&self, canonical: &str) -> String {
        self.roles.iter().find(|(c, _)| *c == canonical)
            .map(|(_, u)| u.clone()).unwrap_or_else(|| canonical.to_string())
    }
}

/// If `body` is exactly one of the self-verifying shapes over the parameter set
/// `params`, return the matched shape.  Deliberately strict: anything that is not the
/// canonical recurrence body (a different guard, a shifted index, an extra conjunct)
/// returns `None` and is left to the ordinary ladder.
pub fn detect(body: &Ast, params: &[String]) -> Option<Shape> {
    detect_fe(body, params)
        .or_else(|| detect_rev(body, params))
        .or_else(|| detect_period(body, params))
        .or_else(|| detect_border(body, params))
}

// -- linear-term helpers -------------------------------------------------

/// `t` is exactly the bare variable `v`.
fn is_var(t: &Lin, v: &str) -> bool { t.is_plain_var().as_deref() == Some(v) }

/// `t == a + b` with both coefficients 1 and no constant; returns the var that is not
/// `inner`.  (Matches an index of the form `p + inner`.)
fn other_of_sum2(t: &Lin, inner: &str) -> Option<String> {
    if t.c != 0 || t.coef.len() != 2 { return None; }
    if t.coef.get(inner).copied() != Some(1) { return None; }
    let mut other = None;
    for (k, c) in &t.coef {
        if k == inner { continue; }
        if *c != 1 { return None; }
        other = Some(k.clone());
    }
    other
}

/// `t == a + b + c` with all three coefficients 1 and no constant, given two of the
/// three variables; returns true iff the third matches nothing extra (exact match of
/// `{x, y, z}` all coef 1).
fn is_sum3(t: &Lin, x: &str, y: &str, z: &str) -> bool {
    t.c == 0 && t.coef.len() == 3
        && t.coef.get(x).copied() == Some(1)
        && t.coef.get(y).copied() == Some(1)
        && t.coef.get(z).copied() == Some(1)
}

/// A guard `inner < BOUND` (or `BOUND > inner`) where BOUND is a bare parameter var;
/// returns BOUND.
fn guard_lt_param(g: &Ast, inner: &str, params: &[String]) -> Option<String> {
    if let Ast::Cmp(a, r, b) = g {
        let (lo, hi) = match r {
            Rel::Lt => (a, b),
            Rel::Gt => (b, a),
            _ => return None,
        };
        if is_var(lo, inner) {
            if let Some(v) = hi.is_plain_var() {
                if params.iter().any(|p| p == &v) { return Some(v); }
            }
        }
    }
    None
}

/// Unwrap `A t. GUARD => CMP` for a single fresh inner variable `t`.
fn as_forall_imp<'a>(body: &'a Ast, params: &[String]) -> Option<(String, &'a Ast, &'a Ast)> {
    let (vs, g, c) = as_forall_imp_n(body, params, 1)?;
    Some((vs[0].clone(), g, c))
}

/// Unwrap `A v1,..,vn. GUARD => CMP` for `n` fresh inner variables.
fn as_forall_imp_n<'a>(body: &'a Ast, params: &[String], n: usize)
    -> Option<(Vec<String>, &'a Ast, &'a Ast)> {
    if let Ast::Forall(vs, inner) = body {
        if vs.len() != n { return None; }
        if vs.iter().any(|v| params.iter().any(|p| p == v)) { return None; }  // all fresh
        if let Ast::Imp(g, c) = inner.as_ref() {
            return Some((vs.clone(), g.as_ref(), c.as_ref()));
        }
    }
    None
}

/// The two conjuncts of `A & B`, in both orders.
fn as_and<'a>(a: &'a Ast) -> Option<(&'a Ast, &'a Ast)> {
    if let Ast::And(x, y) = a { Some((x.as_ref(), y.as_ref())) } else { None }
}

/// `x + y + 1 == v` (either side), with `known` one of the two summands and `v` a bare
/// param; returns the other summand.  Used for the `t+u+1=l` guard of `rev`.
fn sum1_eq(cmp: &Ast, known: &str, params: &[String]) -> Option<(String, String)> {
    if let Ast::Cmp(a, Rel::Eq, b) = cmp {
        for (sum, v) in [(a, b), (b, a)] {
            if let Some(res) = v.is_plain_var() {
                if !params.iter().any(|p| p == &res) { continue; }
                if sum.c == 1 && sum.coef.len() == 2 && sum.coef.get(known).copied() == Some(1) {
                    let other = sum.coef.keys().find(|k| k.as_str() != known)?;
                    if sum.coef.get(other).copied() == Some(1) {
                        return Some((other.clone(), res));
                    }
                }
            }
        }
    }
    None
}

/// `x + y == v` (either side), with `known` one summand and `v` a bare param; returns
/// the other summand.  Used for the `u+b=l` guard of `border`.
fn sum0_eq(cmp: &Ast, known: &str, params: &[String]) -> Option<(String, String)> {
    if let Ast::Cmp(a, Rel::Eq, b) = cmp {
        for (sum, v) in [(a, b), (b, a)] {
            if let Some(res) = v.is_plain_var() {
                if !params.iter().any(|p| p == &res) { continue; }
                if sum.c == 0 && sum.coef.len() == 2 && sum.coef.get(known).copied() == Some(1) {
                    let other = sum.coef.keys().find(|k| k.as_str() != known)?;
                    if sum.coef.get(other).copied() == Some(1) {
                        return Some((other.clone(), res));
                    }
                }
            }
        }
    }
    None
}

/// The two index terms of a `T[..] = T[..]` comparison (Eq only).
fn seqseq_eq(cmp: &Ast) -> Option<(&Lin, &Lin)> {
    if let Ast::SeqSeq(a, Rel::Eq, b) = cmp { Some((a, b)) } else { None }
}

/// The three parameters are exactly `{a, b, c}`.
fn params_are(params: &[String], set: &[&str]) -> bool {
    if params.len() != set.len() { return false; }
    set.iter().all(|s| params.iter().any(|p| p == s))
        && params.iter().all(|p| set.iter().any(|s| p == s))
}

// -- FE:  A t. t<l => T[i+t] = T[j+t] -----------------------------------

fn detect_fe(body: &Ast, params: &[String]) -> Option<Shape> {
    if params.len() != 3 { return None; }
    let (t, g, c) = as_forall_imp(body, params)?;
    let l = guard_lt_param(g, &t, params)?;
    let (lhs, rhs) = seqseq_eq(c)?;
    let i = other_of_sum2(lhs, &t)?;   // i + t
    let j = other_of_sum2(rhs, &t)?;   // j + t
    if i == j { return None; }
    if !params_are(params, &[&i, &j, &l]) { return None; }
    Some(Shape { kind: Kind::Fe, roles: vec![("i", i), ("j", j), ("l", l)] })
}

// -- REV:  A t,u. (t<l & t+u+1=l) => T[i+t] = T[j+u] --------------------
// (the subtraction-free form the ladder can compile; `t+u+1=l` means `u = l-1-t`)

fn detect_rev(body: &Ast, params: &[String]) -> Option<Shape> {
    if params.len() != 3 { return None; }
    let (_vs, g, c) = as_forall_imp_n(body, params, 2)?;
    let (g1, g2) = as_and(g)?;
    // one conjunct is `t < l`, the other is `t + u + 1 = l`
    for (lt_clause, eq_clause) in [(g1, g2), (g2, g1)] {
        let Ast::Cmp(a, r, b) = lt_clause else { continue };
        let (t, l) = match r {
            Rel::Lt if a.is_plain_var().is_some() && b.is_plain_var().is_some() =>
                (a.is_plain_var().unwrap(), b.is_plain_var().unwrap()),
            Rel::Gt if a.is_plain_var().is_some() && b.is_plain_var().is_some() =>
                (b.is_plain_var().unwrap(), a.is_plain_var().unwrap()),
            _ => continue,
        };
        if !params.iter().any(|p| p == &l) { continue; }
        let Some((u, l2)) = sum1_eq(eq_clause, &t, params) else { continue };
        if l2 != l { continue; }
        let Some((lhs, rhs)) = seqseq_eq(c) else { continue };
        let Some(i) = other_of_sum2(lhs, &t) else { continue };   // i + t
        let Some(j) = other_of_sum2(rhs, &u) else { continue };   // j + u
        if !params_are(params, &[&i, &j, &l]) { continue; }
        return Some(Shape { kind: Kind::Rev, roles: vec![("i", i), ("j", j), ("l", l)] });
    }
    None
}

// -- PERIOD:  A t. t+p<l => T[i+t] = T[i+t+p] ---------------------------

fn detect_period(body: &Ast, params: &[String]) -> Option<Shape> {
    if params.len() != 3 { return None; }
    let (t, g, c) = as_forall_imp(body, params)?;
    // guard: t + p < l  (lo = t+p, hi = l)
    let (lo, hi, p, l) = match g {
        Ast::Cmp(a, Rel::Lt, b) => {
            // a = t + p, b = l
            let p = other_of_sum2(a, &t)?;
            let l = b.is_plain_var()?;
            (a, b, p, l)
        }
        Ast::Cmp(a, Rel::Gt, b) => {
            let p = other_of_sum2(b, &t)?;
            let l = a.is_plain_var()?;
            (b, a, p, l)
        }
        _ => return None,
    };
    let _ = (lo, hi);
    if !params.iter().any(|q| q == &l) || !params.iter().any(|q| q == &p) { return None; }
    let (lhs, rhs) = seqseq_eq(c)?;
    let i = other_of_sum2(lhs, &t)?;   // i + t
    if !is_sum3(rhs, &i, &t, &p) { return None; }   // i + t + p
    if !params_are(params, &[&i, &l, &p]) { return None; }
    Some(Shape { kind: Kind::Period, roles: vec![("i", i), ("l", l), ("p", p)] })
}

// -- BORDER:  (b<=l) & (A t,u. (t<b & u+b=l) => T[i+t] = T[i+u+t]) ------
// (the subtraction-free form; `u+b=l` means `u = l-b`, the border offset)

fn detect_border(body: &Ast, params: &[String]) -> Option<Shape> {
    if params.len() != 3 { return None; }
    let (c1, c2) = as_and(body)?;
    // one top-level conjunct is `b <= l`, the other the quantified body
    for (le_clause, quant) in [(c1, c2), (c2, c1)] {
        let (b, l) = match le_clause {
            Ast::Cmp(a, Rel::Le, bb) if a.is_plain_var().is_some() && bb.is_plain_var().is_some() =>
                (a.is_plain_var().unwrap(), bb.is_plain_var().unwrap()),
            Ast::Cmp(a, Rel::Ge, bb) if a.is_plain_var().is_some() && bb.is_plain_var().is_some() =>
                (bb.is_plain_var().unwrap(), a.is_plain_var().unwrap()),
            _ => continue,
        };
        if !params.iter().any(|p| p == &b) || !params.iter().any(|p| p == &l) { continue; }
        let Some((_vs, g, cmp)) = as_forall_imp_n(quant, params, 2) else { continue };
        let Some((g1, g2)) = as_and(g) else { continue };
        // one conjunct is `t < b`, the other `u + b = l`
        for (lt_clause, eq_clause) in [(g1, g2), (g2, g1)] {
            let Ast::Cmp(ta, r, tb) = lt_clause else { continue };
            let t = match r {
                Rel::Lt if ta.is_plain_var().is_some() && tb.is_plain_var().as_deref() == Some(&b) =>
                    ta.is_plain_var().unwrap(),
                Rel::Gt if tb.is_plain_var().is_some() && ta.is_plain_var().as_deref() == Some(&b) =>
                    tb.is_plain_var().unwrap(),
                _ => continue,
            };
            // eq: u + b = l
            let Some((u, l2)) = sum0_eq(eq_clause, &b, params) else { continue };
            if l2 != l { continue; }
            let Some((lhs, rhs)) = seqseq_eq(cmp) else { continue };
            let Some(i) = other_of_sum2(lhs, &t) else { continue };   // i + t
            if !is_sum3(rhs, &i, &u, &t) { continue; }                // i + u + t
            if !params_are(params, &[&i, &l, &b]) { continue; }
            return Some(Shape { kind: Kind::Border, roles: vec![("i", i), ("l", l), ("b", b)] });
        }
    }
    None
}
