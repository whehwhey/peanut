# The symbolic (BDD-backed, MONA-style) strategy — what it is and where it pays

> **Measured with the pre-2026-08-19 defaults.** The flags named here are no longer
> all opt-in: `AM_PAR` defaults to `min(8, cores-2)` and `AM_ANTICHAIN` defaults to on.
> Re-measured head-to-head under the current defaults: `bench/SPEED-ROUND6.md`,
> "Final defaults".

`engine/src/symbolic.rs`, selected per session with `AM_STRATEGY=bdd` (always try it) or
`AM_STRATEGY=auto` (try it only when the alphabet is big enough to pay for itself).
**Default off**: with `AM_STRATEGY` unset the engine takes exactly the path it took
before this file existed.

## What it does

`Dfa::exists` projects one track away and determinizes the resulting NFA. The explicit
construction enumerates, for every subset and every one of the `k^tracks` letters, the
union of the members' targets; the cost of a subset is `alpha * |subset| * k` set
operations, and `alpha` grows exponentially in the number of variables — a
five-variable formula over base 4 already has 1024 letters, most of which behave
identically.

MONA's answer is to hold each state's transition *function* as a decision diagram over
the letter, so letters that behave alike are handled once. This module does that with
the alphabet this engine actually has (tuples of base-`k` digits, not bits): a reduced,
hash-consed **multi-terminal k-ary decision diagram**, one level per track, `k` children
per node, terminals carrying interned sets of source states.

* **Projection** happens while a row is built: the terminal of letter `a` in state `s`'s
  diagram is the interned set `{delta(s, a with digit d in the projected track) : d<k}`.
* **Subset construction** computes a whole subset's transition function as one diagram
  union, memoised on node pairs and on 64-state chunks of the subset, so the cost tracks
  the number of *distinct behaviours* rather than `alpha`. Successors are read off as the
  diagram's leaves, in first-occurrence order, which is exactly the order the explicit
  forward construction discovers them in.
* **Minimisation** is Moore refinement run on the diagrams: recolouring a row's terminals
  and hash-consing the result is a canonical signature for the entire row, computed in one
  memoised walk instead of an `alpha`-sized signature vector per state.
* The result is expanded back to an explicit minimal `Dfa` at the end, when it is small.

Rows are built lazily (a source state whose row is never needed is never expanded) and
the whole pass is capped: at `AM_CAP0` subsets (50 000 by default, the same cap the
explicit ladder uses for its first forward attempt) and at `AM_BDD_NODES` diagram nodes.
On a cap the function returns `None` and `Dfa::exists` falls through to the existing
ladder — the symbolic pass is a *first rung*, not a replacement.

`AM_STRATEGY=auto` puts one more rung in front: a **probe**, a short explicit forward
subset construction (the same `Nfa::determinize_capped` the ladder uses) whose cap is
scaled as `200_000/alpha` so it never does more than ~200k subset-by-letter cells of
work. If the probe finishes, that is the answer and no diagram is ever built; only when
it overflows — i.e. only on the expensive projections — does the symbolic pass run. That
is what keeps `auto` from paying the diagram's constant factor on the hundreds of trivial
projections a formula compiles through.

No new crate dependency: the diagram package is ~150 lines in the same file (unique table
by open addressing over flat `Vec<u32>` node arrays), so `engine/Cargo.toml` still has an
empty `[dependencies]`.

## Flags

```text
AM_STRATEGY=bdd|auto|off   off (default) = the explicit ladder, untouched
AM_BDD_CAP=N               subset cap before giving up   (default AM_CAP0 = 50 000)
AM_BDD_NODES=N             diagram node cap              (default 30 000 000)
AM_BDD_MINALPHA=N          `auto` fires only at alphabets this big (default 16)
AM_BDD_PROBE=N             `auto`'s explicit probe cap  (default 200_000/alpha, clamped)
AM_BDD_DEBUG=1             one stderr line per projection
```


## Method

`bench/bdd_bench.py` runs a fixed case list through **one binary** three times — no
flags (the explicit ladder), `AM_STRATEGY=bdd`, `AM_STRATEGY=auto` — back to back per
case, so a machine that is busy with other jobs costs all three configurations the same.
Recorded per case: the engine's own `ms=` (summed over the predicates the case builds;
the wall clock also contains `explore/engine.py`'s RAM-admission wait, which on a loaded
machine dominates and is not the engine's time), the engine's own `mem` peak in MB, the
minimal state count of every predicate, and a **canonical fingerprint** of every exported
automaton — a breadth-first relabelling from state 0 taking letters in index order, which
is a canonical form for a minimal DFA. Equal fingerprints therefore mean "same language",
not merely "same number of states".

    python3 bench/bdd_bench.py abc   out.json all      # run all three configurations
    python3 bench/bdd_bench.py table out.json          # print the table below

