//! Addable numeration systems: Zeckendorf (Fibonacci), Tribonacci, Pell, ...
//!
//! A numeration system here is exactly the four objects Walnut's `NumberSystem`
//! carries, except that *none* of them is hard-coded:
//!
//! * a digit alphabet `{0, .., D-1}` (must contain 0 and 1, and 0 must pad),
//! * a **validity** DFA over one track, accepting `0*` followed by the canonical
//!   representation of every `n >= 0` (msd-first, leading zeros allowed),
//! * an **addition** DFA over three tracks `(x,y,z)` accepting exactly the triples
//!   of valid representations with `x + y = z`,
//! * a **comparison** DFA over two tracks; if the system does not ship one we use
//!   msd lexicographic order, which agrees with numeric order on valid
//!   representations of equal padded length (this is the defining property of an
//!   *ordered* / radix numeration system and is machine-checked by
//!   `explore/gen_numsys.py`).
//!
//! Values.  We never assume a weight sequence.  The value of a valid word is its
//! **rank** in the radix (length, then lexicographic) ordering of the validity
//! language, computed from the counting table `cnt[q][l] = #{accepted words of
//! length l from state q}`.  For the classical systems this reproduces
//! `sum_i d_i U_i` with `U_l = cnt[q0][l]` (1,2,3,5,8,... for Zeckendorf), but it
//! is defined for any abstract numeration system, so `rep`/`value`/`succ` need no
//! per-system code at all.
//!
//! File format: Walnut's "Custom Bases" text format, so Walnut's own
//! `msd_fib_addition.txt` &c. can be dropped in unchanged (see docs/NUMERATION.md).

use crate::dfa::{Dfa, State, is_lsd};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

/// Longest representation length the counting table covers (a 96-digit
/// Zeckendorf word already exceeds 2^64, so this never binds in practice).
pub const MAXLEN: usize = 96;
/// Saturation ceiling for the counting table (keeps `u64` arithmetic total).
const CNT_CAP: u64 = u64::MAX / 4;

// ------------------------------------------------------------------ the system

/// One loaded numeration system: digit alphabet, validity/addition/comparison
/// automata in both digit orders, and the counting table that turns valid words
/// into integers and back.
pub struct NumSys {
    pub name: String,
    pub digits: usize,
    /// msd-first automata, exactly as loaded from file.
    pub valid_msd: Dfa,
    pub add_msd: Dfa,
    pub lt_msd: Dfa,
    /// lsd-first automata (language reversals of the above).
    pub valid_lsd: Dfa,
    pub add_lsd: Dfa,
    pub lt_lsd: Dfa,
    /// raw msd validity transition table (`nstates * digits`) and accept bits,
    /// used by the value machinery, which is always msd.
    vn: usize,
    vt: Vec<State>,
    vacc: Vec<bool>,
    /// `cnt[q * (MAXLEN+1) + l]` = number of accepted words of length `l` from `q`.
    cnt: Vec<u64>,
    /// Whether a comparison automaton was loaded (vs. derived lexicographically).
    pub lt_loaded: bool,
}

