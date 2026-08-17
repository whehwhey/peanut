# ATTACK 6 — Winning shifts of k-automatic words (Peltomäki–Salo, Question 10.1)

**Target (OPEN-TARGETS #6).** J. Peltomäki, V. Salo, *Automatic winning shifts*,
arXiv:2106.07249v2 (17 Feb 2022) = **Inform. and Comput. 285.B (2022) 104883**, §10:

> **Question 10.1.** Is there an addable ANS `S` such that some `S`-automatic word has a
> winning shift with unbounded sums?

and, from the same section,

> "Besides these theoretical problems, it would be of interest to try to extend the
> practical computations in Section 6 to examples where the winning shift has larger
> coding dimension. We expect that the methods scale very badly, but this intuition has
> often turned out to be wrong in [Walnut-style] automatic theorem-proving."

**Headline.** Question 10.1 has **no content in the base-`k` fragment**: for every Pisot
numeration system — base `k` included — the answer is *negative*, and that is already a
corollary of the source paper's own Corollary 6.3 (equivalently Prop. 6.1 + Cobham).
OPEN-TARGETS #6 is therefore **mis-scoped** (§0). What *is* in-fragment is the authors'
explicit final request, and that is what this attack does: reproduce their four published
coding dimensions, push the record from **4 to 10**, give an effective algorithm and a
sharp upper bound, prove that the coding dimension is unbounded over binary 2-automatic
words (while finite for each), correct one published off-by-one, and cut the positional
variables in the authors' own encoding from `2^{d-2}` to `1`. One engine bug was found
and fixed on the way (§4).

---

## 0. Status re-check (2026-08-17) — and why the target as scoped is already settled

| item | finding |
|---|---|
| arXiv:2106.07249 | latest is **v2, 17 Feb 2022**; published *Inform. and Comput.* 285.B (2022) 104883. No v3, no erratum. |
| citations | Semantic Scholar lists **two**: Ollinger–Shallit, *The Repetition Threshold for Rote Sequences* (2024), and Rigo–Stipulanti–Whiteland, *On extended boundary sequences of morphic and Sturmian words* (2022). Neither touches §10. |
| 2026 follow-up | Salo's blog post *On the shattering-winning shift and Mycielski ideals* (3 Jun 2026) is about the **prehistory** of the notion (Mycielski 1994; Anstee–Rónyai–Sali 2002), not about Question 10.1. |
| Question 10.1 | **open in general**; closed (negatively) for every *Pisot* ANS, hence for base `k` — see Finding 0. |
| §7 computations | never extended by anyone; coding dimension **4** (Rudin–Shapiro) is still the published maximum. |

### Finding 0 (mis-scoping). The base-`k` case of Question 10.1 is negative, and the source paper proves it.

*Definitions* (paper §4): for `y ∈ N^N`, `∑y = ∑_i y_i`; the **coding dimension** of
`Y ⊆ N^N` is the least `d` with `∑y ≤ d` for all `y ∈ Y`. "Unbounded sums" = infinite
coding dimension.

*Chain.* Cobham (1972): every `k`-automatic word has factor complexity `O(n)` —
"sublinear" in this paper's sense, the sense it uses in Cor. 6.3 (where Parry-automatic
words are called sublinear) and in Prop. 6.1's proof (where Cassaigne's theorem is quoted
as "sublinear ⟺ bounded number of right-special factors"). Peltomäki–Salo Prop. 6.1: a
transitive subshift with sublinear factor complexity has a winning shift of finite coding
dimension; `O(x)` is transitive. Hence for every `k`-automatic `x`, `W(O(x))` has
**bounded** sums. The paper states the same conclusion directly as Cor. 6.3 for all Pisot
numeration systems, and base `k` is Pisot.

So a positive answer to Question 10.1 needs an addable ANS carrying an automatic word of
*superlinear* complexity. The paper's only example of an ANS-automatic word whose winning
shift has infinite coding dimension is Cassaigne's `σ: a ↦ abab, b ↦ b` (Prop. 6.5,
complexity `Θ(n²)`) — and it then **proves that this ANS is not addable** (Prop. 6.9).
It also records (§3.1) that the authors are "unaware of any ANS using the radix order
which is not Pisot and is addable". Question 10.1 is therefore blocked on an open question
about *numeration systems* — produce an addable non-Pisot ANS — not about automatic
sequences. Nothing in `<N,+,V_k>` can decide that, so the in-fragment part of
OPEN-TARGETS #6 is exactly the paper's final remark.

