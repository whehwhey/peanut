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

See `BENCHMARKS.md`: parity with Walnut on easy inputs; on the FE-hard inputs and on Tribonacci numeration Peanut answers where Walnut 8-dev runs out of memory at the same 6 GB (Khodier's Tribonacci FE: 3.1 s / 133 MB, or 0.07 s via `learnfe`).

## Quick start

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

This repo is the engine and tooling. The research write-up — the equality-of-factors
blow-up result, the benchmark data, and the paper — lives in a separate (currently
private) research repo; a public link will be added here when it is released.

## Architecture

Engine internals — DFA/DFAO representation, the digit-order machinery, the
determinization ladder, the learner — are in `docs/ARCHITECTURE.md`.

## The name

Peanut is named after the creator's son.

## Creator

Andrew Hingston.

## License

MIT. See `LICENSE`. Copyright (c) 2026 Andrew Hingston.