impl NumSys {
    #[inline] fn vt(&self, q: usize, d: usize) -> usize { self.vt[q * self.digits + d] as usize }
    #[inline] fn cnt(&self, q: usize, l: usize) -> u64 {
        if l > MAXLEN { CNT_CAP } else { self.cnt[q * (MAXLEN + 1) + l] }
    }

    /// The validity automaton for the active digit order.
    pub fn valid(&self) -> &Dfa { if is_lsd() { &self.valid_lsd } else { &self.valid_msd } }
    /// The addition automaton `(x,y,z) : x+y=z` for the active digit order.
    pub fn add(&self) -> &Dfa { if is_lsd() { &self.add_lsd } else { &self.add_msd } }
    /// The comparison automaton `(x,y) : x<y` for the active digit order.
    pub fn lt(&self) -> &Dfa { if is_lsd() { &self.lt_lsd } else { &self.lt_msd } }

    /// `d` conjoined with "every track holds a valid representation".
    pub fn restrict(&self, d: &Dfa) -> Dfa {
        if d.vars.is_empty() { return d.clone(); }
        let v = self.valid();
        let mut out = d.clone();
        for name in d.vars.clone() {
            let vv = Dfa { vars: vec![name], ..v.clone() };
            out = out.and(&vv);
        }
        out
    }

    /// A one-track automaton accepting exactly the valid representations of `x`.
    pub fn valid_for(&self, x: &str) -> Dfa {
        Dfa { vars: vec![x.to_string()], ..self.valid().clone() }
    }

    // ---------------------------------------------------------------- values

    /// Canonical msd-first digit word of `n` (no leading zeros; `[0]` for `n = 0`).
    pub fn rep(&self, n: u64) -> Vec<usize> {
        let mut len = 0usize;
        while len <= MAXLEN && self.cnt(0, len) <= n { len += 1; }
        if len == 0 { len = 1; }
        self.unrank(n, len)
    }

    /// The `n`-th (0-based) accepted word of length `len`, in lexicographic order.
    fn unrank(&self, mut n: u64, len: usize) -> Vec<usize> {
        let mut w = vec![0usize; len];
        let mut q = 0usize;
        for p in 0..len {
            let rem = len - 1 - p;
            let mut chosen = self.digits; // sentinel
            for e in 0..self.digits {
                let q2 = self.vt(q, e);
                let c = self.cnt(q2, rem);
                if n < c { chosen = e; q = q2; break; }
                n -= c;
            }
            if chosen == self.digits { return vec![0; len]; } // unreachable for n < cnt(0,len)
            w[p] = chosen;
        }
        w
    }

    /// Value of an msd-first digit word, or `None` if the word is not valid
    /// (or its value overflows `u64`).
    pub fn value(&self, w: &[usize]) -> Option<u64> {
        let len = w.len();
        let mut q = 0usize;
        let mut v: u64 = 0;
        for p in 0..len {
            let d = w[p];
            if d >= self.digits { return None; }
            let rem = len - 1 - p;
            for e in 0..d {
                v = v.checked_add(self.cnt(self.vt(q, e), rem))?;
            }
            q = self.vt(q, d);
        }
        if !self.vacc[q] { return None; }
        if v >= CNT_CAP { return None; }
        Some(v)
    }

    /// Number of digits in the canonical representation of `n`.
    pub fn replen(&self, n: u64) -> usize { self.rep(n).len() }

    /// In-place successor of a fixed-width valid word.  `vst[p]` is the validity
    /// state after `w[0..p]` (so `vst[0] = 0`); both are updated.  Returns the
    /// leftmost changed position, or `None` if `w` is the largest word of its
    /// width.
    pub fn succ(&self, w: &mut [usize], vst: &mut [u32]) -> Option<usize> {
        let width = w.len();
        for p in (0..width).rev() {
            let q = vst[p] as usize;
            for e in (w[p] + 1)..self.digits {
                let q2 = self.vt(q, e);
                if self.cnt(q2, width - 1 - p) == 0 { continue; }
                w[p] = e;
                vst[p + 1] = q2 as u32;
                let mut qq = q2;
                for r in (p + 1)..width {
                    let rem = width - 1 - r;
                    for e2 in 0..self.digits {
                        let q3 = self.vt(qq, e2);
                        if self.cnt(q3, rem) > 0 { w[r] = e2; qq = q3; vst[r + 1] = q3 as u32; break; }
                    }
                }
                return Some(p);
            }
        }
        None
    }

    /// Validity state reached after reading `w` from the start state.
    pub fn vstates(&self, w: &[usize]) -> Vec<u32> {
        let mut st = vec![0u32; w.len() + 1];
        for p in 0..w.len() { st[p + 1] = self.vt(st[p] as usize, w[p]) as u32; }
        st
    }

    /// Run the msd automaton `a` (whose tracks are `w.len()` digit words, all of the
    /// same length) and report acceptance.
    fn run_msd(&self, a: &Dfa, w: &[&[usize]]) -> bool {
        let len = w[0].len();
        let mut q = 0usize;
        for p in 0..len {
            let mut sym = 0usize;
            let mut mult = 1usize;
            for t in w { sym += t[p] * mult; mult *= self.digits; }
            q = a.trans[q * a.alpha + sym] as usize;
        }
        a.accept[q]
    }

    fn padded(&self, vals: &[u64]) -> Vec<Vec<usize>> {
        let reps: Vec<Vec<usize>> = vals.iter().map(|&v| self.rep(v)).collect();
        let len = reps.iter().map(|r| r.len()).max().unwrap();
        reps.into_iter().map(|r| {
            let mut v = vec![0usize; len - r.len()];
            v.extend(r);
            v
        }).collect()
    }

    /// Refuse to install a numeration system whose three automata do not agree with
    /// each other on small numbers.  This is what makes "drop Walnut's Custom Bases
    /// files in and go" safe: a system whose representations are not radix-ordered
    /// (a negative base, say) would silently give wrong answers everywhere, because
    /// values here are ranks and comparison defaults to lexicographic.
    pub fn self_check(&self) -> Result<(), String> {
        for n in 0..=200u64 {
            let r = self.rep(n);
            match self.value(&r) {
                Some(v) if v == n => {}
                other => return Err(format!(
                    "{}: value(rep({})) = {:?}, not {} -- the validity automaton and the \
rank ordering disagree", self.name, n, other, n)),
            }
        }
        for x in 0..=20u64 {
            for y in 0..=20u64 {
                let p = self.padded(&[x, y, x + y]);
                if !self.run_msd(&self.add_msd, &[&p[0], &p[1], &p[2]]) {
                    return Err(format!("{}: addition automaton rejects {} + {} = {}",
                                       self.name, x, y, x + y));
                }
                let z = x + y + 1;
                let q = self.padded(&[x, y, z]);
                if self.run_msd(&self.add_msd, &[&q[0], &q[1], &q[2]]) {
                    return Err(format!("{}: addition automaton accepts {} + {} = {}",
                                       self.name, x, y, z));
                }
                let c = self.padded(&[x, y]);
                if self.run_msd(&self.lt_msd, &[&c[0], &c[1]]) != (x < y) {
                    return Err(format!("{}: comparison automaton disagrees with {} < {}{}",
                                       self.name, x, y,
                                       if self.lt_loaded { "" } else { " (derived lexicographically)" }));
                }
            }
        }
        Ok(())
    }

    /// `U_l` = number of valid words of length `l` = the weight of digit position
    /// `l` in the classical systems.  Exposed for diagnostics/validation.
    pub fn weight(&self, l: usize) -> u64 { self.cnt(0, l) }
}