---

## 1. Definitions, and the predicate we compile

Let `x` be a binary `k`-automatic word, `X = O(x)`; then `L(X) = Fac(x)`. A **strategy
tree with branchings at levels `a_1 < … < a_d`** is a complete binary tree of depth
`a_d+1` in which every node at depth `a_j` has two children, every other node exactly one,
and every root-to-leaf label lies in `L(X)`. By Peltomäki–Salo Prop. 5.3, for `v = 1^d`,

```
(a_1,…,a_d) ∈ P_{1^d}(W(X))  <=>  a strategy tree with branchings at a_1<…<a_d exists,
```

i.e. `0^{a_1} 1 0^{a_2-a_1-1} 1 … 1 0^ω ∈ W(X)`. `W(X)` is hereditary and the alphabet is
binary, so the coding dimension is

```
D(x) = max { d : some strategy tree has d branchings }.                              (D)
```

`FE(i,j,l) := A t. t<l => T[i+t]=T[j+t]` is the equality-of-factors predicate that
`learnfe` constructs (`docs/LEARNFE.md`); it is the paper's `factorEq(l,i,j)`.

### Proposition 1 (one-position recursion)

Put `B_0(n) := true` and, for `d ≥ 1`,

```
B_d(a_1,…,a_d,n) := a_1<a_2 & E m1,m2. FE(n,m1,a_1) & FE(n,m2,a_1)
                                     & T[m1+a_1] != T[m2+a_1]
                                     & B_{d-1}(a_2,…,a_d,m1) & B_{d-1}(a_2,…,a_d,m2)
```

(the conjunct `a_1<a_2` is dropped when `d = 1`). Then `B_d(a_1,…,a_d,n)` holds **iff**
there is a strategy tree with branchings at `a_1<…<a_d` whose root word — the common
prefix of length `a_1` of all its leaves — is `x[n, n+a_1)`. Consequently

```
D(x) = max { d : E a_1,…,a_d,n. B_d(a_1,…,a_d,n) }.
```

