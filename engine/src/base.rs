//! Arithmetic automata for base-k Presburger arithmetic (msd-first).
//!
//! Every constructor here first asks [`crate::numsys::active`] whether a
//! non-standard numeration system is in force.  If one is, the adder and the
//! comparison come from that system's automata (loaded from file) and every
//! result is conjoined with "each track is a valid representation"; if none is,
//! the built-in base-`k` fast paths below are used unchanged.

use crate::dfa::{Dfa, State, is_lsd};
use crate::numsys;

/// Attach `names` (in coordinate order) to a nameless automaton and permute the
/// coordinates into sorted order, which is the invariant every `Dfa` keeps.
fn named(d: &Dfa, names: &[&str]) -> Dfa {
    let mut out = Dfa { vars: names.iter().map(|s| s.to_string()).collect(), ..d.clone() };
    let mut sorted: Vec<String> = out.vars.clone();
    sorted.sort();
    if sorted != out.vars { out = out.extend_vars(&sorted); }
    out
}

/// LSD-first adder over (x,y,z): x + y = z.  States: 0 = carry 0, 1 = carry 1, 2 = dead.
fn adder_lsd(k: usize, vars: Vec<String>) -> Dfa {
    let alpha = k * k * k;
    let mut trans = vec![2u32; 3 * alpha];
    for carry in 0..2usize {
        for a in 0..k {
            for b in 0..k {
                for c in 0..k {
                    let sym = a + b * k + c * k * k;
                    let sum = a + b + carry;
                    trans[carry * alpha + sym] = if sum % k == c { (sum / k) as State } else { 2 };
                }
            }
        }
    }
    for s in 0..alpha { trans[2 * alpha + s] = 2; }
    Dfa::new(k, vars, 3, trans, vec![true, false, false])
}

/// MSD-first x + y = z.
pub fn adder(k: usize, x: &str, y: &str, z: &str) -> Dfa {
    if let Some(ns) = numsys::active() {
        return ns.restrict(&named(ns.add(), &[x, y, z])).minimize();
    }
    let mut vars = vec![x.to_string(), y.to_string(), z.to_string()];
    // adder_lsd encodes coordinate order (x,y,z); build with placeholder names then rename.
    let names: Vec<String> = vec!["\u{1}a".into(), "\u{1}b".into(), "\u{1}c".into()];
    let raw = adder_lsd(k, names);
    let d = if is_lsd() { raw } else { raw.reverse_determinize() };
    // reverse_determinize keeps coordinate order; now attach real names in that order.
    let mut out = Dfa { vars: vec![x.into(), y.into(), z.into()], ..d };
    // if the caller's names are not already sorted, re-sort coordinates
    vars.sort();
    if vars != out.vars { out = out.extend_vars(&vars); }
    out.minimize()
}

/// MSD-first x < y (leading zeros allowed on both).
pub fn less_than(k: usize, x: &str, y: &str) -> Dfa {
    if let Some(ns) = numsys::active() {
        return ns.restrict(&named(ns.lt(), &[x, y])).minimize();
    }
    // msd: the FIRST differing digit decides, so the verdict is absorbing.
    // lsd: the LAST (most significant) differing digit decides, so each new
    //      differing digit overrides the running verdict.
    let alpha = k * k;
    let mut trans = vec![0u32; 3 * alpha];
    let lsd = is_lsd();
    for a in 0..k {
        for b in 0..k {
            let sym = a + b * k;
            let v: u32 = if a < b { 1 } else if a > b { 2 } else { 0 };
            trans[0 * alpha + sym] = v;
            trans[1 * alpha + sym] = if lsd { if v == 0 { 1 } else { v } } else { 1 };
            trans[2 * alpha + sym] = if lsd { if v == 0 { 2 } else { v } } else { 2 };
        }
    }
    let mut vars = vec![x.to_string(), y.to_string()];
    let d = Dfa::new(k, vec!["\u{1}a".into(), "\u{1}b".into()], 3, trans, vec![false, true, false]);
    let mut out = Dfa { vars: vec![x.into(), y.into()], ..d };
    vars.sort();
    if vars != out.vars { out = out.extend_vars(&vars); }
    out.minimize()
}

/// MSD-first x = y.
///
/// Digit-string equality.  Under a numeration system this is still exactly
/// numeric equality, because canonical representations are unique once both
/// tracks are known valid -- which is what the `restrict` below enforces.
pub fn equal(k: usize, x: &str, y: &str) -> Dfa {
    if let Some(ns) = numsys::active() {
        let d = digit_equal(ns.digits, x, y);
        return ns.restrict(&d).minimize();
    }
    digit_equal(k, x, y)
}