// ------------------------------------------------------------------ global state

static ACTIVE: OnceLock<RwLock<Option<Arc<NumSys>>>> = OnceLock::new();
fn cell() -> &'static RwLock<Option<Arc<NumSys>>> { ACTIVE.get_or_init(|| RwLock::new(None)) }

/// The numeration system currently in force, or `None` for built-in base `k`.
pub fn active() -> Option<Arc<NumSys>> { cell().read().unwrap().clone() }
/// Install (or clear, with `None`) the session's numeration system.
pub fn set_active(ns: Option<Arc<NumSys>>) { *cell().write().unwrap() = ns; }
/// Name of the active system, or `""`.
pub fn active_name() -> String { active().map(|n| n.name.clone()).unwrap_or_default() }

/// `d` conjoined with validity of every track, if a numeration system is active.
pub fn restrict(d: &Dfa) -> Dfa {
    match active() { None => d.clone(), Some(ns) => ns.restrict(d) }
}

// ------------------------------------------------------------------ file format

/// A parsed Walnut-format automaton: `ntracks` input tracks over the digit
/// alphabet `{0..digits-1}`, `nstates` states with integer outputs, and a partial
/// transition function (missing entries mean "dead").
pub struct Parsed {
    pub ntracks: usize,
    pub digits: usize,
    pub nstates: usize,
    pub out: Vec<i64>,
    /// `trans[(state, symbol)] = target`, symbol encoded as in `Dfa`
    /// (coordinate `i` contributes `d_i * digits^i`).
    pub trans: HashMap<(usize, usize), usize>,
    /// Number-system names named on the alphabet line (for a sanity check).
    pub ns_tokens: Vec<String>,
    /// Walnut's convention: the initial state is the FIRST one declared in the
    /// file, which need not be the one numbered 0 (`msd_trib_addition.txt` starts
    /// at state 78).  Reading it as 0 silently gives a different automaton.
    pub start: usize,
}

