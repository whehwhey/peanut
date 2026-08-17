# Referee verdict on `docs/ATTACK-7.md`

Adversarial read, 2026-08-17, to the standard of `paper/proof-verdict.md` and
`paper/attack2-verdict.md`. Every load-bearing step was re-derived by hand. Every finite
claim was re-checked by code **written from scratch for this review**
(`paper/verdict-attack7/`), never through the author's `explore/attack7_*.py`: my own
sequence generators (from arithmetic definitions, not from the `def` lines), my own
factor-set machinery, my own strategy-tree decider, my own Alice/Bob game solver, and my
own engine ladder emitted from the statement of Theorem 1. The primary source was read in
full from the arXiv PDF (`paper/verdict-attack7/aws.txt`), not quoted from the document,
and the predecessor paper as well (`marked.txt`).

## Bottom line

**No error was found in Lemma 1, Theorem 1, Proposition 2 or Proposition 3, and every
number in §4, §5.1, §5.2, §5.3 and §5.4 reproduced exactly from my own code — with one
exception: the "Baum-Sweet" row of §5.4 is not the Baum-Sweet sequence, and the real
Baum-Sweet word has coding dimension 3, not 2.** The period-doubling erratum is real and
I confirmed it three independent ways. The two new closed forms (paperfolding,
Rudin-Shapiro) are correct as iff-sentences over all of `N`, modulo a missing `i < j` side
condition in the prose. The document's own ledger is honest about the main thing:
**Problem 10.2 is not solved**, and even the restricted `k`-uniform case was already
effective by the source's own Theorem 5.5 + Büchi-Bruyère. What is delivered is a
*practical* procedure plus the extension of the source's §7 computations that the authors
explicitly ask for — that part is real, and the measured cost gap is large.

Two framing qualifications the document does not make:

* **The ladder is the source's own `extRS` with its extra positional variables
  existentially quantified level by level.** The source writes
  `extRS3(i,j,k,n1,n2)` and `extRS4(i,j,k,l,n1,n2,n3,n4)`; putting `E n2` (resp.
  `E n2,n3,n4`) in front is the whole change. It is a one-token edit that buys three to
  five orders of magnitude, and nobody made it, but it is a one-token edit. In-repo,
  `docs/ATTACK-6.md` made it first (binary `v = 1^d`); ATTACK-7 generalises to arbitrary
  `v`, which is a further increment.
* **The general-`v` strategy-tree characterisation (Lemma 1) is stated in the source.**
  arXiv:2106.07249v2, p.10: "To cover a general word `v`, one should draw a directed tree
  where all nodes on depth `i` have out-degree `v_i + 1`. These nodes can be indexed
  naturally by words `u` such that `u <= v` … one can then easily program the analogue
  `φ_v` of the formula `φ_def`." ATTACK-7 does not claim Lemma 1 as new, which is correct,
  but a reader of §7's "New here" bullet ("Theorem 1 in general form: arbitrary
  `v`, arbitrary alphabet") should know that only the *ladder* is new there, not the
  general-`v` setting.

## Verdict per claim

