# Walnut determinization strategies -- what they are, and why our first
# BENCHMARKS.md was unfair without them

John Nicol (Walnut's developer) pointed out that Walnut 7.0+ ships alternative
determinization strategies -- Brzozowski's algorithm and two on-the-fly (OTF)
constructions from his [OTF library](https://github.com/jn1z/OTF) -- and that our
first benchmark round compared Peanut only against Walnut's **default** strategy
(plain subset construction, "SC"). That is not a fair comparison of the tools; it
is a comparison of Peanut against one out of six things Walnut can do. This file
documents the exact syntax (found by grep-ing `walnut7/src` and reading
`walnut7/Help Documentation` and `walnut7/CHANGELOG.md`), and `bench/README.md` /
`bench/fib.md` / the public `BENCHMARKS.md` now report the **best of every
strategy we could run**, not just the default.

## Where this comes from

* `walnut7/CHANGELOG.md`, `## [Walnut 7.0]`:
  > Determinization strategy choices: Brzozowski's algorithm; OTF-CCL and
  > OTF-CCLS; Brzozowski-OTF-CCL and Brzozowski-OTF-CCLS. Metacommands:
  > "strategy" ... and "export" ...
* `walnut7/CHANGELOG.md`, `## [Walnut 7.1]`:
  > Upgraded OTF to version 1.1.0. Several performance fixes, sometimes 10x
  > faster; for larger NFAs, sometimes 10x less memory.
* `walnut7/Help Documentation/Commands/Metacommands/[strategy].txt` (verbatim
  syntax, see below).
* `walnut7/src/main/java/Automata/FA/DeterminizationStrategies.java` -- the
  actual `Strategy` enum and dispatch logic.
* `walnut7/src/main/java/Main/MetaCommands.java` -- metacommand parsing and
  per-automaton-index strategy lookup.

## The six strategies

| name (as written in `[strategy]`) | aliases accepted | what it is |
|---|---|---|
| `SC`       | `SC`                 | subset construction (**the default**, what our first BENCHMARKS.md used exclusively) |
| `BRZ`      | `Brz`, `Brzozowski`  | [Brzozowski's algorithm](https://en.wikipedia.org/wiki/DFA_minimization#Brzozowski's_algorithm): reverse, determinize, minimize, reverse again, determinize again |
| `CCL`      | `CCL`                | OTF, Convexity Closure Lattice, no simulation |
| `CCLS`     | `CCLS`               | OTF, Convexity Closure Lattice **with** simulation |
| `BRZ-CCL`  | `BRZCCL`             | Brzozowski's reversal step, then OTF-CCL for the redetermization |
| `BRZ-CCLS` | `BRZCCLS`            | Brzozowski's reversal step, then OTF-CCLS for the redetermization |

(Underscores/dashes are ignored when matching, so `BRZ_CCLS`, `brz-ccls`,
`BRZCCLS` all resolve to the same strategy --
`DeterminizationStrategies.Strategy.fromString`.)

Per Walnut's own help text: "For a given automaton, it's not clear which
algorithm is best; you may need to try them all. Rules of thumb: usually CCLS
outperforms CCL, and BRZ-CCLS outperforms BRZ-CCL. However, if the NFA size is
very large (over 50,000 say), you will need a lot of memory and time to compute
simulation in CCLS and BRZ-CCLS, and they may crash." That is exactly what we
found empirically too -- see the results tables below.

## Exact syntax

    [strategy 0 BRZ]     ## use BRZ for intermediate automaton #0 only
    [strategy * CCLS]    ## use CCLS for every intermediate automaton
    [strategy 5 BRZ-CCLS]eval triboddpal "..."::

The metacommand is a prefix immediately in front of the command it modifies.
**Two things we had to find by testing, not documented in the help text:**

1. **Metacommands only parse on commands ending in `::`, not a single `:`.**
   A single `:` (which is what the original `bench/walnut_fe.py` and
   `bench/fib_bench.py` used to keep output terse) silently drops the
   `[strategy ...]` prefix -- Walnut prints `Metacommands are currently only
   supported for commands ending in ::` and falls back to the default. Worse:
   even when the whole session runs with `printDetails=false` (which is what a
   trailing single `:` sets), `DeterminizationStrategies.determinize()` never
   even reads `MetaCommands.getStrategy()` -- the strategy lookup itself is
   gated behind `Logging.shouldPrintDetails()`, so a `:`-terminated command is
   hard-wired to SC regardless of what metacommand precedes it. Every one of
   our reruns below uses `::`.
2. **Non-SC strategies cannot determinize DFAOs (word automata with output),
   only plain acceptors (DFAs).** `DeterminizationStrategies.determinize()`
   throws `"DFAOs are not supported for non-SC strategies"` if the automaton
   being determinized has an output alphabet. This does not block our FE
   benchmark: `morphism` / `promote` / `image` build the DFAO word automaton
   before we ever touch `[strategy]`, so those steps stay on SC by
   construction (they don't call the general determinizer with a different
   strategy already active); only the subsequent `def`/`eval` quantifier
   elimination -- which produces a plain DFA -- runs under the chosen
   strategy. We therefore place `[strategy * X]` immediately before the final
   `def`/`eval` line and nowhere earlier.
3. **`transduce` does not go through `DeterminizationStrategies` at all.**
   `walnut7/src/main/java/Automata/Transducer.java` implements its own BFS
   determinization (`transduceMsdDeterministic`) with no reference to
   `Strategy`/`strategy` anywhere in the file. The `[strategy]` metacommand has
   no effect on `transduce`, so the one OOM row in `bench/breadth.md` (`LUCAS x
   RUNSUM2`) cannot be fixed by strategy selection -- confirmed by grep, not
   assumed; see `bench/breadth.md` for the honest note.

## `[export]`, for completeness

Not used for timing (writing files costs time we don't want in the numbers),
but documented because Nicol raised it alongside `strategy`:

    [export 0 BA]    ## export the 0th intermediate automaton to BA format
    [export * TXT]   ## export all intermediate automata to TXT format

Exports land in the session's `Result/` directory as `<name>_pre_<idx>.<fmt>`
(before determinization) -- see `walnut7/Help Documentation/Commands/Metacommands/[export].txt`.

## How we reran the benchmarks

`bench/walnut_strategies.py` reruns exactly the panel rows and Tribonacci rows
that OOM'd or timed out under SC, once per non-SC strategy (`BRZ`, `CCL`, `CCLS`,
`BRZ-CCL`, `BRZ-CCLS`), same `-Xmx6g`, same 900 s wall-clock ceiling as the
original runs. Easy rows (both tools already instant under SC) were not rerun --
there is nothing for a smarter strategy to fix there. The Peanut column is
untouched: it was already measured and stands as reported.

Results: `bench/walnut_strategies_results.json` (raw), folded into
`bench/README.md`, `bench/fib.md`, and the public `BENCHMARKS.md`.

## Credit

This whole exercise -- rerunning against Brzozowski/OTF instead of only the
default -- is John Nicol's correction. He also confirmed `learnfe` (Peanut's FE
construction via self-verifying predicates) is a reimplementation of an idea from
Bachir Khodier's thesis, which he is folding into Walnut 8 directly; see the note
in `BENCHMARKS.md` and `docs/LEARNFE.md`.