One field is *not* comparable across configurations: the `peak=` a query prints counts
subsets of whichever construction ran, and the symbolic pass explores a compressed state
space (and, when it gives up, never records what it explored). State counts, verdicts and
the canonical fingerprints are the correctness signal; `peak=` is telemetry.

Machine: the same 18-core / 24 GB Mac used for `bench/STRATEGY-RESULTS.md`, but **under
concurrent load from other benchmark jobs** (load average 8–13 throughout). Absolute
seconds here are therefore 1.2–1.4x higher than the quiet-machine figures in
STRATEGY-RESULTS.md (prism-1 48.7 s here vs 38.6 s there); the three columns are directly
comparable to each other, which is what the ratios report. `AM_MEM_MB=6144`, 240 s
ceiling.

## Correctness

**75/75 cases: identical minimal state counts and identical canonical automata across all
three configurations.** Not one disagreement, in either digit order.

### Fuzz battery — `tools/fuzz_bdd.py`

    python3 tools/fuzz_bdd.py 8          # writes results/fuzz_bdd.json

110 random admissible k-automatic sequences (the PRISM draw and admissibility check
`tools/fuzz_walnut.py` uses, 14 (k, m, coding) cells, `k in {2,3}`, `m in {2..5}`) x the
10 formula templates of the Walnut differential (squarefree, cubefree, overlap,
palindrome, border, FE-with-l-bound, right-special, recurrence, eventually-periodic,
quantifier-alternation-with-multiplication) = **1100 closed formulas**, plus the FE panel
as an *open* three-variable predicate in both formulations (`FE` and `FE2`) and both digit
orders = 56 more, so **1156 jobs**. Each job is run under four engines:

    ref    engine/target/release/peanut_old   the engine as it was, no flags
    off    the current binary, no flags       the explicit ladder
    bdd    the current binary, AM_STRATEGY=bdd
    auto   the current binary, AM_STRATEGY=auto

4624 engine runs. **1153 jobs were answered by all four: identical TRUE/FALSE verdict and
identical minimal state count in every one.** The three unanswered jobs are the 150 s
timeout hitting one heavy `border` instance and one `single5/lsd` FE (30 638 states) —
resource exhaustion, not disagreement; the only row flagged is one where the *default*
path timed out and all three of `ref`/`bdd`/`auto` answered TRUE.

    verdicts   ref: 548 TRUE / 552 FALSE / 54 open / 2 no-answer
               off: 547 TRUE / 552 FALSE / 54 open / 3 no-answer
               bdd: 548 TRUE / 552 FALSE / 54 open / 2 no-answer
               auto:548 TRUE / 552 FALSE / 54 open / 2 no-answer
    disagreements: 0


## Results

Seconds are the engine's own `ms=`; MB is the engine's own live-bytes peak. `x bdd` and
`x auto` are speedups over the default (>1 = symbolic faster).

### FE panel

    case                     base s    bdd s   auto s   x bdd x auto       MB     MB     MB   states
    FE/thue-morse              0.01     0.01     0.01    1.00   1.00        0      0      0   15
    FE/period-doubling         0.01     0.01     0.01    1.00   1.00        0      0      0   8
    FE/rudin-shapiro           0.14     0.28     0.14    0.51   1.00       88     34     88   68
    FE/paperfolding            0.00     0.00     0.00    1.00   1.00        0      1      0   44
    FE/cantor                  0.00     0.01     0.00    0.10   1.00        0      0      0   17
    FE/mephisto                0.00     0.00     0.00    2.00   1.00        1      0      1   14
    FE/prism-1                48.69    47.52    45.31    1.02   1.07      312    312    312   467
    FE/prism-a                 0.01     0.01     0.01    1.00   1.00        0      0      0   24
    FE/prism-d                 0.50     0.27     0.27    1.85   1.88      132     33     33   82
    FE/single3                 0.01     0.05     0.01    0.26   1.09        6     22      6   190
    FE/single4                 0.18     0.21     0.18    0.86   1.01       10     27     10   698
    FE/single5                 2.61     2.66     2.67    0.98   0.98       30     38     30   1877
    FE/single6                26.88    27.62    26.63    0.97   1.01      169    169    169   3971
    FE/champion-m5             0.04     0.10     0.04    0.42   1.00        6     20      6   199
    FE/k3m3-artefact-a         0.05     0.10     0.10    0.46   0.46        7     26     26   216
    FE/k3m3-artefact-b         0.03     0.12     0.12    0.25   0.25        6     37     37   71
    FE/tail-a                172.84   181.45   180.25    0.95   0.96     1480   1480   1480   1165
    FE/tail-b                223.86   220.64   214.41    1.01   1.04     1249   1249   1249   1000
    FE/tail-c                240.18   240.13   240.04    1.00   1.00        -      -      -   -

