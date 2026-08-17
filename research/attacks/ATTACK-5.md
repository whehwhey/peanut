# ATTACK 5 — synchronization delay of k-automatic sequences (Khodier, Open Problem 4)

**Target.** M. Khodier, *New Methods for Analyzing the Properties of Automatic
Sequences*, University of Waterloo, **MMath thesis** (Master of Mathematics in
Computer Science), 19 January 2026 (`docs/khodier2026-thesis.pdf`; UWSpace
`uwspace.uwaterloo.ca/items/41caf0b3-0bdd-4dbf-a630-cf4decabc1af`), Chapter 8,
**Open Problem 4**, restricted as `docs/OPEN-TARGETS.md` #5 does to the
in-fragment half: *k*-uniform morphisms and *k*-automatic sequences.

> **Open Problem 4.** Find an efficient algorithm for computing the synchronization
> delay of an infinite word or fixed points of non-uniform morphisms or
> `k`-automatic sequences.

(`docs/OPEN-TARGETS.md` and `README.md` call this a PhD thesis; it is a Master's
thesis. Corrected here and in `docs/OPEN-TARGETS.md`.)

**Status re-verified 2026-08-17** against the primary sources. The thesis is the
only version (no v2, no derived paper on the synchronization-delay chapter; the
self-verifying-predicate chapters became Khodier–Schaeffer–Shallit, *Self-verifying
Predicates in Büchi Arithmetic*, which does not touch Open Problem 4). Nothing
published in 2026 settles it. **Live**, with one important piece of prior art the
thesis does not cite:

> K. Klouda, K. Medková, *Synchronizing delay for binary uniform morphisms*,
> arXiv:1507.05223, **Theoretical Computer Science 615 (2016) 12–22**.

Klouda–Medková introduce the *graph of overhangs*, prove `L_max ≤ Z_min ≤ L_max +
2M − 3` (their Lemma 13, `L_max` = word-length of the longest admissible walk, `M =
max |φ(a)|`), and give the only published upper bounds on the minimal synchronizing
delay `Z_min` of a **binary** uniform morphism (their Theorem 1). They open with
*"There is no known estimate on the (minimal) synchronizing delay of a
PD0L-system"* and restrict to the binary case because *"it seems it is not easy to
find such a bound."* Their Lemma 13 is a two-sided estimate, not an exact
algorithm, and nothing there covers alphabets of size > 2 or coded (i.e. general
`k`-automatic) sequences.

---

## 1. What is being computed

Fix `k ≥ 2`, a finite alphabet `Σ`, a `k`-uniform morphism `h : Σ → Σ^k`
prolongable at `a` (`h(a)[0] = a`), and the fixed point `w = h^ω(a)`. `F(w)` is
the set of finite factors of `w`.

**Definition (Klouda–Medková Def. 2–4, equivalently Cassaigne / Frid / the thesis).**
An *interpretation* of `u ∈ F(w)` is a triple `(p, v, s)` with `v ∈ F(w)`,
`p, s ∈ Σ*` and `h(v) = p u s`. Two interpretations `(p,v,s)`, `(p',v',s')` are
*synchronized at position κ* (`0 ≤ κ ≤ |u|`) if there are `i, j` with
`h(v[0..i−1]) = p·u[0..κ−1]` and `h(v'[0..j−1]) = p'·u[0..κ−1]`. `u` has a
*synchronizing point at κ* if all of its interpretations are pairwise synchronized
at κ; `u` is *circular* if it has a synchronizing point at some κ.

* Khodier's **synchronization delay** `L` = least `l` with
  `∀u ∈ F(w). |u| ≥ l ⇒ u circular`.
* Klouda–Medková's **minimal synchronizing delay** `Z_min` = length of the longest
  uncircular factor. So **`L = Z_min + 1`**; both conventions are carried below.

---

## 2. Theorem A — the exact circularity criterion

For `u ∈ F(w)` write `Occ(u) = {p : w[p..p+|u|−1] = u}` and
`R(u) = {p mod k : p ∈ Occ(u)}`.

**Lemma 1.** Let `h` be `k`-uniform, `(p,v,s)` an interpretation of `u`, and
`0 ≤ κ ≤ |u|`. There is an `i` with `h(v[0..i−1]) = p·u[0..κ−1]` **iff**
`k | (|p| + κ)`.

*Proof.* (⇒) `|h(v[0..i−1])| = ki`, so `|p| + κ = ki`. (⇐) if `|p| + κ = ki` then
`0 ≤ i ≤ |v|` because `|p| + κ ≤ |p| + |u| ≤ |h(v)| = k|v|`; by uniformity the
prefix of `h(v)` of length `ki` is exactly `h(v[0..i−1])`, and `p·u[0..κ−1]` is
also the prefix of `h(v) = pus` of that length. ∎

So `u` has a synchronizing point at κ iff `k | (|p| + κ)` for **every**
interpretation `(p,v,s)`.

**Lemma 2.** For every `u ∈ F(w)`, `{|p| mod k : (p,v,s) an interpretation of u} =
R(u)`, and this set is non-empty.