fn digit_equal(k: usize, x: &str, y: &str) -> Dfa {
    let alpha = k * k;
    let mut trans = vec![1u32; 2 * alpha];
    for a in 0..k {
        for b in 0..k {
            trans[a + b * k] = if a == b { 0 } else { 1 };
        }
    }
    for s in 0..alpha { trans[alpha + s] = 1; }
    let mut vars = vec![x.to_string(), y.to_string()];
    let d = Dfa::new(k, vec!["\u{1}a".into(), "\u{1}b".into()], 2, trans, vec![true, false]);
    let mut out = Dfa { vars: vec![x.into(), y.into()], ..d };
    vars.sort();
    if vars != out.vars { out = out.extend_vars(&vars); }
    out.minimize()
}

/// Largest constant `constant()` will build an automaton for. The automaton
/// is a plain digit-string recognizer (O(log_k c) states, both digit orders),
/// so this is a sanity cap against typos/garbage input, not a structural
/// limit -- raise it further if a real proof ever needs to.
pub const MAX_CONSTANT: u64 = 1_000_000_000_000; // 10^12

/// MSD-first x = c for a fixed constant c.
///
/// Builds a states-per-digit recognizer of c's base-k digit string (padded
/// with 0s on the appropriate side for the active digit order), so cost is
/// O(log_k c) regardless of mode -- there is no O(c) blowup to guard against.
pub fn constant(k: usize, x: &str, c: u64) -> Result<Dfa, String> {
    if c > MAX_CONSTANT {
        return Err(format!("constant too large (max {})", MAX_CONSTANT));
    }
    if let Some(ns) = numsys::active() {
        // The canonical representation of c, padded with zeros at the padding end.
        // No validity conjunction is needed: the language is the single valid word.
        let rep = ns.rep(c);            // msd-first, no leading zeros
        let d = ns.digits;
        let len = rep.len();
        let n = len + 2;
        let dead = (len + 1) as State;
        let mut trans = vec![dead; n * d];
        let mut accept = vec![false; n];
        if is_lsd() {
            for p in 0..len {
                let dig = rep[len - 1 - p];       // lsd-first
                trans[p * d + dig] = (p + 1) as State;
            }
            trans[len * d] = len as State;        // trailing zero padding
            accept[len] = true;
        } else {
            trans[0] = 0;                          // leading zero padding
            for p in 0..len { trans[p * d + rep[p]] = (p + 1) as State; }
            accept[len] = true;
        }
        // c = 0 is represented by the empty word: 0* only.
        if c == 0 {
            let mut t = vec![1 as State; 2 * d];
            t[0] = 0;
            return Ok(Dfa::new(d, vec![x.to_string()], 2, t, vec![true, false]).minimize());
        }
        return Ok(Dfa::new(d, vec![x.to_string()], n, trans, accept).minimize());
    }
    // lsd-first digits of c (ds[0] is the least significant digit).
    let mut ds: Vec<usize> = Vec::new();
    let mut m = c;
    while m > 0 { ds.push((m % k as u64) as usize); m /= k as u64; }
    let len = ds.len();
    let n = len + 2;              // states 0..len are positions, len+1 is dead
    let dead = (len + 1) as u32;
    let mut trans = vec![dead; n * k];
    let mut accept = vec![false; n];
    if is_lsd() {
        // language = lsd digits of c, then 0* (trailing zero padding).
        for p in 0..len { for d in 0..k { trans[p * k + d] = if d == ds[p] { (p + 1) as u32 } else { dead }; } }
        for d in 0..k { trans[len * k + d] = if d == 0 { len as u32 } else { dead }; }
        accept[len] = true;
    } else {
        // language = 0* (leading zero padding), then msd digits of c.
        // ds[len-1] (the last digit pushed above) is c's most significant
        // digit and is nonzero whenever len > 0, so state 0's self-loop on
        // digit 0 never collides with the "start of the real digits" edge.
        trans[0] = 0;
        for p in 0..len {
            let d = ds[len - 1 - p];
            trans[p * k + d] = (p + 1) as u32;
        }
        accept[len] = true;
    }
    Ok(Dfa::new(k, vec![x.to_string()], n, trans, accept).minimize())
}

/// {k^j : j >= 0}.  In base k the representation of k^j is a single 1 followed by zeros,
/// and padding is zeros at the other end, so in BOTH digit orders the language is 0* 1 0*.
/// This is the V_k part of the signature <N, +, V_k> that the decidability theorem is
/// actually stated over; the engine simply had not needed it until now.
pub fn power_of_k(k: usize, x: &str) -> Dfa {
    // 0 = no 1 seen yet, 1 = exactly one 1 seen, 2 = dead
    let mut trans = vec![2u32; 3 * k];
    trans[0] = 0;                 // leading/trailing zeros
    trans[1] = 1;                 // the single 1
    for d in 0..k { trans[1 * k + d] = if d == 0 { 1 } else { 2 }; }
    for d in 0..k { trans[2 * k + d] = 2; }
    Dfa::new(k, vec![x.to_string()], 3, trans, vec![false, true, false]).minimize()
}
