//! Antichain / lazy evaluation of CLOSED sentences (**default ON** since
//! 2026-08-19; `AM_ANTICHAIN=0` restores the pre-2026-08-19 path).
//!
//! The default pipeline (`logic.rs` -> `Dfa::exists` / `Dfa::forall`) turns *every*
//! quantifier into a determinization: `exists` subset-constructs the projected NFA,
//! `forall` is `complement().exists().complement()`.  For a **closed** sentence the
//! outermost block does not need an automaton at all -- only a yes/no answer -- and
//! there are two classical ways to get that answer without determinizing:
//!
//! * **`E x1..xn. phi`** (all free variables of `phi` bound).  Projecting a track can
//!   neither create nor destroy accepted words, and the padding closure only adds
//!   words, so the sentence is TRUE iff the *body* automaton has any accepted word at
//!   all.  That is one BFS over `phi`'s DFA -- `Dfa::is_nonempty` -- instead of `n`
//!   subset constructions.
//!
//! * **`A x1..xm. (g => E y1..yp. phi)`** (guard optional, either block possibly
//!   empty).  Project `y1..yp` out of `phi`'s DFA to get an NFA `N` over the `x`
//!   tracks, close it under the padding convention (see `close_padding`), and ask
//!   whether `L(G) subset L(N)`, where `G` is the guard conjoined with "every track
//!   holds a valid representation" -- exactly what the `complement/exists/complement`
//!   chain computes.  That is an NFA **universality** question, and universality is
//!   decided by the antichain algorithm of De Wulf, Doyen, Henzinger and Raskin
//!   (*Antichains: a new algorithm for checking universality of finite automata*,
//!   CAV 2006) without ever building the full subset automaton: the subset
//!   construction's reachable sets are explored only up to subsumption, keeping the
//!   `subset`-minimal ones, because `S subset S'` implies every counterexample
//!   reachable from `S'` is reachable from `S`.
//!
//!   Optionally (`AM_AC_SIM`) a forward simulation preorder on `N` refines the
//!   subsumption test to `L(S) subset L(S')` in the sense of Abdulla, Chen, Holik,
//!   Mayr and Vojnar (*When simulation meets antichains*, TACAS 2010): a set may be
//!   pruned down to its simulation-maximal states, and `S` subsumes `S'` as soon as
//!   every state of `S` is simulated by some state of `S'`.
//!
//! Everything below the outermost block is compiled by the ordinary compiler, so the
//! shapes that benefit are precisely those whose *outermost* block is expensive:
//! `E`-blocks over several variables, and `A ... E ...` alternations (the "border",
//! "right-special" and "recurrence" shapes of `docs/FUZZ.md`).  Sentences whose cost
//! is an *inner* quantifier are unaffected -- see `docs/ANTICHAIN.md`.
//!
//! The entry point returns `Ok(None)` for every shape it does not handle, and also
//! when its own search exceeds `AM_AC_CAP`; the caller then falls back to the normal
//! compilation, so this module can only ever answer or abstain, never approximate.

use crate::dfa::{self, Dfa, Nfa, State, digit};
use crate::dfao::Dfao;
use crate::logic::{Ast, Compiler, Defs, Lin};
use std::collections::BTreeSet;

/// Is this module allowed to answer closed sentences?
///
/// **Default ON** since 2026-08-19 (`bench/SPEED-ROUND6.md`, "Final defaults"):
/// zero disagreements over the 1100-script fuzz suite, the 912-script GUI library
/// suite and the 266-sentence FE suite (`tools/antichain_gate.py`), 87x-325x
/// faster on the `A ... E ...` shapes it fires on, and a worst measured overhead
/// of 3.4% when it starts an antichain and then abandons it
/// (`bench/ANTICHAIN-RESULTS.md`).  `AM_ANTICHAIN=0` turns it off; any other value,
/// or no value at all, leaves it on.
pub fn enabled() -> bool {
    static E: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *E.get_or_init(|| std::env::var("AM_ANTICHAIN").map(|v| v != "0").unwrap_or(true))
}

