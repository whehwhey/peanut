//! Ostrowski numeration systems from a quadratic irrational.
//!
//! Fix `alpha in (0,1)` with an eventually periodic continued fraction
//! `alpha = [0; a_1, a_2, a_3, ...]` (equivalently: `alpha` is a quadratic
//! irrational).  Let `q_{-1} = 0`, `q_0 = 1`, `q_i = a_i q_{i-1} + q_{i-2}` be
//! the convergent denominators.  Every `n >= 0` has exactly one representation
//!
//! ```text
//!     n = sum_{i=1}^{L} b_i q_{i-1},
//!     0 <= b_1 < a_1,   0 <= b_i <= a_i (i >= 2),   b_i = a_i => b_{i-1} = 0
//! ```
//!
//! and, written msd-first as `b_L b_{L-1} ... b_1`, the admissible words of a
//! fixed length `L` are exactly `0, 1, ..., q_L - 1` **in lexicographic order**.
//! So an Ostrowski system is an ordered (radix) numeration system and drops
//! straight into `numsys.rs`, whose values are ranks: `U_l = q_l` comes out of
//! the counting table for free.
//!
//! This is the `ost` command of Walnut (`Automata/Numeration/Ostrowski.java`,
//! Baranwal-Shallit), with the same syntax `ost NAME [preperiod] [period]` and
//! the same normalisation of the continued fraction, but a different and
//! (deliberately) transparent construction of the two automata.
//!
//! ## The alignment problem, and the two automata
//!
//! Reading msd-first, the constraint on a digit depends on its continued
//! fraction **index**, which is known only once the length of the word is
//! known.  Both automata are therefore built as NFAs that *guess the length*
//! -- more precisely, guess the index class of the first digit -- and are then
//! determinized and minimised.  With `alpha_ = [0] + preperiod + period`,
//! `A = |alpha_|` and `periodIndex = |preperiod| + 1`, the index classes are
//! `1..A-1`; reading one digit moves class `i` to class `i-1`, and class
//! `periodIndex` may *also* move to class `A-1` (that is the wrap around the
//! period).  Class 1 is terminal: the word ends there.
//!
//! **Validity.**  State `(i, f)`: the next digit sits at class `i`, and `f = 1`
//! means the previous (more significant) digit was maximal, so this one must be
//! `0`.  Otherwise the digit ranges over `0..a_i` (`0..a_1 - 1` at class 1).
//!
//! **Addition.**  State `(i, u, v)`: the next digit sits at class `i`, and the
//! part of `x + y - z` read so far is `D = u q_i + v q_{i-1}`.  Reading digits
//! `(x, y, z)` at class `i` with `s = x + y - z`,
//!
//! ```text
//!     D' = D + s q_{i-1} = (a_i u + v + s) q_{i-1} + u q_{i-2},
//! ```
//!
//! so `(u, v) -> (a_i u + v + s, u)`, started at `(0, 0)`, and at the end
//! (`class 1` consumed) `D = u q_0 + v q_{-1} = u`, so accept iff `u = 0`.
//! Since `x, y, z < q_i` on valid inputs, `D` is confined to
//! `[-2(q_i - 1), q_i - 1]`, which is what keeps `(u, v)` finite; the search
//! uses an explicit cap and *proves after the fact* that the cap did not bind,
//! by checking that no state surviving the trim sits on the boundary.

use crate::dfa::{Dfa, Nfa, State};

/// A normalised continued fraction `alpha = [0; a_1, a_2, ...]`.
#[derive(Clone, Debug)]
pub struct Cf {
    /// `alpha_[0] = 0`, then the preperiod, then the period.
    pub alpha: Vec<u32>,
    /// index in `alpha` where the period starts (`= preperiod.len() + 1`).
    pub period_index: usize,
    /// largest digit of the numeration system.
    pub dmax: usize,
}