fn parse_alphabet(line: &str, expect_digits: Option<usize>) -> Result<(Vec<usize>, Vec<String>), String> {
    // tokens are either "{a, b, c}" or a bare number-system name such as msd_fib
    let mut sizes = Vec::new();
    let mut names = Vec::new();
    let b: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_whitespace() { i += 1; continue; }
        if b[i] == '{' {
            let close = b[i..].iter().position(|&c| c == '}').ok_or("unterminated { in alphabet")? + i;
            let inner: String = b[i + 1..close].iter().collect();
            let mut vals: Vec<i64> = Vec::new();
            for t in inner.split(',') {
                let t = t.trim();
                if t.is_empty() { continue; }
                vals.push(t.parse::<i64>().map_err(|_| format!("bad alphabet element {:?}", t))?);
            }
            vals.sort();
            if vals.is_empty() { return Err("empty alphabet".into()); }
            for (j, v) in vals.iter().enumerate() {
                if *v != j as i64 {
                    return Err(format!("alphabet must be {{0,..,D-1}}; got {:?}", vals));
                }
            }
            sizes.push(vals.len());
            i = close + 1;
        } else {
            let st = i;
            while i < b.len() && !b[i].is_whitespace() { i += 1; }
            let tok: String = b[st..i].iter().collect();
            let d = expect_digits.ok_or_else(|| format!(
                "alphabet token {:?} names a number system; load it with an explicit digit count", tok))?;
            names.push(tok);
            sizes.push(d);
        }
    }
    if sizes.is_empty() { return Err("empty alphabet line".into()); }
    Ok((sizes, names))
}

/// Parse Walnut's automaton text format.  `expect_digits` supplies the digit
/// count when the alphabet line names a number system (`msd_fib`) instead of an
/// explicit set (`{0,1}`).
pub fn parse_walnut(text: &str, expect_digits: Option<usize>) -> Result<Parsed, String> {
    let mut lines = text.lines().map(|l| {
        match l.find('#') { Some(p) => &l[..p], None => l }
    }).filter(|l| !l.trim().is_empty());
    let alpha_line = lines.next().ok_or("empty automaton file")?;
    let (sizes, ns_tokens) = parse_alphabet(alpha_line, expect_digits)?;
    let digits = sizes[0];
    if sizes.iter().any(|&s| s != digits) {
        return Err("all tracks must share one digit alphabet".into());
    }
    let ntracks = sizes.len();
    let mut out: HashMap<usize, i64> = HashMap::new();
    let mut trans: HashMap<(usize, usize), usize> = HashMap::new();
    let mut cur: Option<usize> = None;
    let mut start: Option<usize> = None;
    let mut maxstate = 0usize;
    for raw in lines {
        let line = raw.trim();
        if let Some(arrow) = line.find("->") {
            let (lhs, rhs) = (line[..arrow].trim(), line[arrow + 2..].trim());
            let q = cur.ok_or("transition before any state declaration")?;
            let dests: Vec<usize> = rhs.split_whitespace()
                .map(|t| t.parse::<usize>().map_err(|_| format!("bad destination {:?}", t)))
                .collect::<Result<_, _>>()?;
            if dests.len() != 1 { return Err(format!("nondeterministic transition ({} targets) not supported", dests.len())); }
            let t = dests[0];
            maxstate = maxstate.max(t);
            // input side: one token per track, each a digit or '*'
            let toks: Vec<&str> = lhs.split_whitespace().collect();
            if toks.len() != ntracks {
                return Err(format!("transition has {} inputs, expected {}", toks.len(), ntracks));
            }
            let mut choices: Vec<Vec<usize>> = Vec::with_capacity(ntracks);
            for tk in toks {
                if tk == "*" { choices.push((0..digits).collect()); }
                else {
                    let d: i64 = tk.parse().map_err(|_| format!("bad input digit {:?}", tk))?;
                    if d < 0 || d as usize >= digits { return Err(format!("digit {} outside alphabet", d)); }
                    choices.push(vec![d as usize]);
                }
            }
            // cartesian product over tracks
            let mut syms = vec![0usize];
            let mut mult = 1usize;
            for c in &choices {
                let mut next = Vec::with_capacity(syms.len() * c.len());
                for s in &syms { for d in c { next.push(s + d * mult); } }
                mult *= digits;
                syms = next;
            }
            for s in syms { trans.insert((q, s), t); }
        } else {
            let p: Vec<&str> = line.split_whitespace().collect();
            if p.len() != 2 { return Err(format!("bad state declaration {:?}", line)); }
            let q: usize = p[0].parse().map_err(|_| format!("bad state name {:?}", p[0]))?;
            let o: i64 = p[1].parse().map_err(|_| format!("bad output {:?}", p[1]))?;
            out.insert(q, o);
            maxstate = maxstate.max(q);
            if start.is_none() { start = Some(q); }
            cur = Some(q);
        }
    }
    let nstates = maxstate + 1;
    let outs: Vec<i64> = (0..nstates).map(|q| *out.get(&q).unwrap_or(&0)).collect();
    Ok(Parsed { ntracks, digits, nstates, out: outs, trans, ns_tokens, start: start.unwrap_or(0) })
}

