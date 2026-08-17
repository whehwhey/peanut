# Referee verdict on `docs/ATTACK-5.md`

Adversarial read, 2026-08-17, to the same standard as `paper/proof-verdict.md` and
`paper/attack2-verdict.md`. Every load-bearing lemma was re-derived by hand; every finite
claim was re-checked by brute force **written from scratch for this review**
(`paper/verdict-attack5/`), never through the author's `explore/attack5_*.py`; both primary
sources were re-read in full from their own PDFs (the Waterloo thesis, and
`arXiv:1507.05223` for Klouda–Medková) rather than quoted from the document; and one
piece of prior art the document does not cite was found, which changes two of its
conclusions.

## Bottom line

> **The mathematics is correct and the data reproduces exactly — every single number I
> re-computed independently came out right.** Theorem A, its two lemmas, Corollary A1,
> Definition B and Proposition D all survive re-derivation. All three errata (E1, E2, E3)
> against the thesis are real, and E1 is a genuine correctness bug in the primary source.
> The exhaustive maxima of §6.1, the κ census of §6.3, the 19-sequence panel of §6.4, the
> Proposition-D bulk run of §6.5, the classification comparison of §6.6 and the
> head-to-head state counts of §8 **all reproduce to the digit** under code that shares
> nothing with the repo.

Two findings go the other way, and one of them is large:

* **The `KM bound` column is wrong at composite `k`.** `ATTACK-5.md` (and
  `explore/attack5_summary.py:15`) transcribes Klouda–Medková Theorem 1(iii) as
  `k²(d·k − 1) + 5k − 4`. In the source `dk` is the *fraction* `k/d`, so the bound is
  `k²(k/d − 1) + 5k − 4`: **32 at `k = 4`, not 128; 98 at `k = 6`, not 422.** Nothing in
  the paper is falsified by the fix (19 ≤ 32 and 76 ≤ 98), but §6.2's headline conclusion
  inverts: the even-`k` bound is **not** "off by a factor of 4" — family F2 comes within
  an *additive* `4.5k − 5` of it at every `k` tested. The document badly understates its
  own result.
* **The ledger item "no general upper bound for `m ≥ 3`; Klouda–Medková's *there is no
  known estimate* survives this attack" is ALREADY-KNOWN to be false.** Durand–Leroy,
  *The constant of recognizability is computable for primitive morphisms*, J. Integer Seq.
  **20** (2017), arXiv:1610.05577, gives a computable bound over **any** alphabet for any
  aperiodic primitive morphism, and explicitly identifies recognizability with D0L
  circularity (`C = 2L + 1`, their §2). It is not cited anywhere in `ATTACK-5.md`. It is
  also the paper that *quotes Klouda–Medková Theorem 1 correctly* (their Theorem 6, with
  `k/d` as a fraction) — independent confirmation of the transcription error above.

Open Problem 4 itself is **still open** and the attack's deliverable stands: Durand–Leroy's
bound is a tower, not an algorithm, and nothing published settles the problem.

## Verdict per claim

