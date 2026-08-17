# Referee verdict on `docs/ATTACK-4.md`

Adversarial read, 2026-08-17, to the same standard as `paper/proof-verdict.md` and
`paper/attack2-verdict.md`. Every load-bearing step was re-derived by hand; every finite
claim was re-checked by brute force **written from scratch for this review**
(`paper/verdict-attack4/`), never through the author's `explore/attack4_*.py`; the primary
source was re-read from the **published LaTeX source** of J. Integer Seq. 28 (2025)
25.3.8 (`cs.uwaterloo.ca/journals/JIS/VOL28/Fokkink/fokkink9.tex`) and the arXiv v1 PDF,
not quoted from the document; the prior art (Shtrezi) was read in full; and the eight
uniform sentences were re-posed in a **different formula shape**, as closed universally
quantified sentences rather than `witness` on an open formula, and re-run in both Peanut
and Walnut 8-dev.

## Bottom line — the question the task asks

> **Yes. Conjecture 17 of Bosma–Bruin–Fokkink–Grube–Reuijl–Tromp is genuinely proved,
> and the strengthening is genuine too.** The pen-and-paper argument of §4 is complete
> and correct — I re-derived all twelve interval containments, both covering chains, both
> sharpness cases and the induction, and found no gap. Theorem 1's "if and only if"
> reproduces exactly on 5771 `(g,d)` cells of my own brute force, with Proposition D's
> predicted first failure point `z = 3g+d+2` hit on the nose in every failing cell.
> The machine proof is real and reproduces, and it is not load-bearing: §4 alone proves
> the theorem.

Qualifications, none of them fatal:

* **The mathematics is a competent transposition, not a new technique.** §4 is Shtrezi's
  interval-sumset argument (arXiv:2606.17447, Jun 2026) run on a different modulus. The
  document says so in its ledger. What is genuinely new is (a) that it is *done* for
  `d >= 2`, which nobody had done, and (b) the **exact range** — the "only if" half
  (Proposition D) is the part with no counterpart in Shtrezi and it is what turns a
  conjecture into a sharp theorem.
* **Lemma 1 ("carry split") is correct and is the key to the machine route, but it is
  schoolbook.** It is the ordinary quotient–remainder/carry argument. As a *contribution*
  it is the observation that this removes the source paper's stated blocker, not a lemma.
  The document's own framing ("This is a reformulation, not a prover capability") is the
  honest one; the ledger's bare "**New here**" oversells it slightly.
* Three defects, one of them a factual error in a measurement claim (§6.3). See §7.

## Verdict per claim

