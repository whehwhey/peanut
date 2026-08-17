# ATTACK TARGET 4 — the greedy 3-sumfree meta-conjecture

**Verdict: solved.** Conjecture 17 of Bosma–Bruin–Fokkink–Grube–Reuijl–Tromp
(*Using Walnut to solve problems from the OEIS*, J. Integer Seq. **28** (2025),
Article 25.3.8; = Conjecture 6 of arXiv:2503.04122v1) is **true**, and true on a
strictly larger range than it claims. It is proved here twice: by machine, as eight
Presburger sentences that quantify over all `(g,d)` at once (6.2 s total, peak
1.3 GB), and by hand, by an interval-sumset argument in the style Shtrezi used
for the `d = 1` companion conjecture.

The paper's stated blocker — "`z = k*(5g+2d)+w` needs `*` applied to two
variables, which Walnut does not allow" — is removed by a one-line change of
variables (§2). **This is a reformulation, not a prover capability.** The
rewritten sentences are ordinary Presburger arithmetic, and Walnut 8-dev proves
all eight of them too (52.7 s total against Peanut's 6.2 s; §6.4) — which is the
best possible independent check of the reformulation, and the honest place to put
the credit: the contribution here is the change of variables and the theorem it
unlocks, not an engine capability.

---

## 1. The problem, verbatim, and its status

`S_{x,y,z}` denotes the **greedy 3-sumfree sequence** with start values
`x < y < z`: the increasing sequence beginning `x, y, z` in which each later term
is the least integer exceeding the previous term that is **not** a sum of three
distinct earlier terms. `S_{1,2,3}` and `S_{1,3,4}` are OEIS A026471, A026475.

> **Conjecture 17.** Let `d >= 2`. For every `g >= d+1` the greedy 3-sumfree
> sequence `S_{1,g,g+d}` is characterized as follows:
>
> `z in S_{1,g,g+d}  <=>  z in {1, g, 2g+d-1, 2g+d}` or `z >= g+d` and
> `z mod 5g+2d in {g+d-2, g+d-1, ..., 2g+d-2}`.
>
> In particular, for `d >= 2` and every `g >= d+1` after the first `g+3` entries
> (in a preperiod) the sequence `S_{1,g,g+d}` modulo `5g+2d` is periodic with
> period `g+1`.

(The **arXiv v1** text has `z > g+d` where the published J. Integer Seq. text has
`z >= g+d`. The v1 form is a typo: with `z > g+d` the seed `g+d` itself is
excluded from the right-hand side, and `g+d = a_3` is in the sequence by
definition. The paper's own worked Walnut example for `(g,d) = (4,4)` writes
`z>=8`, i.e. `z >= g+d`. Everything below uses the published `>=` form.)

**Status re-verified 2026-08-17.**
* The paper is published: J. Integer Seq. 28 (2025), Article 25.3.8. arXiv v1
  (6 Mar 2025) is the only arXiv version. arXiv numbering -> JIS numbering:
  Theorem 4 -> 15, Conjecture 5 -> 16, Conjecture 6 -> **17**, Remarks 7 -> 18,
  Example 8 -> 19.
* The **`d = 1` companion** (Conjecture 16, `S_{1,g,g+1}`, modulus `10g+3`) was
  proved by **O. Shtrezi, *The greedy 3-sumfree sequence `S_{1,g,g+1}`*,
  arXiv:2606.17447 (13 Jun 2026)**, by a direct interval-sumset argument.
  Shtrezi's note does **not** mention `d >= 2` or the meta-conjecture at all,
  and the `d = 1` family is genuinely a separate case: Remarks 18 of the source
  paper says Conjecture 16 is *not* a special case of Conjecture 17, its modulus
  and (pre)period being irregular.
* No treatment of Conjecture 17 was found (web/arXiv, Aug 2026). It was open.

Evidence the authors had: individual Walnut runs for fixed `(g,d)` (they show
`(4,4)`), and "firm computational evidence, from an implementation in Magma".

---

## 2. The obstruction, and the change of variables that removes it

The verification of a fixed instance is trivial: with `g,d` numeric, `5g+2d` is a
constant and `z = k*(5g+2d)+w` is a legal Presburger term. The parametrized
statement needs `k*(5g+2d)` with `k`, `g`, `d` all variables — outside Presburger
as any automatic prover accepts it, and genuinely nonlinear.

**The fix: never name `z`.** Carry every integer as its (quotient, remainder)
pair against `m = 5g+2d`. Then the only place a product would appear is the
equation `a+b+c = z`, and there it can be eliminated:

> **Lemma 1 (carry split).** Let `m >= 1` and let `a = ka*m + wa`,
> `b = kb*m + wb`, `c = kc*m + wc`, `z = kz*m + wz` with
> `0 <= wa,wb,wc,wz < m`. Then
>
>     a + b + c = z   <=>   exists j in {0,1,2}:
>                            wa+wb+wc = wz + j*m   and   ka+kb+kc + j = kz.
>
> *Proof.* Put `j = kz - (ka+kb+kc)`; then `j*m = (wa+wb+wc) - wz`, and
> `wa+wb+wc-wz` lies in `[-(m-1), 3m-3]`, so `-1 < j < 3`, i.e. `j in {0,1,2}`.
> The converse is immediate. ∎

`j` is a *constant* in each of the three disjuncts, so `j*m = j*(5g+2d)` is a
linear term. Every remaining condition (membership, distinctness, the parameter
hypotheses, "`z > g+d`") is linear in `(g, d, k*, w*)`. The whole
uniform-in-`(g,d)` statement is therefore a single Presburger sentence in ten
variables, decidable by automaton compilation.

The trick is general: it removes `k*(linear form in parameters)` from any
"residue class modulo a parametrized modulus" statement about sums of a bounded
number of terms (`r` summands give `j in {0,...,r-1}`).

---

## 3. What is proved

Fix `d >= 2`, `g >= 2`, and write

    m = 5g+2d,   p = g+d-2,   q = 2g+d-2   (so q = p+g, and q+2 = 2g+d < m),

    G(g,d) = {1, g}  u  [g+d, 2g+d]  u  U_{k>=1} ( k*m + [p, q] ).

`G(g,d)` is exactly the right-hand side of Conjecture 17: the conjecture's
`{2g+d-1, 2g+d}` together with the `k = 0` part `[p+2, q] = [g+d, 2g+d-2]` of the
residue condition is the single interval `[g+d, 2g+d]`.

> **Theorem 1.** Let `d >= 2` and `g >= 2`. Then
>
>     G(g,d) = S_{1,g,g+d}   <=>   g >= d  and  2g >= d+3.
>
> The right-hand condition is equivalent to "`g >= d` and `(g,d) != (2,2)`".

Conjecture 17 assumes `g >= d+1`, which implies `g >= d` and `2g >= 2d+2 >= d+3`
(as `d >= 1`); so **Theorem 1 contains Conjecture 17** and adds the diagonal
`g = d` for every `d >= 3`, e.g. `S_{1,3,6}`, `S_{1,4,8}`, `S_{1,5,10}`, … . The
diagonal is not covered by the conjecture as stated and is not one of the
"irregular small-`g`" cases of Remarks 18. Amusingly, the paper's own worked
verification (Example 19) is the case `g = d = 4`, i.e. `S_{1,4,8}` — a point on
the diagonal, outside the `g >= d+1` hypothesis of the very conjecture it
illustrates. The regular description simply holds
one column further left than claimed.

Theorem 1 follows by strong induction from four statements, each proved in §4 and
machine-verified in §5:

* **Proposition A (maximality).** If `d >= 2`, `g >= d`, `2g >= d+3`, then every
  `z > g+d` with `z not in G` is a sum of three distinct elements of `G`.
* **Proposition B (3-sumfreeness).** If `d >= 2`, `g >= 2`, then no element of
  `G` is a sum of three distinct elements of `G`. (No `g >= d` needed.)
* **Proposition C (seeds).** If `d >= 2`, `g >= 2`, then
  `G n [1, g+d] = {1, g, g+d}`.
* **Proposition D (sharpness).** If `2 <= g < d` then `z = 3g+d+2` satisfies
  `z > g+d`, `z not in G`, and `z` is not a sum of three distinct elements of
  `G`; hence `G != S_{1,g,g+d}`. For `(g,d) = (2,2)` the same holds at `z = 14`.

*Induction.* By C the two sets agree on `[1, g+d]`. Let `z > g+d` and suppose
they agree below `z`. Every representation `z = a+b+c` with distinct
`a,b,c in G` has `a,b,c < z` (all elements are positive), so by the induction
hypothesis those `a,b,c` are exactly the earlier terms of `S`. If `z not in G`,
A gives such a representation, so the greedy rule rejects `z`; if `z in G`, B
says no such representation exists, so the greedy rule accepts `z`. ∎

**Corollary (period/preperiod, explicitly).** For `d >= 2`, `g >= d`,
`(g,d) != (2,2)`, the sequence begins with the `g+3` terms

    1,  g,  g+d, g+d+1, ..., 2g+d

and from there on repeats, modulo `m = 5g+2d`, the block of `g+1` residues

    g+d-2, g+d-1, ..., 2g+d-2,

one block per period, which is the paper's "after the first `g+3` entries …
periodic with period `g+1`".

---

## 4. The pen-and-paper proof

Notation as in §3. Write `B = [g+d, 2g+d]` (the block-0 part of `G`, `g+1`
integers) and `A_k = k*m + [p,q]` for `k >= 1`. Following Shtrezi, for a set `X`
put `2^X = {x+y : x,y in X, x<y}` and `3^X = {x+y+z : x,y,z in X, x<y<z}`, and
use `2^[u,v] = [2u+1, 2v-1]`, `3^[u,v] = [3u+3, 3v-3]` for nonempty results.
Here (`g >= 2`, so `|B| = g+1 >= 3`)

    2^B = [2g+2d+1, 4g+2d-1],      3^B = [3g+3d+3, 6g+3d-3].

The complement of `G` above the seeds is a union of **gaps**

    Gap_0 = [2g+d+1, 6g+3d-3],
    Gap_k = k*m + [2g+d-1, 6g+3d-3]      (k >= 1),

since block 0 of `G` ends at `2g+d`, `A_k` runs from `km+p` to `km+q` and
`6g+3d-3 = m + p - 1`, one below the start of the next block.

### 4.1 Proposition B: every 3-sum lands in a gap

`G` is partitioned into `{1}`, `{g}`, `B`, and the `A_k` (`k >= 1`); summands
from different parts are automatically distinct, and inside `B` or an `A_k` we
may ignore distinctness, which only enlarges the sumset. Writing `K` for the sum
of the block indices of the `A`-summands:

| triple type | sum lies in | inside |
|---|---|---|
| `1+g+B`       | `[2g+d+1, 3g+d+1]`               | `Gap_0` |
| `1+2^B`       | `[2g+2d+2, 4g+2d]`               | `Gap_0` |
| `g+2^B`       | `[3g+2d+1, 5g+2d-1]`             | `Gap_0` |
| `3^B`         | `[3g+3d+3, 6g+3d-3]`             | `Gap_0` |
| `1+g+A_k`     | `km + [2g+d-1, 3g+d-1]`          | `Gap_k` |
| `1+B+A_k`     | `km + [2g+2d-1, 4g+2d-1]`        | `Gap_k` |
| `g+B+A_k`     | `km + [3g+2d-2, 5g+2d-2]`        | `Gap_k` |
| `2^B+A_k`     | `km + [3g+3d-1, 6g+3d-3]`        | `Gap_k` |
| `1+A_i+A_j`   | `Km + [2p+1, 2q+1] = Km + [2g+2d-3, 4g+2d-3]` | `Gap_K` |
| `g+A_i+A_j`   | `Km + [2p+g, 2q+g] = Km + [3g+2d-4, 5g+2d-4]` | `Gap_K` |
| `B+A_i+A_j`   | `Km + [3g+3d-4, 6g+3d-4]`        | `Gap_K` |
| `A_i+A_j+A_l` | `Km + [3p, 3q] = Km + [3g+3d-6, 6g+3d-6]` | `Gap_K` |

Each containment is one inequality at each end. The only ones that are not
immediate from `g,d >= 2`:

* `1+A_i+A_j`: lower end needs `2g+2d-3 >= 2g+d-1`, i.e. **`d >= 2`** — this is
  the one place the hypothesis `d >= 2` is used, and it is why `d = 1` needs a
  different modulus and a different proof (Shtrezi's).
* `g+A_i+A_j`: lower end needs `g+d >= 3`.
* `A_i+A_j+A_l`: lower end needs `g+2d >= 5`.
* upper ends: `3g+2d >= 4`, `2g+d >= 3`, `g+d >= 2`, `2g+d >= 2`, `g+d >= 1`.

All hold for `g,d >= 2`. Sums with repeated blocks are covered: e.g.
`A_i + A_i` with distinct elements is `2im + [2p+1, 2q-1] ⊆ Km + [2p, 2q]`. So
every sum of three distinct elements of `G` lies in a gap, i.e. outside `G`. ∎

### 4.2 Proposition A: every gap value is a 3-sum

`Gap_0 = [2g+d+1, 6g+3d-3]` is covered by the four intervals

    1+g+B = [2g+d+1, 3g+d+1],   1+2^B = [2g+2d+2, 4g+2d],
    g+2^B = [3g+2d+1, 5g+2d-1], 3^B   = [3g+3d+3, 6g+3d-3],

whose successive junctions require

    2g+2d+2 <= 3g+d+2   i.e.  d <= g        (<- hypothesis g >= d)
    3g+2d+1 <= 4g+2d+1  i.e.  0 <= g
    3g+3d+3 <= 5g+2d    i.e.  d+3 <= 2g     (<- hypothesis 2g >= d+3)

and the two ends match `Gap_0` exactly.

`Gap_k = km + [2g+d-1, 6g+3d-3]` (`k >= 1`) is covered by

    1+g+A_k = km+[2g+d-1, 3g+d-1],   1+B+A_k = km+[2g+2d-1, 4g+2d-1],
    g+B+A_k = km+[3g+2d-2, 5g+2d-2], 2^B+A_k = km+[3g+3d-1, 6g+3d-3],

with junctions `2g+2d-1 <= 3g+d` (i.e. `d-1 <= g`), `3g+2d-2 <= 4g+2d`, and
`3g+3d-1 <= 5g+2d-1` (i.e. `d <= 2g`) — all implied by `g >= d >= 2`.

Distinctness in each identity: `1 != g` (as `g >= 2`); `1, g not in B` (as
`g < g+d`); the `2^`/`3^` operators enforce distinctness inside `B`; and every
element of `A_k` exceeds every element of block 0. ∎

### 4.3 Propositions C and D

**C.** `1 < g < g+d`; `2g+d-1, 2g+d > g+d`; and `[g+d, 2g+d] n [1,g+d] = {g+d}`.
So `G n [1,g+d] = {1, g, g+d}`, the three seeds. ∎

**D.** Let `2 <= g < d` and `z = 3g+d+2`. Then `z > 2g+d` (as `g+2 > 0`) and
`z < m = 5g+2d` (as `2 < 2g+d`), so `z` lies in `Gap_0` and `z not in G`. Any
representation `z = a+b+c` with `a,b,c in G` has all three summands `< z < m`,
i.e. in block 0 (`{1, g} u B`), so the sum lies in one of the four intervals of
§4.2. It misses `1+g+B = [2g+d+1, 3g+d+1]` (above), `g+2^B` and `3^B` (below,
using `d >= 2`), and it misses `1+2^B = [2g+2d+2, 4g+2d]` exactly when
`3g+d+2 < 2g+2d+2`, i.e. when `g < d`. Hence `z` is not a 3-sum, the greedy rule
would admit it, and `G != S`. For `(g,d) = (2,2)`: `m = 14`, `G n [1,14) = {1,2,4,5,6}`, whose 3-sums
are `{7,8,9,10,11,12,13,15}`; `14` is missing, and `14 not in G(2,2)`. ∎

Together with §4.1–4.2 this proves Theorem 1 in both directions.

---

## 5. The machine proof

Driver `explore/attack4_engine.py`; full transcript
`results/attack4_transcript.txt`; structured results `results/attack4_engine.json`.
Base 2, `mode msd`, dummy sequence `def T 2 2 0 01 10 01` (the engine wants a
current sequence; `T` never appears in any formula — everything here is pure
`<N,+,<>`).

**Membership, in `(k,w)` coordinates** (`ING(k,w)`: "`k*m+w in G`"):

    ( k=0 & (w=1 | w=g | w+1=2*g+d | w=2*g+d) )
    | ( g+d <= w+2 & w+2 <= 2*g+d & (k>=1 | w>=g+d) )

Every disjunct forces `w <= 2g+d < m`, so the encoding is injective on the
encoded set — checked as **T1** below, which is what makes Lemma 1 applicable
without an explicit `w < m` guard on the summands.

**Sum, by Lemma 1:**

    (wa+wb+wc = wz            & ka+kb+kc   = kz)
  | (wa+wb+wc = wz +  5*g+2*d & ka+kb+kc+1 = kz)
  | (wa+wb+wc = wz + 10*g+4*d & ka+kb+kc+2 = kz)

**Distinctness:** `(ka!=kb | wa!=wb) & (ka!=kc | wa!=wc) & (kb!=kc | wb!=wc)`.

### 5.1 The two hard sentences

Both are settled by `witness` on an **open** formula. `witness` breadth-first
searches the compiled product automaton for a shortest accepted word; `NONE`
means the language is empty, i.e. the conjunction is unsatisfiable, i.e. the
universally quantified negation is a theorem. For T2 that means **no existential
projection and no subset construction happen at all** — which is the whole reason
a ten-track formula is affordable there.

**T2 = Proposition B** (`d>=2 & g>=2`, ten free variables):

    witness d>=2 & g>=2 & ING(ka,wa) & ING(kb,wb) & ING(kc,wc) & ING(kz,wz)
            & DISTINCT & SUM

    NONE vars=[d,g,ka,kb,kc,kz,wa,wb,wc,wz] states=1 ms=4174
    OK mem live=0MB peak=1289MB

(`states=1` is the minimised product automaton: a single non-accepting state, so
the language really is empty, not merely unexplored.)

**T3 = Proposition A** (`d>=2 & g>=d & 2*g>=d+3`). Here an existential *is*
needed — "there exist three distinct elements of `G` summing to `z`" — and the
unrestricted version is the one thing in this attack that does not fit: `let EX`
over the six variables `ka,wa,kb,wb,kc,wc` was killed by the system guard
(`explore/memguard.sh`, 6 GB RSS) after 286 s.
The fix is to restrict the witness family to the shapes §4.2 actually produces —
two summands in block 0 and the third in block 0 or in block `kz-j` — which
leaves three existential variables:

    let EX(g,d,kz,wz) E wa,wb,wc.  ...   ->  states=377, peak 150MB, 0.96s
    witness d>=2 & g>=d & 2*g>=d+3 & wz+1 <= 5*g+2*d & (kz>=1 | wz>=g+d+1)
            & ~ING(kz,wz) & ~$EX(g,d,kz,wz)

    NONE vars=[d,g,kz,wz] states=1 ms=13

Restricting the existential is sound in this direction: `EX'` implies `EX`, so
"every non-member is hit by `EX'`" is stronger than Proposition A. (`EX'` is also
checked against real sumsets by independent code, §6.2.)

**T4** re-runs T3 under Conjecture 17's own hypothesis `d>=2 & g>=d+1`: `NONE`.

### 5.2 The rest

| check | statement | result | time |
|---|---|---|---|
| T0 | Proposition C: `G n [1,g+d] = {1,g,g+d}` for `d>=2, g>=2` | `NONE` | 0.01 s |
| T1 | no `(k,w)` with `w >= 5g+2d` satisfies `ING` (encoding injective) | `NONE` | 0.01 s |
| T2 | Proposition B, `d>=2, g>=2` | `NONE` | 4.21 s |
| T3 | Proposition A, `d>=2, g>=d, 2g>=d+3` | `NONE` | 0.98 s |
| T4 | Proposition A under Conjecture 17's `g>=d+1` | `NONE` | 0.96 s |
| T5 | sharpness: for `2<=g<d`, `3g+d+2` is `> g+d` and outside `G` | `NONE` | 0.01 s |
| T6 | sharpness: for `2<=g<d`, `3g+d+2` is not a 3-sum in `G` | `NONE` | 0.01 s |
| T7 | sharpness at `(2,2)`: `14 not in G`, `14` not a 3-sum | `FALSE`,`NONE` | 0.01 s |

Total 6.2 s, peak 1.3 GB, for a statement quantified over all `(g,d)` in range
and all `z`.

---

## 6. Verification

### 6.1 Brute force, sharing nothing with the engine

`explore/attack4_brute.py` builds `S_{x,y,z}` from the definition with Python
big-integer bitsets: maintain the member set `S`, the set `P` of sums of two
distinct members and the set `T` of sums of three distinct members, and on
admitting `v` do `T |= P<<v; P |= S<<v; S |= 1<<v`; a candidate `z` is admitted
iff bit `z` of `T` is clear. No automata, no Presburger, no shared code path.

    d = 2..30, g = d+1..200  (5336 pairs, 40 periods of 5g+2d each): 0 failures

### 6.2 The trichotomy, the encoding, and the paper proof

`explore/attack4_verify.py` checks, on a grid of `(g,d)` including `g < d`:

1. Propositions A and B directly against the true 3-sum set of `G` — and that
   they hold **exactly** when `d>=2 & g>=d & 2g>=d+3`, i.e. Theorem 1's
   "if and only if" (not just the "if").
2. Faithfulness of the `(k,w)` encoding: `inG_kw(g,d,z div m, z mod m)` equals
   `z in G` for every `z <= N`, and no `(k,w)` with `w >= m` satisfies it.
3. Lemma 1 exhaustively: for every encoded triple with `k < 3`, exactly one
   `j in {0,1,2}` satisfies the split, and it reproduces `a+b+c`.
4. Soundness and completeness of the restricted witness family `EX'` of §5.1
   against the real sumsets.
5. The twelve interval containments of §4.1 and the two covering chains of §4.2,
   as exact integer arithmetic, at every grid point.

    d = 2..14, g = 2..40, 25 periods: 0 failures

The resulting map (`results/attack4_range.txt`) is a clean staircase — `Y` where
the conjecture's description is the actual sequence, `.` where it is not:

         g:234567890123456789012345678901234567890
    d=  2: .YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d=  3: .YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d=  4: ..YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d=  5: ...YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d=  6: ....YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d=  7: .....YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d=  8: ......YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d=  9: .......YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d= 10: ........YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d= 11: .........YYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d= 12: ..........YYYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d= 13: ...........YYYYYYYYYYYYYYYYYYYYYYYYYYYY
    d= 14: ............YYYYYYYYYYYYYYYYYYYYYYYYYYY

The first `Y` of row `d` is at `g = d` for `d >= 3` and at `g = 3` for `d = 2` —
Theorem 1's `g >= d & 2g >= d+3`, exactly. Conjecture 17 claims only the columns
strictly right of the diagonal. Across the whole grid, including every failing
cell, `G` is 3-sumfree (Proposition B never fails): when the description is wrong
it is always because `G` is too small, never because it collides with itself.

### 6.3 The per-instance route, in Peanut and in Walnut

The paper's own method (fix `g,d`, then one query) was transcribed for both
provers, verbatim from Example 19.

**Walnut 8-dev** (`-Xmx4g`, `explore/attack4_walnut.py`) settles every instance
tried, and reproduces the paper's `(4,4)` TRUE in 0.3 s:

    (g,d)    (4,4) (3,2) (5,2) (7,2) (9,2) (11,2) (5,5) (7,6) (9,6) (11,6) (12,5) (12,6)
    m           28    19    29    39    49     59    35    47    57     67     70     72
    verdict   TRUE  TRUE  TRUE  TRUE  TRUE   TRUE  TRUE  TRUE  TRUE   TRUE   TRUE   TRUE
    seconds    0.3   0.6   1.3   3.0   5.9   10.2   2.1   5.0   9.6   15.8    2.3    0.4

**Peanut**, four encodings per instance (naive/staged x msd/lsd), 3 GB budget,
`(g,d)` with `2<=d<=6`, `d<=g<=12`, `2g>=d+3`:

    27 / 44 instances TRUE;  17 exhausted 3 GB in all four encodings

and the 17 failures are, with one exception (`(12,5)`), exactly the instances
with `g` odd, i.e. `m = 5g+2d` odd. Walnut solves those same instances in 3-16 s.
**So this is a Peanut weakness, recorded as such, not a fact about the problem:**
on this predicate shape (a triple sum over a residue class with odd modulus) the
adaptive ladder loses to Walnut's plain forward construction. It does not touch
the result — the uniform sentences of §5 are what proves Theorem 1, and the
independent check of every instance is §6.1's brute force, not either prover.

What the per-instance route cannot do, in either prover, is settle a statement
about infinitely many `(g,d)`. That is the whole content of the source paper's
blocker, and it is what §2 removes.

### 6.4 Second prover on the uniform sentences

The eight sentences of §5 were re-run in Walnut 8-dev
(`explore/attack4_walnut.py --uniform`, `-Xmx6g`), written the way Walnut wants
them — `eval name "A<vars> ~(body)"`, i.e. as universally quantified negations
rather than as `witness` on an open formula, so Walnut *does* run the ten-fold
projection Peanut avoids:

    check                Walnut     Peanut
    T0-seeds             TRUE  0.2s  NONE 0.01s
    T1-encoding          TRUE  0.2s  NONE 0.01s
    T2-P3-sumfree        TRUE 46.4s  NONE 4.21s
    T3-P2-maximal        TRUE  2.6s  NONE 0.98s
    T4-P2-conj17         TRUE  2.7s  NONE 0.96s
    T5-sharp-nonmember   TRUE  0.2s  NONE 0.01s
    T6-sharp-unforced    TRUE  0.2s  NONE 0.01s
    T7-sharp-22          TRUE  0.2s  NONE 0.01s
    total                     52.7s        6.2s

Two independent implementations of first-order arithmetic, with different
determinization strategies and different formula front ends, agree on every one.
Walnut needs 46 s for the ten-variable Proposition B where Peanut's
projection-free `witness` needs 4.2 s, but it gets there — so nothing in this
attack depends on Peanut being used.

---

## 7. Ledger

**Known before (cited, not claimed):**
* Conjecture 17 itself, its `(4,4)` Walnut verification, the Magma evidence, and
  Remarks 18 (small `g` is irregular) — Bosma–Bruin–Fokkink–Grube–Reuijl–Tromp,
  J. Integer Seq. 28 (2025) 25.3.8.
* The `d = 1` companion (Conjecture 16) and the interval-sumset proof technique —
  Shtrezi, arXiv:2606.17447 (Jun 2026). §4 is that technique applied to `d >= 2`;
  the identities are different but the shape of the argument is his.
* `2^[u,v] = [2u+1,2v-1]`, `3^[u,v] = [3u+3,3v-3]`.

**New here:**
* **Lemma 1 (carry split)** — the change of variables that turns a parametrized
  modulus statement about `r`-fold sums into Presburger, removing the blocker the
  source paper stops at. *Proved; exhaustively machine-checked.*
* **Theorem 1** — `G(g,d) = S_{1,g,g+d}` **iff** `d>=2, g>=d, 2g>=d+3`. This
  **proves Conjecture 17** and extends it to the diagonal `g = d` (`d >= 3`),
  which the conjecture excludes. *Proved by hand (§4) and by machine (§5),
  independently re-proved in Walnut (§6.4); the "only if" half (Proposition D) is
  new and is what pins the range exactly.*
* **Propositions A–D** with their exact hypotheses: B needs only `d,g >= 2`
  (`G` is always 3-sumfree — the conjecture fails for `g < d` purely by
  *maximality*, never by a collision), while A needs `g >= d` and `2g >= d+3`,
  and those two inequalities are exactly the two junctions of the covering chain
  for `Gap_0`. *Proved.*
* Explicit preperiod/period (Corollary in §3), which the source states only as a
  count.
* The observation that arXiv v1's `z > g+d` is a typo for the published
  `z >= g+d` (the seed `g+d` would otherwise be excluded).

**Not established / open:**
* The **irregular cases**: `2 <= g < d` and `(2,2)`. Theorem 1 says the *regular*
  description fails there; it says nothing about what the right description is.
  Remarks 18's Example 19 (`S_{1,5,12}`: modulus 321, period 32) is one such
  case; a general formula for `g < d` is not attempted here.
* Start values other than `1` (`S_{x,y,z}` with `x > 1`) — untouched.
* Whether the unrestricted six-variable existential (§5.1) can be projected
  within budget by a better ladder; we sidestepped it rather than solved it.

**Failed / discarded on the way:**
* `let EX(g,d,kz,wz) E ka,wa,kb,wb,kc,wc. ...` — the honest, unrestricted form of
  Proposition A. Killed by the system guard (`explore/memguard.sh`, 6 GB RSS) at
  286 s (ten-track automaton, six projections).
  Replaced by the restricted witness family, which is sound for the direction
  needed. This is the one place the attack works around the engine rather than
  through it.
* `lsd` for the ten-variable Proposition B: **worse**, not better — exceeded a
  4 GB budget in 5 s where `msd` finished in 4.4 s at 1.4 GB. (The usual
  `docs/TARGET1.md` Finding-1 asymmetry is about `exists`; with no projection
  anywhere, the lsd advantage disappears and the product automaton is simply
  bigger.)
* The naive per-instance encoding at larger moduli (§6.3): 17/44 instances over
  3 GB in Peanut. Staging the triple sum (`a+b`, then `+c`) rescued several but
  not the odd-modulus cases. Walnut 8-dev does all of them in 0.3-15.8 s, so this
  is a Peanut ladder weakness on this predicate shape and is logged as one.

---

## 8. Files

    explore/attack4_brute.py    greedy 3-sumfree generator + Conjecture 17 check (bitsets)
    explore/attack4_verify.py   trichotomy, (k,w) encoding, Lemma 1, EX', the §4 identities
    explore/attack4_engine.py   the eight uniform Peanut sentences + per-instance sweep
    explore/attack4_walnut.py   second prover: the paper's per-instance query, and
                                (--uniform) the eight sentences of section 5, in Walnut 8-dev
    results/attack4_engine.json      structured results for T0-T7 and the sweep
    results/attack4_transcript.txt   full machine transcript (scripts + engine output)
    results/attack4_walnut.json          Walnut per-instance comparison
    results/attack4_walnut_uniform.json  Walnut on the eight uniform sentences
