# LEARN — self-verifying predicates beyond equality of factors

`docs/LEARNFE.md` describes `learnfe`, which builds the equality-of-factors automaton

    FE(i,j,l)  :=  A t. t < l => T[i+t] = T[j+t]

by guessing it with an active learner and then *verifying* the guess against a
recurrence with a unique solution — Khodier's "self-verifying predicate" idea (thesis
ch. 3 / arXiv:2507.19717), which sidesteps the universal quantifier over `t` and hence
the subset construction that blows up.

`learn` generalises that from one predicate to a class of them.

    learn NAME fe                    FE(i,j,l)    A t<l. T[i+t] = T[j+t]
    learn NAME rev                   REV(i,j,l)   A t<l. T[i+t] = T[j+l-1-t]
    learn NAME period                PER(i,l,p)   A t. t+p<l => T[i+t] = T[i+t+p]
    learn NAME border                BOR(i,l,b)   b<=l & A t<b. T[i+t] = T[i+l-b+t]
    learn NAME (v1,..,vn) [on:v] init:PHI0 step:PHI1        your own recurrence

`learnfe NAME` is exactly `learn NAME fe` and is unchanged (same code path, same
counterexample order, same automata, same query counts).

The result is registered like `let`: `$NAME(...)` is callable from any later formula,
with the parameters in the order shown above.

    def T 2 2 0 01 10 01
    learn RV rev
    OK learn RV(i,j,l) kind=rev states=31 iters=3 eqs=1 ces=30 mqs=80537 steps=69606 peak=93 ms=43
    ? A n. E i. $RV(i,i,n)                      # a palindrome of every length?
    TRUE states=1 peak=... ms=2

`REV(i,i,n)` is "the factor of length `n` at `i` is a palindrome": the four classes are
chosen because they are the queries the palindrome and critical-exponent ladders are
built out of.

---

## 1. Why each recurrence pins its predicate down

The pattern is always the same. A predicate class is admissible here when it satisfies
a recurrence that (a) has a **unique** solution and (b) can be written as finitely many
**open** formulas that must hold at every point — no `forall`, so no complement /
subset-construction / complement sandwich at verification time. Then any candidate `H`
that passes the check *is* the predicate, however it was obtained: the learner may be
heuristic and its oracle may even be wrong on some inputs; a bad guess costs iterations,
never correctness.

Below, `T` is the sequence and `H` an arbitrary predicate `N^3 -> {true,false}`.

### FE — equality of factors

    (C1)  H(i,j,0)
    (C2)  H(i,j,l+1)  <=>  ( T[i]=T[j]  and  H(i+1,j+1,l) )

Induction on `l`, uniformly in `(i,j)`. Base: (C1) makes `H(i,j,0)` true and
`FE(i,j,0)` is vacuously true. Step: assume `H(·,·,l) = FE(·,·,l)` everywhere; then
`H(i,j,l+1) = T[i]=T[j] & FE(i+1,j+1,l) = FE(i,j,l+1)`, splitting off `t=0`. Proof in
full: `docs/LEARNFE.md` §2.

### REV — equality with a reversed factor

    REV(i,j,l) := A t<l. T[i+t] = T[j+l-1-t]

    (C1)  H(i,j,0)
    (C2)  H(i,j,l+1)  <=>  ( T[i]=T[j+l]  and  H(i+1,j,l) )

`REV(i,j,l+1)` is `A t<=l. T[i+t] = T[j+l-t]`. Splitting off `t=0` gives
`T[i]=T[j+l]`; the remaining conditions, reindexed by `t' = t-1`, are
`A t'<l. T[(i+1)+t'] = T[j+(l-1)-t']`, i.e. `REV(i+1,j,l)`. Note the **second index
does not advance**: the window at `j` is consumed from its right end, which is why the
recurrence stays inside the Presburger fragment (`j+l` is linear; `j+l-1-t` is not an
admissible index term on its own). Base `l=0` vacuous. Induction on `l`, uniformly in
`(i,j)`, exactly as for FE.

`REV(i,i,l)` is "the length-`l` factor at `i` is a palindrome"; `REV(i,j,l)` with
`i != j` is "the factor at `i` is the reversal of the factor at `j`".

### PERIOD

    PER(i,l,p) := A t. t+p < l => T[i+t] = T[i+t+p]

    (C1)  H(i,0,p)
    (C2)  H(i,l+1,p)  <=>  ( l+1<=p  or  ( T[i]=T[i+p]  and  H(i+1,l,p) ) )

`PER(i,l+1,p)` quantifies over `t < l+1-p`. If `l+1 <= p` that set is empty and the
predicate is (vacuously) true — this is the convention that `p >= l` is a period of a
length-`l` factor. Otherwise `t=0` contributes `T[i]=T[i+p]` and the rest, reindexed,
is `PER(i+1,l,p)`. Induction on `l`, uniformly in `(i,p)`.

