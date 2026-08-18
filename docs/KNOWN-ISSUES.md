# KNOWN ISSUES

What is wrong, what is off by default and why, and what a reader should not trust.
Started 2026-08-19 with the speed round (`bench/SPEED-ROUND6.md`); anything found by a
correctness gate belongs here, whether or not it was fixed.

## 1. Algorithms gated in the speed round

Four algorithms were added behind default-off flags (`det_par.rs` / `AM_PAR`,
`symbolic.rs` / `AM_STRATEGY`, `antichain.rs` / `AM_ANTICHAIN`, the generalised learner
in `learn.rs`) and gated against the pre-round binary, against each other, and against
Walnut.

**No verdict or minimal-state-count disagreement was found for any of them** — 1120
config-vs-default fuzz-diff pairs, 280 pairs against `peanut_old`, 100 pairs against
Walnut 8-dev, 57 `learn`-vs-`let` panel checks, plus each builder's own gate (912 runs
with `AM_FAST_VERIFY`, 4624 runs across four engines for the symbolic core, 2031 scripts
for the antichain). So nothing here is disabled *because of a disagreement*; the two
that stay off stay off for cost, not correctness:

* **`AM_STRATEGY` (symbolic/BDD core) — off.** 2x-4x slower than the explicit ladder on
  small alphabets (base 2, three tracks), up to 18x faster on large ones (five tracks
  over base 3). No alphabet threshold separates the two in the measured data — base-3
  three-track cases land on both sides (`FE/prism-d` 1.9x faster, `FE/k3m3-artefact-a`
  2x slower) — so `AM_STRATEGY=auto` remains a per-session choice rather than a default.
  Evidence: `bench/BDD-RESULTS.md`.
* **`AM_LAZY_CLOSED` — off.** Correct and gated (0 mismatches over 228 pairs), but worth
  0-5 % and usually under 1 %: by the time the last variable is projected the automaton
  is already small. With the antichain now on by default it competes for the same closed
  sentences. Evidence: `bench/DETPAR-RESULTS.md`.
* **`AM_AC_SIM` (simulation-refined subsumption) — off.** It does collapse the antichain
  as the literature says (691 elements -> 1 on tail-a), and the naive greatest-fixpoint
  that computes the preorder costs 17x the search it saves (20 ms -> 338 ms). A
  partition-based simulation would change that arithmetic; nobody has written one.
  Evidence: `bench/ANTICHAIN-RESULTS.md`.

## 2. Differential testing against Walnut must be serial

Running several Walnut JVMs at once against one `walnut7/` checkout produces **wrong
answers**, not just slow ones: the round-6 gate saw a `palindrome` sentence on the
periodic word `1010…` answered TRUE by a Walnut run inside a 3-way
`ThreadPoolExecutor`, and FALSE by the identical script run serially — FALSE also being
the answer from Peanut and from an independent brute force. The JVMs share
`walnut7/Session`.

**Consequence:** `tools/fuzz_walnut.py`, `tools/walnut_suite.py` and any other
Peanut-vs-Walnut comparison must run the Java side one process at a time. A
disagreement produced by a concurrent Walnut run is not evidence of anything. This is a
harness hazard, not a Peanut or a Walnut bug, and it is the reason `bench/SPEED-ROUND6.md`
§1(c) records its Walnut phase as serial.

## 3. `AM_PAR` is per process, and the default is per process