fn dbg_on() -> bool { std::env::var("AM_AC_DEBUG").is_ok() }
fn envnum(k: &str, d: usize) -> usize {
    std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

// ------------------------------------------------------------------ free variables

fn lin_vars(l: &Lin, out: &mut BTreeSet<String>) {
    for v in l.coef.keys() { out.insert(v.clone()); }
}

/// The free variables of a formula.  `Call` arguments are free; the callee's own
/// parameters are bound by the stored automaton and never leak out.
pub fn free_vars(a: &Ast) -> BTreeSet<String> {
    let mut s = BTreeSet::new();
    fv(a, &mut s);
    s
}

fn fv(a: &Ast, out: &mut BTreeSet<String>) {
    match a {
        Ast::Bool(_) => {}
        Ast::Cmp(l, _, r) => { lin_vars(l, out); lin_vars(r, out); }
        Ast::SeqLetter(t, _, _) => lin_vars(t, out),
        Ast::SeqSeq(t1, _, t2) => { lin_vars(t1, out); lin_vars(t2, out); }
        Ast::IsPow(t) => lin_vars(t, out),
        Ast::Call(_, args) => { for l in args { lin_vars(l, out); } }
        Ast::Not(x) => fv(x, out),
        Ast::And(x, y) | Ast::Or(x, y) | Ast::Imp(x, y) | Ast::Iff(x, y) => { fv(x, out); fv(y, out); }
        Ast::Forall(vs, x) | Ast::Exists(vs, x) => {
            let mut inner = BTreeSet::new();
            fv(x, &mut inner);
            for v in vs { inner.remove(v); }
            out.extend(inner);
        }
    }
}

// ------------------------------------------------------------------ entry point

/// Evaluate a closed sentence without determinizing its outermost quantifier block.
///
/// `Ok(Some(v))` is the verdict; `Ok(None)` means "not my shape, or my search hit its
/// budget" and the caller must fall back to the ordinary compiler.
pub fn eval_closed(k: usize, seq: &Dfao, defs: &Defs, ast: &Ast) -> Result<Option<bool>, String> {
    if !free_vars(ast).is_empty() { return Ok(None); }
    let mut c = Compiler::new(k, seq, defs);
    ev(&mut c, k, ast, true)
}

/// Is the sentence `a` (or its negation, when `pos` is false) true?
///
/// Polarity is threaded rather than pushed into the formula so that the two shapes
/// below are recognised wherever they occur: `~E` and `~A` are the same two shapes
/// with the answer flipped, and `A x. g => ~z` is `~E x. g & z`, an E-block again.
fn ev(c: &mut Compiler, k: usize, a: &Ast, pos: bool) -> Result<Option<bool>, String> {
    Ok(match a {
        Ast::Bool(b) => Some(*b == pos),
        Ast::Not(x) => ev(c, k, x, !pos)?,
        // A closed connective has closed operands, so each side is a sentence of its
        // own; De Morgan turns the negative polarity cases into the positive ones.
        Ast::And(x, y) | Ast::Or(x, y) | Ast::Imp(x, y) => {
            let (conj, negl) = match a {
                Ast::And(..) => (pos, false),
                Ast::Or(..) => (!pos, false),
                _ => (!pos, true),                    // x => y  is  ~x | y
            };
            // `conj` = evaluate as a conjunction of (possibly negated) sides
            let l = ev(c, k, x, if negl { !pos } else { pos })?;
            match (l, conj) {
                (Some(false), true) => Some(false),
                (Some(true), false) => Some(true),
                (Some(_), _) => ev(c, k, y, pos)?,
                (None, _) => None,
            }
        }
        Ast::Iff(x, y) => match (ev(c, k, x, true)?, ev(c, k, y, true)?) {
            (Some(p), Some(q)) => Some((p == q) == pos),
            _ => None,
        },
        // `E xs. b` is nonemptiness of the body; `~A xs. b` is `E xs. ~b`.
        Ast::Exists(vs, body) => ev_exists(c, k, vs, body)?.map(|v| v == pos),
        Ast::Forall(vs, body) if !pos =>
            ev_exists(c, k, vs, &Ast::Not(Box::new((**body).clone())))?,
        Ast::Forall(vs, body) => ev_forall(c, k, vs, body)?,
        _ => None,
    })
}

/// Would a product over `nvars + add` tracks stay inside the working-alphabet ceiling?
/// Hoisting an existential out of a conjunction keeps its track alive in the product,
/// and every extra track multiplies the alphabet by `k`.
fn alpha_ok(k: usize, nvars: usize, add: usize) -> bool {
    let cap = envnum("AM_AC_ALPHA", 1 << 16) as u128;
    let mut r: u128 = 1;
    for _ in 0..nvars + add {
        r *= k as u128;
        if r > cap { return false; }
    }
    true
}

/// Split a formula into a conjunction of parts, hoisting nested existential blocks
/// into `bound` as it goes: `E x. (p & E y. q)` becomes bound `[x, y]`, parts `[p, q]`.
///
/// A block is hoisted only when its variables are new (otherwise
/// `(E j. p(j)) & (E j. q(j))` would be silently turned into `E j. p(j) & q(j)`) and
/// only while the product's alphabet stays bounded.  Anything else -- negations,
/// disjunctions, comparisons -- is emitted as a part and compiled by the ordinary
/// compiler.
fn conj_block<'a>(a: &'a Ast, k: usize, bound: &mut Vec<String>, parts: &mut Vec<&'a Ast>) {
    match a {
        Ast::And(x, y) => { conj_block(x, k, bound, parts); conj_block(y, k, bound, parts); }
        Ast::Exists(vs, b)
            if vs.iter().all(|v| !bound.contains(v)) && alpha_ok(k, bound.len(), vs.len()) => {
            for v in vs { bound.push(v.clone()); }
            conj_block(b, k, bound, parts);
        }
        _ => parts.push(a),
    }
}

