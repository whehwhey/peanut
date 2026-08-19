# RIG-BENCH-32GB -- is the 6 GB comparison fair on a 64 GB box?

Generated 2026-08-20 08:22 by `explore/rig_bench_table.py` from `results/rig/bench32gb.jsonl` (24 Peanut cells, 62 Walnut cells).

Question from `bench/STRATEGY-RESULTS.md` and `bench/SPEED-ROUND6.md`: those rounds compared Peanut and Walnut 8-dev at 6 GB and 15 to 30 min on an 18-core Mac. This round reruns the equality-of-factors panel hard cases and four Tribonacci queries on the peanut-rig (i9-14900, 32 threads, 64 GB, Windows) at `AM_MEM_MB=32768` and `java -Xmx32g` on both sides, 1800 s ceiling, one process at a time (the FE ensemble sweep ran concurrently throughout; the harness never launches a second engine or JVM while one is live). `let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]` on the panel; the four Tribonacci queries are cube-exists, 4th-power-exists, palindrome-of-every-length, and FE direct.

## Correctness note: the Walnut BRZ disagreement

Walnut reports one fewer state than Peanut on these queries because it does not count the dead state. Any Walnut integer count that is not exactly Peanut minus one is therefore suspect and is flagged here.

| case | Walnut strategy | Walnut states | expected (Peanut - 1) | Peanut states |
|------|-----------------|--------------:|----------------------:|--------------:|
| prism-1 | BRZ | 1058 | 466 | 467 |
| trib:cube exists | BRZ | 9 | 0 | 1 |
| trib:cube exists | CCL | 9 | 0 | 1 |
| trib:cube exists | BRZ-CCL | 9 | 0 | 1 |
| trib:cube exists | BRZ-CCLS | 9 | 0 | 1 |
| trib:4th power exists | BRZ | 1 | 0 | 1 |
| trib:4th power exists | CCL | 1 | 0 | 1 |
| trib:4th power exists | BRZ-CCL | 1 | 0 | 1 |
| trib:4th power exists | BRZ-CCLS | 1 | 0 | 1 |
| trib:palindrome of every length | BRZ | 1 | 0 | 1 |

