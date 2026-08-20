//! Walnut-compatibility mode: run Walnut command scripts unchanged in Peanut.
//!
//! Walnut (Mousavi/Shallit, v8-dev in `walnut7/`) is the reference decision
//! procedure for automatic sequences.  Its scripts are line-oriented commands
//! whose payload is a first-order formula in a quoted string:
//!
//! ```text
//! reg pows msd_fib "0*10010*":
//! def FactorEq "?msd_fib Ak (k < n) => (F[i + k] = F[j + k])":
//! eval test "?msd_fib Ai (F[i] = @0) => Ej $FactorEq(i, j, n)":
//! ```
//!
//! This module accepts that language and compiles it onto Peanut's `Dfa`
//! (`crate::dfa`), `base` primitives and `numsys` numeration systems.  It keeps
//! its own session state -- Walnut has *many* named word automata live at once,
//! where `crate::main` has a single `cur` sequence -- so nothing here disturbs
//! the native command loop.  `docs/WALNUT-COMPAT.md` is the user-facing map of
//! what is and is not supported and why.
//!
//! Layout:
//!   * `Ns`              a number-system token (`msd_fib`) and its resolution
//!   * `Word`            a Walnut "Word Automaton" (DFAO, possibly multi-track)
//!   * `Aut`             an ordinary Walnut automaton (a saved predicate)
//!   * `Morph`           a morphism from the Morphism Library
//!   * `lex`/`Parser`    Walnut's token grammar and operator priorities
//!   * `Cx`              formula -> Dfa compiler
//!   * `regex`           regular expression -> Dfa over the digit alphabet
//!   * `Compat::exec`    command dispatch

use crate::base;
use crate::dfa::{self, Dfa, Nfa, State};
use crate::logic::{Lin, Rel};
use crate::numsys::{self, NumSys};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use crate::clock::Instant;

// ------------------------------------------------------------------ number systems

/// A Walnut number-system token: `msd_2`, `lsd_fib`, ... `base` is the part
/// after the order prefix; it is either a decimal integer (built-in base k) or
/// the name of a custom base loaded from `Custom Bases/`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ns {
    pub msd: bool,
    pub base: String,
}

impl Ns {
    pub fn parse(tok: &str) -> Ns {
        let t = tok.trim();
        for (p, msd) in [("msd_", true), ("lsd_", false), ("msd", true), ("lsd", false)] {
            if let Some(r) = t.strip_prefix(p) {
                if r.is_empty() { return Ns { msd, base: "2".into() }; }
                return Ns { msd, base: r.to_string() };
            }
        }
        Ns { msd: true, base: t.to_string() }
    }
    pub fn text(&self) -> String { format!("{}_{}", if self.msd { "msd" } else { "lsd" }, self.base) }
    pub fn is_base_k(&self) -> Option<usize> { self.base.parse::<usize>().ok() }
}

impl Default for Ns {
    fn default() -> Ns { Ns { msd: true, base: "2".into() } }
}

// ------------------------------------------------------------------ objects

/// A Walnut Word Automaton: a DFAO with `ntracks` inputs over one digit
/// alphabet.  State `nstates-1` is always a dead sink (`out` there is unused).
#[derive(Clone)]
pub struct Word {
    pub ns: Vec<Ns>,
    pub digits: usize,
    pub ntracks: usize,
    pub nstates: usize,
    pub alpha: usize,
    pub trans: Vec<State>,
    pub out: Vec<i64>,
    /// index of the dead sink appended for missing transitions, if any
    pub dead: Option<usize>,
    /// tracks whose alphabet was a raw `{a,b,c}` set rather than a number system
    pub set_tracks: bool,
}

impl Word {
    pub fn out_alphabet(&self) -> Vec<i64> {
        let mut v: Vec<i64> = (0..self.nstates)
            .filter(|q| Some(*q) != self.dead).map(|q| self.out[q]).collect();
        v.sort();
        v.dedup();
        v
    }
    fn t(&self, q: usize, s: usize) -> usize { self.trans[q * self.alpha + s] as usize }
    /// State count excluding the dead sink, which is what Walnut reports.
    pub fn live(&self) -> usize { self.nstates - self.dead.map_or(0, |_| 1) }
}

/// An ordinary Walnut automaton (what `def`, `reg`, `fixleadzero`, ... save into
/// `Automata Library/`).  Tracks are positional: `$name(a,b)` binds argument `i`
/// to track `i`.  For a `def` the track order is the *sorted* free-variable
/// order, matching Walnut's `sortLabel`.
#[derive(Clone)]
pub struct Aut {
    pub ns: Ns,
    pub ntracks: usize,
    /// vars are the internal positional names `#t00`, `#t01`, ...
    pub dfa: Dfa,
    pub labels: Vec<String>,
}

#[derive(Clone)]
pub struct Morph {
    pub map: BTreeMap<i64, Vec<i64>>,
}

impl Morph {
    fn uniform_len(&self) -> Option<usize> {
        let mut l = None;
        for v in self.map.values() {
            match l { None => l = Some(v.len()), Some(x) if x == v.len() => {}, _ => return None }
        }
        l
    }
}

/// Positional track name `#tNN`; lexicographic order == positional order.
fn tname(i: usize) -> String { format!("#t{:02}", i) }

// ------------------------------------------------------------------ session

pub struct Compat {
    pub on: bool,
    /// accumulated text of a command that has not hit its terminator yet
    buf: String,
    pub root: PathBuf,
    words: HashMap<String, Arc<Word>>,
    auts: HashMap<String, Arc<Aut>>,
    morphs: HashMap<String, Morph>,
    nscache: HashMap<String, Option<Arc<NumSys>>>,
    /// the number system currently installed in the global `numsys`/`dfa` state
    installed: Option<Ns>,
}

/// Heads that mark a line as Walnut source for the `?msd_`/`?lsd_` auto-detect.
pub const IS_WALNUT_CMD: &[&str] = &[
    "eval", "def", "reg", "morphism", "promote", "image", "combine", "reverse",
    "minimize", "alphabet", "fixleadzero", "fixtrailzero", "test", "inf", "load",
    "macro", "transduce", "convert", "split", "rsplit", "join", "ost", "concat",
    "star", "leftquo", "rightquo", "intersect", "union", "describe",
];

/// Commands Walnut has that this layer does not implement.  Listed explicitly so
/// that a typo is still reported as an unknown command rather than silently
/// skipped as "unsupported".
const KNOWN_UNSUPPORTED: &[&str] = &[
    "transduce", "convert", "split", "rsplit", "join", "ost", "macro", "concat",
    "star", "leftquo", "rightquo", "intersect", "union", "describe", "export",
    "cls", "clear", "help", "draw", "fixtrailzero",
];

impl Compat {
    pub fn new() -> Compat {
        Compat {
            on: false, buf: String::new(), root: find_root(),
            words: HashMap::new(), auts: HashMap::new(), morphs: HashMap::new(),
            nscache: HashMap::new(), installed: None,
        }
    }

    /// Feed one physical line.  Walnut commands may span several lines and end
    /// with `;`, `:` or `::`, so this buffers until a terminator is seen and
    /// returns `None` while the command is still incomplete.
    pub fn feed(&mut self, line: &str) -> Option<String> {
        // strip Walnut's line comments: `//` to end of line, and `#` at line start
        let mut l = line;
        if let Some(p) = l.find("//") { l = &l[..p]; }
        let lt = l.trim_start();
        if lt.starts_with('#') && !self.buf.contains('"') { l = ""; }
        if self.buf.is_empty() && l.trim().is_empty() { return None; }
        if !self.buf.is_empty() { self.buf.push(' '); }
        self.buf.push_str(l.trim_end());
        // A terminator only counts outside the quoted predicate.
        let mut q = false;
        let b: Vec<char> = self.buf.chars().collect();
        let mut end = None;
        for i in 0..b.len() {
            if b[i] == '"' { q = !q; continue; }
            if !q && (b[i] == ';' || b[i] == ':') { end = Some(i); break; }
        }
        let Some(e) = end else { return None };
        let cmd: String = b[..e].iter().collect();
        let rest: String = b[e..].iter().collect();
        // trailing prose after the terminator is a Walnut comment
        let term = if rest.starts_with("::") { "::" } else if rest.starts_with(':') { ":" } else { ";" };
        self.buf.clear();
        Some(self.run(cmd.trim(), term))
    }

    /// Anything still buffered when the session ends is an unterminated command.
    pub fn flush(&mut self) -> Option<String> {
        if self.buf.trim().is_empty() { self.buf.clear(); return None; }
        let c = std::mem::take(&mut self.buf);
        Some(format!("WERR unterminated command {:?}", c.trim()))
    }

    // -------------------------------------------------------------- dispatch

    fn run(&mut self, cmd: &str, _term: &str) -> String {
        let t0 = Instant::now();
        let (head, rest) = split1(cmd);
        let head = head.to_string();
        let r = match head.as_str() {
            "" => return String::new(),
            "eval" | "def" => self.cmd_eval(&head, rest),
            "reg" => self.cmd_reg(rest),
            "morphism" => self.cmd_morphism(rest),
            "promote" => self.cmd_promote(rest),
            "image" => self.cmd_image(rest),
            "combine" => self.cmd_combine(rest),
            "reverse" => self.cmd_reverse(rest),
            "minimize" => self.cmd_minimize(rest),
            "alphabet" => self.cmd_alphabet(rest),
            "fixleadzero" => self.cmd_fixzero(rest, true),
            "test" => self.cmd_test(rest),
            "inf" => self.cmd_inf(rest),
            "load" => self.cmd_load(rest),
            "quit" | "exit" => return "WQUIT".to_string(),
            _ if KNOWN_UNSUPPORTED.contains(&head.as_str()) =>
                return format!("ERR unsupported: {}", head),
            _ => return format!("ERR unsupported: {}", head),
        };
        match r {
            Ok(s) if s.is_empty() => String::new(),
            Ok(s) => format!("{} ms={}", s, t0.elapsed().as_millis()),
            Err(e) => format!("WERR {}: {}", head, e),
        }
    }

    // -------------------------------------------------------------- eval / def

