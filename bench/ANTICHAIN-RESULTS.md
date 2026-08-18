# Antichain evaluation of closed sentences — measurements

> **Measured with the pre-2026-08-19 defaults.** The flags named here are no longer
> all opt-in: `AM_PAR` defaults to `min(8, cores-2)` and `AM_ANTICHAIN` defaults to on.
> Re-measured head-to-head under the current defaults: `bench/SPEED-ROUND6.md`,
> "Final defaults".

`engine/src/antichain.rs`, `AM_ANTICHAIN` (default off when these rows were taken; on
since 2026-08-19). Design, rewrite rules and
flags: `docs/ANTICHAIN.md`. Reproduce: `python3 bench/antichain_bench.py fe`.

Machine: the repo's 18-core / 24 GB box, msd, `AM_MEM_MB=6144`, engines launched through
`explore/engine.py`. The tables below marked **serial** were taken one engine at a time
(`AM_WORKERS=1`); the panel-wide sweep was taken with two workers while other build
agents were also running, and is marked as such. `ms` is the engine's own per-query
timer (the `? ...` reply line) — the query alone, not the `learnfe FE` that precedes it.
`MB` is the allocator's high-water mark from the `mem` command, taken after the query;
where it equals the value taken *before* the query, the query is invisible in the
memory trace.

## The shape that moves

    ? A i,n,N. E j. j >= N & $FE(i,j,n)          "every factor recurs after every point"

An `A` block over an `E` block: universality of the NFA `proj_j(FE)`. **serial**

    sequence     base ms    ac ms   speedup    base MB   ac MB   antichain   verdict
    single4          780        9       87x        468      12         ...   TRUE
    single5     no answer       37         -   >6144 MB      50         ...   TRUE
    tail-a          8115       25      325x       3015      47   691 elts    TRUE
    tail-b         16444       89      185x       2775     172         ...   TRUE
    tail-c          3680       27      136x       1572      50         ...   TRUE
    prism-1          266      117      2.3x        199     199         ...   TRUE

`single5`: the default path **exhausts a 6144 MB allocator budget**
(`ERR memory budget exceeded: 6144 MB live`) determinizing `proj_j(FE)` and never
answers; the antichain answers TRUE in 37 ms inside 50 MB — and 50 MB is `learnfe`'s own
high-water mark, so the query itself allocates nothing measurable.

`AM_AC_DEBUG=1` on tail-a: NFA 2246 states over an 8-letter alphabet, **691 antichain
elements, 484 920 subsumption tests**. The determinization it replaces reaches 200 001
subsets.

Two neighbouring shapes, same runs, **serial**:

    query                                          seq        base ms   ac ms   base MB  ac MB
    ? A i,n. E j. j > i & $FE(i,j,n)               tail-b         134      72       172    172
                                                   single5         97      31        67     50
                                                   tail-a          60      19        47     47
                                                   tail-c          46      18        50     50
                                                   single4         15       7        12     12
                                                   prism-1        110     110       199    199
    ? A n. E i,j. i<j & $FE(i,j,n)                 tail-b         132      79       172    172
                                                   single5        102      32        67     50
                                                   tail-a          54      18        47     47
                                                   tail-c          50      19        50     50
                                                   single4         14       7        12     12
                                                   prism-1        103     104       199    199

The same statement written out inline, with no `learnfe` anywhere — so the inner
equality-of-factors quantifier is compiled from scratch too — still moves on single5,
because there the outer block is the blowup and the inner one is not (**serial**):

    ? A i,n,N. E j. j >= N & (A t. t < n => T[i+t] = T[j+t])     on single5
      default    ERR memory budget exceeded: 6144 MB live      (no answer)
      antichain  TRUE, 2149 ms, peak 30 MB, 911 antichain elements over a 3703-state NFA

On tail-a the same sentence is inner-dominated and only the memory moves:
170.7 s / 3015 MB -> 165.7 s / **1480 MB** (two workers, other agents active).

## Where it does not move, and why

Panel-wide sweep, 12 sequences x 14 closed FE sentences = 168 rows, two workers with
other agents active (`bench/antichain_results.json`):

* `fe-cube`, `fe-4power`, `fe-unique`, `fe-rext` and the whole critical-exponent ladder
  `fe-crit-{2/1,7/3,5/2,8/3,3/1,7/2,4/1}` land within +-2 % of the default path on every
  sequence, with identical peak memory. Largest regression in the whole sweep: **+5 ms**
  (prism-1 `fe-crit-8/3`, 296 -> 301 ms). These are E-blocks whose body automaton is the
  entire cost, or A-blocks whose antichain is a handful of elements: swapping a cheap
  projection for a cheap reachability query changes nothing.
* The GUI library's inline sentences — `recurrent`, `mirror`, `arb-pal`, `unbordered`,
  `rs-count`, `ap`, `cube-free`, `overlap-free`, `has-pal`, `peltomaki` — do not move on
  prism-1, prism-d, tail-a or thue-morse, for a structural reason worth stating plainly:
  **their cost is the inner equality-of-factors quantifier, not the outer block.** On
  prism-1, `recurrent` is 47.4 s by default and 45.0 s with the antichain; `mirror` 54.4 s
  vs 54.4 s; both with identical peak memory. ~99 % of both numbers is the `A t`
  construction underneath, which this module never touches.