| claim | verdict |
|---|---|
| Primary source quoted correctly (Khodier, Waterloo **MMath**, Ch. 8, Open Problem 4) | **CONFIRMED** verbatim from the PDF; the "MMath not PhD" correction is right |
| Open Problem 4 unsolved as of Aug 2026 | **CONFIRMED** (7 citations of Klouda–Medková, none after 2023; no derived paper; arXiv sweep clean) |
| Lemma 1 (`∃i. h(v[0..i−1]) = p·u[0..κ−1] ⟺ k \| (\|p\|+κ)`) | **PROVED** |
| Lemma 2 (`{\|p\| mod k} = R(u)`, non-empty) | **PROVED** |
| **Theorem A** (circular ⟺ `\|R(u)\|=1` **and** `(k−r) mod k ≤ n`) | **PROVED** + **MACHINE-VERIFIED** against the literal Klouda–Medková Def. 2/3 on **17 724 morphisms**, every factor at every length ≤ 8, 0 mismatches (the document's own check covered 923) |
| Corollary A1 (`Z_min = max(D2, D1)`, `D1 ≤ k−2`) | **PROVED** |
| Definition B + **Proposition D** (codings never shorten) | **PROVED**; D is a two-line argument. Bonus the document misses: `L_k(x)` depends only on `x` and `k`, not on the chosen DFAO |
| **Theorem F** (cost of the algorithm) | **WRONG in two details, unproved in a third** — alphabet is `k^5`, not `k^3` (engine panics `alphabet too large (5 vars, base 22)` on `let FE` itself); the `O(k·\|FE\|)` cost claim ignores the subset construction after each `exists`; `FE` "constructible in time polynomial in `m` and `\|FE\|`" is contradicted by `docs/LEARNFE.md` §6.2's own timeouts |
| §4 reproductions (Thue–Morse `Z_min=3`, TDC `L=5`, ROT `L_Frid=2`, `eqrot` 37 vs 38) | **MACHINE-VERIFIED**, all four |
| §6.1 exhaustive maxima, `k=2..7` + `k=2,m=3` | **MACHINE-VERIFIED** — I re-derived all **11 163** records independently; 0 disagreements, including every maximiser list and circular count |
| §6.1 `KM bound` column at `k = 4, 6` (128, 422) | **WRONG** — the true bound is 32 and 98 |
| §6.2 F1 `Z_min = k(k−1)` | **MACHINE-VERIFIED** at 17 values (mine, engine-free); closed form remains a conjecture, as the document says |
| §6.2 F2 `Z_min = k(k−1)²/2 + 1` | **MACHINE-VERIFIED** at 11 values, `k = 4..24` — two more than the document, which the engine cannot reach (mine are prefix computations, not engine decisions) |
| §6.2 "the even-`k` bound is off by a factor of 4" | **WRONG** (consequence of the transcription error). Correct statement: KM's Theorem 1 is tight to an additive `4k − 5 + k/d` in **both** regimes |
| §6.2 "F1 and F2 supply the first matching lower bounds" | **PARTLY ALREADY-KNOWN** — F2 *is* Klouda–Medková's own extremal morphism `ϕ(a) = (a b^{k/d−1})^d, ϕ(b) = a^k` from their Lemma 24. The `Z_min` values are new; the family is not |
| §6.3 κ census (2 728 → 114/86; 6 804 → 2/0) | **MACHINE-VERIFIED** exactly, including the smallest injective witness |
| **E1** (`circulartdc` drops the cut-position clause) | **CONFIRMED** — a real correctness bug in the thesis |
| **E2** (Frid case (iii) transcribed without `h(1)=1^k`) | **CONFIRMED** — 240/10 794 injective misclassifications, reproduced exactly |
| **E3** (length equation in `eq1`) | **CONFIRMED but INCOMPLETE** — `eq3` and `eq4` carry the identical error and the document does not say so |
| §6.4 panel, 19 named sequences, morphism and coded | **MACHINE-VERIFIED** — all 19 `L`, all 10 coded `L`, all four `L_Frid` values reproduce |
| §6.5 Proposition D in bulk (0 / 336 / 304 / 140) | **MACHINE-VERIFIED** exactly |
| §6.6 vs Lemma 15 and Frid's list (0/115, 240/251, 0/5) | **MACHINE-VERIFIED** exactly |
| §8 head-to-head (5, 93, 292, 292, 3054, `rhs` ERR, fixed 2986) | **MACHINE-VERIFIED** exactly, including the 551 MB peak and the 8282 MB budget breach |
| Ledger: "no general upper bound for `m ≥ 3`" | **ALREADY-KNOWN to be false** (Durand–Leroy 2017) |
| Ledger: `k ≥ 22` blocked by `MAX_ALPHA`, five tracks | **CONFIRMED** — but the block is `FE` itself, not the delay formula on top of it, so "restructuring the formula to stay at four tracks" is a harder fix than the ledger implies. `explore/attack5_bigk.py`'s docstring ("nothing ever exceeds four tracks") is wrong |
| `\|FE\| ≤ 2^{9m²}` "after Moradi–Rampersad–Shallit" | **attribution unsupported** — the thesis states it uncited in Ch. 8; the repo's own `docs/PRIOR-ART-FE.md` calls it the thesis's own |

---

## 1. The primary sources

**The thesis.** `docs/khodier2026-thesis.pdf`, `pdftotext -layout`, read in full.
Title page: "Master of Mathematics in Computer Science", Waterloo 2026 — the document's
correction of `docs/OPEN-TARGETS.md`/`README.md` (which called it a PhD) is right.
Chapter 8 is "Open problems", and Open Problem 4 reads, verbatim:

> **Open Problem 4.** Find an efficient algorithm for computing the synchronization delay
> of an infinite word or fixed points of non-uniform morphisms or `k`-automatic sequences.

Identical to `ATTACK-5.md` §1, word for word. §7.2.1's definition of a synchronization
point is Cassaigne's — `u = u1u2` plus the `∀v1,v2,∀s ∃s1,s2` clause — so the cut position
`κ = |u1| ∈ [0,|u|]` **is** part of the thesis's own definition, which is exactly why E1
bites: the Walnut predicate `circulartdc` implements only the residue half of it.
Frid's four cases, Algorithm 2, the `L = min{l ≤ k² : ∀n ≥ l, 1ⁿ is the only uncircular
word}` definition, the `eq0`…`rhs`/`synchdelay` encoding and the "we were unable to
generate the rhs automaton" sentence are all as `ATTACK-5.md` describes.

**Klouda–Medková.** I fetched `arXiv:1507.05223v1` and read it rather than trusting the
quotation. Definitions 2–4, Example 11 (`Z_min = 3` for Thue–Morse), Lemma 13
(`L_max ≤ Z_min ≤ L_max + 2M − 3`), Lemma 15 and Theorem 1 are all as described, and the
two quoted sentences are verbatim (§1 "it seems it is not easy to find such a bound"; §3
first line "There is no known estimate on the (minimal) synchronizing delay of a
PD0L-system"). One nit: their Definition 4 asks for a *positive* integer `Z` with
"`|u| > Z ⇒ u` has a synchronizing point", so their `Z_min` is `≥ 1` by fiat, where
`ATTACK-5.md` reports `Z_min = 0` for the degenerate `h(0)=h(1)=01` case. Harmless.

**Theorem 1(iii) is mis-transcribed — see §5.**

**Still open?** Semantic Scholar lists **7** papers citing Klouda–Medková, the most recent
from 2023 (*Symbolic recurrence plot for uniform binary substitutions*); none is about
computing the delay. The thesis has no v2 and no derived paper on Chapter 7/8 (the
self-verifying-predicate chapters became Khodier–Schaeffer–Shallit, *Self-Verifying
Predicates in Büchi Arithmetic*, arXiv:2507.19717 / CIAA 2025, LNCS 15981, which does not
touch Open Problem 4). arXiv full-text sweeps for "synchronizing delay" / "constant of
recognizability" return nothing from 2025–2026 in this area. **Open Problem 4 is live.**

## 2. The proofs — re-derived, all correct

**Lemma 1.** (⇒) `|h(v[0..i−1])| = ki = |p|+κ`. (⇐) `ki = |p|+κ ≤ |p|+|u| ≤ |h(v)| = k|v|`
gives `0 ≤ i ≤ |v|`; by uniformity the length-`ki` prefix of `h(v)` is `h(v[0..i−1])`, and
`p·u[0..κ−1]` is the length-`ki` prefix of `h(v) = pus`. Correct.

**Lemma 2.** (⊆) uses `w = h(w)`, so `v = w[c..c+|v|−1] ⇒ h(v) = w[kc..k(c+|v|)−1]` and the
occurrence lands at `kc+|p| ≡ |p|`. (⊇) `c = ⌊q/k⌋`, `e = ⌈(q+n)/k⌉` gives `kc ≤ q` and
`ke ≥ q+n`, so `[q,q+n) ⊆ [kc,ke)` and `|p| = q − kc ≡ q`. The `n = 0`, `k | q` corner
(where `v = ε`) still works. Correct.

**Theorem A.** By Lemma 1, "synchronized at `κ`" for a *pair* is `k | (|p|+κ)` **and**
`k | (|p'|+κ)`, so "all interpretations pairwise synchronized at `κ`" is the per-
interpretation condition `k | (|p|+κ)`; by Lemma 2 that is `R(u) ⊆ {(−κ) mod k}`, and
`R(u) ≠ ∅` forces `|R(u)| = 1`. The least admissible `κ` is `(k−r) mod k`, which must be
`≤ n`. Correct.

One point the document glides over and should state: a *strictly* pairwise reading of
Klouda–Medková Definition 3 (distinct pairs only) would make a factor with a single
interpretation vacuously circular, and the κ clause would then never fire for it.
Theorem A takes the Cassaigne / thesis reading (the condition holds *per interpretation*),
which is the right one — it is the definition the thesis itself prints, and it is what
Durand–Leroy §2 print too. Worth one sentence in §2, because the whole of E1 rests on it.

**Corollary A1.** `D2` collects `|R(u)| ≥ 2`, `D1` collects the residue-pure short factors;
`n < k−r ≤ k−1` gives `D1 ≤ k−2`; `D2 ≥ 0` because the empty factor occurs at positions 0
and 1. Correct. The count `k(k−1)/2` of extra `D1` sentences is right:
`Σ_{n=0}^{k−2}(k−1−n) = k(k−1)/2`.

**Proposition D.** Correct as written (three lines). I add: because Definition B mentions
only occurrences of factors of `x` and the modulus `k`, `L_k(x)` is an invariant of the
pair `(x, k)` alone — independent of which morphism/coding presents `x`. That is a real
merit of Definition B and the document does not claim it.

**Theorem F.** This is the one statement I could break.
* "an alphabet of at most `k^3` letters" is **false**. Confirmed empirically: at `k = 22`
  the engine dies with `alphabet too large (5 vars, base 22)` — and it dies on
  `let FE(i,j,l) A t. t<l => T[i+t]=T[j+t]` alone (`i, j, l, t` plus the adder's auxiliary
  track), before any of `CIRC`/`SD` is reached; the leaner inlined `UNC2` of
  `explore/attack5_bigk.py` fails the same way, whose docstring's "nothing ever exceeds
  four tracks" is therefore also wrong. The ledger's `k^5` figure is right; §3's `k^3` is
  not, and §3 and §9 contradict each other.
* "The whole computation after `FE` costs a fixed number of operations on automata of size
  `O(k·|FE|)`" conflates *number* of operations with *size*: each `exists` is a subset
  construction with no a-priori polynomial bound. The measured behaviour (max `CIRC` = 18
  states) is the evidence, and it is good evidence — but this is **PLAUSIBLE/empirical**,
  not proved.
* "`FE` is itself constructible in time polynomial in `m` and `|FE|`" is contradicted by
  `docs/LEARNFE.md` §6.2, where 8 of 13 sequences time out at 360 s.

## 3. What I re-computed, and how

Nothing below shares a line with `explore/attack5_*.py`.
`paper/verdict-attack5/ref.py` implements two independent computations from the prefix of
the fixed point:

* `zmin_residue` — Theorem A on a prefix. `D2` by binary search on the downward-closed
  predicate "some length-`n` factor occurs at two positions in different residue classes",
  tested in `O(N)` by a 61-bit polynomial rolling hash **with every hit verified by direct
  string comparison**, so a collision cannot inflate the answer. `D1` by exhaustive
  grouping at `n ≤ k−2`.
* `uncircular_literal` — Klouda–Medková Definitions 2/3 verbatim: enumerate every
  interpretation triple `(p, v, s)` with `v` a factor and `h(v) = p u s`, then test every
  cut `κ ∈ [0,n]` by **string equality** `h(v[0..i−1]) == p·u[0..κ−1]`, with `h(v[0..i−1])`
  built by concatenating images (no uniformity shortcut).

and `paper/verdict-attack5/eng.py` drives the engine through `explore/engine.py` on
Theorem-A formulas I wrote from Theorem A, not copied.

| check | scope | result |
|---|---|---|
| Theorem A vs literal Def. 2/3 | `k=2..7, m=2` (10 920) at every `n ≤ 8`; `k=2,3, m=3` (6 804) at every `n ≤ 6` — **17 724 morphisms**, compared as *sets of uncircular factors*, not just maxima | **0 mismatches** |
| Engine (Theorem-A route, my formulas) vs my prefix computation | `k=2..6, m=2`, all 2 728 morphisms | **0 disagreements** |
| My prefix computation vs the author's `results/attack5_sweep_k*m*.jsonl`, record by record | all **11 163** records (10 920 binary + 243 at `m=3`) | **0 disagreements** |
| Exhaustive maxima, circular counts, maximiser lists | `k=2..7` and `k=2,m=3` | identical to §6.1 in every cell |
| κ census | 2 728 + 6 804 morphisms | 114/86 and 2/0 — identical to §6.3 |
| Lemma 15 / Frid comparison | 10 920 morphisms | 0/115, 240/251, 0/5 — identical to §6.6 |
| Panel, morphism + coded + `L_Frid` | 19 sequences | every value identical to §6.4 |
| Proposition D in bulk | 243 morphisms × 6 codings, 780 testable pairs | 0/336/304/140 — identical to §6.5 |
| §8 head-to-head, fresh engine runs from my own `def` lines | `eq0..eq1`, `rhs`, corrected `eq1` | 5, 93, 292, 292, 3054, `ERR memory budget exceeded (8282 MB)`, 2986 — identical |

Scripts: `paper/verdict-attack5/{ref,sweep,thmA,xcheck,eng,kappa,lemma15,panel,propD,families}.py`.

## 4. The three errata — all real, one incomplete

**E1 (cut-position clause) — CONFIRMED, and it is a correctness bug.** The census is exact
(114 of 2 728 at `m=2`; 86 injective; 2 of 6 804 at `m=3`, both non-injective). The
smallest injective witness `k=6, h(0)=000011, h(1)=001011` reproduces: the factor
`0001 = w[1..4]` occurs at `1, 7, 13, 19, 37, …`, all `≡ 1 (mod 6)`, and the only aligning
cut is `κ = 5 > 4`. My literal Definition-2/3 computation independently marks it
uncircular, together with exactly one other length-4 factor (`0101`), and `L = 5` under
Theorem A against `L = 4` under the residue-only predicate.

One imprecision: E1 says `L` is under-reported "by 1, and by 2 in a few cases" **for the
86 injective morphisms**. The gap histogram over all 114 is `{1: 105, 2: 9}`, but restricted
to the 86 injective ones it is `{1: 86}` — the gap-2 cases are all non-injective.

**E2 (Frid case (iii)) — CONFIRMED.** As printed, `h(0) = 01^{k−1}` with `h(1)` arbitrary
declares Thue–Morse (`01/10`) and period-doubling (`01/00`) uncircular; 240 of the 10 794
injective binary morphisms with `k ≤ 7` are misclassified. Restoring `h(1) = 1^k` makes the
list coincide with Klouda–Medková Lemma 15 and with the engine, up to non-injective
morphisms. All three columns reproduce exactly.

**E3 (length equation) — CONFIRMED but INCOMPLETE.** The thesis's `eq1` imposes
`l6+l3+l7 = 3(i0+l0)−1`, the last *index* of `h(s)` rather than its length `3·l0`.
`ATTACK-5.md` fixes only `eq1`. **The same error is in `eq3` (`l4+l6 = 3(i1+l1)−1`) and
`eq4` (`l5+l7 = 3(i2+l2)−1`)** — both should read `= 3·l1` and `= 3·l2`. I built the fixed
`eq3`: **304 states** (against 292 for the thesis's version). The erratum should list all
three predicates.

## 5. The Klouda–Medková bound — the document's own error

`ATTACK-5.md` §6.1/§6.2 and `explore/attack5_summary.py:15`:

```python
return k * k * (d * k - 1) + 5 * k - 4        # WRONG
```

The source (Theorem 1(iii)) is `Z_min ≤ k²(k/d − 1) + 5k − 4` with `d` the least divisor of
`k` above 1 — `k/d` is a fraction, as is clear from the proof (Lemma 24: `|z| = k/d`,
`R_b ≤ k/d − 1`, extremal morphism `ϕ(a) = (a b^{k/d−1})^d`) and as Durand–Leroy's
Theorem 6 restates it. Corrected:

| `k` | max `Z_min` (exhaustive) | `KM` as printed in ATTACK-5 | **`KM` correct** | ratio |
|---|---|---|---|---|
| 2 | 3 | 8 | 8 | 2.67 |
| 3 | 6 | 14 | 14 | 2.33 |
| 4 | 19 | 128 | **32** | 1.68 |
| 5 | 20 | 36 | 36 | 1.80 |
| 6 | 76 | 422 | **98** | 1.29 |
| 7 | 42 | 66 | 66 | 1.57 |

"loose by a factor of 1.6–6.7" becomes **1.29–2.67**.

### 5.1 The consequence: the document understates its own result badly

With the correct bound, §6.2's "the even-`k` bound has the right cubic order with the
leading constant off by 4" is simply false. `KM(even k) = k³/2 − k² + 5k − 4` and
`F2 = k³/2 − k² + k/2 + 1`: **same leading constant**, difference exactly `4.5k − 5`. I
verified this at every `k` in `4..24`:

| `k` | 4 | 6 | 8 | 10 | 12 | 14 | 16 | 18 | 20 | 22 | 24 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| F2 `Z_min` | 19 | 76 | 197 | 406 | 727 | 1184 | 1801 | 2602 | 3611 | 4852 | 6349 |
| `KM` − F2 | 13 | 22 | 31 | 40 | 49 | 58 | 67 | 76 | 85 | 94 | 103 |

(`k = 22, 24` are beyond the engine's reach and beyond the document's table; my prefix
computation gets them.)

### 5.2 A cleaner family, and a cleaner statement

F2 is not a new family: `h(0) = (0 1^{k/2−1})²`, `h(1) = 0^k` is **Klouda–Medková's own
extremal morphism** for their Lemma 24, specialised to `d = 2` — the morphism their proof
exhibits to show `R_a ≤ k(k/d − 1) + 1` is attained. (I checked: the fixed points of
`0101/0000`, `011011/000000`, `011011011/000000000` have maximal `0`-runs 5, 13, 19,
exactly the Lemma 24 bound.) Running their general extremal morphism
`h(0) = (0 1^{k/d−1})^d`, `h(1) = 0^k` for the least divisor `d`, by prefix computation:

| `k` | 9 | 15 | 21 | 25 | 27 |
|---|---|---|---|---|---|
| `d` | 3 | 3 | 3 | 5 | 3 |
| `Z_min` (prefix, mine) | 169 | 911 | 2661 | 2521 | 5851 |
| `k²(k/d−1) + k − k/d + 1` | 169 | 911 | 2661 | 2521 | 5851 |

so the single closed form

        Z_min = k·R_a − (k/d − 1),      R_a = k(k/d − 1) + 1   (KM Lemma 24, attained)

covers F2 (`d = 2`) **and** the odd-composite case, with

        KM(k) − Z_min = 4k − 5 + k/d      uniformly.

(`k = 9` is the same morphism §6.2 reports at 169 as a one-off; it is the `d = 3` member of
this family, not a curiosity. The prime case is separate — Lemma 24's other branch,
`R_a ≤ k−1`, and F1's `k(k−1)` happens to equal `k·(k−1)` with a vanishing correction, but
F1 is not a Lemma-24 morphism: its own maximal `0`-run is 7 at `k=3`, 21 at `k=5`.)

The honest headline is therefore: **Klouda–Medková's Theorem 1 is tight to an additive
`O(k)` against a cubic bound, in both of their regimes**, attained by their own extremal
morphism. That is much stronger than what §6.2 claims, and §6.2's even-`k` claim is wrong.
Still conjectural as a formula in `k`, exactly as the document says of F1/F2; the `k ≥ 14`
values here are prefix computations, not engine decisions.

## 6. Prior art the document misses

1. **Durand–Leroy 2017** (above). They (a) give a computable upper bound on the
   recognizability constant for *any* aperiodic primitive morphism on *any* alphabet —
   `2|σ|^{6(#A)²+6(#A)|σ|^{28(#A)²}} + |σ|^{#A}`, and a sharper form when `σ` is injective on
   the language; (b) state `C = 2L + 1` between the D0L synchronizing delay and the
   recognizability constant; (c) reproduce the interpretation/synchronizing-point
   definitions verbatim; (d) quote Klouda–Medková Theorem 1 correctly. This makes the
   ledger's "no general upper bound … Klouda–Medková's *there is no known estimate*
   survives this attack for `m ≥ 3`" **wrong as stated**. The defensible version is "no
   *usable* bound": Durand–Leroy's is a tower and does not give an algorithm, and it needs
   primitivity + aperiodicity, which most of the swept corpus fails. That is still a fair
   thing to say — but it has to be said against the 2017 bound, not against a 2016 sentence.
2. **Béal–Perrin–Restivo**, *Recognizability of morphisms* (ETDS 2023, arXiv:2110.10267)
   and *Decidable problems in substitution shifts* (JCSS 2024, arXiv:2112.14499) — the
   modern decidability literature for exactly this notion, uncited.
3. The `2^{9m²}` bound is attributed in §3 to "Moradi–Rampersad–Shallit". The thesis states
   it in Chapter 8 with no citation, and this repo's own `docs/PRIOR-ART-FE.md` §2 calls it
   the thesis's own contribution. The attribution should be dropped or sourced.

## 7. Smaller defects

1. **§3 vs §9 contradict each other on the working alphabet** (`k^3` vs `k^5`); `k^5` is
   right, and the binding constraint is `FE` itself (`let FE` alone panics at `k = 22`),
   not the delay formula built on it. `explore/attack5_bigk.py`'s docstring ("nothing ever
   exceeds four tracks") is wrong.
2. **`L_Frid` is generalised silently.** The thesis's uncircular-case definition is
   `L = min{l ≤ k² : ∀n ≥ l, 1ⁿ is the only uncircular word of length n}` — a fixed letter
   `1`, and a `l ≤ k²` guard that the thesis's own `synchrorot` predicate encodes as
   `l<=9`. §3's `SDF` drops the guard and quantifies over all letters with `h(c) = c^k`.
   Both changes are improvements, but they make `L_Frid` a different constant from the
   thesis's, and the §4 table presents them as agreeing. (They do agree numerically on ROT
   — I checked: at `n ≥ 2` the only uncircular factor is `1ⁿ`, so `L_Frid = 2`.)
3. **Klouda–Medková's `Z_min` is defined to be positive** (their Definition 4), so
   `Z_min = 0` in §6.6 is the document's convention, not theirs.
4. **"923 morphisms" for the Theorem-A validation is thin** relative to what it costs: the
   whole `k=2..7` binary corpus plus `k=2,3` at `m=3` runs in under 4 minutes of plain
   Python (I did it: 17 724 morphisms, 0 mismatches). The strongest check in the document
   is also its smallest sample.
5. **§6.2's "the two regimes … are real"** survives the corrected bound (`k=6 → 76` against
   `k=7 → 42` is the visible crossover), but at `k = 4` vs `k = 5` (19 vs 20) it is not yet
   visible, and the sentence reads as if it were.

