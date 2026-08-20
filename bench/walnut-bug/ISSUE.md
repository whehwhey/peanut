# Suspected Walnut BRZ bug on prism-1 equality-of-factors: NOT A BUG (do not file)

**Verdict: do not file a Walnut issue. The suspected BRZ/CCLS disagreement is an
artifact of our own benchmark harness, not a Walnut defect.** When the identical
command is run to completion, Walnut's BRZ strategy returns 466 states, in exact
agreement with CCLS, BRZ-CCL, BRZ-CCLS, Peanut (467 = 466 + dead state), and an
independent minimization. This note records the verification John Nicol's request
prompted, so the finding is documented and the internal benchmark can be corrected.

## What was suspected

Our rig benchmark (`results/rig/bench32gb.jsonl`, surfaced in
`bench/RIG-BENCH-32GB.md`) recorded that the equality-of-factors (FE) predicate on
the sequence "prism-1" gave **1058 states under strategy BRZ** but **466 under
CCLS** on the same Walnut HEAD build. Since the minimal DFA of a language is unique,
two determinization strategies cannot both be right, so BRZ=1058 looked like a
Walnut bug. John Nicol asked us to file a ticket and include the prism-1 automaton,
which he could not find in our tree.

## prism-1

A 4-uniform morphic DFAO. In Peanut's notation:

    def T 4 6 0 0305 4555 2321 0514 1023 4300 102202

that is: base k=4, 6 states, start state 0, images
sigma(0)=0305, sigma(1)=4555, sigma(2)=2321, sigma(3)=0514, sigma(4)=1023,
sigma(5)=4300, and coding 102202 (state 0->1, 1->0, 2->2, 3->2, 4->0, 5->2).
First terms: 1 2 1 2 1 2 0 0 1 2 1 2 0 2 1 1 1 2 1 2 0 2 1 1 0 2 2 2 0 1 ...

The FE predicate is: A t. t < l => T[i+t] = T[j+t] (free variables i, j, l).

The Walnut Word Automaton for prism-1 is `prism-1.txt` in this directory (verified
below). In Walnut it is built directly from the morphism:

    morphism prism1 "0->0305 1->4555 2->2321 3->0514 4->1023 5->4300";
    promote PRISM1P prism1;
    morphism prism1cod "0->1 1->0 2->2 3->2 4->0 5->2";
    image T prism1cod PRISM1P;

## Reproduction (Walnut only, no Peanut in the loop)

`repro.txt` builds the DFAO T exactly once, then defs the FE predicate four times,
byte-for-byte identical except the `[strategy * X]` prefix and the result name, so
any difference in the reported count would have to be internal to Walnut. Metacommands
apply only to commands ending in `::` (single `:` silently ignores them), so each def
ends in `::`. The base-4 number system (`?msd_4`) is required because T is a base-4
DFAO. Per-strategy single-def files are `run_sc.txt`, `run_brz.txt`, `run_ccls.txt`,
`run_brz_ccls.txt`.

Run on the rig:

    cd C:\peanut\maths\walnut7
    'load run_brz.txt;','quit;'  | java -Xmx48000M -jar target\Walnut-all.jar
    'load run_ccls.txt;','quit;' | java -Xmx8000M  -jar target\Walnut-all.jar

## Observed state counts (fresh, single-process runs on the rig)

| strategy | reported states | completed? | time | source |
|----------|----------------:|:----------:|-----:|--------|
| BRZ      | **466** | yes (Total time printed) | 2422 s | `out_brz.log` (this run) |
| CCLS     | **466** | yes | 362 s | `out_ccls.log` (this run) |
| BRZ-CCL  | 466 | yes | 796 s | benchmark `bench32gb.jsonl` (ms=795823) |
| BRZ-CCLS | 466 | yes | 1013 s | benchmark `bench32gb.jsonl` (ms=1012789) |
| SC       | (does not finish) | timeout | >1800 s | benchmark `bench32gb.jsonl` |
| CCL      | (does not finish) | timeout | >1800 s | benchmark `bench32gb.jsonl` |

Every strategy that runs to completion reports 466. BRZ agrees with CCLS once it is
allowed to finish.

## Root cause: the 1058 was an incomplete run misrecorded by our harness

The original rig row in `results/rig/bench32gb.jsonl` is:

    {"case":"prism-1","side":"walnut","config":"BRZ","states":1058,"peak":25434,"s":401.4,"ms":null,"verdict":""}