impl Cf {
    /// Walnut's normalisation (`Ostrowski.java`): drop leading zeros of the
    /// preperiod; if the preperiod is empty, unroll one copy of the period into
    /// it; and if `a_1 = 1`, fold it away with `[0;1,a_2,a_3,..] ->
    /// [0;a_2+1,a_3,..]`, which deletes the degenerate `q_1 = q_0 = 1` and
    /// leaves the same weight sequence.
    pub fn new(preperiod: &[u32], period: &[u32]) -> Result<Cf, String> {
        let mut pre: Vec<u32> = preperiod.to_vec();
        let mut per: Vec<u32> = period.to_vec();
        while pre.first() == Some(&0) { pre.remove(0); }
        if per.is_empty() { return Err("the period cannot be empty".into()); }
        if per.iter().any(|&d| d == 0) || pre.iter().any(|&d| d == 0) {
            return Err("all continued-fraction partial quotients must be positive".into());
        }
        if pre.is_empty() { pre = per.clone(); }
        if pre[0] == 1 {
            if pre.len() > 1 { pre[0] = pre[1] + 1; pre.remove(1); }
            else { pre[0] = per[0] + 1; let f = per.remove(0); per.push(f); }
        }
        if pre[0] == 1 { return Err("could not normalise a_1 = 1 away".into()); }
        let mut alpha = vec![0u32];
        alpha.extend(&pre);
        let period_index = alpha.len();
        alpha.extend(&per);
        let dmax = alpha[1..].iter().enumerate()
            .map(|(j, &a)| if j == 0 { a - 1 } else { a }).max().unwrap() as usize;
        Ok(Cf { alpha, period_index, dmax })
    }

    /// `a_i` for any `i >= 1` (the tail is periodic).
    pub fn a(&self, i: usize) -> u32 {
        let n = self.alpha.len();
        let idx = if i < n { i } else { self.period_index + (i - n) % (n - self.period_index) };
        self.alpha[idx]
    }

    /// Convergent denominators `q_0, q_1, ..., q_l`.
    pub fn q(&self, l: usize) -> Vec<u128> {
        let mut v = vec![1u128];
        let mut prev = 0u128;
        for i in 1..=l {
            let next = self.a(i) as u128 * v[i - 1] + prev;
            prev = v[i - 1];
            v.push(next);
        }
        v
    }

    /// Number of index classes: `1 ..= nclass()`.
    fn nclass(&self) -> usize { self.alpha.len() - 1 }

    /// Classes reachable in one step from class `i` (empty: class 1 is terminal).
    fn next_classes(&self, i: usize) -> Vec<usize> {
        let mut v = Vec::new();
        if i > 1 { v.push(i - 1); }
        if i == self.period_index { v.push(self.alpha.len() - 1); }
        v
    }

    /// Could a state whose difference is `D = u q_j + v q_{j-1}`, with the next
    /// digit at class `i`, still be completed to `x + y = z` on *valid*
    /// representations?  Then `D` lies in `[-2(q_j - 1), q_j - 1]`, i.e.
    /// `u + v r_j in [-2, 1]` with `r_j = q_{j-1}/q_j = 1/(a_j + r_{j-1})`, so
    /// `r_j in [1/(a_i + 1), 1/a_i]` -- a bound that depends only on the class.
    /// Exact rational test; sound (it never discards a state that occurs in a
    /// genuine addition), which is all correctness needs.
    fn feasible(&self, i: usize, u: i64, v: i64) -> bool {
        let a = self.a(i) as i128;
        let (u, v) = (u as i128, v as i128);
        // endpoints u + v/a and u + v/(a+1), ordered by the sign of v
        let (lo_d, hi_d) = if v >= 0 { (a + 1, a) } else { (a, a + 1) };
        // min = u + v/lo_d <= 1   and   max = u + v/hi_d >= -2
        (u * lo_d + v <= lo_d) && (u * hi_d + v >= -2 * hi_d)
    }

    /// Digits `alpha` accepts at class `i` when the previous digit was not
    /// maximal: `0..a_i`, but `0..a_1 - 1` at the last position.
    fn max_digit(&self, i: usize) -> usize {
        if i == 1 { (self.a(1) - 1) as usize } else { self.a(i) as usize }
    }
}

// ------------------------------------------------------------------ validity