| claim | verdict |
|---|---|
| **the headline: "Conjecture 17 is true, and true on a strictly larger range"** | **PROVED** |
| Conjecture 17 is quoted correctly (published `z >= g+d` form) and numbered correctly | **CONFIRMED** against the JIS LaTeX source |
| arXiv v1's `z > g+d` is a typo for the published `z >= g+d` | **CONFIRMED** (v1 text checked; the seed `g+d = a_3` would be excluded) |
| Conjecture 17 was open as of Aug 2026 | **CONFIRMED** (3 citing papers total; Shtrezi is `d = 1` only; OEIS A026471/A026475 updated Jun 2026 with no `d >= 2` result) |
| **Lemma 1 (carry split)** | **PROVED** (re-derived) + **MACHINE-VERIFIED** (exhaustive, 377 `(g,d)` cells) — but folklore-grade, see above |
| §4.1 twelve interval containments (Proposition B) | **PROVED** — all 24 endpoint inequalities re-derived; every 3-subset of `G` over four blocks checked in 377 cells |
| §4.2 two covering chains (Proposition A) | **PROVED** — junction conditions `d <= g`, `d+3 <= 2g` are exactly right and exactly binding |
| **Proposition B** (`G` is 3-sumfree for all `d,g >= 2`) | **PROVED** + **MACHINE-VERIFIED** (0 violations in 377 cells, `g < d` included) |
| **Proposition A** (`g >= d`, `2g >= d+3`) | **PROVED** + **MACHINE-VERIFIED** — holds *exactly* on the stated hypothesis |
| **Proposition C** (seeds) | **PROVED** |
| **Proposition D** (sharpness, `2 <= g < d`, and `(2,2)`) | **PROVED**, modulo one unstated step (§7.3) + **MACHINE-VERIFIED**: `3g+d+2` is the true first deviation in every failing cell |
| **Theorem 1** — `G(g,d) = S_{1,g,g+d}` **iff** `d>=2, g>=d, 2g>=d+3` | **PROVED** (hand) + **MACHINE-VERIFIED** (5771 cells, `d<=30`, `g<=200`, 40 periods; plus spot checks to `g=d=100`) |
| **Theorem 1 contains Conjecture 17 and extends it to `g = d`, `d >= 3`** | **CONFIRMED** — the diagonal is genuinely outside the conjecture's `g >= d+1` |
| Corollary (explicit preperiod `g+3`, period `g+1`, exact block) | **CONFIRMED** by direct generation |
| the eight uniform sentences T0–T7 | **MACHINE-VERIFIED** — reproduced in Peanut in a *different* formula shape, and in Walnut 8-dev |
| §6.4 "Walnut proves all eight too" | **CONFIRMED** — my own transcription, all TRUE, ten-variable Prop B in 41.8 s |
| §6.3 "the 17 per-instance failures are, with one exception, exactly the odd-`g` instances" | **WRONG** — 5 odd-`g` instances succeed (§7.1) |
| §5.1 "no projection … is the whole reason a ten-track formula is affordable" | **OVERSTATED** — the projected form costs the same (§7.2) |
| the unrestricted six-variable existential is out of reach | **CONFIRMED** (§6) |

---

## 1. The primary source

I did not take the quotation on trust. The published paper is
**W. Bosma, R. Bruin, R. Fokkink, J. Grube, A. Reuijl, T. Tromp, *Using Walnut to Solve
Problems from the OEIS*, J. Integer Seq. 28 (2025), Article 25.3.8**; the LaTeX source is
served at `cs.uwaterloo.ca/journals/JIS/VOL28/Fokkink/fokkink9.tex`. Verbatim from it:

> **Conjecture 17.** Let `d>=2`. For every `g>=d+1` the greedy 3-sumfree sequence
> `S_{1,g,g+d}` is characterized as follows:
> `z in S_{1,g,g+d} <=> z in {1,g,2g+d-1,2g+d}` or `z >= g+d` and
> `z mod 5g+2d in {g+d-2, g+d-1, ..., 2g+d-2}`.
> In particular, for `d>=2` and every `g>=d+1` after the first `g+3` entries (in a
> preperiod) the sequence `S_{1,g,g+d}` modulo `5g+2d` is periodic with period `g+1`.

**Numbering.** All theorem-like environments share one counter. Counting them in order in
the source: 9 lemmas (1–9), Theorem 10 (Hajdu), Theorem 11 (Zaslavsky), Theorem 12,
Conjecture 13 (Clergyman), Conjecture 14 (`conj123`), Theorem 15, Conjecture 16
(`conj_g`, the `d=1` family), **Conjecture 17** (`conj_d_g`), Remarks 18, Example 19.
`ATTACK-4.md`'s numbering and its arXiv→JIS map are exactly right.

**The `>=` vs `>` point is real.** arXiv:2503.04122v1 (the only arXiv version) prints
`z > g + d` (and, incidentally, "3-subsumfree"). With `>` the value `g+d` is excluded from
the right-hand side, and `g+d = a_3` is a seed of the sequence by definition, so the v1
form is false for every `(g,d)`; the published `>=` form is the intended one, and the
paper's own `(4,4)` Walnut fragment writes `z>=8`. `ATTACK-4.md` §1 has this right.