impl Parsed {
    /// Swap the declared start state with 0, so that state 0 is the start state
    /// (the invariant every `Dfa`/`Dfao` in this engine keeps).
    #[inline] fn relab(&self, q: usize) -> usize {
        if q == self.start { 0 } else if q == 0 { self.start } else { q }
    }

    /// Total DFA over `vars` (one per track), with a dead sink for the missing
    /// transitions.  Accepting = nonzero output.
    pub fn to_dfa(&self, vars: &[String]) -> Result<Dfa, String> {
        if vars.len() != self.ntracks {
            return Err(format!("automaton has {} tracks, {} names given", self.ntracks, vars.len()));
        }
        let alpha = self.digits.pow(self.ntracks as u32);
        let n = self.nstates + 1;
        let dead = self.nstates as State;
        let mut trans = vec![dead; n * alpha];
        for q in 0..self.nstates {
            for s in 0..alpha {
                if let Some(&t) = self.trans.get(&(q, s)) {
                    trans[self.relab(q) * alpha + s] = self.relab(t) as State;
                }
            }
        }
        let mut accept: Vec<bool> = (0..n).map(|q| q < self.nstates && self.out[self.relab(q)] != 0).collect();
        accept[self.nstates] = false;
        let d = Dfa::new(self.digits, vars.to_vec(), n, trans, accept);
        Ok(d.minimize())
    }

    /// `(nstates, transitions, outputs)` for a DFAO, with a dead sink appended.
    pub fn to_dfao_tables(&self) -> Result<(usize, Vec<State>, Vec<u8>), String> {
        if self.ntracks != 1 { return Err("a DFAO must have exactly one input track".into()); }
        let n = self.nstates + 1;
        let dead = self.nstates as State;
        let mut trans = vec![dead; n * self.digits];
        for q in 0..self.nstates {
            for d in 0..self.digits {
                if let Some(&t) = self.trans.get(&(q, d)) {
                    trans[self.relab(q) * self.digits + d] = self.relab(t) as State;
                }
            }
        }
        let mut out = vec![0u8; n];
        for q in 0..self.nstates {
            let o = self.out[self.relab(q)];
            if o < 0 || o > 255 { return Err(format!("output {} out of range", o)); }
            out[q] = o as u8;
        }
        Ok((n, trans, out))
    }
}

// ------------------------------------------------------------------ construction

/// msd lexicographic `x < y` over `D` digits (equal padded lengths): the first
/// differing digit decides and the verdict is absorbing.  Correct as *numeric*
/// comparison exactly on valid representations of an ordered numeration system.
fn lex_less_than(digits: usize) -> Dfa {
    let alpha = digits * digits;
    let mut trans = vec![0u32; 3 * alpha];
    for a in 0..digits {
        for b in 0..digits {
            let sym = a + b * digits;
            let v: u32 = if a < b { 1 } else if a > b { 2 } else { 0 };
            trans[sym] = v;
            trans[alpha + sym] = 1;
            trans[2 * alpha + sym] = 2;
        }
    }
    Dfa::new(digits, vec!["\u{1}a".into(), "\u{1}b".into()], 3, trans, vec![false, true, false]).minimize()
}

