# DETPAR — measured before/after (flat determinization core, `engine/src/det_par.rs`)

> **Measured with the pre-2026-08-19 defaults.** The flags named here are no longer
> all opt-in: `AM_PAR` defaults to `min(8, cores-2)` and `AM_ANTICHAIN` defaults to on.
> Re-measured head-to-head under the current defaults: `bench/SPEED-ROUND6.md`,
> "Final defaults".

Machine: 18-core Apple Silicon, 24 GB, macOS. Engine budget `AM_MEM_MB=6144` on
every row (the ceiling `bench/README.md` and `bench/STRATEGY-RESULTS.md` used).
Digit order msd.  Query, unless stated otherwise:

    let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]

**before** = `engine/target/release/peanut_old`, the engine built from `fb6a648`
before this work. **after** = the same tree plus `engine/src/det_par.rs` and the
`engine/src/dfa.rs` dispatch, run with `AM_FAST=1` (serial flat core) or
`AM_PAR=8` (frontier-parallel; implies `AM_FAST`). Both defaults are off: with no
flag set the binary runs the old code path.

Seconds are the engine's own `ms=` field; MB is the allocator high-water mark
(`mem` command), not RSS. The three configurations of each row were run
back-to-back, one process at a time, because other jobs share this machine —
interleaving is what makes the ratios meaningful, not the absolute seconds.

## Equality of factors, base k

    case      states   before          AM_FAST=1       AM_PAR=8        vs before   Walnut best
              (all 3 identical)  s   MB      s     MB      s     MB                (STRATEGY-RESULTS)
    prism-1      467      44.8   312    19.2   109    2.86   109       15.7x        90.8s  CCLS
    single6     3971      27.2   169    11.2    69    1.76    84       15.5x         897s  BRZ-CCLS
    tail-a      1165     156.6  1480    69.1   736    15.2   741       10.3x          66s  CCLS
    tail-b      1000     178.3  1249    84.3   721    20.6   729        8.7x         163s  CCLS
    tail-c      1382   *FAILED*  >6144  248.0  2819   202.2  2818        --          10.6s CCLS
    single5     1877      2.30    30    1.47    11    0.24    27        9.6x          67s  BRZ-CCL
    single4      698      0.16    10    0.11     6    0.025    8        6.4x         3.8s  BRZ-CCL
    single3      190     0.012     6   0.006     3    0.004    4        3.0x         0.4s  BRZ-CCL
    prism-d       82      0.49   132    0.34    93    0.079  102        6.2x
    rudin-shap    68      0.14    88    0.084   74    0.027   83        5.0x
    champ-m5     199     0.037     6    0.020    3    0.008    4        4.6x
    k3m3-art-a   216     0.045     7    0.026    4    0.011    8        4.1x

`*FAILED*` = the reference engine exits 3 ("memory budget exceeded: 6144 MB")
after 392.7 s. This row is the one qualitative change: the flat core's smaller
footprint (2819 MB against >6144 MB) is what lets the ordinary ladder finish
tail-c at all — 1382 states, agreeing with the `learnfe` answer for the same
sequence, which is how we know it is right.

## Tribonacci numeration system

    query                       before        AM_FAST=1     AM_PAR=8      Walnut best
    FE(i,j,l) [ladder]          3.35s 133MB   1.86s 85MB    0.39s 93MB    62s BRZ-CCLS

## Reading

- The serial flat core is a **1.6x - 2.5x** wall-clock win and a **1.7x - 2.9x**
  memory win, everywhere, with no change to a single state count.
- `AM_PAR=8` multiplies that by a further **3x - 7x** on the cases whose subset
  constructions are large enough to fill a frontier block (prism-1, single5,
  single6, tail-a, tail-b, Tribonacci FE); on tail-c the extra threads buy only
  1.2x, because that case spends most of its time in one long Brzozowski pass
  whose frontier stays narrow.
