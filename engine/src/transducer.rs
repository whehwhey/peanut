//! Finite-state transducers, and Dekking-style transduction of an automatic
//! sequence.
//!
//! A **transducer** here is exactly Walnut's: a deterministic, 1-uniform,
//! all-states-final finite-state machine `(P, p0, tau, sigma)` over an input
//! alphabet `A` (the *output* alphabet of the sequence it will be applied to)
//! and an output alphabet `B`.  Fed the infinite word `x = x_0 x_1 x_2 ...` it
//! emits `y_n = sigma(p_n, x_n)` where `p_0 = p0` and `p_{n+1} = tau(p_n, x_n)`
//! -- one output letter per input letter, the running state carrying whatever
//! left-to-right accumulation the transducer encodes (running sums, run-length
//! parities, ...).
//!
//! **Theorem (Dekking 1994).**  If `x` is `k`-automatic then so is `y`.  The
//! proof is effective and is what [`transduce`] implements.
//!
//! The construction.  Read a DFAO `M` as a morphism: state `q` is a letter and
//! `h(q) = delta(q, e_0) delta(q, e_1) ...` over the digits `e` that `q` may
//! legally read.  Then `x` is the coding `O` of the fixed point of `h` at `q0`,
//! and for `n` with msd representation `d_1 ... d_m` the prefix `x[0..n)` is
//!
//! ```text
//!     h^{m-1}(s_1) h^{m-2}(s_2) ... h^0(s_m),
//!     s_i = the first d_i letters of h(q_{i-1}),  q_i = delta(q_{i-1}, d_i)
//! ```
//!
//! (Dumont-Thomas).  Write `phi_w : P -> P` for "run the transducer over the
//! coding of the state word `w`".  The transducer state at position `n` is
//! `phi_{x[0..n)}(p0)`, so a DFAO for `y` may carry, per state, the *word* `w`
//! accumulated so far -- except that `w` grows without bound.  What is finite
//! is the tuple
//!
//! ```text
//!     (delta(q0, d_1..d_m),  phi_w, phi_{h(w)}, ..., phi_{h^{L-1}(w)})
//! ```
//!
//! because `i -> (phi_{h^i(c)})_{c in Q}` is eventually periodic (finitely many
//! tuples of maps `P -> P`), with preperiod `Q_` and period `P_`; `L = Q_ + P_`.
//! Reading one more digit `d` from state `(q, f_0..f_{L-1})` gives
//! `w' = h(w) s` with `s = h(q)[0..d)`, hence
//!
//! ```text
//!     f'_i = phi_{h^i(s)} . f_{wrap(i+1)},    wrap(m) = m if m < L else Q_ + (m-Q_) mod P_
//! ```
//!
//! which is a genuine transition function -- no strings are stored.  (Walnut
//! stores the string `w` itself and recomputes the iterates from it, which is
//! exponential in the BFS depth; the recurrence above is the same automaton at
//! `O(|P| * L)` work per edge.)  Output of a state is `sigma(f_0(p0), O(q))`.
//!
//! Under a numeration system the "digits `q` may legally read" are the ones the
//! validity automaton allows, so `M` is first crossed with the validity DFA;
//! for base `k` every digit is legal and the cross is the identity.  This is
//! what makes the same code transduce a Fibonacci- or Pell-automatic word.

use crate::dfa::State;
use crate::dfao::Dfao;
use crate::numsys;
use std::collections::HashMap;

/// A deterministic 1-uniform transducer, all states final.  State 0 is the
/// initial state (the first one declared in the file).
#[derive(Clone, Debug)]
pub struct Transducer {
    pub name: String,
    pub nstates: usize,
    /// Input alphabet, ascending.  Position in this vector is the letter index.
    pub letters: Vec<i64>,
    /// `tau[p * na + a]` -- next state.
    pub tau: Vec<u32>,
    /// `sigma[p * na + a]` -- output letter.
    pub sigma: Vec<i64>,
}

impl Transducer {
    pub fn na(&self) -> usize { self.letters.len() }
    fn letter_index(&self, v: i64) -> Option<usize> { self.letters.iter().position(|&x| x == v) }

