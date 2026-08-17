# Referee verdict on `docs/ATTACK-6.md`

Adversarial read, 2026-08-17, to the standard of `paper/proof-verdict.md` and
`paper/attack2-verdict.md`. Every load-bearing step was re-derived by hand; every finite
claim was re-checked by code **written from scratch for this review**
(`paper/verdict-attack6/`), never through the author's `explore/attack6_*.py`; every engine
number was recomputed from engine scripts typed for this review; and the primary source was
re-read in full (`arXiv:2106.07249v2`, `pdftotext -layout` of the arXiv PDF) rather than
quoted from the document.

## Bottom line

**The mathematics is sound and the headline results reproduce.** Proposition 1, Theorem 3
and Theorem 4 are all correct — I re-derived each and could not break any of them. Every
coding dimension in the document (3, 2, 3, 4 published; 2, 3, 5, 6, 6, and 1…10 for the
`x_r` family) reproduces exactly under three independent algorithms of mine plus fresh
engine runs, and the published-erratum claim is confirmed in both directions. The engine bug
is real and correctly described.

Four defects, one of them substantive:

1. **§3.3 "`x_6`, `x_7` (`R = 2^{r-1}`, `D = r`)" is WRONG.** `R(x_6) = 112` and
   `R(x_7) = 256`, not 32 and 64. The two words do **not** sit at equality in Theorem 3;
   the slack is 1 and 2. The author's own `attack6_brute.py` capped levels at 30 and missed
   the spike at `l = 2^r − 1`. Theorem 3 is untouched (a larger `R` only weakens the bound),
   and the "equality is attained" conclusion survives via the other seven words — but the
   cited witnesses are wrong.
2. **§3.2 "`D = 4` at 4 states (Rudin–Shapiro)" is superseded.** `def T 3 3 0 001 102 110 010`
   is a **minimal 3-state** base-3 DFAO with `D = 4` **exactly** (engine ladder: `B_4`
   nonempty, `B_5` empty). Found by exhausting all 3-letter morphisms, which the document's
   random-plus-hill-climb search did not do.
3. **§3.3 "1 411 aperiodic instances" is wrong.** `attack6_gap.py` filters only constant
   words; **124 of the 1 411 (8.8 %)** are eventually periodic. Corrected slack distribution:
   51.1 / 43.3 / 5.6 % (document: 55.4 / 39.5 / 5.1 %), so "false 45 % of the time" is 49 %.
4. **Theorem 4's stated content is achieved trivially.** Every eventually periodic word is
   `k`-automatic, so the periodic word `(B_r)^ω` on a de Bruijn cycle of order `r` is binary
   2-automatic with `D = r`. That family beats `x_r` on every axis measured here: **727
   states at `r = 10` against 4 463**, a 9.4 s certificate against 149 s, and — unlike
   `x_r` — an **exactly proved** `D = r` (`R = 2^{r-1}` in closed form, so Theorem 3 closes
   the upper bound). The real content of Theorem 4, that the family is *aperiodic*, is
   never stated.

Nothing in the ledger is fabricated and no verdict flips from "proved" to "wrong".

## Verdict per claim

