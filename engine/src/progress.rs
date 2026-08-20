//! Structured progress events on stderr (`AM_PROGRESS=1`).
//!
//! A long `let FE(...)` is opaque: the process sits there for two minutes and either
//! answers or dies at the memory budget.  With `AM_PROGRESS=1` the engine narrates what
//! it is doing as one JSON object per line on **stderr**, so stdout stays exactly the
//! line protocol every script in `explore/` already parses.
//!
//!     {"ev":"phase","name":"forward","ms":12,"detail":"exists l"}
//!     {"ev":"subsets","n":150000,"mb":41,"ms":880}
//!     {"ev":"states","n":1382,"ms":1502}
//!     {"ev":"mem","mb":50,"ms":1502}
//!     {"ev":"done","cmd":"let","ms":1503}
//!
//! Cost when off: one relaxed atomic load at each call site, and in the subset
//! construction a single `usize` compare per subset.  Nothing is formatted, nothing is
//! allocated, and no syscall is made unless the flag is set.
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use crate::clock::Instant;

static ON: AtomicBool = AtomicBool::new(false);
static START: OnceLock<Instant> = OnceLock::new();

/// Read the env flag once, at the top of main.
pub fn init() {
    let on = std::env::var("AM_PROGRESS").map(|v| v != "0" && !v.is_empty()).unwrap_or(false);
    ON.store(on, Ordering::Relaxed);
    let _ = START.set(Instant::now());
}

#[inline]
pub fn on() -> bool { ON.load(Ordering::Relaxed) }

fn ms() -> u128 {
    START.get().map(|t| t.elapsed().as_millis()).unwrap_or(0)
}

fn emit(body: &str) {
    let mut e = std::io::stderr().lock();
    let _ = writeln!(e, "{{{},\"ms\":{}}}", body, ms());
    let _ = e.flush();
}

/// One of: forward | brzozowski | minimize | learn | verify | compile.
pub fn phase(name: &str, detail: &str) {
    if !on() { return; }
    emit(&format!("\"ev\":\"phase\",\"name\":\"{}\",\"detail\":\"{}\"", name, esc(detail)));
}

/// Subset-construction tick (~every 50k subsets).
pub fn subsets(n: usize) {
    if !on() { return; }
    emit(&format!("\"ev\":\"subsets\",\"n\":{},\"mb\":{}", n, crate::membudget::live_mb()));
}

/// Size of an automaton that has just been built.
pub fn states(n: usize, what: &str) {
    if !on() { return; }
    emit(&format!("\"ev\":\"states\",\"n\":{},\"what\":\"{}\"", n, esc(what)));
}

/// Emit a snapshot of current and peak live memory (from [`crate::membudget`]).
pub fn mem() {
    if !on() { return; }
    emit(&format!("\"ev\":\"mem\",\"mb\":{},\"peak_mb\":{}",
                  crate::membudget::live_mb(), crate::membudget::peak_mb()));
}

/// Learner heartbeat: one per equivalence query.
pub fn learn(eqs: usize, states: usize, mqs: u64) {
    if !on() { return; }
    emit(&format!("\"ev\":\"learn\",\"eqs\":{},\"states\":{},\"mqs\":{}", eqs, states, mqs));
}

/// End of a top-level command.
pub fn done(cmd: &str, detail: &str) {
    if !on() { return; }
    emit(&format!("\"ev\":\"done\",\"cmd\":\"{}\",\"detail\":\"{}\",\"mb\":{},\"peak_mb\":{}",
                  cmd, esc(detail), crate::membudget::live_mb(), crate::membudget::peak_mb()));
}

/// Minimal JSON string escaping — our detail strings are formulas, which can contain
/// quotes and backslashes.
fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' | '\r' | '\t' => o.push(' '),
            c if (c as u32) < 0x20 => o.push(' '),
            c => o.push(c),
        }
    }
    o
}