`PER` is the exponent predicate: a factor of length `l` and period `p` has exponent
`l/p`, so "some factor has exponent `>= a/b`" is
`E i,n,l. n>=1 & b*l = a*n & $PER(i,l,n)`.

### BORDER

    BOR(i,l,b) := b<=l  &  A t<b. T[i+t] = T[i+l-b+t]

(the length-`b` prefix and the length-`b` suffix of the length-`l` factor at `i` agree).

    (C1)   H(i,l,0)
    (C1')  H(i,0,b)  <=>  b=0
    (C2)   H(i,l+1,b+1)  <=>  ( T[i+b]=T[i+l]  and  H(i,l,b) )

Here the recurrence moves along the **diagonal** `(l,b) -> (l+1,b+1)`, because what the
comparison actually depends on is the offset `l-b`, which the diagonal keeps fixed.
Splitting off the *last* index `t=b-1` of `BOR(i,l+1,b+1)` gives `T[i+b]=T[i+l]`, and
the rest is `BOR(i,l,b)` — same offset, one shorter. Uniqueness is induction on
`min(l,b)`: if `min(l,b)=0` then one of the two base sentences applies (and `b>l` is
false, as `BOR` requires `b<=l`); otherwise `(l,b) = (l'+1,b'+1)` with `min(l',b')` one
smaller. Every `(l,b)` is reached from a base point — from `(l-b,0)` when `b<=l`, from
`(0,b-l)` otherwise — so the two bases plus the diagonal step determine `H` everywhere.

This is the one built-in class whose recurrence is not a single-coordinate induction,
which is why it is built in rather than expressible in the generic template below.

### Your own predicate

    learn NAME (v1,...,vn) [on:v] init:PHI0 step:PHI1

builds and checks

    (C1)  H(v1,..,0,..,vn)    <=>  PHI0        (v replaced by 0, also inside PHI0)
    (C2)  H(v1,..,v+1,..,vn)  <=>  PHI1

where `v` is the recursion coordinate (`on:v`; default: the last parameter), `$H` is the
hole, and `PHI0`/`PHI1` are ordinary engine formulas over the parameters, `T[...]`, and
any predicate already bound by `let`/`learn`.

**When this is sound.** (C1)&(C2) has exactly one solution — so that passing the check
proves `H` is the predicate you meant, provided the predicate you meant satisfies
(C1)&(C2) — exactly when

* `PHI0` does not mention `$H`, and
* every occurrence of `$H` in `PHI1` has the recursion coordinate equal to the bare
  variable `v` (or to the constant `0`).

Then the slice `v = L+1` of `H` is a function of the slice `v = L` (and of `T` and
arithmetic), so induction on `v` determines `H` everywhere. Both conditions are checked
before anything is learned, and violating either is an error, not a warning:

    learn BAD (i,j,l) init:true step:$H(i,j,l+1)
    ERR learn every $H in step: must use l (or 0) as its 3-th argument, so that H at l+1
        depends only on H at l; otherwise the recurrence need not have a unique solution

Quantifiers are refused in `PHI0`/`PHI1` too. Not for soundness — the verifier would
compile them correctly — but because the membership oracle for a user-supplied class is
the recurrence itself, unrolled (§2), and unrolling cannot evaluate a quantifier.

What the check does **not** do is confirm that your recurrence describes the predicate
you had in mind. It proves `H` is the unique solution of the recurrence you wrote. FE
and REV written out by hand reproduce the built-ins exactly:

    learn G  (i,j,l) init:true step:(T[i]=T[j]) & ($H(i+1,j+1,l))     # = fe
    learn G2 (i,j,l) init:true step:(T[i]=T[j+l]) & ($H(i+1,j,l))     # = rev
    learn G4 (i,l,p) on:l init:true step:(l+1<=p) | ((T[i]=T[i+p]) & ($H(i+1,l,p)))   # = period

each verified equal to its built-in inside the engine (`A i,j,l. $G(i,j,l) <=> $FE(i,j,l)`
returns `TRUE`).

### How the sentences are actually checked

Each sentence has the shape "for all free variables, `Phi`", so the engine compiles only
the **open** formula `Phi` over those variables and asks whether the resulting (trimmed,
minimised) DFA has a non-accepting reachable state. Since every tuple of naturals has a
representation and every constituent automaton is value-based, "all reachable states
accept" is exactly "the sentence holds". No `forall` is ever compiled. Cost: `iff`
products of automata that are *equal languages* once `H` is right, so the reachable
product stays O(|H|), not O(|H|²).

A witness of a violated sentence is a point where the *recurrence* fails, not directly a
point where `H` differs from the predicate. But the predicate satisfies the recurrence,
so if `H` agreed with it at every `$H`-argument tuple occurring in that sentence, the
sentence would hold there. Hence at least one of those tuples is a genuine
counterexample; all of them are handed to the learner, which tests each against the
membership oracle and keeps the ones that differ. Witnesses come from `Dfa::bfs_tree` /
`word_to`: a **shortest** word reaching each rejecting state, one per rejecting state (up
to `AM_LEARN_WITNESS`, default 256).

## 2. Membership oracles

Direct evaluation on the sequence, never through an automaton. All four built-in classes
reduce to one of two longest-common-extension walks:

    LCP(i,j)   = max m with T[i+t] = T[j+t]   for all t < m        (forward / forward)
    RLCE(i,e)  = max m with T[i+t] = T[e-t]   for all t < m        (forward / backward)

    FE(i,j,l)   <=>  l <= LCP(i,j)
    REV(i,j,l)  <=>  l = 0  or  l <= RLCE(i, j+l-1)
    PER(i,l,p)  <=>  l <= p  or  l-p <= LCP(i, i+p)
    BOR(i,l,b)  <=>  b <= l  and  b <= LCP(i, i+l-b)

Both walks read the DFAO with counters that carry their state along the digit path, so a
position step is O(1) amortised and memory is O(log n) — no prefix array is materialised
and positions of any size are reachable. Results are memoised per pair. `RLCE` needs a
*decrementing* counter: in base `k` that is the schoolbook borrow, O(1) amortised like
the increment; under a `numsys` there is no in-place predecessor (`numsys::succ` has no
mirror), so the backward counter re-derives its digits from the value at each step,
O(width) per step. That is the only place where a `numsys` costs the oracle more than
base `k` does, and it only affects `rev`.

A hard cap `AM_LEARN_LCP` (default 2²²) bounds the work per pair; a pair that survives
the cap is treated as matching forever. That is genuinely wrong for eventually periodic
`T` — and harmless, by §1: the recurrence check, not the oracle, decides correctness.
It can only cost convergence, and a stall is detected and answered by raising the cap
16× and relearning from scratch (up to `AM_LEARN_LCP_MAX`, default 2²⁶).

For a user-supplied class there is no closed form, so the oracle is **the recurrence
itself, unrolled**: `H(v)` at recursion coordinate `0` evaluates `PHI0`, and at `L+1`
evaluates `PHI1` with the coordinate bound to `L`, recursing on `$H` and memoising every
tuple. `T[...]`, arithmetic, and calls to already-built predicates (run as automata on
the encoded values) are all evaluated concretely. The unrolling is capped at
`AM_LEARN_UNROLL` levels (default 5000) and the sampler is told not to draw recursion
coordinates it cannot afford, so a user-supplied class learns from a smaller magnitude
window than a built-in one does.

## 3. Learner

Unchanged from `docs/LEARNFE.md` §4 except that it is now generic in the number of
tracks (`k^n` letters for `n` parameters) and that the two heuristics which know about
the *shape* of the language are per class:

* **boundary sampling.** Each class has a surface where one more matching position flips
  the answer, and that is where counterexamples live: `l = LCP(i,j)` for FE;
  `l = RLCE(i,e)` for REV (draw `(i,e)` and read `j = e+1-l` back off); `l = p + LCP(i,i+p)`
  for PERIOD; `b = LCP(i,i+d)` at fixed offset `d = l-b` for BORDER, plus draws in the
  `b > l` region where the predicate is false for a different reason. A user-supplied
  class has no known boundary and gets plain magnitude-mixed random tuples.
* **local probe.** Errors cluster, so from every counterexample found we crawl a bounded
  magnitude-preserving neighbourhood: the length-like coordinate ±3, each coordinate ±1,
  the step the recurrence itself takes (`(i+1,j+1,l-1)` for FE, `(i+1,j,l-1)` for REV,
  `(i+1,l-1,p)` for PERIOD, `(i,l-1,b-1)` for BORDER), and all coordinates divided by `k`.
  Magnitude-preserving on purpose: enlarging the coordinates lengthens the words, long
  words become long distinguishing suffixes, and those make every later sift an expensive
  long walk.

The FE neighbourhood and sampler are byte-for-byte the ones `learnfe` already used, so
FE results are unchanged (§4.1).

## 4. Results

`states` are minimal DFA states including the dead state (this repo's convention;
Walnut reports one fewer). msd. Machine: M-series Mac, 24 GB, release build, engines
launched through `explore/engine.py` admission control, **with three other build
agents using the same machine throughout** — every wall time below is an upper bound
and the learned/direct pairs in one row were run minutes apart, so treat ratios below
about 1.5x as noise. Panel budgets: `learn` 3 GB / 600 s, `let` 4 GB / 240 s,
equivalence check 4 GB / 840 s. Raw rows: `results/learn_panel.json`.

### 4.1 FE is unchanged

`learn NAME fe` and `learnfe NAME` are the same code path, and the generalisation kept
the FE sampler and the FE local-probe neighbourhood byte-for-byte.

Direct regression against a **pristine build of the engine at the commit before this
work** (`git archive HEAD engine`, built in its own target directory): `learnfe` run on
all 19 panel sequences plus Tribonacci and Fibonacci, comparing every field of the reply
line except `ms` and `peak`. **21 cases compared, 0 differences** — identical `states`,
`iters`, `eqs`, `ces` and `mqs` throughout, i.e. the learner issued exactly the same
sequence of oracle queries it issued before the refactor and reached exactly the same
automaton. Reproduce with `python3 explore/learn_bench.py regress OLD_BINARY NEW_BINARY`;
the numbers also match the table in `docs/LEARNFE.md` §6 row for row.

### 4.2 The panel: learned vs the direct `let` construction

76 (sequence, class) pairs. **66 pairs have both** — in all 66 the learned automaton has
exactly the same number of minimal states as the direct one, and `? A vars. $L(vars)
<=> $D(vars)` returns `TRUE` (the engine proving the two languages equal). **0
mismatches.**

Of the remaining 10, **8 are cases the direct construction cannot do** at 4 GB / 240 s
and the learner can — new automata with no direct value to compare against, checked
instead by brute force (§4.4):

| sequence | class | learned states | learn s | learn MB |
|---|---|---|---|---|
| tail-b | fe | 1000 | 36.0 | 172 |
| tail-c | fe | 1382 | 18.2 | 50 |
| tail-c | rev | 1442 | 20.3 | 50 |
| tail-a | period | 1937 | 16.2 | 146 |
| tail-b | period | 2045 | 244.1 | 811 |
| tail-c | period | 1861 | 19.5 | 50 |
| tail-b | border | 1717 | 101.4 | 237 |
| tail-c | border | 2086 | 46.6 | 50 |

The other 2 go the other way. Re-run at 6 GB / 2400 s (`results/learn_panel_gaps.json`):
`single6` rev reaches **8683 states in 1495 s / 3399 MB** — the same 8683 the direct
construction gets in 61.3 s, so the sizes agree there too, making it **67 of 67
agreements and 0 mismatches** over every pair where both methods produce an answer at
any budget tried. `single6` period still exceeds 6 GB in the learner (direct: 5916
states, 68.4 s). One further pair, `tail-a` border, is out of reach for both: the direct
construction times out at 240 s and the learner at 2400 s / 6 GB.

**fe**

| sequence | learned | `let` | same | learn s | `let` s | learn MB | `let` MB | proved equal |
|---|---|---|---|---|---|---|---|---|
| thue-morse | 15 | 15 | yes | 0.1 | 0.0 | 0 | 0 | TRUE |
| period-doubling | 8 | 8 | yes | 0.3 | 0.0 | 0 | 0 | TRUE |
| rudin-shapiro | 68 | 68 | yes | 0.3 | 0.1 | 1 | 88 | TRUE |
| paperfolding | 44 | 44 | yes | 0.7 | 0.0 | 1 | 0 | TRUE |
| cantor | 17 | 17 | yes | 1.5 | 0.0 | 1 | 0 | TRUE |
| mephisto | 14 | 14 | yes | 0.1 | 0.0 | 1 | 1 | TRUE |
| prism-1 | 467 | 467 | yes | 18.2 | 44.6 | 199 | 312 | TRUE |
| prism-a | 24 | 24 | yes | 0.5 | 0.0 | 1 | 0 | TRUE |
| prism-d | 82 | 82 | yes | 0.6 | 0.5 | 7 | 132 | TRUE |
| single3 | 190 | 190 | yes | 0.5 | 0.0 | 3 | 6 | TRUE |
| single4 | 698 | 698 | yes | 5.4 | 0.2 | 12 | 10 | TRUE |
| single5 | 1877 | 1877 | yes | 23.6 | 2.3 | 50 | 30 | TRUE |
| single6 | 3971 | 3971 | yes | 203.5 | 28.6 | 627 | 169 | TRUE |
| champion-m5 | 199 | 199 | yes | 12.3 | 0.0 | 3 | 6 | TRUE |
| k3m3-artefact-a | 216 | 216 | yes | 2.5 | 0.1 | 21 | 7 | TRUE |
| k3m3-artefact-b | 71 | 71 | yes | 0.6 | 0.0 | 6 | 6 | TRUE |
| tail-a | 1165 | 1165 | yes | 9.4 | 187.4 | 47 | 1480 | TRUE |
| tail-b | 1000 | **timeout** | — | 36.0 | — | 172 | None | — |
| tail-c | 1382 | **budget** | — | 18.2 | — | 50 | None | — |

**rev**

| sequence | learned | `let` | same | learn s | `let` s | learn MB | `let` MB | proved equal |
|---|---|---|---|---|---|---|---|---|
| thue-morse | 31 | 31 | yes | 0.0 | 0.0 | 2 | 0 | TRUE |
| period-doubling | 9 | 9 | yes | 0.0 | 0.0 | 1 | 0 | TRUE |
| rudin-shapiro | 102 | 102 | yes | 0.3 | 0.0 | 6 | 6 | TRUE |
| paperfolding | 51 | 51 | yes | 0.1 | 0.0 | 1 | 6 | TRUE |
| cantor | 20 | 20 | yes | 0.9 | 0.0 | 2 | 1 | TRUE |
| mephisto | 23 | 23 | yes | 0.1 | 0.0 | 3 | 6 | TRUE |
| prism-1 | 405 | 405 | yes | 40.7 | 52.0 | 189 | 236 | TRUE |
| prism-a | 39 | 39 | yes | 0.1 | 0.0 | 3 | 0 | TRUE |
| prism-d | 133 | 133 | yes | 0.8 | 1.1 | 13 | 307 | TRUE |
| single3 | 344 | 344 | yes | 0.9 | 0.0 | 12 | 6 | TRUE |
| single4 | 1333 | 1333 | yes | 8.6 | 0.3 | 50 | 13 | TRUE |
| single5 | 3792 | 3792 | yes | 117.4 | 4.5 | 100 | 48 | TRUE |
| single6 | — | 8683 | — | — | 61.3 | None | 307 | — |
| champion-m5 | 250 | 250 | yes | 15.0 | 0.1 | 12 | 7 | TRUE |
| k3m3-artefact-a | 217 | 217 | yes | 2.1 | 0.1 | 25 | 7 | TRUE |
| k3m3-artefact-b | 87 | 87 | yes | 0.3 | 0.0 | 8 | 5 | TRUE |
| tail-a | 1075 | 1075 | yes | 8.1 | 202.4 | 50 | 1526 | TRUE |
| tail-b | 947 | 947 | yes | 35.3 | 230.9 | 101 | 1498 | TRUE |
| tail-c | 1442 | **timeout** | — | 20.3 | — | 50 | None | — |

**period**

| sequence | learned | `let` | same | learn s | `let` s | learn MB | `let` MB | proved equal |
|---|---|---|---|---|---|---|---|---|
| thue-morse | 27 | 27 | yes | 0.1 | 0.0 | 1 | 1 | TRUE |
| period-doubling | 16 | 16 | yes | 0.2 | 0.0 | 1 | 0 | TRUE |
| rudin-shapiro | 114 | 114 | yes | 0.4 | 0.0 | 1 | 6 | TRUE |
| paperfolding | 68 | 68 | yes | 0.2 | 0.0 | 1 | 11 | TRUE |
| cantor | 28 | 28 | yes | 1.4 | 0.0 | 1 | 2 | TRUE |
| mephisto | 27 | 27 | yes | 0.1 | 0.0 | 1 | 10 | TRUE |
| prism-1 | 1017 | 1017 | yes | 37.9 | 139.8 | 255 | 357 | TRUE |
| prism-a | 28 | 28 | yes | 0.4 | 0.0 | 1 | 0 | TRUE |
| prism-d | 148 | 148 | yes | 0.4 | 1.5 | 6 | 423 | TRUE |
| single3 | 299 | 299 | yes | 0.7 | 0.0 | 6 | 8 | TRUE |
| single4 | 1027 | 1027 | yes | 3.5 | 0.4 | 14 | 18 | TRUE |
| single5 | 2735 | 2735 | yes | 25.1 | 5.7 | 64 | 46 | TRUE |
| single6 | — | 5916 | — | — | 68.4 | None | 279 | — |
| champion-m5 | 271 | 271 | yes | 12.4 | 0.1 | 6 | 8 | TRUE |
| k3m3-artefact-a | 303 | 303 | yes | 2.9 | 0.1 | 13 | 8 | TRUE |
| k3m3-artefact-b | 123 | 123 | yes | 0.4 | 0.0 | 6 | 6 | TRUE |
| tail-a | 1937 | **timeout** | — | 16.2 | — | 146 | None | — |
| tail-b | 2045 | **timeout** | — | 244.1 | — | 811 | None | — |
| tail-c | 1861 | **timeout** | — | 19.5 | — | 50 | None | — |

**border**

| sequence | learned | `let` | same | learn s | `let` s | learn MB | `let` MB | proved equal |
|---|---|---|---|---|---|---|---|---|
| thue-morse | 23 | 23 | yes | 0.1 | 0.0 | 1 | 4 | TRUE |
| period-doubling | 13 | 13 | yes | 0.1 | 0.0 | 1 | 0 | TRUE |
| rudin-shapiro | 128 | 128 | yes | 0.4 | 0.0 | 3 | 6 | TRUE |
| paperfolding | 74 | 74 | yes | 0.4 | 0.1 | 1 | 55 | TRUE |
| cantor | 34 | 34 | yes | 1.7 | 0.0 | 3 | 3 | TRUE |
| mephisto | 25 | 25 | yes | 0.1 | 0.1 | 2 | 25 | TRUE |
| prism-1 | 1000 | 1000 | yes | 62.4 | 145.2 | 615 | 357 | TRUE |
| prism-a | 24 | 24 | yes | 0.3 | 0.0 | 1 | 0 | TRUE |
| prism-d | 120 | 120 | yes | 0.6 | 1.1 | 11 | 288 | TRUE |
| single3 | 282 | 282 | yes | 0.6 | 0.0 | 3 | 8 | TRUE |
| single4 | 1018 | 1018 | yes | 6.4 | 0.4 | 25 | 18 | TRUE |
| single5 | 2741 | 2741 | yes | 51.3 | 6.3 | 53 | 46 | TRUE |
| single6 | 5883 | 5883 | yes | 524.5 | 89.8 | 349 | 279 | TRUE |
| champion-m5 | 292 | 292 | yes | 15.6 | 0.1 | 6 | 7 | TRUE |
| k3m3-artefact-a | 331 | 331 | yes | 5.1 | 0.1 | 40 | 8 | TRUE |
| k3m3-artefact-b | 109 | 109 | yes | 0.3 | 0.0 | 11 | 6 | TRUE |
| tail-a | — | **timeout** | — | — | — | None | None | — |
| tail-b | 1717 | **timeout** | — | 101.4 | — | 237 | None | — |
| tail-c | 2086 | **timeout** | — | 46.6 | — | 50 | None | — |

### 4.3 Numeration systems

Tribonacci (`numsys trib`, `dfao TR 2 0:0,1 1:0,2 2:0,-`) and Fibonacci
(`numsys fib`, `dfao F 2 0:0,1 1:0,-`), 6 GB / 900 s. Raw:
`results/learn_numsys.json`.

| sequence | class | learned | `let` | same | learn s | `let` s | proved equal |
|---|---|---|---|---|---|---|---|
| tribonacci | fe | 27 | 27 | yes | 0.08 | 3.55 | TRUE |
| tribonacci | rev | 211 | 211 | yes | 84.71 | 65.48 | TRUE |
| tribonacci | period | 404 | 404 | yes | 1.01 | 120.29 | TRUE |
| tribonacci | border | 286 | budget | — | 1.81 | — | — |
| fibonacci | fe | 12 | 12 | yes | 0.04 | 0.00 | TRUE |
| fibonacci | rev | 35 | 35 | yes | 0.12 | 0.01 | TRUE |
| fibonacci | period | 47 | 47 | yes | 0.11 | 0.00 | TRUE |
| fibonacci | border | 42 | 42 | yes | 0.09 | 0.01 | TRUE |

Tribonacci PERIOD is the largest gap in this table: 1.01 s learned against 120.29 s for
the direct construction, same 404 states, proved equal. Tribonacci BORDER (286 states)
has no direct value at all — `let` exhausts 6 GB. Tribonacci REV is the one case in the
table where learning *loses*: 84.71 s against 65.48 s. On Fibonacci every direct
construction finishes in milliseconds and learning loses by 10-100x; that is the
expected shape (see §4.6).

### 4.4 Correctness

Four independent checks.

1. **Same minimal size as the direct construction**, on all 66 panel pairs where both
   finish inside the panel budget, plus `single6` rev at 6 GB, plus 7 of 8
   numeration-system pairs: 74/74 identical, 0 differences.
2. **Proved equal inside the engine** — `? A vars. $L(vars) <=> $D(vars)` — on the same
   73: `TRUE` every time.
3. **Pure-Python brute force** (`explore/learn_bench.py brute`), written from the
   mathematical definitions in §1 and sharing no code with the engine: the predicate is
   enumerated over a morphism-generated prefix for all tuples with every coordinate < 12
   and diffed against `enum 12 $L(...)`. 29 of 31 (sequence, class) combinations tried
   pass; the two that are missing are `single6` rev/period, where `learn` did not finish
   inside the check's own budget. This is the only independent check available for the
   eight automata the direct construction cannot build, and it covers all eight
   (`tail-b` and `tail-c` x all four classes).

* thue-morse     fe      PASS (380 tuples, coords < 12)
* thue-morse     rev     PASS (374 tuples, coords < 12)
* thue-morse     period  PASS (1167 tuples, coords < 12)
* thue-morse     border  PASS (375 tuples, coords < 12)
* rudin-shapiro  fe      PASS (416 tuples, coords < 12)
* rudin-shapiro  rev     PASS (352 tuples, coords < 12)
* rudin-shapiro  period  PASS (1177 tuples, coords < 12)
* rudin-shapiro  border  PASS (385 tuples, coords < 12)
* cantor         fe      PASS (442 tuples, coords < 12)
* cantor         rev     PASS (408 tuples, coords < 12)
* cantor         period  PASS (1236 tuples, coords < 12)
* cantor         border  PASS (444 tuples, coords < 12)
* prism-1        fe      PASS (370 tuples, coords < 12)
* prism-1        rev     PASS (259 tuples, coords < 12)
* prism-1        period  PASS (1143 tuples, coords < 12)
* prism-1        border  PASS (351 tuples, coords < 12)
* single6        fe      PASS (412 tuples, coords < 12)
* single6        rev     FAIL (None tuples, coords < 12)
* single6        period  FAIL (None tuples, coords < 12)
* single6        border  PASS (433 tuples, coords < 12)
* tail-a         fe      PASS (584 tuples, coords < 12)
* tail-a         rev     PASS (544 tuples, coords < 12)
* tail-a         period  PASS (1264 tuples, coords < 12)
* tail-b         fe      PASS (394 tuples, coords < 12)
* tail-b         rev     PASS (367 tuples, coords < 12)
* tail-b         period  PASS (1164 tuples, coords < 12)
* tail-b         border  PASS (372 tuples, coords < 12)
* tail-c         fe      PASS (524 tuples, coords < 12)
* tail-c         rev     PASS (521 tuples, coords < 12)
* tail-c         period  PASS (1261 tuples, coords < 12)
* tail-c         border  PASS (469 tuples, coords < 12)

4. **200 random formulas** (`explore/learn_bench.py fuzz`, seed 653658211): closed
   formulas with bounded quantifiers over `$P(...)`, five templates, random panel
   sequence and random class per formula, each run twice — once with the learned
   predicate, once with the direct `let` predicate — and the `TRUE`/`FALSE` verdicts
   compared.

   **184 agree, 0 disagree, 16 incomplete** (of 200). The 16 are resource exhaustion on
   one side or the other, not verdicts: `single6` rev (6), `tail-c` fe (5), `tail-c`
   border (2), `tail-c` period (2), `single6` period (1) — the same hard corners as
   §4.2, re-run once at 6 GB / 900 s before being counted as incomplete. Every formula
   on which both routes produced a verdict produced the *same* verdict. Class mix over
   the 200: rev 62, fe 53, period 53, border 32; sequence drawn uniformly from the
   panel.

### 4.5 End-to-end: the palindrome and critical-exponent ladders

The reason to learn REV and PERIOD is that they are the predicates the standard ladders
are built from. Each ladder was run three ways in one engine session: **learned** (build
the predicate with `learn`, then the queries), **`let`** (build the same predicate with
the direct construction, then the same queries), and **inline** (no predicate at all —
the universally quantified body written into every query). 6 GB / 1800 s.

    palindrome of every length   ? A n. E i. (REV(i,i,n))
    exponent >= a (integer)      ? E i,n. n>=1 & (PER(i,a*n,n))
    exponent >= a/b              ? E i,n,l. n>=1 & b*l >= a*n & (PER(i,l,n))

| case | ladder | learned s | `let` s | inline s | learned MB | `let` MB | inline MB |
|---|---|---|---|---|---|---|---|
| tribonacci | palindrome | 88.0 | 75.2 | 0.8 | 3 | 4891 | 140 |
| tribonacci | exponent | 25.7 | 148.1 | 242.1 | 5393 | 5393 | 5393 |
| prism-1 | palindrome | 37.5 | 41.0 | 0.1 | 189 | 236 | 25 |
| prism-1 | exponent | 37.2 | 120.4 | 244.4 | 255 | 357 | 357 |

**tribonacci / palindrome**

| query | learned | `let` | inline |
|---|---|---|---|
| palindrome-of-every-length | TRUE | TRUE | TRUE |

**tribonacci / exponent**

| query | learned | `let` | inline |
|---|---|---|---|
| exp>=2 | TRUE | TRUE | TRUE |
| exp>=5/2 | TRUE | TRUE | TRUE |
| exp>=3 | TRUE | TRUE | TRUE |
| exp>=7/2 | FALSE | FALSE | FALSE |
| exp>=4 | FALSE | FALSE | FALSE |

**prism-1 / palindrome**

| query | learned | `let` | inline |
|---|---|---|---|
| palindrome-of-every-length | FALSE | FALSE | FALSE |

**prism-1 / exponent**

| query | learned | `let` | inline |
|---|---|---|---|
| exp>=2 | TRUE | TRUE | TRUE |
| exp>=5/2 | TRUE | TRUE | TRUE |
| exp>=3 | TRUE | TRUE | TRUE |
| exp>=7/2 | TRUE | TRUE | TRUE |
| exp>=4 | TRUE | TRUE | TRUE |

Two opposite results, both worth stating plainly.

* **The exponent ladder is where learning pays.** Tribonacci: 25.7 s learned vs 148.1 s
  `let` (5.8x) vs 242.1 s inline (9.4x). PRISM-1: 37.2 s vs 120.4 s (3.2x) vs 244.4 s
  (6.6x). Same five verdicts in all three columns; on Tribonacci they bracket the
  critical exponent in [3, 7/2), which is the known value 3.19148....
* **The palindrome ladder is where it does not.** `A n. E i. REV(i,i,n)` is a single
  query, and building the whole REV automaton to answer it is wasted work: Tribonacci
  0.8 s inline vs 88.0 s learned (110x slower), PRISM-1 0.1 s vs 37.5 s (375x slower).
  The `let` column is no better than the learned one (75.2 s / 41.0 s), so this is not a
  learner problem — it is that a *reusable* predicate is the wrong thing to build for one
  question. What learning buys here is memory, not time: 3 MB against 4891 MB for `let`
  on Tribonacci.

The ladder stops at exponent 4 because the *arithmetic* runs out of room before the
predicate does: `5*l >= 16*n` folds 5 copies of `l` and 16 copies of `n` through adder
chains and exhausts 6 GB on Tribonacci in all three variants, learned included
(measured: 7/2 finishes in 22.4 s, 10/3 dies at 9.1 s, 13/4 dies at 24.9 s). That is a
cost of the multiplication, not of the predicate, so including those rungs would measure
the wrong thing.

### 4.6 When not to use it

The running time of `learn` scales with the size of the *answer* (roughly one
counterexample per state, one equivalence query per 10-40 states), not with the
intermediate blowup — which is exactly why it wins where the direct method dies and
loses where the direct method is fine. Concretely, on this panel:

* Direct wins, usually by 10-100x, on everything small and on the whole `single`
  family: `single5` period 5.7 s direct vs 25.1 s learned, `single6` border 89.8 s vs
  524.5 s, `single6` rev 61.3 s vs 1495 s. `single6` period exceeds 6 GB in the learner
  while the direct construction finishes in 68.4 s. Large answers over a small alphabet
  are the learner's worst case: it pays one counterexample per state and the equivalence
  automata grow with the hypothesis.
* Learning wins where the direct construction is in trouble: `tail-a` fe 187.4 s -> 9.4 s,
  `tail-a` rev 202.4 s -> 8.1 s, `tail-b` rev 230.9 s -> 35.3 s, `prism-1` period
  139.8 s -> 37.9 s, Tribonacci period 120.3 s -> 1.0 s; and eight panel automata plus
  Tribonacci BORDER exist only because of it.
* A sensible driver is still the one `docs/LEARNFE.md` recommends: run the direct
  construction with a small cap first, fall back to `learn`.

## 5. Knobs

| env | default | meaning |
|---|---|---|
| `AM_LEARN_LCP` | 2²² | LCP/RLCE step cap per pair |
| `AM_LEARN_LCP_MAX` | 2²⁶ | ceiling for automatic escalation on a stall |
| `AM_LEARN_UNROLL` | 5000 | recursion depth cap for a user-supplied oracle |
| `AM_LEARN_SAMPLES` | 4000 | tuples drawn per boundary-sampling round |
| `AM_LEARN_PROBE` | 2000 | counterexamples harvested per local-probe crawl |
| `AM_LEARN_WITNESS` | 256 | rejecting states harvested per equivalence query |
| `AM_LEARN_DIGITS` | 22 / 14 / 11 / 9 for k=2/3/4/other | max digits of sampled values |
| `AM_LEARN_ITERS` | 20000 | iteration ceiling before giving up |
| `AM_LEARN_DEBUG` | unset | per-round trace on stderr |
| `AM_MEM_MB` | 2048 | unchanged: the counting allocator still bounds the process |

## 6. Files

* `engine/src/learn.rs` — specs, oracles, discrimination-tree learner, verifier, driver
* `engine/src/main.rs` — the `learn` command (4-line dispatch into `learn::cmd_learn`)
* `explore/learn_bench.py` — `panel` / `trib` / `fuzz` / `brute` / `ladder`
* `results/learn_panel.json`, `results/learn_numsys.json`, `results/learn_fuzz.json`,
  `results/learn_brute.json`, `results/learn_ladder.json`
* `docs/LEARNFE.md` — the FE-only predecessor, and the proof for the FE case in full
