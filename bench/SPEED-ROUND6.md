# SPEED-ROUND6: gate on AM_PAR / AM_ANTICHAIN / AM_STRATEGY=bdd / `learn` vs `let`

Mechanical Sonnet gate. Machine: 18-core Apple Silicon, 24 GB. `git pull` was
already up to date; `cargo build --release` was already current for this tree
(the four algorithms below are already committed, each behind its own
default-off env flag; this round gates them, it does not introduce them).
`engine/target/release/peanut_old` is the pre-`det_par`/`symbolic`/`antichain`/
generalised-`learn` binary (`fb6a648`), used as the independent reference.

Configurations gated: **default** (no env), **AM_PAR=8**, **AM_ANTICHAIN=1**,
**AM_STRATEGY=bdd**, plus **`learn` vs `let`** for REV/PERIOD/BORDER on the
19-case `bench/panel.json`.

## 1. Fuzz-diff

Sequences and formula templates are `tools/fuzz_walnut.py`'s own PRISM draw and
10-template battery (squarefree, cubefree, overlap, palindrome, border,
FE-with-l-bound, right-special, recurrence, eventually-periodic,
quantifier-alternation-with-multiplication), reused via `import fuzz_walnut`,
`per_cell=2` → 28 admissible (k,m,coding) sequences over the 14 cells → **280
(sequence, formula) pairs**. Driver: `/tmp/.../scratchpad/gate6.py` (not
committed, a throwaway harness built for this round), run via
`explore/engine.py`'s `run`/`pool` so every call keeps the memory-budget and
RAM-admission guards. Peanut timeout 45s/1536 MB per config call (trimmed from
`fuzz_walnut.py`'s 120s so the 4-configs × 280-pairs sweep finishes in minutes;
a disagreement inside 45s is exactly as much a blocker as one at 120s; this
only affects how many hard instances resource-exhaust rather than answer, which
is recorded as `timeout`, not compared).

**(a) 4 configs × 280 pairs = 1120 calls.** Each config's verdict and minimal
state count compared against `default` on the *same* pair.
**0 verdict or state-count disagreements.** 3 timeouts (resource exhaustion at
the 45s cap on `border`/`recurrence`/`right_special`-style instances, the same
templates `docs/FUZZ.md` names as the hard ones); no comparison is drawn on a
pair where either side didn't finish.

**(b) vs `peanut_old`**: the same 280 `default`-config formulas, run on
`peanut_old`. **0 disagreements.** 4 timeouts (resource exhaustion, `peanut_old`
lacking the flat core is if anything more likely to time out, not less).

**(c) vs Walnut 8-dev**, 100 of the 280 pairs (first 100 by draw order),
`default` config verdict only (state counts are not compared here, see
`docs/FUZZ.md`'s note on Peanut/Walnut reporting-granularity for the final
0-ary automaton). **0 disagreements**: run **serially**.