    fn cmd_eval(&mut self, head: &str, rest: &str) -> Result<String, String> {
        let (name, pred) = parse_name_and_string(rest)?;
        let (ns, body) = strip_ns(&pred)?;
        self.install(&ns)?;
        let k = self.k(&ns)?;
        let node = Parser::parse(&body)?;
        let mut cx = Cx { c: self, k, ns: ns.clone(), fresh: 0 };
        let d = cx.form(&node)?;
        let free: Vec<String> = d.vars.iter().filter(|v| !v.starts_with('#')).cloned().collect();
        if free.len() != d.vars.len() {
            return Err(format!("internal variables leaked: {:?}", d.vars));
        }
        let verdict = if free.is_empty() {
            if d.accepts_epsilon() { "TRUE" } else { "FALSE" }
        } else { "OPEN" };
        let ntracks = free.len();
        let mut ren = HashMap::new();
        for (i, v) in free.iter().enumerate() { ren.insert(v.clone(), tname(i)); }
        let stored = d.rename(&|v| ren.get(v).cloned().unwrap_or_else(|| v.to_string()));
        if !name.is_empty() {
            self.auts.insert(name.clone(), Arc::new(Aut {
                ns: ns.clone(), ntracks, dfa: stored, labels: free.clone(),
            }));
        }
        Ok(format!("WOK {} {} states={} vars=[{}] verdict={}",
                   head, if name.is_empty() { "-" } else { &name }, d.nstates, free.join(","), verdict))
    }

    // -------------------------------------------------------------- reg

    fn cmd_reg(&mut self, rest: &str) -> Result<String, String> {
        let q = rest.find('"').ok_or("expected a quoted regular expression")?;
        let qe = rest.rfind('"').ok_or("unterminated regular expression")?;
        if qe <= q { return Err("unterminated regular expression".into()); }
        let re = &rest[q + 1..qe];
        let head: Vec<&str> = rest[..q].split_whitespace().collect();
        if head.len() < 2 { return Err("usage: reg <name> <ns> ... <ns> \"<regex>\"".into()); }
        let name = head[0].to_string();
        let mut nss = Vec::new();
        let mut sets: Vec<Option<Vec<i64>>> = Vec::new();
        // alphabets are either number-system tokens or `{a,b,c}` sets
        let alpha_src = rest[..q][head[0].len()..].trim().to_string();
        for tok in split_alphabets(&alpha_src) {
            if tok.starts_with('{') {
                let inner = tok.trim_start_matches('{').trim_end_matches('}');
                let mut v: Vec<i64> = Vec::new();
                for p in inner.split(',') {
                    let p = p.trim();
                    if p.is_empty() { continue; }
                    v.push(p.parse().map_err(|_| format!("bad alphabet element {:?}", p))?);
                }
                v.sort(); v.dedup();
                sets.push(Some(v));
                nss.push(Ns::default());
            } else {
                sets.push(None);
                nss.push(Ns::parse(&tok));
            }
        }
        if nss.is_empty() { return Err("no alphabet given".into()); }
        // every track must share one digit alphabet
        let ns0 = nss[0].clone();
        if sets.iter().any(|s| s.is_some()) {
            return Err("set alphabets ({a,b,c}) in reg are not supported".into());
        }
        if nss.iter().any(|n| *n != ns0) {
            return Err("mixed number systems in one automaton are not supported".into());
        }
        self.install(&ns0)?;
        let k = self.k(&ns0)?;
        let ntracks = nss.len();
        let d = regex::compile(re, k, ntracks)?;
        let d = numsys::restrict(&d).minimize();
        self.auts.insert(name.clone(), Arc::new(Aut {
            ns: ns0, ntracks, dfa: d.clone(), labels: (0..ntracks).map(tname).collect(),
        }));
        Ok(format!("WOK reg {} states={}", name, d.nstates))
    }

    // -------------------------------------------------------------- morphisms

    fn cmd_morphism(&mut self, rest: &str) -> Result<String, String> {
        let (name, body) = parse_name_and_string(rest)?;
        if name.is_empty() { return Err("morphism needs a name".into()); }
        let map = parse_morphism(&body)?;
        let n = map.len();
        self.morphs.insert(name.clone(), Morph { map });
        Ok(format!("WOK morphism {} letters={}", name, n))
    }

    fn cmd_promote(&mut self, rest: &str) -> Result<String, String> {
        let p: Vec<&str> = rest.split_whitespace().collect();
        if p.len() != 2 { return Err("usage: promote <name> <morphism>".into()); }
        let m = self.morph(p[1])?;
        let l = m.uniform_len().ok_or("morphism is not uniform")?;
        if l < 2 { return Err("promote needs a k-uniform morphism with k >= 2".into()); }
        let letters: Vec<i64> = m.map.keys().cloned().collect();
        if letters.iter().enumerate().any(|(i, &a)| a != i as i64) {
            return Err("promote needs the domain to be {0,..,n-1}".into());
        }
        let n = letters.len();
        let nstates = n + 1;
        let dead = n as State;
        let mut trans = vec![dead; nstates * l];
        for (a, img) in &m.map {
            for (d, &b) in img.iter().enumerate() {
                if b < 0 || b as usize >= n { return Err(format!("image letter {} outside the domain", b)); }
                trans[*a as usize * l + d] = b as State;
            }
        }
        let mut out: Vec<i64> = (0..n as i64).collect();
        out.push(0);
        let w = Word { ns: vec![Ns { msd: true, base: l.to_string() }], digits: l, ntracks: 1,
                       nstates, alpha: l, trans, out, dead: Some(n), set_tracks: false };
        self.words.insert(p[0].to_string(), Arc::new(w));
        Ok(format!("WOK promote {} states={}", p[0], n))
    }

    /// `image <new> <morphism> <DFAO>` -- Walnut builds this from predicates
    /// (`Morphism.makeInterPredicate`); so do we, so the two agree by construction.
    fn cmd_image(&mut self, rest: &str) -> Result<String, String> {
        let p: Vec<&str> = rest.split_whitespace().collect();
        if p.len() != 3 { return Err("usage: image <new> <morphism> <DFAO>".into()); }
        let m = self.morph(p[1])?.clone();
        let l = m.uniform_len().ok_or("morphism is not uniform")?;
        if l == 0 { return Err("morphism must have positive uniform length".into()); }
        let w = self.word(p[2])?;
        if w.ntracks != 1 { return Err("image requires a unary word automaton".into()); }
        let ns = w.ns[0].clone();
        let mut range: Vec<i64> = m.map.values().flat_map(|v| v.iter().cloned()).collect();
        range.sort(); range.dedup();
        let mut parts: Vec<(Dfa, i64)> = Vec::new();
        let mut last = 0usize;
        for &v in &range {
            let mut s = format!("?{} E q, r (n={}*q+r & r>=0 & r<{}", ns.text(), l, l);
            for (a, img) in &m.map {
                let js: Vec<usize> = img.iter().enumerate().filter(|&(_, &x)| x == v).map(|(j, _)| j).collect();
                if js.is_empty() {
                    s.push_str(&format!(" & ({}[q]!= @{})", p[2], a));
                } else {
                    let alts: Vec<String> = js.iter().map(|j| format!("r={}", j)).collect();
                    s.push_str(&format!(" & ({}[q]= @{} => ({}))", p[2], a, alts.join("|")));
                }
            }
            s.push(')');
            let (nss, body) = strip_ns(&s)?;
            self.install(&nss)?;
            let k = self.k(&nss)?;
            let node = Parser::parse(&body)?;
            let mut cx = Cx { c: self, k, ns: nss.clone(), fresh: 0 };
            let d = cx.form(&node)?;
            if d.vars.len() != 1 {
                return Err(format!("image: intermediate predicate has vars {:?}", d.vars));
            }
            last = d.nstates;
            parts.push((d, v));
        }
        let w2 = combine_dfas(&parts, &[ns])?;
        let dfao = w2.live();
        self.words.insert(p[0].to_string(), Arc::new(w2));
        // Walnut logs the LAST intermediate predicate's size here, not the DFAO's
        Ok(format!("WOK image {} states={} dfao={}", p[0], last, dfao))
    }

    fn cmd_combine(&mut self, rest: &str) -> Result<String, String> {
        let p: Vec<&str> = rest.split_whitespace().collect();
        if p.len() < 2 { return Err("usage: combine <new> <aut>[=v] ...".into()); }
        let mut parts: Vec<(Dfa, i64)> = Vec::new();
        let mut ns = Ns::default();
        for (i, tok) in p[1..].iter().enumerate() {
            let (nm, v) = match tok.split_once('=') {
                Some((a, b)) => (a, b.parse::<i64>().map_err(|_| format!("bad value in {:?}", tok))?),
                None => (*tok, i as i64 + 1),
            };
            let a = self.aut(nm)?;
            if i == 0 { ns = a.ns.clone(); }
            parts.push((a.dfa.clone(), v));
        }
        self.install(&ns)?;
        let ntracks = parts[0].0.vars.len();
        let w = combine_dfas(&parts, &vec![ns; ntracks.max(1)])?;
        let st = w.live();
        self.words.insert(p[0].to_string(), Arc::new(w));
        Ok(format!("WOK combine {} states={}", p[0], st))
    }

    // -------------------------------------------------------------- automaton surgery

    fn cmd_reverse(&mut self, rest: &str) -> Result<String, String> {
        let p: Vec<&str> = rest.split_whitespace().collect();
        if p.len() != 2 { return Err("usage: reverse <new> <old>".into()); }
        if let Some(old) = p[1].strip_prefix('$') {
            let a = self.aut(old)?;
            self.install(&a.ns)?;
            let d = a.dfa.reverse_determinize();
            let ns = Ns { msd: !a.ns.msd, base: a.ns.base.clone() };
            let st = d.nstates;
            self.auts.insert(p[0].trim_start_matches('$').to_string(),
                             Arc::new(Aut { ns, ntracks: a.ntracks, dfa: d, labels: a.labels.clone() }));
            return Ok(format!("WOK reverse {} states={}", p[0], st));
        }
        let w = self.word(p[1])?;
        self.install(&w.ns[0].clone())?;
        let alph = w.out_alphabet();
        let mut parts: Vec<(Dfa, i64)> = Vec::new();
        for v in alph {
            if v == 0 { continue; }              // 0 is `combine`'s default output
            parts.push((word_dfa(&w, v, &(0..w.ntracks).map(tname).collect::<Vec<_>>())?
                            .reverse_determinize(), v));
        }
        let ns: Vec<Ns> = w.ns.iter().map(|n| Ns { msd: !n.msd, base: n.base.clone() }).collect();
        let nw = combine_dfas(&parts, &ns)?;
        let st = nw.live();
        self.words.insert(p[0].to_string(), Arc::new(nw));
        Ok(format!("WOK reverse {} states={}", p[0], st))
    }