*Proof.* (⊆) `v ∈ F(w)` means `v = w[c..c+|v|−1]`; since `w = h(w)`, `h(v) =
w[kc .. kc+k|v|−1]`, so `u` occurs in `w` at `kc + |p| ≡ |p| (mod k)`.
(⊇) Given `q ∈ Occ(u)`, `n = |u|`, put `c = ⌊q/k⌋`, `e = ⌈(q+n)/k⌉`,
`v = w[c..e−1] ∈ F(w)`. Then `h(v) = w[kc..ke−1]` contains `u` at offset
`q − kc ≡ q (mod k)`, giving an interpretation with `|p| ≡ q (mod k)`.
Non-emptiness: `Occ(u) ≠ ∅` for a factor. ∎

**Theorem A.** Let `h` be `k`-uniform with fixed point `w`, and `u ∈ F(w)`,
`n = |u|`. Then

> `u` is circular  ⟺  `∃ κ, 0 ≤ κ ≤ n`, with `p + κ ≡ 0 (mod k)` for every
> `p ∈ Occ(u)`
> ⟺  `|R(u)| = 1`, say `R(u) = {r}`, **and** `(k − r) mod k ≤ n`.

*Proof.* Immediate from Lemmas 1 and 2: a synchronizing point at κ is exactly the
condition `R(u) ⊆ {(−κ) mod k}`, and `R(u) ≠ ∅`, so κ is forced to be `≡ −r`, whose
least non-negative representative is `(k − r) mod k`; some such κ is `≤ n` iff
`(k − r) mod k ≤ n`. In particular `r = 0` (occurrences all at multiples of `k`) is
always circular, via `κ = 0`. ∎

**Corollary A1.** With `LCP(i,j)` the length of the longest common prefix of the
suffixes of `w` at `i` and at `j`,

        Z_min  =  max( D2 , D1 ),
        D2 = max{ LCP(i,j) : i, j ≥ 0, i ≢ j (mod k) },
        D1 = max{ n : some length-n factor has R = {r}, r ≠ 0, and k − r > n },

with `D2 ≥ 0` always (the empty factor occurs at every position, so `LCP(0,1) ≥ 0`
with `0 ≢ 1 mod k`), `D1 ≤ k − 2`, and `D1 = −1` when no such `n` exists. (`D2`
collects the factors with at least two occurrence residues:
if `i ≢ j (mod k)` and `LCP(i,j) = ℓ` then `w[i..i+ℓ−1]` is uncircular, and
conversely two occurrences in different classes give such a pair. `D1` collects the
residue-pure factors too short to reach their own cut position; `n < (k−r) ≤ k−1`.)

**Definition B (extension to all `k`-automatic sequences).** For a coding
`τ : Σ → Δ` and `x = τ(w)`, call a factor `v` of `x` *`k`-circular* if there is
`κ ∈ [0,|v|]` with `p + κ ≡ 0 (mod k)` for every occurrence `p` of `v` in `x`, and
put `L_k(x) = 1 + max{|v| : v a factor of x that is not k-circular}`. Theorem A says
`L_k(w) = L` for `τ = id`: Definition B *is* the classical notion when there is no
coding, and extends it to every `k`-automatic sequence — which is the half of Open
Problem 4 attacked here.

**Proposition D (codings never shorten the delay).** `L_k(τ(w)) ≥ L_k(w)` for every
coding `τ`.

*Proof.* Let `u ∈ F(w)` be uncircular, `v = τ(u)`. Every occurrence of `u` in `w` is
an occurrence of `v` in `x`, so `R_w(u) ⊆ R_x(v)`. If `|R_w(u)| ≥ 2` then
`|R_x(v)| ≥ 2` and `v` is not `k`-circular. If `R_w(u) = {r}` with `(k−r) mod k >
|u|`, then either `|R_x(v)| ≥ 2` (not `k`-circular) or `R_x(v) = {r}` and the same
cut-position obstruction applies. Either way `x` has an uncircular factor of length
`|u|`. ∎

The inequality is routinely strict, and by a wide margin (§6): the Rudin–Shapiro
morphism has `L = 1` while the Rudin–Shapiro *sequence* has `L_2 = 8`.

---

## 3. The formulas

With `FE(i,j,l) := ∀t. t < l ⇒ T[i+t] = T[j+t]` (the engine's `learnfe`/`let`
predicate), Theorem A is one line of first-order logic over `⟨ℕ,+,V_k⟩`:

```
let FE(i,j,l)  A t. t<l => T[i+t]=T[j+t]          # or: learnfe FE
let CIRC(i,n)  E c. (c<=n & c<k & (A j. $FE(i,j,n) => (E q. j+c = k*q)))
let SD(l)      A i,n. n>=l => $CIRC(i,n)
? E l. $SD(l)                                     # finite delay?
enum B $SD(l)                                     # L = min accepted l
```

`SD` is upward closed, so `min` of the enumerated set is `L`; `? E l. $SD(l)` is
`FALSE` exactly when no finite delay exists. Corollary A1 gives a second,
complement-free route:

```
let SAME(i,j)  E q. (i = j + k*q) | (j = i + k*q)
let UNC2(n)    E i,j. $FE(i,j,n) & ~$SAME(i,j)
? A n. $UNC2(n)                                   # TRUE => no finite delay
enum B ~$UNC2(n)                                  # D2 = (min accepted) - 1
```

plus `k(k−1)/2` closed sentences of the shape
`E i. i ≡ r (mod k) & (A j. FE(i,j,n) => j ≡ r (mod k))` for the `D1` term
(`n ≤ k−2`, `k − r > n`). `UNC2` is downward closed in `n` because a common factor
of length `n` gives one of every shorter length, so it is either `{0,…,D2}` or all
of `ℕ`, which is what the two commands test. The two routes are independent
formulations and are cross-checked against each other in §5.

**Uncircular case.** When no finite delay exists, Frid (and the thesis, §7.2.1)
use a weaker constant: `L_Frid = min{l : every uncircular factor of length ≥ l is
c^n for one of the letters c with h(c) = c^k}`. That, too, is one formula:

```
let ONES(i,n)  (A t. t<n => T[i+t]=c1) | (A t. t<n => T[i+t]=c2) | ...
let SDF(l)     A i,n. (n>=l & ~$CIRC(i,n)) => $ONES(i,n)
```

so the procedure never needs to know in advance whether the morphism is circular
(thesis Algorithm 2, step 2, decides that by table lookup against Frid's list of
patterns — see the erratum in §7).