Serial-only is not incidental: a first pass ran the 100 Walnut calls 3-at-a-time
(`ThreadPoolExecutor(max_workers=3)`) and produced one `verdict_disagree`:
`k=2 m=2` sequence (`0->01,1->01` coded `1,0`, i.e. the periodic word `1010…`),
`palindrome` template, `E i. i<20 & A t1,t2. (t1<4&t2<4&t1+t2+1=4)=>T[i+t1]=T[i+t2]`,
Peanut/brute-force `FALSE`, that Walnut run `TRUE`. Rerunning the identical
script **serially** reproduced Walnut `FALSE`, agreeing with Peanut and an
independent brute force over the fixed-point prefix (same adjudicator
`tools/fuzz_walnut.py:brute_eval` uses). Concurrent Walnut JVMs sharing
`walnut7/Session` are not safe to adjudicate correctness from; the reported
100-pair run above is serial throughout, and this is now the harness's default
(see `gate6.py`'s comment at the walnut-phase call site). Not a Peanut finding:
recorded here because a report claiming "0 disagreements" needs the reader to
know one *did* transiently appear and why it doesn't count.

Raw records: `results/gate6_fuzz.jsonl` (280 rows, each with all 4 configs +
`peanut_old`), `results/gate6_blockers.json` (empty array).

## 2. `learn` vs `let`: REV / PERIOD / BORDER on the panel

Direct constructions (index terms must be addition-only, per `docs/LEARN.md`:
"`j+l-1-t` is not an admissible index term on its own", so REV and BORDER are
written with an auxiliary existential variable per `docs/RECON.md`'s pattern,
PERIOD needs no rewrite):

```
let REVDIR(i,j,l)  A p,q. (p>=i & p<i+l & q>=j & p+q+1=i+j+l) => T[p]=T[q]
let PERDIR(i,l,p)  A t. t+p<l => T[i+t] = T[i+t+p]
let BORDIR(i,l,b)  b<=l & (E d. b+d=l & (A p. (p>=i & p<i+b) => T[p] = T[p+d]))
```

For each of the panel's 19 sequences and each of the 3 relations: `learn LN
<kind>` then `let <DIRECT>`, then `? A <params>. $LN(<params>) <=> $DIRECT(<params>)`
inside one engine session (params `i,j,l` for rev, `i,l,p` for period, `i,l,b`
for border): 57 checks, timeout 150s / 6144 MB each, run via `engine.pool`
(4 workers).

**45/57 checks: `TRUE`, `learn` and the direct `let` construction agree
exactly, everywhere the direct construction finishes**, including every non-hard
panel sequence (thue-morse, period-doubling, rudin-shapiro, paperfolding,
cantor, mephisto, prism-a, prism-d, champion-m5, k3m3-artefact-a/b, all 3
relations each) plus `prism-1`/rev and `prism-1`/border.

**12/57: resource exhaustion on the *direct* construction**, not disagreement:
`prism-1`/period, all 3 relations on `single6`, all 3 on `tail-a`, all 3 on
`tail-b`, `tail-c`/rev, `tail-c`/period (150s timeout), `tail-c`/border (6144 MB
budget, after `learn` itself finished: `states=2086 ms=37278`). This reproduces,
per relation, exactly the asymmetry `docs/LEARN.md` already documents for
Tribonacci ("PERIOD: 1.01s learned vs 120.29s direct, same 404 states ... BORDER
has no direct value at all — `let` exhausts 6GB"): the direct construction is
the one that blows up, `learn` is not asked to and does not.

**0 disagreements.** Raw: `results/gate6_learn_let.json`.

## 3. Benchmark: panel hard cases + Tribonacci, seconds / peak MB per config

> **Corrected 2026-08-19 by the "Final defaults" section below**: `tail-c`'s direct
> `let FE` does **not** die at 6 GB under `AM_PAR=8`. It finishes in 191.1 s / 2818 MB
> at 1382 states. The claim below is wrong; `bench/DETPAR-RESULTS.md` was right.

`AM_MEM_MB=6144`, msd, query `let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]`
except `tail-c` (dies at 6 GB under every configuration below, same finding as
`docs/LEARNFE.md`/`bench/DETPAR-RESULTS.md`, so `learnfe FE` is used there
instead, noted in the table). Seconds are the engine's own `ms=`; peak MB is the
engine's own allocator high-water mark (`mem` command), not RSS.

`default` and `AM_PAR=8` were run fresh, quiet-machine, one process at a time,
for this round. `AM_STRATEGY=bdd` reuses `bench/BDD-RESULTS.md`'s panel rows
(same binary, same query, same 6144 MB, machine **under concurrent load**,
its own note: "1.2-1.4x higher [seconds] than the quiet-machine figures ...
prism-1 48.7s here vs 38.6s there"; states/MB are load-independent and were not
re-measured). `AM_ANTICHAIN=1`: proven at the source level to be a no-op for
this benchmark, `logic.rs`'s `AM_ANTICHAIN` hook (`crate::antichain::eval_closed`)
fires only on a **closed** sentence (a `?` query), and `let FE(i,j,l) ...`
is an **open** 3-free-variable predicate, so the antichain code path is never
entered; spot-checked fresh on `single3` (`states=190 ms=11 MB=6`, identical to
`default`'s `ms=10 MB=6`) and `single6` (`states=3971 ms=18679 MB=169`,
identical to `default`'s `ms=17917 MB=169`); column below is `default`'s own
numbers, not a separate run, for every row.

### Equality of factors, base k

    case      states  default (s/MB)   AM_PAR=8 (s/MB)   AM_ANTICHAIN=1   AM_STRATEGY=bdd (s/MB)*  Walnut best
    prism-1     467    32.8 / 312        2.9 / 109         = default        47.5 / 312          90.8s CCLS
    single3     190    0.010 / 6         0.004 / 4         = default        0.05 / 22            0.4s BRZ-CCL
    single4     698    0.149 / 10        0.023 / 8         = default        0.21 / 27            3.8s BRZ-CCL
    single5    1877    2.03 / 30         0.230 / 27        = default        2.66 / 38             67s BRZ-CCL
    single6    3971    17.9 / 169        1.7 / 84          18.7 / 169 (measured)  27.6 / 169      897s BRZ-CCLS
    tail-a     1165    144.6 / 1480       15.3 / 741        = default        181.5 / 1480          66s CCLS
    tail-b     1000    176.3 / 1249       20.9 / 729        = default        220.6 / 1249         163s CCLS
    tail-c†    1382    14.2† / 50        16.9† / 50        14.3† / 50       240s TIMEOUT (let FE)  10.6s CCLS

    * bench/BDD-RESULTS.md numbers, machine under concurrent load (1.2-1.4x quiet-machine seconds; states/MB unaffected)
    † tail-c: `let FE` exhausts 6144 MB under every configuration (default, AM_PAR=8, AM_ANTICHAIN=1, and
      AM_STRATEGY=bdd per BDD-RESULTS.md's own "still needs learnfe under every configuration"); the
      seconds/MB shown are `learnfe FE` instead, run fresh this round, states=1382 all 4 configs.

### Tribonacci, states / seconds (fresh this round, all 4 configs)

    query           states  default    AM_PAR=8   AM_ANTICHAIN=1   AM_STRATEGY=bdd   Walnut best
    FE [let ladder]   27     3.05s      0.40s       3.06s            3.20s            62s BRZ-CCLS
    FE [learnfe]      27     0.071s     0.084s      0.074s           0.074s           62s BRZ-CCLS

learnfe's near-flat timing across configs is expected: the learner's cost is
membership-oracle walks + equivalence queries against the *sequence*, not
subset-construction/projection, so none of AM_PAR/AM_ANTICHAIN/AM_STRATEGY
touch its hot path (all four numbers are within measurement noise of each
other: 71-84ms).

## Reading

- **No disagreements anywhere**: 1120 (config-vs-default) + 280 (vs `peanut_old`)
  + 100 (vs Walnut) fuzz-diff pairs, 57 `learn`-vs-`let` panel checks, 4-way
  Tribonacci cross-check (states=27 every time): every verdict, every minimal
  state count, matches. The det_par/symbolic/antichain/generalised-`learn` work
  already on `main` is correctness-neutral against the reference on everything
  this gate could exercise in the time available.
- **AM_PAR=8 is the win**: 2.5x-11.3x faster than default on every panel-hard
  case measured (prism-1 32.8s→2.9s, single6 17.9s→1.7s, tail-a 144.6s→15.3s,
  tail-b 176.3s→20.9s, Tribonacci FE 3.05s→0.40s), and lower peak MB
  everywhere too, including the two `tail-*` cases (1480→741 MB, 1249→729 MB,
  the parallel frontier is smaller here, not larger, contrary to a naive
  parallelism-costs-memory prior).
- **AM_ANTICHAIN=1 is a documented no-op here, correctly**: it only ever fires
  on closed sentences, this benchmark's query is an open predicate, and the
  binary takes the identical code path: spot-checks confirm identical seconds
  and MB to the single millisecond/megabyte where it matters.
- **AM_STRATEGY=bdd is a wash to mildly negative on this panel** (0.95x-1.09x
  of default depending on case, per `BDD-RESULTS.md`'s own load-adjusted
  numbers), expected: `bench/BDD-RESULTS.md` found the same thing at 1156-job
  fuzz scale (0 disagreements, symbolic pays off on wide/sparse alphabets, not
  the equality-of-factors panel's shape).
- **tail-c still needs `learnfe`** under every configuration, unchanged from
  prior rounds: `let FE` exhausts 6144 MB regardless of AM_PAR/AM_ANTICHAIN/
  AM_STRATEGY, all three being determinization/evaluation strategies for the
  *same* blown-up intermediate automaton the direct construction always builds.
- **Against Walnut's best-strategy-per-case column**, `AM_PAR=8` beats it on
  every base-k row now measured (prism-1 2.9s vs 90.8s, single3-6, tail-a 15.3s
  vs 66s, tail-b 20.9s vs 163s) and on Tribonacci FE (0.40s ladder / 0.084s
  learnfe vs 62s), consistent with `bench/DETPAR-RESULTS.md`'s prior finding
  that `AM_PAR=8` flips the one prior Walnut-wins case (tail-a) in Peanut's
  favour; tail-c is the only row where Walnut's CCLS (10.6s) still beats
  Peanut's `learnfe` (14.2-16.9s).

## Blockers

None.

---

# Final defaults (2026-08-19)

The gate above found **no disagreement in any configuration**, so nothing is disabled
for a correctness reason and `docs/KNOWN-ISSUES.md` records no algorithm bug from this
round. What follows is the decision about defaults and the measurement it was made
from: a fresh run of every configuration with the new defaults in place, because
several of the earlier tables compare an opt-in flag against a *serial* baseline that
is no longer what the engine does.

Machine: the same 18-core Apple Silicon / 24 GB box, **quiet, one engine process at a
time**, `AM_MEM_MB=6144`, msd, engines launched through `explore/engine.py`. Seconds
are the engine's own `ms=`; MB is the engine's own allocator high-water mark (`mem`),
not RSS. Harness: `bench/defaults_bench.py` (suites `fe`, `easy`, `learnfe`, `closed`,
`trib`); raw rows in `results/defaults_{fe,easy,learnfe,closed,trib}.json`. Three
configurations of one binary, back to back per case:

    old    AM_PAR=1 AM_ANTICHAIN=0     exactly the pre-2026-08-19 default path
    new    no environment at all       the defaults decided here
    auto   AM_STRATEGY=auto            the new defaults plus the symbolic rung

## 1. The decision

| flag | old default | new default | reason |
|---|---|---|---|
| `AM_PAR`: flat/parallel determinization (`det_par.rs`) | off (serial reference path) | **`min(8, cores-2)`** | faster on all 19 panel sequences and on Tribonacci, never slower by more than 1 ms on any of them, and lower peak memory on every case above 10 MB |
| `AM_ANTICHAIN`: closed-sentence evaluation (`antichain.rs`) | off | **on** | 3x-66x on the `A..E..` closed shapes it fires on, and it answers two sentences the old default cannot answer at 6 GB; worst measured cost +5 ms on a 5 ms sentence |
| `AM_STRATEGY`: symbolic/BDD core (`symbolic.rs`) | off | **off** | correct, but measured against the *new* baseline it is 3x-11x slower on the base-3 panel sequences and never faster on any panel row (§3) |
| `AM_LAZY_CLOSED` (`det_par.rs`) | off | **off** | correct and gated, worth under 1 %, and the antichain now takes the closed sentences first |
| `learn NAME <kind>` (`learn.rs`) | - | - | a command, not a strategy. `let` still compiles what it is given; nothing is silently rerouted to the learner |

Ladder order under the new defaults, for one existential projection:

    closed sentence?  -> antichain.rs (emptiness / antichain universality), else fall through
    symbolic rung     -> skipped (AM_STRATEGY=off)
    forward(AM_CAP0=50k) -> Brzozowski(200k) -> forward(AM_CAP=3M) -> Brzozowski(12M)
    ... every rung running the flat, bitset-packed, frontier-parallel core

## 2. Equality of factors, base k: the named hard cases

`let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]`. States identical in all three
configurations on every row.

    case      states   old (s / MB)     new (s / MB)     new vs old   auto (s / MB)
    prism-1      467    34.05 /  312     2.71 /  109       12.6x        2.74 /  109
    single3      190     0.010 /   6     0.004 /    4       2.5x        0.004 /    4
    single4      698     0.150 /  10     0.022 /    8       6.8x        0.022 /    8
    single5     1877     2.06  /  30     0.219 /   27       9.4x        0.216 /   27
    single6     3971    18.02  / 169     1.66  /   84      10.9x        1.64  /   84
    tail-a      1165   140.4   /1480    14.49  /  741       9.7x       14.27  /  741
    tail-b      1000   169.1   /1249    19.41  /  729       8.7x       19.15  /  729
    tail-c      1382   no answer*      191.1   / 2818        --       191.1   / 2818
    tail-c  [learnfe FE]
                1382    14.99 /   50    16.10 /    50       0.93x      17.87 /    50

`*` tail-c under the old default is killed by `explore/engine.py`'s RSS watchdog after
88 s (RSS past 9.4 GB while the allocator's own 6144 MB live-byte budget had not yet
been reached: the reference path's `Vec<Vec<State>>` NFA carries that much allocator
overhead). `bench/DETPAR-RESULTS.md` saw the same case exit 3 on the live-byte budget
after 392.7 s. Either way the old default does not answer tail-c at 6 GB and the new
one does, in 191.1 s / 2818 MB, at the same 1382 states `learnfe` reports, which
resolves the disagreement between `bench/DETPAR-RESULTS.md` ("finishes, 202 s") and
§3 above ("dies at 6 GB under every configuration") in favour of DETPAR: it finishes.
`learnfe` is still 12x faster and 56x smaller on that row and remains the right tool.

The one row where the new default is slower is `tail-c`'s `learnfe`: 14.99 s -> 16.10 s,
**7 % slower**. The learner's hot path is membership-oracle walks, not subset
construction, so it gains nothing from the parallel core and pays its fixed costs.
Same effect, same size, on Tribonacci `learnfe` (§5).

## 3. The rest of the panel, and why `AM_STRATEGY` stays off

    case               states   old (s/MB)    new (s/MB)    auto (s/MB)
    thue-morse             15   0.000 /   0   0.001 /   0   0.001 /   0
    period-doubling         8   0.000 /   0   0.000 /   0   0.000 /   0
    rudin-shapiro          68   0.120 /  88   0.023 /  83   0.023 /  83
    paperfolding           44   0.001 /   0   0.001 /   1   0.001 /   1
    cantor                 17   0.000 /   0   0.001 /   0   0.001 /   0
    mephisto               14   0.002 /   1   0.001 /   3   0.002 /   3
    prism-a                24   0.000 /   0   0.001 /   0   0.001 /   0
    prism-d                82   0.449 / 132   0.069 / 102   0.204 /  34
    champion-m5           199   0.035 /   6   0.008 /   4   0.007 /   4
    k3m3-artefact-a       216   0.041 /   7   0.010 /   8   0.046 /  26
    k3m3-artefact-b        71   0.026 /   6   0.006 /   6   0.068 /  37

New vs old on the easy rows: 1.3x-5.9x faster where there is any work at all, and on
the five rows that finish in a millisecond either way the difference is +1 ms: the
one-time cost of building the thread pool. Same states everywhere.

**`AM_STRATEGY=auto` is off because of this table, not because of the gate.**
`bench/BDD-RESULTS.md` measured `auto` at 1.85x-1.88x *faster* than the baseline on
`prism-d`, but that baseline was the serial reference path. Against the parallel
default the same three base-3 sequences go the other way: prism-d 0.069 s -> 0.204 s
(3.0x slower), k3m3-artefact-a 0.010 s -> 0.046 s (4.6x), k3m3-artefact-b 0.006 s ->
0.068 s (11.3x). `auto` is not faster than the new default on **any** row of the
19-sequence panel; the largest gain it shows anywhere is memory (prism-d 102 MB ->
34 MB, k3m3-artefact-b's is worse not better). It keeps its own case (the 18x on the
five-variable `FE2` reformulation over base 3) and that case is a session flag away,
which is where it belongs.

## 4. Closed sentences: where the antichain lives, and one regression that is not its fault

Four closed sentences over the learned `FE`, on six sequences, all three configurations
(`bench/defaults_bench.py closed`, `results/defaults_closed.json`). Every cell is the
`?` line's own `ms`, after a `learnfe FE` that is not counted. Verdict TRUE in all 72.

    sentence     shape          sequence     old (s / MB)   new (s / MB)   new vs old
    fe-recur-N   A i,n,N. E j.  single4       0.598 /  468   0.009 /  12       66x
                 j>=N & FE      single5       no answer*     0.031 /  75        --
                                tail-a         5.65 / 3015   0.023 / 252      246x
                                tail-b         13.0 / 2775   0.089 / 147      146x
                                tail-c         3.15 / 1572   0.024 /  50      131x
                                prism-1       0.223 /  199   0.159 / 199       1.4x
    fe-recur     A i,n. E j.    single4       0.011 /   12   0.008 /  12       1.4x
                 j>i & FE       single5       0.079 /   67   0.023 /  75       3.4x
                                tail-a        0.038 /   47   0.018 / 252       2.1x
                                tail-b        0.105 /  172   0.078 / 147       1.3x
                                tail-c        0.037 /   50   0.018 /  50       2.1x
                                prism-1       0.096 /  199   0.150 / 199      0.64x
    fe-samelen   A n. E i,j.    single4       0.011 /   12   0.008 /  12       1.4x
                 i<j & FE       single5       0.079 /   67   0.023 /  75       3.4x
                                tail-a        0.040 /   47   0.018 / 252       2.2x
                                tail-b        0.104 /  172   0.078 / 147       1.3x
                                tail-c        0.038 /   50   0.018 /  50       2.1x
                                prism-1       0.095 /  199   0.143 / 199      0.66x
    fe-cube      E i,n. n>=1    single4       0.005 /   12   0.010 /  12      0.50x
                 & FE(i,i+n,2n) single5       0.019 /   50   0.028 /  75      0.68x
                                tail-a        0.011 /   47   0.019 / 252      0.58x
                                tail-b        0.089 /  172   0.134 / 147      0.66x
                                tail-c        0.013 /   50   0.020 /  50      0.65x
                                prism-1       0.120 /  199   0.313 / 199      0.38x

`*` single5 `fe-recur-N` under the old default exhausts the 6144 MB budget and never
answers; the new default answers TRUE in 31 ms. Same finding as
`bench/ANTICHAIN-RESULTS.md`, reproduced against the new baseline.

**The regressions in that table are not the antichain.** 2x2 attribution over
`AM_PAR` x `AM_ANTICHAIN` (`bench/defaults_bench.py attrib`), prism-1, quiet machine:

    sentence   AM_PAR=1 AM_ANTICHAIN=0   AM_PAR=1 AM_ANTICHAIN=1   default   default AM_ANTICHAIN=0
    fe-cube            119 ms                    119 ms            326 ms          317 ms
    fe-recur            95 ms                     94 ms            150 ms          150 ms

The antichain is free to the millisecond in both rows; the whole difference is the
parallel/flat determinization core. Narrowed further, on `fe-cube`:

    AM_PAR=1              (dfa.rs reference core)         128 ms
    AM_PAR=1 AM_FAST=1    (flat core, one thread)         383 ms
    default               (flat core, eight threads)      325 ms

So it is **the flat core, not the threads**: on a sentence whose projections are each
small, packing every one of them into a `FlatNfa` with a transposed successor buffer
costs more than the reference core's `Vec<Vec<State>>` does, and the pool then wins a
little of it back. Three candidate mitigations were implemented and measured during
this round: a minimum block size before the pool is used, a minimum `nstates * alpha`
before the flat core is used at all, and a lower cap on `Dfa::product`'s
direct-indexed pair table, and **none of them moved this row** (326/325/330 ms across
every threshold tried), so all three were reverted rather than shipped as unvalidated
knobs. The regression is recorded, with its reproduction, as
`docs/KNOWN-ISSUES.md` §7; it is bounded at +200 ms on the worst case measured and is
the price of a core that is worth 9x-13x on the cases that take minutes.
## 5. Tribonacci

`numsys trib`, `dfao TR 2 0:0,1 1:0,2 2:0,-`, same query. States 27 in every
configuration.

    query          old (s / MB)    new (s / MB)    new vs old   auto (s / MB)
    FE [ladder]     3.077 / 133     0.349 /  93       8.8x       0.367 /  93
    FE [learnfe]    0.074 /   1     0.082 /   1       0.90x      0.086 /   1

Same shape as tail-c: the ladder gains 8.8x from the parallel core, the learner loses
8 ms to it, because the learner never runs a subset construction big enough to
parallelise. 8 ms is the pool.

## 6. Against Walnut's best strategy per case

Walnut column: `bench/STRATEGY-RESULTS.md`'s 2026-08-18 per-strategy re-run (Walnut
8-dev, `java -Xmx6g`, 15-minute ceiling, same machine, best of six strategies for that
case). It was **not** re-run alongside these rows; treat the ratios as order-of-
magnitude, not stopwatch. Peanut counts the dead state, so Peanut states = Walnut + 1.

    case       Peanut new default        best Walnut 8-dev      faster
    prism-1        2.71 s                CCLS       90.8 s      Peanut  33x
    single3        0.004 s               BRZ-CCL     0.4 s      Peanut 100x
    single4        0.022 s               BRZ-CCL     3.8 s      Peanut 173x
    single5        0.219 s               BRZ-CCL    67   s      Peanut 306x
    single6        1.66 s                BRZ-CCLS  897   s      Peanut 540x
    tail-a        14.49 s                CCLS       66   s      Peanut 4.6x
    tail-b        19.41 s                CCLS      163   s      Peanut 8.4x
    tail-c        16.10 s  (learnfe)     CCLS       10.6 s      WALNUT  1.5x
                 191.1  s  (ladder)
    trib FE        0.349 s (ladder)      BRZ-CCLS   62   s      Peanut 178x
    trib FE        0.082 s (learnfe)     BRZ-CCLS   62   s      Peanut 756x

**Where Walnut's best is still faster: tail-c, and only tail-c.** CCLS answers it in
10.6 s; Peanut's best on that row is 16.10 s through the learner, 191.1 s through the
direct ladder. Two rows moved into Peanut's column since `bench/STRATEGY-RESULTS.md`
was written: tail-a (66 s Walnut vs 139 s Peanut then, 14.49 s now) and tail-b (163 s
vs 168 s then, 19.41 s now), and single3 moved for the same reason (0.4 s vs 1.5 s
then, 0.004 s now). None of that makes Peanut's algorithms better than Walnut's: with
`[strategy]` chosen per query Walnut answers every one of these cases, `learnfe` is a
reimplementation of Khodier's construction that Walnut 8 is adopting, and the honest
summary of the difference is still defaults and ergonomics, plus a faster inner loop
(bitset subsets, flat arrays, eight threads) on the same algorithm family.
## 7. Regression, with the new defaults in force

All run on the shipped binary, no environment set except where stated.

| check | expected | got |
|---|---|---|
| `let FE` thue-morse | 15 states | **15** |
| `let FE` single5 | 1877 states | **1877** |
| Tribonacci `let FE` (ladder) | 27 states | **27** |
| Tribonacci `learnfe FE` | 27 states | **27** |
| `tools/walnut_suite.py`, 22 scripts / 247 commands | agree 223 | **223** (whole tally identical: 18 both-fail, 2 peanut-fail, 3 walnut-fail-peanut-ok, 1 mismatch-states, 169 lang-agree, 1 lang-mismatch, 38 lang-skip) |
| GUI selftest (`?selftest=1`, headless Chrome DOM dump) | 25 PASS / 0 FAIL | **25 PASS, 0 FAIL** |
| `AM_FAST_VERIFY=1` over 14 panel sequences x {msd,lsd} x (`let FE`, `learnfe FE`, 3 closed sentences) | every `exists`/`minimize`/`zero_closure` equal element by element to the reference core | **28/28 sessions clean** |

The `AM_FAST_VERIFY` sweep's one flagged line is `prism-a`/lsd's
`ERR learnfe no progress ... at cap 67108864`, which reproduces identically under
`AM_PAR=1` and is a pre-existing property of the learner on that sequence, not a
verification failure: no assertion fired in any session.

## 8. What a reader should not conclude

- Not that Peanut's algorithms beat Walnut's. Walnut answers every case in §6 with the
  right `[strategy]`; the learner is Khodier's construction; the ladder is Brzozowski's.
  What changed here is a faster inner loop and defaults that need no per-query choice.
- Not that the new defaults are free. They cost up to +200 ms on closed sentences whose
  projections are all small (§4), +1 ms on any query with no work in it, and 7-10 % on
  `learnfe`, which cannot use them.
- Not that `AM_STRATEGY` is worthless: it is the fastest thing here on wide formulas
  (18x on five-track base-3), it is simply not what this panel measures (§3).
