# Research produced with Peanut

Everything here was produced with the engine in this repository, then **adversarially
refereed** by an independent pass (own brute-force code, hand re-derivation, counterexample
search, primary-source re-check). The referee reports are in `referee-verdicts/` and are the
authority on status. Read the verdict before citing a claim.

| item | status (per referee) | where |
|---|---|---|
| **A tight family for the state complexity of linear subsequences** — answers Open Problem 1 of Moradi–Rampersad–Shallit (arXiv:2512.10017): `sc(h_m(ni)) = m²(n−1) − m·max(2^{p−1}, r)` for odd n, so ≥ ½·n·m² | **PROVED** (`attack2-verdict.md`) | `notes/linear-subsequence-note.pdf`, `attacks/ATTACK-2.md` |
| Greedy 3-sum-free sequences, JIS 2025 Conjecture 17 | **PROVED**, on a larger range than stated (`attack4-verdict.md`) | `attacks/ATTACK-4.md` |
| Shifted Thue–Morse state complexity (their Open Problem 2): structure theorem, exact algorithm, non-2-regularity evidence | theorems **PROVED**; problem still open (`attack1-verdict.md`) | `attacks/ATTACK-1.md` |
| Fokkink–Joshi rusty numbers a=(1,d), d=3..12 (Integers 2026) | **MACHINE-VERIFIED**, 249 forms (`attack3-verdict.md`) | `attacks/ATTACK-3.md` |
| Synchronization delay of k-automatic sequences (Khodier Open Problem 4) | characterisation theorem **PROVED**; problem open (`attack5-verdict.md`) | `attacks/ATTACK-5.md` |
| Peltomäki–Salo Q10.1 / Problem 10.2 | 10.1 **already known**; 10.2 ladder proved, core already in source (`attack6/7-verdict.md`) | `attacks/ATTACK-6.md`, `ATTACK-7.md` |
| **Size of the equality-of-factors automaton** (Khodier Open Problem 1): full-output generalised Thue–Morse |FE| = Θ(p³) proved; singleton coding ≥ p⁴ proved; Λ not polynomial; random-ensemble sweep to m=7 (typical ~m³); no exponential family found; **conjecture |FE| = poly(m) OPEN** | mixed — see `proof-verdict.md`, `proof3-verdict.md` | `target1/`, `notes/fe-size.pdf` (draft) |
| Numeration systems (Fibonacci/Tribonacci/Pell) and the Tribonacci FE benchmark | **MACHINE-VERIFIED** (`numeration-verdict.md`) | `../BENCHMARKS.md`, `../docs/NUMERATION.md` |

Status of the write-ups: `notes/linear-subsequence-note.pdf` is a preprint being sent to the
problem's authors; `notes/fe-size.pdf` is a working draft. Neither has been externally
peer-reviewed. Author: Andrew Hingston. Comments, prior-art pointers and counterexamples are
very welcome — open an issue.