### FE2

    case                     base s    bdd s   auto s   x bdd x auto       MB     MB     MB   states
    FE2/thue-morse             0.00     0.00     0.00    3.00   1.00        2      0      2   15
    FE2/period-doubling        0.00     0.01     0.00    0.10   1.00        0      0      0   8
    FE2/rudin-shapiro          0.01     0.09     0.01    0.15   0.93        5     33      5   68
    FE2/paperfolding           0.18     0.02     0.18    8.48   0.99      123     10    123   44
    FE2/cantor                 0.01     0.00     0.01    2.33   1.00        2      1      2   17
    FE2/mephisto               0.04     0.01     0.04    6.67   1.00       14      2     14   14
    FE2/prism-a                0.01     0.01     0.01    2.20   0.85        8      1      8   24
    FE2/prism-d                4.09     0.23     0.23   18.03  17.95     1105     37     37   82
    FE2/single3                0.02     0.05     0.02    0.37   1.05        6     21      6   190
    FE2/single4                0.39     0.42     0.39    0.93   1.01       10     30     10   698
    FE2/k3m3-artefact-a        0.10     0.13     0.13    0.83   0.81        6     21     21   216
    FE2/k3m3-artefact-b        0.06     0.09     0.08    0.72   0.74        5     19     19   71
    FE2/champion-m5            0.10     0.14     0.10    0.74   0.99        7     21      7   199

### pelt

    case                     base s    bdd s   auto s   x bdd x auto       MB     MB     MB   states
    pelt/thue-morse            0.01     0.01     0.01    1.00   1.00        0      0      0   12,15,7,7
    pelt/period-doubling       0.01     0.01     0.01    1.00   1.00        0      0      0   8,8,4,5
    pelt/cantor                0.01     0.00     0.01    1.25   1.00        1      1      1   15,17,6,9
    pelt/paperfolding          0.01     0.00     0.01    1.25   1.00        0      1      0   34,44,7,20
    pelt/rudin-shapiro         0.13     0.14     0.13    0.91   0.99       88     26     88   49,68,12,20
    pelt/prism-d               0.51     0.23     0.24    2.26   2.14      132     34     34   87,82,19,29
    pelt/mephisto              0.01     0.00     0.01    1.50   1.20        1      0      1   11,14,7,8
    pelt/single3               0.17     0.17     0.17    1.05   1.00       35     81     31   271,190,39,84

### many-variable

    case                     base s    bdd s   auto s   x bdd x auto       MB     MB     MB   states
    SQ/thue-morse              0.01     0.01     0.01    1.00   1.00        0      0      0   6
    CU/thue-morse              0.01     0.01     0.01    1.00   1.00        0      0      0   1
    OCC/thue-morse             0.01     0.01     0.01    1.00   1.00        0      0      0   15
    RSP/thue-morse             0.01     0.01     0.01    1.00   1.00        0      0      0   7
    PAL/thue-morse             0.01     0.01     0.01    1.00   1.00        0      0      0   16
    SQ/cantor                  0.01     0.01     0.01    1.00   1.00        0      0      0   10
    CU/cantor                  0.00     0.00     0.00    1.00   1.00        0      0      0   11
    OCC/cantor                 0.00     0.01     0.01    0.10   0.10        0      0      0   17
    RSP/cantor                 0.00     0.01     0.00    0.10   1.00        0      0      0   9
    PAL/cantor                 0.01     0.01     0.01    1.00   1.00        0      0      0   12
    SQ/prism-d                 0.01     0.02     0.01    0.65   1.00        7      5      7   22
    CU/prism-d                 0.01     0.01     0.01    0.92   0.92        7      4      7   9
    OCC/prism-d                0.47     0.23     0.23    2.03   2.07      132     33     33   82
    RSP/prism-d                0.47     0.23     0.23    2.06   2.07      132     33     33   29
    PAL/prism-d                0.01     0.01     0.00    0.56   1.25        2      4      2   30
    SQ/single3                 0.01     0.20     0.01    0.04   0.73        7     69      7   23
    CU/single3                 0.02     0.25     0.01    0.06   1.14        9     99      9   12
    OCC/single3                0.01     0.04     0.01    0.29   0.92        6     22      6   190
    RSP/single3                0.01     0.04     0.01    0.30   1.09        6     22      6   84
    PAL/single3                0.01     0.34     0.01    0.03   1.00        6     88      6   73
    SQ/prism-1                 0.28     0.39     0.40    0.72   0.70       32    170    170   28
    CU/prism-1                 0.40     0.59     0.61    0.67   0.65       46    450    450   24
    OCC/prism-1               39.50    41.34    41.48    0.96   0.95      312    312    312   467
    RSP/prism-1               40.80    41.35    42.52    0.99   0.96      312    312    312   81
    PAL/prism-1                0.09     0.21     0.23    0.44   0.40       25    139    139   39
    SQ/k3m3-artefact-a         0.01     0.10     0.01    0.14   0.93        7     31      7   27
    CU/k3m3-artefact-a         0.02     0.10     0.02    0.20   0.95        8     46      8   16
    OCC/k3m3-artefact-a        0.04     0.10     0.10    0.45   0.45        7     26     26   216
    RSP/k3m3-artefact-a        0.05     0.10     0.10    0.50   0.48        7     26     26   80
    PAL/k3m3-artefact-a        0.03     0.15     0.03    0.19   0.94        6     43      6   45
    SQ/champion-m5             0.02     0.23     0.03    0.10   0.96        7     42      7   22
    CU/champion-m5             1.02     0.17     1.01    5.95   1.00     1151     46   1151   18
    OCC/champion-m5            0.04     0.10     0.04    0.38   0.95        6     20      6   199
    RSP/champion-m5            0.04     0.10     0.04    0.42   1.00        6     20      6   60
    PAL/champion-m5            0.05     0.07     0.05    0.73   1.02       41     22     41   41