/// The msd validity automaton of the Ostrowski system for `cf`.
pub fn validity(cf: &Cf) -> Dfa {
    let d = cf.dmax + 1;
    let nc = cf.nclass();
    // states: (class-1)*2 + flag, then the accepting sink
    let idx = |i: usize, f: usize| (i - 1) * 2 + f;
    let fin = nc * 2;
    let n = fin + 1;
    let mut trans: Vec<Vec<State>> = vec![Vec::new(); n * d];
    for i in 1..=nc {
        for f in 0..2 {
            let hi = if f == 1 { 0 } else { cf.max_digit(i) };
            for dig in 0..=hi.min(cf.dmax) {
                let nf = if f == 0 && dig == cf.a(i) as usize { 1 } else { 0 };
                if i == 1 {
                    trans[idx(i, f) * d + dig].push(fin as State);
                } else {
                    for j in cf.next_classes(i) { trans[idx(i, f) * d + dig].push(idx(j, nf) as State); }
                }
            }
        }
    }
    let mut accept = vec![false; n];
    accept[fin] = true;
    let init: Vec<State> = (1..=nc).map(|i| idx(i, 0) as State).collect();
    Nfa { k: d, vars: vec!["\u{1}a".into()], alpha: d, nstates: n, trans, init, accept }
        .determinize().minimize()
}

// ------------------------------------------------------------------ addition