    /// Run the transducer over `x`, returning `y` -- the ground truth that
    /// [`transduce`] must reproduce, used by the cross-checks.
    pub fn apply(&self, x: &[u8]) -> Option<Vec<i64>> {
        let mut p = 0usize;
        let mut y = Vec::with_capacity(x.len());
        for &c in x {
            let a = self.letter_index(c as i64)?;
            y.push(self.sigma[p * self.na() + a]);
            p = self.tau[p * self.na() + a] as usize;
        }
        Some(y)
    }
}

// ------------------------------------------------------------------ parsing

fn strip(line: &str) -> &str {
    let l = match line.find('#') { Some(p) => &line[..p], None => line };
    let l = match l.find("//") { Some(p) => &l[..p], None => l };
    l.trim()
}

/// Parse Walnut's `Transducer Library/*.txt` format:
///
/// ```text
/// {0, 1}
///
/// 0
/// 0 -> 0 / 0
/// 1 -> 1 / 1
/// ```
///
/// One alphabet set (transducers take a single input track), then one block per
/// state: a bare state number, then `<input> -> <new state> / <output>` lines.
/// `*` on the input side is a wildcard over the whole alphabet.
pub fn parse(name: &str, text: &str) -> Result<Transducer, String> {
    let mut it = text.lines().map(strip).filter(|l| !l.is_empty());
    let alpha_line = it.next().ok_or("empty transducer file")?;
    let open = alpha_line.find('{').ok_or("first line must be an alphabet such as {0, 1}")?;
    let close = alpha_line.find('}').ok_or("unterminated { in the alphabet line")?;
    if alpha_line[close + 1..].contains('{') {
        return Err("a transducer takes exactly one input track".into());
    }
    let mut letters: Vec<i64> = Vec::new();
    for t in alpha_line[open + 1..close].split(',') {
        let t = t.trim();
        if t.is_empty() { continue; }
        letters.push(t.parse::<i64>().map_err(|_| format!("bad alphabet element {:?}", t))?);
    }
    letters.sort_unstable();
    letters.dedup();
    if letters.is_empty() { return Err("empty input alphabet".into()); }
    let na = letters.len();
    let index = |v: i64| letters.iter().position(|&x| x == v);

    // state number (as written) -> (tau row, sigma row); declaration order recorded
    let mut order: Vec<usize> = Vec::new();
    let mut tau: HashMap<usize, Vec<Option<u32>>> = HashMap::new();
    let mut sig: HashMap<usize, Vec<Option<i64>>> = HashMap::new();
    let mut cur: Option<usize> = None;
    for line in it {
        if let Some(arrow) = line.find("->") {
            let q = cur.ok_or("transition before any state declaration")?;
            let lhs = line[..arrow].trim();
            let rhs = line[arrow + 2..].trim();
            let (dst, outp) = rhs.split_once('/')
                .ok_or_else(|| format!("transition {:?} has no ' / <output>' part", line))?;
            let dst: usize = dst.trim().parse().map_err(|_| format!("bad destination in {:?}", line))?;
            let outp: i64 = outp.trim().parse().map_err(|_| format!("bad output in {:?}", line))?;
            let ins: Vec<usize> = if lhs == "*" {
                (0..na).collect()
            } else {
                let v: i64 = lhs.parse().map_err(|_| format!("bad input letter {:?}", lhs))?;
                vec![index(v).ok_or_else(|| format!("input letter {} is not in the alphabet", v))?]
            };
            for a in ins {
                tau.get_mut(&q).unwrap()[a] = Some(dst as u32);
                sig.get_mut(&q).unwrap()[a] = Some(outp);
            }
        } else {
            let q: usize = line.parse().map_err(|_| format!("bad state declaration {:?}", line))?;
            if tau.contains_key(&q) { return Err(format!("state {} declared twice", q)); }
            order.push(q);
            tau.insert(q, vec![None; na]);
            sig.insert(q, vec![None; na]);
            cur = Some(q);
        }
    }
    if order.is_empty() { return Err("no states".into()); }
    // Walnut indexes sigma by the literal state number, so the states must be
    // 0..Q-1; we additionally relabel so the first-declared state is 0.
    let n = order.len();
    let mut seen: Vec<bool> = vec![false; n];
    for &q in &order {
        if q >= n { return Err(format!("states must be numbered 0..{}, saw {}", n - 1, q)); }
        seen[q] = true;
    }
    if !seen.iter().all(|&b| b) { return Err(format!("states must be exactly 0..{}", n - 1)); }
    let start = order[0];
    let relab = |q: usize| if q == start { 0 } else if q == 0 { start } else { q };

    let mut t = vec![0u32; n * na];
    let mut s = vec![0i64; n * na];
    for q in 0..n {
        let src = relab(q);
        for a in 0..na {
            let d = tau[&src][a].ok_or_else(|| format!(
                "state {} has no transition on input {} (a transducer must be total)", src, letters[a]))?;
            if d as usize >= n { return Err(format!("state {}: destination {} out of range", src, d)); }
            t[q * na + a] = relab(d as usize) as u32;
            s[q * na + a] = sig[&src][a].unwrap();
        }
    }
    Ok(Transducer { name: name.to_string(), nstates: n, letters, tau: t, sigma: s })
}