| claim | verdict |
|---|---|
| Problem 10.2 quoted verbatim; §10 context quotes verbatim | **CONFIRMED** against the v2 PDF, word for word |
| Target still open (no v3, no erratum, no follow-up) | **CONFIRMED**, and pushed further than the document (dblp, OpenAlex, Semantic Scholar, arXiv version list) |
| §2's "per-`v` effectiveness is already in the source" | **CONFIRMED** — and this is the honest core of the ledger |
| **Lemma 1** (strategy trees, general `v`, general alphabet) | **PROVED**, but the last step of the `(<=)` proof is a non-sequitur; repairable in one line (§3.1). Statement itself is **ALREADY-KNOWN** (source p.10) |
| **Theorem 1** (the ladder; `v = 1^d` binary is ATTACK-6 Prop. 1) | **PROVED** — re-derived line by line, no gap — and **MACHINE-VERIFIED** on 228 random-substitution cells + 2403 classical cells against two independent deciders |
| **Corollary** (effective procedure; arity linear in `d`) | **PROVED**; the arity counts `∏_t c_t` and `2^{d-2}` both **CONFIRMED** against the source |
| **Proposition 2** (empty-layer termination over compositions) | **PROVED** (trivially, given heredity + Prop. 6.1); the *existence* of a finite description of all of `W(X)` is the source's own Theorem 6.2 |
| **Proposition 3** (`∏_{t<d}(v_t+1) <= S`, `v_d <= |A|-1`) | **PROVED as far as it goes**, but **UNDERSTATED**: the §5.4 bound column needs `v_t <= |A|-1` for *every* `t`, which the statement omits (§4.3) |
| §4 cost table (ladder vs `extRS` vs direct) | **REPRODUCES**, including the exact figures `extRS4 = 4297 states` and `7861 MB` |
| §4 three-way equivalence | **MACHINE-VERIFIED** independently: 18 sentences, all TRUE wherever computable |
| §5.1 coding dimensions 3, 2, 3, 4 and states 1/7/6, 1/4, 1/7/6, 1/12/14/9 | **MACHINE-VERIFIED** (my own ladder) |
| §5.1 figure reproduction (useful states 8, 6, 14, 17) | **MACHINE-VERIFIED**; figure node counts **CONFIRMED** from the PDF |
| §5.2 Thue-Morse two- and three-occurrence characterisations correct | **MACHINE-VERIFIED** as iffs over all of `N` |
| §5.2 period-doubling erratum + corrected form | **CONFIRMED** three ways (brute force, raw game, engine); smallest counterexample `(a,b) = (2,4)`. **Not new in this repo** — `docs/ATTACK-6.md` §3.5, as ATTACK-7 says |
| §5.3 paperfolding `P_11`, `P_111`, Rudin-Shapiro `P_11` closed forms | **MACHINE-VERIFIED** over all of `N`, sporadic clauses genuinely required — **but the prose omits the `i < j` side condition** (§4.4) |
| §5.4 coding dimensions, 13 sequences | **MACHINE-VERIFIED**, 12 of 13 exactly as printed |
| §5.4 "Baum-Sweet", coding dimension 2 | **WRONG** — the `def` line generates the characteristic word of `{2^k - 1}`, not Baum-Sweet; Baum-Sweet has coding dimension **3** (§4.1) |
| §5.4 `dim(t x p) = 6`, `dim(t x r) = 7`, exact | **MACHINE-VERIFIED**, independently, including the 27- and 17-element nonempty lists |
| §5.4 `S` (prefix) column and Prop.-3 bound column | **REPRODUCE** exactly (14 sequences) |
| §2 "[Peltomäki-Salo 2019] covers Thue-Morse and the period-doubling word" | **WRONG** — `0 -> 01, 1 -> 00` is not left-marked, as §5.4 itself says (§4.2) |
| §7 ledger's "Failed / not done" bullets | **HONEST**; the first one (Problem 10.2 not solved) is the correct headline |

---

## 1. The primary source

`arXiv:2106.07249`: exactly two versions, **v1** 14 Jun 2021 and **v2** 17 Feb 2022; no v3,
no erratum. Journal ref *Information and Computation* **285.B** (2022) 104883,
DOI `10.1016/j.ic.2022.104883`. From the v2 PDF, verbatim:

> **Problem 10.2.** Devise an effective procedure that, given a fixed point x of a
> substitution τ and a word v, computes a finite description of `P_v(W(X))` where X is the
> subshift generated by x.

Identical to `ATTACK-7.md` §1. The three context quotations are also verbatim, including
"In our attempt to directly input the formula of Theorem 5.5, Walnut quickly ran out of
memory even in the case of one of the simplest automatic words, the Thue-Morse word."
(p.14, first line of §7) and "The proof of the main result is constructive, so in principle
an automaton for the winning shift can be found algorithmically." (p.2, intro — the
document attributes this correctly; the *other* constructivity sentence, "Theorem 6.2 is
constructive: it is in principle possible to find an automaton accepting a coding of
`W(X)`", is on p.14). ATTACK-7's parenthetical that §10 says "Section 6" where it means
§7 is **correct**: §7 is *Automata for the Winning Shifts of Certain Automatic Words*
(p.14) and there is no §6 computation.