/// Build a numeration system from the parsed validity / addition / (optional)
/// comparison automata.
pub fn build(name: &str, valid: &Parsed, add: &Parsed, lt: Option<&Parsed>) -> Result<NumSys, String> {
    if valid.ntracks != 1 { return Err("validity automaton must have 1 track".into()); }
    if add.ntracks != 3 { return Err("addition automaton must have 3 tracks".into()); }
    let digits = valid.digits;
    if add.digits != digits { return Err("addition and validity disagree on the digit alphabet".into()); }
    if digits < 2 { return Err("a numeration system needs at least the digits {0,1}".into()); }

    let valid_msd = valid.to_dfa(&["\u{1}a".to_string()])?;
    let add_msd = add.to_dfa(&["\u{1}a".into(), "\u{1}b".into(), "\u{1}c".into()])?;
    let lt_msd = match lt {
        Some(p) => {
            if p.ntracks != 2 { return Err("comparison automaton must have 2 tracks".into()); }
            p.to_dfa(&["\u{1}a".into(), "\u{1}b".into()])?
        }
        None => lex_less_than(digits),
    };

    // The empty word is our representation of 0.  Walnut files sometimes exclude it
    // (msd_tib.txt's initial state is non-accepting and steps to the real start on a
    // leading zero), so add it and re-minimise; the load-time self-check below is what
    // decides whether the resulting system is coherent.
    let valid_msd = if valid_msd.accept[0] {
        valid_msd
    } else {
        let mut acc = valid_msd.accept.clone();
        acc[0] = true;
        Dfa { accept: acc, ..valid_msd }.minimize()
    };
    // raw msd validity tables (post-minimisation; state 0 is the start state)
    let vn = valid_msd.nstates;
    let vt: Vec<State> = valid_msd.trans.clone();
    let vacc = valid_msd.accept.clone();
    if vt[0] != 0 { return Err("validity automaton must allow leading zeros (delta(q0,0) = q0)".into()); }

    // counting table
    let mut cnt = vec![0u64; vn * (MAXLEN + 1)];
    for q in 0..vn { cnt[q * (MAXLEN + 1)] = if vacc[q] { 1 } else { 0 }; }
    for l in 1..=MAXLEN {
        for q in 0..vn {
            let mut s: u64 = 0;
            for d in 0..digits {
                s = s.saturating_add(cnt[vt[q * digits + d] as usize * (MAXLEN + 1) + l - 1]);
            }
            cnt[q * (MAXLEN + 1) + l] = s.min(CNT_CAP);
        }
    }

    let valid_lsd = valid_msd.reverse_determinize();
    let add_lsd = add_msd.reverse_determinize();
    let lt_lsd = lt_msd.reverse_determinize();

    let ns = NumSys {
        name: name.to_string(), digits,
        valid_msd, add_msd, lt_msd, valid_lsd, add_lsd, lt_lsd,
        vn, vt, vacc, cnt, lt_loaded: lt.is_some(),
    };
    ns.self_check()?;
    Ok(ns)
}

/// Directories searched for numeration-system files, in order.
pub fn search_dirs() -> Vec<std::path::PathBuf> {
    let mut v: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(d) = std::env::var("AM_NUMSYS_DIR") { v.push(d.into()); }
    if let Ok(exe) = std::env::current_exe() {
        // engine/target/release/peanut -> engine/numeration
        if let Some(p) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            v.push(p.join("numeration"));
        }
    }
    v.push("engine/numeration".into());
    v.push("numeration".into());
    if let Ok(d) = std::env::var("AM_WALNUT_BASES") { v.push(d.into()); }
    v
}

fn find(name: &str, suffix: &str) -> Option<std::path::PathBuf> {
    for d in search_dirs() {
        for stem in [format!("{}{}", name, suffix), format!("msd_{}{}", name, suffix)] {
            let p = d.join(&stem);
            if p.is_file() { return Some(p); }
        }
    }
    None
}

