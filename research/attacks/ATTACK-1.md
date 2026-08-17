# ATTACK-1 — Shifted Thue-Morse state complexity (Moradi–Rampersad–Shallit, Open Problem 2)

**Refereed:** see `paper/attack1-verdict.md` — verdict **PROVED** (Theorems 1–3 and the
corollary algorithm; F1–F4 remain conjectural, as stated below).

**Target.** *"What is a good formula for the exact number of states in the minimal automaton
generating `(t(i+c))_{i>=0}` with msd-first input, as a function of `c`?"*
— D. Moradi, N. Rampersad, J. Shallit, *Complexity of Linear Subsequences of k-Automatic
Sequences*, arXiv:2512.10017, §4.2 Open Problem 2. `t` = Thue-Morse (A010060).
Write `a(c)` for that state count; `a` is OEIS **A382296** (offset 0, `a(0) = 2`).

---

## 0. Status re-check (2026-08-17)

| item | finding |
|---|---|
| arXiv:2512.10017 | latest is **v5, 2 Apr 2026** ("fixed some typos"). Open Problem 2 still stated, unresolved. |
| A382296 | b-file **n = 0..12000** by Shallit; entry last edited 10 Aug 2026. |
| new since the paper | **R. Biswas, *State Complexity of Thue-Morse Shifts by Powers of Two* (Aug 2026)**, linked from A382296, proves the **`c = 2^r` subcase only**: `a(2^r) = 2 F_{r+3}`. The OEIS formula line was updated 2 Aug 2026 from conjecture to "proved. See Biswas link." |
| general `c` | **still open.** Biswas's abstract says so explicitly ("This manuscript determines the exact value for the power-of-two subfamily"). No formula, no recurrence, no extension of the table past 12000 anywhere found. |

So the target is live, and the power-of-two line is now closed by someone else. Everything
below is about **general `c`**.

---

## 1. What we prove

### Theorem 1 (offset-set characterisation of the minimal msd DFAO, all `c`)

Fix `c >= 0`. Let `N(c) ⊆ N` be the closure of `{c}` under the two maps
`n ↦ ⌊n/2⌋` and `n ↦ ⌊(n+1)/2⌋`.

**(i)** With `ℓ = ` bit-length of `c` and `v = v_2(c)` (for `c >= 1`),

```
N(c) = { ⌊c/2^L⌋ : 0 <= L <= ℓ }  ∪  { ⌊c/2^L⌋ + 1 : v < L <= ℓ },     max N(c) = c,
```

so `|N(c)| <= 2ℓ + 1 = O(log c)`.

**(ii)** The **minimal** msd-first base-2 DFAO for `i ↦ t(i+c)` (leading zeros allowed) is

```
states     Q_c = { V_c(p) : p >= 0 },     V_c(p) = ( t(p+n) )_{n ∈ N(c)} ∈ {0,1}^{N(c)}
initial    V_c(0) = ( t(n) )_{n ∈ N(c)}
transition (V · b)[n] = V[ (n+b) >> 1 ] XOR ( (n+b) mod 2 )        (b ∈ {0,1})
output     λ(V) = V[c]
```

and hence `a(c) = |{ V_c(p) : p >= 0 }|`.

*Proof.* (i) is a routine induction on the closure. For (ii): closure gives
`(n+b)>>1 ∈ N(c)`, and `t(2p+n+b) = t(p+m) XOR ((n+b) mod 2)` with `m = (n+b)>>1`
(from `t(2x)=t(x)`, `t(2x+1)=1-t(x)`), so the transition is well defined and
`δ*(V_c(0), w) = V_c(V(w))`; `c ∈ N(c)` gives the output, and `V_c(0)·0 = V_c(0)`
handles leading zeros.

Minimality. Put `q_L = ⌊c/2^L⌋`, `r_L = c mod 2^L`. For a suffix `s < 2^L` write
`s + r_L = ε·2^L + s'` with `ε ∈ {0,1}`; then

```
t( p·2^L + s + c ) = t( p + q_L + ε ) XOR t( s' ).                                  (*)
```