    fn cmd_minimize(&mut self, rest: &str) -> Result<String, String> {
        let p: Vec<&str> = rest.split_whitespace().collect();
        if p.len() != 2 { return Err("usage: minimize <new> <old>".into()); }
        let w = self.word(p[1])?;
        let m = minimize_word(&w);
        let st = m.live();
        self.words.insert(p[0].to_string(), Arc::new(m));
        Ok(format!("WOK minimize {} states={}", p[0], st))
    }

    fn cmd_alphabet(&mut self, rest: &str) -> Result<String, String> {
        let p: Vec<&str> = rest.split_whitespace().collect();
        if p.len() < 3 { return Err("usage: alphabet <new> <ns> ... <old>".into()); }
        let old = p[p.len() - 1];
        let nss: Vec<Ns> = p[1..p.len() - 1].iter().map(|t| Ns::parse(t)).collect();
        if let Some(o) = old.strip_prefix('$') {
            let a = self.aut(o)?;
            if nss.len() != a.ntracks { return Err("wrong number of alphabets".into()); }
            if nss.iter().any(|n| *n != nss[0]) { return Err("mixed number systems are not supported".into()); }
            let mut a2 = (*a).clone();
            a2.ns = nss[0].clone();
            let st = a2.dfa.nstates;
            self.auts.insert(p[0].trim_start_matches('$').to_string(), Arc::new(a2));
            return Ok(format!("WOK alphabet {} states={}", p[0], st));
        }
        let w = self.word(old)?;
        if nss.len() != w.ntracks { return Err("wrong number of alphabets".into()); }
        let mut w2 = (*w).clone();
        w2.ns = nss;
        let st = w2.live();
        self.words.insert(p[0].to_string(), Arc::new(w2));
        Ok(format!("WOK alphabet {} states={}", p[0], st))
    }

    fn cmd_fixzero(&mut self, rest: &str, lead: bool) -> Result<String, String> {
        let p: Vec<&str> = rest.split_whitespace().collect();
        if p.len() != 2 { return Err("usage: fixleadzero <new> <old>".into()); }
        let a = self.aut(p[1].trim_start_matches('$'))?;
        self.install(&a.ns)?;
        let saved = dfa::is_lsd();
        dfa::set_lsd(!lead);
        let d = a.dfa.zero_closure().minimize();
        dfa::set_lsd(saved);
        let st = d.nstates;
        self.auts.insert(p[0].trim_start_matches('$').to_string(),
                         Arc::new(Aut { ns: a.ns.clone(), ntracks: a.ntracks, dfa: d, labels: a.labels.clone() }));
        Ok(format!("WOK fixleadzero {} states={}", p[0], st))
    }

    /// `test <name> <n>` -- the first `n` non-empty accepted words in shortlex
    /// order, with representations that start with the all-zero symbol removed
    /// (Walnut's `Main/Commands/Test.java`, via `removeLeadingZeros`).  Printed
    /// in Walnut's own format so the two engines' output can be diffed verbatim.
    fn cmd_test(&mut self, rest: &str) -> Result<String, String> {
        let p: Vec<&str> = rest.split_whitespace().collect();
        if p.len() != 2 { return Err("usage: test <name> <number>".into()); }
        let n: usize = p[1].parse().map_err(|_| "bad count")?;
        let a = self.aut(p[0].trim_start_matches('$'))?;
        self.install(&a.ns)?;
        let words = shortlex(&a.dfa, n, 40);
        let tracks = a.ntracks.max(1);
        let fmt: Vec<String> = words.iter().map(|w| {
            let mut s = String::new();
            for &sym in w {
                let ds: Vec<usize> = (0..tracks).map(|t| dfa::digit(sym, t, a.dfa.k)).collect();
                if tracks == 1 && ds[0] <= 9 { s.push_str(&ds[0].to_string()); }
                else {
                    s.push('[');
                    s.push_str(&ds.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(","));
                    s.push(']');
                }
            }
            s
        }).collect();
        Ok(format!("WOK test {} n={} words={}", p[0], fmt.len(), fmt.join(" ")))
    }

    fn cmd_inf(&mut self, rest: &str) -> Result<String, String> {
        let name = rest.split_whitespace().next().unwrap_or("");
        let a = self.aut(name.trim_start_matches('$'))?;
        self.install(&a.ns)?;
        let inf = is_infinite(&a.dfa);
        Ok(format!("WOK inf {} infinite={}", name, inf))
    }

    fn cmd_load(&mut self, rest: &str) -> Result<String, String> {
        let f = rest.trim();
        let mut cands = vec![self.root.join("Command Files").join(f), self.root.join(f), PathBuf::from(f)];
        cands.retain(|p| p.is_file());
        let p = cands.into_iter().next().ok_or_else(|| format!("no such file {:?}", f))?;
        let text = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let mut outs = Vec::new();
        for line in text.lines() {
            if let Some(o) = self.feed(line) { if !o.is_empty() { outs.push(o); } }
        }
        if let Some(o) = self.flush() { outs.push(o); }
        Ok(format!("WOK load {}\n{}", f, outs.join("\n")))
    }

    // -------------------------------------------------------------- helpers

    fn morph(&self, n: &str) -> Result<&Morph, String> {
        if let Some(m) = self.morphs.get(n) { return Ok(m); }
        Err(format!("no morphism {:?} (Morphism Library is not consulted for this)", n))
    }

    fn aut(&mut self, n: &str) -> Result<Arc<Aut>, String> {
        if let Some(a) = self.auts.get(n) { return Ok(a.clone()); }
        // fall back to Automata Library/
        let p = self.root.join("Automata Library").join(format!("{}.txt", n));
        if p.is_file() {
            let text = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
            let (w, ns) = self.parse_word_text(&text)?;
            if w.set_tracks { return Err(format!("automaton {:?} has a raw set alphabet", n)); }
            let vars: Vec<String> = (0..w.ntracks).map(tname).collect();
            let ns0 = ns[0].clone();
            self.install(&ns0)?;
            let d = word_dfa_nz(&w, &vars)?;     // accept = nonzero output
            let a = Arc::new(Aut { ns: ns0, ntracks: w.ntracks, dfa: d, labels: vars });
            self.auts.insert(n.to_string(), a.clone());
            return Ok(a);
        }
        Err(format!("no automaton {:?}", n))
    }

    fn word(&mut self, n: &str) -> Result<Arc<Word>, String> {
        if let Some(w) = self.words.get(n) { return Ok(w.clone()); }
        let p = self.root.join("Word Automata Library").join(format!("{}.txt", n));
        if !p.is_file() { return Err(format!("no word automaton {:?} ({})", n, p.display())); }
        let text = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let (w, _) = self.parse_word_text(&text)?;
        let w = Arc::new(w);
        self.words.insert(n.to_string(), w.clone());
        Ok(w)
    }

    /// Resolve a number system and install it in the process-global `numsys`
    /// / digit-order state that `dfa`/`base` read.
    fn install(&mut self, ns: &Ns) -> Result<(), String> {
        if self.installed.as_ref() == Some(ns) && numsys_matches(ns) { return Ok(()); }
        install_ns(self, ns)?;
        self.installed = Some(ns.clone());
        Ok(())
    }

    fn k(&mut self, ns: &Ns) -> Result<usize, String> {
        if let Some(k) = ns.is_base_k() {
            if k < 2 { return Err(format!("base {} is too small", k)); }
            return Ok(k);
        }
        let n = self.load_ns(&ns.base)?.ok_or_else(|| format!("unknown number system {:?}", ns.base))?;
        Ok(n.digits)
    }

    fn load_ns(&mut self, base: &str) -> Result<Option<Arc<NumSys>>, String> {
        if let Some(v) = self.nscache.get(base) { return Ok(v.clone()); }
        let r = numsys::load(base).map(|n| Arc::new(n));
        match r {
            Ok(n) => { self.nscache.insert(base.into(), Some(n.clone())); Ok(Some(n)) }
            Err(e) => { self.nscache.insert(base.into(), None); Err(e) }
        }
    }
}

fn numsys_matches(ns: &Ns) -> bool {
    let want_lsd = !ns.msd;
    if dfa::is_lsd() != want_lsd { return false; }
    match (ns.is_base_k(), numsys::active()) {
        (Some(_), None) => true,
        (None, Some(a)) => a.name == ns.base,
        _ => false,
    }
}

fn install_ns(c: &mut Compat, ns: &Ns) -> Result<(), String> {
    dfa::set_lsd(!ns.msd);
    match ns.is_base_k() {
        Some(k) => { if k < 2 { return Err(format!("base {} is too small", k)); }
                     numsys::set_active(None); Ok(()) }
        None => {
            let n = c.load_ns(&ns.base)?.ok_or_else(|| format!("unknown number system {:?}", ns.base))?;
            numsys::set_active(Some(n));
            Ok(())
        }
    }
}

/// Locate the Walnut checkout (for `Word Automata Library/` &c).
fn find_root() -> PathBuf {
    if let Ok(d) = std::env::var("AM_WALNUT_DIR") { return PathBuf::from(d); }
    for c in ["walnut7", "../walnut7", "../../walnut7", "."] {
        let p = PathBuf::from(c);
        if p.join("Word Automata Library").is_dir() { return p; }
    }
    PathBuf::from(".")
}

fn split1(s: &str) -> (&str, &str) {
    match s.find(char::is_whitespace) {
        Some(i) => (&s[..i], s[i..].trim_start()),
        None => (s, ""),
    }
}

/// `NAME "payload"` or just `"payload"` (Walnut's headless form).
fn parse_name_and_string(rest: &str) -> Result<(String, String), String> {
    let q = rest.find('"').ok_or("expected a quoted argument")?;
    let qe = rest.rfind('"').ok_or("unterminated quoted argument")?;
    if qe <= q { return Err("unterminated quoted argument".into()); }
    Ok((rest[..q].trim().to_string(), rest[q + 1..qe].to_string()))
}

/// Split an alphabet list into `{...}` groups and bare tokens.
fn split_alphabets(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_whitespace() { i += 1; continue; }
        if b[i] == '{' {
            let mut j = i;
            while j < b.len() && b[j] != '}' { j += 1; }
            out.push(b[i..=j.min(b.len() - 1)].iter().collect());
            i = j + 1;
        } else {
            let st = i;
            while i < b.len() && !b[i].is_whitespace() { i += 1; }
            out.push(b[st..i].iter().collect());
        }
    }
    out
}