**The stated blocker is quoted correctly.** The source's own words, for the `d = 1`
companion: `def mod "Ek,w ... & z=k*(10*g+3)+w"` is rejected with
"the operator * cannot be applied to two variables", and "Whenever a numerical value is
substituted for `g` … all is fine." So the obstruction the document removes is the one the
authors actually report.

**Also confirmed from the source, and correctly used by the document:** Remarks 18 says
Conjecture 16 is *not* a special case of Conjecture 17; that "most, but not *all*" greedy
3-sumfree sequences are covered, "usually the cases of small values for `g` for given `d`
are special"; and Example 19 is `d=7, g=5` (`S_{1,5,12}`, modulus 321, period 32).
§3 of the document mislabels the `(4,4)` fragment as "Example 19" (see §7.4); the
substance — the paper illustrates its own conjecture with a point on the diagonal, outside
its `g >= d+1` hypothesis — is correct and worth keeping.

## 2. Was it already solved?

No.

* Semantic Scholar lists **three** papers citing arXiv:2503.04122: Shtrezi
  (arXiv:2606.17447), Fokkink–Joshi *Anti-recurrence sequences* (arXiv:2506.13337, a
  different conjecture in the same paper), and Nicol–Frohme *Deconstructing Subset
  Construction* (a Walnut engineering paper). None touches Conjecture 17.
* **Shtrezi, read in full.** His Theorem 1 is exactly Conjecture 16 (`d = 1`, modulus
  `10g+3`, preperiod `g+4`, period `2g+1`). The strings "17", "meta", "`g+d`" do not occur
  in the paper. He never mentions `d >= 2`. His §2 fixes `2^X`, `3^X`,
  `2^[u,v] = [2u+1,2v-1]`, `3^[u,v] = [3u+3,3v-3]`, partitions `A_g` into
  `{1},{e},{f},I_k,J_k`, and closes with the same two-proposition induction. So
  `ATTACK-4.md`'s attribution — "§4 is that technique applied to `d >= 2`; the identities
  are different but the shape of the argument is his" — is exactly accurate.
* OEIS **A026471** and **A026475** were both edited 18 Jun 2026 to add the Shtrezi
  reference. Neither carries any `d >= 2` result. A026471's `%F` line credits Matthew
  Akeran with the `g = 2` case, as both the source paper and Shtrezi say.
* No later Shtrezi or Bosma/Fokkink follow-up on `d >= 2` was found (web, Aug 2026). The
  source paper's closing remark that pen-and-paper proofs are "some of which we hope to
  present elsewhere" has, as of now, produced only the `d = 1` note by a third party.

**Conjecture 17 was open.**

## 3. Lemma 1 — re-derived, correct

Let `m >= 1`, `a = ka*m+wa`, …, `z = kz*m+wz`, all `w` in `[0,m)`. If `a+b+c = z`, put
`j = kz-(ka+kb+kc)`; then `j*m = (wa+wb+wc)-wz in [-(m-1), 3m-3]`, so `-1 < j < 3`. The
converse is immediate, and the `j` is unique because `j*m` determines `j`. **PROVED.**

Machine check (mine): for every encoded triple from the first 26 elements of `G` in each of
377 `(g,d)` cells, **exactly one** `j in {0,1,2}` satisfies the split and it reproduces
`a+b+c` — 0 failures.

Two side conditions the lemma silently needs, and which the document does check:
`w < m` for all four values (the document's **T1**: every disjunct of `ING` forces
`w <= 2g+d < 5g+2d`; I re-verified T1 independently and by exhaustion), and — for T3, where
`z` is a *non*-member and so is not covered by T1 — the explicit guard
`wz+1 <= 5*g+2*d`, which is present.

**Novelty.** This is the schoolbook carry argument for `r`-fold sums. I could not find it
stated as a named lemma anywhere, and the source paper plainly did not think of it, so as a
*move* it is new to this problem; as mathematics it is folklore. Verdict: **PROVED**, novelty
low, and the document's headline framing (credit to the change of variables, not the engine)
is the right one.

