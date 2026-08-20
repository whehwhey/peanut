//! Peanut engine entry point: a stdin/stdout REPL over the command language
//! documented in `docs/GUARD.md` (`mode`, `def`, `let`, `?`, `witness`,
//! `learnfe`, `enum`, `finite`, `mem`, `quit`). Reads one command per line,
//! dispatches to [`base`]/[`logic`]/[`learn`] to build or query automata, and
//! prints one `OK ...` / `ERR ...` (or command-specific) reply per line so a
//! driver such as `explore/engine.py` can pipe commands in and parse replies
//! out without any framing beyond newlines. Always invoke this binary through
//! that Python wrapper, never with unguarded parallel instances: several
//! engines racing on the same machine can exhaust memory and lock it up.
mod antichain;
mod autolearn;
mod clock;
mod compat;
mod det_par;
mod dfa;
mod numsys;
mod base;
mod dfao;
mod export;
mod learn;
mod logic;
mod membudget;
mod negbase;
mod ostrowski;
mod picture;
mod progress;
mod simsub;
mod symbolic;
mod transducer;

#[global_allocator]
static GLOBAL: membudget::Budgeted = membudget::Budgeted;

use dfao::Dfao;
use std::io::Write;
use std::sync::Arc;
use crate::clock::Instant;

/// `dfao NAME D o0:t00,t01,.. o1:.. ..`  or  `dfao NAME @path` (Walnut word-automaton
/// file).  Builds an explicit DFA-with-output over the active digit alphabet: the way
/// a sequence enters the engine when it is not the fixed point of a k-uniform morphism
/// (every Fibonacci-, Tribonacci- or Pell-automatic word).  `-` as a transition target
/// means "dead" (an implicit extra sink state); valid representations never reach it.
fn parse_dfao(rest: &str) -> Result<Dfao, String> {
    const USAGE: &str = "usage: dfao NAME D o0:t0,t1,.. ..  |  dfao NAME @file";
    let rest = rest.trim();
    let cut = rest.find(char::is_whitespace).ok_or(USAGE)?;
    let (name, tail) = (&rest[..cut], rest[cut..].trim());
    // `@path` takes the whole rest of the line, so file names may contain spaces
    if let Some(path) = tail.strip_prefix('@') {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?;
        let expect = numsys::active().map(|n| n.digits);
        let p = numsys::parse_walnut(&text, expect)?;
        if let Some(d) = expect { if p.digits != d {
            return Err(format!("automaton has {} digits, numeration system has {}", p.digits, d)); } }
        let (n, trans, out) = p.to_dfao_tables()?;
        return Dfao::from_tables(p.digits, n, trans, out, name);
    }
    let toks: Vec<&str> = tail.split_whitespace().collect();
    if toks.is_empty() { return Err(USAGE.into()); }
    let d: usize = toks[0].parse().map_err(|_| "bad digit-alphabet size")?;
    if d < 2 { return Err("digit alphabet must have at least 2 letters".into()); }
    if let Some(ns) = numsys::active() {
        if ns.digits != d {
            return Err(format!("numeration system {} has {} digits, got {}", ns.name, ns.digits, d));
        }
    }
    let rows = &toks[1..];
    if rows.is_empty() { return Err("no states given".into()); }
    let n = rows.len();
    let dead = n as u32;                 // implicit sink, appended below
    let mut trans = vec![dead; (n + 1) * d];
    let mut out = vec![0u8; n + 1];
    for (q, row) in rows.iter().enumerate() {
        let (o, ts) = row.split_once(':').ok_or_else(|| format!("state {}: expected out:t0,t1,..", q))?;
        out[q] = o.parse::<u8>().map_err(|_| format!("state {}: bad output {:?}", q, o))?;
        let parts: Vec<&str> = ts.split(',').collect();
        if parts.len() != d { return Err(format!("state {}: {} transitions, expected {}", q, parts.len(), d)); }
        for (dg, t) in parts.iter().enumerate() {
            if *t == "-" { continue; }
            let tt: usize = t.parse().map_err(|_| format!("state {}: bad target {:?}", q, t))?;
            if tt >= n { return Err(format!("state {}: target {} >= {} states", q, tt, n)); }
            trans[q * d + dg] = tt as u32;
        }
    }
    for dg in 0..d { trans[n * d + dg] = dead; }
    Dfao::from_tables(d, n + 1, trans, out, name)
}