/// `?msd_fib rest` -> (msd_fib, rest).  A number-system token deeper inside the
/// formula is accepted only if it names the same system.
fn strip_ns(pred: &str) -> Result<(Ns, String), String> {
    let mut ns: Option<Ns> = None;
    let b: Vec<char> = pred.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '?' {
            let st = i + 1;
            let mut j = st;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == '_') { j += 1; }
            if j > st {
                let tok: String = b[st..j].iter().collect();
                let n = Ns::parse(&tok);
                match &ns {
                    None => ns = Some(n),
                    Some(o) if *o == n => {}
                    Some(o) => return Err(format!(
                        "mixed number systems ({} and {}) in one predicate", o.text(), n.text())),
                }
                out.push(' ');
                i = j;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    Ok((ns.unwrap_or_default(), out))
}

fn parse_morphism(s: &str) -> Result<BTreeMap<i64, Vec<i64>>, String> {
    // "0->01 1->10", symbols above 9 / below 0 in brackets: "[10]->[11]"
    let mut map = BTreeMap::new();
    for part in s.split(|c: char| c == ',' || c.is_whitespace()).filter(|p| !p.trim().is_empty()) {
        let (l, r) = part.split_once("->").ok_or_else(|| format!("bad mapping {:?}", part))?;
        let key = parse_sym_list(l)?;
        if key.len() != 1 { return Err(format!("bad morphism domain {:?}", l)); }
        map.insert(key[0], parse_sym_list(r)?);
    }
    if map.is_empty() { return Err("morphism has no mappings".into()); }
    Ok(map)
}

/// `0012`, `[10]0[-3]` -> a list of integer symbols.
fn parse_sym_list(s: &str) -> Result<Vec<i64>, String> {
    let b: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_whitespace() { i += 1; continue; }
        if b[i] == '[' {
            let j = b[i..].iter().position(|&c| c == ']').ok_or("unterminated [")? + i;
            let t: String = b[i + 1..j].iter().collect();
            out.push(t.trim().parse::<i64>().map_err(|_| format!("bad symbol {:?}", t))?);
            i = j + 1;
        } else if b[i].is_ascii_digit() {
            out.push((b[i] as u8 - b'0') as i64);
            i += 1;
        } else {
            return Err(format!("bad symbol {:?} in {:?}", b[i], s));
        }
    }
    Ok(out)
}

// ------------------------------------------------------------------ Walnut file format

impl Compat {
    /// Parse a Walnut automaton/word-automaton file.  Differs from
    /// [`crate::numsys::parse_walnut`] in that the alphabet line's number-system
    /// tokens are resolved here (so `msd_fib` yields 2 digits) and raw set
    /// alphabets such as `{-1,1}` are recorded rather than rejected.
    fn parse_word_text(&mut self, text: &str) -> Result<(Word, Vec<Ns>), String> {
        let mut lines = text.lines()
            .map(|l| match l.find('#') { Some(p) => &l[..p], None => l })
            .filter(|l| !l.trim().is_empty());
        let alpha_line = lines.next().ok_or("empty automaton file")?;
        let mut nss: Vec<Ns> = Vec::new();
        let mut sizes: Vec<usize> = Vec::new();
        let mut set_tracks = false;
        // a raw `{a,b,c}` track is indexed by position in the sorted set, so the
        // transition digits in the file are values, not indices
        let mut maps: Vec<Option<HashMap<i64, usize>>> = Vec::new();
        for tok in split_alphabets(alpha_line) {
            if tok.starts_with('{') {
                let inner = tok.trim_start_matches('{').trim_end_matches('}');
                let mut vals: Vec<i64> = Vec::new();
                for p in inner.split(',') {
                    let p = p.trim();
                    if p.is_empty() { continue; }
                    vals.push(p.parse::<i64>().map_err(|_| format!("bad alphabet element {:?}", p))?);
                }
                vals.sort(); vals.dedup();
                let n = vals.len();
                if vals.iter().enumerate().all(|(i, &v)| v == i as i64) {
                    nss.push(Ns { msd: true, base: n.to_string() });
                    maps.push(None);
                } else {
                    set_tracks = true;
                    nss.push(Ns::default());
                    maps.push(Some(vals.iter().enumerate().map(|(i, &v)| (v, i)).collect()));
                }
                sizes.push(n);
            } else {
                let ns = Ns::parse(&tok);
                sizes.push(self.k(&ns)?);
                nss.push(ns);
                maps.push(None);
            }
        }
        if sizes.is_empty() { return Err("empty alphabet line".into()); }
        let digits = sizes[0];
        if sizes.iter().any(|&s| s != digits) {
            return Err("tracks with different digit-alphabet sizes are not supported".into());
        }
        let ntracks = sizes.len();
        let alpha = digits.pow(ntracks as u32);
        let mut out: HashMap<usize, i64> = HashMap::new();
        let mut tr: HashMap<(usize, usize), usize> = HashMap::new();
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
                if dests.len() != 1 { return Err("nondeterministic transitions are not supported".into()); }
                let t = dests[0];
                maxstate = maxstate.max(t);
                let toks: Vec<&str> = lhs.split_whitespace().collect();
                if toks.len() != ntracks {
                    return Err(format!("transition has {} inputs, expected {}", toks.len(), ntracks));
                }
                let mut choices: Vec<Vec<usize>> = Vec::with_capacity(ntracks);
                for (ti, tk) in toks.iter().enumerate() {
                    if *tk == "*" { choices.push((0..digits).collect()); continue; }
                    let d: i64 = tk.parse().map_err(|_| format!("bad input digit {:?}", tk))?;
                    let idx: i64 = match &maps[ti] {
                        Some(m) => *m.get(&d).ok_or_else(|| format!("digit {} outside the alphabet", d))? as i64,
                        None => d,
                    };
                    if idx < 0 || idx as usize >= digits {
                        return Err(format!("digit {} outside the alphabet", d));
                    }
                    choices.push(vec![idx as usize]);
                }
                let mut syms = vec![0usize];
                let mut mult = 1usize;
                for c in &choices {
                    let mut next = Vec::with_capacity(syms.len() * c.len());
                    for s in &syms { for d in c { next.push(s + d * mult); } }
                    mult *= digits;
                    syms = next;
                }
                for s in syms { tr.insert((q, s), t); }
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
        let n = maxstate + 1;
        let s0 = start.unwrap_or(0);
        let relab = |q: usize| if q == s0 { 0 } else if q == 0 { s0 } else { q };
        let nstates = n + 1;
        let dead = n as State;
        let mut trans = vec![dead; nstates * alpha];
        for ((q, s), t) in &tr { trans[relab(*q) * alpha + *s] = relab(*t) as State; }
        let mut o = vec![0i64; nstates];
        for q in 0..n { o[relab(q)] = *out.get(&q).unwrap_or(&0); }
        Ok((Word { ns: nss.clone(), digits, ntracks, nstates, alpha, trans, out: o,
                   dead: Some(n), set_tracks }, nss))
    }
}

// ------------------------------------------------------------------ automaton helpers

/// `W[v0]..[vr-1] = out` as a `Dfa` over `vars` (given in track order).
fn word_dfa(w: &Word, out: i64, vars: &[String]) -> Result<Dfa, String> {
    if vars.len() != w.ntracks { return Err("wrong number of index variables".into()); }
    let mut sorted = vars.to_vec();
    sorted.sort();
    if sorted.windows(2).any(|p| p[0] == p[1]) { return Err("repeated index variable".into()); }
    let accept: Vec<bool> = (0..w.nstates).map(|q| Some(q) != w.dead && w.out[q] == out).collect();
    let d = Dfa::new(w.digits, vars.to_vec(), w.nstates, w.trans.clone(), accept);
    let d = d.reorder(&sorted);
    // Raw on purpose: Walnut applies the leading/trailing-zero fix to the whole
    // `W[expr] REL …` atom *after* the index expression has been bound and
    // projected, not to the bare word predicate.  For a word automaton whose
    // start state does not loop on 0 (`C_alpha`) the two differ, and matching
    // Walnut means fixing at the atom, in `Cx::atom`.
    Ok(numsys::restrict(&d).minimize())
}

/// `W[..] != 0`, i.e. the ordinary automaton a Walnut `Automata Library/` file
/// denotes (Walnut stores predicates as DFAOs with outputs 0/1).
fn word_dfa_nz(w: &Word, vars: &[String]) -> Result<Dfa, String> {
    let mut sorted = vars.to_vec();
    sorted.sort();
    let accept: Vec<bool> = (0..w.nstates).map(|q| Some(q) != w.dead && w.out[q] != 0).collect();
    let d = Dfa::new(w.digits, vars.to_vec(), w.nstates, w.trans.clone(), accept).reorder(&sorted);
    Ok(numsys::restrict(&d).minimize())
}

/// `combine`: the DFAO whose output is the value of the LAST automaton in the
/// list that accepts, or 0 when none does.
fn combine_dfas(parts: &[(Dfa, i64)], ns: &[Ns]) -> Result<Word, String> {
    if parts.is_empty() { return Err("combine needs at least one automaton".into()); }
    let k = parts[0].0.k;
    let mut vars: Vec<String> = parts[0].0.vars.clone();
    for (d, _) in parts { for v in &d.vars { if !vars.contains(v) { vars.push(v.clone()); } } }
    vars.sort();
    let ext: Vec<Dfa> = parts.iter().map(|(d, _)| d.extend_vars(&vars)).collect();
    let alpha = ext[0].alpha;
    let mut index: HashMap<Vec<u32>, u32> = HashMap::new();
    let mut order: Vec<Vec<u32>> = Vec::new();
    let init: Vec<u32> = vec![0; ext.len()];
    index.insert(init.clone(), 0);
    order.push(init);
    let mut trans: Vec<State> = Vec::new();
    let mut i = 0;
    while i < order.len() {
        let cur = order[i].clone();
        for s in 0..alpha {
            let nxt: Vec<u32> = cur.iter().enumerate()
                .map(|(j, &q)| ext[j].trans[q as usize * alpha + s]).collect();
            let id = *index.entry(nxt.clone()).or_insert_with(|| { order.push(nxt); (order.len() - 1) as u32 });
            trans.push(id);
        }
        i += 1;
    }
    let n = order.len();
    let out: Vec<i64> = order.iter().map(|tup| {
        let mut v = 0i64;
        for (j, &q) in tup.iter().enumerate() { if ext[j].accept[q as usize] { v = parts[j].1; } }
        v
    }).collect();
    let ntracks = vars.len();
    let nsv: Vec<Ns> = (0..ntracks).map(|i| ns.get(i).cloned().unwrap_or_default()).collect();
    let w = Word { ns: nsv, digits: k, ntracks, nstates: n, alpha, trans, out,
                   dead: None, set_tracks: false };
    Ok(minimize_word(&w))
}

