# Peanut

A first-order-logic decision procedure for k-automatic sequences, with an adaptive
determinization ladder, guess-and-verify construction of hard predicates, a hard
resource guard, and a GUI.

Given a k-uniform (or k,l-uniform, msd or lsd) morphism and a coding, Peanut compiles
sentences of first-order logic over `<N, +, <, V_k>` extended by the resulting
automatic sequence `T` into deterministic finite automata — compilation *is* the
decision procedure and, for closed sentences, the proof. Quantifiers become
existential projection plus subset construction; `A i,j,k. ...` sentences about a
sequence a computer can only sample become questions a computer can answer exactly,
for all `n`, in finite time.

On top of that core sits `learnfe`, a guess-and-verify active-learning construction
for the "equality of factors" predicate

    FE(i,j,l) := A t. t < l => T[i+t] = T[j+t]

which for some sequences blows up by orders of magnitude under direct forward
construction between an intermediate automaton and its minimal final size. Peanut
answers `FE` two ways: an adaptive ladder (small forward subset-construction cap,
fail fast into Brzozowski/reverse-first determinization, escalate) for the general
case, and `learnfe`, which never builds the blow-up intermediate at all — see
`docs/LEARNFE.md` for the construction and its correctness argument.

## Research

See `research/README.md`: results obtained with Peanut, each with its referee verdict (two open problems from 2026 papers answered; several advanced; one honest 'already known').

## Benchmarks

See `BENCHMARKS.md`. Updated 2026-08-19 for the new defaults (frontier-parallel bitset determinization, antichain evaluation of closed sentences): 2.5x-13x faster than Peanut's own previous default on the equality-of-factors panel, and faster than the best per-case Walnut 8-dev strategy on seven of the eight base-k cases and on Tribonacci — Walnut's CCLS still wins `tail-c` (10.6 s against 16.1 s). Known limitations: `docs/KNOWN-ISSUES.md`.

Earlier reading, still true, from the 2026-08-18 correction after John Nicol's note: with the right per-query strategy (Brzozowski/OTF, `[strategy]`) Walnut 8-dev answers every hard case too; Peanut's advantages are one default that needs no strategy tuning, mostly-faster times where both finish, `learnfe` (Khodier's construction, also coming to Walnut 8), a scriptable guarded runner for sweeps, and the GUI. State counts agree to the dead state everywhere.

## Quick start

One command (needs Rust + Python 3): `./start.sh` — builds the engine if needed, starts the web GUI, opens your browser. Add `--port 8080` to change the port.

Manual:

```
cd engine && cargo build --release
echo 'mode msd
def T 2 2 0 01 10 01
? A i. T[i]=T[i]
quit' | ./target/release/peanut
```

From Python, always through the resource-guarded runner (never call the binary
directly — see `docs/GUARD.md`):

```python
import sys; sys.path.insert(0, "explore")
import engine
r = engine.run("mode msd\ndef T 2 2 0 01 10 01\n? A i. T[i]=T[i]\n")
print(r.stdout)
```

Full stdin command language: `docs/COMMANDS.md`. Python API: `docs/PYTHON-API.md`.
Three runnable examples: `examples/`. Benchmark harness vs Walnut: `bench/README.md`.

## GUI

```
python3 gui/serve.py            # http://0.0.0.0:7373, prints the LAN URL
```

Standard library only, no build step. See `gui/README.md`.

## Results

The research produced with Peanut is in `research/`: preprint notes (including the answer to
Open Problem 1 of Moradi–Rampersad–Shallit on linear subsequences of automatic sequences), the
equality-of-factors sweep and proofs, seven open-problem attack write-ups, and the adversarial
referee reports that state exactly what is proved, machine-verified, or open. Benchmark data is
in `BENCHMARKS.md`.

## Architecture

Engine internals — DFA/DFAO representation, the digit-order machinery, the
determinization ladder, the learner — are in `docs/ARCHITECTURE.md`.

## The name

Peanut is named after the creator's son.

## AI usage

Peanut was designed and largely written with Claude (Anthropic, `claude-fable-5`) via Claude Code, directed by the author; see `research/README.md` for the full declaration covering the research results.

## Creator

Andrew Hingston.


## How to cite

If you use Peanut in research, please cite it (GitHub shows a "Cite this repository" button from `CITATION.cff`). A DOI will be added once the software paper is published.

> Hingston, A. *Peanut: a decision procedure for automatic sequences.* 2026. https://github.com/whehwhey/peanut

## License

MIT. See `LICENSE`. Copyright (c) 2026 Andrew Hingston.