/// Compile the parts of a conjunction and intersect them.
fn compile_parts(c: &mut Compiler, parts: &[&Ast]) -> Result<Dfa, String> {
    let mut acc: Option<Dfa> = None;
    for p in parts {
        let d = c.compile(p)?;
        acc = Some(match acc { None => d, Some(a) => a.and(&d) });
    }
    Ok(acc.expect("empty conjunction"))
}

/// `E x1..xn. body`, every free variable of `body` bound: TRUE iff the body automaton
/// accepts anything at all -- one BFS, no subset construction.
fn ev_exists(c: &mut Compiler, k: usize, vs: &[String], body: &Ast) -> Result<Option<bool>, String> {
    let mut bound: Vec<String> = Vec::new();
    for v in vs { if bound.contains(v) { return Ok(None); } bound.push(v.clone()); }
    let mut parts: Vec<&Ast> = Vec::new();
    conj_block(body, k, &mut bound, &mut parts);
    let d = compile_parts(c, &parts)?;
    if d.vars.is_empty() { return Ok(Some(d.accepts_epsilon())); }
    if !d.vars.iter().all(|v| bound.contains(v)) { return Ok(None); }
    if dbg_on() {
        eprintln!("  [ac] E-block over [{}]: {} parts, body {} states, reachability only",
                  bound.join(","), parts.len(), d.nstates);
    }
    Ok(Some(d.is_nonempty()))
}