## 4. The pen-and-paper proof — re-derived, correct

Notation as in §3/§4 of the document: `m = 5g+2d`, `p = g+d-2`, `q = 2g+d-2`,
`B = [g+d, 2g+d]`, `A_k = km+[p,q]`, `G = {1,g} u B u U_{k>=1} A_k`.

**The two readings of `G` agree.** The document's `G` (§3) and the published right-hand
side of Conjecture 17 define the same set: the `k=0` part of the residue condition, cut by
`z >= g+d`, is `[g+d, 2g+d-2]`, and adjoining the listed `{2g+d-1, 2g+d}` gives the single
interval `[g+d, 2g+d]`. I checked this by direct enumeration for `d = 2..8`, `g = 2..19`
over six periods — identical sets, 0 differences (`greedy.py`).

**Gaps.** `G` is disjoint from `Gap_0 = [2g+d+1, 6g+3d-3]` and
`Gap_k = km+[2g+d-1, 6g+3d-3]`, and above `g+d` the complement of `G` is exactly their
union. Verified: `6g+3d-3 = m+p-1`, one below the start of the next block.

**§4.1 (Proposition B).** The twelve rows are the twelve multisets of parts
`{1},{g},B,A`, so the case analysis is complete. I re-derived every one; the sumset
endpoints are correct (`1+A_i+A_j` is `Km+[2p+1, 2q+1]`, the `+1`/`+1` coming from the
`i != j` case, and `i = j` sits strictly inside). Each containment is two inequalities; the
non-trivial lower ends are exactly the four the document flags —
`d >= 2` (for `1+A_i+A_j`), `g+d >= 3`, `g+2d >= 5`, and `g+2d >= 3` (immediate) — and the
upper ends are the five it lists. All hold for `g,d >= 2`.

The document's claim that **`d >= 2` is the single place the hypothesis is used, and is why
`d = 1` needs a different modulus**, is not just plausible — it is exactly what happens. I
ran the `d = 1` case with the meta-conjecture's own modulus `5g+2`: for `g = 2..11` the
description fails **and Proposition B fails with it** (`G` is not 3-sumfree). For `g = 2`
the first collision is `39 = 3m+3`, which lies in both `A_3` and `1+A_1+A_2` — precisely the
row whose lower-end inequality needs `d >= 2`.

**§4.2 (Proposition A).** `Gap_0` is covered by `1+g+B`, `1+2^B`, `g+2^B`, `3^B` iff the
three junctions `d <= g`, `0 <= g`, `d+3 <= 2g` hold, and the two ends match exactly.
`Gap_k` is covered by `1+g+A_k`, `1+B+A_k`, `g+B+A_k`, `2^B+A_k` under the weaker
`d-1 <= g` and `d <= 2g`. I confirmed the four `Gap_0` intervals really are the sumsets
they are named after (exact endpoints, and each really is an interval, not just contained in
one), and likewise for `Gap_k`, in every cell; and that `chain0` holds **iff**
`g >= d & 2g >= d+3` and `chain1` holds **iff** `d <= g+1`. So the two hypotheses of
Theorem 1 are exactly the two junctions of the `Gap_0` chain — the document's claim, and it
is true on the nose.

Distinctness inside each covering identity is fine: `1 != g` (as `g >= 2`), `1,g not in B`
(as `g < g+d`), `2^`/`3^` enforce distinctness inside `B`, and `A_k` lies above block 0.

**§4.3 and the induction.** Proposition C is immediate. Proposition D: for `2 <= g < d`,
`z = 3g+d+2` satisfies `2g+d < z < m`, so `z in Gap_0`, and it misses all four covering
intervals — above `1+g+B`, below `g+2^B` and `3^B` (using `d >= 2`), and below `1+2^B`
exactly when `g < d`. Correct. The `(2,2)` computation is correct
(`G n [1,14) = {1,2,4,5,6}`, 3-sums `{7,...,13,15}`, `14` missing, `14 not in G(2,2)`);
I reproduced it. The induction is the standard one and is stated correctly: every
representation of `z` has all summands `< z`, so the induction hypothesis converts
"3-sum in `G`" into "3-sum of earlier terms of `S`".