/// `transducer NAME D q0:t/o,t/o,.. q1:..` -- the same machine typed inline,
/// over the letter alphabet `{0..D-1}`.
pub fn parse_inline(name: &str, toks: &[&str]) -> Result<Transducer, String> {
    if toks.len() < 2 { return Err("usage: transducer NAME D q0:t/o,t/o,.. ..".into()); }
    let na: usize = toks[0].parse().map_err(|_| "bad letter-alphabet size")?;
    if na < 1 { return Err("letter alphabet must be non-empty".into()); }
    let rows = &toks[1..];
    let n = rows.len();
    let mut tau = vec![0u32; n * na];
    let mut sigma = vec![0i64; n * na];
    for (q, row) in rows.iter().enumerate() {
        let body = row.split_once(':').map(|x| x.1).unwrap_or(row);
        let parts: Vec<&str> = body.split(',').collect();
        if parts.len() != na { return Err(format!("state {}: {} entries, expected {}", q, parts.len(), na)); }
        for (a, p) in parts.iter().enumerate() {
            let (t, o) = p.split_once('/').ok_or_else(|| format!("state {}: expected target/output, got {:?}", q, p))?;
            let t: usize = t.trim().parse().map_err(|_| format!("state {}: bad target {:?}", q, t))?;
            if t >= n { return Err(format!("state {}: target {} >= {} states", q, t, n)); }
            tau[q * na + a] = t as u32;
            sigma[q * na + a] = o.trim().parse().map_err(|_| format!("state {}: bad output {:?}", q, o))?;
        }
    }
    Ok(Transducer { name: name.to_string(), nstates: n, letters: (0..na as i64).collect(), tau, sigma })
}

// ------------------------------------------------------------------ the morphism read off a DFAO

/// `M` seen as a (generally non-uniform) morphism: for each state, the list of
/// `(digit, target)` pairs the state may legally read, in digit order, plus the
/// output letter.  Under a numeration system the state is a pair (DFAO state,
/// validity state) so that "legal" is exactly "the validity automaton allows
/// this digit here"; for base `k` every digit is legal.
struct Morphism {
    digits: usize,
    nstates: usize,
    /// `edges[s]` = ascending `(digit, target)`.
    edges: Vec<Vec<(usize, usize)>>,
    out: Vec<u8>,
    /// Is the word that reaches this state a *representation* -- i.e. does the
    /// validity automaton accept it?  A live-but-rejecting validity state
    /// (Pell's "a 2 has been read, a 0 must follow") is a real node of the
    /// tree but occupies no position of the sequence, so it contributes no
    /// letter to the transducer's input.  For base `k` every state accepts.
    acc: Vec<bool>,
}