/// `A x1..xm. body`, where `body` may be `guard => ...` and may end in an existential
/// block: an NFA universality question, answered by [`universal`].
fn ev_forall(c: &mut Compiler, k: usize, vs: &[String], body: &Ast) -> Result<Option<bool>, String> {
    let mut univ: Vec<String> = Vec::new();
    for v in vs { if univ.contains(v) { return Ok(None); } univ.push(v.clone()); }
    let mut inner = body;
    while let Ast::Forall(w, b) = inner {
        for v in w { if univ.contains(v) { return Ok(None); } univ.push(v.clone()); }
        inner = b;
    }
    // `g => h` and `g | h` both put a guard in front of the interesting half
    let (guard, inner): (Option<Ast>, &Ast) = match inner {
        Ast::Imp(g, h) => (Some((**g).clone()), &**h),
        Ast::Or(g, h) => (Some(Ast::Not(Box::new((**g).clone()))), &**h),
        _ => (None, inner),
    };
    // `A x. g => ~z` is `~E x. g & z`: no universality question at all
    if let Ast::Not(z) = inner {
        let e = match &guard {
            Some(g) => Ast::And(Box::new(g.clone()), z.clone()),
            None => (**z).clone(),
        };
        return Ok(ev_exists(c, k, &univ, &e)?.map(|v| !v));
    }
    let mut bound = univ.clone();
    let mut parts: Vec<&Ast> = Vec::new();
    conj_block(inner, k, &mut bound, &mut parts);
    let ys: Vec<String> = bound[univ.len()..].to_vec();
    let dphi = compile_parts(c, &parts)?;
    let dguard = match &guard { Some(g) => Some(c.compile(g)?), None => None };

    // the tracks the universality question is asked over
    let mut keep_vars: Vec<String> = dphi.vars.iter().filter(|v| !ys.contains(v)).cloned().collect();
    if let Some(g) = &dguard {
        for v in &g.vars { if !keep_vars.contains(v) { keep_vars.push(v.clone()); } }
    }
    keep_vars.sort();
    if !keep_vars.iter().all(|v| univ.contains(v)) { return Ok(None); }

    // G = guard AND "every track holds a valid representation"
    let all = Dfa::constant(k, keep_vars.clone(), true);
    let g = match &dguard { Some(d) => all.and(d), None => all };

    // N = the conjunction projected along the ys, over exactly `keep_vars`
    let mut full = keep_vars.clone();
    for v in &dphi.vars { if ys.contains(v) && !full.contains(v) { full.push(v.clone()); } }
    full.sort();
    let n = close_padding(project(&dphi.extend_vars(&full), &ys));

    if dbg_on() {
        eprintln!("  [ac] A-block over [{}]: E[{}], guard {} states, nfa {} states, alpha {}",
                  keep_vars.join(","), ys.join(","), g.nstates, n.nstates, n.alpha);
    }
    // Never abstain from here: the caller's fallback would recompile the body, and the
    // body is the expensive part.  If the antichain runs out of budget, finish the job
    // the ordinary way from the NFA that is already in hand.
    Ok(Some(match universal(&g, &n) {
        Some(v) => v,
        None => {
            if dbg_on() { eprintln!("  [ac] antichain gave up; determinizing the projection"); }
            contains(&g, &determinize_ladder(&n))
        }
    }))
}

/// `Dfa::exists`'s determinization ladder, applied to an already-projected NFA:
/// forward(AM_CAP0) -> Brzozowski -> forward(AM_CAP) -> Brzozowski, same caps and same
/// order as `dfa.rs`, so the fallback costs what the ordinary path would have cost.
fn determinize_ladder(n: &Nfa) -> Dfa {
    let cap0 = envnum("AM_CAP0", 50_000);
    let cap = envnum("AM_CAP", 3_000_000);
    let brz = |c: usize| -> Option<Dfa> {
        let r1 = n.reversed().determinize_capped(c)?;
        r1.as_nfa().reversed().determinize_capped(c)
    };
    if let Some(d) = n.determinize_capped(cap0) { return d; }
    if let Some(d) = brz(cap0.saturating_mul(4).max(200_000)) { return d; }
    if let Some(d) = n.determinize_capped(cap) { return d; }
    brz(cap.saturating_mul(4).max(8_000_000)).expect("forward and reverse determinization both blew up")
}

/// Is every word accepted by `g` accepted by `d`?  Reachability in the product.
fn contains(g: &Dfa, d: &Dfa) -> bool {
    assert_eq!(g.alpha, d.alpha);
    let mut seen: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
    let mut stack = vec![(0u32, 0u32)];
    seen.insert((0, 0));
    while let Some((p, q)) = stack.pop() {
        if g.accept[p as usize] && !d.accept[q as usize] { return false; }
        for a in 0..g.alpha {
            let nx = (g.trans[p as usize * g.alpha + a], d.trans[q as usize * d.alpha + a]);
            if seen.insert(nx) { stack.push(nx); }
        }
    }
    true
}