So the residual of the msd prefix of value `p` depends on `p` only through the values
`t(p+q_L)` (always) and `t(p+q_L+1)` (only when `r_L > 0`, i.e. `L > v`) — exactly the
coordinates listed in (i). Hence equal `V_c` ⇒ equivalent prefixes. Conversely, if
`V_c(p) ≠ V_c(p')` they differ at some `n ∈ N(c)`; by (i) that `n` is `q_L` (take `ε = 0`,
realised by `s = 0`) or `q_L + 1` with `L > v` (take `ε = 1`, realised because `r_L > 0`),
and `(*)` turns it into a distinguishing suffix of length `L`. For `L > ℓ` one has
`q_L = 0` and `r_L = c > 0`, and `0, 1 ∈ N(c)`, so nothing is missing. ∎

This is the exact generalisation of Biswas's automaton: for `c = 2^r`, `r_L = 0` for all
`L <= r`, so `N(2^r) = {0, 1, 2, 4, …, 2^r}` and `V` is his `⟨u; ⟨d_0,…,d_r⟩⟩` after
XOR-ing out `t(p)`.

### Theorem 2 (`a(c)` is always even)

Complementing every coordinate commutes with both transitions (the gather is `F_2`-linear
and the flip mask is constant). Since `max N(c) = c`, choosing `2^K > c` gives
`t(2^K + n) = 1 XOR t(n)` for all `n ∈ N(c)`, i.e. `V_c(2^K) = ¬V_c(0)`. So the reachable
set is closed under `¬`, which is a fixed-point-free involution: `a(c)` is even. ∎
(Equivalently `Q_c ≅ {0,1} × D_c`, matching Biswas's `Q_r = {0,1} × E_{r+1}`.)

### Theorem 3 (prefix monotonicity)

`N(⌊c/2⌋) ⊆ N(c)` (the closure of `{c}` contains `⌊c/2⌋` and is closed), so
`V_c(p) ↦ V_{⌊c/2⌋}(p)` is a well-defined **surjection** `Q_c ↠ Q_{⌊c/2⌋}`, whence

```
a(c) >= a(⌊c/2⌋)      for every c >= 1.
```

∎  No matching upper bound is proved: for odd `c`, `N(c) \ N(⌊c/2⌋)` can have `Θ(log c)`
elements (e.g. `c = 2^k+1` adds `{2^j+1 : 1 <= j <= k}`), so the naive
"two extra coordinates ⇒ factor 4" argument fails. Empirically
`a(c)/a(⌊c/2⌋) <= 8/3` for all `c < 2^18`, the maximum being attained at `c = 5`
(**observation, not a theorem**).

### Remarks added following the referee's review (`paper/attack1-verdict.md`)

**(a) Free corollary.** Theorem 3 iterated with Biswas's `a(2^r) = 2F_{r+3}` gives an
**unconditional** general-`c` lower bound: writing `c = 2^m + r` and `s` = bit-length of
`r`, `⌊c/2^s⌋ = 2^{m-s}`, so `a(2^m+r) >= a(2^{m-s}) = 2F_{m-s+3}` for `m >= s`. It is
weak (median slack factor `2.2·10^4`; it only bites when `r ≪ c`) but it is the only
*proved* lower bound for general `c` anywhere in this note.

**(b) Chain projection — a smaller target than F1.** Let
`P(c) = #{ (t(p + ⌊c/2^L⌋))_{L=0..ℓ} : p >= 0 }`, the projection of `Q_c` onto the
descending chain `c, ⌊c/2⌋, …, 1, 0` alone (chain values are a subset of `N(c)`, so
`P(c) <= a(c)`). Empirically `P(c) >= 2F_{m+3}` for `2 <= c < 4096`, with equality exactly
at `c = 2^m` (checked `2 <= c < 2048`), and `min P(c)/a(c) = 0.569`. So F1 reduces to a
Fibonacci-minimality statement about the single chain `c, ⌊c/2⌋, …, 1, 0` — literally
Biswas's setting with the chain `2^r,…,1,0` replaced by an arbitrary one — a materially
smaller target than F1 as stated.

**(c) MRS Theorem 19 is false as stated.** It asserts `a(c) <= (10/3)c` for all `c >= 1`,
but `a(1) = 4 > 10/3`, and `a(1) = 4` is in Shallit's own b-file; the proof's case split
`c = 2^p+r'`, `0 < r' <= 2^p` or `2^{p-1} < r' <= 2^p`, never covers `r' = 0`. `c = 1` is
the unique failure in `c < 262144`. The bound is otherwise correct and exactly tight at
`c = 3` (`a(3) = 10 = (10/3)·3`); `max_{c >= 100} a(c)/c = 2.3867` at `c = 181`.

### Corollary (algorithm)

Theorem 1 gives an `O(a(c)·log c)` **exact** algorithm using `O(log c)`-bit state words —
no subset construction, no minimisation. `explore/attack1_states.c` reproduces all
**12 001** OEIS b-file terms in **2.1 s** on one core.

---

## 2. New data

`results/attack1_a.txt.gz` — `a(c)` for **`c = 0 … 262 143`**, a **21×** extension of the
published b-file (which stops at 12 000). Values agree with the b-file on the whole
overlap.

Dyadic-block statistics (`explore/attack1_analyse.py`):

```
  m       min  argmin   2F_{m+3}  min-law       max  argmax(base 2)     max/prev  mean a/c
  1         6     [2]          6  True           10  11                   —       3.1667
  2        10     [4]         10  True           20  111               2.0000    2.8893
  4        26    [16]         26  True           70  11001             1.8421    2.4386
  8       178   [256]        178  True          854  110110001         1.8728    1.7650
 12      1220  [4096]       1220  True        11202  1110111000101     1.8993    1.3400
 16      8362 [65536]       8362  True       149928  11101111101000011 1.9175    1.0705
 17     13530 [131072]     13530  True       287738  111011111011100011 1.9192   1.0187
```

(full table in the script's output; the law holds for every `m = 1 … 17`).

---

## 3. New results (empirical, verified over the whole sweep)

### F1 — sharp global lower bound (conjecture)

> For every `m >= 1`, `min { a(c) : 2^m <= c < 2^{m+1} } = a(2^m) = 2 F_{m+3}`,
> and the minimiser is **unique**.

Verified for `m = 1 … 17` (all 262 142 values). Consequences:

```
a(c) >= 2 F_{⌊log2 c⌋ + 3}                for all c >= 1,  equality iff c is a power of 2
a(c) >  2.342 · c^{log2 φ}  =  2.342 · c^{0.694242…}
```

This upgrades the paper's `~c^0.694` bound — stated there **only for `c = 2^r`**
(their Thm 20 / Biswas) — to a bound valid for **every** `c`, with a characterisation of
the equality case. `2.342 = 2φ²/√5` is what the block-boundary inequality above yields at
the *top* of a block, but it is **not** the optimal constant for `a` itself, and the data
undersells it: `min_{1 <= c < 262144} a(c)/c^{0.694242} = 3.7082` (attained at `c = 2`),
and along the minimisers `a(2^m)/2^{0.694242 m} → 2φ³/√5 = 3.7889`. So the honest bound
supported by the whole range is

```
a(c) >= 3.708 · c^{0.694242}                for all 1 <= c < 262144,
```

58% stronger than the `2.342` headline; `2.342` is optimal only as a *consequence* of the
block-boundary inequality above, not for `a(c)/c^{0.694242}` itself.

### F2 — family laws `a(2^r + d) = 2 F_{r+4} + κ(d)`

The difference `a(2^r + d) − 2F_{r+4}` becomes **constant in `r`** exactly for
`d ∈ {−1} ∪ { 2^k, 2^k + 1 : k >= 0 }` (checked for all `−6 <= d <= 16` and
`d ∈ {17,20,24,31,32,33,63,64,65}`, up to `r = 24`). For all other `d` it grows.

| `d` | −1 | 1 | 2 | 3 | 4 | 5 | 8 | 9 | 16 | 17 | 32 | 33 | 64 | 65 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `κ(d)` | −6 | 0 | 2 | 8 | 6 | 20 | 16 | 42 | 36 | 82 | 76 | 154 | 152 | 282 |
| `r0(d)` | 1 | 1 | 2 | 3 | 3 | 4 | 4 | 5 | 8 | 8 | 10 | 10 | 12 | 12 |

i.e. `a(2^r + d) = 2F_{r+4} + κ(d)` for all `r >= r0(d)`. Special cases worth naming:

```
a(2^r + 1) = 2 F_{r+4} = a(2^{r+1})          (r >= 1)   — a shift by 2^r+1 costs exactly
                                                          as much as one by 2^{r+1}
a(2^r − 1) = 2 F_{r+4} − 6                   (r >= 1)
a(2^r + 2) = 2 F_{r+4} + 2                   (r >= 2)
a(2^r + 4) = 2 F_{r+4} + 6                   (r >= 3)
a(2^r + 2^k)     = 2 F_{r+4} + κ(2^k)        (r >= r0)
a(2^r + 2^k + 1) = 2 F_{r+4} + κ(2^k+1)      (r >= r0)
```

Contrast with the (proved) `a(2^r) = 2F_{r+3}` — `d = 0` is the *only* small `d` whose
family sits one Fibonacci step lower, which is exactly why the powers of two are the
block minima (F1).

Sequences `r ↦ a(A·2^r + B)` are **eventually** C-finite for other `(A,B)` too, at higher
order: `a(2^r+3)`, `a(2^r+2)`, `a(2^r-1)`, `a(2^r-2)`, `a(2^r+4)` all satisfy
`x_n = 2x_{n-1} − x_{n-3}` (char. poly `(x−1)(x²−x−1)`) for `r` past the transient — `d = 3`
and `d = 4` fail the recurrence at `r = 5` because `r = 2` is pre-asymptotic; `a(3·2^r)` and
`a(5·2^r)` need order 7. We did **not** find a uniform order bound.

### F3 — there is no "good formula" of transfer-matrix type

The 2-kernel of `a` is **not** finite-dimensional as far as we can compute:

```
kernel rows { n ↦ a(2^e·n + r) : e <= 6, r < 2^e } :  127 rows × 2000 cols, rank = 127
kernel rows { n ↦ a(2^e·n + r) : e <= 8, r < 2^e } :  511 rows × 1000 cols, rank = 511
```

(exact over `Q` at `e <= 6`; over `F_p`, `p = 2^31−1`, at `e <= 8` — a mod-`p` rank is a
lower bound for the rational one). Every kernel subsequence is linearly independent of all
the others, with no sign of saturation; the rank appears to be exactly `2^{e+1}−1`.

**Consequence.** `a` is (almost certainly) **not 2-regular**, so there is **no**
representation `a(c) = u · M_{b_1} ⋯ M_{b_L} · v` over the binary digits of `c` in either
digit order, for any fixed finite-dimensional `M_0, M_1`. That rules out the shape a
"good formula" would most naturally take in this literature — and it explains why the
authors found only a bound. Structurally: for `c = 2^r` the achievable difference vectors
are cut out by the *local* constraint "no two adjacent zeros" (Fibonacci); for general `c`
the constraints are **not** local along the prefix chain, e.g. `t(0) = t(3) = t(6) = 0`
already breaks the adjacent-pair lemma at `q = 3`.

### F4 — growth of the maximum

`max { a(c) : c < 2^m }` grows by a factor converging to `≈ 1.919` per doubling
(ratios `1.900, 1.913, 1.920, 1.918, 1.919` for `m = 13…17`), i.e.

```
max_{c < 2^m} a(c)  ≈  C · c^{0.9405…}
```

so `a(c) = O(c^{0.941})` in the observed range — well below the trivial linear ceiling, and
notably *not* an integer/simple exponent.

**Retraction.** An earlier draft claimed the block argmaxima converge digit-by-digit to a
fixed infinite binary word. That is false on the note's own data (`paper/attack1-verdict.md`
§4): the longest common prefix of consecutive argmaxima is **not monotone**,

```
m           :  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17
lcp(m-1, m) :  2  2  2  4  6  2  7  2  9 11 11  7 10 15 14 11
```

collapsing from 11 (`m = 12`) to 7 (`m = 13`), and again from 15 (`m = 15`) to 14 to 11.
There is no digit-by-digit convergence in the range computed; at best the argmax words
share a `111011111`-ish head. We report the raw argmax table instead
(`argmax` for `m = 9…17`: `1110111001`, `11101110001`, `111011100011`, `1110111000101`,
`11101111100101`, `111011111010001`, `1110111110100011`, `11101111101000011`,
`111011111011100011`) and make no convergence claim.

Average behaviour: `mean{ a(c)/c : 2^m <= c < 2^{m+1} }` **decreases** steadily —
`1.53, 1.43, 1.34, 1.26, 1.19, 1.13, 1.07, 1.02` for `m = 10…17` — consistent with
`mean a(c) = o(c)`, roughly `c/(log c)^{0.76}` on this range. Not enough data to call it.

---

## 4. Verification

Four independent implementations, pairwise cross-checked:

1. **`explore/attack1_shift.py`** — builds the explicit `O(log c)`-state **lsd** DFAO for
   `i ↦ t(i+c)` (ripple-carry adder × parity accumulator, padding-invariant output), then
   the msd minimal DFAO by the reverse-vector (transition-monoid) construction. Pure
   Python, no use of Theorem 1.
2. **`explore/attack1_states.c`** — the Theorem 1 algorithm (offset-closure + 128-bit
   pattern BFS). Different mathematics, different language.
3. **The Peanut engine** — `mode msd; def T 2 2 0 01 10 01; dfa T[i+c]=1`, i.e. the
   engine's own subset construction and Hopcroft minimisation, no shared code at all.
4. **OEIS b-file** `b382296.txt` (Shallit), `n = 0..12000`.

| check | range | result |
|---|---|---|
| (2) vs (4) | `c = 0 … 12000` | **all 12 001 agree** |
| (1) vs (2) | `c = 0 … 2999` | all agree |
| (3) vs (2) | `c = 0 … 400` | all agree (`results/attack1_engine_check.json`) |
| (3) vs (1) | `c = 0 … 40`, lsd too | all agree; lsd counts reproduce **A382298** exactly |
| (3) vs F2 | `c = 2^r + d`, `d ∈ {0,1,2,3,4,5,8,9,16,17,32,33}`, `r0(d) <= r <= 16` | all agree (`results/attack1_engine.log`) |

The engine transcript is the machine proof of record for the family laws in the range it
can reach.

### Machine proofs of the lemmas (`explore/attack1_lemmas.py`, `results/attack1_transcript.txt`)

Theorem 1's proof rests on facts that *are* expressible as closed sentences over
`<N,+,V_2>` (a constant times a variable is a legal term, which is what makes the
`2^L·p` in `(*)` reachable). The engine decided **48 sentences, all TRUE, 0 FALSE, no
errors**, in 9.3 s / 311 MB peak:

| block | sentences | result |
|---|---|---|
| `A n. T[2*n]=T[n]`, `A n. T[2*n+1]!=T[n]` | 2 | TRUE — the morphism |
| `A x,s. s < 2^L => (T[2^L*x+s]=T[x] <=> T[s]=0)`, `L = 1…11` | 11 | TRUE — digit concatenation, the engine of `(*)` |
| `(*)` itself, both carry branches, for `(c,L) ∈ {(5,1),(5,2),(11,3),(37,4),(300,5),(1000,7),(2000,8)}` | 14 | TRUE |
| Biswas's adjacent-coordinate lemma `A p. T[p]!=T[p+2^j] \| T[p]!=T[p+2^{j+1}]`, `j = 0…12` | 13 | TRUE |
| locality **fails** off the powers of two: `E p. T[p]=T[p+q] & T[p]=T[p+2q]` for `q ∈ {3,5,6,7,9,11,12,13}` | 8 | TRUE (witnesses enumerated, e.g. `q=3`: `p = 0,1,3,6,8,9,12,…`) |

The last block is the machine-checked form of the obstruction in F3: for `c = 2^r` the
prefix chain is `1,2,4,…,2^r` and the adjacent-pair lemma holds at every step, giving
Biswas's Fibonacci count; for a general `c` the chain contains steps like `3`, where the
lemma is false, so the constraint set is not local and no transfer matrix over the digits
of `c` can exist.


---

## 5. Honest ledger

**Known before us**
- `a(c)` is the OEIS sequence A382296; b-file to `c = 12000`; the problem is Open Problem 2
  of arXiv:2512.10017 v5.
- `a(2^r) = 2 F_{r+3}` — conjectured by Shallit, **proved by Biswas (Aug 2026)**, which we
  merely re-derive and re-verify. We did **not** find this.
- The `~c^0.694` lower bound (Moradi–Rampersad–Shallit Thm 20), stated for `c = 2^r`.
- A382298 (the lsd count) and its unproven empirical recursion — untouched here beyond
  reproducing 41 terms with the engine.
- **MRS Theorem 9(b)** and **Theorem 19** — the prior art Theorem 1 supersedes. Thm 9(b)
  gives an automaton for `t(i+c)` with `ρ_{t'}(c+1) = Θ(c)` states (a window of `c+1`
  interior values), explicitly *not* minimal; Thm 19 turns it into `a(c) <= (10/3)c` for
  all `c`. Theorem 1 replaces the `Θ(c)`-size window by the `<= 2ℓ+1 = O(log c)` offsets of
  `N(c)` **and** proves minimality — a genuine `Θ(c) -> Θ(log c)` gain, plus exactness. See
  remark (c) above: Theorem 19 as published is false at `c = 1`.

**New here (proved)**
- **Theorem 1**: the offset-set characterisation of the minimal msd DFAO for *every* `c`,
  with the explicit transition rule and `|N(c)| = O(log c)`. Generalises Biswas's
  power-of-two automaton; his `E_{r+1}` "no adjacent zeros" state set is the `c = 2^r` case.
- **Theorem 2**: `a(c)` is even for every `c` (with the `V_c(2^K) = ¬V_c(0)` argument).
- **Theorem 3**: `a(c) >= a(⌊c/2⌋)` (prefix monotonicity).
- The resulting `O(a(c)·log c)` exact algorithm.

**New here (data / conjecture, not proved)**
- The table extension to `c < 262144` (21× the published range).
- **F1**: block minima are exactly the powers of two ⇒ the `c^{0.694}` bound holds for all
  `c` with the sharp constant. *Verified `m <= 17`, not proved.*
- **F2**: the `κ(d)` family laws and the characterisation of the `d` for which they hold.
  *Verified to `r = 24` (and to `r = 16` independently by the engine), not proved.*
- **F3**: 2-kernel rank `= 2^{e+1}−1` up to `e = 8` ⇒ no transfer-matrix / 2-regular closed
  form. *Computational evidence, not a proof of non-regularity.*
- **F4**: `max_{c<2^m} a(c) ~ C·1.919^m`; the limiting maximiser word; the decaying mean.
  *Observation only — we cannot identify `1.919`.*
- The ratio bound `a(c)/a(⌊c/2⌋) <= 8/3`. *Observed on `c < 2^18`; only `>= 1` is proved.*

**Failed / not attempted**
- **No closed form for general `c`.** F3 says the natural shape does not exist; we have no
  substitute. Open Problem 2 is **not** solved.
- No proof of F1, F2 or F3. F1 and F2 look provable by Biswas's method (explicit automaton
  + reachability + distinguishability), one family at a time; we did not carry it out.
  F2 in particular is a finite check per `d` **if** one first proves an induction step in
  `r`, which we did not set up.