fn morphism_of(m: &Dfao) -> Morphism {
    let k = m.k;
    match numsys::active() {
        None => Morphism {
            digits: k,
            nstates: m.nstates,
            edges: (0..m.nstates).map(|q| (0..k).map(|d| (d, m.t(q, d))).collect()).collect(),
            out: m.out.clone(),
            acc: vec![true; m.nstates],
        },
        Some(ns) => {
            // Always the msd automaton: `Dfao` keeps its msd tables as the
            // definition of the sequence (`Dfao::at`), and the Dumont-Thomas
            // prefix decomposition this construction rests on is msd.
            let v = &ns.valid_msd;
            // states of the validity DFA from which some word is still accepted
            let mut live = v.accept.clone();
            loop {
                let mut ch = false;
                for s in 0..v.nstates {
                    if !live[s] {
                        for a in 0..v.alpha { if live[v.t(s, a)] { live[s] = true; ch = true; break; } }
                    }
                }
                if !ch { break; }
            }
            let mut index: HashMap<(usize, usize), usize> = HashMap::new();
            let mut order: Vec<(usize, usize)> = vec![(0, 0)];
            index.insert((0, 0), 0);
            let mut edges: Vec<Vec<(usize, usize)>> = Vec::new();
            let mut out: Vec<u8> = Vec::new();
            let mut acc: Vec<bool> = Vec::new();
            let mut i = 0;
            while i < order.len() {
                let (q, vs) = order[i];
                out.push(m.out[q]);
                acc.push(v.accept[vs]);
                let mut e = Vec::new();
                for d in 0..k {
                    let v2 = v.t(vs, d);
                    if !live[v2] { continue; }
                    let key = (m.t(q, d), v2);
                    let idx = *index.entry(key).or_insert_with(|| { order.push(key); order.len() - 1 });
                    e.push((d, idx));
                }
                edges.push(e);
                i += 1;
            }
            Morphism { digits: k, nstates: order.len(), edges, out, acc }
        }
    }
}

// ------------------------------------------------------------------ transduction

type Map = Vec<u32>; // a function P -> P

fn compose(f: &Map, g: &Map) -> Map { f.iter().map(|&x| g[x as usize]).collect() }

/// Apply `t` to the sequence `m`, returning the DFAO of the transduced
/// sequence.  Errors if an output letter of `m` is outside the transducer's
/// input alphabet, or an output letter of `t` does not fit a `u8`.
pub fn transduce(m: &Dfao, t: &Transducer, name: &str) -> Result<Dfao, String> {
    let h = morphism_of(m);
    let np = t.nstates;
    let na = t.na();

    // output letter of each morphism state, as a transducer input index.  Only
    // states that carry a position of the sequence have to be in the alphabet.
    let mut lidx = vec![None; h.nstates];
    for s in 0..h.nstates {
        let i = t.letters.iter().position(|&x| x == h.out[s] as i64);
        if i.is_none() && h.acc[s] {
            return Err(format!("sequence letter {} is outside the transducer's input alphabet {:?}",
                               h.out[s], t.letters));
        }
        lidx[s] = i;
    }

    // A[i][s] = the map P -> P induced by running the transducer over the
    // *positions* under `s` at depth `i` (`P_i(s)` in the module note).
    let ident: Map = (0..np as u32).collect();
    let mut a: Vec<Vec<Map>> = Vec::new();
    a.push((0..h.nstates).map(|s| match (h.acc[s], lidx[s]) {
        (true, Some(l)) => (0..np).map(|p| t.tau[p * na + l]).collect(),
        _ => ident.clone(),
    }).collect());
    let mut seen: HashMap<Vec<Map>, usize> = HashMap::new();
    seen.insert(a[0].clone(), 0);
    let (pre, per) = loop {
        let last = a.last().unwrap();
        let next: Vec<Map> = (0..h.nstates).map(|s| {
            let mut f = ident.clone();
            for &(_, tgt) in &h.edges[s] { f = compose(&f, &last[tgt]); }
            f
        }).collect();
        let i = a.len();
        if let Some(&j) = seen.get(&next) { break (j, i - j); }
        seen.insert(next.clone(), i);
        a.push(next);
        if a.len() > 4096 { return Err("iterate maps did not become periodic within 4096 steps".into()); }
    };
    let l = pre + per;
    let wrap = |i: usize| if i < l { i } else { pre + (i - pre) % per };

    // BFS over (morphism state, [f_0 .. f_{L-1}])
    #[derive(Hash, PartialEq, Eq, Clone)]
    struct Key(usize, Vec<Map>);
    let init = Key(0, vec![ident.clone(); l]);
    let mut index: HashMap<Key, usize> = HashMap::new();
    let mut order: Vec<Key> = vec![init.clone()];
    index.insert(init, 0);
    let dcount = h.digits;
    let mut trans: Vec<i64> = Vec::new(); // -1 = dead, filled in below
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < order.len() {
        let Key(s, f) = order[i].clone();
        // A state the validity automaton rejects holds no position of the
        // sequence, so its output is never read; 0 keeps it deterministic.
        let o = match lidx[s] { Some(l) => t.sigma[f[0][0] as usize * na + l], None => 0 };
        if !(0..=255).contains(&o) { return Err(format!("transducer output {} does not fit a byte", o)); }
        out.push(o as u8);
        let mut row = vec![-1i64; dcount];
        // g = phi_{O(h^i(prefix))} for the prefix of h(s) consumed so far
        let mut g: Vec<Map> = vec![ident.clone(); l];
        for &(d, tgt) in &h.edges[s] {
            let nf: Vec<Map> = (0..l).map(|j| compose(&f[wrap(j + 1)], &g[j])).collect();
            let key = Key(tgt, nf);
            let idx = match index.get(&key) {
                Some(&x) => x,
                None => { order.push(key.clone()); index.insert(key, order.len() - 1); order.len() - 1 }
            };
            row[d] = idx as i64;
            for j in 0..l { g[j] = compose(&g[j], &a[j][tgt]); }
        }
        trans.extend_from_slice(&row);
        i += 1;
        if order.len() > 1_000_000 { return Err("transduced automaton exceeded 1e6 states".into()); }
    }

    // materialise, with a dead sink for the illegal digits
    let n = order.len();
    let dead = n as State;
    let mut tt = vec![dead; (n + 1) * dcount];
    for s in 0..n {
        for d in 0..dcount {
            let x = trans[s * dcount + d];
            if x >= 0 { tt[s * dcount + d] = x as State; }
        }
    }
    let mut oo = out;
    oo.push(0);
    let (n2, tt2, oo2) = minimize(dcount, n + 1, &tt, &oo);
    Dfao::from_tables(dcount, n2, tt2, oo2, name)
}