**Things I tried to break and could not.** (a) Theorem A against the literal definition on
17 724 morphisms and every factor of every length up to 8 — the strongest attack available,
and it found nothing, including on the non-injective, periodic and eventually-periodic
degenerate cases (`h(0)=h(1)=01` at `k=2` gives `Z_min = 0`; `h(0)=h(1)=0101` at `k=4` has
no finite delay — both as §6.6 says). (b) The exhaustive maxima: I re-derived all 11 163
sweep records with code that shares nothing with the repo and got zero disagreements, so
the §6.1 table is not a harness artefact. (c) The msd/lsd cross-check: 21 panel pairs, all
agreeing, and the document does not overclaim it (8 heavy entries have only one mode, and
it says so). (d) `Z_min = 0` for the Rudin–Shapiro *morphism*: each of its four letters sits
in one residue class mod 2, so `L = 1` really is right, and the jump to `L = 8` for the
coded sequence is real.

## 8. What is actually new here

* **New and correct:** Theorem A *with the cut clause* and for arbitrary alphabets;
  Corollary A1 and the complement-free route; Definition B / Proposition D (the extension
  to coded `k`-automatic sequences, and its well-definedness); errata E1, E2, E3; the
  exhaustive maxima for `k = 2..7`; the delay panel; the κ census. E1 in particular is a
  bug in a published (well, deposited) primary source, found with a machine witness.