**Completeness of the trichotomy.** For `d >= 2, g >= 2`: `g < d` → D; `g >= d` and
`2g < d+3` → forces `(g,d) = (2,2)` → D'; otherwise A+B+C. Exhaustive, and
`g >= d & 2g >= d+3` really is equivalent to `g >= d & (g,d) != (2,2)`. **Theorem 1 is
proved by §4 alone.**

## 5. Independent verification of Theorem 1

Everything below is my own code (`paper/verdict-attack4/`), sharing nothing with
`explore/attack4_*.py`.

**Generators.** Three mutually independent implementations of `S_{x,y,z}` — explicit
`O(n^3)` triple enumeration, a pair-scan with membership lookup, and incremental big-int
bitsets — agree on nine start triples. The bitset generator reproduces the paper's own
**proved** Theorem 15 closed forms for `S_{1,2,3}`, `S_{1,3,4}`, `S_{1,4,5}` exactly, and
the first 59 terms of OEIS A026471 and A026475 as published.

| check | scope | result |
|---|---|---|
| Theorem 1's **iff** vs. real greedy sequences | `d = 2..30`, `g = 2..200`, 40 periods of `5g+2d` — **5771 cells** | **0 mismatches**; staircase identical to `results/attack4_range.txt` |
| Proposition D's first-failure prediction `3g+d+2` (and `14` at `(2,2)`) | every failing cell above | **exact in all of them** |
| adversarial spot checks | `(100,100) (101,100) (100,99) (60,60) (59,60) (200,3) (3,200) (52,50) (50,52) (51,50) (50,51)` | **0 mismatches** |
| Propositions A, B, C; Lemma 1; `(k,w)` encoding + injectivity; the 12 containments on *every* 3-subset of `G` over four blocks; both covering chains; EX' soundness **and** completeness | `d = 2..14`, `g = 2..30` — **377 cells** | **0 failures** |
| Proposition B in the *failing* region `g < d` | same grid | **never fails** — confirms "too small, never a collision" |
| corollary: first `g+3` terms and the first repeating block | `(4,4) (3,2) (5,3) (3,3)` | **exact** |
| `d = 1` control (modulus `5g+2`) | `g = 2..11` | description fails **and** Prop B fails — the `d>=2` inequality is load-bearing |

## 6. The machine proof, re-posed

I rewrote the membership predicate in a different (equivalent) shape and posed the checks as
**closed universally quantified sentences** (`? A ... => ...`), i.e. the route that does run
the projection the document's `witness` form avoids. Also, for Proposition A I used a
**less** restricted existential than the document's `EX'`: two summands in block 0 and the
third's block index `kc` left free (four existentials, not three).

| my check | Peanut | Walnut 8-dev |
|---|---|---|
| encoding injective (T1) | TRUE 0.06 s | TRUE 0.2 s |
| seeds / Prop C (T0) | TRUE 0.03 s | TRUE 0.2 s |
| Prop B, **closed** 10-variable sentence | **TRUE 3.50 s, peak 1252 MB** | **TRUE 41.8 s** |
| Prop B, `witness` on the open formula | NONE 3.46 s, peak 1289 MB | — |
| Prop A, `kc` free (`d>=2, g>=d, 2g>=d+3`) | TRUE 1.96 s, peak 195 MB | TRUE 1.8 s |
| Prop A under Conjecture 17's `g >= d+1` | TRUE 1.98 s | TRUE 1.8 s |
| Prop D non-membership / not-a-3-sum | TRUE 0.01/0.02 s | TRUE 0.2/0.3 s |
| `(2,2)` sharpness | TRUE 0.01 s | TRUE 0.3 s |
| **the paper's own `(4,4)` Walnut fragment**, transcribed verbatim from the JIS LaTeX | — | **TRUE 0.3 s** |

Two independent provers, two different formula shapes, two different quantifier
presentations: everything agrees. The document's §6.4 Walnut timings reproduce
(their 46.4 s vs my 41.8 s for the ten-variable Prop B).

**The one open item is confirmed open.** `let EXF(g,d,kz,wz) E ka,wa,kb,wb,kc,wc. …` —
the honest unrestricted six-variable existential for Proposition A — is out of reach here
too. My run (`AM_MEM_MB=3072`, msd) was killed by the runner watchdog at **4874 MB RSS
after 274.5 s**, still inside the `let EXF` — the same failure mode, at the same order of
time, that the document reports (killed by `explore/memguard.sh` at 286 s). Independently
confirmed. The document's ledger already records this as
"the one place the attack works around the engine rather than through it", which is the
correct characterisation: the *restriction* is sound (`EX' => EX`, so "every non-member is
hit by `EX'`" is strictly stronger than Proposition A) and I verified `EX'` soundness by
reconstruction and its completeness against the true sumsets in all 377 cells.