/// Moore minimisation of a complete DFAO (state 0 stays the start state).
pub fn minimize(k: usize, n: usize, trans: &[State], out: &[u8]) -> (usize, Vec<State>, Vec<u8>) {
    // reachable
    let mut map = vec![usize::MAX; n];
    let mut order = vec![0usize];
    map[0] = 0;
    let mut i = 0;
    while i < order.len() {
        let s = order[i];
        for d in 0..k {
            let x = trans[s * k + d] as usize;
            if map[x] == usize::MAX { map[x] = order.len(); order.push(x); }
        }
        i += 1;
    }
    let m = order.len();
    let rt: Vec<State> = (0..m).flat_map(|q| (0..k).map(move |d| (q, d)))
        .map(|(q, d)| map[trans[order[q] * k + d] as usize] as State).collect();
    let ro: Vec<u8> = order.iter().map(|&q| out[q]).collect();
    let mut color: Vec<u32> = ro.iter().map(|&x| x as u32).collect();
    loop {
        let mut sig: HashMap<Vec<u32>, u32> = HashMap::new();
        let mut nc = vec![0u32; m];
        for q in 0..m {
            let mut key = Vec::with_capacity(k + 1);
            key.push(color[q]);
            for d in 0..k { key.push(color[rt[q * k + d] as usize]); }
            let next = sig.len() as u32;
            nc[q] = *sig.entry(key).or_insert(next);
        }
        let before = color.iter().collect::<std::collections::HashSet<_>>().len();
        color = nc;
        if sig.len() == before { break; }
    }
    let ncol = *color.iter().max().unwrap() as usize + 1;
    let mut relab = vec![usize::MAX; ncol];
    relab[color[0] as usize] = 0;
    let mut cnt = 1;
    for q in 0..m { let c = color[q] as usize; if relab[c] == usize::MAX { relab[c] = cnt; cnt += 1; } }
    let mut nt = vec![0u32; cnt * k];
    let mut no = vec![0u8; cnt];
    for q in 0..m {
        let c = relab[color[q] as usize];
        no[c] = ro[q];
        for d in 0..k { nt[c * k + d] = relab[color[rt[q * k + d] as usize] as usize] as State; }
    }
    (cnt, nt, no)
}