/// Load the numeration system called `name` from the search path.
pub fn load(name: &str) -> Result<NumSys, String> {
    let vp = find(name, ".txt").ok_or_else(|| format!(
        "no validity automaton for {:?} (looked for {0}.txt / msd_{0}.txt in {:?})",
        name, search_dirs()))?;
    let ap = find(name, "_addition.txt").ok_or_else(|| format!(
        "no addition automaton for {:?} ({0}_addition.txt / msd_{0}_addition.txt)", name))?;
    let vtext = std::fs::read_to_string(&vp).map_err(|e| format!("{}: {}", vp.display(), e))?;
    let valid = parse_walnut(&vtext, None).map_err(|e| format!("{}: {}", vp.display(), e))?;
    let atext = std::fs::read_to_string(&ap).map_err(|e| format!("{}: {}", ap.display(), e))?;
    let add = parse_walnut(&atext, Some(valid.digits)).map_err(|e| format!("{}: {}", ap.display(), e))?;
    let lt = match find(name, "_less_than.txt") {
        Some(lp) => {
            let t = std::fs::read_to_string(&lp).map_err(|e| format!("{}: {}", lp.display(), e))?;
            Some(parse_walnut(&t, Some(valid.digits)).map_err(|e| format!("{}: {}", lp.display(), e))?)
        }
        None => None,
    };
    build(name, &valid, &add, lt.as_ref())
}

// ------------------------------------------------------------------ words <-> values

/// Number of base-`k` digits of `v` (at least 1).
fn ndigits_k(mut v: u64, k: u64) -> usize { let mut n = 1; while v >= k { v /= k; n += 1; } n }

/// Word length needed to carry every value in `vals` in the active system.
pub fn width_for(k: usize, vals: &[u64]) -> usize {
    match active() {
        None => vals.iter().map(|&v| ndigits_k(v, k as u64)).max().unwrap_or(1).max(1),
        Some(ns) => vals.iter().map(|&v| ns.replen(v)).max().unwrap_or(1).max(1),
    }
}

/// Encode a tuple of values as a word over the product alphabet, in the ACTIVE
/// digit order, using the shortest common length.  Coordinate `c` of a symbol
/// carries the digit of `vals[c]`, matching [`crate::dfa::digit`] and the sorted
/// variable order.
pub fn encode_word(k: usize, vals: &[u64]) -> Vec<usize> {
    let len = width_for(k, vals);
    let ns = active();
    // msd-first digit strings, one per track, left-padded to `len`
    let tracks: Vec<Vec<usize>> = vals.iter().map(|&v| {
        let mut d = match &ns {
            None => { let mut t = Vec::new(); let mut m = v;
                      while m > 0 { t.push((m % k as u64) as usize); m /= k as u64; }
                      if t.is_empty() { t.push(0); } t.reverse(); t }
            Some(n) => n.rep(v),
        };
        while d.len() < len { d.insert(0, 0); }
        d
    }).collect();
    let mut w = vec![0usize; len];
    for pos in 0..len {
        let idx = if is_lsd() { len - 1 - pos } else { pos };
        let mut sym = 0usize;
        let mut mult = 1usize;
        for t in &tracks { sym += t[idx] * mult; mult *= k; }
        w[pos] = sym;
    }
    w
}

/// Inverse of [`encode_word`]: the `n` track values of a word, or `None` if a
/// track is not a valid representation or the value overflows `u64`.
pub fn decode_word(k: usize, n: usize, w: &[usize]) -> Option<Vec<u64>> {
    let len = w.len();
    match active() {
        None => {
            let mut v = vec![0u64; n];
            if is_lsd() {
                let mut place = 1u64;
                for (p, &s) in w.iter().enumerate() {
                    for c in 0..n {
                        let d = crate::dfa::digit(s, c, k) as u64;
                        if d != 0 { v[c] = v[c].checked_add(d.checked_mul(place)?)?; }
                    }
                    if p + 1 < len { place = place.checked_mul(k as u64)?; }
                }
            } else {
                for &s in w.iter() {
                    for c in 0..n {
                        v[c] = v[c].checked_mul(k as u64)?.checked_add(crate::dfa::digit(s, c, k) as u64)?;
                    }
                }
            }
            Some(v)
        }
        Some(ns) => {
            let mut out = Vec::with_capacity(n);
            for c in 0..n {
                let mut d: Vec<usize> = (0..len).map(|p| crate::dfa::digit(w[p], c, k)).collect();
                if is_lsd() { d.reverse(); }
                out.push(ns.value(&d)?);
            }
            Some(out)
        }
    }
}