* **Not new:** the residue criterion (Frid, Cassaigne; stated in the thesis §7.2.1, which
  already gives a Walnut recipe for the circular binary case — the document says so);
  Klouda–Medková Theorem 1 and Lemma 15; the F2 morphism itself (Klouda–Medková Lemma 24);
  the existence of a computable general bound (Durand–Leroy 2017).
* **Not established:** the F1/F2/`Fd` closed forms (conjectures — correctly labelled);
  Theorem F's cost bound; the non-uniform half of Open Problem 4 (out of scope and said so).

## 9. Check scripts (all written for this review)

```
paper/verdict-attack5/ref.py         fixed point; residue route (verified rolling hash);
                                     literal Klouda-Medkova Def 2/3 by string equality
paper/verdict-attack5/sweep.py       exhaustive Z_min, all binary k-uniform, k=2..7  -> sweep.json
paper/verdict-attack5/thmA.py        Theorem A vs the literal definition, 17 724 morphisms
paper/verdict-attack5/eng.py         my own Theorem-A engine formulas, via explore/engine.py
paper/verdict-attack5/xcheck.py      engine vs prefix, exhaustive k=2..6              -> xcheck_*.json
paper/verdict-attack5/kappa.py       kappa-clause census                              -> kappa.json
paper/verdict-attack5/lemma15.py     verdicts vs Lemma 15 / Frid printed / repaired   -> lemma15.json
paper/verdict-attack5/panel.py       the 19-sequence panel, morphism + coded + L_Frid -> panel.json
paper/verdict-attack5/propD.py       Proposition D in bulk                            -> propD.json
paper/verdict-attack5/families.py    F1, F2, the unified d-family, corrected KM bound -> families.json
```