/// The msd adder `(x, y, z) : x + y = z` of the Ostrowski system for `cf`.
/// `cap` bounds `|u|, |v|`; the construction fails rather than truncate.
pub fn addition(cf: &Cf, cap: i64) -> Result<Dfa, String> {
    let d = cf.dmax + 1;
    let alpha = d * d * d;
    let nc = cf.nclass();
    // BFS over (class, u, v)
    use std::collections::HashMap;
    let mut index: HashMap<(usize, i64, i64), usize> = HashMap::new();
    let mut order: Vec<(usize, i64, i64)> = Vec::new();
    for i in 1..=nc {
        index.insert((i, 0, 0), order.len());
        order.push((i, 0, 0));
    }
    let mut edges: Vec<Vec<(usize, usize)>> = Vec::new(); // (symbol, target index) or FIN
    let fin = usize::MAX;
    let mut q = 0usize;
    while q < order.len() {
        let (i, u, v) = order[q];
        let a = cf.a(i) as i64;
        let mut e = Vec::new();
        for x in 0..d { for y in 0..d { for z in 0..d {
            let s = x as i64 + y as i64 - z as i64;
            let u2 = a * u + v + s;
            let sym = x + y * d + z * d * d;
            if i == 1 {
                if u2 == 0 { e.push((sym, fin)); }
                continue;
            }
            for j in cf.next_classes(i) {
                if !cf.feasible(j, u2, u) { continue; }
                if u2.abs() > cap || u.abs() > cap { return Err(format!(
                    "Ostrowski adder: carry ({}, {}) exceeded the cap {}", u2, u, cap)); }
                let key = (j, u2, u);
                let t = match index.get(&key) {
                    Some(&t) => t,
                    None => { index.insert(key, order.len()); order.push(key); order.len() - 1 }
                };
                e.push((sym, t));
            }
        }}}
        edges.push(e);
        q += 1;
        if order.len() > 2_000_000 { return Err("Ostrowski adder: too many carry states".into()); }
    }
    let n = order.len() + 1;
    let f = order.len();
    let mut trans: Vec<Vec<State>> = vec![Vec::new(); n * alpha];
    for (s, e) in edges.iter().enumerate() {
        for &(sym, t) in e {
            trans[s * alpha + sym].push(if t == fin { f as State } else { t as State });
        }
    }
    // Prove the cap did not bind: no state that both is reachable from an
    // initial state and can still reach acceptance may sit on the boundary.
    let mut co = vec![false; n];
    co[f] = true;
    loop {
        let mut ch = false;
        for s in 0..n {
            if co[s] { continue; }
            'o: for sym in 0..alpha {
                for &t in &trans[s * alpha + sym] { if co[t as usize] { co[s] = true; ch = true; break 'o; } }
            }
        }
        if !ch { break; }
    }
    let mut reach = vec![false; n];
    let mut stack: Vec<usize> = (0..nc).collect();
    for &s in &stack { reach[s] = true; }
    while let Some(s) = stack.pop() {
        for sym in 0..alpha {
            for &t in &trans[s * alpha + sym] {
                if !reach[t as usize] { reach[t as usize] = true; stack.push(t as usize); }
            }
        }
    }
    for s in 0..order.len() {
        let (_, u, v) = order[s];
        if reach[s] && co[s] && (u.abs() == cap || v.abs() == cap) {
            return Err(format!("Ostrowski adder: the carry cap {} binds", cap));
        }
    }
    let mut accept = vec![false; n];
    accept[f] = true;
    let init: Vec<State> = (0..nc).map(|i| i as State).collect();
    Ok(Nfa { k: d, vars: vec!["\u{1}a".into(), "\u{1}b".into(), "\u{1}c".into()],
             alpha, nstates: n, trans, init, accept }.determinize().minimize())
}

/// The empty word is the representation of 0 (Walnut's convention and this
/// engine's), so both automata must accept it.
fn with_empty_word(v: &Dfa) -> Dfa {
    if v.accept[0] { return v.clone(); }
    let mut acc = v.accept.clone();
    acc[0] = true;
    Dfa { accept: acc, ..v.clone() }.minimize()
}

/// Adders are always used conjoined with "every track is a valid
/// representation", so their behaviour on a triple in which some track is
/// *already* an invalid prefix is a don't-care: the whole tail of the run is
/// unconstrained.  Exact don't-care minimisation is NP-hard, but the freedom
/// here has a simple shape -- one "we have left the care region" sink, whose
/// behaviour may be *anything* -- so we sweep it over every state of the
/// product (plus an accepting and a rejecting sink), minimise each time, and
/// keep the smallest.  Every candidate agrees with `a` on all-valid triples by
/// construction, so this cannot change any answer.
fn dont_care_reduce(a: &Dfa, valid: &Dfa) -> Dfa {
    let names = ["\u{1}a", "\u{1}b", "\u{1}c"];
    let mut v3: Option<Dfa> = None;
    for n in names {
        let t = Dfa { vars: vec![n.to_string()], ..valid.clone() };
        v3 = Some(match v3 { None => t, Some(p) => p.and(&t) });
    }
    let v3 = v3.unwrap().minimize();
    if v3.alpha != a.alpha { return a.clone(); }
    // states of v3 from which some word is still accepted
    let mut live = v3.accept.clone();
    loop {
        let mut ch = false;
        for s in 0..v3.nstates {
            if !live[s] { for x in 0..v3.alpha { if live[v3.t(s, x)] { live[s] = true; ch = true; break; } } }
        }
        if !ch { break; }
    }
    // reachable product states; every edge that leaves the care region is
    // rerouted to the placeholder `sink`
    let mut index: std::collections::HashMap<(usize, usize), usize> = Default::default();
    let mut order: Vec<(usize, usize)> = vec![(0, 0)];
    index.insert((0, 0), 0);
    let mut edge: Vec<i64> = Vec::new(); // -1 = sink
    let mut i = 0;
    while i < order.len() {
        let (p, q) = order[i];
        for x in 0..a.alpha {
            let (p2, q2) = (a.t(p, x), v3.t(q, x));
            if !live[q2] { edge.push(-1); continue; }
            let k = (p2, q2);
            let t = *index.entry(k).or_insert_with(|| { order.push(k); order.len() - 1 });
            edge.push(t as i64);
        }
        i += 1;
        if order.len() > 20_000 { return a.clone(); }
    }
    let n = order.len();
    let rej = n;
    let acc = n + 1;
    let total = n + 2;
    let mut best = a.clone();
    for cand in 0..total {
        let mut trans = vec![0u32; total * a.alpha];
        for s in 0..n {
            for x in 0..a.alpha {
                let e = edge[s * a.alpha + x];
                trans[s * a.alpha + x] = if e < 0 { cand as u32 } else { e as u32 };
            }
        }
        for x in 0..a.alpha { trans[rej * a.alpha + x] = rej as u32; trans[acc * a.alpha + x] = acc as u32; }
        let mut accept: Vec<bool> = (0..total).map(|s| if s < n { a.accept[order[s].0] && v3.accept[order[s].1] } else { s == acc }).collect();
        accept[rej] = false;
        let m = Dfa::new(a.k, a.vars.clone(), total, trans, accept).minimize();
        if m.nstates < best.nstates { best = m; }
    }
    best
}

// ------------------------------------------------------------------ serialisation

/// Serialise a `Dfa` in Walnut's "Custom Bases" text format: an alphabet line
/// with one `{0,..,D-1}` per track, then `state output` followed by its
/// transitions.  States that cannot reach acceptance are dropped (as in
/// Walnut's own files, a missing transition means dead), and the start state is
/// written first.
pub fn to_walnut(a: &Dfa, tracks: usize, header: &str) -> String {
    let d = a.k;
    let mut co = a.accept.clone();
    loop {
        let mut ch = false;
        for s in 0..a.nstates {
            if !co[s] { for x in 0..a.alpha { if co[a.t(s, x)] { co[s] = true; ch = true; break; } } }
        }
        if !ch { break; }
    }
    let keep: Vec<usize> = (0..a.nstates).filter(|&s| co[s]).collect();
    let mut id = vec![usize::MAX; a.nstates];
    for (n, &s) in keep.iter().enumerate() { id[s] = n; }
    let mut out = String::new();
    for line in header.lines() { out += &format!("# {}\n", line); }
    out.push('\n');
    let set = format!("{{{}}}", (0..d).map(|x| x.to_string()).collect::<Vec<_>>().join(","));
    out += &vec![set; tracks].join(" ");
    out.push('\n');
    for &s in &keep {
        out += &format!("\n{} {}\n", id[s], if a.accept[s] { 1 } else { 0 });
        for x in 0..a.alpha {
            let t = a.t(s, x);
            if !co[t] { continue; }
            let digits: Vec<String> = (0..tracks).map(|c| crate::dfa::digit(x, c, d).to_string()).collect();
            out += &format!("{} -> {}\n", digits.join(" "), id[t]);
        }
    }
    out
}

/// Build both automata and write `msd_NAME.txt` / `msd_NAME_addition.txt` into
/// `dir`.  Returns `(paths, validity states, adder states, first weights)`.
pub fn generate(name: &str, pre: &[u32], per: &[u32], dir: &std::path::Path)
    -> Result<(Vec<String>, usize, usize, Vec<u128>), String> {
    let cf = Cf::new(pre, per)?;
    let v = with_empty_word(&validity(&cf));
    let mut cap = 16i64;
    let a0 = loop {
        match addition(&cf, cap) {
            Ok(a) => break a,
            Err(e) if cap < 512 => { cap *= 4; let _ = e; }
            Err(e) => return Err(e),
        }
    };
    // The empty word is the representation of 0 on every track, so `0 + 0 = 0`
    // must be accepted there as well (Walnut's adders do).
    let a = with_empty_word(&dont_care_reduce(&a0, &v));
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
    let cfs = format!("alpha = [0; {}, bar({})]",
                      cf.alpha[1..cf.period_index].iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "),
                      cf.alpha[cf.period_index..].iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "));
    let mut paths = Vec::new();
    for (suffix, txt, tracks, what) in [
        ("", &v, 1usize, "valid Ostrowski representations, msd, leading zeros allowed"),
        ("_addition", &a, 3usize, "(x,y,z) with x + y = z"),
    ] {
        let p = dir.join(format!("msd_{name}{suffix}.txt"));
        let head = format!("msd_{name}{suffix}: Ostrowski numeration for {cfs}\n\
                            {what}\nweights q_l = {:?}\ngenerated by Peanut `ost {name} [{}] [{}]`",
                           cf.q(8), pre.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "),
                           per.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "));
        std::fs::write(&p, to_walnut(txt, tracks, &head))
            .map_err(|e| format!("{}: {}", p.display(), e))?;
        paths.push(p.display().to_string());
    }
    Ok((paths, v.nstates, a.nstates, cf.q(8)))
}
