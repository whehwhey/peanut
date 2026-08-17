//! Deterministic finite automata with output: the k-automatic sequences themselves.

use crate::dfa::{Dfa, State, is_lsd};
use crate::numsys;

/// A DFA-with-output: a base-`k` automatic sequence `T`, defined by an
/// automaton over single digits whose state reached by reading `n`'s digits
/// labels the output `T[n]`. Kept in *both* digit orders simultaneously
/// (`trans`/`out` msd-first, `ltrans`/`lout` lsd-first, built once by
/// [`build_lsd`]) so the active [`crate::dfa::is_lsd`] mode can be switched at
/// runtime without recomputing the sequence.
#[derive(Clone, Debug)]
pub struct Dfao {
    pub k: usize,
    pub nstates: usize,
    pub trans: Vec<State>, // nstates * k, msd-first
    pub out: Vec<u8>,
    pub name: String,
    // least-significant-digit-first form of the same sequence
    pub lnstates: usize,
    pub ltrans: Vec<State>,
    pub lout: Vec<u8>,
}

impl Dfao {
    /// Build from a k-uniform morphism sigma on {0..m-1} with coding `coding`,
    /// taking the fixed point starting from letter `start` (requires sigma(start)[0] == start).
    pub fn from_morphism(k: usize, m: usize, sigma: &[Vec<u8>], coding: &[u8], start: usize, name: &str) -> Result<Dfao, String> {
        if sigma.len() != m { return Err(format!("sigma has {} words, expected {}", sigma.len(), m)); }
        for (a, w) in sigma.iter().enumerate() {
            if w.len() != k { return Err(format!("word {} has length {}, expected {}", a, w.len(), k)); }
        }
        if sigma[start][0] as usize != start { return Err("morphism not prolongable on start letter".into()); }
        // Relabel so that `start` becomes state 0 (needed: state 0 must be the start state).
        let relab = |a: usize| if a == start { 0 } else if a == 0 { start } else { a };
        let mut trans = vec![0u32; m * k];
        let mut out = vec![0u8; m];
        for a in 0..m {
            let orig = relab(a);
            out[a] = coding[orig];
            for d in 0..k { trans[a * k + d] = relab(sigma[orig][d] as usize) as u32; }
        }
        let (lnstates, ltrans, lout) = build_lsd(k, m, &trans, &out)?;
        Ok(Dfao { k, nstates: m, trans, out, name: name.to_string(), lnstates, ltrans, lout })
    }

    /// Build directly from an explicit msd transition table (`nstates * k`) and
    /// per-state outputs -- the `dfao` command, and the way a sequence over a
    /// non-uniform numeration system (Fibonacci, Tribonacci, ...) enters the
    /// engine, since it is not the fixed point of a k-uniform morphism.
    pub fn from_tables(k: usize, nstates: usize, trans: Vec<State>, out: Vec<u8>, name: &str) -> Result<Dfao, String> {
        if trans.len() != nstates * k { return Err(format!("expected {} transitions, got {}", nstates * k, trans.len())); }
        if out.len() != nstates { return Err(format!("expected {} outputs, got {}", nstates, out.len())); }
        if let Some(&bad) = trans.iter().find(|&&t| t as usize >= nstates) {
            return Err(format!("transition target {} >= nstates {}", bad, nstates));
        }
        if trans[0] != 0 { return Err("state 0 must loop on digit 0 (leading zeros must not change the value)".into()); }
        let (lnstates, ltrans, lout) = build_lsd(k, nstates, &trans, &out)?;
        Ok(Dfao { k, nstates, trans, out, name: name.to_string(), lnstates, ltrans, lout })
    }

    /// Active transition table / outputs for the current digit order.
    pub fn active(&self) -> (usize, &Vec<State>, &Vec<u8>) {
        if is_lsd() { (self.lnstates, &self.ltrans, &self.lout) }
        else { (self.nstates, &self.trans, &self.out) }
    }

    #[inline]
    pub fn t(&self, s: usize, d: usize) -> usize { self.trans[s * self.k + d] as usize }

    /// The n-th term of the sequence.  Always evaluated on the msd automaton,
    /// reading `n`'s canonical representation in the active numeration system
    /// (plain base-k digits when none is active).
    pub fn at(&self, n: u64) -> u8 {
        if n == 0 { return self.out[0]; }
        let digits: Vec<usize> = match numsys::active() {
            Some(ns) => ns.rep(n),
            None => {
                let mut d = Vec::new();
                let mut m = n;
                while m > 0 { d.push((m % self.k as u64) as usize); m /= self.k as u64; }
                d.reverse(); d
            }
        };
        let mut s = 0usize;
        for d in digits { s = self.t(s, d); }
        self.out[s]
    }

    /// The first `n` terms `T[0..n)`.
    pub fn prefix(&self, n: usize) -> Vec<u8> { (0..n as u64).map(|i| self.at(i)).collect() }