// ------------------------------------------------------------------ NFA construction

/// Drop every track named in `ys` from a DFA, producing the NFA of the existential
/// projection (before the padding closure).  Projecting all of them at once is the
/// same language as projecting them one at a time -- both are
/// `{u : exists m, exists y of length |u|+m with (u 0^m, y) accepted}`.
fn project(d: &Dfa, ys: &[String]) -> Nfa {
    let k = d.k;
    let keep: Vec<usize> = (0..d.vars.len()).filter(|&i| !ys.contains(&d.vars[i])).collect();
    let newvars: Vec<String> = keep.iter().map(|&i| d.vars[i].clone()).collect();
    let nalpha = k.pow(keep.len() as u32);
    // old symbol -> new symbol
    let mut proj = vec![0usize; d.alpha];
    for a in 0..d.alpha {
        let mut s = 0usize;
        let mut mult = 1usize;
        for &i in &keep { s += digit(a, i, k) * mult; mult *= k; }
        proj[a] = s;
    }
    let mut trans: Vec<Vec<State>> = vec![Vec::new(); d.nstates * nalpha];
    for st in 0..d.nstates {
        for a in 0..d.alpha {
            trans[st * nalpha + proj[a]].push(d.trans[st * d.alpha + a]);
        }
    }
    for t in trans.iter_mut() { t.sort_unstable(); t.dedup(); }
    Nfa { k, vars: newvars, alpha: nalpha, nstates: d.nstates, trans, init: vec![0], accept: d.accept.clone() }
}

/// Re-establish the padding convention on a projected NFA, the NFA-level equivalent of
/// `Dfa::zero_closure` composed with the determinization it follows.
///
/// msd (padding = leading zeros): `L'' = {w : exists m, 0^m w in L}`, so close the
/// *initial* set forward under the all-zero symbol.
/// lsd (padding = trailing zeros): `L'' = {w : exists m, w 0^m in L}`, so close the
/// *accepting* set backward under the all-zero symbol.
fn close_padding(mut n: Nfa) -> Nfa {
    if dfa::is_lsd() {
        let mut acc = n.accept.clone();
        loop {
            let mut changed = false;
            for q in 0..n.nstates {
                if acc[q] { continue; }
                if n.trans[q * n.alpha].iter().any(|&d| acc[d as usize]) { acc[q] = true; changed = true; }
            }
            if !changed { break; }
        }
        n.accept = acc;
    } else {
        let mut seen = vec![false; n.nstates];
        let mut stack = n.init.clone();
        for &s in &n.init { seen[s as usize] = true; }
        while let Some(s) = stack.pop() {
            for &d in &n.trans[s as usize * n.alpha] {
                if !seen[d as usize] { seen[d as usize] = true; stack.push(d); }
            }
        }
        n.init = (0..n.nstates as u32).filter(|&s| seen[s as usize]).collect();
    }
    n
}

// ------------------------------------------------------------------ bitsets

#[inline] fn bs_words(n: usize) -> usize { (n + 63) / 64 }
#[inline] fn bs_set(b: &mut [u64], i: usize) { b[i / 64] |= 1u64 << (i % 64); }
#[inline] fn bs_get(b: &[u64], i: usize) -> bool { b[i / 64] >> (i % 64) & 1 != 0 }
#[inline] fn bs_subset(a: &[u64], b: &[u64]) -> bool { a.iter().zip(b).all(|(x, y)| x & !y == 0) }
#[inline] fn bs_disjoint(a: &[u64], b: &[u64]) -> bool { a.iter().zip(b).all(|(x, y)| x & y == 0) }
#[inline] fn bs_count(a: &[u64]) -> u32 { a.iter().map(|w| w.count_ones()).sum() }

// ------------------------------------------------------------------ simulation