Figure node counts read off the PDF: Fig. 2 (Thue-Morse) states `0..7` = **8**;
Fig. 3 (period-doubling) `0..5` = **6**; Fig. 4 (paperfolding) `0..13` = **14**;
Fig. 5 (Rudin-Shapiro) `0..16` = **17**. All four match ATTACK-7 §5.1.

The published characterisations in §7 are quoted correctly, and the period-doubling list is
indeed introduced as "the following characterization for the winning shift `W(X)` of the
period-doubling word" — so reading it as an iff is fair, and the erratum stands.

**Still open?** Yes, and by a wider net than the document cast.

* **dblp** full-text `winning shift`: exactly four records — arXiv+journal versions of
  *Automatic winning shifts* and of *On winning shifts of marked uniform substitutions*.
  Nothing else, ever.
* **Semantic Scholar** citations of `arXiv:2106.07249`: exactly two — Ollinger-Shallit,
  *The Repetition Threshold for Rote Sequences* (2024), and Rigo-Stipulanti-Whiteland,
  *On extended boundary sequences of morphic and Sturmian words* (2022). Neither touches
  §10. (Same two ATTACK-6 found.)
* **OpenAlex** `W3176792490`: `cited_by_count = 0`.
* dblp 2025-2026 output of both authors: Peltomäki — CPS requirement falsification, ternary
  rich words; Salo — cellular automata, Game of Life preimages, SFTs. Nothing on winning
  shifts.

Problem 10.2 has not been revisited. The target is live.

---

## 2. Re-derivation of the theory

### 2.1 Lemma 1 — correct, with a repairable proof step

`(=>)` is fine: `L(O(x)) = Fac(x)`, so any play in the first `i_d+1` rounds of a winning
strategy is a factor, and Alice must offer exactly `y_j+1` distinct letters.

`(<=)` as written ends "the resulting infinite word has all factors in `L(X)`, hence lies
in the (closed, shift-invariant) `X`". **That implication is not available in general** —
`X = O(x)` need not equal `{y : Fac(y) ⊆ L(X)}` for a non-recurrent `x`, and the class
ATTACK-7 works in (uniform, not necessarily primitive substitutions) contains
non-recurrent fixed points, e.g. `a -> ab, b -> bc, c -> cc`.

The repair is one sentence and does not touch anything else: after round `i_d` Alice holds
a word `w ∈ Fac(x)`; pick **any** occurrence `w = x[n .. n+|w|-1]` and let her play
`x[n+|w|], x[n+|w|+1], …`. The play is exactly `σ^n x ∈ O(x) = X`. Done, for every `x`.
(The source's Prop. 5.3 has the same gap and the same repair; ATTACK-7 is no worse than
the paper it generalises.)

I ran the ladder against both of my brute forces on three deliberately non-recurrent
fixed points (`0->01,1->11`; `a->ab,b->bc,c->cc`; Thue-Morse plus an absorbing letter) —
10 `(sequence, v)` cells, all AGREE.

### 2.2 Theorem 1 — correct

Both halves of the induction check out. The load-bearing points, in order:

* **Base `t = d`.** `B_d(i_d,m)` forces `m_1 = m`, so `x[m+i_d]` is one of the `c_d`
  distinct extension letters; hence `B_d(i_d,m)` ⟺ "the length-`i_d` factor at `m` has
  `>= c_d` distinct right extensions in `L(X)`" — a property of the *word*, which is what
  makes (b) hold with `m' = m`. Correct.
* **Step (a).** `FE(m,m_r,i_t)` + `i_t < i_{t+1}` gives all `u_r` the common length-`i_t`
  prefix `w`; the glue is a legal tree because every root-to-leaf word of the glued tree is
  a root-to-leaf word of some `T_r`, hence in `L(X)`. Correct.
* **Step (b).** The only delicate point is that `m_r` (produced by IH(b) from `p_r`)
  still has prefix `w` and letter `a_r` at offset `i_t`; this needs `i_t + 1 <= i_{t+1}`,
  which is exactly the conjunct `i_t < i_{t+1}`. Correct.
