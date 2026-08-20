//! Peanut CLI entry point: a thin stdin/stdout driver over the shared command
//! REPL in `lib.rs`. Reads one command per line and streams one reply per line
//! so a driver such as `explore/engine.py` can pipe commands in and parse
//! replies out with no framing beyond newlines. The dispatch itself, and the
//! `#[global_allocator]` memory budget, live in the library so the wasm
//! playground (`peanut::run_script`) runs the exact same engine in a browser.
//! Always invoke this binary through that Python wrapper, never with unguarded
//! parallel instances: several engines racing on one machine can exhaust memory.
use peanut::run_loop;
use std::io::{self, BufRead, Write};

fn main() {
    peanut::init();
    let stdin = io::stdin();
    let mut out = io::stdout();
    // run_loop flushes `out` after every command, so a piping driver sees each
    // reply immediately; map_while stops at the first read error (old EOF break).
    run_loop(stdin.lock().lines().map_while(Result::ok), &mut out);
    let _ = out.flush();
}
