# Bench: Peanut vs Walnut, equality-of-factors predicate

> Peanut's defaults changed on 2026-08-19 (parallel bitset determinization,
> antichain evaluation of closed sentences). The current head-to-head table is
> `../BENCHMARKS.md` and `SPEED-ROUND6.md` ("Final defaults"); anything quoting
> older Peanut seconds predates that change.

Like-for-like harness comparing Peanut's `let`/`learnfe` construction of

    FE(i,j,l) := A t. t < l => T[i+t] = T[j+t]

against a plain Walnut (`morphism` / `promote` / `image` / `def`) formulation of the
same predicate, on the same sequences, same memory budget.

## Setup

1. Build Peanut: `cd ../engine && cargo build --release` (from the repo root).
2. Get Walnut (github.com/Walnut-Theorem-Prover/Walnut), build it to
   `target/Walnut-all.jar`, and point `WALNUT_HOME` at that checkout (defaults to
   `../walnut` next to this repo). `JAVA` defaults to `java` on `PATH`.

## Run

```
python3 bench/walnut_fe.py bench/panel.json bench/results.json
```

`bench/panel.json` is the sequence panel: an array of `[name, "def ..."]` pairs, each
a morphism/coding definition in Peanut's `def` syntax (see `docs/COMMANDS.md`).
`walnut_fe.py` runs both engines on each sequence and writes a JSON array of
per-sequence rows (`ours`, `ours_how`, `ours_s`, `walnut`, `walnut_s`,
`walnut_ms_sum`) plus a summary table to stdout. Results are not checked into this
repo; re-run to reproduce them on your own hardware.

## Reading the output

`ours`/`walnut` are minimal DFA state counts (Peanut's includes the dead state, so
`ours = walnut + 1` whenever both finish: a useful independent cross-check).
`walnut` can also be `timeout`, `OOM`, or `error`. `ours_how` records which
determinization strategy (`mode`/`cap`) succeeded.