| claim | verdict |
|---|---|
| §0 status table (v2 only; 2 citations; Salo's 2026 blog post is prehistory; Q10.1 open) | **CONFIRMED** |
| Finding 0 — Q10.1 negative for every Pisot ANS, base `k` included | **CORRECT but ALREADY-KNOWN** (it is Cor. 6.3 + Def. 4.3 of the source; the authors' own remark under Q10.1 shows they know it) |
| OPEN-TARGETS #6 is mis-scoped | **CONFIRMED** |
| Proposition 1 (one positional variable per level) | **PROVED** + **MACHINE-VERIFIED** (10 exhaustive tuple-set comparisons on 5 words) |
| Cost accounting `d + 2^{d-2}` vs `d + 1`; `MAX_ALPHA`; `Ast::Call` | **CONFIRMED** against the source and `engine/src/{logic,dfa}.rs` |
| Corollary 2 (finite description of `W(X)`) | **CORRECT** (superseded the same day by `docs/ATTACK-7.md`) |
| Theorem 3 — `D <= 1 + log2 R` | **PROVED**; the counting step is verbatim inside the published proof of Prop. 6.1, as the document says |
| Theorem 4 — `D(x_r) >= r`, `x_r` binary 2-automatic | **PROVED** + **MACHINE-VERIFIED** to `r = 10`; **novelty overstated** (see §6) |
| Four published values 3 / 2 / 3 / 4 reproduced | **MACHINE-VERIFIED** (engine + three independent brute forces) |
| `D(dim5) = 5` exactly, 4-state base-2 DFAO | **MACHINE-VERIFIED** (engine ladder re-run: `B_5` 7 states, `B_6` 1, peak 187 575, 1 173 MB, 32.4 s) |
| `D(dim6) = D(dim6b) = 6`; `D(x_r) = r` for `r <= 8` | **MACHINE-VERIFIED** (lower bounds certified; upper bounds reproduce under three algorithms, still uncertified — as the document says) |
| `D(x_10) >= 10`, 4 463-state DFAO | **MACHINE-VERIFIED** independently (my DFAO, my 1 024 leaf sentences, 163.8 s) |
| Erratum: published period-doubling two-`1`s bullet is necessary, not sufficient; exact form `a − 1 < 2^{k−1}`; smallest counterexample `(2,4)` | **CONFIRMED** both ways (brute force + my own engine biconditionals) |
| The two Thue–Morse bullets are exact as printed | **CONFIRMED** (both biconditionals TRUE; the strict variant is FALSE for both) |
| `x_r` minimal msd DFAO sizes 3…4 463 | **CONFIRMED** (rebuilt from the arithmetic definition) |
| Engine fix `Dfao::build_lsd` `Vec<u8>` → `Vec<State>` | **CONFIRMED** (diff read; the reported panic index 204 = 460 mod 256 is consistent) |
| §3.3 sweep: slack never negative, never > 2, ≈ 55/40/5 | **REPRODUCES** (mine: 1 242 instances, 55.3 / 41.2 / 3.5, 0 negative) |
| §3.3 "1 411 **aperiodic** instances" | **WRONG** — 124 are eventually periodic |
| §3.3 "`x_6`, `x_7` at equality, `R = 2^{r-1}`" | **WRONG** — `R = 112`, `256`; slack 1 and 2 |
| §3.2 "`D = 5` only at `m = 4`, never at `m = 3`" | **CONFIRMED, and strengthened to exhaustive** |
| §3.2 "`D = 4` at 4 states" | **WRONG** — 3 states suffice (base 3), engine-exact |
| §3.2 "`R <= k m²`, 0 violations, largest ratio 0.66" | **CONFIRMED as scoped** (0.656 in the author's sample; 0 violations in 271 k further instances of mine, where the ratio reaches 0.75) |
| 7/7 `enum` vs brute-force tuple-set comparisons | **CONFIRMED**, and re-done over the *full* enum box (the author's script derives the box from the engine's own output) |

---

## 1. The primary source

Verified from the v2 PDF, not from the document. Question 10.1, verbatim:

> **Question 10.1.** Is there an addable ANS `S` such that some `S`-automatic word has a
> winning shift with unbounded sums?

and the final remark of §10, verbatim:

> Besides these theoretical problems, it would be of interest to try to extend the practical
> computations in Section 6 to examples where the winning shift has larger coding dimension.
> We expect that the methods scale very badly, but this intuition has often turned out to be
> wrong in the setting of automatic theorem-proving; see [12, Remark 3] and [22] …

Both quoted correctly by `ATTACK-6.md`. (The paper's own cross-reference is off by one: the
practical computations are in **§7**, "Automata for the Winning Shifts of Certain Automatic
Words"; §6 is "`S`-codable Winning Shifts". The document silently uses §7 elsewhere, which
is right.)

Also checked verbatim and correctly used: Def. 4.2 (coding dimension), Def. 4.3
(`S`-codable = weakly `S`-codable **and** finite coding dimension), Prop. 5.3 and its
`w_{def} = u·d·u_d·e·u_de·f` leaf pattern, Prop. 6.1 and its proof (which contains the
"final branching … corresponds to `2^{B_i−1}` right special words of a common length"
counting step that Theorem 3 quantifies), Thm. 6.2, Cor. 6.3, Props. 6.5/6.8/6.9, the §3.1
sentence "we are unaware of any ANS using the radix order which is not Pisot and is
addable", the §7 `factorEq`/`isRS`/`extRS2..4` Walnut code (`extRS4` has 8 free variables —
4 levels + 4 positions — confirming `d + 2^{d-2}`), "Walnut quickly ran out of memory",
"Both computations finished in a few seconds", the four coding dimensions 3/3/4/2, and the
three Thue–Morse / period-doubling characterisation bullets.

**Still open, and still unextended.** Semantic Scholar's citation list for `arXiv:2106.07249`
contains exactly the two papers the document names (Ollinger–Shallit 2024;
Rigo–Stipulanti–Whiteland 2022), neither touching §10. arXiv lists v1 (14 Jun 2021) and v2
(17 Feb 2022) only, journal-ref *Inform. and Comput.* 285.B (2022) 104883. Salo's 3 Jun 2026
post *On the shattering-winning shift and Mycielski ideals* is a historical note (Mycielski
1969/1994, Anstee–Rónyai–Sali 2002) and does not touch Question 10.1. §0 is accurate.

## 2. Finding 0 — correct, but it is the source's own corollary

The chain is right: `k`-automatic ⟹ `O(n)` factor complexity (Cobham) ⟹ (Prop. 6.1,
`O(x)` transitive) finite coding dimension. And Cor. 6.3 says `W(X)` is `S`-codable for every
Pisot `S`, which by Def. 4.3 *includes* finite coding dimension. So the base-`k` case of
Question 10.1 is settled negatively **by a definition unfolding of a corollary the authors
state**. The authors also say so themselves in the sentence immediately after Question 10.1
("If the answer is negative, then one can drop the assumption of sublinear complexity in
Theorem 6.2 … Cassaigne's example provides only a comparable ANS with this property"), which
only makes sense if they know that every sublinear-complexity case is already negative.

The document's ledger lists Finding 0 under "**New here** … *Proved (assembly of published
results)*". The assembly is correct and the mis-scoping call on OPEN-TARGETS #6 is right, but
"new" is the wrong shelf: **ALREADY-KNOWN**.

## 3. Proposition 1 — re-derived, correct, and machine-checked past the document's range

I re-derived both directions. The (⇒) direction grafts `T_1, T_2` under a branch at level
`a_1`; the (⇐) direction uses that the two depth-`(a_1+1)` nodes carry `u·b·v_b` of length
`a_2`, each a factor, hence each occurring at some `m_b`, with `FE(n,m_b,a_1)` and
`x[m_b+a_1] = b`. The subtree hanging at `m_b` is exactly a strategy tree for
`a_2 < … < a_d` with root word `x[m_b, m_b+a_2)`; the initial chain of length `a_1+1`
spelling `u·b` is what the grafting replaces. Both directions check out for every `d`. The
binary alphabet is used exactly once, in `T[m1+a1] != T[m2+a1]` ⟹ the two branch letters are
*the* two letters. **PROVED.**

Machine check (mine, `eng.py` + `enumcheck.py`), comparing the compiled `L_d = E n. B_d`
against my top-down brute force **as sets, over the full enumeration box**:

| word | `d = 1` | `d = 2` | `d = 3` |
|---|---|---|---|
| Thue–Morse | — | 37 = 37 | 14 = 14 |
| period-doubling | — | 17 = 17 | — |
| paperfolding | — | 40 = 40 | 15 = 15 |
| Rudin–Shapiro | — | 57 = 57 | 44 = 44 |
| **`dim5`** (new here) | 18 = 18 | **69 = 69** | **69 = 69** |

Ten exact set equalities, zero differences. The author's `attack6_charact.py` derives the
comparison box from `1 + max` of the engine's *own* output, so a missing largest tuple could
not be seen; re-doing it against the declared `enum` bound changes nothing.

The cost accounting is right. `Ast::Call` (`engine/src/logic.rs:508`) allocates one
`self.newvar()` per argument and projects it away, so a `d+1`-argument call costs `2d+2`
tracks, i.e. `k^{2d+2}` symbols; `MAX_ALPHA = 1 << 22` (`engine/src/dfa.rs:101`); `3^14 >
2^22 > 3^12`, so base 3 loses the sentence at `d = 6` exactly as claimed. The
state-count emptiness criterion is sound and I verified the premise directly: `let ALL(a,b)
true` and `let NONE(a,b) false` both minimise to `states=1`, `let LT(a,b) a<b` to 3. In
every ladder I ran where both criteria were computable they agreed.

## 4. Theorems 3 and 4

**Theorem 3.** Re-derived: the `2^{d-1}` level-`a_d` nodes all branch, so their labels are
right-special factors of length `a_d`; two of them diverge at the first branch level where
their paths differ and the two children there carry distinct letters, so the labels are
pairwise distinct; hence `2^{d-1} <= s_x(a_d) <= R`. Correct, including the `d = 1`,
`a_1 = 0` edge case (`s_x(0) = 1`). **PROVED.** The document is honest that this is the
counting step of Prop. 6.1 "with the constant kept" — it is, verbatim.

**Theorem 4.** Re-derived. (a) `x_r[m] = bit_{d_0}(d_1)` for `d_0 < r` is a function of the
two lowest base-`2^r` digits, so `x_r` is `2^r`-automatic hence 2-automatic. (b) `n = Σ u_i 2^i`
puts `u` at `2^r n`. (c) branching at `0,…,r−1` has leaf set `{0,1}^r ⊆ L`. All correct.
**PROVED.**

Independent machine checks:

* I rebuilt the minimal msd base-2 DFAO of `x_r` from the arithmetic definition (window of
  the last `2r` bits, my own Moore minimisation) and replayed it against `x_r` on `2^18`
  positions: **3, 7, 16, 41, 85, 206, 459, 1026, 1974, 4463** for `r = 1..10`, 0 mismatches —
  the document's table exactly.
* `D(x_r) = r` for `r = 1..8` under my top-down search, and it survives a much wider level
  window than the author's `LMAX = 30`: my backward DP gives `D = 4, 5, 6, 7` for
  `r = 4,5,6,7` with levels up to 40, 40, 70, **140**.
* `D(x_8), D(x_9), D(x_10) >= 8, 9, 10` re-certified against my DFAO with my own leaf
  sentences: 256/256, 512/512, **1024/1024 TRUE** in 5.1 s, 27.0 s, 163.8 s.

### 4.1 But the stated content of Theorem 4 is free

Every eventually periodic word is `k`-automatic. Take `B_r` a de Bruijn cycle of order `r`
and `w_r = (B_r)^ω`. Then `{0,1}^r ⊆ L(w_r)`, so `D(w_r) >= r`, and `sup { D(x) : x` binary
2-automatic `} = ∞` — Theorem 4's conclusion, with no construction. Worse for the document's
framing, `w_r` dominates `x_r` on everything measured (`paper/verdict-attack6/debruijn.py`,
`cert.py`):

| `r` | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|
| `x_r` minimal msd states | 85 | 206 | 459 | 1026 | 1974 | **4463** |
| `w_r` minimal msd states | 25 | 47 | 94 | 170 | 359 | **727** |
| `w_r` certificate time | — | — | — | 0.4 s | — | **9.4 s** (vs 163.8 s) |

and `D(w_r) = r` is **proved exactly**, which `D(x_r) = r` is not: `p_{w_r}(n) = 2^n` for
`n <= r` and `2^r` for `n >= r`, so `s_{w_r}(n) = 2^n` for `n < r` and `0` for `n >= r`, i.e.
`R(w_r) = 2^{r-1}` in closed form, and Theorem 3 closes `D <= r`. My brute force confirms
`D(w_r) = r` and `R = 2^{r-1}` for `r <= 8`, and the engine certifies `D(w_10) >= 10` from a
727-state DFAO.

What is genuinely new in Theorem 4 is that `x_r` is **aperiodic** — which the document never
says, and which is the property that makes the statement non-trivial. Recommend restating
Theorem 4 as "…an explicit *aperiodic* family…", and demoting "certifies coding dimension 10"
from a record to what it is: a scaling demonstration for the certificate route.

## 5. Defects

### 5.1 §3.3: `x_6`, `x_7` are not at equality in Theorem 3 — WRONG

> "All four classical words sit at equality … and so do `x_6, x_7` (`R = 2^{r-1}`, `D = r`)"

`R(x_r)` has a spike at `l = 2^r − 1` that the author's `LMAX = 30` cut off
(`attack6_brute.py` uses `L = max(LMAX, 2r+4)`, i.e. 30 for `r <= 6`). Measured on prefixes of
`2^17`/`2^19` with levels to 70/140 (`R` from my own extension-dictionary pass; the `D`
row is the value all three of my algorithms return):

| `r` | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| `R(x_r)` (`attack6_brute.json`, levels <= 30) | 1 | 3 | 8 | 20 | 28 | 32 | 64 | 128 |
| `R(x_r)` (referee, levels <= 140) | 1 | 3 | 8 | 20 | **48** | **112** | **256** | ≥ 128 |
| `argmax l` | 0 | 3 | 7 | 15 | 31 | 63 | 127 | (255) |
| `D(x_r)` | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
| slack `1+⌊log2 R⌋ − D` | 0 | 0 | 1 | 1 | 1 | **1** | **2** | — |

So `R(x_6) = 112` (not 32) and `R(x_7) = 256` (not 64), and neither word is at equality.
`2^{r-1}` is the *plateau* value of `s_{x_r}(l)` for `5 <= l < 2^r − 1`, not the maximum.
Theorem 3 is untouched — a larger `R` only weakens the bound, and no `D` value changes — and
the surrounding conclusion ("equality is attained") still holds via Thue–Morse,
period-doubling, paperfolding, Rudin–Shapiro, `dim5`, `dim6` and `dim6b`, all of which I
verified sit at equality, and (with proof) via the de Bruijn family of §4.1. Only the two
cited witnesses are wrong.

### 5.2 §3.2: coding dimension 4 needs only 3 states — WRONG as stated

The table's "`D = 4` at 4 (Rudin–Shapiro)" is beaten. Exhausting **all** `k`-uniform
`m`-letter morphisms with non-constant binary coding (`paper/verdict-attack6/exh.py`,
`exh4.py`):

| `k`, `m` | instances | max `D` | witness |
|---|---|---|---|
| 2, 2 | 16 | 3 | Thue–Morse |
| 3, 2 | 64 | 3 | `0→001, 1→110` (Mephisto) |
| 2, 3 | 1 458 | 3 | — |
| **3, 3** | **39 366** | **4** | **`0→001, 1→102, 2→110`, coding `010`** |
| 2, 4 | 229 376 | 5 | `0→01, 1→12, 2→33, 3→20`, coding `0011` (≅ `dim5`) |

The base-3 witness is a **minimal** 3-state DFAO (Moore: all three classes distinct) and the
engine ladder settles `D = 4` **exactly**:

```
def T 3 3 0 001 102 110 010
learnfe FE states=109
C1 47   C2 284   C3 134   C4 4 states (TRUE)   C5 1 state (FALSE)     466 MB, ~7 s
```

`R = 9`, so Theorem 3 gives `D <= 4` too. My brute force and naive tree enumerator agree
(`D = 4`, levels `(0,1,4,7)`). The corrected "state cost of a coding dimension" row is
`D = 2` at 2, `D = 3` at 2, **`D = 4` at 3**, `D = 5` at 4, `D = 6` at 7.

Two of the document's own claims in the same sentence come out **stronger** than stated,
because the exhaustion settles them: `D = 5` is *provably* unreachable at `m <= 3` for
`k ∈ {2,3}` (not merely "a dedicated search never found it"), and `D = 6` is *provably*
unreachable at `m = 4, k = 2`.

### 5.3 §3.3: the gap sample is not aperiodic

`attack6_gap.py`'s only filters are `len(set(coding)) >= 2` and `len(set(x)) >= 2`; neither
excludes eventual periodicity. Applying Morse–Hedlund (`p(n) <= n` for some `n`) to all 1 411
stored records: **124 are eventually periodic** (123 of them with `D = 1`). Restricting to
the 1 287 genuinely aperiodic instances:

| slack | 0 | 1 | 2 |
|---|---|---|---|
| document (labelled "aperiodic") | 782 (55.4 %) | 557 (39.5 %) | 72 (5.1 %) |
| referee, aperiodic only | 658 (51.1 %) | 557 (43.3 %) | 72 (5.6 %) |
| referee, own sample (n = 1 242, seed 20260817) | 687 (55.3 %) | 512 (41.2 %) | 43 (3.5 %) |

Conclusions unchanged (slack never negative, never > 2, `D = 1 + ⌊log2 R⌋` false about half
the time — 49 %, not 45 %); the word "aperiodic" is simply not earned.

### 5.4 Cosmetic

* §4 says `x_7` has **460** states; §3.4 and `attack6_lib.xr_dfao_line` say **459** (I get
  459). The quoted panic (`len is 204`, `204 = 460 mod 256`) does come from a 460-state
  automaton, so one of the two numbers is from a pre-minimisation build.
* §4: "checks it against the arithmetic definition on `2^18` positions" — the code checks
  `2^min(2r+4,18)`, i.e. `2^18` only for `r >= 7` (64 positions at `r = 1`).
* §3.2: "`B_2` then exceeds 4 GB" for base-3 `dim6` — `attack6_dim.py` ran it at
  `mem_mb = 6144`, so the budget breached was 6 GB.
* §3.2: "largest ratio `R/(k m²) = 0.66`" is correct for the 1 411-instance sample (0.656),
  but `dim5` itself sits at `24/32 = 0.75`, which is also the exhaustive maximum over all
  229 376 base-2 4-letter morphisms. No violation of `R <= k m²` in 271 k further instances
  of mine.
* §3.2's "~500 hill-climb runs of 250 candidates each" is unverifiable:
  `results/attack6_search.json` is truncated to the top 200 records by `attack6_search.py`.

## 6. Two things I tried to break and could not

* **The erratum.** Enumerating every winning pair for the period-doubling word from a `2^19`
  prefix over `0 <= a_1 < a_2 <= 34` gives exactly
  `(0,2),(0,4),(1,5),(0,8),(1,9),(2,10),(3,11),(0,16),…,(7,23),(0,32),(1,33)` — the
  document's list. The paper's printed condition (`a − 1 <= 2^{k−1}`) has exactly four false
  positives in that range, `(1,3),(2,6),(4,12),(8,24)`, smallest `(1,3)` 0-based = `(2,4)`
  1-based; the strict form has none. My own engine sentences: implication **TRUE**, printed
  form as a biconditional **FALSE**, strict form as a biconditional **TRUE**, `W2(1,3)`
  **FALSE**, `W2(0,2)` **TRUE**, `W2(2,6)` **FALSE**. Both Thue–Morse bullets are exact
  biconditionals as printed (**TRUE**), and their strict variants are **FALSE** — so the slip
  really is isolated to the period-doubling bullet, as claimed. (Guard the quantifiers: without
  `b < c` the Thue–Morse biconditional is vacuously false at `b = c = 0`.)
* **The engine ladder.** Re-run from scripts typed for this review, the four classical words
  give `FE` = 15 / 8 / 44 / 68 states, ladder peaks 392 / 61 / 1 711 / 7 148, `D` = 3 / 2 / 3
  / 4, in 0.1 / 0.3 / 0.7 / 0.4 s — the document's table to the digit. `dim5` gives `B_5` 7
  states / `B_6` 1 state, peak 187 575, 1 173 MB, 32.4 s (document: 187 575, 1.17 GB, 37 s).

## 7. Verification code (all written for this review)

```
paper/verdict-attack6/ref.py          word generation, RS sets, top-down memoised D, naive tree enumerator
paper/verdict-attack6/fast.py         integer-encoded D/R for exhaustive sweeps
paper/verdict-attack6/dp.py           third algorithm: backward DP over level suffixes
paper/verdict-attack6/eng.py          engine driver + my transcription of the B_d ladder
paper/verdict-attack6/run_classics.py D and R for the whole catalogue          -> classics.json
paper/verdict-attack6/run_cross.py    top-down vs naive-tree cross-check
paper/verdict-attack6/run_xr.py       my own minimal msd DFAO for x_r          -> xr_states.json
paper/verdict-attack6/run_xr_dim.py   D, R for x_1..x_8                        -> xr_dim.json
paper/verdict-attack6/errata.py       every winning pair/triple; both published characterisations
paper/verdict-attack6/enumcheck.py    engine `enum` vs brute force over the full box
paper/verdict-attack6/gap.py          my own Theorem-3 tightness sweep         -> gap.json
paper/verdict-attack6/exh.py          exhaustive k,m <= 3 morphisms            -> exhaustive_small.json
paper/verdict-attack6/exh4.py         exhaustive k=2, m=4 (229 376 instances)  -> exh_k2m4.json
paper/verdict-attack6/check_m3.py     the 3-state base-3 D=4 word
paper/verdict-attack6/debruijn.py     periodic de Bruijn family                -> debruijn.json
paper/verdict-attack6/cert.py         independent leaf-sentence certificates   -> cert.json
```

## 8. Ledger, as the referee would write it

**Known before (correctly cited):** the game and `W(X)` (Salo–Törmä); `P_v`, coding
dimension, Thms 5.5/5.6/6.2, Props 6.1/6.5/6.9, Cor. 6.3, the `factorEq`/`isRS`/`extRS_d`
encoding, and the four coding dimensions (Peltomäki–Salo §7); Cobham; Cassaigne.

**Known before, but shelved as new by the document:** Finding 0 (Cor. 6.3 unfolded);
Theorem 3's counting step (inside the published proof of Prop. 6.1 — the document says so);
and the *statement* of Theorem 4, which periodic de Bruijn words give away.

**New and correct:** Proposition 1 (the `d+1`-variable recursion, a real `2^{d-2} → 1`
improvement on the source's own encoding, machine-verified against brute force on five
words); the erratum; the aperiodic family `x_r`; the concrete small DFAOs of coding
dimension 5 and 6; the engine fix.

**Wrong:** `R(x_6) = 2^5`, `R(x_7) = 2^6` and the resulting "`x_6, x_7` at equality";
"`D = 4` at 4 states"; "1 411 aperiodic instances".

**Not established (document is honest about all of these):** Question 10.1 itself; Problem
10.2 for non-uniform substitutions; exact `D` for `dim6`, `dim6b`, `x_r` (`r >= 4`); a lower
bound on DFAO size per coding dimension; closed forms for the paperfolding and Rudin–Shapiro
winning shifts.
