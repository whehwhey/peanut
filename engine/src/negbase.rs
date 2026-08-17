//! Negative bases: `msd_neg_k`, the numeration system with digits
//! `{0, .., k-1}` and place values `(-k)^i`.
//!
//! Every integer -- negative ones included -- has exactly one representation
//! with no leading zero, so a first-order sentence over `msd_neg_2` quantifies
//! over **Z**, not **N**.  That is Walnut's semantics too (`NumberSystem.isNeg`,
//! `baseNegNAddition`, `baseNegNLessThan`), and it is the reason a negative base
//! cannot be handled by `numsys.rs`'s default machinery: there, the value of a
//! word is its *rank* in the radix ordering of the validity language, and in a
//! negative base radix order is not numeric order (`1 0 = -k < 0 1 = 1`).  So
//! `NumSys` carries a `neg_base` flag and this module supplies `rep`/`value`.
//!
//! Reference: J. Shallit, *Automatic sequences in negative bases and
//! decidability of Buchi arithmetic*, arXiv:2208.06025.
//!
//! The three automata (all msd-first, leading zeros allowed, digits
//! `{0..k-1}`):
//!
//! * **validity** -- one accepting state looping on every digit: every word is
//!   a representation.
//! * **addition** `(x, y, z) : x + y = z` -- 3 states.  Reading msd-first, let
//!   `t_p = sum_{q >= p} (x_q + y_q - z_q) (-k)^{q-p}`; then
//!   `t_p = (x_p + y_p - z_p) - k * t_{p+1}`, `t_L = 0`, and `x + y = z` iff
//!   `t_0 = 0`.  The reachable values are `t in {-1, 0, 1}`, which is the state.
//! * **comparison** `(x, y) : x < y` -- 3 states.  The first differing position
//!   decides, but each further digit flips the verdict, because consecutive
//!   place values have opposite signs.
//!
//! All three are *generated*, then validated against `i64` arithmetic
//! (`self_check` in `numsys.rs`, plus `explore/negbase_check.py` on 10^5 random
//! pairs and against Walnut itself).

use crate::numsys::Parsed;

/// msd-first digits of `n` in base `-k` (no leading zeros; `[0]` for `n = 0`).
/// Standard algorithm: `r = n mod k` taken non-negative, `n <- (n - r) / (-k)`.
pub fn rep(mut n: i64, k: u32) -> Vec<usize> {
    if n == 0 { return vec![0]; }
    let kk = k as i64;
    let mut out = Vec::new();
    while n != 0 {
        let mut r = n % kk;
        if r < 0 { r += kk; }
        out.push(r as usize);
        n = (n - r) / -kk;
    }
    out.reverse();
    out
}

/// Value of an msd-first word in base `-k`, or `None` on overflow / a digit
/// outside the alphabet.
pub fn value(w: &[usize], k: u32) -> Option<i64> {
    let kk = -(k as i64);
    let mut v: i64 = 0;
    for &d in w {
        if d >= k as usize { return None; }
        v = v.checked_mul(kk)?.checked_add(d as i64)?;
    }
    Some(v)
}

/// `Some(k)` if `name` denotes base `-k`: `neg_2`, `msd_neg_2`, `lsd_neg_10`, ...
pub fn base_of(name: &str) -> Option<u32> {
    let s = name.strip_prefix("msd_").or_else(|| name.strip_prefix("lsd_")).unwrap_or(name);
    let d = s.strip_prefix("neg_")?;
    let k: u32 = d.parse().ok()?;
    if k >= 2 { Some(k) } else { None }
}

// ------------------------------------------------------------------ the automata, as text

/// Walnut "Custom Bases" text for the validity automaton of base `-k`.
pub fn validity_txt(k: u32) -> String {
    let mut s = format!("# msd_neg_{k}: base -{k}, digits {{0..{}}}, place values (-{k})^i.\n\
# Every word is a representation, so the language is all of {{0..{}}}*.\n\n{{{}}}\n\n0 1\n",
        k - 1, k - 1, (0..k).map(|d| d.to_string()).collect::<Vec<_>>().join(","));
    for d in 0..k { s += &format!("{d} -> 0\n"); }
    s
}

/// Walnut "Custom Bases" text for the msd adder of base `-k` (3 states).
pub fn addition_txt(k: u32) -> String {
    let n = k as i64;
    let mut s = format!("# msd_neg_{k}_addition: (x,y,z) with x + y = z in base -{k}.\n\
# State = t, the value of the prefix read so far in the recurrence\n\
#   t_p = (x_p + y_p - z_p) - {k} * t_(p+1),   t_L = 0,   accept iff t_0 = 0.\n\
# state 0 = t 0 (accepting), state 1 = t -1, state 2 = t +1.\n\n{{{a}}} {{{a}}} {{{a}}}\n",
        a = (0..k).map(|d| d.to_string()).collect::<Vec<_>>().join(","));
    let code = |t: i64| -> Option<usize> { match t { 0 => Some(0), -1 => Some(1), 1 => Some(2), _ => None } };
    for (q, c) in [(0usize, 0i64), (1, -1), (2, 1)] {
        s += &format!("\n{} {}\n", q, if q == 0 { 1 } else { 0 });
        for x in 0..n { for y in 0..n { for z in 0..n {
            if let Some(t) = code(x + y - z - n * c) {
                s += &format!("{x} {y} {z} -> {t}\n");
            }
        }}}
    }
    s
}

/// Walnut "Custom Bases" text for the msd comparison of base `-k` (3 states).
pub fn less_than_txt(k: u32) -> String {
    let n = k as i64;
    let mut s = format!("# msd_neg_{k}_less_than: (x,y) with x < y in base -{k}.\n\
# The first differing digit decides, and every later digit flips the verdict,\n\
# because consecutive place values have opposite signs.\n\
# state 0 = equal so far, state 1 = x < y (accepting), state 2 = x > y.\n\n{{{a}}} {{{a}}}\n",
        a = (0..k).map(|d| d.to_string()).collect::<Vec<_>>().join(","));
    s += "\n0 0\n";
    for x in 0..n { for y in 0..n {
        s += &format!("{x} {y} -> {}\n", if x == y { 0 } else if x < y { 1 } else { 2 });
    }}
    s += "\n1 1\n";
    for x in 0..n { for y in 0..n { s += &format!("{x} {y} -> 2\n"); } }
    s += "\n2 0\n";
    for x in 0..n { for y in 0..n { s += &format!("{x} {y} -> 1\n"); } }
    s
}

/// The three automata of base `-k`, parsed and ready for `numsys::build_neg`.
pub fn parsed(k: u32) -> Result<(Parsed, Parsed, Parsed), String> {
    let v = crate::numsys::parse_walnut(&validity_txt(k), None)?;
    let a = crate::numsys::parse_walnut(&addition_txt(k), Some(k as usize))?;
    let l = crate::numsys::parse_walnut(&less_than_txt(k), Some(k as usize))?;
    Ok((v, a, l))
}

/// Write `msd_neg_k.txt`, `msd_neg_k_addition.txt` and `msd_neg_k_less_than.txt`
/// into `dir`, returning the paths written.
pub fn write_files(k: u32, dir: &std::path::Path) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
    let mut out = Vec::new();
    for (suffix, text) in [("", validity_txt(k)),
                           ("_addition", addition_txt(k)),
                           ("_less_than", less_than_txt(k))] {
        let p = dir.join(format!("msd_neg_{k}{suffix}.txt"));
        std::fs::write(&p, text).map_err(|e| format!("{}: {}", p.display(), e))?;
        out.push(p.display().to_string());
    }
    Ok(out)
}
