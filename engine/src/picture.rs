//! `pic` — the two-dimensional view of a predicate.
//!
//! `pic NAME W H [i0 j0 [scale]]` walks a rectangle of the (i, j) plane and prints
//! one cell value per point, as a single line.  Nothing is *built*: for a predicate
//! the cell is decided by running its already-compiled DFA on the two-track base-`k`
//! digit string of the pair, in the active digit order; for the sequence's own DFAO
//! the cell is `T[i+j]`, read straight off the morphism automaton.  So the cost is
//! `W*H*(digits)` transitions and no memory beyond the picture itself — a 1024x1024
//! picture of a predicate is the same automaton the `?` command would answer with,
//! asked a million times.
//!
//! Axes.  `W` is the width in cells (the `j` axis, across); `H` is the height (the
//! `i` axis, down); `scale` is the step in both.  Cell `(r, c)` of the output is the
//! point `i = i0 + r*scale`, `j = j0 + c*scale`, so the printed rows are `i`-major
//! and read like an image: `H` rows of `W` cells.  This is the same orientation as
//! `fe_map` (i down, j across).

use crate::dfa;
use crate::dfao::Dfao;
use crate::logic::Defs;
use crate::clock::Instant;

/// Hard cap on the number of cells one `pic` may evaluate.
pub const MAX_CELLS: u64 = 1 << 20;

/// Base-`k` digits of `n`, padded to exactly `len` places, in the order the active
/// digit convention reads them (msd: most significant first; lsd: least first).
fn digits(n: u64, k: u64, len: usize) -> Vec<usize> {
    let mut d = vec![0usize; len];
    let mut m = n;
    for p in 0..len {                       // p = place value k^p, least significant first
        let dig = (m % k) as usize;
        m /= k;
        d[if dfa::is_lsd() { p } else { len - 1 - p }] = dig;
    }
    d
}

/// Number of base-`k` digits needed to write `n` (at least 1).
fn width_of(n: u64, k: u64) -> usize {
    let mut len = 1usize;
    let mut cap = k;
    while cap <= n { cap = cap.saturating_mul(k); len += 1; }
    len
}

#[inline]
fn hex(v: u8) -> char { std::char::from_digit(v as u32 & 0xf, 16).unwrap_or('f') }

/// One row of cells as either `W` hex digits, or — when that is longer — a
/// run-length form `~<hex><count>.<hex><count>…`.  The leading `~` is the marker;
/// a row never otherwise starts with one.
fn encode_row(vals: &[u8]) -> String {
    let plain: String = vals.iter().map(|&v| hex(v)).collect();
    let mut rle = String::from("~");
    let mut i = 0usize;
    while i < vals.len() {
        let mut j = i + 1;
        while j < vals.len() && vals[j] == vals[i] { j += 1; }
        if i > 0 { rle.push('.'); }
        rle.push(hex(vals[i]));
        rle.push_str(&(j - i).to_string());
        i = j;
    }
    if rle.len() < plain.len() { rle } else { plain }
}

/// `pic NAME W H [i0 j0 [scale]]` — returns the exact line to print (`PIC …` or `ERR …`).
pub fn cmd(cur: Option<&Dfao>, defs: &Defs, rest: &str) -> String {
    const USAGE: &str = "ERR usage: pic NAME W H [i0 j0 [scale]]";
    let Some(d) = cur else { return "ERR no sequence".into() };
    let p: Vec<&str> = rest.split_whitespace().collect();
    if p.len() < 3 { return USAGE.into(); }
    let name = p[0];
    let (Ok(w), Ok(h)) = (p[1].parse::<u64>(), p[2].parse::<u64>()) else { return USAGE.into() };
    if w == 0 || h == 0 { return "ERR pic: W and H must be positive".into(); }
    if w * h > MAX_CELLS {
        return format!("ERR pic: {} cells exceeds the cap of {}", w * h, MAX_CELLS);
    }
    let num = |i: usize, dflt: u64| p.get(i).and_then(|x| x.parse::<u64>().ok()).unwrap_or(dflt);
    let (i0, j0) = (num(3, 0), num(4, 0));
    let scale = num(5, 1).max(1);
    let (w, h) = (w as usize, h as usize);
    let k = d.k as u64;

    let imax = i0 + (h as u64 - 1) * scale;
    let jmax = j0 + (w as u64 - 1) * scale;
    let len = width_of(imax.max(jmax), k);
    let idig: Vec<Vec<usize>> = (0..h).map(|r| digits(i0 + r as u64 * scale, k, len)).collect();
    let jdig: Vec<Vec<usize>> = (0..w).map(|c| digits(j0 + c as u64 * scale, k, len)).collect();

    let t0 = Instant::now();
    let mut rows: Vec<String> = Vec::with_capacity(h);
    let mut vals = vec![0u8; w];
    let maxval;

    if name == "T" || name == d.name {
        // The sequence itself has one variable, so the two-dimensional reading of it
        // is the addition table T[i+j]: the cell is an output letter, not a truth value.
        for r in 0..h {
            let i = i0 + r as u64 * scale;
            for c in 0..w { vals[c] = d.at(i + j0 + c as u64 * scale); }
            rows.push(encode_row(&vals));
        }
        maxval = d.out_alphabet().last().copied().unwrap_or(0);
    } else {
        let Some((params, a)) = defs.get(name) else {
            return format!("ERR pic: no such predicate {:?} (have: T{}{})", name,
                           if defs.is_empty() { "" } else { ", " },
                           defs.keys().cloned().collect::<Vec<_>>().join(", "));
        };
        if a.vars.len() != 2 {
            return format!("ERR pic: {} has {} free variables [{}], need exactly 2",
                           name, a.vars.len(), a.vars.join(","));
        }
        // The picture's axes follow the *declared* parameter order, not the automaton's
        // sorted variable order: pic P is P(i, j) with i down and j across.
        let axis = |v: &String| a.vars.iter().position(|x| x == v);
        let (Some(ti), Some(tj)) = (
            axis(params.first().unwrap_or(&a.vars[0])),
            axis(params.get(1).unwrap_or(&a.vars[1])),
        ) else { return format!("ERR pic: {} parameters do not match its variables", name) };
        if ti == tj { return format!("ERR pic: {} uses one variable twice", name); }
        let (mi, mj) = (d.k.pow(ti as u32), d.k.pow(tj as u32));
        for r in 0..h {
            let ir = &idig[r];
            for c in 0..w {
                let jc = &jdig[c];
                let mut s = 0usize;
                for q in 0..len { s = a.t(s, ir[q] * mi + jc[q] * mj); }
                vals[c] = a.accept[s] as u8;
            }
            rows.push(encode_row(&vals));
        }
        maxval = 1;
    }

    format!("PIC {} {} i0={} j0={} scale={} vals={} ms={} rows={}",
            w, h, i0, j0, scale, maxval as usize + 1, t0.elapsed().as_millis(), rows.join(","))
}
