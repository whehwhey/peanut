//! Peanut engine entry point: a stdin/stdout REPL over the command language
//! documented in `docs/GUARD.md` (`mode`, `def`, `let`, `?`, `witness`,
//! `learnfe`, `enum`, `finite`, `mem`, `quit`). Reads one command per line,
//! dispatches to [`base`]/[`logic`]/[`learn`] to build or query automata, and
//! prints one `OK ...` / `ERR ...` (or command-specific) reply per line so a
//! driver such as `explore/engine.py` can pipe commands in and parse replies
//! out without any framing beyond newlines. Always invoke this binary through
//! that Python wrapper, never with unguarded parallel instances: several
//! engines racing on the same machine can exhaust memory and lock it up.
mod dfa;
mod base;
mod dfao;
mod export;
mod learn;
mod logic;
mod membudget;
mod progress;

#[global_allocator]
static GLOBAL: membudget::Budgeted = membudget::Budgeted;

use dfao::Dfao;
use std::io::{self, BufRead, Write};
use std::time::Instant;

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

fn main() {
    membudget::init();
    progress::init();
    let stdin = io::stdin();
    let mut cur: Option<Dfao> = None;
    let mut defs: logic::Defs = logic::Defs::new();
    let mut out = io::stdout();
    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let (cmd, rest) = match line.find(' ') {
            Some(i) => (&line[..i], line[i + 1..].trim()),
            None => (line, ""),
        };
        if progress::on() {
            match cmd {
                "?" | "eval" | "let" | "witness" | "enum" | "dfa" | "finite" | "learnfe" =>
                    progress::phase("compile", line),
                _ => {}
            }
        }
        match cmd {
            "quit" | "exit" => break,
            "mem" => { progress::mem();
                       println!("OK mem live={}MB peak={}MB", membudget::live_mb(), membudget::peak_mb()) }
            "mode" => {
                let lsd = rest.trim() == "lsd";
                dfa::set_lsd(lsd);
                println!("OK mode {}", if lsd { "lsd" } else { "msd" });
            }
            "def" => {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                match parse_def(&parts) {
                    Ok(d) => {
                        println!("OK def {} k={} states={} lsd_states={} mode={}", d.name, d.k, d.nstates,
                                 d.lnstates, if dfa::is_lsd() { "lsd" } else { "msd" });
                        defs.clear();
                        cur = Some(d);
                    }
                    Err(e) => println!("ERR {}", e),
                }
            }
            "seq" => {
                // seq N -- the first N symbols of T as one string of digits
                let n: usize = rest.split_whitespace().next().and_then(|x| x.parse().ok()).unwrap_or(60);
                match &cur {
                    Some(d) => println!("SEQ n={} k={} {}", n, d.k,
                        d.prefix(n).iter().map(|x| x.to_string()).collect::<Vec<_>>().join("")),
                    None => println!("ERR no sequence"),
                }
            }
            "export" => {
                // export NAME -- the automaton as one JSON line.  NAME = T (the DFAO)
                // or any predicate bound by `let` / `learnfe`.
                let Some(d) = &cur else { println!("ERR no sequence"); continue };
                let name = rest.split_whitespace().next().unwrap_or("T");
                if name == "T" || name == d.name {
                    println!("EXPORT {}", export::dfao_json(d));
                } else if let Some((params, a)) = defs.get(name) {
                    println!("EXPORT {}", export::dfa_json(name, params, a));
                } else {
                    println!("ERR export: no such predicate {:?} (have: T{}{})", name,
                             if defs.is_empty() { "" } else { ", " },
                             defs.keys().cloned().collect::<Vec<_>>().join(", "));
                }
            }
            "fe_map" => {
                // fe_map i0 j0 size L -- a size x size grid of FE(i,j,L) for
                // i in [i0, i0+size), j in [j0, j0+size), computed by direct LCP walk
                // through the DFAO.  No automaton is built: this is the ground truth the
                // FE automaton is supposed to encode, and it is what the heatmap draws.
                let Some(d) = &cur else { println!("ERR no sequence"); continue };
                let p: Vec<u64> = rest.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                if p.len() < 4 { println!("ERR usage: fe_map i0 j0 size L"); continue }
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
                println!("FEMAP i0={} j0={} size={} l={} ms={} rows={}",
                         i0, j0, size, l, t0.elapsed().as_millis(), rows.join(","));
            }
            "?" | "eval" => {
                let Some(d) = &cur else { println!("ERR no sequence"); continue };
                let t0 = Instant::now();
                dfa::peak_reset();
                match logic::compile_str(d.k, d, &defs, rest) {
                    Ok(a) => {
                        let ms = t0.elapsed().as_millis();
                        if a.vars.is_empty() {
                            println!("{} states={} peak={} ms={} :: {}",
                                if a.accepts_epsilon() { "TRUE" } else { "FALSE" }, a.nstates,
                                dfa::peak_get(), ms, rest);
                        } else {
                            let w = a.enumerate(12, 14);
                            let ws: Vec<String> = w.iter().map(|v| format!("({})", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))).collect();
                            println!("OPEN vars=[{}] states={} nonempty={} ms={} witnesses={} :: {}",
                                a.vars.join(","), a.nstates, a.is_nonempty(), ms, ws.join(" "), rest);
                        }
                    }
                    Err(e) => println!("ERR {}", e),
                }
            }
            "witness" => {
                // witness <formula>  -- one satisfying assignment (the SHORTEST accepted
                // word), or NONE.  For a closed formula this degenerates to TRUE/FALSE.
                let Some(d) = &cur else { println!("ERR no sequence"); continue };
                let t0 = Instant::now();
                dfa::peak_reset();
                match logic::compile_str(d.k, d, &defs, rest) {
                    Ok(a) => {
                        let ms = t0.elapsed().as_millis();
                        if a.vars.is_empty() {
                            println!("{} states={} ms={} :: {}",
                                if a.accepts_epsilon() { "TRUE" } else { "FALSE" }, a.nstates, ms, rest);
                        } else {
                            match a.shortest_word() {
                                None => println!("NONE vars=[{}] states={} ms={} :: {}",
                                                 a.vars.join(","), a.nstates, ms, rest),
                                Some(w) => {
                                    let v = learn::decode(a.k, a.vars.len(), &w)
                                        .unwrap_or_else(|| vec![0; a.vars.len()]);
                                    let asg: Vec<String> = a.vars.iter().zip(v.iter())
                                        .map(|(n, x)| format!("{}={}", n, x)).collect();
                                    println!("WITNESS {} states={} len={} ms={} :: {}",
                                             asg.join(" "), a.nstates, w.len(), ms, rest);
                                }
                            }
                        }
                    }
                    Err(e) => println!("ERR {}", e),
                }
            }
            "learnfe" => {
                // learnfe NAME  -- build FE(i,j,l) by guess-and-verify (see learn.rs)
                let Some(d) = &cur else { println!("ERR no sequence"); continue };
                let name = rest.split_whitespace().next().unwrap_or("FE").to_string();
                dfa::peak_reset();
                match learn::learn_fe(d, &defs) {
                    Ok((a, st)) => {
                        println!("OK learnfe {}(i,j,l) states={} iters={} eqs={} ces={} mqs={} \
steps={} peak={} ms={}{}",
                                 name, st.states, st.iters, st.eqs, st.ces, st.mqs, st.steps,
                                 dfa::peak_get(), st.ms,
                                 if st.assumed_inf > 0 { format!(" capped_lcp={}", st.assumed_inf) } else { String::new() });
                        defs.insert(name, (vec!["i".into(), "j".into(), "l".into()], a));
                    }
                    Err(e) => println!("ERR learnfe {}", e),
                }
            }
            "enum" => {
                // enum <B> <formula>  -- list all accepted tuples with every coordinate < B
                let Some(d) = &cur else { println!("ERR no sequence"); continue };
                let (bs, f) = match rest.find(' ') { Some(i) => (&rest[..i], rest[i+1..].trim()), None => { println!("ERR usage"); continue } };
                let b: u64 = bs.parse().unwrap_or(20);
                match logic::compile_str(d.k, d, &defs, f) {
                    Ok(a) => {
                        let n = a.vars.len();
                        if n == 0 { println!("CLOSED {}", if a.accepts_epsilon() { "TRUE" } else { "FALSE" }); continue; }
                        // common word length
                        let mut len = 1usize; let mut cap = d.k as u64;
                        while cap < b { cap *= d.k as u64; len += 1; }
                        let mut acc: Vec<String> = Vec::new();
                        let total = b.pow(n as u32);
                        for code in 0..total {
                            let mut vals = vec![0u64; n];
                            let mut c = code;
                            for i in 0..n { vals[i] = c % b; c /= b; }
                            // build msd word
                            let mut word = vec![0usize; len];
                            for pos in 0..len {
                                let place = if dfa::is_lsd() { pos } else { len - 1 - pos };
                                let mut sym = 0usize; let mut mult = 1usize;
                                for i in 0..n {
                                    let dig = (vals[i] / (d.k as u64).pow(place as u32)) % d.k as u64;
                                    sym += dig as usize * mult; mult *= d.k;
                                }
                                word[pos] = sym;
                            }
                            if a.run(&word) { acc.push(vals.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")); }
                        }
                        println!("ENUM vars=[{}] n={} {}", a.vars.join(","), acc.len(), acc.join(" "));
                    }
                    Err(e) => println!("ERR {}", e),
                }
            }
            "dfa" => {
                let Some(d) = &cur else { println!("ERR no sequence"); continue };
                match logic::compile_str(d.k, d, &defs, rest) {
                    Ok(a) => {
                        println!("DFA vars=[{}] states={} ({} base {}, padding allowed)", a.vars.join(","), a.nstates, if dfa::is_lsd() {"lsd"} else {"msd"}, a.k);
                        for s in 0..a.nstates {
                            let arrows: Vec<String> = (0..a.alpha).map(|x| format!("{}", a.t(s, x))).collect();
                            println!("  q{}{} -> [{}]", s, if a.accept[s] { "*" } else { " " }, arrows.join(" "));
                        }
                        let w = a.enumerate(40, 12);
                        println!("  members: {}", w.iter().map(|v| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")).collect::<Vec<_>>().join(" "));
                    }
                    Err(e) => println!("ERR {}", e),
                }
            }
            "let" => {
                // let NAME(p1,p2,..) formula
                let Some(d) = &cur else { println!("ERR no sequence"); continue };
                let Some(op) = rest.find('(') else { println!("ERR usage: let NAME(args) formula"); continue };
                let Some(cp) = rest.find(')') else { println!("ERR usage: let NAME(args) formula"); continue };
                let name = rest[..op].trim().to_string();
                let params: Vec<String> = rest[op+1..cp].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                let body = rest[cp+1..].trim();
                let t0 = Instant::now();
                match logic::compile_str(d.k, d, &defs, body) {
                    Ok(a) => {
                        let missing: Vec<&String> = params.iter().filter(|p| !a.vars.contains(p)).collect();
                        // a parameter the body does not constrain is legal (cylindrify it in)
                        let mut vars = a.vars.clone();
                        for p in &missing { vars.push((*p).clone()); }
                        vars.sort();
                        let a2 = a.extend_vars(&vars);
                        let extra: Vec<String> = a2.vars.iter().filter(|v| !params.contains(v)).cloned().collect();
                        if !extra.is_empty() {
                            println!("ERR ${} body has unbound variables {:?} not in the parameter list", name, extra);
                            continue;
                        }
                        println!("OK let {}({}) states={} ms={}", name, params.join(","), a2.nstates, t0.elapsed().as_millis());
                        defs.insert(name, (params, a2));
                    }
                    Err(e) => println!("ERR {}", e),
                }
            }
            "finite" => {
                // Is the set defined by a one-variable formula finite?  A regular
                // language is finite exactly when no cycle lies on a path from the
                // start state to an accepting state.  If finite, report the largest
                // member, which is what turns "PPL(n) <= c only finitely often" into
                // a proof that PPL tends to infinity.
                let Some(d) = &cur else { println!("ERR no sequence"); continue };
                match logic::compile_str(d.k, d, &defs, rest) {
                    Ok(a) => {
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
                            println!("EMPTY :: {}", rest);
                        } else if infinite {
                            println!("INFINITE states={} :: {}", a.nstates, rest);
                        } else {
                            let w = a.enumerate(200000, 40);
                            let mx = w.iter().map(|v| v[0]).max().unwrap_or(0);
                            println!("FINITE size={} max={} states={} :: {}", w.len(), mx, a.nstates, rest);
                        }
                    }
                    Err(e) => println!("ERR {}", e),
                }
            }
            _ => println!("ERR unknown command {:?}", cmd),
        }
        progress::done(cmd, "");
        let _ = out.flush();
    }
}