- We could not express the problem usefully inside the engine's own logic: the state count
  is a *meta*-property (a Myhill-Nerode class count), and the residual relation
  `p ≡_c p'` needs the term `p·2^L` with `L` a variable, which is outside the engine's
  linear-term grammar. The engine therefore serves only as an independent oracle for
  individual `c`, not as a prover of the parametrised statements. `learnfe` was not
  applicable (no factor-equality predicate is involved).
- We did not touch the exponential lower bound / non-regularity as a theorem; proving
  `a` non-2-regular is itself an open sub-problem raised by F3.

---

## 6. Reproduce

```sh
cc -O3 -o /tmp/a1states explore/attack1_states.c
/tmp/a1states 0 262144 > /tmp/a.txt        # ~15 min on 1 core, seconds per 10^4
python3 explore/attack1_analyse.py         # tables of §2, §3 from results/attack1_a.txt.gz
python3 explore/attack1_engine_check.py 40 # engine vs reference, msd + lsd
python3 explore/attack1_families.py 16 400 # engine verification of the kappa laws
python3 explore/attack1_regular.py         # 2-kernel rank (exact, small)
python3 explore/attack1_lemmas.py 11 12    # machine proofs of the Theorem-1 lemmas
```

Data: `results/attack1_a.txt.gz`, `results/attack1_engine.log`,
`results/attack1_families.json`, `results/attack1_engine_check.json`,
`results/attack1_transcript.txt`.