* Degenerate cases behave: `c_t > |A|` makes the pairwise-distinctness unsatisfiable, so
  `P_v = ∅` (checked: `tm` `v = 2`, `v = 12`, `pd` `v = 2`, `gtm3` `v = 3` all FALSE);
  `d = 1` reduces to "some length-`i_1` factor has `>= c_1` right extensions".

**Machine attack.** 57 random `k`-uniform substitutions (`k ∈ {2,3}`, `m ∈ {2,3,4}`,
codings usually non-injective, primitivity not imposed, markedness not imposed), 4 random
`v` each with letters up to 2 — **228 cells**, ladder vs my strategy-tree DP (all tuples
`< 12`) and vs my raw Alice/Bob game (all tuples `< 9`): **0 mismatches**, 3 cells skipped
because the engine hit its budget. Plus 2403 cells on the nine classical sequences
comparing my two deciders to each other: **0 mismatches**. Plus msd/lsd agreement of the
ladder on 4 `(sequence, v)` pairs.

### 2.3 Proposition 2 — correct, and lighter than it looks

The termination argument is right. Every `v` in layer `s+1` has either some `v_t >= 2`
(decrement into layer `s`) or is `1^{s+1}` (delete a letter into layer `s`); heredity
carries emptiness upward, so one empty layer kills all later layers. Layers are finite
(compositions of `s`). Finiteness of the coding dimension comes from the source's Prop 6.1.

Two caveats. (i) The *conclusion* — a finite description of the whole `W(X)` — is the
source's Theorem 6.2 (`W(X)` is `S`-codable, i.e. `ν(W(X))` is `S`-recognizable), which
the source also says is constructive. Prop. 2's added value is the stopping criterion, not
the existence. (ii) §1's justification, "Since `W(X)` is hereditary and shift-invariant,
the family `{P_v(W(X))}_v` *is* `W(X)`", names the wrong reason: what makes it true is
that finite coding dimension forces **every** `y ∈ W(X)` to have finite support, so every
`y` lies in some `Q_v`. Heredity and shift-invariance are not needed for that step.