The 2026-08-19 default `AM_PAR = min(8, cores-2)` is chosen for **one engine at a time**.
A harness that runs N engines concurrently (`explore/engine.py`'s `pool`) now
oversubscribes the machine by roughly a factor of N, which inflates wall-clock times and
makes benchmark rows incomparable. Set `AM_PAR=1` (or a small value) in the environment
of any multi-engine sweep; every benchmark table in `bench/` that quotes seconds was
taken one process at a time.

## 4. Walnut-compatibility differences (pre-existing)

Unchanged by the speed round, restated here so there is one list. Detail and status for
each: `docs/WALNUT-COMPAT.md` §7.

* `pf_theorem5.txt`'s last `appearance` command: Walnut returns the empty automaton,
  Peanut a 4-state one. Cause is the padding convention on a word-automaton track whose
  alphabet is a raw integer set (`{-1,1}`), which Walnut treats as non-numeric and
  Peanut does not. **Limitation**, 1 command of 247.
* Mixed number systems in one predicate (`?msd_3` and `?msd_10` in the same formula) are
  refused: a Peanut `Dfa` has one base for all tracks. **Structural**, 2 commands.
* Identifiers beginning with `A`/`E`/`I` (`$Inc`, `$IsLargeFibo`): Peanut accepts them,
  Walnut's tokenizer does not. Peanut is the more permissive side. **Intentional**,
  3 commands.

The suite otherwise agrees on 223 of 247 commands, with 18 scripts that fail in Walnut
itself; that count is a regression target and must not drop.

## 5. Cases no configuration answers

* **`tail-c`, `let FE` by the direct construction.** It finishes under the 2026-08-19
  defaults — 191.1 s, 2818 MB, 1382 states — and does **not** finish under the old ones
  (the reference path is killed at 6 GB). That resolves a contradiction between two
  earlier notes in this repo: `bench/DETPAR-RESULTS.md` said it finished with `AM_PAR=8`,
  `bench/SPEED-ROUND6.md` §3 said it "dies at 6 GB under every configuration"; the first
  is right and the second was measuring the flag combination without the flat core.
  It is still 2818 MB of a 6144 MB budget on a case whose answer `learnfe` reaches in
  16.1 s and 50 MB, so `learnfe FE` / `learn NAME fe` remains the right tool for it.
* **Some direct `let` constructions of REV/PERIOD/BORDER** on the hard panel sequences
  (`single6`, `tail-a`, `tail-b`, `tail-c`, `prism-1`/period): 12 of 57 exhaust 150 s or
  6144 MB. `learn` answers all of them. This is the asymmetry the learner exists for,
  not a defect; it does mean "check it against the direct construction" is not available
  as a correctness argument on exactly those rows — the recurrence check inside `learn`
  is (`docs/LEARN.md` §1).

## 6. Documentation that predates the 2026-08-19 defaults

Every seconds column in `bench/README.md`, `bench/STRATEGY-RESULTS.md`,
`bench/DETPAR-RESULTS.md`, `bench/ANTICHAIN-RESULTS.md`, `bench/BDD-RESULTS.md`,
`docs/TARGET1*.md` and `docs/LEARNFE.md` was measured under the **old** defaults
(single-threaded determinization, no antichain). Those pages now carry a banner
saying so where they are head-to-head tables; the numbers were not retaken, because
the state counts, the memory figures and the conclusions do not change and the
re-measured head-to-head lives in one place: `bench/SPEED-ROUND6.md`, "Final defaults".

## 7. The flat determinization core is slower on small closed sentences

`AM_PAR` (on by default since 2026-08-19) implies the flat core in
`engine/src/det_par.rs`. On a **closed** sentence whose projections are all small, that
core is slower than `dfa.rs`'s reference core — the flat one builds a `FlatNfa` and a
transposed successor buffer for every projection, which does not pay for itself until
the projection is big.

Reproduction (quiet machine, `AM_MEM_MB=6144`, msd, prism-1 = panel entry
`def T 4 6 0 0305 4555 2321 0514 1023 4300 102202`):

```
learnfe FE
? E i,n. n>=1 & $FE(i,i+n,2*n)

AM_PAR=1              (reference core)             0.128 s
AM_PAR=1 AM_FAST=1    (flat core, one thread)      0.383 s
(no environment)      (flat core, eight threads)   0.325 s
```

Measured across six sequences and four closed sentences (`bench/SPEED-ROUND6.md` §4 of
"Final defaults"), the effect is 0.38x-0.68x on the pure `E`-block sentence and
0.64x-0.66x on prism-1's two `A..E..` sentences; every other closed row is 1.3x-246x
*faster*. It is bounded at +200 ms on the worst case seen and it is not the antichain
(the 2x2 attribution in that section separates them).

Three mitigations were implemented and measured on 2026-08-19 — a minimum block size
before rayon is used, a minimum `nstates * alpha` before the flat core is used at all,
and a lower cap on `Dfa::product`'s direct-indexed pair table — and **none of them
changed this row**, so none were shipped. The cost is somewhere else inside the flat
core and finding it needs a profile, not a threshold. Until then: `AM_PAR=1` is the
escape hatch for a workload that is entirely small closed sentences.
