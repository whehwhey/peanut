//! First-order logic over <N, +, <, 0, V_k> extended by an automatic sequence T.
//! Parsing, and compilation of formulas into DFAs (this compilation *is* the proof).

use crate::base;
use crate::dfa::Dfa;
use crate::dfao::Dfao;
use std::collections::BTreeMap;

fn dbg_on() -> bool { std::env::var("AM_DEBUG").is_ok() }
fn trace(tag: &str, d: &Dfa) -> Dfa {
    if dbg_on() { eprintln!("  [{}] vars=[{}] states={}", tag, d.vars.join(","), d.nstates); }
    d.clone()
}

// ---------------------------------------------------------------- tokenizer

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Id(String),
    Num(u64),
    Sym(String),
}

/// Tokenize formula source into identifiers, numbers, and symbols (longest-match
/// on the symbol table; whitespace-separated otherwise).
pub fn lex(s: &str) -> Result<Vec<Tok>, String> {
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i];
        if c.is_whitespace() { i += 1; continue; }
        if c.is_ascii_alphabetic() || c == '_' {
            let st = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == '_') { i += 1; }
            out.push(Tok::Id(b[st..i].iter().collect()));
            continue;
        }
        if c.is_ascii_digit() {
            let st = i;
            while i < b.len() && b[i].is_ascii_digit() { i += 1; }
            let t: String = b[st..i].iter().collect();
            out.push(Tok::Num(t.parse().map_err(|_| "bad number")?));
            continue;
        }
        // multi-char symbols first
        for cand in ["<=>", "=>", "<=", ">=", "!=", "~=", "&", "|", "~", "!", "(", ")", "[", "]", ",", ".", "+", "-", "*", "=", "<", ">", "$"] {
            if s[i..].starts_with(cand) {
                out.push(Tok::Sym(cand.to_string()));
                i += cand.len();
                break;
            }
        }
        // safety: if nothing matched we would loop forever
        if out.last().map(|t| matches!(t, Tok::Sym(_))) != Some(true) && i < b.len() && !b[i].is_whitespace() {
            let last_ok = out.len();
            let _ = last_ok;
            return Err(format!("unexpected character {:?} at {}", c, i));
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------- AST

#[derive(Clone, Debug)]
pub struct Lin {
    pub c: i64,
    pub coef: BTreeMap<String, i64>,
}

impl Lin {
    /// The linear form `0`.
    pub fn zero() -> Lin { Lin { c: 0, coef: BTreeMap::new() } }
    /// The linear form for a constant `n`.
    pub fn num(n: i64) -> Lin { Lin { c: n, coef: BTreeMap::new() } }
    /// The linear form for a bare variable `v` (coefficient 1).
    pub fn var(v: &str) -> Lin {
        let mut m = BTreeMap::new();
        m.insert(v.to_string(), 1i64);
        Lin { c: 0, coef: m }
    }
    /// `self + sign * o`; `sign = -1` gives subtraction. Zero coefficients are pruned.
    pub fn add(&self, o: &Lin, sign: i64) -> Lin {
        let mut r = self.clone();
        r.c += sign * o.c;
        for (k, v) in &o.coef {
            let e = r.coef.entry(k.clone()).or_insert(0);
            *e += sign * v;
            if *e == 0 { r.coef.remove(k); }
        }
        r
    }
    /// Multiply every coefficient and the constant by `m`.
    pub fn scale(&self, m: i64) -> Lin {
        Lin { c: self.c * m, coef: self.coef.iter().map(|(k, v)| (k.clone(), v * m)).collect() }
    }
    /// Split into (positive part, negated negative part), both with nonneg coefficients.
    pub fn split(&self) -> (Lin, Lin) {
        let mut p = Lin::zero();
        let mut n = Lin::zero();
        if self.c >= 0 { p.c = self.c } else { n.c = -self.c }
        for (k, v) in &self.coef {
            if *v > 0 { p.coef.insert(k.clone(), *v); } else { n.coef.insert(k.clone(), -*v); }
        }
        (p, n)
    }
    /// If this form is exactly `1*v` (no constant, no other terms), return `v`.
    pub fn is_plain_var(&self) -> Option<String> {
        if self.c == 0 && self.coef.len() == 1 {
            let (k, v) = self.coef.iter().next().unwrap();
            if *v == 1 { return Some(k.clone()); }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub enum Rel { Eq, Ne, Lt, Le, Gt, Ge }

#[derive(Clone, Debug)]
pub enum Ast {
    Cmp(Lin, Rel, Lin),
    SeqLetter(Lin, Rel, u8),          // T[t] REL a   (Eq/Ne only)
    SeqSeq(Lin, Rel, Lin),            // T[t1] REL T[t2] (Eq/Ne only)
    Not(Box<Ast>),
    And(Box<Ast>, Box<Ast>),
    Or(Box<Ast>, Box<Ast>),
    Imp(Box<Ast>, Box<Ast>),
    Iff(Box<Ast>, Box<Ast>),
    Forall(Vec<String>, Box<Ast>),
    Exists(Vec<String>, Box<Ast>),
    Bool(bool),
    Call(String, Vec<Lin>),
    IsPow(Lin),
}

// ---------------------------------------------------------------- parser

/// Recursive-descent parser for the formula grammar (precedence low to high:
/// `<=>`, `=>`, `|`, `&`, quantifiers/atoms), tracking the sequence's bound name
/// so `T[...]` references resolve.
pub struct Parser {
    t: Vec<Tok>,
    i: usize,
    seqname: String,
}

impl Parser {
    /// Create a parser over a token stream, given the automatic sequence's name
    /// (so occurrences of it in formula source are recognized as `T[...]`).
    pub fn new(t: Vec<Tok>, seqname: &str) -> Parser { Parser { t, i: 0, seqname: seqname.to_string() } }

    fn peek(&self) -> Option<&Tok> { self.t.get(self.i) }
    fn eat_sym(&mut self, s: &str) -> bool {
        if let Some(Tok::Sym(x)) = self.peek() { if x == s { self.i += 1; return true; } }
        false
    }
    fn expect_sym(&mut self, s: &str) -> Result<(), String> {
        if self.eat_sym(s) { Ok(()) } else { Err(format!("expected {:?} at token {} ({:?})", s, self.i, self.peek())) }
    }
    fn eat_id(&mut self) -> Option<String> {
        if let Some(Tok::Id(x)) = self.peek() { let r = x.clone(); self.i += 1; return Some(r); }
        None
    }

    /// Parse a complete formula; errors if input remains after the top-level term.
    pub fn parse(&mut self) -> Result<Ast, String> {
        let a = self.p_iff()?;
        if self.i != self.t.len() { return Err(format!("trailing input at token {} ({:?})", self.i, self.peek())); }
        Ok(a)
    }

    fn p_iff(&mut self) -> Result<Ast, String> {
        let mut a = self.p_imp()?;
        while self.eat_sym("<=>") { a = Ast::Iff(Box::new(a), Box::new(self.p_imp()?)); }
        Ok(a)
    }
    fn p_imp(&mut self) -> Result<Ast, String> {
        let a = self.p_or()?;
        if self.eat_sym("=>") { return Ok(Ast::Imp(Box::new(a), Box::new(self.p_imp()?))); }
        Ok(a)
    }
    fn p_or(&mut self) -> Result<Ast, String> {
        let mut a = self.p_and()?;
        while self.eat_sym("|") { a = Ast::Or(Box::new(a), Box::new(self.p_and()?)); }
        Ok(a)
    }
    fn p_and(&mut self) -> Result<Ast, String> {
        let mut a = self.p_not()?;
        while self.eat_sym("&") { a = Ast::And(Box::new(a), Box::new(self.p_not()?)); }
        Ok(a)
    }
    fn p_not(&mut self) -> Result<Ast, String> {
        if self.eat_sym("~") || self.eat_sym("!") { return Ok(Ast::Not(Box::new(self.p_not()?))); }
        self.p_primary()
    }

    fn p_primary(&mut self) -> Result<Ast, String> {
        // "(" is ambiguous: it may open a sub-formula OR a parenthesised term such
        // as "(i) < (2*j+1)".  Try the formula reading; if it does not consume a
        // balanced group followed by something that is not a relation symbol, back
        // off and let p_atom re-read the whole thing as a comparison.
        if let Some(Tok::Sym(s0)) = self.peek() {
            if s0 == "(" {
                let save = self.i;
                self.i += 1;
                let ok = (|| -> Result<Ast, String> {
                    let a = self.p_iff()?;
                    self.expect_sym(")")?;
                    Ok(a)
                })();
                match ok {
                    Ok(a) => {
                        let is_rel = matches!(self.peek(), Some(Tok::Sym(r))
                            if ["=", "!=", "~=", "<", "<=", ">", ">="].contains(&r.as_str()));
                        if !is_rel { return Ok(a); }
                        self.i = save;
                    }
                    Err(_) => { self.i = save; }
                }
            }
        }
        // quantifiers
        if let Some(Tok::Id(w)) = self.peek().cloned() {
            let lower = w.as_str();
            let isq = lower == "A" || lower == "E" || lower == "forall" || lower == "exists";
            if isq {
                let save = self.i;
                self.i += 1;
                let mut vs = Vec::new();
                while let Some(v) = self.eat_id() {
                    vs.push(v);
                    if !self.eat_sym(",") { break; }
                }
                if vs.is_empty() { self.i = save; } else {
                    // optional '.' separator
                    self.eat_sym(".");
                    let body = self.p_iff()?;
                    return Ok(if lower == "A" || lower == "forall" {
                        Ast::Forall(vs, Box::new(body))
                    } else {
                        Ast::Exists(vs, Box::new(body))
                    });
                }
            }
        }
        self.p_atom()
    }

    fn at_seq(&self) -> bool {
        if let Some(Tok::Id(w)) = self.peek() {
            if *w == self.seqname {
                return matches!(self.t.get(self.i + 1), Some(Tok::Sym(s)) if s == "[");
            }
        }
        false
    }

    fn p_seqindex(&mut self) -> Result<Lin, String> {
        self.i += 1; // name
        self.expect_sym("[")?;
        let t = self.p_lin()?;
        self.expect_sym("]")?;
        Ok(t)
    }

    fn p_rel(&mut self) -> Result<Rel, String> {
        for (s, r) in [("<=", Rel::Le), (">=", Rel::Ge), ("!=", Rel::Ne), ("~=", Rel::Ne), ("=", Rel::Eq), ("<", Rel::Lt), (">", Rel::Gt)] {
            if self.eat_sym(s) { return Ok(r); }
        }
        Err(format!("expected relation at token {} ({:?})", self.i, self.peek()))
    }

    fn p_atom(&mut self) -> Result<Ast, String> {
        if let Some(Tok::Id(w)) = self.peek().cloned() {
            if w == "pow" && matches!(self.t.get(self.i+1), Some(Tok::Sym(s)) if s=="(") {
                self.i += 2;
                let t = self.p_lin()?;
                self.expect_sym(")")?;
                return Ok(Ast::IsPow(t));
            }
        }
        if self.eat_sym("$") {
            let name = self.eat_id().ok_or("expected name after $")?;
            let mut args = Vec::new();
            if self.eat_sym("(") {
                loop {
                    args.push(self.p_lin()?);
                    if !self.eat_sym(",") { break; }
                }
                self.expect_sym(")")?;
            }
            return Ok(Ast::Call(name, args));
        }
        if let Some(Tok::Id(w)) = self.peek() {
            if w == "true" { self.i += 1; return Ok(Ast::Bool(true)); }
            if w == "false" { self.i += 1; return Ok(Ast::Bool(false)); }
        }
        if self.at_seq() {
            let t1 = self.p_seqindex()?;
            let r = self.p_rel()?;
            if !matches!(r, Rel::Eq | Rel::Ne) { return Err("only = and != allowed on sequence values".into()); }
            if self.at_seq() {
                let t2 = self.p_seqindex()?;
                return Ok(Ast::SeqSeq(t1, r, t2));
            }
            if let Some(Tok::Num(n)) = self.peek().cloned() {
                self.i += 1;
                return Ok(Ast::SeqLetter(t1, r, n as u8));
            }
            return Err("expected a letter or T[..] on the right".into());
        }
        let l = self.p_lin()?;
        let r = self.p_rel()?;
        if self.at_seq() { return Err("sequence term on right of arithmetic comparison".into()); }
        let rr = self.p_lin()?;
        Ok(Ast::Cmp(l, r, rr))
    }

    fn p_lin(&mut self) -> Result<Lin, String> {
        let mut sign = 1i64;
        if self.eat_sym("-") { sign = -1; } else { self.eat_sym("+"); }
        let mut acc = self.p_prod()?.scale(sign);
        loop {
            if self.eat_sym("+") { acc = acc.add(&self.p_prod()?, 1); }
            else if self.eat_sym("-") { acc = acc.add(&self.p_prod()?, -1); }
            else { break; }
        }
        Ok(acc)
    }

    fn p_prod(&mut self) -> Result<Lin, String> {
        match self.peek().cloned() {
            Some(Tok::Num(n)) => {
                self.i += 1;
                if self.eat_sym("*") {
                    let v = self.eat_id().ok_or("expected variable after *")?;
                    Ok(Lin::var(&v).scale(n as i64))
                } else { Ok(Lin::num(n as i64)) }
            }
            Some(Tok::Id(v)) => {
                self.i += 1;
                if self.eat_sym("*") {
                    if let Some(Tok::Num(n)) = self.peek().cloned() { self.i += 1; return Ok(Lin::var(&v).scale(n as i64)); }
                    return Err("expected number after *".into());
                }
                Ok(Lin::var(&v))
            }
            Some(Tok::Sym(s)) if s == "(" => {
                self.i += 1;
                let t = self.p_lin()?;
                self.expect_sym(")")?;
                Ok(t)
            }
            other => Err(format!("expected term, found {:?}", other)),
        }
    }
}

// ---------------------------------------------------------------- compiler

/// Named `let`-bound predicates: formula name -> (parameter names, compiled DFA).
pub type Defs = std::collections::HashMap<String, (Vec<String>, Dfa)>;

/// Compiles a parsed [`Ast`] into a [`Dfa`] whose accepted words are exactly the
/// satisfying assignments. Each syntactic construct maps to an automaton
/// operation (`&`/`|`/`!` -> product/complement, quantifiers -> [`Dfa::exists`]/
/// [`Dfa::forall`], arithmetic atoms -> [`crate::base`] constructors); the
/// resulting automaton *is* the decision procedure, not merely a description of one.
pub struct Compiler<'a> {
    pub k: usize,
    pub seq: &'a Dfao,
    pub defs: &'a Defs,
    fresh: usize,
}

impl<'a> Compiler<'a> {
    /// Start a fresh compiler over base `k`, the sequence `seq`, and the `let`
    /// environment `defs` (empty fresh-variable counter).
    pub fn new(k: usize, seq: &'a Dfao, defs: &'a Defs) -> Compiler<'a> { Compiler { k, seq, defs, fresh: 0 } }

    fn newvar(&mut self) -> String { self.fresh += 1; format!("${}", self.fresh) }

    /// z = x + y, safe even when x and y name the same variable.
    fn add_auto(&mut self, x: &str, y: &str, z: &str) -> Dfa {
        if x != y { return base::adder(self.k, x, y, z); }
        let u = self.newvar();
        base::equal(self.k, x, &u)
            .and(&base::adder(self.k, x, &u, z))
            .exists(&u)
    }

    /// Automaton asserting `out = value of the nonneg linear form t`, plus the name
    /// of `out`.  Returns (automaton, varname, is_fresh).
    ///
    /// Kept as flat as possible: every extra intermediate variable multiplies the
    /// working alphabet by k, so we fold the summands directly instead of starting
    /// from a constant-zero automaton.
    fn lin_auto(&mut self, t: &Lin) -> Result<(Dfa, String, bool), String> {
        for v in t.coef.values() { assert!(*v >= 0); }
        assert!(t.c >= 0);
        if let Some(v) = t.is_plain_var() {
            return Ok((Dfa::constant(self.k, vec![v.clone()], true), v, false));
        }
        // summands, as (name, automaton constraining it, is_fresh)
        let mut parts: Vec<(String, Option<Dfa>)> = Vec::new();
        for (v, m) in &t.coef {
            for _ in 0..*m { parts.push((v.clone(), None)); }
        }
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
        let mut cur_fresh = acc.nstates > 1 || acc.vars.len() != 1 || false;
        // `cur_fresh` really means "cur is an internal name we may project away"
        cur_fresh = !t.coef.contains_key(&cur);
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

    fn cmp_auto(&mut self, a: &Lin, r: &Rel, b: &Lin) -> Result<Dfa, String> {
        // Move everything so both sides have nonneg coefficients.
        let d = a.add(b, -1);
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

    /// Compile an AST node to a DFA, recursively compiling subterms first.
    pub fn compile(&mut self, a: &Ast) -> Result<Dfa, String> {
        Ok(match a {
            Ast::Bool(b) => Dfa::constant(self.k, vec![], *b),
            Ast::Cmp(l, r, rr) => self.cmp_auto(l, r, rr)?,
            Ast::SeqLetter(t, r, a) => {
                let p = { let (p, n) = t.split(); if !(n.c == 0 && n.coef.is_empty()) { return Err("negative index".into()); } p };
                let (at, v, fresh) = self.lin_auto(&p)?;
                let mut d = at.and(&self.seq.pred_letter(&v, *a));
                if matches!(r, Rel::Ne) {
                    // recompute: need ~ inside the scope of the index variable
                    let mut e = at.and(&self.seq.pred_letter(&v, *a).complement());
                    if fresh { e = e.exists(&v); }
                    return Ok(e);
                }
                if fresh { d = d.exists(&v); }
                d
            }
            Ast::SeqSeq(t1, r, t2) => {
                let p1 = { let (p, n) = t1.split(); if !(n.c == 0 && n.coef.is_empty()) { return Err("negative index".into()); } p };
                let p2 = { let (p, n) = t2.split(); if !(n.c == 0 && n.coef.is_empty()) { return Err("negative index".into()); } p };
                let (a1, v1, f1) = self.lin_auto(&p1)?;
                let (a2, v2, f2) = self.lin_auto(&p2)?;
                let core = if v1 == v2 {
                    Dfa::constant(self.k, vec![v1.clone()], matches!(r, Rel::Eq))
                } else {
                    let e = self.seq.pred_eq(&v1, &v2);
                    if matches!(r, Rel::Ne) { e.complement() } else { e }
                };
                let mut d = a1.and(&a2).and(&core);
                if f1 { d = d.exists(&v1); }
                if f2 && v2 != v1 { d = d.exists(&v2); }
                d
            }
            Ast::IsPow(t) => {
                let (p, n) = t.split();
                if !(n.c == 0 && n.coef.is_empty()) { return Err("negative argument to pow".into()); }
                let (aut, v, fresh) = self.lin_auto(&p)?;
                let mut d = aut.and(&base::power_of_k(self.k, &v));
                if fresh { d = d.exists(&v); }
                d
            }
            Ast::Call(name, args) => {
                let (params, body) = self.defs.get(name).ok_or_else(|| format!("undefined predicate ${}", name))?;
                if params.len() != args.len() {
                    return Err(format!("${} expects {} arguments, got {}", name, params.len(), args.len()));
                }
                // Bind each parameter to a fresh variable, constrain it to the argument
                // term, conjoin, then project the fresh bindings away.
                let mut binders: Vec<(String, Dfa, bool)> = Vec::new();
                let mut fresh_names = Vec::new();
                for a in args.iter() {
                    let (p, n) = a.split();
                    if !(n.c == 0 && n.coef.is_empty()) { return Err("negative argument".into()); }
                    let w = self.newvar();
                    let (aut, v, was_fresh) = self.lin_auto(&p)?;
                    let eqv = base::equal(self.k, &w, &v);
                    binders.push((v, aut.and(&eqv), was_fresh));
                    fresh_names.push(w);
                }
                let mut ren = std::collections::HashMap::new();
                for (p, w) in params.iter().zip(fresh_names.iter()) { ren.insert(p.clone(), w.clone()); }
                let mut acc = body.rename(&|v| ren.get(v).cloned().unwrap_or_else(|| v.to_string()));
                for (v, aut, was_fresh) in binders.iter() {
                    acc = acc.and(aut);
                    if *was_fresh { acc = acc.exists(v); }
                }
                for w in fresh_names.iter() { acc = acc.exists(w); }
                acc
            }
            Ast::Not(x) => self.compile(x)?.complement(),
            Ast::And(x, y) => { let a = self.compile(x)?; let b = self.compile(y)?; a.and(&b) }
            Ast::Or(x, y) => { let a = self.compile(x)?; let b = self.compile(y)?; a.or(&b) }
            Ast::Imp(x, y) => { let a = self.compile(x)?; let b = self.compile(y)?; a.implies(&b) }
            Ast::Iff(x, y) => { let a = self.compile(x)?; let b = self.compile(y)?; a.iff(&b) }
            Ast::Exists(vs, x) => {
                let mut d = self.compile(x)?;
                trace("body-of-E", &d);
                for v in vs.iter().rev() { d = d.exists(v); trace(&format!("after E {}", v), &d); }
                d
            }
            Ast::Forall(vs, x) => {
                let mut d = self.compile(x)?;
                trace("body-of-A", &d);
                for v in vs.iter().rev() { d = d.forall(v); trace(&format!("after A {}", v), &d); }
                d
            }
        })
    }
}

/// Convenience: lex, parse, and compile a formula string in one call.
pub fn compile_str(k: usize, seq: &Dfao, defs: &Defs, src: &str) -> Result<Dfa, String> {
    let toks = lex(src)?;
    let ast = Parser::new(toks, "T").parse()?;
    Compiler::new(k, seq, defs).compile(&ast)
}