/// Moore partition refinement on (output, transition classes), then a BFS
/// renumbering from the start state.  Unreachable states are dropped.
fn minimize_word(w: &Word) -> Word {
    let n = w.nstates;
    let alpha = w.alpha;
    let mut seen = vec![false; n];
    let mut st = vec![0usize];
    seen[0] = true;
    while let Some(q) = st.pop() {
        for s in 0..alpha { let t = w.t(q, s); if !seen[t] { seen[t] = true; st.push(t); } }
    }
    let mut outs: Vec<i64> = w.out.iter().cloned().collect();
    outs.sort(); outs.dedup();
    let mut cls: Vec<usize> = (0..n).map(|q| outs.binary_search(&w.out[q]).unwrap()).collect();
    let mut ncls_count = outs.len();
    loop {
        let mut sig: HashMap<Vec<usize>, usize> = HashMap::new();
        let mut next = vec![0usize; n];
        let mut c = 0usize;
        for q in 0..n {
            let mut key = Vec::with_capacity(alpha + 1);
            key.push(cls[q]);
            for s in 0..alpha { key.push(cls[w.t(q, s)]); }
            let id = *sig.entry(key).or_insert_with(|| { let v = c; c += 1; v });
            next[q] = id;
        }
        cls = next;
        if c == ncls_count { break; }
        ncls_count = c;
    }
    // one representative state per class, then BFS from the start class
    let mut rep: HashMap<usize, usize> = HashMap::new();
    for q in 0..n { if seen[q] { rep.entry(cls[q]).or_insert(q); } }
    for q in 0..n { rep.entry(cls[q]).or_insert(q); }
    let mut map: HashMap<usize, usize> = HashMap::new();
    let mut order: Vec<usize> = vec![cls[0]];
    map.insert(cls[0], 0);
    let mut i = 0;
    while i < order.len() {
        let q = rep[&order[i]];
        for s in 0..alpha {
            let tc = cls[w.t(q, s)];
            if !map.contains_key(&tc) { map.insert(tc, order.len()); order.push(tc); }
        }
        i += 1;
    }
    let m = order.len();
    let mut trans = vec![0 as State; m * alpha];
    let mut out = vec![0i64; m];
    for (ci, c) in order.iter().enumerate() {
        let q = rep[c];
        out[ci] = w.out[q];
        for s in 0..alpha { trans[ci * alpha + s] = map[&cls[w.t(q, s)]] as State; }
    }
    let dead = w.dead.and_then(|d| map.get(&cls[d]).cloned());
    Word { ns: w.ns.clone(), digits: w.digits, ntracks: w.ntracks, nstates: m, alpha,
           trans, out, dead, set_tracks: w.set_tracks }
}

/// The first `want` non-empty accepted words in shortlex order over the symbol
/// index, with the redundant representations dropped: in msd the all-zero
/// *first* symbol is disallowed, in lsd the all-zero *last* symbol is
/// (`AutomatonLogicalOps.removeLeadingZerosHelper` reverses its filter for lsd).
/// Words are enumerated length by length with a reachability table so the search
/// never explores a prefix that cannot be completed.
fn shortlex(a: &Dfa, want: usize, maxlen: usize) -> Vec<Vec<usize>> {
    let lsd = dfa::is_lsd();
    let mut out: Vec<Vec<usize>> = Vec::new();
    // cnt[l][q] = does some word of length exactly l take q to an accepting state?
    let mut cnt: Vec<Vec<bool>> = vec![a.accept.clone()];
    for l in 1..=maxlen {
        let prev = cnt[l - 1].clone();
        cnt.push((0..a.nstates).map(|q| (0..a.alpha).any(|s| prev[a.t(q, s)])).collect());
    }
    for l in 1..=maxlen {
        if out.len() >= want { break; }
        let mut w = vec![0usize; l];
        fn go(a: &Dfa, cnt: &Vec<Vec<bool>>, q: usize, d: usize, l: usize, lsd: bool,
              w: &mut Vec<usize>, out: &mut Vec<Vec<usize>>, want: usize) {
            if out.len() >= want { return; }
            if d == l { if a.accept[q] { out.push(w.clone()); } return; }
            for s in 0..a.alpha {
                if s == 0 && ((!lsd && d == 0) || (lsd && d + 1 == l)) { continue; }
                let t = a.t(q, s);
                if !cnt[l - d - 1][t] { continue; }
                w[d] = s;
                go(a, cnt, t, d + 1, l, lsd, w, out, want);
                if out.len() >= want { return; }
            }
        }
        go(a, &cnt, 0, 0, l, lsd, &mut w, &mut out, want);
    }
    out
}

/// Does the automaton accept infinitely many words?  (A cycle on some
/// start-to-accept path.)
fn is_infinite(a: &Dfa) -> bool {
    let n = a.nstates;
    let mut useful = a.accept.clone();
    loop {
        let mut ch = false;
        for s in 0..n { if !useful[s] {
            for x in 0..a.alpha { if useful[a.t(s, x)] { useful[s] = true; ch = true; break; } }
        }}
        if !ch { break; }
    }
    let mut seen = vec![false; n];
    if !useful[0] { return false; }
    let mut stack = vec![0usize];
    seen[0] = true;
    while let Some(s) = stack.pop() {
        for x in 0..a.alpha { let t = a.t(s, x); if useful[t] && !seen[t] { seen[t] = true; stack.push(t); } }
    }
    // cycle detection on the useful+reachable subgraph
    let mut colour = vec![0u8; n];
    fn dfs(a: &Dfa, s: usize, seen: &[bool], colour: &mut Vec<u8>) -> bool {
        colour[s] = 1;
        for x in 0..a.alpha {
            let t = a.t(s, x);
            if !seen[t] { continue; }
            if colour[t] == 1 { return true; }
            if colour[t] == 0 && dfs(a, t, seen, colour) { return true; }
        }
        colour[s] = 2;
        false
    }
    dfs(a, 0, &seen, &mut colour)
}

// ------------------------------------------------------------------ lexer

#[derive(Clone, Debug, PartialEq)]
enum Tk {
    Log(String),       // & | ^ ~ => <=> ` E A I
    Rel(String),
    Ari(char),         // + - * / _
    Ident(String),     // a variable or a word-automaton name
    Fun(String),       // $name
    Macro(String),     // #name
    Num(i64),
    Letter(i64),
    LP, RP, LB, RB, Comma,
}

/// Walnut's tokenizer (`Main/Predicate.java`): the alternatives are tried in a
/// fixed order, and `A`, `E`, `I` are *always* logical operators, which is why
/// Walnut identifiers may not begin with those letters (the `.NAME` form exists
/// to escape that).
fn lex(s: &str) -> Result<Vec<Tk>, String> {
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i];
        if c.is_whitespace() { i += 1; continue; }
        if s[i..].starts_with("<=>") { out.push(Tk::Log("<=>".into())); i += 3; continue; }
        if s[i..].starts_with("=>") { out.push(Tk::Log("=>".into())); i += 2; continue; }
        if "&|^~`".contains(c) { out.push(Tk::Log(c.to_string())); i += 1; continue; }
        if c == 'E' || c == 'A' || c == 'I' { out.push(Tk::Log(c.to_string())); i += 1; continue; }
        if s[i..].starts_with(">=") { out.push(Tk::Rel(">=".into())); i += 2; continue; }
        if s[i..].starts_with("<=") { out.push(Tk::Rel("<=".into())); i += 2; continue; }
        if s[i..].starts_with("!=") { out.push(Tk::Rel("!=".into())); i += 2; continue; }
        if "<>=".contains(c) { out.push(Tk::Rel(c.to_string())); i += 1; continue; }
        if "_/*+-".contains(c) { out.push(Tk::Ari(c)); i += 1; continue; }
        if c == '.' && i + 1 < b.len() && b[i + 1].is_ascii_alphabetic() {
            let st = i + 1; let mut j = st;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == '_') { j += 1; }
            out.push(Tk::Ident(b[st..j].iter().collect())); i = j; continue;
        }
        if c == '$' || c == '#' {
            let st = i + 1; let mut j = st;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == '_') { j += 1; }
            if j == st { return Err(format!("expected a name after {:?}", c)); }
            let n: String = b[st..j].iter().collect();
            out.push(if c == '$' { Tk::Fun(n) } else { Tk::Macro(n) });
            i = j; continue;
        }
        if c.is_ascii_alphabetic() {
            let st = i; let mut j = i;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == '_') { j += 1; }
            out.push(Tk::Ident(b[st..j].iter().collect())); i = j; continue;
        }
        if c.is_ascii_digit() {
            let st = i; let mut j = i;
            while j < b.len() && b[j].is_ascii_digit() { j += 1; }
            let t: String = b[st..j].iter().collect();
            out.push(Tk::Num(t.parse().map_err(|_| "number too large")?)); i = j; continue;
        }
        if c == '@' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_whitespace() { j += 1; }
            let neg = if j < b.len() && (b[j] == '-' || b[j] == '+') { let n = b[j] == '-'; j += 1; n } else { false };
            let st = j;
            while j < b.len() && b[j].is_ascii_digit() { j += 1; }
            if j == st { return Err("expected a number after @".into()); }
            let t: String = b[st..j].iter().collect();
            let v: i64 = t.parse().map_err(|_| "bad letter")?;
            out.push(Tk::Letter(if neg { -v } else { v })); i = j; continue;
        }
        match c {
            '(' => out.push(Tk::LP), ')' => out.push(Tk::RP),
            '[' => out.push(Tk::LB), ']' => out.push(Tk::RB),
            ',' => out.push(Tk::Comma),
            _ => return Err(format!("unexpected character {:?}", c)),
        }
        i += 1;
    }
    Ok(out)
}

// ------------------------------------------------------------------ AST