/// Greatest forward simulation on `n`: bit `t` of row `s` means "`t` simulates `s`",
/// i.e. `t` accepts whenever `s` does and can match every move of `s`.  Then
/// `L(s) subset L(t)`, which is what makes the refined subsumption test below sound.
///
/// Computed by the naive `O(|Q|^2)`-pairs fixpoint, so it is only worth doing on small
/// automata; the caller gates it on `AM_AC_SIM` (a state-count ceiling).
fn simulation(n: &Nfa) -> Vec<u64> {
    let q = n.nstates;
    let w = bs_words(q);
    let mut sim = vec![0u64; q * w];
    for s in 0..q {
        for t in 0..q {
            if !n.accept[s] || n.accept[t] { bs_set(&mut sim[s * w..(s + 1) * w], t); }
        }
    }
    loop {
        let mut changed = false;
        for s in 0..q {
            for t in 0..q {
                if !bs_get(&sim[s * w..(s + 1) * w], t) { continue; }
                let mut ok = true;
                'a: for a in 0..n.alpha {
                    for &s2 in &n.trans[s * n.alpha + a] {
                        let row = &sim[s2 as usize * w..(s2 as usize + 1) * w];
                        if !n.trans[t * n.alpha + a].iter().any(|&t2| bs_get(row, t2 as usize)) {
                            ok = false;
                            break 'a;
                        }
                    }
                }
                if !ok { sim[s * w + t / 64] &= !(1u64 << (t % 64)); changed = true; }
            }
        }
        if !changed { break; }
    }
    sim
}

// ------------------------------------------------------------------ universality

struct Entry { bits: Vec<u64>, pc: u32, dead: bool }

/// Is every word accepted by the guard DFA `g` also accepted by the NFA `n`?
///
/// Ladder, in the spirit of the determinization ladder in `dfa.rs`: try the plain
/// subset-inclusion antichain first with a small element budget, and only pay for the
/// simulation preorder if that budget is hit.  Simulation shrinks the antichain
/// dramatically when it matters (2246-state NFA on tail-a: 691 elements -> 1) but its
/// naive fixpoint costs more than the whole search does when it does not.
///
/// `AM_AC_SIM` = `off` | `auto` (default) | `on`.
/// `None` means every attempt exceeded its budget and the caller must fall back.
fn universal(g: &Dfa, n: &Nfa) -> Option<bool> {
    let cap = envnum("AM_AC_CAP", 200_000);
    // One work budget for the WHOLE call, spent across attempts.  Giving up costs the
    // caller a full recompilation down the ordinary path, so a failure has to be cheap:
    // this bounds the wasted work at a few tenths of a second.
    let mut budget = envnum("AM_AC_WORK", 4_000_000) as u64;
    let trig = envnum("AM_AC_SIM_TRIGGER", 5_000);
    let mode = std::env::var("AM_AC_SIM").unwrap_or_else(|_| "auto".into());
    let affordable = n.nstates > 1
        && n.nstates.saturating_mul(n.nstates).saturating_mul(n.alpha)
            <= envnum("AM_AC_SIMWORK", 8_000_000);
    if mode == "on" && affordable {
        let sm = simulation(n);
        return search(g, n, Some(&sm), cap, &mut budget);
    }
    let first = if mode == "off" || !affordable { cap } else { cap.min(trig) };
    if let Some(v) = search(g, n, None, first, &mut budget) { return Some(v); }
    if mode != "off" && affordable && budget > 0 {
        if dbg_on() { eprintln!("  [ac] plain antichain > {} elements; computing simulation", first); }
        let sm = simulation(n);
        if let Some(v) = search(g, n, Some(&sm), cap, &mut budget) { return Some(v); }
    }
    None
}