The `"ms": null` is the tell: our parser (`explore/rig_bench_32gb.py`, `run_walnut`)
sets `ms` from the "Total computation time: Nms" line, and takes the state count as
`final = sizes[-1]` (the last ":N states" printed). A null `ms` means that line was
never printed, i.e. the JVM exited before the query finished. The 1058 is the size
of an intermediate NFA, not a final DFA. In the completed BRZ log it appears as:

    computed ~:1058 states           (the complemented inner formula)
    quantifying:1058 states
    Determinizing [#4, strategy: Brzozowski]: 1058 states   <-- last size before the process died
    ...(then ~1500 s more of Brzozowski work)...
    Reverse of reverse: 467 states
    Determinized: 467 states
    computed ~:466 states
    (A t (t<l=>T[(i+t)]=T[(j+t)])):466 states
    Total computation time: 2422116ms.

The BRZ computation needs about 2422 s even at 48 GB, which is longer than the
benchmark's 1800 s ceiling, so BRZ cannot finish this case in that window regardless;
a correct harness would have recorded "timeout" (as it did for SC and CCL). Instead
the JVM was terminated early (at 401 s, under memory pressure from the FE-ensemble
sweep that RIG-BENCH-32GB.md notes was running concurrently; a native/OS kill, not a
Java OutOfMemoryError, so the `"OutOfMemoryError" in out` guard did not catch it), and
the parser fell through to `sizes[-1] = 1058`. The recorded `peak` of 25434 (well below
the 74318 that the completing BRZ-CCL/CCLS runs reach) confirms the run stopped early,
before Brzozowski's large reverse phase.

## Why 466 is correct (independent of both Walnut and Peanut)

1. **prism-1.txt is the right sequence.** `verify_prism.py` builds prism-1 two ways,
   from the Peanut morphism definition and by running `prism-1.txt` as a DFAO, and
   compares 20000 terms: 0 mismatches.
2. **Independent minimization.** `min_check.py` reads Walnut's saved FE automaton,
   completes it to the full 64-letter product alphabet (msd_4 x msd_4 x msd_4) with an
   explicit dead sink, and runs its own partition-refinement minimizer. Result: the
   completed minimal DFA has 467 states (466 plus the dead state Walnut does not
   count). So 466/467 is the true minimal size, computed without trusting either
   prover's internal minimizer.
3. **BRZ and CCLS agree exactly.** The FE automaton saved by the completed BRZ run is
   byte-for-byte identical to the one saved by the CCLS run (146029 bytes each), and
   `min_check.py`'s product-reachability equivalence check confirms they accept the
   same language.
4. Peanut independently reports 467 (466 + dead state), and CCLS=466 matched an
   earlier brute force.

## Environment

- Walnut commit: f308bc1a1c703be770fb08bf87ae006cfec6b26f
  ("Reduce invocations of enable/disable print(). Use better whitespace trimming.",
  2026-06-15), Walnut v8.0-alpha. Same commit on the rig and the Mac.
- Rig: Windows 10.0.26200, Microsoft OpenJDK 21.0.12, jar built 2026-08-19,
  `C:\peanut\maths\walnut7`.
- Mac cross-check: darwin 25.5.0, Homebrew OpenJDK 26.0.2,
  `/Users/andrew/maths/walnut7`. STRATEGY-RESULTS.md (2026-08-18) recorded BRZ=466
  (791 s) here, i.e. it never showed the artifact.

Both builds are the same commit and both give 466 when BRZ runs to completion; the
1058 appeared only in the one rig run that was killed early. There is no
commit-specific or build-specific Walnut defect.

## Recommendation

- Do not open a Walnut issue. Nothing to fix on Walnut's side.
- Fix our harness (`explore/rig_bench_32gb.py`): treat a missing "Total computation
  time" line as a failed/incomplete run rather than reading `sizes[-1]` as the answer,
  and do not launch memory-heavy work concurrently with a benchmarked JVM.
- Correct the flagged cell and the "Walnut BRZ disagreement" note in
  `bench/RIG-BENCH-32GB.md` (BRZ on prism-1 is a timeout, and returns 466 when given
  enough time, not 1058).
- If we reply to John Nicol, thank him for prompting the check and let him know the
  discrepancy was on our side; no Walnut ticket is warranted.

Thanks to John Nicol for asking us to reproduce this cleanly, which is what surfaced
the harness artifact.