#[derive(Clone, Debug)]
enum N {
    Num(i64),
    Letter(i64),
    Var(String),
    Bin(char, Box<N>, Box<N>),
    Neg(Box<N>),
    Wrd(String, Vec<N>),
    Slot(usize),
    Cmp(Box<N>, Rel, Box<N>),
    Log(char, Box<N>, Box<N>),        // & | ^ i(mply) e(quiv)
    Not(Box<N>),
    Quant(bool, Vec<String>, Box<N>), // true = forall
    Call(String, Vec<N>),
    Bool(bool),
}

struct Parser { t: Vec<Tk>, i: usize }

impl Parser {
    fn parse(src: &str) -> Result<N, String> {
        let s = src.trim();
        if s.eq_ignore_ascii_case("true") { return Ok(N::Bool(true)); }
        if s.eq_ignore_ascii_case("false") { return Ok(N::Bool(false)); }
        let mut p = Parser { t: lex(s)?, i: 0 };
        let n = p.p_iff()?;
        if p.i != p.t.len() { return Err(format!("trailing input at token {} ({:?})", p.i, p.t.get(p.i))); }
        Ok(n)
    }
    fn peek(&self) -> Option<&Tk> { self.t.get(self.i) }
    fn eat_log(&mut self, s: &str) -> bool {
        if let Some(Tk::Log(x)) = self.peek() { if x == s { self.i += 1; return true; } }
        false
    }
    fn eat(&mut self, t: Tk) -> bool {
        if self.peek() == Some(&t) { self.i += 1; return true; }
        false
    }
    fn expect(&mut self, t: Tk) -> Result<(), String> {
        if self.eat(t.clone()) { Ok(()) } else { Err(format!("expected {:?} at token {} ({:?})", t, self.i, self.peek())) }
    }

    fn p_iff(&mut self) -> Result<N, String> {
        let mut a = self.p_imp()?;
        while self.eat_log("<=>") { a = N::Log('e', Box::new(a), Box::new(self.p_imp()?)); }
        Ok(a)
    }
    fn p_imp(&mut self) -> Result<N, String> {
        let mut a = self.p_bool()?;
        while self.eat_log("=>") { a = N::Log('i', Box::new(a), Box::new(self.p_bool()?)); }
        Ok(a)
    }
    /// `&`, `|` and `^` share one priority level in Walnut and associate left.
    fn p_bool(&mut self) -> Result<N, String> {
        let mut a = self.p_not()?;
        loop {
            let op = match self.peek() {
                Some(Tk::Log(x)) if x == "&" || x == "|" || x == "^" => x.chars().next().unwrap(),
                _ => break,
            };
            self.i += 1;
            a = N::Log(op, Box::new(a), Box::new(self.p_not()?));
        }
        Ok(a)
    }
    fn p_not(&mut self) -> Result<N, String> {
        if let Some(Tk::Log(x)) = self.peek().cloned() {
            if x == "~" { self.i += 1; return Ok(N::Not(Box::new(self.p_not()?))); }
            if x == "`" { return Err("the reverse operator ` is not supported".into()); }
            if x == "I" { return Err("the infinite quantifier I is not supported".into()); }
            if x == "E" || x == "A" {
                self.i += 1;
                let mut vs = Vec::new();
                loop {
                    match self.peek().cloned() {
                        Some(Tk::Ident(v)) => { self.i += 1; vs.push(v); }
                        _ => return Err("expected a variable list after a quantifier".into()),
                    }
                    if !self.eat(Tk::Comma) { break; }
                }
                let body = self.p_iff()?;
                return Ok(N::Quant(x == "A", vs, Box::new(body)));
            }
        }
        self.p_rel()
    }
    fn p_rel(&mut self) -> Result<N, String> {
        let a = self.p_arith()?;
        if let Some(Tk::Rel(r)) = self.peek().cloned() {
            self.i += 1;
            let rel = match r.as_str() {
                "=" => Rel::Eq, "!=" => Rel::Ne, "<" => Rel::Lt,
                "<=" => Rel::Le, ">" => Rel::Gt, _ => Rel::Ge,
            };
            let b = self.p_arith()?;
            return Ok(N::Cmp(Box::new(a), rel, Box::new(b)));
        }
        Ok(a)
    }
    fn p_arith(&mut self) -> Result<N, String> {
        let mut a = self.p_term()?;
        loop {
            let c = match self.peek() { Some(Tk::Ari(c)) if *c == '+' || *c == '-' => *c, _ => break };
            self.i += 1;
            a = N::Bin(c, Box::new(a), Box::new(self.p_term()?));
        }
        Ok(a)
    }
    fn p_term(&mut self) -> Result<N, String> {
        let mut a = self.p_unary()?;
        loop {
            let c = match self.peek() { Some(Tk::Ari(c)) if *c == '*' || *c == '/' => *c, _ => break };
            self.i += 1;
            a = N::Bin(c, Box::new(a), Box::new(self.p_unary()?));
        }
        Ok(a)
    }
    fn p_unary(&mut self) -> Result<N, String> {
        if let Some(Tk::Ari('_')) = self.peek() { self.i += 1; return Ok(N::Neg(Box::new(self.p_unary()?))); }
        self.p_prim()
    }
    fn p_prim(&mut self) -> Result<N, String> {
        match self.peek().cloned() {
            Some(Tk::Num(n)) => { self.i += 1; Ok(N::Num(n)) }
            Some(Tk::Letter(a)) => { self.i += 1; Ok(N::Letter(a)) }
            Some(Tk::LP) => { self.i += 1; let n = self.p_iff()?; self.expect(Tk::RP)?; Ok(n) }
            Some(Tk::Fun(f)) => {
                self.i += 1;
                self.expect(Tk::LP)?;
                let mut args = Vec::new();
                if !self.eat(Tk::RP) {
                    loop {
                        args.push(self.p_iff()?);
                        if !self.eat(Tk::Comma) { break; }
                    }
                    self.expect(Tk::RP)?;
                }
                Ok(N::Call(f, args))
            }
            Some(Tk::Macro(m)) => Err(format!("macros (#{}) are not supported", m)),
            Some(Tk::Ident(v)) => {
                self.i += 1;
                if self.peek() == Some(&Tk::LB) {
                    let mut idx = Vec::new();
                    while self.eat(Tk::LB) {
                        idx.push(self.p_iff()?);
                        self.expect(Tk::RB)?;
                    }
                    return Ok(N::Wrd(v, idx));
                }
                Ok(N::Var(v))
            }
            other => Err(format!("expected a term, found {:?}", other)),
        }
    }
}

/// Walnut's leading/trailing-zero fix, reproduced exactly.
///
/// lsd (`fixTrailingZerosProblem`) only widens the accepting set, which is what
/// `Dfa::zero_closure` already does.  msd (`fixLeadingZerosProblem`) is subtler:
/// Walnut **adds a 0-self-loop at the start state** before collecting the
/// zero-reachable set and re-determinizing (`zeroReachableStates`), so the
/// resulting automaton also absorbs zeros at any later point where the start
/// state recurs.  For every leading-zero-robust automaton (delta(q0,0) = q0) the
/// two agree; they differ for a raw `reg` automaton such as `(012)*2*|(012)*01`,
/// and matching Walnut there is the whole point of this layer.
fn fix_zeros(d: &Dfa) -> Dfa {
    if dfa::is_lsd() { return d.zero_closure().minimize(); }
    let mut trans: Vec<Vec<State>> = (0..d.nstates * d.alpha).map(|i| vec![d.trans[i]]).collect();
    if !trans[0].contains(&0) { trans[0].push(0); }          // the added self-loop
    let mut init: Vec<State> = Vec::new();
    let mut stack = vec![0u32];
    while let Some(q) = stack.pop() {
        if init.contains(&q) { continue; }
        init.push(q);
        for &t in &trans[q as usize * d.alpha] { if !init.contains(&t) { stack.push(t); } }
    }
    init.sort();
    Nfa { k: d.k, vars: d.vars.clone(), alpha: d.alpha, nstates: d.nstates,
          trans, init, accept: d.accept.clone() }.determinize().minimize()
}

/// Union of two automata whose variable lists differ.
///
/// `Dfa::product` skips the validity re-restriction for `|`/`^` on the grounds
/// that "both operands reject every invalid word".  That holds only while both
/// operands *mention* the track: cylindrifying the narrower operand up to the
/// union of the variable lists lets it accept anything on the new tracks, so
/// under a numeration system the union can pick up words that are not valid
/// representations.  Re-restricting exactly when the lists differ costs one
/// small product and keeps every intermediate automaton validity-closed.
fn or_r(x: &Dfa, y: &Dfa) -> Dfa {
    let r = x.or(y);
    if x.vars != y.vars { numsys::restrict(&r) } else { r }
}

fn xor_r(x: &Dfa, y: &Dfa) -> Dfa {
    let r = x.product(y, |p, q| p != q);
    if x.vars != y.vars { numsys::restrict(&r) } else { r }
}

// ------------------------------------------------------------------ compiler

struct Cx<'a> {
    c: &'a mut Compat,
    k: usize,
    ns: Ns,
    fresh: usize,
}

impl<'a> Cx<'a> {
    fn newvar(&mut self) -> String { self.fresh += 1; format!("#v{}", self.fresh) }

    /// `z = x + y`, safe when `x` and `y` are the same variable.
    fn add_auto(&mut self, x: &str, y: &str, z: &str) -> Dfa {
        if x != y { return base::adder(self.k, x, y, z); }
        let u = self.newvar();
        base::equal(self.k, x, &u).and(&base::adder(self.k, x, &u, z)).exists(&u)
    }