*Proof.* Induction on `d`. `d = 1`: `B_1(a,n)` says the length-`a` factor at `n` has two
distinct one-letter right extensions, i.e. it is right special — a one-branching tree.
(This is the paper's `isRS(a,n)`.)

`d−1 → d`, (⇒): let `u = x[n,n+a_1)`. `FE(n,m_b,a_1)` makes each `m_b` an occurrence of
`u`, and `x[m_1+a_1] ≠ x[m_2+a_1]` are the two letters of the alphabet. By induction each
`m_b` carries a strategy tree `T_b` with branchings at `a_2<…<a_d` whose root word
`x[m_b,m_b+a_2)` extends `u·x[m_b+a_1]`. Grafting `T_1, T_2` under a branch at level `a_1`
gives the tree; its leaves are the leaves of `T_1, T_2`, hence factors.

(⇐): given a tree with root word `u = x[n,n+a_1)`, its two depth-`(a_1+1)` subtrees have
root words `u·b·v_b` (`b ∈ {0,1}`) of length `a_2`, each a factor, hence each occurring at
some `m_b`; then `FE(n,m_b,a_1)`, `x[m_b+a_1] = b`, and that subtree witnesses
`B_{d-1}(a_2,…,a_d,m_b)`. ∎

**Cost.** The paper's §7 encoding `extRS_d(a_1,…,a_d, n_1,…,n_{2^{d-2}})` unrolls the same
recursion but carries one positional variable per subtree it has already unrolled
(`extRS2`: 1, `extRS3`: 2, `extRS4`: 4), so it has `d + 2^{d-2}` free variables — 22 at
`d = 6`, an alphabet of `2^22` before any projection, which is exactly this engine's
`MAX_ALPHA`; `extRS4` (8 variables) was the largest predicate the authors built.
Proposition 1 keeps **one** positional variable at every level: `d+1` free variables,
`2^7` symbols at `d = 6`.

**Emptiness from the state count.** `Dfa::minimize` trims, so a minimal automaton has one
state iff its language is `∅` or everything; for `d ≥ 2` the conjunct `a_1<a_2` excludes
"everything". Hence `states=1` on the `let B_d` line ⟺ `B_d ≡ false`. This matters
because a *call* `$B_d(a_1,…,a_d,n)` costs `d+1` fresh variables on top of the `d+1` it is
applied to (`engine/src/logic.rs`, `Ast::Call`), so the sentence `E a_1..a_d,n. $B_d(…)`
needs `k^{2d+2}` symbols and exceeds `dfa.rs`'s `MAX_ALPHA = 2^22` at `d = 6` in base 3.
Both criteria are computed whenever both are affordable and
`attack6_lib.dimension_from_stdout` *asserts* they agree; they always did.

### Corollary 2 (the whole of `W(X)`, effectively)

For binary `x` the winning shift is determined by the sets `P_{1^d}(W(X))`, `d ≤ D(x)`.
Proposition 1 gives each of them as a `d`-track automaton `L_d(a_1,…,a_d) := E n. B_d(…)`,
and Theorem 3 below bounds the `d` at which to stop. So "compute a finite description of
`W(X)`" — Problem 10.2 restricted to `k`-uniform substitutions, where it is subsumed by
the paper's Theorem 5.6 — is implemented here, not merely known to be possible.
`results/attack6_charact.txt` contains the exported `L_2`, `L_3` automata for the four
classical words.

---

## 2. Two theorems

### Theorem 3 (effective upper bound; the quantitative form of Prop. 6.1)

Let `s_x(n)` be the number of right-special factors of length `n` (with `s_x(0) = 1` for a
non-unary word: the empty word is right special) and `R(x) = sup_{n≥0} s_x(n)`, finite for
every automatic `x` by Cobham + Cassaigne. Then

```
D(x) <= 1 + log2 R(x).
```

*Proof.* In a strategy tree with `d` branchings the `2^{d-1}` nodes at level `a_d` all
branch, so their labels are right-special factors of the common length `a_d`, and they are
pairwise distinct (two of them differ at the first branch level where their paths
diverge). Hence `2^{d-1} <= s_x(a_d) <= R(x)`. ∎

This is the counting step inside the paper's Prop. 6.1, with the constant kept. Both sides
are computable from a DFAO for `x`: `D` by the ladder of Proposition 1, which terminates
by this bound; `R` by the sentences `E a,i_1<…<i_q. RSF(a,i_1) & … & RSF(a,i_q)` with
`RSF(a,i) := B_1(a,i) & A j. j<i => ~FE(j,i,a)` selecting first occurrences
(`explore/attack6_rmax.py`). The encoding costs `k^{q+1}` symbols, so it settles small
`R` only: `R = 4` (Thue–Morse), `2` (period-doubling), `6` (paperfolding) come out exactly
and agree with the prefix computation, while Rudin–Shapiro is certified only as `R >= 10`
(the `q = 11` sentence exceeds 4 GB; the prefix value is 12).

### Theorem 4 (the coding dimension is unbounded over the class)

For `r >= 1` let

```
x_r[2^r n + i] = bit_i(n)   (0 <= i < r),        x_r[2^r n + i] = 0   (r <= i < 2^r).
```

Then `x_r` is a binary 2-automatic word and `D(x_r) >= r`. Hence
`sup { D(x) : x binary, 2-automatic } = ∞`, although `D(x) < ∞` for each `x` (Theorem 3).

*Proof.* (a) *Automaticity.* Write `m` in base `K = 2^r` as `… d_2 d_1 d_0`. Then
`i = m mod K = d_0`, `n = ⌊m/K⌋`, and `bit_i(n) = bit_{d_0}(n mod K) = bit_{d_0}(d_1)`, so

```
x_r[m] = bit_{d_0}(d_1)  if d_0 < r,   else 0,
```

a function of the two lowest base-`K` digits: the `K`-uniform morphism on states `(c,b)`
with `δ((c,b), e) = (e, bit_e(c)·[e<r])`, output `b`, start `(0,0)`, generates it. So `x_r`
is `2^r`-automatic, hence 2-automatic (Cobham).

(b) *`{0,1}^r ⊆ L(x_r)`.* For `u ∈ {0,1}^r` take `n = ∑_i u_i 2^i`; then
`x_r[2^r n, 2^r n + r) = u`.

(c) *The tree.* Branch at levels `0,1,…,r−1`. A node at level `j < r` is a word
`v ∈ {0,1}^j`; both `v0` and `v1` are prefixes of members of `{0,1}^r ⊆ L(X)`, hence
factors, so the node branches; the `2^r` leaves are exactly `{0,1}^r ⊆ L(X)`. Therefore
`1^r 0^ω ∈ W(O(x_r))` and `D(x_r) >= r`. ∎

`explore/attack6_lib.xr_dfao_line` builds the **minimal msd base-2 DFAO** of `x_r` (3, 7,
16, 41, 85, 206, 459, 1026 states for `r = 1..8`) and checks it against the arithmetic
definition on `2^18` positions before emitting it.

**Remark.** Step (c) used only `{0,1}^r ⊆ L(x)`: *any* binary automatic word whose
language contains every word of length `r` has `D >= r`. Every high-dimension example
found by search works this way — except the base-3 `dim6` of §3.2, whose optimal tree
(levels `0,1,2,3,4,6`) skips a level and is not a full block.

**Remark (large alphabets are cheap).** Over an `A`-letter alphabet the configuration
`(|A|−1) 0^ω` is winning as soon as every letter occurs, so the coding dimension is
`>= |A|−1` for trivial reasons. Theorem 4 is about the binary case, where nothing is free.

---

## 3. Results

### 3.1 The four published values, reproduced

| word | `D` (engine) | published §7 | `FE` states | ladder peak | time |
|---|---|---|---|---|---|
| Thue–Morse | 3 | 3 | 15 | 392 | 0.1 s |
| period-doubling | 2 | 2 | 8 | 61 | 0.3 s |
| regular paperfolding | 3 | 3 | 44 | 1 711 | 0.9 s |
| Rudin–Shapiro | 4 | 4 | 68 | 7 148 | 0.5 s |

`peak` is the maximum subset-construction size over the whole ladder (`docs/COMMANDS.md`).
The paper reports that feeding Walnut the direct Theorem-5.5 formula for the Thue–Morse
word "quickly ran out of memory", and that their reformulated `extRS` computation took "a
few seconds"; here the entire ladder to `d = 6`, `FE` construction included, is 0.1 s.

### 3.2 New values, and the record

| word | `k` | states | `D` | how | `R` | time |
|---|---|---|---|---|---|---|
| Cantor `0→010, 1→111` | 3 | 2 | 2 | ladder (exact) | 2 | 1.9 s |
| Mephisto waltz `0→001, 1→110` | 3 | 2 | 3 | ladder (exact) | 4 | 0.3 s |
| `x_1, x_2, x_3` | 2 | 3, 7, 16 | 1, 2, 3 | ladder (exact) | 1, 3, 8 | 0.6/1.8/16.2 s |
| **`dim5`** `0→02,1→33,2→21,3→10`, coding `1010` | 2 | 4 | **5** | ladder (exact) | 24 | 36.7 s |
| **`dim6b`** (8 letters, base 2) | 2 | 8 | **≥ 6** (= 6 by brute force) | tree certificate | 55 | 0.7 s |
| **`dim6`** (7 letters, base 3) | 3 | 7 | **≥ 6** (= 6 by brute force) | tree certificate | 50 | 0.0 s |
| `x_4 … x_10` | 2 | 41…4463 | **≥ 4 … ≥ 10** (`= r` by brute force for `r ≤ 8`) | tree certificate | | ≤ 149 s |

Two mechanisms are used, both machine-checked:

* **ladder (exact)** — the `B_d` chain of Proposition 1 gives `D` exactly: `B_D` nonempty,
  `B_{D+1}` empty. `dim5` is a **4-state base-2 DFAO of coding dimension 5**, one above the
  published record, settled this way (`B_5` has 7 states, `B_6` has 1, peak 187 575,
  1.17 GB, 37 s). Its optimal levels are `(0,1,2,3,4)` and `s(4) = 16 = 2^4`: the language
  contains all 32 binary words of length 5.
* **tree certificate** — `D >= d` is a *finite* statement (§1): exhibit the `2^d` leaves,
  check the Prop. 5.3 pattern combinatorially, and check each leaf is a factor with one
  closed sentence `E n. T[n]=b_0 & … & T[n+ℓ]=b_ℓ` per leaf. No `FE`, no wide alphabet.
  This certifies **coding dimension 10** for `x_10` — 1 024 leaf sentences against a
  4 463-state DFAO in 149 s — and 6 for the two `dim6` words, where the full ladder is out
  of reach (`FE` alone reaches 2 218 states on the base-3 `dim6`, and `B_2` then exceeds
  4 GB).

The upper bounds `D(dim6) = D(dim6b) = 6` and `D(x_r) = r` (`r ≤ 8`) are brute force
(levels ≤ 30), not certified; the lower bounds are proved (`x_r`: Theorem 4) or certified.

**State cost of a coding dimension** (smallest DFAO found, over a 1 411-instance random
sample plus ~500 hill-climb runs of 250 candidates each):
`D = 2` at 2 states, `D = 3` at 2 (Thue–Morse), `D = 4` at 4 (Rudin–Shapiro),
`D = 5` at 4 (`dim5`; a dedicated `m <= 4` search found `D = 5` only at `m = 4`, never at
`m = 3`, whose best is 4), `D = 6` at 7 (base 3) / 8 (base 2). Since `R >= 2^{D-1}` and
`R <= k m²` in everything measured (0 violations in the 1 411-instance sample, largest
ratio `R/(k m²) = 0.66`), the state count should grow like `2^{D/2}`; no lower bound is
proved.

### 3.3 Theorem 3 is attained, but usually loose by 1

`explore/attack6_gap.py` samples uniform morphisms (`k ∈ {2,3}`, `m ≤ 6`, random binary
coding; 1 411 aperiodic instances) and compares `D` with `1 + ⌊log2 R⌋`:

| slack `1+⌊log2 R⌋ − D` | 0 | 1 | 2 |
|---|---|---|---|
| instances | 782 (55.4 %) | 557 (39.5 %) | 72 (5.1 %) |

Never negative (as Theorem 3 requires — an earlier off-by-one in the definition of `R`,
which omitted `n = 0`, showed up here as 84 apparent violations and was fixed), and never
more than 2 in this sample. All four classical words sit at equality
(`3 = 1+2`, `2 = 1+1`, `3 = 1+⌊log2 6⌋`, `4 = 1+⌊log2 12⌋`), and so do `x_6, x_7`
(`R = 2^{r-1}`, `D = r`), so no bound of the form `D <= f(R)` beats Theorem 3 — but `R`
does not *determine* `D`: the natural conjecture `D = 1+⌊log2 R⌋` is false 45 % of the
time. The largest `D` in the random sample was 4 (largest `R`: 28); reaching 5 and 6
required hill-climbing (`explore/attack6_search.py`).

### 3.4 The family `x_r`

| `r` | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| minimal msd DFAO states | 3 | 7 | 16 | 41 | 85 | 206 | 459 | 1026 | 1974 | 4463 |
| `D >= r` proved (Thm 4) + certified | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| certificate time (s) | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.2 | 0.8 | 4.6 | 25.4 | 148.8 |
| `D` exact, engine ladder | 1 | 2 | 3 | — | — | — | — | — | — | — |
| `D` brute force (levels ≤ 30) | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | — | — |
| `FE` states (`learnfe`) | 12 | 109 | 851 | — | — | — | — | — | — | — |

Past `r = 3` the ladder is blocked by `learnfe`, not by the `B_d` chain: `x_r` has runs of
`2^r − r` zeros, the LCP oracle saturates, and `FE` grows fast (12 → 109 → 851). The
certificate route is unaffected because it never builds `FE`.

### 3.5 Erratum in the published description of `W(X)` for the period-doubling word

§7 states, for the period-doubling word (1-based positions):

> If `x` in `W(X)` contains exactly two occurrences of `1` at positions `a, b` with
> `a < b`, then `b − a = 2^k` for some `k ≥ 1` and `a − 1 ≤ 2^{k−1}`.

The implication is **true** (engine: `TRUE`), but it is not a characterisation, as the
surrounding text presents it. The engine's counterexample, from `witness`, is `a_1 = 1,
a_2 = 3` (0-based) = `a = 2, b = 4` (1-based): `b − a = 2 = 2^1` and `a − 1 = 1 = 2^{k−1}`,
so the right-hand side holds, yet `0 1 0 1 0^ω ∉ W(X)`. Exactly:

```
0^{a_1} 1 0^{a_2-a_1-1} 1 0^ω ∈ W(X_pd)  <=>  a_2 − a_1 = 2^k (k ≥ 1)  and  a_1 < 2^{k-1}
```

— i.e. `a − 1 < 2^{k−1}`, strict, in the paper's notation. Machine-verified as an
`A a_1,a_2` biconditional (`TRUE`), and confirmed independently by brute-force enumeration
of every winning pair with `a_2 ≤ 33`:
`(0,2),(0,4),(1,5),(0,8),(1,9),(2,10),(3,11),(0,16),(1,17),…,(7,23),(0,32),(1,33)`.
The two Thue–Morse statements (two 1s, three 1s) are **exact** as printed — both verified
here as biconditionals — so the slip is isolated to the period-doubling bullet.

---

## 4. Verification

Every verdict above was recomputed by `explore/attack6_brute.py`, which shares **no code**
with the engine: it iterates the morphism in Python, collects the factors of a `2^16`–`2^17`
prefix with a sliding window, and computes the coding dimension **twice** —

* `dim_tuples`: for each level tuple `(a_1<…<a_d)` (pruned by `s(a_j) >= 2^{j-1}`), run the
  backward set recursion `V ← {u ∈ RS[a_j] : u0, u1 ∈ prefixes(V)}` from `V = RS[a_d]`;
* `dim_dp`: a backward dynamic programme over the *maximal* achievable node sets, which
  never enumerates tuples.

The two agree with each other and with the engine on every sequence in the catalogue
(`results/attack6_brute.json` vs `results/attack6_dim.json`). Three further cross-checks:

* **Full tuple lists.** `explore/attack6_charact.py` compares the engine's `enum` output
  for `L_2`, `L_3` against the brute-force list of winning tuples, as sets: Thue–Morse
  37 and 14 tuples, period-doubling 17, paperfolding 40 and 15, Rudin–Shapiro 57 and 44 —
  **equal in all seven comparisons** (`results/attack6_charact.txt`).
* **Certificates.** Each tree in `results/attack6_cert.json` is checked three ways: the
  Prop. 5.3 pattern combinatorially, the leaves against the prefix factor set, and the
  leaves against the engine (one closed sentence each). All `struct/prefix/engine` flags
  are `True` for all thirteen certificates (`dim5`, `dim6`, `dim6b`, `x_1 … x_10`).
* **DFAO.** The `x_r` automaton is checked against the arithmetic definition of `x_r` on
  `2^18` positions before it is handed to the engine.

**Two bugs the cross-checks caught.**

1. *In this attack's own code.* The first `dim_dp` let the two subtrees of a branch node
   pick *different* later level sets; it reported `D = 6` for paperfolding and `7` for
   Rudin–Shapiro, contradicting Theorem 3 (`R = 6, 12`). Both brute-force algorithms are
   kept for that reason.
2. *In the engine.* `Dfao::build_lsd` stored the transformation `g : Q → Q` of the
   transition monoid as a `Vec<u8>` and started from `(0..m as u8)`, so **any DFAO with
   `m >= 256` states was silently truncated mod 256** and then panicked with an
   out-of-bounds index (`x_7`, 460 states: "the len is 204 but the index is 204"). Fixed
   to `Vec<State>` in `engine/src/dfao.rs` (2026-08-17); `x_7`/`x_8` then load and their
   certificates verify. The path is only reachable through `dfao`/`dfao @file` with a
   large automaton — `def` takes one character per letter — which is why nothing had hit
   it before. Regression-checked: Thue–Morse and Rudin–Shapiro ladders unchanged.

---

## 5. Ledger

**Known before (cited, not claimed):**
* The game, `W(X)`, hereditariness, complexity preservation — Salo–Törmä (2014).
* `P_v`, coding dimension, weak `S`-codability, Theorems 5.5/5.6, Props. 6.1/6.5/6.9,
  Theorem 6.2, Cor. 6.3, and the `factorEq`/`isRS`/`extRS_d` encoding — Peltomäki–Salo.
* Coding dimensions 3, 2, 3, 4 for Thue–Morse, period-doubling, regular paperfolding,
  Rudin–Shapiro, and the `W(X)` descriptions of the first two — Peltomäki–Salo §7
  (the period-doubling one is corrected here).
* Cobham 1972 (`O(n)` factor complexity of automatic words); Cassaigne's theorem
  (`O(n)` ⟺ boundedly many right-special factors).

**New here:**
* **Finding 0** — Question 10.1 is negative for every Pisot ANS, base `k` included, as a
  corollary of the source's own Cor. 6.3; what remains of the question is the existence of
  an addable non-Pisot ANS. *Proved (assembly of published results).*
* **Proposition 1** — the one-positional-variable recursion for `P_{1^d}(W(X))`: `d+1`
  variables against the published encoding's `d + 2^{d-2}`. *Proved.*
* **Theorem 3** — `D(x) <= 1 + log2 sup_n s_x(n)`, both sides effective. *Proved.*
* **Theorem 4** — an explicit family of binary 2-automatic words with `D(x_r) >= r`: the
  coding dimension is unbounded over the class though finite for each word. *Proved*,
  and certified by machine for `r <= 10`.
* **Coding dimensions 5, 6 and 10** for concrete binary automatic words (published
  maximum: 4), from 4-, 8-, 7- and 4463-state DFAOs. *Machine-verified* (engine ladder for
  5, engine-checked tree certificates for 6 and 10, independent brute force throughout).
* **Erratum** — the published two-`1`s description of `W(X)` for the period-doubling word
  is necessary but not sufficient; the exact condition is `a − 1 < 2^{k−1}`, and the
  smallest counterexample is `(a,b) = (2,4)`. *Machine-verified both ways.*
* **Engine fix** — the `Vec<u8>` truncation in `Dfao::build_lsd` (§4), which made every
  DFAO of `>= 256` states unusable.
* The measurement that `1 + log2 R` is attained but usually loose by exactly 1, and that
  `D = 1+⌊log2 R⌋` is false 45 % of the time. *Machine-verified, not proved.*

**Not established / open:**
* Question 10.1 itself: it needs an addable non-Pisot ANS, which is outside `<N,+,V_k>`
  and outside this engine.
* Problem 10.2 for non-uniform substitutions (Dumont–Thomas numeration). For `k`-uniform
  substitutions Corollary 2 implements it.
* Exact `D` for `dim6`, `dim6b` and `x_r` (`r >= 4`): only `D >= d` is certified; the
  matching upper bound is brute force over levels `<= 30`, and was not computed at all for
  `x_9`, `x_10` (the prefix needed grows like `2^{2r}`).
* A lower bound on the DFAO size needed for coding dimension `d` (data suggests `2^{d/2}`;
  only `R >= 2^{d-1}` is proved, and `R <= k m²` is an observation, not a theorem).
* Closed forms for `W(X)` of the paperfolding and Rudin–Shapiro words: we produce the
  `L_2`, `L_3` automata and their exact tuple lists, but extracted no formula.

**Failed / discarded on the way:**
* *Conjecture `D = 1 + ⌊log2 R⌋`*, suggested by all four published values — refuted on the
  random sweep (§3.3).
* *Certifying `R` in the engine past small values* — the `q`-first-occurrence sentence
  needs `k^{q+1}` symbols; `q = 13` exceeds 4 GB on Thue–Morse and `q = 11` already does on
  Rudin–Shapiro. `R` in the tables is the prefix value except for Thue–Morse,
  period-doubling and paperfolding, where the engine confirms it exactly
  (`results/attack6_rmax.json`).
* *`x_4`, `x_5` through the full ladder* — `learnfe` does not converge in 30 min at the
  default LCP cap and gives a >3 000-state `FE` at a reduced cap. The ladder itself is
  cheap; this is a `learnfe` limit on words with long constant runs.
* *A base-3 `D = 6` sentence* — `E a_1..a_6,n. $B_6(…)` needs `3^{14} > MAX_ALPHA`; solved
  by the state-count emptiness criterion of §1 rather than by raising the cap.
* *The first `dim5` candidate* (`k=3, m=7`, `R=43`) — `FE` reached 1 242 states and `B_2`
  blew the 4 GB budget; replaced by the 4-letter base-2 word.
* *Lowering `AM_LEARN_LCP` globally* — no effect on the classical words (paperfolding
  gives the same 44-state `FE` in the same 0.8 s at `2^16` as at the default); it is kept
  only for the `x_r` family, where the default cap makes the LCP oracle saturate on the
  long zero runs (`x_1`: 13 s at the default, 0.6 s at `2^16`).

---

## 6. Reproducing

```
python3 explore/attack6_brute.py 30 16                # independent brute force
python3 explore/attack6_dim.py                        # engine ladder, exact D
python3 explore/attack6_cert.py                       # tree certificates, D >= d
python3 explore/attack6_charact.py                    # published descriptions + enum cross-check
python3 explore/attack6_gap.py 300 3                  # tightness of Theorem 3
python3 explore/attack6_search.py 11 600 2 5,6,7,8    # hill-climb for large D
python3 explore/attack6_rmax.py                       # R = sup_n s(n) in the engine
```

Outputs: `results/attack6_{brute,dim,cert,gap,search,rmax}.json`,
`results/attack6_charact.txt`, `results/attack6_dim.log`.