* Same story for the `border` template of `docs/FUZZ.md` on a hard PRISM draw
  (`E i. i<20 & (E b,bp. b>=1 & b+bp=4 & (A t. t<b => T[i+t]=T[i+bp+t]))`, k=3, m=5):
  120.0 s default vs 133.2 s antichain (both measured under load), with the same
  3 000 006-subset peak inside the `A t`. Hoisting `b,bp` removes the outer
  determinization; the outer determinization was never the problem.

**The named panel cases of `bench/STRATEGY-RESULTS.md` (prism-1 38.6 s, single3..6,
tail-a/b/c) are unchanged and cannot change: they are open formulas** — `FE(i,j,l)` with
three free variables. This module only fires on `?` sentences with no free variables.

## The cost of giving up

prism-1 `unbordered` (`? A n. n>=1 => E i. ~(E b,j. ...)`) is the case where the
antichain does exceed its budget: a 2473-state NFA over a 4-letter alphabet whose
antichain passes 5 000 elements. **serial**

    default                                        40.8 s   peak 74327   312 MB
    antichain, abstain-and-let-the-caller-recompile 87.9 s  peak 74327   312 MB
    antichain, finish from the NFA already in hand  42.2 s  peak 74327   312 MB

The middle row is why `ev_forall` never abstains after compiling: the fallback has to
finish the job itself (replaying `dfa.rs`'s determinization ladder on the projected NFA
and answering by product reachability), because handing the question back means
compiling the body a second time. Shipped behaviour is the third row — a 3.4 % overhead
for the abandoned antichain attempt, identical peak and identical memory.

## Simulation

Measured on tail-a `fe-recur-N` (2246-state NFA, alphabet 8), **serial**:

    AM_AC_SIM=off   691 antichain elements, 484 920 subsumption tests,  20 ms
    AM_AC_SIM=on      1 antichain element,        8 subsumption tests, 338 ms

The simulation preorder does exactly what the literature says — it collapses the
antichain to a single element — and the naive greatest-fixpoint that computes it costs
seventeen times the search it saves. So it is a ladder, not a default: plain antichain
first, simulation only if the plain one passes `AM_AC_SIM_TRIGGER` elements *and*
`|Q|^2 * alpha <= AM_AC_SIMWORK` (8e6, which excludes both of the automata above). On
this panel that condition never fires; the flag is kept because a cheaper simulation
(partition-based rather than the naive fixpoint) would change the arithmetic.

## Gate

`python3 tools/antichain_gate.py` — every script run three times: the new binary with
the flag off, the new binary with the flag on, and `engine/target/release/peanut_old`
(the engine as it was before this work). Replies are compared line by line after
removing `ms=` and `peak=`; a run cut short by a timeout or the runner's RSS watchdog on
one binary but not another is counted as skipped, not as a disagreement.

    suite     scripts   flag-off vs flag-on   peanut_old vs flag-on   skipped
    fuzz         1100          0 mismatches           0 mismatches          0
    library       912          0 mismatches           2 (see below)       101
    fe             19          0 mismatches           0 mismatches          2

* `fuzz` — the ten formula templates of `tools/fuzz_walnut.py` over 110 PRISM sequences,
  same seed and admissibility filter as `docs/FUZZ.md`.
* `library` — all 24 scripts of the GUI library (`gui/serve.py`) over all 19 sequences of
  `bench/panel.json`, in **both** msd and lsd. Includes the open formulas (`dfa`, `enum`,
  `witness`, `let`) as a control: the antichain must never fire on them, and their state
  counts are unchanged. The two `peanut_old` differences are `lsd/prism-a/fe-learn` and
  `lsd/prism-a/fe-use`, and they are not this module's: another agent reworded a
  `learnfe` error string in `learn.rs` ("at LCP cap" -> "at cap") in the same working
  tree. Flag-off and flag-on agree on both.
* `fe` — `learnfe FE` plus all 14 closed FE sentences of `bench/antichain_bench.py`, on
  all 19 panel sequences (266 sentences).

Two further checks, outside the suites:

* **Forced fallback.** With `AM_AC_WORK=1` every A-block abandons its antichain and goes
  down the `determinize_ladder` + `contains` path. Over the A-block sentences of the
  library on prism-a, in msd and lsd, the replies are byte-identical to `peanut_old`.
* **Numeration systems.** `numsys trib` + `dfao TR`, lsd: `learnfe FE`, six closed FE
  sentences and two inline ones, compared across `peanut_old`, flag-off and flag-on
  (and again with `AM_AC_WORK=1` to force the fallback). All four runs are
  byte-identical, down to the point at which the two inline sentences exhaust the same
  2048 MB budget. The guard automaton is the validity language of every kept track, so
  `A` still means "for every valid representation" and not "for every word".