**Theorem F (the algorithm and its cost).** Given a `k`-uniform morphism on `m`
letters, or any `m`-state base-`k` DFAO, the synchronization delay is computed by:
build `FE`; build `CIRC`, `SD` (or `SAME`, `UNC2`) by a bounded number of automaton
products, projections and complements over an alphabet of at most `k^3` letters;
read `L` off a one-track automaton by minimum-accepted-value. The whole computation
after `FE` costs a fixed number of operations on automata of size `O(k·|FE|)`, and
`FE` is itself constructible in time polynomial in `m` and `|FE|` (Khodier's own
`L*`-based construction, this repo's `learnfe`, `docs/LEARNFE.md`), with
`|FE| ≤ 2^{9m²}` — the bound quoted in the thesis's own Chapter 8, after
Moradi–Rampersad–Shallit. Measured behaviour is far below any of that: over the
**11 163** morphisms swept in §6.1 the largest `FE` was 95 states, the largest
`CIRC` 18 states, and the largest peak allocation 502 MB.

---

## 4. Reproducing what the sources report

Three published numbers exist for this quantity. All three come out of the
formulas of §3 unchanged.

| source | claim | our value |
|---|---|---|
| Klouda–Medková, Example 11 | Thue–Morse (`0→01, 1→10`) is circular with `Z_min = 3` | `Z_min = 3`, `L = 4` |
| Khodier thesis §7.2.1 | the 3-adic word TDC (`0→001, 1→000`) has `L = 5` (Walnut prints `12`, base 3) | `L = 5`, `Z_min = 4` |
| Khodier thesis §7.2.1 | the Rote word ROT (`0→001, 1→111`) is uncircular; Frid-type delay `L = 2` | non-circular; `L_Frid = 2` |

The thesis's `eqrot` (equality of factors for ROT) is reported at 37 states; our
`FE` is 38, which is the same automaton (this repo counts the dead state, Walnut
does not). Likewise the five index/length predicates of the thesis's Open Problem 4
encoding reproduce exactly — see §8.

## 5. How the verdicts were checked

Four independent layers, in increasing distance from the engine.

1. **Both digit orders.** Every panel entry was computed in `msd` and in `lsd`;
   `L` agreed in all 21 pairs (§6.4).
2. **Two inequivalent formulas.** The `CIRC`/`SD` route (two complementations) and
   the `SAME`/`UNC2` route of Corollary A1 (no complementation) were run against
   each other on **2 728** morphisms (`k = 2,3,4,5,6`, `m = 2`):
   `results/attack5_route2_small.json`, `results/attack5_route2_k56.json` —
   **0 disagreements**.
3. **Theorem A against the literal definition.** `explore/attack5_thmA.py`
   implements Klouda–Medková Definitions 2–3 verbatim (enumerate every
   interpretation `(p,v,s)` with `h(v) = p u s`, `v` a factor; test each cut κ by
   *string equality* `h(v[0..i−1]) = p·u[0..κ−1]`) and compares the **per-length
   count of uncircular factors** with the residue criterion, for every morphism in
   five families: `k=2,3,4,5` at `m=2` and `k=2` at `m=3` — **923 morphisms, all
   lengths up to `n = 6…8`, 0 disagreements**
   (`results/attack5_thmA_k*.json`).
4. **Brute force on a prefix, sharing nothing with the engine.**
   `explore/attack5_crosscheck.py` regenerates the fixed point as a Python string
   and finds the longest residue-mixed factor by binary search (the predicate is
   downward closed in `n`), plus the κ-clause lengths. Run against every engine
   record: **11 192 records, 11 163 sweep + 29 panel — 10 695 exact `Z_min`
   matches, 497 non-circular confirmations, 0 mismatches**
   (`results/attack5_crosscheck_*.json`), plus the 16 extremal-family values of
   §6.2 (`results/attack5_family_crosscheck.json`).

Layer 4's non-circular check demands that the brute force still exhibit a
residue-mixed factor of length ≥ 1024 in a 40 000-symbol prefix — far past the
largest finite `Z_min` in the corpus (76) — since a finite prefix cannot witness
"uncircular at every length" outright.

## 6. Results

### 6.1 Exhaustive maxima over binary `k`-uniform morphisms

Every `k`-uniform morphism on `{0,1}` prolongable at `0` (`2^{2k−1}` of them),
decided by the engine. `KM` is Klouda–Medková's Theorem 1 upper bound on `Z_min`
(their only published bound; `8` for `k=2`, `k²+3k−4` for odd prime `k`,
`k²(dk−1)+5k−4` otherwise, `d` the least divisor of `k` above 1).

| `k` | morphisms | circular | **max `Z_min`** | max `L` | KM bound | maximiser(s) | max `|FE|` | peak MB |
|---|---|---|---|---|---|---|---|---|
| 2 | 8 | 3 | **3** | 4 | 8 | `01/10` (Thue–Morse) | 15 | 0 |
| 3 | 32 | 20 | **6** | 7 | 14 | `001/100`, `011/000`, `011/110` | 38 | 25 |
| 4 | 128 | 104 | **19** | 20 | 128 | `0101/0000` | 53 | 97 |
| 5 | 512 | 464 | **20** | 21 | 36 | `00011/11000`, `00111/11100`, `01111/00000` | 63 | 192 |
| 6 | 2048 | 1949 | **76** | 77 | 422 | `011011/000000` | 73 | 292 |
| 7 | 8192 | 8000 | **42** | 43 | 66 | `0000111/1110000`, `0001111/1111000`, `0111111/0000000` | 75 | 502 |

(`results/attack5_sweep_k*m2.jsonl`, `results/attack5_summary.json`.) Each row is a
finite set of individually machine-proved statements, not a sample: every morphism
in the family was decided. Beyond binary — where no bound is published at all —
`k = 2`, `m = 3` gives 243 morphisms, 130 circular, **max `Z_min` = 4**, attained by
four morphisms (`results/attack5_sweep_k2m3.jsonl`).

Reading: the published bound is loose by a factor of 1.6–6.7 at these `k`
(8/3, 14/6, 128/19, 36/20, 422/76, 66/42), and the two regimes Klouda–Medková
separate in their Theorem 1 are real — odd prime `k` sits around `k²`, composite `k`
an order of `k` higher.

### 6.2 Two extremal families, and the order of `Z_min`

The maximisers above are not sporadic. Two families extend them, each value
individually machine-proved by the decision procedure of §3
(`explore/attack5_bigk.py`, `results/attack5_family_*.json`):

**F1 (all `k`).** `h(0) = 0 1^{k−1}`, `h(1) = 0^k`:

        Z_min = k(k − 1)

verified at `k = 2,3,4,5,6,7,8,9,10,11,12,13,15,16,17,19,20` — 17 values, no
exception. This family attains the exhaustive maximum at every odd prime `k` tested
(`k = 3, 5, 7`).

**F2 (even `k ≥ 4`).** `h(0) = (0 1^{k/2−1})²`, `h(1) = 0^k`:

        Z_min = k(k − 1)²/2 + 1

verified at `k = 4,6,8,10,12,14,16,18,20`: 19, 76, 197, 406, 727, 1184, 1801, 2602,
3611. This family attains the exhaustive maximum at `k = 4` and `k = 6`. (`k ≥ 22`
is out of reach not for mathematical reasons but because the engine caps a working
alphabet at `2^22` and this formula needs `k^5`.)

**Consequence.** Klouda–Medková's Theorem 1 gives `Z_min ≤ k² + 3k − 4` for odd
prime `k` and `Z_min ≤ 2k³ − k² + 5k − 4` for even `k` (their case (iii) with
`d = 2`), and they say nothing about tightness. F1 and F2 supply matching lower
bounds at every `k` where they were verified:

* odd prime `k ∈ {3,5,7,11,13,17,19}`: `k² − k ≤ max Z_min ≤ k² + 3k − 4` — a gap of
  exactly `4k − 4`, so at those `k` the published bound is within `1 + 4/k` of the
  truth;
* even `k ∈ {4,…,20}`: `k³/2 − k² + k/2 + 1 ≤ max Z_min ≤ 2k³ − k² + 5k − 4` — the
  same cubic order, leading constants `1/2` against `2`.

If the two closed forms persist for all `k` (they are conjectures — §9), the
odd-prime bound is asymptotically tight and the even-`k` bound is off by a factor
of 4. Either way the *order* of Klouda–Medková's Theorem 1 is now pinned from below
in both regimes, which their paper leaves open.

Every F1 value at `k ≤ 13` and every F2 value at `k ≤ 10` was reproduced by the
engine-free prefix computation (`results/attack5_family_crosscheck.json`, 16 values,
0 mismatches).

Other periods are strictly worse: at `k = 12`, `h(0) = (0111)³` gives 441 and
`h(0) = (011)⁴` gives 298, against F2's 727; at `k = 6`, `(01)³` gives 41 against 76.
At odd *composite* `k` the cubic regime is present too: `k = 9`, `h(0) = (011)³`,
`h(1) = 0^9` gives `Z_min = 169`, well past F1's `72`
(`results/attack5_family_square2.json`).

### 6.3 The κ clause is not decoration

Theorem A's second condition — a synchronizing point must sit at a cut position
`κ ≤ |u|` — is dropped by the thesis's Walnut predicate `circulartdc`, which tests
only "all occurrences congruent mod k". Census over 9 532 morphisms
(`explore/attack5_kappa.py`, `results/attack5_kappa_m2.json`,
`results/attack5_kappa_m3.json`):

| family | morphisms | delays differ | of those, injective |
|---|---|---|---|
| `m = 2`, `k = 2..6` | 2 728 | 114 | 86 |
| `m = 3`, `k = 2,3` | 6 804 | 2 | 0 |

Smallest injective (hence Klouda-legitimate) counterexample: `k = 6`,
`h(0) = 000011`, `h(1) = 001011`. Machine transcript:

```
mode msd
def T 6 2 0 000011 001011 01
let FE(i,j,l) A t. t<l => T[i+t]=T[j+t]                       states=20
let CIRCA(i,n) E c. (c<=n & c<6 & (A j. $FE(i,j,n) => (E q. j+c = 6*q)))   states=3
let CIRCR(i,n) A j. $FE(i,j,n) => (E q. (i=j+6*q | j=i+6*q))              states=4
let SDA(l) A i,n. n>=l => $CIRCA(i,n)          # Theorem A
let SDR(l) A i,n. n>=l => $CIRCR(i,n)          # the thesis's predicate
enum 40 $SDA(l)   ->  5 6 7 8 …        (Theorem A:      L = 5)
enum 40 $SDR(l)   ->  4 5 6 7 …        (residue only:   L = 4)
? A i,n. $CIRCR(i,n) => $CIRCA(i,n)    ->  FALSE
witness $CIRCR(i,n) & ~$CIRCA(i,n)     ->  WITNESS i=1 n=4
```

The witness is the factor `w[1..4] = 0001`, whose occurrences are `1, 7, 13, 19, 37,
…`, all `≡ 1 (mod 6)`: residue-pure, but the only cut position that would align them
is `κ = 5 > 4 = |u|`, so it has no synchronizing point. The literal
Definition-2/3 computation agrees (2 uncircular factors at `n = 4`, this one among
them). Since `L` feeds step 4 of the thesis's Algorithm 2 (`smallest t with
k(t−1)+b+1 ≥ L`), an under-reported `L` can change the subword-complexity formula
that algorithm outputs.

### 6.4 Named `k`-automatic sequences

`morphism` = the fixed point read over the full `m`-letter alphabet (the classical
quantity); `coded` = the `k`-automatic sequence itself (Definition B). `oo` = no
finite delay; `L_Frid` in brackets. msd and lsd agree in every row that has both.

| sequence | `k` | `m` | `|FE|` morphism | `L` morphism | `|FE|` coded | `L` coded |
|---|---|---|---|---|---|---|
| thue-morse | 2 | 2 | 15 | 4 | — | — |
| period-doubling | 2 | 2 | 8 | 3 | — | — |
| rudin-shapiro | 2 | 4 | 37 | **1** | 68 | **8** |
| paperfolding | 2 | 4 | 19 | **1** | 44 | **7** |
| cantor | 3 | 2 | 17 | oo (`L_Frid` 4) | — | — |
| mephisto waltz | 3 | 2 | 14 | 4 | — | — |
| stewart choral | 3 | 2 | 20 | 3 | — | — |
| TDC (thesis) | 3 | 2 | 8 | 5 | — | — |
| ROT (thesis) | 3 | 2 | 38 | oo (`L_Frid` 2) | — | — |
| gtm3 (`s₂ mod 3`) | 2 | 3 | 35 | 4 | — | — |
| gtm5 (`s₂ mod 5`) | 2 | 5 | 133 | 4 | — | — |
| k3m3-a | 3 | 3 | 58 | 6 | 216 | 19 |
| k3m3-b | 3 | 3 | 41 | 7 | 71 | 7 |
| prism-a | 2 | 4 | 21 | 2 | 24 | 3 |
| prism-d | 3 | 3 | 45 | 4 | 82 | 9 |
| champion-m5 | 2 | 5 | 147 | oo (`L_Frid` 6) | 199 | oo (`L_Frid` 11) |
| prism-1 | 4 | 6 | 369 | 5 | 467 | 13 |
| tail-b | 3 | 5 | 594 | 6 | 1000 | 11 |
| tail-c | 2 | 6 | 291 | 6 | 1382 | **35** |

`tail-c` is the sequence whose `FE` the direct construction cannot build at 6 GB in
either digit order (`docs/LEARNFE.md` §6.2); its `FE` here is the 1382-state
`learnfe` automaton, and the delay comes out in 16 s on top of it.

Rudin–Shapiro and paperfolding are the sharp illustration of Proposition D: the
underlying 4-letter morphism has `L = 1` (each of its four letters occupies one
residue class mod 2, so every non-empty factor is circular) while the binary
sequence it codes has `L = 8` and `L = 7`.

### 6.5 Codings never shorten the delay — and often destroy it

Proposition D checked in bulk on every `(morphism, coding)` pair for `k = 2`,
`m = 3` and every non-constant coding onto `{0,1}` — 1 701 engine runs, of which
780 have a circular morphism underneath and so test the inequality
(`explore/attack5_coded.py`, `results/attack5_coded_k2m3.jsonl`):

* **0 violations** of `L(τ(w)) ≥ L(w)`;
* 336 pairs with `L(τ(w)) > L(w)`, 304 with equality;
* **140 pairs where the morphism is circular but the coded sequence has no finite
  delay at all** — e.g. `h = (0→01, 1→00, 2→02)` with `τ = 110`: `L = 3` for the
  morphism, `oo` for the coded sequence.

The last line is the reason Definition B is not a formality: `k`-synchronization is
a property of the coded sequence, and can fail for it while holding for the
substitution underneath.

### 6.6 Classification of the non-circular binary morphisms

The engine's circular/non-circular verdict on all **10 920** binary `k`-uniform
morphisms with `k = 2..7`, against the two published classifications
(`explore/attack5_lemma15.py`, `results/attack5_lemma15.json`):

| claimed list | mismatches (injective morphisms) | mismatches (all) |
|---|---|---|
| Klouda–Medková Lemma 15 | **0** / 10 794 | 115 |
| Frid's patterns as quoted in the thesis §7.2.1 | **240** / 10 794 | 251 |
| Frid's patterns with case (iii) repaired (§7) | **0** / 10 794 | 5 |

Every residual mismatch in rows 1 and 3 is a *non-injective* morphism
(`h(0) = h(1)`), which both classifications place outside their scope:
Klouda–Medková's Definition 4 presupposes injectivity on the factor language, so
their case (i) is that precondition rather than a statement about synchronizing
points, and under the synchronizing-point definition alone such a system may well be
circular (`h(0)=h(1)=01`, `k=2`: `Z_min = 0`) or not (`h(0)=h(1)=0101`, `k=4`: the
word is `(01)^ω`, period 2 ∤ 4, so every factor is residue-mixed).

## 7. Three errata in the primary source

**E1 — `circulartdc` is missing the cut-position condition (§6.3).** The thesis's
`def circulartdc "?msd_3 Aj ($eqtdc(i,j,n) => $mod33(i,j))"` implements only
"all occurrences congruent mod k", while the definition it is implementing (quoted
in the same section, and Klouda–Medková Definition 3) asks for a synchronizing
point *at some position `0 ≤ κ ≤ |u|`*. Fixed formula: §3. Effect: `L` is
under-reported (by 1, and by 2 in a few cases) for 86 of the 2 666 injective
binary morphisms with `k ≤ 6`;
smallest example `k=6`, `h(0)=000011`, `h(1)=001011`, with an explicit witness. No
effect on the thesis's own two worked examples (TDC and ROT), where the clause never
bites.

**E2 — Frid's case (iii) is transcribed without its hypothesis on `h(1)`.** The
thesis §7.2.1 lists the uncircular binary `k`-uniform morphisms as
`(i) h(0)=(01)^x0, h(1)=(10)^x1`; `(ii) h(0)=0^k`, `h(1)` arbitrary;
`(iii) h(0)=01^{k−1}`, **`h(1)` arbitrary**; `(iv) h(1)=1^k`, `h(0) ∉ {0^k,01^{k−1}}`.
As printed, case (iii) declares Thue–Morse itself (`k=2`, `h(0)=01`) uncircular, and
misclassifies 240 of the 10 794 injective binary morphisms with `k ≤ 7`. The
hypothesis `h(1) = 1^k` must be restored to (iii) — then `w = 01^ω`, which is the
ultimately periodic word with `ρ(n)=2` that the thesis says case (iii) produces, and
the repaired list coincides exactly with Klouda–Medková Lemma 15 and with the
engine's verdicts (§6.6). This matters operationally: step 2 of the thesis's
Algorithm 2 decides circularity by matching against this list.

**E3 (minor) — the length equation in the Open Problem 4 encoding.** The thesis's
`eq1` (`v1 u v2 = h(s)`) imposes `l6+l3+l7 = 3*(i0+l0)-1`, the *last index* of
`h(s)`; the length of `h(s)` is `3*l0`. Both variants are built in §8; with the
length equation corrected `eq1` has 2986 states rather than 3054.

## 8. Head-to-head with the thesis's Open Problem 4 encoding

`explore/attack5_thesis.py` writes the thesis's index/length predicates for the same
3-adic word TDC in Peanut, one at a time (`AM_MEM_MB=6144`, one engine at a time).
State counts are this repo's convention (Walnut's, plus the dead state).

| predicate | free vars | thesis (Walnut) | Peanut | s | peak MB |
|---|---|---|---|---|---|
| `eq0` (`u = u1u2`) | 6 | 4 | **5** | 0.01 | 2 |
| `eq2` (`s = s1s2`) | 6 | 92 | **93** | 0.01 | 2 |
| `eq3` (`v1u1 = h(s1)`) | 6 | 291 | **292** | 0.02 | 5 |
| `eq4` (`u2v2 = h(s2)`) | 6 | 291 | **292** | 0.02 | 5 |
| `eq1` (`v1uv2 = h(s)`) | 8 | 3053 | **3054** | 0.66 | 551 |
| `rhs` (`eq2 ∧ eq3 ∧ eq4`) | 14 | *"we were unable to generate"* | `ERR memory budget exceeded (8282 MB)` after 13 s | — | >6144 |
| **Theorem-A route** (`FE`, `CIRC`, `SD`, min `l`) | 3 | — | **4** (the `SD` automaton) | **0.00** | **0** |

All five predicates the thesis reports a size for reproduce exactly. `rhs` fails
here as it does there — 14 free variables over base 3 is a `3^14 = 4.78×10^6`-letter
working alphabet, past this engine's `2^22` cap and past Walnut's patience — and the
route of §3 answers the same question, on the same word, with a four-state
automaton in unmeasurable time. That is the "efficient algorithm" Open Problem 4
asks for, for the `k`-uniform / `k`-automatic half of it.

Cost of the whole campaign for scale: **11 163 morphisms decided, 13 101 s of engine
CPU (1.2 s each on average), largest `FE` 1382 states (`tail-c`, via `learnfe`),
largest `CIRC` 18 states, largest peak allocation 502 MB in the sweeps and 1249 MB
in the panel (`tail-b` coded, `|FE| = 1000`).**

## 9. Honest ledger

**Known before this attack (prior art, reproduced not discovered).**
* The residue criterion for circularity under a uniform morphism — Frid [18],
  Cassaigne [12]; stated (without the κ clause) in the thesis §7.2.1, which also
  gives the Walnut recipe for the circular binary case. The thesis's §7.2.1 route
  is already an efficient algorithm *for that case*; what was missing is
  correctness in full (E1), generality (any alphabet, any coding, no a-priori
  circularity test), and the uncircular case handled by the same machinery.
* `Z_min ≤ 8 / k²+3k−4 / k²(dk−1)+5k−4` for binary `k`-uniform — Klouda–Medková
  Theorem 1 (2016), not cited by the thesis.
* Klouda–Medková Lemma 15's exact list of non-circular binary `k`-uniform systems.
* `FE` and `learnfe` — this repo, `docs/LEARNFE.md`.

**New here.**
* **Theorem A** with the cut-position clause, for arbitrary alphabets, with proof
  (§2); **Corollary A1** reducing the delay to a cross-residue longest-common-prefix
  maximum, which gives a complement-free formula (§3).
* **Definition B and Proposition D**: the extension of the delay to every
  `k`-automatic sequence (the half of Open Problem 4 that mentions `k`-automatic
  sequences, not just morphisms), the proof that a coding never shortens it, and
  the bulk verification including 140 morphisms whose coded sequence loses
  circularity entirely (§6.5).
* **Exact maxima** of `Z_min` over all binary `k`-uniform morphisms for `k = 2..7`
  (3, 6, 19, 20, 76, 42) and over all `k=2`, `m=3` morphisms (4) — each an
  exhaustive, individually machine-proved statement. Individual values are known
  (Klouda–Medková give `Z_min = 3` for Thue–Morse); an exact *maximum over a whole
  family* does not appear in the literature.
* **Families F1 and F2** with closed forms `k(k−1)` and `k(k−1)²/2 + 1`, machine-
  proved at 17 and 9 values of `k` respectively, giving the first matching lower
  bounds for Klouda–Medková's Theorem 1: at the `k` tested, their odd-prime bound is
  within `1 + 4/k` of the truth and their even-`k` bound has the right cubic order
  with the leading constant off by 4 (§6.2).
* **Errata E1, E2, E3** to the primary source, each with a machine transcript
  (§6.3, §6.6, §7).
* A **table of delays for 19 named `k`-automatic sequences**, morphism and coded,
  in both digit orders (§6.4).

**Failed / not achieved.**
* The closed forms of F1 and F2 are **conjectures**: every listed value is proved,
  the formula in `k` is not. No proof was found.
* **No general upper bound** on `Z_min` for alphabets larger than 2. Klouda–
  Medková's "there is no known estimate" survives this attack for `m ≥ 3`; the only
  general statement here is the vacuous effective one that follows from
  automaticity (`Z_min < k^{|A|}` for the one-track automaton `A`).
* **No algorithm polynomial in the DFAO size.** Theorem F is polynomial in `m`, `k`
  and `|FE|`, and `|FE|` is `2^{O(m²)}` in the worst case. A route that computes
  `max{LCP(i,j) : i ≢ j mod k}` directly from the morphism, without `FE`, was
  designed and abandoned: it reduces to matching `w` against a sliding-block image
  of itself at an arbitrary offset, which is another instance of the same problem
  (this is exactly why Klouda–Medková's `G`-admissibility condition is the hard part
  of their Lemma 13).
* **`k ≥ 22` for family F2** is blocked by the engine's `MAX_ALPHA = 2^22` cap
  (`engine/src/dfa.rs:101`), not by time or memory: the formula puts five tracks in
  play at once and `22^5 > 2^22`. Raising the cap, or restructuring the formula to
  stay at four tracks, would extend the table.
* The **non-uniform-morphism half** of Open Problem 4 is untouched — out of scope by
  the `docs/OPEN-TARGETS.md` framing, and genuinely harder (the thesis's own sketch
  needs a symbol-counting predicate whose existence is not known a priori).
* **`k = 8` and beyond were not swept exhaustively** (`2^15 = 32 768` morphisms at
  `k=8`; the `k=7` sweep alone cost 1757 s wall / 11 716 s CPU). The maxima table
  stops at `k = 7`.

**Machine-verified (what the engine actually proved).**
* 11 163 morphisms decided (circular/non-circular, and `L` when finite), each by
  compilation of a closed sentence — `results/attack5_sweep_k*m*.jsonl`.
* Two inequivalent formulas agree on 2 728 of them; msd and lsd agree on all 21
  panel pairs.
* Theorem A agrees with the literal Definition-2/3 computation on 923 morphisms, at
  every length up to 6–8, counting uncircular factors not just their maximum.
* An independent pure-Python prefix computation reproduces 11 192 records with
  0 mismatches.
* The five state counts the thesis reports for its Open Problem 4 encoding
  reproduce exactly (§8).

## 10. Files

| path | what |
|---|---|
| `explore/attack5_synch.py` | the single-sequence driver (`python3 explore/attack5_synch.py "3 2 0 001 000 01"`) |
| `explore/attack5_sweep.py` | exhaustive family sweeps, one engine per morphism |
| `explore/attack5_route2.py` | the complement-free route of Corollary A1, and the cross-check against the `CIRC` route |
| `explore/attack5_bigk.py` | the fully inlined route used for the extremal families at large `k` |
| `explore/attack5_brute.py` | engine-free: the literal Definition-2/3 computation and the residue computation |
| `explore/attack5_thmA.py` | exhaustive Theorem A validation against the literal definition |
| `explore/attack5_crosscheck.py` | engine-free prefix cross-check of every sweep record |
| `explore/attack5_kappa.py` | census of the morphisms where the κ clause changes the answer |
| `explore/attack5_lemma15.py` | engine verdicts vs Klouda–Medková Lemma 15 and Frid's list |
| `explore/attack5_panel.py` | the named-sequence panel, both digit orders, morphism and coded |
| `explore/attack5_coded.py` | Proposition D in bulk |
| `explore/attack5_thesis.py` | head-to-head with the thesis's Open Problem 4 encoding |
| `explore/attack5_summary.py` | rolls the sweeps up into §6.1 |
| `results/attack5_sweep_k*m*.jsonl` | one line per morphism: `L`, `Z_min`, `|FE|`, `|CIRC|`, peak MB, s |
| `results/attack5_summary.json` | §6.1, with the `Z_min` histograms |
| `results/attack5_family_*.json` | F1, F2 and the other period families |
| `results/attack5_thmA_*.json` | Theorem A vs the literal definition |
| `results/attack5_crosscheck_*.json` | brute-force cross-checks |
| `results/attack5_kappa_m*.json` | κ-clause census |
| `results/attack5_lemma15.json` | classification comparison |
| `results/attack5_panel_light.jsonl`, `results/attack5_panel_heavy.jsonl` | §6.4 |
| `results/attack5_coded_k2m3.jsonl` | §6.5 |
| `results/attack5_thesis_headtohead.json` | §8 |
| `results/attack5_route2_*.json` | route cross-checks |

Reproduce the headline numbers:

```
python3 explore/attack5_synch.py "2 2 0 01 10 01"     # Thue-Morse   L=4  (Z_min=3)
python3 explore/attack5_synch.py "3 2 0 001 000 01"   # TDC          L=5
python3 explore/attack5_synch.py "3 2 0 001 111 01"   # ROT          oo, L_Frid=2
python3 explore/attack5_sweep.py --k 4 --m 2 --out /tmp/k4.jsonl
python3 explore/attack5_bigk.py --family square --ks 4,6,8,10,12
python3 explore/attack5_thesis.py
```