## 7. Defects found

### 7.1 §6.3's odd-`g` claim is factually wrong

> "the 17 failures are, with one exception (`(12,5)`), exactly the instances with `g` odd"

From the author's own `results/attack4_engine.json`: the sweep has 44 instances, 21 of them
with odd `g`, and **5 of those 21 succeed** — `(3,2) m=19`, `(3,3) m=21`, `(5,2) m=29`,
`(5,3) m=31`, `(5,4) m=33`. So the failures are a strict subset of the odd-`g` instances,
not equal to them. The correct statement, which the data supports cleanly, is by
**modulus size**: every instance with even `m` succeeds except `(12,5)` (`m = 70`), every
instance with odd `m <= 33` succeeds, and every instance with odd `m >= 35` fails. Since
`m = 5g+2d` is odd iff `g` is odd, "odd `g`" is the right *shape* but the wrong
*quantifier*. Nothing downstream depends on it — §6.3 is explicitly a note about a Peanut
weakness, not part of the proof.

### 7.2 §5.1's explanation of why T2 is affordable is not supported

> "For T2 that means **no existential projection and no subset construction happen at all**
> — which is the whole reason a ten-track formula is affordable there."

The projection-free reading is right (a purely Boolean combination of Presburger atoms is
built by products and complements only). The *causal* claim is not: I posed the same
statement as `? A g,d,ka,…,wz. (d>=2 & g>=2) => ~( … )`, which does force the ten-fold
projection, and Peanut returns **TRUE in 3.86 s at 1252 MB** against **3.78 s at 1289 MB**
for the `witness` form. The reason a ten-track formula is affordable is that the *product*
automaton minimises to a single non-accepting state, which makes the subsequent projection
trivial — not that the projection is avoided. (Walnut, which builds the universal form, pays
41.8 s, but that is a Walnut/Peanut difference, not a projection/no-projection difference.)

### 7.3 Proposition D's last step is unstated

§4.3 concludes "`z` is not a 3-sum, the greedy rule would admit it, and `G != S`". The
middle clause needs `G` and `S` to agree on `[1, z)` — otherwise "the greedy rule would
admit it" is about the wrong set. They do agree, and the reason is neat and worth one line:
the only part of `Gap_0` below `z = 3g+d+2` is `[2g+d+1, 3g+d+1] = 1+g+B`, whose coverage
needs **no** hypothesis at all, so the induction of §3 runs unobstructed up to `z` for every
`g,d >= 2`. As stated the proposition has a hole; with that sentence it is complete. (My
brute force confirms the conclusion: `3g+d+2` is the exact first point of disagreement in
every failing cell.)

### 7.4 Cosmetic