fn search(g: &Dfa, n: &Nfa, sim: Option<&Vec<u64>>, cap: usize, budget: &mut u64) -> Option<bool> {
    assert_eq!(g.alpha, n.alpha);
    let alpha = n.alpha;
    let w = bs_words(n.nstates);
    let workcap = *budget;
    let mut work_done: u64 = 0;

    let mut fmask = vec![0u64; w];
    for s in 0..n.nstates { if n.accept[s] { bs_set(&mut fmask, s); } }

    // "L(a) subset L(b)": every state of `a` is simulated by some state of `b`
    // (plain set inclusion when no simulation was computed).
    let covered = |a: &[u64], b: &[u64]| -> bool {
        match sim {
            None => bs_subset(a, b),
            Some(sm) => {
                for wi in 0..w {
                    let mut bits = a[wi];
                    while bits != 0 {
                        let s = wi * 64 + bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        if bs_disjoint(&sm[s * w..(s + 1) * w], b) { return false; }
                    }
                }
                true
            }
        }
    };
    // drop states of `s` that another state of `s` simulates: same language, fewer bits
    let reduce = |s: &mut Vec<u64>| {
        let Some(sm) = sim else { return };
        let members: Vec<usize> = (0..n.nstates).filter(|&i| bs_get(s, i)).collect();
        for &x in &members {
            for &y in &members {
                if x == y || !bs_get(s, x) { continue; }
                let xy = bs_get(&sm[x * w..(x + 1) * w], y);      // y simulates x
                let yx = bs_get(&sm[y * w..(y + 1) * w], x);
                if xy && (!yx || y < x) { s[x / 64] &= !(1u64 << (x % 64)); }
            }
        }
    };

    let mut keep: Vec<Vec<Entry>> = (0..g.nstates).map(|_| Vec::new()).collect();
    let mut work: Vec<(u32, u32)> = Vec::new();
    let mut total = 0usize;
    let mut maxac = 0usize;

    let mut init = vec![0u64; w];
    for &s in &n.init { bs_set(&mut init, s as usize); }

    // insert (gs, set) into the antichain; Some(false) = counterexample found
    macro_rules! insert {
        ($gs:expr, $set:expr) => {{
            let gs: usize = $gs;
            let mut set: Vec<u64> = $set;
            reduce(&mut set);
            let pc = bs_count(&set);
            // popcount is a sound short-circuit only for plain set inclusion: under a
            // simulation preorder a larger set can still be covered by a smaller one.
            let bysize = sim.is_none();
            let mut dominated = false;
            for e in keep[gs].iter() {
                if e.dead || (bysize && e.pc > pc) { continue; }
                work_done += 1;
                if covered(&e.bits, &set) { dominated = true; break; }
            }
            if work_done > workcap {
                if dbg_on() { eprintln!("  [ac] antichain exceeded work budget {}", workcap); }
                *budget = 0;
                return None;
            }
            if !dominated {
                if g.accept[gs] && bs_disjoint(&set, &fmask) { return Some(false); }
                for e in keep[gs].iter_mut() {
                    if e.dead || (bysize && e.pc < pc) { continue; }
                    work_done += 1;
                    if covered(&set, &e.bits) { e.dead = true; }
                }
                total += 1;
                if total > cap {
                    if dbg_on() { eprintln!("  [ac] antichain exceeded cap {}", cap); }
                    *budget = budget.saturating_sub(work_done);
                    return None;
                }
                work.push((gs as u32, keep[gs].len() as u32));
                keep[gs].push(Entry { bits: set, pc, dead: false });
                if keep[gs].len() > maxac { maxac = keep[gs].len(); }
                dfa::peak_bump(total);
            }
        }};
    }

    insert!(0usize, init);
    let mut buf = vec![0u64; w];
    while let Some((gs, idx)) = work.pop() {
        if keep[gs as usize][idx as usize].dead { continue; }
        let cur = keep[gs as usize][idx as usize].bits.clone();
        for a in 0..alpha {
            for x in buf.iter_mut() { *x = 0; }
            for wi in 0..w {
                let mut bits = cur[wi];
                while bits != 0 {
                    let s = wi * 64 + bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    for &d in &n.trans[s * alpha + a] { bs_set(&mut buf, d as usize); }
                }
            }
            insert!(g.t(gs as usize, a), buf.clone());
        }
    }
    if dbg_on() {
        eprintln!("  [ac] universal: {} antichain elements, widest {}, {} subsumption tests, sim={}",
                  total, maxac, work_done, sim.is_some());
    }
    *budget = budget.saturating_sub(work_done);
    Some(true)
}