    /// Distinct output values the sequence takes, sorted ascending.
    pub fn out_alphabet(&self) -> Vec<u8> {
        let mut v: Vec<u8> = self.out.clone();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// Padding-robustness: for msd, delta(q0,0) == q0.  For lsd the construction
    /// guarantees out(g . 0) == out(g) for every state g, which is checked here.
    pub fn zero_stable(&self) -> bool {
        if is_lsd() {
            (0..self.lnstates).all(|q| self.lout[self.ltrans[q * self.k] as usize] == self.lout[q])
        } else { self.t(0, 0) == 0 }
    }

    /// DFA over [x] accepting base-k representations of positions i with T[i] = a.
    pub fn pred_letter(&self, x: &str, a: u8) -> Dfa {
        assert!(self.zero_stable());
        let (n, tr, out) = self.active();
        let accept = out.iter().map(|&o| o == a).collect();
        let d = Dfa::new(self.k, vec![x.to_string()], n, tr.clone(), accept).minimize();
        numsys::restrict(&d).minimize()
    }

    /// DFA over sorted [x,y] accepting (i,j) with T[i] = T[j].
    pub fn pred_eq(&self, x: &str, y: &str) -> Dfa {
        assert!(self.zero_stable());
        let k = self.k;
        let (n, tr, out) = self.active();
        let alpha = k * k;
        let mut trans = vec![0u32; n * n * alpha];
        for p in 0..n {
            for q in 0..n {
                for a in 0..k {
                    for b in 0..k {
                        trans[(p * n + q) * alpha + a + b * k] =
                            (tr[p * k + a] as usize * n + tr[q * k + b] as usize) as u32;
                    }
                }
            }
        }
        let accept = (0..n * n).map(|s| out[s / n] == out[s % n]).collect();
        let mut vars = vec![x.to_string(), y.to_string()];
        let d = Dfa::new(k, vec!["\u{1}a".into(), "\u{1}b".into()], n * n, trans, accept);
        let mut out = Dfa { vars: vec![x.into(), y.into()], ..d };
        vars.sort();
        if vars != out.vars { out = out.extend_vars(&vars); }
        numsys::restrict(&out).minimize()
    }
}


/// Convert an msd-first DFAO into an equivalent lsd-first DFAO.
///
/// Reading the base-k digits of n from the least significant end, the state is the
/// transformation g : Q -> Q with g(q) = delta*(q, d_{L-1} ... d_0).  Starting from
/// the identity, a new (more significant) digit d updates g'(q) = g(delta(q,d)).
/// The output is tau(g(q0)).  These states form the transition monoid, then Moore
/// minimisation collapses it.
fn build_lsd(k: usize, m: usize, trans: &[State], out: &[u8]) -> Result<(usize, Vec<State>, Vec<u8>), String> {
    use std::collections::HashMap;
    // The transformation g : Q -> Q is stored as a Vec<State>, not Vec<u8>: `m` is a
    // state count and routinely exceeds 255 for a DFAO loaded with `dfao` (before
    // 2026-08-17 this was `(0..m as u8)`, which silently truncated m mod 256 and then
    // panicked with an out-of-bounds index on any sequence with >= 256 states).
    let id: Vec<State> = (0..m as State).collect();
    let mut index: HashMap<Vec<State>, usize> = HashMap::new();
    let mut order: Vec<Vec<State>> = Vec::new();
    index.insert(id.clone(), 0);
    order.push(id);
    let mut lt: Vec<State> = Vec::new();
    let mut i = 0;
    while i < order.len() {
        let g = order[i].clone();
        for d in 0..k {
            let ng: Vec<State> = (0..m).map(|q| g[trans[q * k + d] as usize]).collect();
            let id_ = match index.get(&ng) {
                Some(&x) => x,
                None => { let x = order.len(); index.insert(ng.clone(), x); order.push(ng); x }
            };
            lt.push(id_ as State);
        }
        i += 1;
        if order.len() >= 2_000_000 { return Err("transition monoid too large (max 2_000_000 states)".into()); }
    }
    let lo: Vec<u8> = order.iter().map(|g| out[g[0] as usize]).collect();
    // Moore minimisation of this DFAO
    let n = order.len();
    let mut color: Vec<u32> = lo.iter().map(|&x| x as u32).collect();
    loop {
        let mut sig: HashMap<Vec<u32>, u32> = HashMap::new();
        let mut newc = vec![0u32; n];
        for q in 0..n {
            let mut key = vec![color[q]];
            for d in 0..k { key.push(color[lt[q * k + d] as usize]); }
            let next = sig.len() as u32;
            newc[q] = *sig.entry(key).or_insert(next);
        }
        if sig.len() == color.iter().collect::<std::collections::HashSet<_>>().len() { color = newc; break; }
        color = newc;
    }
    let nc = *color.iter().max().unwrap() as usize + 1;
    let mut relabel = vec![usize::MAX; nc];
    relabel[color[0] as usize] = 0;
    let mut cnt = 1;
    for q in 0..n { let c = color[q] as usize; if relabel[c] == usize::MAX { relabel[c] = cnt; cnt += 1; } }
    let mut ntrans = vec![0u32; cnt * k];
    let mut nout = vec![0u8; cnt];
    for q in 0..n {
        let c = relabel[color[q] as usize];
        nout[c] = lo[q];
        for d in 0..k { ntrans[c * k + d] = relabel[color[lt[q * k + d] as usize] as usize] as u32; }
    }
    Ok((cnt, ntrans, nout))
}