fn parse_def(parts: &[&str]) -> Result<Dfao, String> {
    // def <name> <k> <m> <start> <w0> .. <w_{m-1}> <coding>
    if parts.len() < 5 { return Err("usage: def name k m start w0..w_{m-1} coding".into()); }
    let name = parts[0];
    let k: usize = parts[1].parse().map_err(|_| "bad k")?;
    let m: usize = parts[2].parse().map_err(|_| "bad m")?;
    let start: usize = parts[3].parse().map_err(|_| "bad start")?;
    if parts.len() != 4 + m + 1 { return Err(format!("expected {} words + coding, got {}", m, parts.len() - 4)); }
    let mut sigma = Vec::new();
    for a in 0..m {
        let w: Vec<u8> = parts[4 + a].chars().map(|c| c as u8 - b'0').collect();
        if w.len() != k { return Err(format!("word {} has length {}, expected {}", a, w.len(), k)); }
        if w.iter().any(|&x| x as usize >= m) { return Err(format!("word {} has a letter >= m", a)); }
        sigma.push(w);
    }
    let coding: Vec<u8> = parts[4 + m].chars().map(|c| c as u8 - b'0').collect();
    if coding.len() != m { return Err("coding length != m".into()); }
    if sigma[start][0] as usize != start { return Err("not prolongable at start letter".into()); }
    Dfao::from_morphism(k, m, &sigma, &coding, start, name)
}

/// Where generated numeration-system files go: the first directory on the
/// numeration search path that exists, else `engine/numeration`.
fn numsys_dir() -> std::path::PathBuf {
    numsys::search_dirs().into_iter().find(|d| d.is_dir())
        .unwrap_or_else(|| "engine/numeration".into())
}