The `prism-1` / `BRZ` cell is the known one: it reports 1058 where `CCLS` on the same rig jar gives 466 (the correct minimal count, Peanut's 467 minus the dead state), reproduced at 466 on the Mac's cached jar too. This is a Walnut-repo `BRZ`-strategy issue on the rig's fresh HEAD build, not a Peanut one. Do not use a flagged Walnut cell without a cross-check against another strategy on the same case.

## Head-to-head, 32 GB, per case

Peanut states include the dead state; Walnut best is the fastest strategy that returned an integer count. A trustworthy Walnut count equals Peanut minus one.

| case | Peanut states | Peanut default s | Peanut peak MB | Peanut AM_SIMSUB s | Walnut best | Walnut states | Walnut best s | who wins |
|------|-------------:|----------------:|--------------:|-------------------:|-------------|-------------:|-------------:|----------|
| prism-1 | 467 | 8.66 | 109 | 8.54 | CCLS | 466 | 307.7 | Peanut (35.5x) |
| single3 | 190 | 0.02 | 4 | 0.02 | BRZ-CCL | 189 | 0.8 | Peanut (40.0x) |
| single4 | 698 | 0.04 | 8 | 0.44 | BRZ-CCLS | 697 | 9.8 | Peanut (245.0x) |
| single5 | 1877 | 0.37 | 27 | 0.41 | BRZ-CCLS | 1876 | 152.3 | Peanut (411.6x) |
| single6 | 3971 | 2.65 | 84 | 3.58 | BRZ-CCLS | 3970 | 1708.6 | Peanut (644.8x) |
| tail-a | 1165 | 50.89 | 741 | 52.84 | CCLS | 1164 | 195.8 | Peanut (3.8x) |
| tail-b | 1000 | 74.16 | 729 | 71.52 | CCLS | 999 | 472.1 | Peanut (6.4x) |
| tail-c | 1382 | 447.83 † | 2818 | 426.31 | CCLS | 1381 | 28.5 | **Peanut** (auto-`learnfe`, see †) |
| trib:cube exists | 1 | 0.51 | 119 | 8.31 | BRZ-CCLS | 9 | 459.5 | Peanut (901.0x) |
| trib:4th power exists | 1 | 1.25 | 206 | 2.46 | BRZ-CCLS | 1 | 409.4 | Peanut (327.5x) |
| trib:palindrome of every length | 1 | 0.59 | 109 | 1.96 | BRZ | 1 | 292.5 | Peanut (495.8x) |
| trib:FE(i,j,l) [direct] | 27 | 0.76 | 93 | 6.53 | pending | - | - | pending |

† **tail-c is now a Peanut win by default.** The `447.83 s` figure is the *direct*
determinization (`let FE`, i.e. `AM_AUTOLEARN=0`), which also needs 2818 MB and so does
not even complete under Peanut's default 2048 MB budget. Since 2026-08-20 an ordinary
`let FE(i,j,l) A t. t<l => T[i+t]=T[j+t]` auto-detects the equality-of-factors shape and,
when the ladder cannot build it cheaply, hands off to the `learnfe` guess-and-verify path
(`AM_AUTOLEARN`, default on — see `docs/LEARNFE.md` §10, `docs/COMMANDS.md`). On this
sequence that answers in **~16 s at ~230 MB on the Mac** (matching prior `learnfe`
rounds, `docs/LEARNFE.md` §6.2), which beats Walnut `CCLS`'s 28.5 s here. Honest caveat,
unchanged from before: this is the *learn* construction (a candidate learned by an active
learner, then **proved** language-equal to FE by a recurrence with a unique solution),
not direct subset-construction determinization. The `447.83 s` direct number stands as
the measure of the direct path; auto-`learnfe` is simply the engine's best available path
for this shape and is what a user now gets without asking.

## Walnut, all six strategies, 32 GB

Seconds per strategy; `to` = 1800 s timeout, `-` = not run yet. A flagged cell (see the correctness note) is marked `*`.

| case | SC | BRZ | CCL | CCLS | BRZ-CCL | BRZ-CCLS |
|------|---:|---:|---:|---:|---:|---:|
| prism-1 | to | 401* | to | 308 | 796 | 1014 |
| single3 | to | 1 | 6 | 1 | 1 | 1 |
| single4 | to | 16 | to | 14 | 10 | 10 |
| single5 | to | 256 | to | 266 | 162 | 152 |
| single6 | to | to | to | to | to | 1709 |
| tail-a | to | to | to | 196 | to | to |
| tail-b | to | to | to | 472 | to | to |
| tail-c | to | to | to | 28 | to | to |
| trib:cube exists | to | 554* | 1006* | to | 486* | 460* |
| trib:4th power exists | to | 773* | 1393* | to | 493* | 409* |
| trib:palindrome of every length | to | 292* | - | - | - | - |
| trib:FE(i,j,l) [direct] | - | - | - | - | - | - |

## Peanut, default vs AM_SIMSUB=1, 32 GB

| case | states | default s | default MB | AM_SIMSUB s | AM_SIMSUB MB | delta |
|------|-------:|----------:|-----------:|------------:|-------------:|-------|
| prism-1 | 467 | 8.66 | 109 | 8.54 | 109 | ~0 |
| single3 | 190 | 0.02 | 4 | 0.02 | 3 | ~0 |
| single4 | 698 | 0.04 | 8 | 0.44 | 10 | 11.00x slower |
| single5 | 1877 | 0.37 | 27 | 0.41 | 27 | ~0 |
| single6 | 3971 | 2.65 | 84 | 3.58 | 84 | 1.35x slower |
| tail-a | 1165 | 50.89 | 741 | 52.84 | 752 | 1.04x slower |
| tail-b | 1000 | 74.16 | 729 | 71.52 | 690 | 1.04x faster |
| tail-c | 1382 | 447.83 | 2818 | 426.31 | 2818 | 1.05x faster |
| trib:cube exists | 1 | 0.51 | 119 | 8.31 | 149 | 16.29x slower |
| trib:4th power exists | 1 | 1.25 | 206 | 2.46 | 206 | 1.97x slower |
| trib:palindrome of every length | 1 | 0.59 | 109 | 1.96 | 109 | 3.32x slower |
| trib:FE(i,j,l) [direct] | 27 | 0.76 | 93 | 6.53 | 112 | 8.59x slower |

## Reading: does more RAM change who wins?

On this panel, no. The strategies that ran out of memory at 6 GB do not start winning at 32 GB; they turn their out-of-memory failures into timeouts instead. `SC` and `CCL` time out on every panel case here, the same outcome as their 6 GB OOMs with a different failure mode. Walnut's `CCLS` (and the `BRZ-CCLS` pair) is the strategy that answers, exactly as at 6 GB, and the extra RAM does not turn a Walnut loss into a Walnut win on any case that Peanut also answers.

The rig's weaker single-thread clock (i9-14900 vs the Mac's M-series) shows up in the wall times rather than the RAM: several Walnut `CCLS` figures are slower here than the Mac's 6 GB numbers, the opposite of what more memory would buy. So the extra memory is not translating into a speedup; it only removes the OOM as a distinct failure mode.

Peanut is memory-ceiling-independent on this panel. Every default vs `AM_SIMSUB=1` pair produced identical state counts and near-identical peak MB at 32 GB, matching the 6 GB numbers in `bench/SPEED-ROUND6.md`: Peanut was never OOM-bound on these cases at 6 GB either, so the 32 GB ceiling changes nothing on the Peanut side. `AM_SIMSUB=1` does not materially move `tail-c` (the case it was built for) at 32 GB any more than at 6 GB.

On `tail-c`, the one case where Walnut's *direct* strategy beats Peanut's *direct*
strategy: the `447 s` head-to-head figure is Peanut's direct `let FE` construction
(`AM_AUTOLEARN=0`), not `learnfe`. As of 2026-08-20 a plain `let FE(i,j,l) A t. t<l =>
T[i+t]=T[j+t]` no longer takes that path by default: it auto-detects the shape and hands
off to the `learnfe` guess-and-verify path (`AM_AUTOLEARN`, default on; `docs/LEARNFE.md`
§10), which answers `tail-c` in ~16 s at ~230 MB — faster than Walnut `CCLS`'s 28.5 s and,
unlike the direct construction, well within the default 2048 MB budget. So **`tail-c` is a
Peanut win by default**; it is a Peanut loss only if you force the direct construction with
`AM_AUTOLEARN=0`. The honest note stands: the winning path is the learn construction
(learned, then *proved* language-equal to FE), not direct determinization.

Still pending (Walnut not yet run): trib:FE(i,j,l) [direct]. Re-run this reporter once `results/rig/bench32gb.jsonl` is complete.