* **The document is internally inconsistent about Example 19.** §3 calls the `(4,4)`
  Walnut fragment "Example 19"; §7's ledger correctly calls `S_{1,5,12}` (modulus 321,
  period 32) "Example 19". §7 is right — Example 19 *is* `S_{1,5,12}`, and the `(4,4)`
  fragment is the unnumbered display that follows it. The substantive point in §3 (the
  paper illustrates its own conjecture at `g = d = 4`, outside its `g >= d+1` hypothesis)
  survives; only the label is wrong.
* §5.1 reports `ms=4174` for T2, §5.2's table says 4.21 s, and the committed
  `results/attack4_engine.json` says 4.01 s. Run-to-run variance; harmless.
* T5/T6 test `ING0` at the *value* `3g+d+2`, which is only the right membership test
  because `3g+d+2 < 5g+2d`. That inequality is checked in the hand proof (§4.3) but not by
  the sentences themselves. Sound, but the machine checks are not self-contained without it.

Two things I tried to break and could not: the completeness of the twelve-row case analysis
in §4.1 (the twelve multisets of `{1},{g},B,A` are all of them, and the "repeated block"
cases really do sit inside the stated bounds), and the claim that `g >= d & 2g >= d+3` is
equivalent to `g >= d & (g,d) != (2,2)` under `d >= 2`.

## 8. What is actually new here

* **New and correct:** Theorem 1 — the proof of Conjecture 17, its extension to the
  diagonal `g = d` for `d >= 3`, and the sharp "only if". Proposition D is the piece with
  no counterpart in the literature and it is what makes the statement an equivalence rather
  than an implication. The explicit preperiod/period corollary (the source states only the
  counts) is a real, if small, addition.
* **New as a move, folklore as mathematics:** Lemma 1. It removes the exact blocker the
  source paper stops at, and the general form ("`r` summands give `j in {0,…,r-1}`") is
  correct. It is a change of variables, not a theorem, and the document's headline says so.
* **Not new:** the interval-sumset technique and the `2^`/`3^` identities (Shtrezi 2026);
  Conjecture 17 and its `(4,4)` verification, Remarks 18, and the Magma evidence
  (Bosma et al. 2025). All correctly cited.
* **Correctly left open:** the irregular region `2 <= g < d` and `(2,2)`; start values
  `x > 1`; and whether a better ladder can project the unrestricted six-variable
  existential. The ledger is honest on all three.

## 9. Check scripts (all written for this review)

```
paper/verdict-attack4/greedy.py         three independent greedy 3-sumfree generators; the two
                                        readings of G; OEIS + paper Theorem 15 sanity checks
paper/verdict-attack4/thm1.py           Theorem 1's iff and Prop D's first-failure point,
                                        d=2..30 x g=2..200 x 40 periods  -> thm1.json
paper/verdict-attack4/props.py          Props A-D, Lemma 1, the (k,w) encoding + T1, the twelve
                                        section-4.1 containments on every 3-subset of G, both
                                        section-4.2 chains, EX' soundness and completeness
paper/verdict-attack4/engine_check.py   my own shape of the uniform sentences, posed as CLOSED
                                        universal sentences, in Peanut  -> engine_check.json
paper/verdict-attack4/walnut_check.py   the same in Walnut 8-dev, plus the paper's own (4,4)
                                        fragment transcribed from the JIS LaTeX -> walnut_check.json
```

Reproduce:

```
cd /Users/andrew/maths
.venv/bin/python paper/verdict-attack4/greedy.py            # 0 failures
.venv/bin/python paper/verdict-attack4/thm1.py 30 200 40    # 0 failures (82 s)
.venv/bin/python paper/verdict-attack4/props.py 14 30       # 0 failures (90 s)
.venv/bin/python paper/verdict-attack4/engine_check.py      # V1-V7 pass; V8 is the open item
.venv/bin/python paper/verdict-attack4/walnut_check.py      # W0-W8 all TRUE
```