/// Drive the command REPL over an arbitrary line source and output sink.
///
/// This is the single implementation shared by the native binary (stdin lines ->
/// stdout) and the wasm playground (an in-memory script string -> an in-memory
/// buffer). The command dispatch below is byte-for-byte the CLI's; the only
/// difference between the two front ends is where the lines come from and where
/// the replies go. `p!` stands in for the old `println!` and writes to `out`.
pub fn run_loop(lines: impl Iterator<Item = String>, out: &mut impl Write) {
    macro_rules! p { ($($a:tt)*) => {{ let _ = writeln!(out, $($a)*); }} }
    let mut cur: Option<Dfao> = None;
    let mut defs: logic::Defs = logic::Defs::new();
    let mut trs: std::collections::HashMap<String, transducer::Transducer> = Default::default();
    let mut wal = compat::Compat::new();
    for line in lines {
        let line = line.trim();
        // --- Walnut-compatibility mode (docs/WALNUT-COMPAT.md) ---------------
        // `walnut` toggles it; a line carrying a `?msd_`/`?lsd_` number-system
        // prefix turns it on by itself, so a Walnut script runs unchanged.
        if line == "walnut" || line.starts_with("walnut ") {
            let arg = line[6..].trim();
            wal.on = match arg { "on" => true, "off" => false, _ => !wal.on };
            if !wal.on { if let Some(m) = wal.flush() { p!("{}", m); } }
            p!("OK walnut {} root={}", if wal.on { "on" } else { "off" }, wal.root.display());
            let _ = out.flush();
            continue;
        }
        if !wal.on && (line.contains("?msd_") || line.contains("?lsd_")) {
            // auto-detect, but only for something that really looks like a Walnut
            // command -- a native command must never be swallowed by accident
            let head = line.split_whitespace().next().unwrap_or("");
            if compat::IS_WALNUT_CMD.contains(&head) { wal.on = true; }
        }
        if wal.on {
            if line == "quit" || line == "exit" || line == "quit;" || line == "exit;" {
                if let Some(m) = wal.flush() { p!("{}", m); }
                break;
            }
            if let Some(reply) = wal.feed(line) {
                if reply == "WQUIT" { break; }
                if !reply.is_empty() { p!("{}", reply); }
                let _ = out.flush();
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') { continue; }
        let (cmd, rest) = match line.find(' ') {
            Some(i) => (&line[..i], line[i + 1..].trim()),
            None => (line, ""),
        };
        if progress::on() {
            match cmd {
                "?" | "eval" | "let" | "witness" | "enum" | "dfa" | "finite" | "learnfe" | "learn" =>
                    progress::phase("compile", line),
                _ => {}
            }
        }
        match cmd {
            "quit" | "exit" => break,
            "mem" => { progress::mem();
                       p!("OK mem live={}MB peak={}MB", membudget::live_mb(), membudget::peak_mb()) }
            "mode" => {
                let lsd = rest.trim() == "lsd";
                dfa::set_lsd(lsd);
                p!("OK mode {}", if lsd { "lsd" } else { "msd" });
            }
            "numsys" => {
                // numsys NAME | numsys off  -- switch the session's numeration system.
                let arg = rest.split_whitespace().next().unwrap_or("");
                if arg.is_empty() {
                    match numsys::active() {
                        None => p!("OK numsys base-k (built in)"),
                        Some(n) => p!("OK numsys {} digits={} valid={} add={} lt={}",
                                            n.name, n.digits, n.valid_msd.nstates, n.add_msd.nstates,
                                            if n.lt_loaded { "loaded" } else { "lexicographic" }),
                    }
                } else if arg == "off" || arg == "base" || arg == "none" {
                    numsys::set_active(None);
                    cur = None; defs.clear();
                    p!("OK numsys base-k (built in)");
                } else {
                    match numsys::load(arg) {
                        Ok(n) => {
                            // In a negative base the place values are (-k)^l, not the
                            // word counts the rank machinery reports.
                            let w: Vec<String> = match n.neg_base {
                                Some(b) => (0..8u32).map(|l| (-(b as i128)).pow(l).to_string()).collect(),
                                None => (0..8).map(|l| n.weight(l).to_string()).collect(),
                            };
                            p!("OK numsys {} digits={} valid={} add={} lt={} weights={},...",
                                     n.name, n.digits, n.valid_msd.nstates, n.add_msd.nstates,
                                     if n.lt_loaded { "loaded" } else { "lexicographic" }, w.join(","));
                            numsys::set_active(Some(Arc::new(n)));
                            cur = None; defs.clear();
                        }
                        Err(e) => p!("ERR numsys {}", e),
                    }
                }
            }
            "dfao" => {
                match parse_dfao(rest) {
                    Ok(d) => {
                        p!("OK dfao {} k={} states={} lsd_states={} ns={} mode={}",
                                 d.name, d.k, d.nstates, d.lnstates,
                                 if numsys::active().is_some() { numsys::active_name() } else { "base".into() },
                                 if dfa::is_lsd() { "lsd" } else { "msd" });
                        defs.clear();
                        cur = Some(d);
                    }
                    Err(e) => p!("ERR dfao {}", e),
                }
            }
            "def" => {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                match parse_def(&parts) {
                    Ok(d) => {
                        p!("OK def {} k={} states={} lsd_states={} mode={}", d.name, d.k, d.nstates,
                                 d.lnstates, if dfa::is_lsd() { "lsd" } else { "msd" });
                        defs.clear();
                        cur = Some(d);
                    }
                    Err(e) => p!("ERR {}", e),
                }
            }
            "seq" => {
                // seq N -- the first N symbols of T as one string of digits
                let n: usize = rest.split_whitespace().next().and_then(|x| x.parse().ok()).unwrap_or(60);
                match &cur {
                    Some(d) => p!("SEQ n={} k={} {}", n, d.k,
                        d.prefix(n).iter().map(|x| x.to_string()).collect::<Vec<_>>().join("")),
                    None => p!("ERR no sequence"),
                }
            }
            "export" => {
                // export NAME -- the automaton as one JSON line.  NAME = T (the DFAO)
                // or any predicate bound by `let` / `learnfe`.
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                let name = rest.split_whitespace().next().unwrap_or("T");
                if name == "T" || name == d.name {
                    p!("EXPORT {}", export::dfao_json(d));
                } else if let Some((params, a)) = defs.get(name) {
                    p!("EXPORT {}", export::dfa_json(name, params, a));
                } else {
                    p!("ERR export: no such predicate {:?} (have: T{}{})", name,
                             if defs.is_empty() { "" } else { ", " },
                             defs.keys().cloned().collect::<Vec<_>>().join(", "));
                }
            }
            "fe_map" => {
                // fe_map i0 j0 size L -- a size x size grid of FE(i,j,L) for
                // i in [i0, i0+size), j in [j0, j0+size), computed by direct LCP walk
                // through the DFAO.  No automaton is built: this is the ground truth the
                // FE automaton is supposed to encode, and it is what the heatmap draws.
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                let p: Vec<u64> = rest.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                if p.len() < 4 { p!("ERR usage: fe_map i0 j0 size L"); continue }
                let (i0, j0, size, l) = (p[0], p[1], p[2].min(512) as usize, p[3]);
                let t0 = Instant::now();
                let hardcap: u64 = std::env::var("AM_LEARN_LCP").ok().and_then(|v| v.parse().ok())
                    .unwrap_or(1 << 22);
                let mut or = learn::Oracle::new(d, hardcap.max(l + 1));
                let mut rows: Vec<String> = Vec::with_capacity(size);
                for r in 0..size {
                    let mut row = String::with_capacity(size);
                    for c in 0..size {
                        row.push(if or.fe(i0 + r as u64, j0 + c as u64, l) { '1' } else { '0' });
                    }
                    rows.push(row);
                }
                p!("FEMAP i0={} j0={} size={} l={} ms={} rows={}",
                         i0, j0, size, l, t0.elapsed().as_millis(), rows.join(","));
            }
            "transducer" => {
                // transducer NAME @file          -- Walnut "Transducer Library" format
                // transducer NAME D q:t/o,t/o ..  -- the same machine typed inline
                let r = (|| -> Result<transducer::Transducer, String> {
                    let rest = rest.trim();
                    let cut = rest.find(char::is_whitespace)
                        .ok_or("usage: transducer NAME @file | transducer NAME D q0:t/o,.. ..")?;
                    let (name, tail) = (&rest[..cut], rest[cut..].trim());
                    if let Some(path) = tail.strip_prefix('@') {
                        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?;
                        transducer::parse(name, &text)
                    } else {
                        transducer::parse_inline(name, &tail.split_whitespace().collect::<Vec<_>>())
                    }
                })();
                match r {
                    Ok(t) => {
                        p!("OK transducer {} states={} alphabet={:?}", t.name, t.nstates, t.letters);
                        trs.insert(t.name.clone(), t);
                    }
                    Err(e) => p!("ERR transducer {}", e),
                }
            }
            "transduce" => {
                // transduce NEW TRANS SEQ -- Dekking-style transduction; the
                // result becomes the current sequence.
                let p: Vec<&str> = rest.split_whitespace().collect();
                if p.len() < 3 { p!("ERR usage: transduce NEW TRANSDUCER SEQ"); continue }
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                let src = p[2].trim_start_matches('$');
                if src != "T" && src != d.name {
                    p!("ERR transduce: {:?} is not the current sequence ({})", src, d.name);
                    continue;
                }
                let Some(t) = trs.get(p[1]) else {
                    p!("ERR transduce: no transducer {:?} (have: {})", p[1],
                             trs.keys().cloned().collect::<Vec<_>>().join(", "));
                    continue;
                };
                let t0 = Instant::now();
                match transducer::transduce(d, t, p[0]) {
                    Ok(n) => {
                        p!("OK transduce {} states={} lsd_states={} from={} via={} ms={}",
                                 n.name, n.nstates, n.lnstates, d.name, t.name, t0.elapsed().as_millis());
                        defs.clear();
                        cur = Some(n);
                    }
                    Err(e) => p!("ERR transduce {}", e),
                }
            }
            "negbase" => {
                // negbase K -- write msd_neg_K{,_addition,_less_than}.txt
                let k: u32 = rest.split_whitespace().next().and_then(|x| x.parse().ok()).unwrap_or(0);
                if k < 2 { p!("ERR usage: negbase K   (K >= 2, the system is base -K)"); continue }
                match negbase::write_files(k, &numsys_dir()) {
                    Ok(paths) => p!("OK negbase -{} wrote {}", k, paths.join(" ")),
                    Err(e) => p!("ERR negbase {}", e),
                }
            }
            "ost" => {
                // ost NAME [preperiod] [period] -- an Ostrowski numeration system
                match (|| -> Result<String, String> {
                    let name = rest.split_whitespace().next()
                        .ok_or("usage: ost NAME [preperiod] [period]")?.to_string();
                    let mut groups: Vec<Vec<u32>> = Vec::new();
                    let mut it = rest.char_indices();
                    while let Some((i, c)) = it.next() {
                        if c != '[' { continue }
                        let j = rest[i..].find(']').ok_or("unterminated [")? + i;
                        groups.push(rest[i + 1..j].split(|c: char| c == ',' || c.is_whitespace())
                            .filter(|t| !t.is_empty())
                            .map(|t| t.parse::<u32>().map_err(|_| format!("bad partial quotient {:?}", t)))
                            .collect::<Result<_, _>>()?);
                        while let Some((x, _)) = it.next() { if x >= j { break } }
                    }
                    if groups.len() != 2 { return Err("usage: ost NAME [preperiod] [period]".into()); }
                    let (paths, vs, as_, w) = ostrowski::generate(&name, &groups[0], &groups[1], &numsys_dir())?;
                    Ok(format!("OK ost {} valid={} add={} weights={} wrote {}", name, vs, as_,
                               w.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                               paths.join(" ")))
                })() {
                    Ok(m) => p!("{}", m),
                    Err(e) => p!("ERR ost {}", e),
                }
            }
            "pic" => p!("{}", picture::cmd(cur.as_ref(), &defs, rest)),
            "?" | "eval" => {
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                let t0 = Instant::now();
                dfa::peak_reset();
                match logic::compile_str(d.k, d, &defs, rest) {
                    Ok(a) => {
                        let ms = t0.elapsed().as_millis();
                        if a.vars.is_empty() {
                            p!("{} states={} peak={} ms={} :: {}",
                                if a.accepts_epsilon() { "TRUE" } else { "FALSE" }, a.nstates,
                                dfa::peak_get(), ms, rest);
                        } else {
                            let w = a.enumerate(12, 14);
                            let ws: Vec<String> = w.iter().map(|v| format!("({})", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))).collect();
                            p!("OPEN vars=[{}] states={} nonempty={} ms={} witnesses={} :: {}",
                                a.vars.join(","), a.nstates, a.is_nonempty(), ms, ws.join(" "), rest);
                        }
                    }
                    Err(e) => p!("ERR {}", e),
                }
            }
            "witness" => {
                // witness <formula>  -- one satisfying assignment (the SHORTEST accepted
                // word), or NONE.  For a closed formula this degenerates to TRUE/FALSE.
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                let t0 = Instant::now();
                dfa::peak_reset();
                match logic::compile_str(d.k, d, &defs, rest) {
                    Ok(a) => {
                        let ms = t0.elapsed().as_millis();
                        if a.vars.is_empty() {
                            p!("{} states={} ms={} :: {}",
                                if a.accepts_epsilon() { "TRUE" } else { "FALSE" }, a.nstates, ms, rest);
                        } else {
                            match a.shortest_word() {
                                None => p!("NONE vars=[{}] states={} ms={} :: {}",
                                                 a.vars.join(","), a.nstates, ms, rest),
                                Some(w) => {
                                    let v = learn::decode(a.k, a.vars.len(), &w)
                                        .unwrap_or_else(|| vec![0; a.vars.len()]);
                                    let asg: Vec<String> = a.vars.iter().zip(v.iter())
                                        .map(|(n, x)| format!("{}={}", n, x)).collect();
                                    p!("WITNESS {} states={} len={} ms={} :: {}",
                                             asg.join(" "), a.nstates, w.len(), ms, rest);
                                }
                            }
                        }
                    }
                    Err(e) => p!("ERR {}", e),
                }
            }
            "learn" => {
                // learn NAME <kind> | learn NAME (v..) init:.. step:..   (docs/LEARN.md)
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                match learn::cmd_learn(d, &defs, rest) {
                    Ok((nm, ps, a, msg)) => { p!("{}", msg); defs.insert(nm, (ps, a)); }
                    Err(e) => p!("ERR learn {}", e),
                }
            }
            "learnfe" => {
                // learnfe NAME  -- build FE(i,j,l) by guess-and-verify (see learn.rs)
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                let name = rest.split_whitespace().next().unwrap_or("FE").to_string();
                dfa::peak_reset();
                match learn::learn_fe(d, &defs) {
                    Ok((a, st)) => {
                        p!("OK learnfe {}(i,j,l) states={} iters={} eqs={} ces={} mqs={} \
steps={} peak={} ms={}{}",
                                 name, st.states, st.iters, st.eqs, st.ces, st.mqs, st.steps,
                                 dfa::peak_get(), st.ms,
                                 if st.assumed_inf > 0 { format!(" capped_lcp={}", st.assumed_inf) } else { String::new() });
                        defs.insert(name, (vec!["i".into(), "j".into(), "l".into()], a));
                    }
                    Err(e) => p!("ERR learnfe {}", e),
                }
            }
            "enum" => {
                // enum <B> <formula>  -- list all accepted tuples with every coordinate < B
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                let (bs, f) = match rest.find(' ') { Some(i) => (&rest[..i], rest[i+1..].trim()), None => { p!("ERR usage"); continue } };
                let b: u64 = bs.parse().unwrap_or(20);
                match logic::compile_str(d.k, d, &defs, f) {
                    Ok(a) => {
                        let n = a.vars.len();
                        if n == 0 { p!("CLOSED {}", if a.accepts_epsilon() { "TRUE" } else { "FALSE" }); continue; }
                        let mut acc: Vec<String> = Vec::new();
                        let total = b.pow(n as u32);
                        for code in 0..total {
                            let mut vals = vec![0u64; n];
                            let mut c = code;
                            for i in 0..n { vals[i] = c % b; c /= b; }
                            let word = numsys::encode_word(d.k, &vals);
                            if a.run(&word) { acc.push(vals.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")); }
                        }
                        p!("ENUM vars=[{}] n={} {}", a.vars.join(","), acc.len(), acc.join(" "));
                    }
                    Err(e) => p!("ERR {}", e),
                }
            }
            "dfa" => {
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                match logic::compile_str(d.k, d, &defs, rest) {
                    Ok(a) => {
                        p!("DFA vars=[{}] states={} ({} base {}, padding allowed)", a.vars.join(","), a.nstates, if dfa::is_lsd() {"lsd"} else {"msd"}, a.k);
                        for s in 0..a.nstates {
                            let arrows: Vec<String> = (0..a.alpha).map(|x| format!("{}", a.t(s, x))).collect();
                            p!("  q{}{} -> [{}]", s, if a.accept[s] { "*" } else { " " }, arrows.join(" "));
                        }
                        let w = a.enumerate(40, 12);
                        p!("  members: {}", w.iter().map(|v| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")).collect::<Vec<_>>().join(" "));
                    }
                    Err(e) => p!("ERR {}", e),
                }
            }
            "let" => {
                // let NAME(p1,p2,..) formula
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                let Some(op) = rest.find('(') else { p!("ERR usage: let NAME(args) formula"); continue };
                let Some(cp) = rest.find(')') else { p!("ERR usage: let NAME(args) formula"); continue };
                let name = rest[..op].trim().to_string();
                let params: Vec<String> = rest[op+1..cp].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                let body = rest[cp+1..].trim();
                let t0 = Instant::now();

                // AM_AUTOLEARN (default on): if the body is exactly one of the
                // self-verifying predicate shapes (FE / rev / period / border), probe the
                // ordinary ladder cheaply; if it cannot build it cheaply, hand the whole
                // construction to `learn_pred` (guess-and-verify).  See autolearn.rs.
                // The learned automaton is *proved* language-equal to the predicate, so
                // this returns the same minimal DFA the ladder would, on every case the
                // ladder can finish -- but wins the hard cases (tail-c: 448 s -> ~15 s).
                let shape = if autolearn::enabled() {
                    logic::lex(body).ok()
                        .and_then(|toks| logic::Parser::new(toks, &d.name).parse().ok())
                        .and_then(|ast| autolearn::detect(&ast, &params))
                } else { None };

                if let Some(shape) = shape {
                    // 1. cheap-ladder probe
                    dfa::peak_reset();
                    autolearn::probe_begin();
                    let probed = logic::compile_str(d.k, d, &defs, body);
                    autolearn::probe_end();

                    if let Ok(a) = probed {
                        if !autolearn::gave_up() {
                            // Cheap rungs succeeded: identical to AM_AUTOLEARN=0.
                            let missing: Vec<&String> = params.iter().filter(|p| !a.vars.contains(p)).collect();
                            let mut vars = a.vars.clone();
                            for p in &missing { vars.push((*p).clone()); }
                            vars.sort();
                            let a2 = numsys::restrict(&a.extend_vars(&vars));
                            p!("OK let {}({}) states={} peak={} ms={} via=ladder", name, params.join(","),
                                     a2.nstates, dfa::peak_get(), t0.elapsed().as_millis());
                            defs.insert(name, (params, a2));
                            continue;
                        }
                    }
                    // 2. hand off to guess-and-verify; on ANY failure fall through to
                    // the ordinary full ladder below -- the probe only ruled out the
                    // cheap rungs, so the expensive rungs may still answer.
                    let mut handed_off = false;
                    match learn::Spec::builtin(shape.kind) {
                        Ok(spec) => {
                            dfa::peak_reset();
                            match learn::learn_pred(d, &defs, &spec) {
                                Ok((a, st)) => {
                                    // rename canonical (i,j,l / ...) -> user parameter names
                                    let a2 = a.rename(&|v| shape.user_name(v));
                                    p!("OK let {}({}) states={} peak={} ms={} via=learnfe \
kind={} eqs={} ces={} mqs={}", name, params.join(","), a2.nstates, dfa::peak_get(),
                                             t0.elapsed().as_millis(), shape.kind.name(), st.eqs, st.ces, st.mqs);
                                    defs.insert(name.clone(), (params.clone(), a2));
                                    handed_off = true;
                                }
                                Err(e) => p!("WARN let {} learnfe handoff failed ({}); \
falling back to the full ladder", name, e),
                            }
                        }
                        Err(e) => p!("WARN let {} no learner spec ({}); falling back \
to the full ladder", name, e),
                    }
                    if handed_off { continue; }
                }

                // ordinary path (no shape, or AM_AUTOLEARN=0)
                dfa::peak_reset();
                match logic::compile_str(d.k, d, &defs, body) {
                    Ok(a) => {
                        let missing: Vec<&String> = params.iter().filter(|p| !a.vars.contains(p)).collect();
                        // a parameter the body does not constrain is legal (cylindrify it in)
                        let mut vars = a.vars.clone();
                        for p in &missing { vars.push((*p).clone()); }
                        vars.sort();
                        let a2 = numsys::restrict(&a.extend_vars(&vars));
                        let extra: Vec<String> = a2.vars.iter().filter(|v| !params.contains(v)).cloned().collect();
                        if !extra.is_empty() {
                            p!("ERR ${} body has unbound variables {:?} not in the parameter list", name, extra);
                            continue;
                        }
                        p!("OK let {}({}) states={} peak={} ms={}", name, params.join(","),
                                 a2.nstates, dfa::peak_get(), t0.elapsed().as_millis());
                        defs.insert(name, (params, a2));
                    }
                    Err(e) => p!("ERR {}", e),
                }
            }
            "finite" => {
                // Is the set defined by a one-variable formula finite?  A regular
                // language is finite exactly when no cycle lies on a path from the
                // start state to an accepting state.  If finite, report the largest
                // member, which is what turns "PPL(n) <= c only finitely often" into
                // a proof that PPL tends to infinity.
                let Some(d) = &cur else { p!("ERR no sequence"); continue };
                match logic::compile_str(d.k, d, &defs, rest) {
                    Ok(a) => {
                        // A value has infinitely many padded representations, so
                        // the raw automaton always carries the padding cycle.
                        // Analyse the canonical (pad-quotient) language instead,
                        // which has exactly one word per value under the active
                        // numeration system (base-k or numsys).
                        let a = a.pad_quotient().minimize();
                        let n = a.nstates;
                        // states that can reach an accepting state
                        let mut useful = a.accept.clone();
                        loop {
                            let mut ch = false;
                            for s in 0..n { if !useful[s] {
                                for x in 0..a.alpha { if useful[a.t(s,x)] { useful[s]=true; ch=true; break; } }
                            }}
                            if !ch { break; }
                        }
                        // reachable from start among useful states
                        let mut seen = vec![false; n];
                        let mut stack = vec![0usize];
                        if useful[0] { seen[0]=true; } else { stack.clear(); }
                        while let Some(s) = stack.pop() {
                            for x in 0..a.alpha {
                                let t = a.t(s,x);
                                if useful[t] && !seen[t] { seen[t]=true; stack.push(t); }
                            }
                        }
                        // cycle detection restricted to seen+useful
                        let mut colour = vec![0u8; n];
                        let mut infinite = false;
                        fn dfs(a: &dfa::Dfa, s: usize, seen: &[bool], colour: &mut Vec<u8>, inf: &mut bool) {
                            colour[s] = 1;
                            for x in 0..a.alpha {
                                let t = a.t(s,x);
                                if !seen[t] { continue; }
                                if colour[t] == 1 { *inf = true; return; }
                                if colour[t] == 0 { dfs(a, t, seen, colour, inf); if *inf { return; } }
                            }
                            colour[s] = 2;
                        }
                        if seen[0] { dfs(&a, 0, &seen, &mut colour, &mut infinite); }
                        if !seen.iter().any(|&b| b) {
                            p!("EMPTY :: {}", rest);
                        } else if infinite {
                            p!("INFINITE states={} :: {}", a.nstates, rest);
                        } else {
                            let w = a.enumerate(200000, 40);
                            let mx = w.iter().map(|v| v[0]).max().unwrap_or(0);
                            p!("FINITE size={} max={} states={} :: {}", w.len(), mx, a.nstates, rest);
                        }
                    }
                    Err(e) => p!("ERR {}", e),
                }
            }
            _ => p!("ERR unknown command {:?}", cmd),
        }
        progress::done(cmd, "");
        let _ = out.flush();
    }
}

/// Run a whole script (newline-separated commands) to completion, returning the
/// engine's stdout as one string. This is the wasm playground's transport and a
/// convenient in-process entry for tests.
pub fn run_script(input: &str) -> String {
    let mut buf: Vec<u8> = Vec::new();
    run_loop(input.lines().map(|s| s.to_string()), &mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

/// One-time process init: read the memory budget from the environment (native)
/// or apply the fixed default (wasm, where there is no environment), and arm the
/// progress channel. Call once before the first `run_loop`/`run_script`.
pub fn init() {
    membudget::init();
    progress::init();
}

// --------------------------------------------------------------- wasm bindings
// The browser playground: `run(script)` mirrors the CLI's stdin->stdout loop
// over in-memory buffers. No threads, no files, no OS RAM probe -- see the
// target_arch="wasm32" gates in det_par.rs and membudget.rs.
#[cfg(target_arch = "wasm32")]
mod wasmapi {
    use wasm_bindgen::prelude::*;
    use std::sync::Once;
    static START: Once = Once::new();

    /// Run a Peanut script and return the engine's stdout as a string.
    #[wasm_bindgen]
    pub fn run(script: &str) -> String {
        START.call_once(super::init);
        super::run_script(script)
    }

    /// Set the in-browser memory budget (MB); mirrors AM_MEM_MB on the CLI.
    /// Runs one-time init FIRST so the default-2048 from init() cannot clobber
    /// this on the very first run (init has no env to read on wasm).
    #[wasm_bindgen]
    pub fn set_budget(mb: usize) {
        START.call_once(super::init);
        super::membudget::set_limit_mb(mb);
    }

    /// The engine version string, for the playground footer.
    #[wasm_bindgen]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
