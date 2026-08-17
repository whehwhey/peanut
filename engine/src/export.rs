//! `export NAME` — the automaton as one line of JSON, for the Peanut GUI.
//!
//! `NAME` is either the current sequence (`T`, the DFAO) or any predicate bound by
//! `let` / `learnfe`.  The shape is deliberately flat and array-based: the viewer wants
//! to lay out states, not to re-parse a grammar.
//!
//!   dfa   {"kind":"dfa","name":"FE","k":2,"mode":"msd","ns":"base","vars":["i","j","l"],
//!          "alpha":8,"nstates":15,"initial":0,"accepting":[0,3,..],
//!          "labels":[[0,0,0],[1,0,0],..],            // symbol -> digit per track
//!          "trans":[[t0,t1,..],..]}                  // state -> symbol -> state
//!   dfao  {"kind":"dfao","name":"T","k":2,"mode":"msd","ns":"base","nstates":2,"initial":0,
//!          "out":[0,1],"trans":[[0,1],[1,0]],
//!          "lsd":{"nstates":..,"out":[..],"trans":[[..]]}}
//!
//! Big automata are truncated to `AM_EXPORT_MAX` states (default 4000) — a graph view
//! is useless past a few thousand nodes and the JSON would be tens of megabytes.
//! Transitions leaving the exported prefix are written as -1 and `"truncated"` is true.
//! `"ns"` is the active numeration system (`"base"` for built-in base k): the digit
//! tuples in `"labels"` are that system's digits, not necessarily base-k ones.
use crate::dfa::{digit, Dfa};
use crate::dfao::Dfao;

fn max_states() -> usize {
    std::env::var("AM_EXPORT_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(4000)
}

fn mode() -> &'static str { if crate::dfa::is_lsd() { "lsd" } else { "msd" } }

/// Active numeration system name, or `"base"` for built-in base k.  The viewer needs
/// it to label digits: under `fib` the tracks are Zeckendorf digits, not base-2 ones.
fn ns() -> String {
    match crate::numsys::active() { Some(n) => n.name.clone(), None => "base".to_string() }
}

fn row(v: &[i64]) -> String {
    let mut s = String::from("[");
    for (i, x) in v.iter().enumerate() { if i > 0 { s.push(','); } s.push_str(&x.to_string()); }
    s.push(']');
    s
}

/// Serialize a `Dfa` (bound to formula-var order and any extra display `params`)
/// to the `dfa` JSON shape described in the module docs.
pub fn dfa_json(name: &str, params: &[String], a: &Dfa) -> String {
    let cap = max_states();
    let n = a.nstates.min(cap);
    let truncated = n < a.nstates;
    let mut s = String::with_capacity(n * a.alpha * 4 + 256);
    s.push_str(&format!(
        "{{\"kind\":\"dfa\",\"name\":\"{}\",\"k\":{},\"mode\":\"{}\",\"ns\":\"{}\",\"vars\":[{}],\
\"params\":[{}],\"alpha\":{},\"nstates\":{},\"shown\":{},\"truncated\":{},\"initial\":0",
        name, a.k, mode(), ns(),
        a.vars.iter().map(|v| format!("\"{}\"", v)).collect::<Vec<_>>().join(","),
        params.iter().map(|v| format!("\"{}\"", v)).collect::<Vec<_>>().join(","),
        a.alpha, a.nstates, n, truncated));
    // accepting states
    s.push_str(",\"accepting\":[");
    let mut first = true;
    for q in 0..n { if a.accept[q] { if !first { s.push(','); } s.push_str(&q.to_string()); first = false; } }
    s.push(']');
    // symbol -> per-track digits, in the automaton's own (sorted) variable order
    let tracks = a.vars.len();
    s.push_str(",\"labels\":[");
    for sym in 0..a.alpha {
        if sym > 0 { s.push(','); }
        let d: Vec<i64> = (0..tracks).map(|c| digit(sym, c, a.k) as i64).collect();
        s.push_str(&row(&d));
    }
    s.push(']');
    s.push_str(",\"trans\":[");
    for q in 0..n {
        if q > 0 { s.push(','); }
        let r: Vec<i64> = (0..a.alpha)
            .map(|x| { let t = a.t(q, x); if t < n { t as i64 } else { -1 } }).collect();
        s.push_str(&row(&r));
    }
    s.push_str("]}");
    s
}

/// Serialize a `Dfao` (in both digit orders) to the `dfao` JSON shape
/// described in the module docs.
pub fn dfao_json(d: &Dfao) -> String {
    let cap = max_states();
    let n = d.nstates.min(cap);
    let ln = d.lnstates.min(cap);
    let mut s = String::with_capacity((n + ln) * d.k * 4 + 256);
    s.push_str(&format!(
        "{{\"kind\":\"dfao\",\"name\":\"{}\",\"k\":{},\"mode\":\"{}\",\"ns\":\"{}\",\"nstates\":{},\
\"shown\":{},\"truncated\":{},\"initial\":0",
        d.name, d.k, mode(), ns(), d.nstates, n, n < d.nstates));
    s.push_str(",\"out\":[");
    for q in 0..n { if q > 0 { s.push(','); } s.push_str(&d.out[q].to_string()); }
    s.push_str("],\"trans\":[");
    for q in 0..n {
        if q > 0 { s.push(','); }
        let r: Vec<i64> = (0..d.k).map(|x| { let t = d.t(q, x); if t < n { t as i64 } else { -1 } }).collect();
        s.push_str(&row(&r));
    }
    s.push_str("]");
    // the lsd form of the same sequence, so the viewer can show either digit order
    s.push_str(&format!(",\"lsd\":{{\"nstates\":{},\"shown\":{},\"out\":[", d.lnstates, ln));
    for q in 0..ln { if q > 0 { s.push(','); } s.push_str(&d.lout[q].to_string()); }
    s.push_str("],\"trans\":[");
    for q in 0..ln {
        if q > 0 { s.push(','); }
        let r: Vec<i64> = (0..d.k)
            .map(|x| { let t = d.ltrans[q * d.k + x] as usize; if t < ln { t as i64 } else { -1 } }).collect();
        s.push_str(&row(&r));
    }
    s.push_str("]}}");
    s
}