I used heredity as a pruning rule in my own layer sweeps, which is legitimate exactly
because it is a theorem of the source ([29] Salo-Törmä, quoted in the source's §1).

### 2.4 Proposition 3 — correct as proved, understated as stated

The counting is right: the `∏_{t<d} c_t` depth-`i_d` nodes are pairwise distinct words
(they diverge at the first branch depth separating them) and each is right special. This
is the counting step inside the source's Prop. 6.1 with the constant kept — the document
says so.

But the printed constraint set is `∏_{t<d}(v_t+1) <= S` **and `v_d <= |A|-1`**, and that is
*not* what produces the §5.4 "Prop. 3 bound" column. Maximising `Σv` under the printed
constraints gives

| | tm | pd | bs | cantor | pf | rs | rs111 | rs101 | gtm3 | gtm3b | tribcode | t×p | t×r |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **printed** table | 3 | 2 | 2 | 2 | 3 | 4 | 5 | 5 | 5 | 5 | 2 | 8 | 9 |
| Prop. 3 **as stated** | 4 | 2 | 2 | 2 | 6 | 12 | 16 | 20 | 7 | 7 | 2 | 14 | 18 |
| Prop. 3 **+ `v_t <= |A|-1` for all `t`** | 3 | 2 | 2 | 2 | 3 | 4 | 5 | 5 | 5 | 5 | 2 | 8 | 9 |

(my computation, `paper/verdict-attack7/`). The missing hypothesis is trivially true — the
`c_t` branch labels at any node are distinct letters — so this is a statement bug, not a
mathematical one. It should be added to the statement.

With it, the bound holds for all 14 sequences I tested (13 of the document's plus the real
Baum-Sweet) and is attained in 7, exactly as §5.4 says.

---

## 3. Independent reproduction of the computations

All of this is my own code and my own engine scripts; times and numbers are from this run.

### 3.1 The four published examples (§5.1)

My ladder, emitted from the statement of Theorem 1 (and, reassuringly, character-for-character
identical to the listing printed in ATTACK-7 §4):

| word | `P_1` | `P_11` | `P_111` | `P_1111` | `P_11111` | dim |
|---|---|---|---|---|---|---|
| Thue-Morse | 1 | 7 | 6 | empty | — | **3** |
| period-doubling | 1 | 4 | empty | — | — | **2** |
| paperfolding | 1 | 7 | 6 | empty | — | **3** |
| Rudin-Shapiro | 1 | 12 | 14 | 9 | empty | **4** |

Identical to §5.1 and to the source's published dimensions. Dimensions are decided, not
sampled: `E i_1..i_D. $PV` TRUE and `E i_1..i_{D+1}. $PV` FALSE.

Rebuilding the source's own `(a,b,c)` / `(a,b,c,d)` encoding of the whole winning shift
(1-based, `0` = missing) and doing reachability on the exported automaton:

| word | minimal states | useful states | Fig. nodes |
|---|---|---|---|
| Thue-Morse | 9 | **8** | 8 |
| period-doubling | 7 | **6** | 6 |
| paperfolding | 15 | **14** | 14 |
| Rudin-Shapiro | 18 | **17** | 17 |

`$WS(1,5,13)` TRUE, `$WS(0,3,11)` TRUE, `$WS(1,2,3)` FALSE — the source's two worked
encodings accepted, a non-example rejected. §5.1 reproduces exactly.

### 3.2 The period-doubling erratum (§5.2)

Three independent confirmations.

1. **Brute force** over all `0 <= i < j <= 40`: the printed condition holds but the pair is
   not winning at exactly `(i,j) = (1,3), (2,6), (4,12), (8,24)` — 1-based
   `(a,b) = (2,4), (3,7), (5,13), (9,25)`. The corrected condition has **0** disagreements.
2. **Raw Alice/Bob game** from the definition: agrees with the tree decider on all 2403
   classical cells, including these.
3. **Engine**, over all of `N`: printed condition as an implication **TRUE**, as an iff
   **FALSE**; corrected condition as an iff **TRUE**.

My winning-pair list for `P_11(W(X_pd))` up to 33 is
`(0,2),(0,4),(0,8),(0,16),(0,32),(1,5),(1,9),(1,17),(1,33),(2,10),(2,18),(3,11),(3,19),(4,20),(5,21),(6,22),(7,23)`
— identical to ATTACK-6 §3.5's list. The hand certificate in ATTACK-7 §5.2 also checks
out: the length-4 factors of the period-doubling word are exactly
`{0001, 0010, 0100, 0101, 1000, 1010}`, so `010` is its **only** right-special factor of
length 3, and a tree branching at depths 1 and 3 needs two of them.

The two Thue-Morse statements are exact as printed (engine, all of `N`): **TRUE** as iffs.

### 3.3 The new closed forms (§5.3)

My own brute force over all tuples with entries `<= 26`: **0 disagreements** for all three.
My own engine sentences over all of `N`, with `i < j` (resp. `i<j<l`) added:

| statement | verdict |
|---|---|
| paperfolding `P_11` | **TRUE** |
| paperfolding `P_11` with the `d ∈ {3,5}` clause dropped | FALSE (clause is needed) |
| paperfolding `P_111` | **TRUE** |
| paperfolding `P_111` with `(0,1,4)` dropped | FALSE (clause is needed) |
| Rudin-Shapiro `P_11` | **TRUE** |
| Rudin-Shapiro `P_11` with `d ∈ {3,5}` dropped | FALSE |
| Rudin-Shapiro `P_11` with `i = 0` dropped | FALSE |

So the sporadic clauses are genuine, as claimed. Side note the document could add: for
Rudin-Shapiro `A j. j>0 => $PV(0,j)` is **TRUE** (every `(0,j)` is winning) while the same
sentence for paperfolding is **FALSE** — that is the whole content of the `i = 0` disjunct.

### 3.4 Cost of the three encodings (§4)

My transcription of the source's `extRS` ladder (from the PDF) and of the Theorem-5.5
formula (`2^d` positional variables, block agreement written with `factorEq` — the
*charitable* version; the paper's own `∀i ∈ (a,b)` form is worse), run in the same engine
against my ladder, `AM_MEM_MB = 6144`:

| word | `d` | ladder | extRS | direct |
|---|---|---|---|---|
| Thue-Morse | 2 | 7 st, 0.1 s | 7 st, 0.1 s | 7 st, 0.7 s, 183 MB |
| Thue-Morse | 3 | 6 st, 0.1 s | 6 st, 0.1 s, 1 MB | **budget** (7511 MB) |
| Thue-Morse | 4 | empty, 0.1 s | empty, 0.1 s | **budget** (7788 MB) |
| period-doubling | 2 | 4 st | 4 st | 4 st |
| period-doubling | 4 | empty | empty, 2 MB | empty, 812 MB |
| paperfolding | 3 | 6 st, 1 MB | 6 st, 2 MB | **killed** (RSS watchdog) |
| Rudin-Shapiro | 2 | 12 st, 1 MB | 12 st, 1 MB | **budget** (8081 MB) |
| Rudin-Shapiro | 3 | 14 st, 1 MB | 14 st, 11 MB | **budget** (6144 MB) |
| Rudin-Shapiro | 4 | 9 st, 1 MB | **budget** — `extRS4` builds at **4297 states**, then **7861 MB** | **killed** |

The two exact figures ATTACK-7 quotes for the hardest cell — `extRS4` = 4297 states and
7861 MB at the final projection — came out identical on my independent transcription.
The `direct` column reproduces what the authors report: unusable already on Thue-Morse.
§4's honest caveat (ii) — that Walnut evidently *did* complete `extRS4` for Rudin-Shapiro,
since Figure 5 exists — is the right thing to say and should stay.

**Equivalence.** 18 sentences `A i_1..i_d. $PV <=> $EXT` / `<=> $DIRECT`: every one that
is computable is **TRUE**; the rest hit the budget. No encoding disagrees anywhere.

### 3.5 Coding dimensions over general alphabets (§5.4)

My layered sweep (engine per `v`, heredity used only for pruning):

| sequence | dim (ATTACK-7) | dim (here) | maximal `v` (here) |
|---|---|---|---|
| Thue-Morse | 3 | **3** | `111` |
| period-doubling | 2 | **2** | `11` |
| "Baum-Sweet" (`def` line as printed) | 2 | **2** | `11` |
| **real Baum-Sweet** | — | **3** | `111`, witness depths `(0,1,3)` |
| Cantor (complement, see §4.5) | 2 | **2** | `11` |
| paperfolding | 3 | **3** | `111` |
| Rudin-Shapiro | 4 | **4** | `1111` |
| `#111` mod 2 | 4 | **4** | `1111` |
| `#101` mod 2 | 4 | **4** | `1111` |
| `s_3(n) mod 3` | 4 | **4** | `211`, `22` |
| `s_2(n) mod 3` | 4 | **4** | `211`, `22` |
| periodic test `(0110)^ω` | 2 | **2** | `11` |
| Thue-Morse × paperfolding | 6 | **6** | `33` |
| **Thue-Morse × Rudin-Shapiro** | 7 | **7** | `331` |

`t × r`: 176 words `v` over `{1,2,3}` with `Σv <= 8` (`1+2+4+7+13+24+44+81 = 176`, correct),
**27 nonempty**, and my nonempty list is *identical* to `results/attack7_products.json`
element for element. Every `Σv = 8` word is empty (each has an empty child in layer 7,
where only `331` survives), so the dimension is exactly 7. `t × p`: 95 words
(`Σv <= 7`), **17 nonempty**, list identical, dimension exactly 6. The asymmetry claims
also hold: `P_{211} ≠ ∅` while `P_{121} = P_{112} = ∅` for `s_3(n) mod 3`; `P_{331} ≠ ∅`
while `P_{313} = P_{133} = ∅` for `t × r` (and `P_{3111} ≠ ∅` while `P_{3211}`, `P_{3121}`,
`P_{3112}`, `P_{31111}` are all empty).

Right-special counts `S` (prefix, `n <= 54`) and the Prop.-3 bound reproduce for all 13
sequences (table in §2.4 above). The markedness claims check out: `0->01,1->00` and
`0->01,1->02,2->31,3->32` and `0->01,1->23,2->01,3->23` all have repeated first letters, so
none is left-marked (definition confirmed from arXiv:1705.08747: "left-marked if all of its
τ-images begin with distinct letters").

---

## 4. Defects found

### 4.1 `WRONG` — the "Baum-Sweet" row of §5.4 is not Baum-Sweet

`SEQS['bs'] = def T 2 3 0 01 21 22 110` generates `1,1,0,1,0,0,0,1,0,…,1(n=15),…` — the
characteristic word of `{2^k - 1}` (its DFAO has an absorbing 0-state entered by any digit
`0` after the leading `1`). The Baum-Sweet sequence is
`1,1,0,1,1,0,0,1,0,1,0,0,1,0,0,1,…`; they first differ at `n = 4`
(`b(4) = 1`, since `100` has an even zero-block). The two words are not equal, not
complementary, and not shifts of each other.

The reason is structural, not a typo: the msd Baum-Sweet automaton must move off its start
state on a leading `0` (`q0 -0-> q1`), which the `def` command forbids
(`not prolongable at start letter` / leading zeros must not change the value). Baum-Sweet
needs four letters:

```
def T 2 4 0 01 21 13 33 1100          # a->ab, b->cb, c->bd, d->dd, coding a,b -> 1, c,d -> 0
```

verified against `b(n) = [no odd block of 0s in (n)_2]` on 200 000 terms. Its winning shift
has **coding dimension 3** (witness depths `(0,1,3)`, confirmed by tree DP, raw game and
engine; `P_1111 = ∅`), `S_prefix = 7`, Prop.-3 bound 3 (tight). So the §5.4 row should be
*two* rows, or one corrected one; the "13 sequences" count and the "attained in 7 of the 13"
count both survive the fix.

Nothing else in the document depends on this row.

### 4.2 `WRONG` — §2 says the 2019 marked-substitution theorem covers the period-doubling word

§2: "Peltomäki-Salo … Theorem 4.9 … That covers Thue-Morse and the period-doubling word".
Theorem 4.9 assumes `τ` **marked**, and arXiv:1705.08747 defines left-marked as "all of its
τ-images begin with distinct letters". `0 -> 01, 1 -> 00` begins both images with `0`.
§5.4 of ATTACK-7 says exactly this ("the period-doubling morphism `0->01, 1->00` … [is]
non-left-marked"), so the document contradicts itself. Delete "and the period-doubling
word" from §2. (Thue-Morse `0->01,1->10` is genuinely marked, so the rest of the sentence
is fine.)

### 4.3 `UNDERSTATED` — Proposition 3 omits a hypothesis it uses

See §2.4. Add `v_t <= |A| - 1` for **all** `t`, not just `t = d`. Without it the bound
column of §5.4 does not follow from the proposition as stated (it would read
4, 2, 2, 2, 6, 12, 16, 20, 7, 7, 2, 14, 18).

### 4.4 `COSMETIC` — §5.3's iffs need the ordering side condition

As printed, "`(i,j) ∈ P_11(W(X))  <=>  i = 0 or …`" for Rudin-Shapiro is false at
`(i,j) = (0,0)`: `P_v` is empty off the increasing diagonal but the right-hand side is true.
The engine sentences in `explore/attack7_chars.py` do carry the guard (`E d. i1+d=i2 &
d>=1 & (i1=0 | …)`), so only the prose is affected. §5.2 states the analogous guard
explicitly ("adding only `b > a`"); §5.3 should too.

### 4.5 `COSMETIC` — two smaller things

* §1: "Since `W(X)` is hereditary and shift-invariant, the family `{P_v(W(X))}_v` *is*
  `W(X)`." The operative fact is **finite coding dimension** (so every `y ∈ W(X)` has
  finite support and lies in some `Q_v`), not heredity or shift-invariance.
* §5.4's "Cantor characteristic word" row: `def T 3 2 0 010 111 01` is the characteristic
  word of the **complement** of the Cantor set (`x(n) = 1` iff some base-3 digit is 1);
  `x(0) = 0`, whereas the Cantor characteristic word has `x(0) = 1`. Since relabelling the
  alphabet is an isomorphism of the game, `W(X)` and the coding dimension are unaffected —
  the name is off, the number is right.

### 4.6 Non-defects I tried and could not break

* Branch degree exceeding the alphabet (`c_t > |A|`): correctly empty.
* msd vs lsd: identical tuple sets on 4 `(sequence, v)` pairs.
* Non-recurrent fixed points: 10 cells, ladder = tree DP = raw game.
* 228 random-substitution cells: 0 mismatches.
* `results/attack7_chars.json`: 34 entries, 0 failures; I re-derived the substantive 11 of
  them from scratch and got the same verdicts. The document's `3*i2<=i3` / `3*i1<i2`
  encodings are equivalent to my `2*i2<=d` / `2*i1+2<=d` ones (`d` a power of 2, hence even).
* `results/attack7_products.json`: 27/17 nonempty lists — element-for-element identical to
  mine.

---

## 5. What is actually new

**New and correct (relative to the literature).**
* The **closed forms for `P_11`, `P_111` of the paperfolding word and `P_11` of the
  Rudin-Shapiro word** (§5.3). The source gives only Figures 4 and 5, and ATTACK-6's ledger
  lists these as not established. Machine-proved over all of `N`. This is the cleanest new
  mathematical content in the document.
* **Exact coding dimension 7 for `t × r`** (and 6 for `t × p`), over a 4-letter alphabet,
  from a complete decision of every `v` with `Σv <= 8`. Exact, not certified-from-below.
  The published maximum was 4. (ATTACK-6's binary 10 is a bigger number but its upper
  bounds are brute force or absent.)
* **Measured cost of the two published encodings**, machine-checked equivalent to the
  ladder. The authors guessed "the methods scale very badly"; this quantifies where.
* The **figure-level reproduction** of the source's §7, useful-state counts and all.

**New in this repo but not in the literature.**
* Theorem 1 for general `v` and general alphabets. The *ladder* idea is ATTACK-6's
  (binary `v = 1^d`) and is a one-quantifier edit of the source's `extRS`; the general-`v`
  strategy-tree setting is sketched in the source itself. Correct, useful, incremental.
* Propositions 2 and 3 in general form.
* The raw-game verification layer.

**Not new (correctly not claimed).**
* Effectiveness of `P_v` for `k`-automatic `x` — source Theorem 5.5 + Büchi-Bruyère.
* A finite description of all of `W(X)` — source Theorem 6.2.
* The period-doubling erratum — `docs/ATTACK-6.md` §3.5, same day, same counterexample.
* The four published coding dimensions and the Thue-Morse characterisations.

**Not established.**
* Problem 10.2 as posed (general, non-uniform substitutions). Untouched, and the document
  says so.
* Closed forms for `P_111`/`P_1111` of Rudin-Shapiro and for the products.
* `S` above `B = 6-7` by machine.
* Any relation between `dim W(X × Y)` and `dim W(X)`, `dim W(Y)`.

---

## 6. Check scripts (all written for this review)

```
paper/verdict-attack7/aws.txt               arXiv:2106.07249v2, pdftotext -layout of the arXiv PDF
paper/verdict-attack7/marked.txt            arXiv:1705.08747, ditto (the 2019 marked-substitution paper)
paper/verdict-attack7/ref.py                sequences from arithmetic definitions; audit of every
                                            `def` line; factor sets with a prefix-stability check;
                                            strategy-tree decider; raw Alice/Bob game solver
paper/verdict-attack7/eng.py                the Theorem-1 ladder, emitted from the statement
paper/verdict-attack7/run_brute.py          tree DP vs raw game, 2403 cells
paper/verdict-attack7/run_chars_brute.py    period-doubling erratum + TM two-occurrence, brute force
paper/verdict-attack7/run_chars3.py         all four §5.2/§5.3 formulas, brute force to depth 26
paper/verdict-attack7/run_whole.py          the source's (a,b,c) encoding; useful-state counts
paper/verdict-attack7/run_layers.py         layered `Σv` sweep with heredity pruning (per sequence)
paper/verdict-attack7/run_encodings.py      ladder vs extRS vs Theorem-5.5-direct, cost + equivalence
paper/verdict-attack7/run_dims.py           depth-bounded dimension search (brute force)
paper/verdict-attack7/run_stress.py         57 random uniform substitutions x 4 v = 228 cells
paper/verdict-attack7/*.json, *.log         their outputs
```