    /// Automaton asserting `v = <the nonnegative linear form t>`; returns
    /// `(automaton, v, v_is_internal)`.
    fn lin_auto(&mut self, t: &Lin) -> Result<(Dfa, String, bool), String> {
        if t.c < 0 || t.coef.values().any(|v| *v < 0) { return Err("negative linear form".into()); }
        if let Some(v) = t.is_plain_var() {
            return Ok((Dfa::constant(self.k, vec![v.clone()], true), v, false));
        }
        let mut parts: Vec<(String, Option<Dfa>)> = Vec::new();
        for (v, m) in &t.coef { for _ in 0..*m { parts.push((v.clone(), None)); } }
        if t.c > 0 || parts.is_empty() {
            let w = self.newvar();
            let a = base::constant(self.k, &w, t.c as u64)?;
            parts.push((w, Some(a)));
        }
        if parts.len() == 1 {
            let (name, aut) = parts.pop().unwrap();
            return match aut {
                Some(a) => Ok((a, name, true)),
                None => Ok((Dfa::constant(self.k, vec![name.clone()], true), name, false)),
            };
        }
        let (first, faut) = parts.remove(0);
        let mut acc = faut.unwrap_or_else(|| Dfa::constant(self.k, vec![first.clone()], true));
        let mut cur = first;
        let mut cur_fresh = !t.coef.contains_key(&cur);
        for (name, aut) in parts {
            let next = self.newvar();
            let step = self.add_auto(&cur, &name, &next);
            acc = acc.and(&step);
            if let Some(a) = aut { acc = acc.and(&a); }
            if cur_fresh { acc = acc.exists(&cur); }
            if !t.coef.contains_key(&name) { acc = acc.exists(&name); }
            cur = next;
            cur_fresh = true;
        }
        Ok((acc, cur, true))
    }

    /// `a REL b` for two linear forms, moving negative coefficients across the
    /// relation -- exactly Walnut's semantics for subtraction over N.
    fn cmp_auto(&mut self, a: &Lin, r: &Rel, b: &Lin) -> Result<Dfa, String> {
        let d = a.add(b, -1);
        if d.coef.is_empty() {
            let v = d.c;
            let t = match r {
                Rel::Eq => v == 0, Rel::Ne => v != 0, Rel::Lt => v < 0,
                Rel::Le => v <= 0, Rel::Gt => v > 0, Rel::Ge => v >= 0,
            };
            // The variables have cancelled (`a+1-1 = a`), but they are still free
            // variables of the formula for Walnut, so cylindrify rather than drop.
            let mut vars: Vec<String> = a.coef.keys().chain(b.coef.keys()).cloned().collect();
            vars.sort(); vars.dedup();
            return Ok(Dfa::constant(self.k, vars, t));
        }
        let (p, n) = d.split();
        let (ap, vp, fp) = self.lin_auto(&p)?;
        let (an, vn, fn_) = self.lin_auto(&n)?;
        let rel = match r {
            Rel::Eq => base::equal(self.k, &vp, &vn),
            Rel::Ne => base::equal(self.k, &vp, &vn).complement(),
            Rel::Lt => base::less_than(self.k, &vp, &vn),
            Rel::Ge => base::less_than(self.k, &vp, &vn).complement(),
            Rel::Gt => base::less_than(self.k, &vn, &vp),
            Rel::Le => base::less_than(self.k, &vn, &vp).complement(),
        };
        let mut res = ap.and(&an).and(&rel);
        if fp { res = res.exists(&vp); }
        if fn_ && vn != vp { res = res.exists(&vn); }
        Ok(res)
    }

    fn lin(&self, n: &N, subst: &[i64]) -> Result<Lin, String> {
        Ok(match n {
            N::Num(v) => Lin::num(*v),
            N::Letter(v) => Lin::num(*v),
            N::Slot(j) => Lin::num(subst[*j]),
            N::Var(v) => Lin::var(v),
            N::Neg(x) => self.lin(x, subst)?.scale(-1),
            N::Bin('+', a, b) => self.lin(a, subst)?.add(&self.lin(b, subst)?, 1),
            N::Bin('-', a, b) => self.lin(a, subst)?.add(&self.lin(b, subst)?, -1),
            N::Bin('*', a, b) => {
                let (x, y) = (self.lin(a, subst)?, self.lin(b, subst)?);
                if x.coef.is_empty() { y.scale(x.c) }
                else if y.coef.is_empty() { x.scale(y.c) }
                else { return Err("multiplication of two non-constant terms is not supported".into()) }
            }
            N::Bin('/', _, _) => return Err("integer division (/) is not supported".into()),
            N::Wrd(w, _) => return Err(format!("word automaton {} used inside a word index", w)),
            _ => return Err("expected an arithmetic term".into()),
        })
    }

    /// Replace every `W[..]` occurrence by a `Slot` and record it.
    fn strip(&self, n: &N, occ: &mut Vec<(String, Vec<N>)>) -> N {
        match n {
            N::Wrd(w, idx) => { occ.push((w.clone(), idx.clone())); N::Slot(occ.len() - 1) }
            N::Bin(c, a, b) => N::Bin(*c, Box::new(self.strip(a, occ)), Box::new(self.strip(b, occ))),
            N::Neg(a) => N::Neg(Box::new(self.strip(a, occ))),
            other => other.clone(),
        }
    }

    /// A comparison atom.  Word values range over a finite output alphabet, so
    /// the atom is the disjunction, over every combination of output letters, of
    /// "each word takes that letter" AND "the resulting arithmetic comparison holds".
    fn atom(&mut self, l: &N, r: &Rel, rr: &N) -> Result<Dfa, String> {
        let mut occ: Vec<(String, Vec<N>)> = Vec::new();
        let ls = self.strip(l, &mut occ);
        let rs = self.strip(rr, &mut occ);
        if occ.is_empty() {
            let (a, b) = (self.lin(&ls, &[])?, self.lin(&rs, &[])?);
            return self.cmp_auto(&a, r, &b);
        }
        // bind every index term to a variable
        let mut binder = Dfa::constant(self.k, vec![], true);
        let mut projs: Vec<String> = Vec::new();
        let mut wvars: Vec<Vec<String>> = Vec::new();
        let mut alphas: Vec<Vec<i64>> = Vec::new();
        let mut words: Vec<Arc<Word>> = Vec::new();
        for (name, idx) in &occ {
            let w = self.c.word(name)?;
            if w.ntracks != idx.len() {
                return Err(format!("{} takes {} indices, {} given", name, w.ntracks, idx.len()));
            }
            if w.digits != self.k {
                return Err(format!("{} is over a {}-digit alphabet, used under {} ({} digits)",
                                   name, w.digits, self.ns.text(), self.k));
            }
            let mut vs: Vec<String> = Vec::new();
            for t in idx {
                let lin = self.lin(t, &[])?;
                match lin.is_plain_var() {
                    Some(v) if !vs.contains(&v) => vs.push(v),
                    _ => {
                        let z = self.newvar();
                        let b = self.cmp_auto(&Lin::var(&z), &Rel::Eq, &lin)?;
                        binder = binder.and(&b);
                        projs.push(z.clone());
                        vs.push(z);
                    }
                }
            }
            alphas.push(w.out_alphabet());
            wvars.push(vs);
            words.push(w);
        }
        // cartesian product over the output alphabets
        let mut acc = Dfa::constant(self.k, vec![], false);
        let mut combo = vec![0usize; occ.len()];
        loop {
            let vals: Vec<i64> = combo.iter().enumerate().map(|(j, &i)| alphas[j][i]).collect();
            let (a, b) = (self.lin(&ls, &vals)?, self.lin(&rs, &vals)?);
            let cmp = self.cmp_auto(&a, r, &b)?;
            if cmp.is_nonempty() {
                let mut branch = cmp;
                for j in 0..occ.len() {
                    branch = branch.and(&word_dfa(&words[j], vals[j], &wvars[j])?);
                }
                acc = or_r(&acc, &branch);
            }
            // odometer
            let mut i = 0;
            loop {
                if i == combo.len() { break; }
                combo[i] += 1;
                if combo[i] < alphas[i].len() { break; }
                combo[i] = 0;
                i += 1;
            }
            if i == combo.len() { break; }
        }
        let mut res = acc.and(&binder);
        for z in &projs { res = res.exists(z); }
        // Walnut's `Word.act()` finishes with the leading/trailing-zero fix; for a
        // zero-robust word automaton this is a no-op, for `C_alpha` it is not.
        Ok(fix_zeros(&res))
    }

    fn call(&mut self, name: &str, args: &[N]) -> Result<Dfa, String> {
        let a = self.c.aut(name)?;
        // Walnut attaches a number system to each *track*, not to the command, and
        // happily uses an automaton saved under one system inside a predicate
        // evaluated under another as long as the digit alphabets match.  Matching
        // that is the whole point of this layer, so only the alphabet is checked.
        if a.dfa.k != self.k {
            return Err(format!("${} is over a {}-digit alphabet, used under {} ({} digits)",
                               name, a.dfa.k, self.ns.text(), self.k));
        }
        if a.ntracks != args.len() {
            return Err(format!("${} takes {} arguments, {} given", name, a.ntracks, args.len()));
        }
        let body = numsys::restrict(&fix_zeros(&a.dfa)).minimize();
        // fast path: all arguments are distinct plain variables -> just rename
        let plains: Vec<Option<String>> = args.iter().map(|x| match x {
            N::Var(v) => Some(v.clone()), _ => None }).collect();
        if plains.iter().all(|p| p.is_some()) {
            let names: Vec<String> = plains.iter().map(|p| p.clone().unwrap()).collect();
            let mut s = names.clone(); s.sort(); s.dedup();
            if s.len() == names.len() {
                let mut ren = HashMap::new();
                for (i, v) in names.iter().enumerate() { ren.insert(tname(i), v.clone()); }
                return Ok(body.rename(&|v| ren.get(v).cloned().unwrap_or_else(|| v.to_string())));
            }
        }
        let mut ren = HashMap::new();
        let mut fresh = Vec::new();
        for i in 0..args.len() {
            let w = self.newvar();
            ren.insert(tname(i), w.clone());
            fresh.push(w);
        }
        let mut acc = body.rename(&|v| ren.get(v).cloned().unwrap_or_else(|| v.to_string()));
        for (i, arg) in args.iter().enumerate() {
            let mut occ = Vec::new();
            let sk = self.strip(arg, &mut occ);
            if !occ.is_empty() { return Err("word automata inside predicate arguments are not supported".into()); }
            let lin = self.lin(&sk, &[])?;
            let b = self.cmp_auto(&Lin::var(&fresh[i]), &Rel::Eq, &lin)?;
            acc = acc.and(&b).exists(&fresh[i]);
        }
        Ok(acc)
    }

