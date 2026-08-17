# Peanut ROADMAP — queued 2026-08-17 (do NOT start until Andrew says go; token budget)

Guiding idea (from Andrew, spitballing with Claude): the professor's dream tool is not "a
faster Walnut". It is: *"I have a conjecture. I want to know if it's true; if false I want the
counterexample; if true I want a proof I can put in a paper — and I don't want to learn a
syntax to get it."* Everything below serves that sentence.

## 1. Natural-language front end via a LOCAL, lightweight LLM  (Andrew: "really like this")
- Constraint: must not contend with the engine. So: small model, CPU/GPU-light, invoked only
  when the engine is idle, and NEVER in the proof path (it only *drafts* formulas; the engine
  decides; the user sees and confirms the formula before it runs).
- Candidates (pick by benchmark on the Mac, in this order): Qwen2.5-Coder-1.5B-Instruct or
  3B via Ollama/llama.cpp (Metal), Llama-3.2-3B-Instruct, SmolLM2-1.7B. Use
  grammar-constrained decoding (llama.cpp GBNF) with Peanut's formula grammar so output is
  always syntactically valid; few-shot with docs/COMMANDS.md examples + the GUI library.
- UX: a "Ask in English" box in the Playground: NL -> proposed formula (editable) -> run ->
  result explained back in one English sentence (template-based, not LLM, for honesty).
- Guardrail: the LLM never claims a verdict; only the engine's TRUE/FALSE is shown as a result.

## 2. Citeable, exportable proofs
- `certify NAME` -> a self-contained proof certificate: the sentence, the sequence definition,
  every intermediate automaton (or a hash + regeneration recipe), the final automaton, and a
  tiny independent checker (Python, no engine code) that re-verifies the certificate.
- Human-readable proof transcript (Markdown/LaTeX) suitable for a paper appendix, with a
  citation block (Peanut version, git commit, date, command transcript).
- Stretch: Lean 4 export of the automaton + decision (prior art exists for Walnut->Lean).
- Every result in research/ gets a certificate retroactively.

## 3. Discovery mode
- Point Peanut at a sequence: it enumerates a template library (squares, cubes, overlaps,
  palindromes, borders, FE size, critical exponent ladder, recurrence, special factors,
  d/c/gap spectra from explore/), machine-proves each, and reports what is *distinctive*
  vs a reference panel — the theorem-generator line, productised (docs/THEOREMGEN.md).
- Output: a "sequence report" (Markdown + GUI page) with proved facts, conjectures the
  fragment cannot decide, and open-problem hooks from docs/OPEN-TARGETS.md.

## 4. Library / packaging
- Rust crate `peanut-core` (crates.io) + Python package `peanut` (PyPI) wrapping
  explore/engine.py with a typed API; `pip install peanut`, `cargo add peanut-core`.
- Stable JSON API (already in gui/serve.py) documented as the integration surface.
- Homebrew formula / single binary releases via GitHub Actions (build matrix mac/linux).

## Extra detail on 1-4 (from the full spitball text)
- (1) NL *and LaTeX* statements ("the Thue-Morse word contains no overlaps", "every factor of
  length n of the Fibonacci word occurs at a position = 0 mod n") -> formal predicate SHOWN for
  confirmation -> verdict. Peanut checks the logic; the model only writes it.
- (2) one button "give me the certificate": Lean/Isabelle proof term or checkable trace so a
  referee never has to trust Peanut. Precedent: Walnut->Lean exports.
- (3) discovery output = "here are 30 true statements about your sequence that aren't in the
  literature, ranked by how surprising they are, with proofs" + the sweep machinery as a
  first-class tool ("run this predicate over 10,000 random sequences, show the distribution").
- (4) `pip install peanut`, `peanut.prove("...")`; Jupyter cell rendering automaton + tape
  inline; SageMath interface (combinatorics-on-words people live in Sage); OEIS hook
  ("is A382296 automatic? prove these properties").

## 5. Fields to shape it for (reach, not breadth)
- Number theory / digit problems (digit sums, Zeckendorf, automatic sets in the plane).
- Symbolic dynamics / tilings (2-D automatic sets, substitution tilings; Shapes view is halfway).
- Formal verification / model checking (Buchi machinery; they'd value engine + certificates).
- Combinatorial game theory (winning shifts, rusty numbers; Fokkink-Joshi are game theorists).
- Music theory / procedural generation (Thue-Morse, paperfolding in composition; turtle+tape
  + "hear this sequence" audio view).
- Teaching (first-year discrete maths: watch a proof being built).
- Explicit NON-goal: bolting on unrelated maths (graph theory, partitions). Depth in the
  fragment + reach into adjacent fields, not breadth into fields the logic can't serve.

## 6. Two ambitious engine ideas
- BDD / symbolic subset construction (MONA approach) to blow through remaining intermediate
  blowups (Rust: biodivine-lib-bdd or CUDD binding).
- "Why" mode: FALSE -> the SMALLEST counterexample, painted on the tape, automaton path
  highlighted; TRUE -> the structure of the proof (which sub-automata mattered, which collapsed).
  A tool that explains its answers becomes part of how the professor thinks.

## Suggested first three when budget allows
NL/LaTeX-to-logic with shown translation; exportable proof certificates; pip install peanut
+ Jupyter/Sage integration. Those turn "a better Walnut" into "the tool people outside the
field reach for".

## Also queued from earlier sessions
- Walnut-compat + Walnut test-suite diff, transducers, negative bases, Ostrowski, fuzz-diff,
  GUI sweep — IN FLIGHT as round 5 (wf_2afa5f6a); results land in docs/WALNUT-COMPAT.md,
  docs/BREADTH.md, docs/FUZZ.md, gui/TESTING.md.
- BDD/symbolic determinization (only if learnfe leaves cases uncracked).
- Second research note: greedy 3-sum-free (ATTACK-4) to the JIS authors.
- arXiv submission of the linear-subsequence note once Shallit endorses.

## Rules for executing this list
- One workflow per item, Opus for design/core, Sonnet for docs/tests; referee anything that
  claims a mathematical result; sync the public repo (tools/export_public.py) at the end.
- Don't start until Andrew says the token budget allows.