- Against `bench/STRATEGY-RESULTS.md`'s "best Walnut strategy per case" column,
  `AM_PAR=8` is faster on **seven of the eight** base-k cases, including the three
  (single3, tail-a, tail-b) where Walnut's best strategy previously won. The
  eighth is tail-c: Walnut's CCLS answers it in 10.6 s against 202 s for the
  ladder here, and Peanut's own `learnfe` (14.2 s) remains the right tool for it.
  Caveat: the Walnut column is quoted from the earlier `bench/STRATEGY-RESULTS.md`
  run, not re-measured alongside these rows, and this machine had other jobs on it
  during both. Treat it as an order-of-magnitude comparison, not a stopwatch.
- Parallel peak memory is slightly *higher* than serial on small cases (the
  frontier block's scratch buffer), never on the large ones.

## `AM_LAZY_CLOSED=1`

Closed sentences (`? ...`), where projecting the last variable leaves a one-letter
alphabet. Verdict and reported state count are identical in every configuration
(a zero-closed language over one letter is empty or `0*`, so the answer automaton
has one state either way); only the work done to get there changes. Seconds:

    sentence                                        before   AM_FAST  +LAZY   AM_PAR=8+LAZY
    prism-1  E i<20. has a border of length 4        82.59   37.87    37.32    5.70
    single6  E i,n. n>=1 & cube at i                  0.412   0.248    0.244   0.057
    trib     E i,n. n>=1 & cube at i                  1.365   0.825    0.825   0.254
    trib     E i,n. n>=1 & 4th power at i             4.761   2.822    2.820   0.614
    trib     A n. E i. palindrome of length n         0.790   0.654    0.650   0.388
    trib     E p,N. eventually periodic               0.155   0.079    0.075   0.034
    single5  E i,n. n>=1 & cube at i                  0.108   0.064    0.062   0.020
    prism-1  E p,N. eventually periodic               0.044   0.025    0.025   0.009
    tail-a   E i,n. n>=1 & overlap at i               0.203   0.119    0.114   0.030
    single5  A i<20. E j<20. i!=j & FE(i,j,4)         0.002   0.001    0.001   0.005
    tail-a   E i<20. right-special of length 3        0.004   0.003    0.003   0.009

Reading, without varnish: **`AM_LAZY_CLOSED` is worth 0-5 %, usually under 1 %.**
It removes the final subset construction, zero closure and minimization, but by
the time the *last* variable is projected the automaton is already small -- the
earlier quantifiers are where the cost is. It never costs anything (the
reachability scan is O(edges) against work it replaces), it is the only one of
these flags that changes what is computed rather than how, and it is off by
default. The two sub-10 ms rows show the parallel pool's fixed start-up cost
(~5 ms) dominating a query that has no work in it.

The FE panel is unaffected by construction: `let FE(i,j,l) ...` leaves three free
variables, so no projection in it is closed.

## Memoizing the base automata

Measured before implementing: under the hardest configuration (Tribonacci, where
`base::adder` goes through `numsys` files rather than the built-in fast path),
`? E i,j,t. i+t=j+t` — two adders, one comparator, three constants — reports
`ms=0`. The base constructors are already sub-millisecond and are called O(#terms)
times per formula, so a memo table would be unmeasurable; it is not worth an edit
to a file this builder does not own. Not implemented, deliberately.

## Correctness gate

Run: `python3 tools/fuzz_engines.py 200 8`.  Raw timings: `results/detpar_bench.json`.

- **228 (sequence, formula) pairs x 4 configurations = 912 engine runs, 0
  mismatches.** 200 random closed sentences (107 TRUE / 93 FALSE) over PRISM-drawn
  admissible k-automatic sequences across all ten `tools/fuzz_walnut.py` templates,
  plus the FE panel in both digit orders (28 runs). Compared per pair: verdict,
  minimal state count, and peak intermediate automaton size (`peak=`, which reached
  3,000,006 on the hardest pairs — i.e. the ladder's big forward cap was exercised).
- **`AM_FAST_VERIFY=1`** builds every `exists`, `minimize` and `zero_closure` both
  ways and asserts `nstates`, `alpha`, `vars`, `accept` and `trans` are equal
  element by element. Clean over the whole panel in both digit orders.
- **Command surface**: `learnfe`, `? $FE(...)`, `witness`, `enum`, `finite` and
  `dfa` output compared byte for byte (modulo `ms=`) between the reference binary
  and all three configurations, on thue-morse / single3 / rudin-shapiro / tail-c,
  msd and lsd. All identical.