mismatches: 0 of 75


## Where it wins

* **Large product alphabets with a forward construction that is expensive but bounded.**
  The clearest case is Khodier's `FE2` reformulation
  `A u,v. (u>=i & u<i+n & u+j=v+i) => T[u]=T[v]` — five variables, so `k^5` letters
  before projection — on a base-3 sequence: `FE2/prism-d` **4.09 s / 1105 MB -> 0.23 s /
  37 MB, 18x faster and 30x smaller**, same 82-state answer. `FE2/paperfolding` 8.5x,
  `FE2/mephisto` 6.7x, `FE2/thue-morse` 3.0x, `FE2/cantor` 2.3x, `FE2/prism-a` 2.2x.
* **The Peltomaki `extRS2` stack** (`factorEq` -> `isRS` -> `extRS2` -> `E n`) on a base-3
  sequence: `pelt/prism-d` 0.51 s -> 0.23 s (2.3x), 132 MB -> 34 MB.
* **Three-free-variable queries over base-3/4 sequences**: `OCC/prism-d` and `RSP/prism-d`
  2.0x (132 MB -> 33 MB), `FE/prism-d` 1.9x, `CU/champion-m5` **1.02 s / 1151 MB -> 0.17 s
  / 46 MB (6x faster, 25x smaller)**.
* **Memory, even where time is a wash.** `FE/rudin-shapiro` 88 MB -> 34 MB at 0.5x the
  speed; the diagram holds one node per *distinct behaviour* instead of one `u32` per
  (subset, letter) cell, so the intermediate automaton is what shrinks.

The pattern: the win is the ratio between the alphabet size and the number of distinct
transition behaviours per state. Base 3 and 4 with four or five tracks give 81–1024
letters and a few dozen behaviours; base 2 with three tracks gives 8 letters and up to 8
behaviours, and there is nothing to compress.

## Where it loses

* **Small alphabets.** Every base-2 case with `alpha <= 16` is 2–30x *slower* under
  `bdd` in relative terms (`SQ/single3` 0.01 s -> 0.20 s, `PAL/single3` 0.01 s -> 0.34 s):
  hashing diagram nodes costs more than the explicit inner loop it replaces. The absolute
  loss is tens to hundreds of milliseconds; `auto` removes almost all of it (0.73–1.14x).
* **Projections that need millions of subsets.** The symbolic pass is a *forward*
  construction, and the number of subsets is a property of the NFA, not of the
  representation. On the equality-of-factors panel (`prism-1`, `single3..6`, `tail-a/b/c`)
  the winning strategy is reverse-first (Brzozowski), so the symbolic pass hits its cap
  and hands back to the ladder: those rows are unchanged to within the noise of a loaded
  machine (0.95–1.07x), and `FE/tail-c` still needs `learnfe` under every configuration.
  A symbolic *reversal* (transposing the diagram-encoded relation, so Brzozowski itself
  could run symbolically) is the obvious next step and is not implemented.
* **`auto`'s remaining bad rows** are the ones where the probe fails and the symbolic pass
  then also fails: `FE/k3m3-artefact-b` 0.03 s -> 0.12 s, `PAL/prism-1` 0.09 s -> 0.23 s.
  Worst observed `auto` regression: 0.25x on a 30 ms case; worst absolute: +140 ms.

## Recommendation

`AM_STRATEGY=auto` is the setting worth having: it is within noise of the default
everywhere it does not help, keeps the 18x/30x FE2 win and the 2x base-3 wins, and needs
no per-query choice. It stays **off by default** until someone decides otherwise —
this note is the evidence, not the decision.
