# ATTACK 3 — the General Clergyman's Conjecture, and the rusty numbers

**Refereed:** see `paper/attack3-verdict.md` — verdict **MACHINE-VERIFIED** (after the
`LAST`-predicate repair described in §3/§4 below; all 249 records re-verify).

**Target.** R. Fokkink, G. Joshi, *Anti-recurrence sequences*, arXiv:2506.13337v1
(16 Jun 2025), **Conjecture 4** — proved there (Theorem 12) only for *A₁-bounded*
linear forms, and singled out as a challenge for the **rusty numbers** `a = (1,d)`.
Builds on W. Bosma et al., arXiv:2503.04122, Conjecture 2 ("the Clergyman's
Conjecture"). Published as R. Fokkink, G. Joshi, *Anti-recurrence sequences*,
**Integers 26 (2026) #A24**, which renumbers everything cited here: Conjecture 4 →
**Conjecture 2**, Definition 9 → **Definition 1**, Theorem 12 → **Theorem 4**, Lemma 11
→ **Lemma 5**, Theorem 8 (anti-bonacci) → **Theorem 3**, §4 (rusty numbers) → **§5**.
The arXiv v1 numbering is used throughout this document; the published numbering is
noted here for cross-reference.

**Status re-verified 2026-08-17** against the primary source (arXiv abstract + full
PDF **and** the published PDF, `https://math.colgate.edu/~integers/aa24/aa24.pdf`):
only v1 exists on arXiv, no later arXiv version, but the paper **is published** as
*Integers* 26 (2026) #A24 — see the renumbering table above. No follow-up settling the
general case was found in either version. §4/§5 of the paper and its "Final remark"
both state the general case as open, in these words: *"Does the conjecture hold
without the restriction of `A₁`-boundedness? Are the rusty numbers sums of linear
sequences and automatic sequences?"* The target is live.

---

## 1. The statement, verbatim

> **Conjecture 4.** Let `a = (a₁,…,a_k)` be positive and integral of dimension
> `k > 1`. Let `Aₙ` be the anti-recurrence sequence for the linear form `f(x)=a·x`.
> Then `Aₙ − κn` is `τ`-automatic for `κ = kτ+1` and `τ` the trace of the linear form.

> **Definition 9.** A positive linear form `a` of dimension `k` and trace `τ` is
> `A₁`-bounded if `A₁ ≤ (k−1)τ + 2`, where `A₁` is the first anti-recurrence number.

> **Theorem 12.** If `a` is `A₁`-bounded, then it generates an anti-recurrence
> sequence `Aₙ` such that `Aₙ − κn` is `τ`-automatic. … It can be recognized by a
> DFAO with at most `2τ−1` states.

> **§4.** "The subsequence `A_{5n+1}` is equal to the arithmetic progression
> `A₁ + 55n` up until `n = 348`, when `A₁₇₄₁ ≠ 9 + 55·348`. … Surprisingly, it is a
> challenge to prove, or disprove, the conjecture for the rusty numbers."

`A₁ = Σⱼ j·aⱼ`, so for `a = (1,d)` we have `k = 2`, `τ = d+1`, `κ = 2d+3`,
`A₁ = 2d+1`, and `A₁`-boundedness `2d+1 ≤ d+3` holds **iff `d ≤ 2`**. Hence
`(1,1)` (anti-Fibonacci, Zaslavsky) and `(1,2)`/`(2,1)` (anti-Pell/anti-Jacobsthal,
the paper's Theorems 5 and 6) are covered, and **`d ≥ 3` — the rusty numbers — is
exactly the part of Conjecture 4 that no published proof reaches.**

## 2. Definitions used here

`Aₙ`, `Bₙ` are complementary strictly increasing sequences of positive integers
(every natural is in exactly one), with

        Aₙ = Σ_{j=1..k} aⱼ · B_{(n−1)k+j}.

Fokkink–Joshi Lemma 3: they exist, are unique, and are generated blockwise by the
mex rule — the `n`-th **B-block** is the `k` smallest naturals not yet in `A ∪ B`,
and `Aₙ` is the `a`-weighted sum of that block. The proof of Lemma 3 (via Lemma 2)
shows `[b, b+k−1]` (`b` the mex) contains **at most one** already-decided
anti-recurrence number, so a B-block is a run of `k` consecutive naturals with
**at most one value skipped** — the encoding of §3 assumes exactly this, and any
form for which it failed would simply make the obligations FALSE.

## 3. Encoding in the engine

Automatic sequences are 0-indexed, so we work with `A(m) := A_{m+1}`, `m ≥ 0`.
Let `T` be a base-`τ` DFAO (via `def`, as a `τ`-uniform morphism + coding) and

        AA(m,s)  :≡  s = κ·m + lo + T[m]                          "A_{m+1} = s"

(`lo` a constant; `T` has output alphabet `{0,…,hi−lo}`). This is written as a
finite disjunction over output letters, since `T[t]` is not an arithmetic term.

**Block encoding.** Write the B-block of `A(m)` as

        x_j = x + (j−1) + [j > g],     j = 1..k,   g ∈ {1..k}

(`g = k` ⇔ no skip; otherwise the value `x+g` is skipped). Then

        A(m) = τ·x + c_g,     c_g = (A₁ − τ) + Σ_{j>g} aⱼ.

Because `0 ≤ Σ_{j>g} aⱼ ≤ τ − a₁ < τ` and that sum is strictly decreasing in `g`,
the `c_g` are **pairwise distinct mod τ**, so `(x,g)` is recovered from `A(m)`
uniquely and no existential over `g` is needed:

        G_g(m,x) :≡ AA(m, τ·x + c_g)                      (one `let` per g)
        B1(m,x)  :≡ ⋁_g G_g(m,x)                          block start
        LAST(m,y):≡ ⋁_{g<k} (∃x. G_g(m,x) ∧ y = x+k) ∨ (∃x. G_k(m,x) ∧ y = x+k−1)   block end
        INBLK(m,t):≡ ⋁_{g,j} ∃u. G_g(m,u) ∧ t = u + (j−1) + [j>g]
        USED(j,t) :≡ AA(j,t) ∨ INBLK(j,t)
        ISA(t) :≡ ∃n. AA(n,t)        ISB(t) :≡ ∃m. INBLK(m,t)

`LAST` must be a genuine forward shift of the block start `x` (`y = x+k` or `y = x+k−1`),
**not** its inverse `x = y+k` — an earlier draft of both this document and
`explore/attack3_general.py` wrote `G_g(m, y+k)`, which substitutes `x := y+k` into `G_g`
and so asserts `y = x−k`, the wrong direction. That bug made two of the eleven proof
obligations below (P5, P6) vacuously true and silently dropped a step of the §4
induction; see `paper/attack3-verdict.md` §2.2 for the derivation and the fix, applied
here and in the script.

## 4. The proof obligations

All are **closed** sentences over `⟨ℕ,+,V_τ⟩` plus `T`; all must be `TRUE`.

| id | sentence |
|---|---|
| P1a | `∀m,s,t. AA(m,s) ∧ AA(m,t) ⇒ s=t` |
| P1b | `∀m ∃s. AA(m,s)` |
| P2  | `∀m ∃x. B1(m,x)` |
| P2u | `∀m,x,u. B1(m,x) ∧ B1(m,u) ⇒ x=u` |
| P4  | `G_k(0,1) ∧ AA(0, A₁)`   (initial block `{1,…,k}`) |
| P5  | `∀m,y,u. LAST(m,y) ∧ B1(m+1,u) ⇒ y < u` |
| P6  | `∀m,y,s. LAST(m,y) ∧ AA(m,s) ⇒ y < s` |
| P7  | `∀m,s,t. AA(m,s) ∧ AA(m+1,t) ⇒ s < t` |
| P8  | `∀t. ¬(ISA(t) ∧ ISB(t))` |
| P9  | `∀m,x,t. B1(m,x) ∧ 1 ≤ t < x ⇒ ∃j<m. USED(j,t)` |
| P10_g | `∀m,x. G_g(m,x) ⇒ ∃j<m. AA(j, x+g)`   (one per `g = 1..k−1`) |

**Why this is a proof.** Induction on `m`, using the **corrected** `LAST` above (with the
old, inverted `LAST`, P5 is vacuously true for every DFAO and this step of the argument is
unsupported — see the note after the definitions). Write `S_m = {A(j)} ∪ blk(j)` over
`j<m`. *Base:* P4 gives block `{1,…,k}` and `A(0) = A₁ = Σⱼ j·aⱼ`, the true first step.
*Step:* assume agreement below `m`; write `x = B1(m)` and let `g` be its (unique, by
P1a + P2u + the `c_g` being distinct mod `τ`) gap index.
 (i) `mex(S_m) ≥ x`: P9 says every `t ∈ [1, x)` lies in `S_m`.
 (ii) `x ∉ S_m`: **P5, with `LAST` reading `y = x+k` (resp. `x+k−1`), puts `x` strictly
 above `LAST(m−1)`, i.e. above every earlier B-value**, and P8 says no B-value is ever an
 A-value. Hence `mex(S_m) = x = B1*(m)`.
 (iii) Every other element of the encoded block is likewise `∉ S_m`, by the same two
 facts (they are B-values, and they exceed `LAST(m−1)`).
 (iv) The only value of `[x, x+k]` missing from the encoded block is the skipped
 `x+g` (when `g<k`), and P10_g says `x+g = A(j)` for some `j<m`, so `x+g ∈ S_m`.
By (i)–(iv) the encoded block is exactly the `k` smallest naturals outside `S_m` —
the true B-block. By construction `A(m) = τx + c_g = Σⱼ aⱼ x_j`, and P1a/P1b/P2/P2u
make `AA` and `B1` single-valued and total, so the encoded object is a genuine pair
of sequences. P6/P7 supply the monotonicity used implicitly above (`A(m)` lies above its
own block, and `A` increases), and together with P5/P8/P9 pin the encoded pair to the mex
generator term by term directly — the appeal to the uniqueness half of Lemma 3 is correct
but, given P5–P9, redundant. ∎

**Which obligations actually carry weight.** P1a, P1b, P2u are tautologies of the encoding
(`AA` is a function by construction; `x` is unique because the `c_g` are pairwise distinct
mod `τ`), and P6, P7 are free (`A*(m+1) − A*(m) ≥ (k−2)τ+3 > 0` unconditionally). The
effective obligation set — the one doing real work, confirmed empirically by perturbing
verified DFAOs one transition/output at a time — is `{P2, P4, P5, P8, P9, P10_g}`, with P9
catching the overwhelming majority of broken candidates and the corrected P5 the one this
document's earlier draft accidentally disabled. See `paper/attack3-verdict.md` §2.4.

Then `Aₙ − κn = (lo − κ) + T[n−1]` is a constant plus a shift of a `τ`-automatic
sequence, hence `τ`-automatic — Conjecture 4 for that `a`.

**Re-verification after the fix.** With the corrected `LAST`, all **249** records of §6
were rebuilt and re-run independently (`paper/verdict-attack3/rerun_all.py`, output
`paper/verdict-attack3/rerun_fixed.jsonl`): **249/249 all obligations TRUE**, 0 failures,
total engine time 541.2 s, worst case 14.9 s (`a=(1,12)`, 36 states). The gap is closed
and every conclusion below is unchanged.

By the Bruyère–Hansel–Michaux–Villemaire theorem the engine's compilation of each
sentence to a 1-state automaton *is* the decision procedure, so a `TRUE` line is a
proof, exactly as in `docs/THEOREMS.md`.

## 5. Getting the DFAO: guess, then refute-or-verify

`explore/attack3_gen.py` generates `Aₙ, Bₙ` directly from the mex rule.
`kernel_guess` builds the Myhill–Nerode quotient of ℕ under
`v ∼ w ⇔ ∀j,i<τ^j: C[vτ^j+i] = C[wτ^j+i]`, decided by a lockstep BFS over the two
subtrees restricted to the known prefix (so the test can only *over*-merge, never
split spuriously), then Hopcroft-minimises. `attack3_general.guess` runs the
adaptive ladder: guess on a prefix, replay the guess against the whole computed
prefix, and on the first mismatch at index `f` re-guess from a prefix `> 4f`. This
is `TARGET1.md`'s forward-cap ladder applied to the state budget.

The guess is *never* trusted: it is only a candidate handed to §4. Several under-sized
guesses fail — `a=(1,6)` with 15 states (needs 16), `a=(2,6)` with 16 (needs 17), and most
sharply `a=(2,9)` with 26 states, which reproduces the mex definition for **all 1.2·10⁷
terms computed** and still fails P8/P9, and `a=(1,12)` with 35 states, correct on
**5.128·10⁷ terms** and still wrong. That is the negative control for the method: prefix
agreement, at any length one can afford, is not evidence. **Correction:** an earlier draft
said these guesses "passed a long prefix check" before being "caught by the engine". That
is only true of `a=(2,9)` and `a=(1,12)`: the `(1,6)`/15-state guess actually breaks at
index **335 132**, which is *inside* this document's own `Ncheck = 400 000` prefix-ladder
check, so the adaptive guesser rejects it before the engine ever sees it — a valid
demonstration that prefix agreement at `3·10⁵` is worthless, but not an example of the
engine catching what a long prefix check missed. `(2,9)` and `(1,12)` are the genuine
examples of that (below).

**Closing the loop with the engine.** When an obligation is FALSE the engine can be
asked for the counterexample rather than guessing a longer prefix blind
(`attack3_general.counterexample`): run `witness` on the *open* negation, e.g.

        witness $B1(m,x) & t>=1 & t<x & ~(E j. j<m & $USED(j,t))

For the bad `a=(2,9)` automaton this returned `m = 13535104, t = 28300672,
x = 28300673` in 224 ms — i.e. it named the index just past the computed prefix at
which the guess breaks. Re-guessing from a 1.6·10⁷-term prefix gives 27 states, and
all obligations then pass. Same for `a=(1,12)`: witness `m = 51280580`, re-guess at
7·10⁷, 36 states, all TRUE. This is the `learnfe` equivalence-query loop
(`docs/LEARNFE.md` §2) with the mex recurrence in place of the LCP oracle: the
engine is the equivalence oracle, the mex generator is the membership oracle — and
it is what makes prefix length a *derived* quantity rather than a guess.

## 6. Results

Every number below is machine-verified: the DFAO reproduces the mex-generated
sequence term-by-term over the stated prefix **and** all obligations of §4 are
`TRUE`. Raw data `results/attack3.jsonl`; transcript for the rusty numbers
`results/attack3-rusty-transcript.txt`; table regenerated by
`python3 explore/attack3_table.py`.

```
forms attempted: 249   verified: 249   not A1-bounded (open before this run): 61

## Rusty numbers  a = (1,d)

      d  tau  kappa  A_1  A1-bnd  states  alphabet  window [lo,hi]  2tau-1
      1    2      5    3  True        2         2   [3,4]           3
      2    3      7    5  True        3         3   [4,6]           5
      3    4      9    7  False       6         6   [4,9]           7
      4    5     11    9  False       9         8   [5,12]          9
      5    6     13   11  False      11         9   [6,14]         11
      6    7     15   13  False      16        12   [6,17]         13
      7    8     17   15  False      19        14   [7,20]         15
      8    9     19   17  False      21        15   [8,22]         17
      9   10     21   19  False      26        18   [8,25]         19
     10   11     23   21  False      29        20   [9,28]         21
     11   12     25   23  False      31        21   [10,30]        23
     12   13     27   25  False      36        24   [10,33]        25

## By dimension and trace  (states: min-max over forms of that (k,tau))

      k  tau  forms  A1-bdd  open  states min-max  max alphabet  2tau-1
      2    2      1       1     0     2-2              2       3
      2    3      2       2     0     3-4              4       5
      2    4      3       2     1     5-6              6       7
      2    5      4       2     2     7-9              8       9
      2    6      5       2     3     9-11            10      11
      2    7      6       2     4    11-16            12      13
      2    8      7       2     5    13-19            14      15
      2    9      8       2     6    15-21            16      17
      2   10      9       2     7    17-26            18      19
      2   11      8       0     8    20-29            20      21
      2   12      1       0     1    31-31            21      23
      2   13      1       0     1    36-36            24      25
      3    3      1       1     0     3-3              3       5
      3    4      3       3     0     4-5              5       7
      3    5      6       6     0     5-6              6       9
      3    6     10       9     1     7-8              8      11
      3    7     15      13     2     8-9              9      13
      3    8     21      17     4    10-11            11      15
      3    9     28      22     6    11-12            12      17
      3   10     36      27     9    13-14            14      19
      4    4      1       1     0     4-4              4       7
      4    5      4       4     0     5-6              6       9
      4    6     10      10     0     6-7              7      11
      4    7     20      20     0     7-8              8      13
      4    8     35      35     0     9-10            10      15
      4    9      1       0     1    11-11            11      17
      5    5      1       1     0     5-5              5       9
      6    6      1       1     0     6-6              6      11
      7    7      1       1     0     7-7              7      13

## Observations across all 249 verified forms
  alphabet size (hi-lo+1) <= 2*tau-2 :  max excess over 2*tau-1 = -1
  lo >= A_1 - tau        :  min (lo - (A_1-tau)) = -2
  hi <= A_1 + tau - 1    :  max (hi - (A_1+tau-1)) = 0
  A_n inside I_n (hi<=kappa) fails for 30 forms; max overflow hi-kappa = 6
  states > 2*tau-1 (Theorem 12's bound) for 17 forms: [([1, 6], 16, 13), ([1, 7], 19, 15), ([2, 6], 17, 15), ([1, 8], 21, 17), ([2, 7], 20, 17), ([3, 6], 18, 17), ([1, 9], 26, 19), ([2, 8], 22, 19), ([3, 7], 21, 19), ([4, 6], 20, 19), ([1, 10], 29, 21), ([2, 9], 27, 21), ([3, 8], 23, 21), ([4, 7], 23, 21), ([5, 6], 22, 21), ([1, 11], 31, 23), ([1, 12], 36, 25)]
```



### The named case: the 4-rusty numbers `a = (1,4)`

`τ=5, κ=11, A₁=9`; `A₁`-bound is `7`, so Theorem 12 does not apply. Nine-state
base-5 DFAO, morphism / coding

        01234 01244 01254 01554 01230 01236 01784 01236 11236   coding 432156076

`A_{m+1} = 11m + 5 + T[m]`. All 11 obligations `TRUE` in **0.1 s / 5 MB**
(`results/attack3-rusty-transcript.txt`). The DFAO was checked term-by-term against
the mex definition for `n ≤ 2·10⁶`.

**It also explains the paper's anomaly.** Exactly one state, `q₈`, has
`δ(q₈,0) ≠ q₀` (`δ(q₈,0) = q₁`). `A_{5n+1}` is an arithmetic progression precisely
while `state(n) ≠ q₈`, because `state(5n) = δ(state(n),0)`. The **first** `n` whose
base-5 expansion lands in `q₈` is `n = 348` — the paper's breaking point.
`A₁₇₄₁ = 19148 = 11·1740 + 5 + T[1740]`, one less than `9 + 55·348 = 19149`.
So the arithmetic-progression property fails, the automaticity does not.

## 7. Verification, independently of the automata

- **Second generator.** A naive `mex`-over-a-`set` generator (no pointer, no array,
  written separately) agrees with `attack3_gen.antirec` on `A` and `B` for the first
  3000 terms for 13 forms across `k = 2,3,4`.
- **Against the literature.** The generator reproduces the published values exactly:
  A075326 `(1,1)`, A304502 `(1,2)`, A304499 `(2,1)`, A265389 `(1,1,1)`,
  A299409 `(1,1,1,1)`, the paper's anti-5-bonacci list `(1,1,1,1,1)`, and both rusty
  lists `(1,3)`, `(1,4)` from §4.
- **Replay of the committed artefacts** (`explore/attack3_recheck.py`): for all 249
  records, the DFAO is rebuilt from the *stored* `def` morphism/coding strings and
  replayed against a freshly run naive mex generator for `n ≤ 20000` — 0 mismatches.
  This path shares no code with the guesser.
- **The paper's own anomaly** reproduced to the index: first `n` with
  `A_{5n+1} ≠ 9 + 55n` is `n = 348`, `A₁₇₄₁ = 19148`.
- **Known theorems reproduced.** `(1,1)` (Zaslavsky), `(1,2)` and `(2,1)`
  (Fokkink–Joshi Thms 5, 6), `(1,1,1)` and `(1,1,1,1)` (Bosma et al.),
  `(1,1,1,1,1)` (the paper's guessed `a11111`) all verify. Minimal state counts
  2, 3, 4, 3, 4, 5 — and `(1,…,1)` of dimension 6 and 7 give 6 and 7 — so the
  anti-`k`-bonacci family matches Theorem 8's "the number of states is equal to `k`"
  exactly, out to `k = 7`. The paper does not tabulate counts for `(1,2)`/`(2,1)`.

## 8. Ledger

**Known before this run (reproduced here).**
Every `A₁`-bounded form — Fokkink–Joshi Theorem 12 — which is most of the sweep.
`a=(1,3)` is in a grey zone: it is *not* `A₁`-bounded, but the paper exhibits a
base-4 DFAO for it (Fig. 7) and says "the Walnut verification … can also be applied
to this DFAO". No transcript or state count is given there; we verify it
independently and find the minimal automaton has **6** states.

**New here.** Machine proofs of Conjecture 4 for every *non-`A₁`-bounded* form in the
sweep — i.e. for forms no published theorem covers. In particular the **rusty
numbers `a=(1,d)` for `d = 4,…,12`**, the case the paper names as a challenge, each
with an explicit minimal DFAO and a complete machine-checked induction.

The complete list of 61 non-`A₁`-bounded forms proved here (`k=2` then `k=3`, `k=4`):

```
(1,3) (1,4) (2,3) (1,5) (2,4) (3,3) (1,6) (2,5) (3,4) (4,3) (1,7) (2,6)
(3,5) (4,4) (5,3) (1,8) (2,7) (3,6) (4,5) (5,4) (6,3) (1,9) (2,8) (3,7)
(4,6) (5,5) (6,4) (7,3) (1,10) (2,9) (3,8) (4,7) (5,6) (6,5) (7,4) (8,3)
(1,11) (1,12) (1,1,4) (1,1,5) (1,2,4) (1,1,6) (1,2,5) (1,3,4) (2,1,5)
(1,1,7) (1,2,6) (1,3,5) (1,4,4) (2,1,6) (2,2,5) (1,1,8) (1,2,7) (1,3,6)
(1,4,5) (1,5,4) (2,1,7) (2,2,6) (2,3,5) (3,1,6) (1,1,1,6)
```

Plus three structural observations:

1. **The alphabet bound of Theorem 12 survives `A₁`-boundedness failing.** The
   difference sequence takes values in a window of width `≤ 2τ−2` in every case
   (Theorem 12 proves `2τ−1` possible values, but only under `A₁`-boundedness).
   Window: `lo ≥ A₁−τ−2` and `hi ≤ A₁+τ−1` throughout.
2. **A state-count law for the rusty numbers.** The minimal base-`τ` DFAO of
   `a=(1,d)` has

           |Q|(d) = 2, 3, 6, 9, 11, 16, 19, 21, 26, 29, 31, 36     (d = 1..12)

   and

           |Q|(d+3) = |Q|(d) + 10        for every d >= 3   (7 instances, d = 3..9)

   — a period-3 arithmetic law, unexplained (for reference `τ(d+3)−τ(d) = 3`,
   `κ(d+3)−κ(d) = 6`). It fails at `d = 1, 2` (`+7`, `+8`), i.e. exactly on the two
   `A₁`-bounded members of the family. Every value is the Myhill–Nerode quotient of a
   machine-verified automaton, so these are exact minimal state counts, not estimates.
   **Two of them were predictions made before the run:** the law predicted
   `|Q|(11) = 31` and `|Q|(12) = 36`; both were then confirmed, the second only after
   an engine counterexample killed a 35-state candidate that survived 2.5·10⁷ terms.
   Outstanding prediction: `|Q|(13) = 39`, `|Q|(14) = 41`, `|Q|(15) = 46`.
3. **The conclusions of Lemma 11 and Theorem 12 do not extend past their own hypothesis.**
   `|Q| > 2τ−1` happens for 17 of the 249 forms; the smallest witness is `a=(1,6)` (`τ=7`):
   the minimal DFAO has **16** states, above **13**. The others are
   `(1,7),(1,8),(1,9),(1,10)`, `(1,11),(1,12)`, `(2,6),(2,7),(2,8),(2,9)`, `(3,6),(3,7),
   (3,8)`, `(4,6),(4,7)`, `(5,6)` — i.e. all have `k = 2` and `a₂ ≥ 6`. Equivalently, `Aₙ`
   escapes its interval `Iₙ = [κ(n−1)+1, κn]` (upward, by up to `6`) for 30 of the verified
   forms — first at `a=(1,4)`. **This is not a failure of anything published**: Lemma 11
   and Theorem 12 are both stated only for `A₁`-bounded forms, and every `A₁`-bounded form
   in the sweep does satisfy both — `hi ≤ κ`, `lo ≥ A₁−τ+1`, `hi ≤ A₁+τ−1`, `|Q| ≤ 2τ−1`,
   with no exception found. The correct reading is that the *conclusions* of Lemma 11 and
   Theorem 12 fail outside their `A₁`-bounded hypothesis, which is exactly what makes the
   general case hard — not that the published statements are contradicted. `(1,3)` is
   *not* `A₁`-bounded yet still satisfies both conclusions, which is why the paper's
   `k`-bonacci-style picture still works there and breaks first at `d = 4` — exactly the
   case the paper flags. (`paper/attack3-verdict.md` §6.)

**Not done / failed.**
- **Conjecture 4 is not proved in general.** What is proved is a finite (large) set
  of instances. Nothing here gives a uniform-in-`a` argument, and the observation
  that `|Q|` outruns `2τ−1` says a general proof cannot just relax Theorem 12's
  constant — the substitution alphabet is no longer "value of `Aₙ mod κ`".
- No infinite subfamily was closed. The natural next step is a *bounded-window
  lemma* (`lo ≥ A₁−τ−2`, `hi ≤ A₁+τ−1`, observation 1) proved uniformly in `a`,
  plus "τ-uniform substitution with bounded look-ahead ⇒ τ-automatic".
- Guessing, not verification, is the bottleneck as `τ` grows: `a=(1,9)` needed an
  `8·10⁶`-term prefix, `(1,10)` `1.2·10⁷`, `(2,9)` `1.6·10⁷`, `(1,12)` `7·10⁷` — the
  last two located by the engine's counterexample rather than by blind escalation.
  Verifying any of them costs seconds to a minute (`(1,12)`: 36 states, base 13,
  16.7 s, 2.9 GB). `k = 2` with `τ ≥ 14` was not attempted; `k = 3` stops at `τ = 11`
  and `k = 4` at `τ = 9`.
- `learnfe` was **not** used: no factor-equality predicate arises in this encoding —
  the self-verifying object here is the mex recurrence, discharged directly by `?`.

## 9. Reproduce

```
python3 explore/attack3_gen.py 20000            # sequences + first guesses
python3 explore/attack3_general.py "[(1,4)]"     # guess + verify one form
python3 explore/attack3_sweep.py "{2:10,3:8,4:7}"   # the sweep -> results/attack3.jsonl
python3 explore/attack3_retry.py "[(1,9)]" 8000000  # longer prefix for stragglers
python3 explore/attack3_table.py                 # the tables in §6
python3 explore/attack3_recheck.py 20000         # independent replay of every record
python3 explore/attack3_seqcheck.py              # engine `seq` vs Python, checks `def`
```

Files: `explore/attack3_gen.py` (generator + kernel guesser),
`attack3_guess.py` (ladder + minimisation), `attack3_general.py` (general-`k`
encoding + verification), `attack3_sweep.py`, `attack3_retry.py`,
`attack3_table.py`; `explore/attack3_verify.py` is the earlier `k=2`-only encoding,
kept because it is a *different* encoding of the same obligations (it existentially
quantifies the block gap instead of splitting on `c_g`) and therefore an independent
check — it agrees with the general one on every `a=(1,d)`, `d ≤ 8`.
