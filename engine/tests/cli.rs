//! Integration tests for the peanut CLI, seeded from the 2026-08-20 live-debug
//! session. Each test drives the built binary over stdin and asserts on stdout.
//! Covers: the `finite` pad-quotient fix (msd + lsd + numsys fib), the
//! seqname parser fix (non-"T" sequences indexable), and two classical
//! theorem sanity checks (Thue-Morse squares, Fibonacci-word 4th powers).
use std::io::Write;
use std::process::{Command, Stdio};

fn run(script: &str) -> String {
    let mut c = Command::new(env!("CARGO_BIN_EXE_peanut"))
        .stdin(Stdio::piped()).stdout(Stdio::piped())
        .spawn().expect("spawn peanut");
    c.stdin.as_mut().unwrap().write_all(script.as_bytes()).unwrap();
    let out = c.wait_with_output().expect("run peanut");
    String::from_utf8(out.stdout).expect("utf8")
}

const TM: &str = "mode msd\ndef T 2 2 0 01 10 01\n";

#[test]
fn finite_pad_quotient_msd() {
    let o = run(&format!("{TM}finite i<3\nfinite i<1\nfinite T[i]=5\nfinite T[i]=1\nquit\n"));
    assert!(o.contains("FINITE size=3 max=2"), "i<3 must be finite: {o}");
    assert!(o.contains("FINITE size=1 max=0"), "value 0 via the word \"0\": {o}");
    assert!(o.contains("EMPTY :: T[i]=5"), "{o}");
    assert!(o.contains("INFINITE") && o.contains(":: T[i]=1"), "{o}");
}

#[test]
fn finite_pad_quotient_lsd() {
    let o = run("mode lsd\ndef T 2 2 0 01 10 01\nfinite i<3\nfinite T[i]=1\nquit\n");
    assert!(o.contains("FINITE size=3 max=2"), "lsd padding is trailing zeros: {o}");
    assert!(o.contains("INFINITE"), "{o}");
}

#[test]
fn finite_correct_answer_not_stale_doc() {
    // COMMANDS.md's old transcript says size=4 max=9; T[9]=0, so the correct
    // answer is size=5 max=8 (values 1,2,4,7,8).
    let o = run(&format!("{TM}finite T[i]=1 & i<10\nquit\n"));
    assert!(o.contains("FINITE size=5 max=8"), "{o}");
}

#[test]
fn seqname_not_hardcoded_to_t() {
    let o = run("mode msd\ndfao W 2 0:0,1 1:1,0\n? W[0]=0\n? E i. W[i]=1\nquit\n");
    assert!(o.contains("TRUE") && o.contains(":: W[0]=0"), "non-T names must index: {o}");
    assert!(o.contains(":: E i. W[i]=1"), "{o}");
    assert!(!o.contains("expected relation"), "{o}");
}

#[test]
fn fib_numeration_finite_and_powers() {
    let o = run("mode msd\nnumsys fib\ndfao F 2 0:0,1 1:0,-\nseq 13\n\
                 finite F[n]=1 & n<10\n\
                 ? E i,l. l > 0 & (A t. t < 2*l => F[i+t] = F[i+l+t])\n\
                 ? E i,l. l > 0 & (A t. t < 3*l => F[i+t] = F[i+l+t])\nquit\n");
    assert!(o.contains("SEQ n=13 k=2 0100101001001"), "Fibonacci word: {o}");
    assert!(o.contains("FINITE size=4 max=9"), "values 1,4,6,9: {o}");
    assert!(o.contains("TRUE"), "Fibonacci word contains cubes: {o}");
    assert!(o.contains("FALSE"), "but no 4th powers: {o}");
}

#[test]
fn tm_squares_still_true() {
    let o = run(&format!(
        "{TM}? E i,l. l > 0 & (A t. t < l => T[i+t] = T[i+l+t])\nquit\n"));
    assert!(o.contains("TRUE"), "{o}");
}

#[test]
fn learn_seqname_not_hardcoded_to_t() {
    // learn.rs previously hardcoded the word name "T" in parse_formula, so a
    // custom `learn` spec referencing a differently-named current sequence
    // failed to parse. Both learnfe and a custom spec must work under name "S".
    let o = run("mode msd\ndef S 2 2 0 01 10 01\n\
                 learnfe FE\n\
                 learn EQ (i,j) on:j init:S[i]=S[i] step:(S[i]=S[j+1]) & $H(i,j)\nquit\n");
    assert!(o.contains("OK learnfe FE(i,j,l)"), "learnfe under non-T: {o}");
    assert!(o.contains("OK learn EQ(i,j)"), "custom learn under non-T: {o}");
    assert!(!o.contains("expected relation"), "{o}");
}