    fn form(&mut self, n: &N) -> Result<Dfa, String> {
        Ok(match n {
            N::Bool(b) => Dfa::constant(self.k, vec![], *b),
            N::Cmp(a, r, b) => self.atom(a, r, b)?,
            N::Not(a) => self.form(a)?.complement(),
            N::Log(op, a, b) => {
                let x = self.form(a)?;
                let y = self.form(b)?;
                match op {
                    '&' => x.and(&y),
                    '|' => or_r(&x, &y),
                    '^' => xor_r(&x, &y),
                    'i' => x.implies(&y),
                    _ => x.iff(&y),
                }
            }
            N::Quant(all, vs, body) => {
                let mut d = self.form(body)?;
                for v in vs.iter().rev() {
                    d = if *all { d.forall(v) } else { d.exists(v) };
                }
                d
            }
            N::Call(f, args) => self.call(f, args)?,
            N::Wrd(w, _) => return Err(format!("word automaton {} used as a formula", w)),
            _ => return Err("expected a formula".into()),
        })
    }
}

// ------------------------------------------------------------------ regular expressions

/// Walnut's `reg` command: a regular expression over the *digit-tuple* alphabet.
/// Single-track expressions may use bare digits (`0*10*`); multi-track ones use
/// bracketed vectors (`[1,0][0,0]*`).  The operator set is dk.brics's, which
/// Walnut delegates to: union `|`, intersection `&`, concatenation, `*`, `+`,
/// `?`, complement `~`, and `.` for "any letter".
mod regex {
    use super::*;

    /// epsilon-NFA fragment
    struct E {
        n: usize,
        eps: Vec<Vec<usize>>,
        sym: Vec<Vec<(usize, usize)>>, // (symbol, target)
        start: usize,
        acc: Vec<bool>,
    }

    impl E {
        fn empty() -> E { E { n: 1, eps: vec![vec![]], sym: vec![vec![]], start: 0, acc: vec![true] } }
        fn none() -> E { E { n: 1, eps: vec![vec![]], sym: vec![vec![]], start: 0, acc: vec![false] } }
        fn lit(syms: &[usize]) -> E {
            let mut e = E { n: 2, eps: vec![vec![], vec![]], sym: vec![vec![], vec![]], start: 0, acc: vec![false, true] };
            for &s in syms { e.sym[0].push((s, 1)); }
            e
        }
        fn shift(&self, off: usize) -> (Vec<Vec<usize>>, Vec<Vec<(usize, usize)>>) {
            (self.eps.iter().map(|v| v.iter().map(|x| x + off).collect()).collect(),
             self.sym.iter().map(|v| v.iter().map(|(s, t)| (*s, t + off)).collect()).collect())
        }
        fn union(a: &E, b: &E) -> E {
            let (ae, asy) = a.shift(1);
            let (be, bsy) = b.shift(1 + a.n);
            // eps to each side's OWN start state -- `star` moves the start to a
            // freshly appended state, so assuming 0 silently drops that branch
            let mut eps = vec![vec![1 + a.start, 1 + a.n + b.start]];
            eps.extend(ae); eps.extend(be);
            let mut sym = vec![vec![]];
            sym.extend(asy); sym.extend(bsy);
            let mut acc = vec![false];
            acc.extend(a.acc.iter().cloned());
            acc.extend(b.acc.iter().cloned());
            E { n: 1 + a.n + b.n, eps, sym, start: 0, acc }
        }
        fn concat(a: &E, b: &E) -> E {
            let (mut ae, mut asy) = (a.eps.clone(), a.sym.clone());
            let (be, bsy) = b.shift(a.n);
            for q in 0..a.n { if a.acc[q] { ae[q].push(a.n + b.start); } }
            ae.extend(be); asy.extend(bsy);
            let mut acc = vec![false; a.n];
            acc.extend(b.acc.iter().cloned());
            E { n: a.n + b.n, eps: ae, sym: asy, start: a.start, acc }
        }
        fn star(a: &E) -> E {
            let (mut ae, mut asy) = (a.eps.clone(), a.sym.clone());
            for q in 0..a.n { if a.acc[q] { ae[q].push(a.start); } }
            ae.push(vec![a.start]);
            asy.push(vec![]);
            let mut acc = a.acc.clone();
            acc.push(true);
            E { n: a.n + 1, eps: ae, sym: asy, start: a.n, acc }
        }
        fn plus(a: &E) -> E { E::concat(a, &E::star(a)) }
        fn opt(a: &E) -> E { E::union(a, &E::empty()) }

        fn from_dfa(d: &Dfa) -> E {
            let mut sym = vec![Vec::new(); d.nstates];
            for q in 0..d.nstates {
                for s in 0..d.alpha { sym[q].push((s, d.t(q, s))); }
            }
            E { n: d.nstates, eps: vec![vec![]; d.nstates], sym, start: 0, acc: d.accept.clone() }
        }

        fn to_dfa(&self, k: usize, ntracks: usize) -> Dfa {
            let alpha = k.pow(ntracks as u32);
            let close = |set: &mut Vec<usize>, eps: &Vec<Vec<usize>>| {
                let mut i = 0;
                while i < set.len() {
                    let q = set[i];
                    for &t in &eps[q] { if !set.contains(&t) { set.push(t); } }
                    i += 1;
                }
                set.sort();
            };
            let mut s0 = vec![self.start];
            close(&mut s0, &self.eps);
            let mut index: HashMap<Vec<usize>, usize> = HashMap::new();
            let mut order: Vec<Vec<usize>> = Vec::new();
            index.insert(s0.clone(), 0);
            order.push(s0);
            let mut trans: Vec<State> = Vec::new();
            let mut i = 0;
            while i < order.len() {
                let cur = order[i].clone();
                for s in 0..alpha {
                    let mut nxt: Vec<usize> = Vec::new();
                    for &q in &cur {
                        for &(sy, t) in &self.sym[q] { if sy == s && !nxt.contains(&t) { nxt.push(t); } }
                    }
                    close(&mut nxt, &self.eps);
                    let id = *index.entry(nxt.clone()).or_insert_with(|| { order.push(nxt); order.len() - 1 });
                    trans.push(id as State);
                }
                i += 1;
            }
            let accept: Vec<bool> = order.iter().map(|s| s.iter().any(|&q| self.acc[q])).collect();
            let vars: Vec<String> = (0..ntracks).map(tname).collect();
            Dfa::new(k, vars, order.len(), trans, accept).minimize()
        }
    }

    struct P { b: Vec<char>, i: usize, k: usize, r: usize }

    pub fn compile(src: &str, k: usize, ntracks: usize) -> Result<Dfa, String> {
        let cleaned: String = src.chars().filter(|c| !c.is_whitespace() || true).collect();
        let mut p = P { b: cleaned.chars().collect(), i: 0, k, r: ntracks };
        let e = p.union()?;
        if p.i != p.b.len() { return Err(format!("unexpected {:?} in regular expression", p.b[p.i])); }
        Ok(e.to_dfa(k, ntracks))
    }

    impl P {
        fn skip_ws(&mut self) { while self.i < self.b.len() && self.b[self.i].is_whitespace() { self.i += 1; } }
        fn peek(&mut self) -> Option<char> { self.skip_ws(); self.b.get(self.i).cloned() }
        fn union(&mut self) -> Result<E, String> {
            let mut a = self.inter()?;
            while self.peek() == Some('|') { self.i += 1; a = E::union(&a, &self.inter()?); }
            Ok(a)
        }
        fn inter(&mut self) -> Result<E, String> {
            let mut a = self.concat()?;
            while self.peek() == Some('&') {
                self.i += 1;
                let b = self.concat()?;
                let da = a.to_dfa(self.k, self.r);
                let db = b.to_dfa(self.k, self.r);
                a = E::from_dfa(&da.and(&db));
            }
            Ok(a)
        }
        fn concat(&mut self) -> Result<E, String> {
            let mut acc: Option<E> = None;
            loop {
                match self.peek() {
                    None | Some('|') | Some('&') | Some(')') => break,
                    _ => {}
                }
                let r = self.repeat()?;
                acc = Some(match acc { None => r, Some(a) => E::concat(&a, &r) });
            }
            Ok(acc.unwrap_or_else(E::empty))
        }
        fn repeat(&mut self) -> Result<E, String> {
            let mut a = self.atom()?;
            loop {
                match self.peek() {
                    Some('*') => { self.i += 1; a = E::star(&a); }
                    Some('+') => { self.i += 1; a = E::plus(&a); }
                    Some('?') => { self.i += 1; a = E::opt(&a); }
                    _ => break,
                }
            }
            Ok(a)
        }
        fn atom(&mut self) -> Result<E, String> {
            let alpha = self.k.pow(self.r as u32);
            match self.peek() {
                Some('~') => {
                    self.i += 1;
                    let a = self.atom()?;
                    let d = a.to_dfa(self.k, self.r).complement();
                    Ok(E::from_dfa(&d))
                }
                Some('(') => {
                    self.i += 1;
                    let a = self.union()?;
                    if self.peek() != Some(')') { return Err("expected ')' in regular expression".into()); }
                    self.i += 1;
                    Ok(a)
                }
                Some('.') => { self.i += 1; Ok(E::lit(&(0..alpha).collect::<Vec<_>>())) }
                Some('[') => {
                    self.i += 1;
                    let st = self.i;
                    while self.i < self.b.len() && self.b[self.i] != ']' { self.i += 1; }
                    if self.i >= self.b.len() { return Err("unterminated [ in regular expression".into()); }
                    let inner: String = self.b[st..self.i].iter().collect();
                    self.i += 1;
                    let vals: Vec<i64> = inner.split(',').map(|t| t.trim().parse::<i64>()
                        .map_err(|_| format!("bad vector element {:?}", t)))
                        .collect::<Result<_, _>>()?;
                    if vals.len() != self.r {
                        return Err(format!("vector {:?} has {} elements, expected {}", inner, vals.len(), self.r));
                    }
                    let mut sym = 0usize;
                    let mut mult = 1usize;
                    for v in &vals {
                        if *v < 0 || *v as usize >= self.k {
                            return Err(format!("letter {} is outside the digit alphabet 0..{}", v, self.k - 1));
                        }
                        sym += *v as usize * mult;
                        mult *= self.k;
                    }
                    Ok(E::lit(&[sym]))
                }
                Some(c) if c.is_ascii_digit() => {
                    self.i += 1;
                    if self.r != 1 {
                        return Err("a multi-track regular expression must use [a,b] vectors".into());
                    }
                    let d = c as usize - '0' as usize;
                    if d >= self.k { return Ok(E::none()); }
                    Ok(E::lit(&[d]))
                }
                Some(c) => Err(format!("unexpected {:?} in regular expression", c)),
                None => Ok(E::empty()),
            }
        }
    }
}
