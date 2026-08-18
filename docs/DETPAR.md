# DETPAR — the flat determinization core (`engine/src/det_par.rs`)

**Since 2026-08-19 this is the default path** (`bench/SPEED-ROUND6.md`, "Final
defaults"): with no environment set, `AM_PAR = min(8, cores - 2)`, which routes
determinization, minimization and product onto the code in
`engine/src/det_par.rs` and runs the subset construction's frontier in parallel.
The measurements below were taken while it was still opt-in and use the flag names
as they were then; `AM_PAR=8` there is the default now.

`AM_PAR=1` restores the pre-2026-08-19 reference path; `AM_PAR=1 AM_FAST=1` the
serial flat core. `AM_LAZY_CLOSED=1` (still off by default) answers a closed
sentence's last quantifier by graph reachability instead of determinizing.
`AM_FAST_VERIFY=1` (off) runs the old and the new code side by side on every call
and asserts the two automata are equal element by element.

## What was slow

A 20-second `sample(1)` of the `tail-a` panel case (`let FE(i,j,l) A t. t<l =>
T[i+t]=T[j+t]`, k=2, m=7) put **99.2 % of wall clock inside
`Nfa::determinize_capped`** — not in hashing, not in the allocator, in the
subset construction's own inner loop. Three things were paying for that:

1. `Nfa::trans` is a `Vec<Vec<State>>`: one heap allocation per (state, symbol)
   pair, so the inner loop dereferences a cold pointer for every source state of
   every subset, and building the NFA for `exists` allocates
   `nstates * alpha` vectors before the construction even starts.
2. Every discovered subset was stored **twice** as an owned `Vec<u64>` — once as
   the `HashMap` key, once in the `order` vector — so N subsets cost 2N
   allocations and 2N live bitsets plus hashbrown's control bytes.
3. The successor bitsets were computed one symbol at a time, so the set bits of
   the current subset were scanned `alpha` times per step.

## What replaced it

- **`FlatNfa`** — transitions in one flat `u32` array. Either a fixed arity
  (`arity = k` for an existential projection, `arity = 1` for a DFA), which
  needs no offset table at all, or CSR with a `u32` offset array (used for
  reversal, which is built with a counting sort rather than per-edge `push`).
- **One arena for the subsets** — a flat `Vec<u64>` of `n * words`, indexed by an
  open-addressing table of `u32` ids (with the 64-bit hashes kept alongside so a
  rehash never re-reads the arena). One copy of each bitset, 12 bytes of index
  overhead per subset, no per-subset allocation at all.
- **Transposed inner loop** — the set bits of a subset are scanned once and all
  `alpha` successor bitsets are accumulated in the same pass, reading each source
  state's whole transition row contiguously. Used when `alpha * words` fits in
  32 MB of scratch, otherwise the old symbol-at-a-time loop.
- **Radix Moore minimization** — the refinement's signature table
  (`HashMap<Vec<u32>, u32>`, one owned key per state per round) is replaced by an
  LSD counting sort on `(colour, colour of each successor)`. Same partition, no
  allocation inside the loop.
- **Direct-indexed product** — when `|A| * |B| <= 2^26` the pair-to-id map is an
  array instead of a hash probe. Pairs are still discovered in BFS order.

## Why the output is identical, not merely equivalent

The subset construction discovers subsets in exactly the order the old one did
(queue order, symbols ascending), so state *i* of the new DFA **is** the same
subset as state *i* of the old one; the transition table and accept vector are
equal element by element, not up to renumbering. The parallel version keeps that
property: a block of the frontier is expanded and probed against the
(immutable during the phase) index in parallel, and the misses are then interned
**serially, in the order the serial construction would have reached them**. The
number of threads therefore cannot change a single state number.

`AM_FAST_VERIFY=1` asserts exactly this at run time: `Dfa::exists`,
`Dfa::minimize` and `Dfa::zero_closure` each build both results and compare
`nstates`, `alpha`, `vars`, `accept` and `trans`.

## `AM_LAZY_CLOSED=1`

When `exists` projects away the **last** variable, the remaining alphabet has one
letter and the only thing the engine ever reads off the result is
`zero_closure().accept[0]` — "is some `0^n`, `n >= 0`, accepted". That is plain
reachability from the initial to an accepting state in the projected NFA, so the
whole subset construction, the zero closure and the minimization can be skipped.
The 1-state automaton this returns is what `minimize` would have produced anyway:
a zero-closed language over a one-letter alphabet is either empty or all of `0*`.

This fires on the outermost quantifier of every closed sentence (`? ...`), and on
nothing else — the FE panel's `let FE(i,j,l) ...` keeps three free variables, so
it is unaffected.

## Testing

`tools/fuzz_engines.py` is the gate: it runs the reference binary
(`engine/target/release/peanut_old`, a copy taken before this work) and the new
binary under each flag over 200 random (sequence, formula) pairs drawn from the
same PRISM sequences and templates `tools/fuzz_walnut.py` uses, plus the FE panel
in both digit orders, and compares verdict, minimal state count and peak
intermediate size. See `bench/DETPAR-RESULTS.md` for the measured before/after.
