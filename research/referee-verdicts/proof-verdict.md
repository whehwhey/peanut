# Referee verdict on `paper/proof-upper.md`, `proof-family.md`, `proof-lower.md`

Adversarial read, 2026-08-17. Every load-bearing lemma was re-derived by hand and, where
it is a finite statement, re-checked by an *independent* brute force written for this
review (not the authors' scripts) plus fresh engine runs. The check scripts are committed
under `paper/verdict-checks/` (`fam_check.py`, `thm44.py`, `prop51.py`, `exh_nonbin.py`).

## One-line verdict per file

* **`proof-upper.md` — CORRECT, CONDITIONAL.** No error found in any lemma, by hand or by
  machine (including five adversarial DFAOs the authors never tested). The main theorem is
  true but its unconditional content, $2^{O(m^3)}$, is weaker than the literature's
  $2^{9m^2}$; the deliverable is $\Lambda$ as a cheap predictor, not a bound. Two cosmetic
  overstatements in §7.3 / Cor 5.6.
* **`proof-family.md` — CORRECT, AND THE ONLY NEW THEOREM.** Lemma 3.3, Prop 3.4, Cor 3.5,
  Prop 5.1, Prop 5.3 and Theorem 4.4 all survive an independent re-derivation and an
  independent brute force; $|FE_{G_p}|_{msd}\ge p^3$ is genuinely proved and I reconstructed
  it from scratch. Two overclaims: "the lossy-coding lead is closed" is **circular**
  (level 0 of the grading *is* the original predicate), and "$O(m^3)$ unconditionally"
  is quoted from a 15-point polynomial fit (the $O(m^3)$ itself is fine, by inspection of
  the state space).
* **`proof-lower.md` — DATA SOUND, THEORY WEAKEST.** Every measured table reproduces
  exactly, and the model predicts $|FE|$ better than the document claims (I measured it).
  But Lemma 7 is stated backwards, Lemma 3's hypothesis is unsatisfiable, Lemma 1's last
  sentence is false, §5's "dead, by proof" is an overclaim (the model is *coding-blind*,
  and coding is the whole subject of §5), and §7's "exhaustive over all $m$-state DFAOs"
  was binary-coding only — I re-ran the missing cases and the maxima happen to survive.

## Ranking

1. **`proof-family.md`** — the only document containing a *new, correct, unconditional
   theorem* (Thm 4.4, $|FE_{G_p}|_{msd}\ge p^3$). Independently re-verified.
2. **`proof-upper.md`** — every claim I checked is correct and the machine checks pass with
   0 violations; but the main theorem is conditional and its unconditional specialisation is
   weaker than the bound already in the literature. No errors found.
3. **`proof-lower.md`** — the measurements are exactly reproducible (I re-ran three of its
   tables from scratch), but the theoretical spine is the weakest of the three: the model
   that carries §2–§5 is *provably blind to the very parameter §5 is about*, one lemma is
   stated backwards, one lemma rules out a hypothesis no sequence satisfies, and §7's
   "exhaustive" claim is narrower than stated.

---

## 1. `proof-upper.md` — **correct; conditional; no errors found**

### Re-derived by hand
* **Lemma 1.2 / 1.3** (block factorisation, refinement) — correct.
* **Lemma 3.1** (window locality). $i+l=(I+L)k^s+i'+l'<(I+L+2)k^s$: correct, and the
  $L+2$ blocks per side are exactly right.
* **Corollary 3.2** ($m^4$, $m^6$) — correct.
* **Proposition 4.3** (middle condition $\equiv$ triple-set containment). The $e\ge0$ /
  $e<0$ split is right; $e<0$ needs $c\ge1$, which holds because $c$ ranges over
  $[1,L-1]$. Correct.
* **Theorem 4.4** (8 states + $\Theta$). I checked each of the three regions independently:
  head $\subseteq$ blocks $J,J+1$; tail $i$-side $\subseteq I+L,I+L+1$ and $j$-side
  $\subseteq J+L-1,J+L,J+L+1$ (since $|e|<k^s$); middle non-empty because $L\ge2$; and none
  of the three conditions depends on $L$ numerically once $\Theta$ is given. Correct.
* **Lemma 5.1** (descent). Checked the chunk arithmetic: chunk $r$ of $B_s(u)$ sits at
  offset $\varepsilon'$ inside $Z$-chunk $d+r$, and $d+r+1\le 2k-1$. Correct.
* **Theorem 5.4**. $\Theta$ really is a function of the pair
  $(A^+,A^-)=(\bigcap_{t\in\mathcal T^+}P_t,\bigcap_{t\in\mathcal T^-}P_t)$ because
  $(s,\varepsilon)\mapsto \mathrm{Sh}_{s,\varepsilon}$ depends on $T$ only. Correct.
* **Theorem 6.1**. (i) needs $\Delta=|i'-j'|<k^s$ and $l\ge Lk^s\ge\rho k^s>(\rho-1)\Delta$
  — fine, and $l\ge1$ always holds for $L\ge2$ (and at $s=0$, $i'=j'$ forcibly).
  (ii) needs $\Delta>0$, which holds because $I\ne J$ forces
  $\Delta\ge k^s-(k^s-1)=1$. Correct.

### Machine check
`paper/proof-upper-check.py` run to completion: **0 violations** on all five worked
examples, all of `[Cor 3.2]`, `[Lem 5.1]`, `[Prop 4.2]`, `[Thm 4.3]`.

I then re-ran the same battery on **five adversarial examples the authors did not test** —
the $m=4$ extremal automaton $C_4$ (the $|FE|=1152$ champion of `proof-lower.md` §7), the
thin set "exactly 2 ones", the ruler $r=4$, and *both* codings of the group automaton
$\mathbb Z_4$ — i.e. the four structures every document flags as dangerous
(unbounded critical exponent, thin sets, lossy codings). **0 violations in every cell**,
$I,J,L<10$, $s\le4$. Theorem 4.4's 8-tuple and Prop 4.3's middle language hold up
everywhere I could push them.

| example | $\gamma$ | $\Lambda$ | $\#\Theta$ | $\lvert FE\rvert$ (engine) |
|---|---|---|---|---|
| $C_4$ champion `0 01 22 33 10 / 1110` | 23 | **74** | 61 | **1152** |
| ruler $r=4$ | 9 | 35 | 61 | 113 |
| exactly-2-ones | 14 | 30 | 10 | 183 |
| $\mathbb Z_4$ singleton coding | 13 | 28 | 24 | 698 |
| $\mathbb Z_4$ faithful coding | 3 | **4** | 2 | 72 |

Two things fall out. (a) The extremal $m=4$ automaton has the largest $\Lambda$ of any
$m=4$ DFAO I looked at, and the faithful group automaton the smallest — $\Lambda$ does
separate the extremes. (b) $\Lambda$ is **not** monotone in $|FE|$ across families: the
ruler has $\Lambda=35$ with $|FE|=113$ while the singleton $\mathbb Z_4$ has
$\Lambda=28$ with $|FE|=698$. This is a concrete instance of the one-way-ness the document
concedes in §8.2, and it is worth putting in §7 as a caveat.

### Weakest step
Not a *wrong* step — the document is unusually honest — but the weakest link is
**Definition 5.3 + Theorem 5.4 as a bound**: since $\Lambda\le 2^{\min(\gamma,m^3)}$ and
nothing controls $\gamma$, the unconditional content is $2^{O(m^3)}$, i.e. *worse* than
Khodier's $2^{9m^2}$. §8.1 says so. The theorem is a re-parametrisation of the obstruction,
not a bound. Its actual value is Corollary 5.6: $\Lambda$ is a cheap, engine-free predictor,
and it is empirically the only invariant found anywhere in this repo that separates the
faithful from the lossy coding of the same DFAO.

### Defects found (both cosmetic)
* **§7.3 prose contradicts its own table.** "In all three families the growth of $\Lambda$
  tracks the growth of $|FE|$" is false for the faithful GTM family: $\Lambda\equiv4$ for
  all $p$ while $|FE_{G_p}|=p^3+8$ grows cubically (and I confirmed $|FE|$ at
  $p=3,7,11,13,16$ = $35,351,1339,2205,4104$). $\Lambda$ separates *codings*, it does not
  track $|FE|$.
* **Corollary 5.6** says "time polynomial in $m^3$ and $\Lambda$" but the orbit cost is
  $O(k\gamma m^3)$ — it is output-polynomial in $(\gamma,\Lambda)$, and $\gamma$ can a priori
  be exponential.

### Status
| claim | verdict |
|---|---|
| Lemmas 1.2–1.3, 3.1, Cor 3.2, Prop 4.3, Thm 4.4, Lem 5.1, Thm 5.4, Cor 5.5, Thm 6.1 | **PROVED** |
| $\Lambda$ as a computable predictor (Cor 5.6, §7) | **PROVED** (algorithm) + measured |
| $\Lambda(T)=\mathrm{poly}(m)$, hence $|FE|=\mathrm{poly}(m)$ | **OPEN** — explicitly not claimed |
| Theorem 5.4 as an improvement on $2^{9m^2}$ | **NO** — explicitly disclaimed |

---

## 2. `proof-family.md` — **the real result; one overclaimed headline**

### Independently re-verified (my own brute force, from the definition of $FE$)
* **Lemma 3.3 (parity rigidity).** Hand proof checked line by line: (t=0,1) give
  $G[b'{+}1]-G[b']=2$, (t=2,3) give $G[b'{+}2]-G[b'{+}1]=2$, so
  $\nu(b')\equiv\nu(b'{+}1)\equiv-1\pmod p$; $\nu(b')\equiv-1$ forces $\nu(b')\ge1$ so
  $b'$ is odd, hence $b'{+}1$ is even and $\nu(b'{+}1)=0\not\equiv-1$ for $p\ge2$.
  **Correct.** Brute force $p=2..9$, $a,b<300$: **0 failures**. (I also re-derived (I2),
  $G[n{+}1]-G[n]=1-\nu(n)$, which the whole file rests on.)
* **Prop 3.4 (complete characterisation).** Brute force $p=2..7$, $i,j,l<70$
  (2 058 000 triples): **0 mismatches**.
* **Cor 3.5 (bit-level form).** Brute force $p=2..7$, $i,j<200$: **0 failures** on all
  three clauses and on the $L$ formula (I re-derived
  $L=l_{hi}+\lfloor(r+l_{lo}-1)/2^k\rfloor+1$ and it matches the three cases).
* **Theorem 4.4 ($\ge p^3$).** This is the load-bearing claim, so I rebuilt the whole
  construction from scratch — the words $X_{A,\rho}$, the three suffix families
  $A_{c,k},B_{c,k},C_{c,k}$ — and evaluated $FE$ **from its definition** (digit sums), not
  through Cor 3.5. Result: distinct response vectors $=27,64,125,216,343,512$ for
  $p=3..8$, i.e. exactly $p^3$, and the partition of prefixes by response vector refines
  exactly to the key $(D,\rho,\rho')$. Every step of the written proof also checks out by
  hand: $L=1,2,3$ for the three families (including the $k=0$ edge cases),
  $\beta=(S{-}1{-}k)+\rho'$ and $\alpha=(S{-}1{-}k)+\rho$ pick up the prefix's trailing run
  correctly because the $X$'s have an explicit separating $0$, $\beta\equiv-1$ is automatic
  in family $C$ because the run has length $p-1$, and all bit positions used are
  $\le 3p-2<S=3p+6$. **PROVED.**
* **Prop 5.1 (block characterisation of $T_p$).** Independent brute force $p=2..7$,
  $i,j,l<48$: **0 mismatches**.
* **Prop 5.3 (level decomposition).** Re-derived: $(\mu+[0,\kappa])\cap\{1,1-\theta\}
  =\emptyset$ $\iff$ $\mu\notin V_\kappa$ and $\mu+\theta\notin V_\kappa$, since
  $V_\kappa=\{1-x:x\in[0,\kappa]\}$; adding the $\theta=0$ escape gives exactly
  $\chi_\kappa(\mu)=\chi_\kappa(\mu+\theta)$ (the "both in $V_\kappa$ and equal" branch
  is only reachable at $\theta=0$). **Correct.**
* **§1's aside** that $[s_2\not\equiv0]$ and $[s_2\equiv1]$ give the same $|FE|$: confirmed
  by engine — every singleton target $0..\min(p,4)-1$ gives the identical count
  ($190,698,1877,3971,7243$ for $p=3..7$).
* **The exact laws.** Fresh engine runs: msd $=35,351,1339,2205,4104$ and lsd
  $=39,131,287,389,572$ at $p=3,7,11,13,16$ — exactly $p^3+8$ and $2p^2+3p+12$.

### Weakest step 1 — "the last standing lead is closed" is **circular**
The abstract, §2 and §6.2 assert that Prop 5.3 *closes* the "lossy coding of a group
automaton" lead. It does not, and the reason is sharper than gap G3 admits:

> **Level $0$ of the grading is the original problem.** Prop 5.3 says the effective coding
> at level $k=v_2(j-i)$ is $\chi_{\min(k,p-1)}$. At $k=0$, $V_0=\{1\}$ and $\chi_0$ *is* the
> singleton coding, so level $0$ is $FE_{T_p}$ restricted to odd differences. The
> conclusion "it cannot manufacture an exponential unless some single level's own $FE$ is
> already exponential" therefore reads "$|FE_{T_p}|$ is not exponential unless
> $|FE_{T_p}|$ is exponential."

The document's own §5.4 point 2 states this ("level 0 *is* $T_p$, so this is not by itself
a bound"), and G3 says "no upper bound is proved at all" — but the abstract and §6 still
say "closed". **Fix the headline, not the mathematics.**

There is a second, unstated hole in the same inference. The claimed ceiling
$O(p)\cdot\max_\kappa|FE_{\chi_\kappa\circ G_p}|$ assumes the levels *add*. In msd the
automaton cannot know $\kappa=v_2(j-i)$ from a prefix — it is a low-bit quantity — so a
residual may have to record behaviour at *every* level simultaneously, which permits a
product $\prod_\kappa$, not a sum. Nothing in §5 excludes that, and a product of $p$
polynomials is exactly the shape an exponential family would have. (Empirically it does not
happen — §5.4's ratios converge — but that is data, not proof.) Also, Prop 5.3 covers only
*fully covered* blocks; the two boundary blocks are outside the grading entirely.

### Weakest step 2 — "$O(m^3)$ unconditionally" in §4.1/§4.2 is a *fit*, not a proof
"$|\mathcal A^{msd}_p| = 34p^3+98p^2+41p$ (exact fit, third difference constant, $2\le p\le16$)"
is a 15-point polynomial fit to a program's output; "so $\dots\le34p^3+98p^2+41p$
**unconditionally**" does not follow. What *does* follow, and should be said instead, is
that the declared state space $(D,\rho_i,\rho_j,H,M)$ has size $O(p^3)$ **by inspection**
($p^3\cdot5\cdot(1+4\cdot5\cdot|\Sigma|)$), which is all Theorem 4.5 needs. Same for lsd:
the $O(p^2)$ follows from the "both registers active $\Rightarrow D=\pm1$ and equal counts"
coupling, not from the fitted $128p^2+160p+19$.

### Weakest step 3 — Theorem 4.3 is computer-assisted, and only partly cross-checked
$|FE|=p^3+8$ for each $p\le24$ rests on (a) a construction proved correct *as a design*
from Cor 3.5 but implemented in Python and brute-forced only to $p\le6$, $i,j,l<48$, and
(b) exact minimisation. Three independent computations agree for $p\le16$; for
$17\le p\le24$ **only the engine** was run, so those eight values rest on one implementation.
That is still strong, but "proof" should be qualified as *machine-verified*.

### Status
| claim | verdict |
|---|---|
| Lemma 3.1, Cor 3.2, **Lemma 3.3**, **Prop 3.4**, Cor 3.5, **Prop 5.1**, Prop 5.3 | **PROVED** (hand proof correct + independently brute-forced) |
| **Thm 4.4: $\lvert FE_{G_p}\rvert_{msd}\ge p^3$ for all $p\ge3$** | **PROVED** — independently reconstructed and confirmed |
| $\lvert FE_{G_p}\rvert_{msd}=O(p^3)$, $\lvert FE\rvert_{lsd}=O(p^2)$ | **PROVED**, but by inspection of the state space, *not* by the quoted fitted polynomials |
| $\Theta(m^3)$ msd for $G_p$ | **PROVED** (modulo the construction's implementation) |
| $\lvert FE_{G_p}\rvert_{msd}=p^3+8$, $\lvert FE\rvert_{lsd}=2p^2+3p+12$, $p\le24$ | **MACHINE-VERIFIED** for each $p$; not proved uniformly (G1, correctly flagged) |
| "the lossy-coding lead is closed" | **NOT PROVED — circular.** Level $0$ of the grading is the original predicate |
| $\Omega(p^4)$ or any upper bound for $T_p$ | **NOT ESTABLISHED** (correctly flagged, G3/G4) |

---

## 3. `proof-lower.md` — **honest negative result, correct data, weak theory**

### Data: reproduces exactly
I re-ran three tables from scratch with the engine (fresh definitions, not the authors'
drivers):
* ruler family $T_r[n]=[\nu_2(n{+}1)\equiv0\bmod r]$: $8,34,113,305,712,1471,2751$ for
  $r=2..8$ — **exact match**.
* $D_m$ ($m$-th most significant digit): $71,113,171,241,319,405$ for $M=5..10$ —
  **exact match**.
* the $m=4$ exhaustive champion `def T 2 4 0 01 22 33 10 1110` $\to$ **1152**, the $m=3$
  champion `0 01 22 10 / 110` $\to$ **264** (full binary exhaust re-run, 109 classes, 4 s),
  and the $m=5$ sample champion `0 02 33 41 24 10 / 00001` $\to$ **2415** (186 s) — all exact.

### First, a point in the model's favour that the document does not make
Nowhere does `proof-lower.md` measure whether $\mathcal I$ or $\mathcal T$ actually
predicts $|FE|$ on the random ensemble. I did: over the 385 morphisms of
`results/blowup.json`, the within-$(k,m)$ Spearman correlation of $\mathcal T$ (tracker
states of the base automaton) against measured $|FE|$ is

    k=2, m=2..7:  0.91  0.65  0.86  0.65  0.62  0.39
    k=3, m=2..7:  0.61  0.69  0.34  0.56  0.44  0.35

median $\approx+0.6$, positive in all twelve cells — *better* than
`proof-upper.md`'s $\Lambda$ (median $\approx+0.5$) on the same data. So the model is a
real predictor within a random size class, and §2 undersells it. That makes what follows a
scoping criticism, not a dismissal.

### Weakest step 1 — the interval-image model is **coding-blind**, which kills its only stated credential
`explore/interval_img.py::ii_states(k, trans)` takes **only the transition function**. The
output coding $\tau$ never enters. So $\mathcal I$ and $\mathcal T$ are identical for the
faithful $G_p$ and for the singleton $T_p$ — two sequences with the *same* DFAO whose
$|FE|$ differ by two orders of magnitude and, more importantly, by their exponent.

§2 defends the model with: "its only credential is that it predicts the one exponent that
is independently known (quartic, for group automata with lossy codings)". That credential
is backwards:

* the exponent that is *independently known* is the one **proved** in `proof-family.md`:
  $|FE_{G_p}|_{msd}=\Theta(p^3)$, **cubic**;
* the model gives $\mathcal I(\mathbb Z_p)$ **quartic** (I reproduced
  $3,7,15,31,61,113,197,325,511,\dots$) for the *same automaton*;
* the family whose measured exponent is $\approx4$ is $T_p$, whose exponent is exactly the
  thing `proof-family.md` §5.4/G4 says is **unsettled** (a cubic $44.4p^3-\dots$ fits the
  same six points, and the local log-log slope is falling *through* 4).

So the model's one quantitative success is a coincidence attached to the wrong family, and
it demonstrably cannot see the parameter (the coding) that §5 is entirely about. It is also
not ordinally faithful on the authors' own §5 table: $\mathcal I$ rises $31\to46$ from
$\mathbb Z_5$ to $\mathbb Z_6$ while $|FE|$ *falls* $1579\to991$. And the model's stated
form is $|FE|\approx\mathcal T(\text{correlation automaton})$, an automaton with
$\Theta(m^2)$ states — but every number reported is $\mathcal T$ or $\mathcal I$ of the
$m$-state *base* automaton (e.g. $\mathcal T(\mathbb Z_8)=3608$ vs $|FE|=520$ faithful /
$11988$ singleton). Under its own stated form the prediction would be $\mathcal I(m^2)\sim m^8$.

**Consequence:** the section heading "Mechanism 3 — codings of group automata: dead, by
proof" is **wrong as stated**. Lemma 5 and Corollary 6 are correct theorems *about
$\mathcal I$ of an abelian translation automaton* (I checked the proof: the abelian
regrouping $u_d a_\alpha S_{d'-d-1}a_{c'}S_{n-d'-1}=u_d(a_\alpha a_{c'})S_{n-d-2}$ is valid,
the $k=2$ and linear-$a_c=cg$ instantiations are valid), but they say **nothing** about
$|FE|$ of any coding of that automaton, and no upper bound on $|FE_{T_p}|$ exists in any of
the three documents.

### Weakest step 2 — Lemma 7 is stated backwards (but is repairable)
"$|FE_T|\le|FE_1||FE_2|\le C^2m_1^am_2^a$, so if $|FE_i|\le Cm_i^a$ then
$|FE_T|\le C^2m(T)^a$" requires $m_1m_2\le m(T)$, whereas the preceding sentence correctly
says $m(T)\le m_1m_2$. As written the implication does not hold, and the case it silently
excludes — $m(T)\ll m_1m_2$ — is precisely the leverage a designer of an exponential family
would want.
**Repair:** $T_i=\pi_i\circ T$ is a *coding of $T$*, so $m_i\le m(T)$, giving
$|FE_T|\le C^2m(T)^{2a}$. Polynomiality is preserved with exponent $2a$, not $a$. Please
substitute this argument.

### Weakest step 3 — Lemma 3 rules out a hypothesis nothing satisfies, with an informal proof
"$FE(i,j,l)\iff i\equiv j\pmod{k^{f(l)}}$" is a *complete* description of the language, so
the hypothesis already determines $|FE|$ with no reference to $T$ — the "$m$" in the
conclusion $O(k^Cm)$ is spurious. No automatic sequence satisfies it: for Thue–Morse,
$FE(0,3,1)$ holds and $FE(0,1,1)$ fails, so the length-1 slice is not a congruence
condition on $i-j$ for any $f(1)$; and $G_p$'s real characterisation is Prop 3.4, which is
a congruence *plus* two run-length conditions *plus* $L\le3$. The proof ("the automaton therefore knows at each digit position, up to an
additive constant $C$, …") is a sketch. The real content of §3 is the ruler measurement,
which is sound. Label the lemma as a heuristic.

### Weakest step 4 — §7's "exhaustive over all $m$-state DFAOs" is only over **binary** codings
`explore/fe_exhaust4.py` iterates `for cbits in range(1, 2**m - 1)` — 2-block output
partitions only. Since $FE$ depends on $\tau$ only through the induced partition of $Q$,
the search covered $7$ of the $\mathrm{Bell}(4)=15$ partitions at $m=4$ and $3$ of $5$ at
$m=3$. "the exact maxima $\max_{|Q|=m}|FE|$ … over all minimal zero-stable DFAOs" is
therefore not what was computed.

**I closed this gap.** I enumerated the missing partitions ($\ge3$ blocks) exhaustively,
same canonicalisation, engine-evaluated, 0 failures:

| $m$ | classes with $\ge3$-block coding | max $\lvert FE\rvert$ | binary max (theirs) |
|---|---|---|---|
| 3 | 71 | **50** (`0 01 21 10 / 012`) | 264 |
| 4 | 5329 | **492** (`0 01 22 33 10 / 0122`) | **1152** |

So the numbers $264$ and $1152$ *do* survive as the true maxima over all output alphabets —
but the document did not establish that, and the finer codings lose by a factor $\ge2.3$,
which is itself evidence for the coding mechanism of `proof-family.md`. (Script:
`paper/verdict-checks/exh_nonbin.py`.)

### Weakest step 5 — smaller defects
* **Lemma 1**, last sentence: "any window wide enough to contain a full cone of depth
  $\le n-|P|$ therefore has image exactly $R_n(p_0)$" is **false**. Counterexample: $k=2$,
  $P=\{p_0,a\}$, $\delta(p_0,0)=p_0$, $\delta(p_0,1)=a$, $\delta(a,\cdot)=a$; the interval
  $A=10^{n-1}$, $B=1^n$ contains a saturated cone rooted at $a$ but
  $\mathrm{Img}=\{a\}\subsetneq R_n(p_0)=\{p_0,a\}$. The containment
  $\mathrm{Img}\subseteq R_n(p_0)$ and the monotonicity $R_r(p_0)\subseteq R_{r+1}(p_0)$
  are both correct, and only those are used later.
* **§5** repeats `docs/TARGET1.md`'s "$3p^4$ within 2.5%"; at $p=3$ the error is $28\%$
  ($243$ vs $190$). `proof-family.md` §5.4 states this correctly. The two documents should
  be reconciled — as it stands `proof-lower.md` treats as an established law the very fit
  `proof-family.md` retracts.
* **§8** the Conjecture ($\mathcal I=c^{O(1)}$) directly contradicts route **(I)** stated
  three paragraphs above it ($2^{\Theta(\sqrt{|P|})}$ images are permitted, which with
  $|P|=\Theta(m^2)$ is $2^{\Theta(m)}$). Both may be defensible, but the document should
  say which it believes and why the arc-counting ceiling is not achievable.
* **§7 "the extremal family" — I resolved the censored cell, and it changes the
  conclusion.** $C_5$ = `def T 2 5 0 01 22 33 44 10 11110` was reported as censored
  ("3–5 GB, ten minutes — no answer"). It finishes: **$|FE(C_5)|_{\mathrm{msd}}=3846$** in
  238 s at `AM_CAP=100000`, `AM_MEM_MB=5000` (lsd on the same input was killed at the
  900 s timeout, so it is the msd ladder that gets there). Consequences:
  * Two engine runs at different forward caps ($2\cdot10^4$: 245 s, and $10^5$: 238 s) agree
    on $3846$. The independent Python reference builder `explore/fe_fast.py::fe_states_brz`
    **cannot** reach it — it blows a $3\cdot10^6$-subset cap in 72 s — so this number
    currently rests on the Rust engine's ladder alone, and $C_5$ is itself a good aspect-(B)
    stress case.
  * $C_5$ beats the 1200-class random sample ($2415$) by $1.6\times$, so the extremal-family
    claim **survives** at $m=5$ — good news for the document.
  * but the $m=5$ row of the §7 table should read $\ge3846$, not $\ge2415$, and the ratio
    column becomes $17.6,\ 4.36,\ \mathbf{3.34}$ instead of $17.6,\ 4.36,\ 2.10$. The
    corresponding local power-law exponents are $7.1,\ 5.1,\ \mathbf{5.4}$ — **no longer
    falling**. The $\log_2$ increments of $15,264,1152,3846$ are $4.14,\ 2.13,\ 1.74$.
  * §7's argument is "an exponential law requires a *constant* ratio; ours fall hard".
    With the corrected point the ratios fall much more slowly and the exponent ticks back
    up, so **the strongest empirical argument in `proof-lower.md` against an exponential
    family is materially weaker than stated**. Three ratios, one of which was wrong, is not
    enough; $C_6$ and $C_7$ are now the decisive measurements. $C_6$ msd
    (`def T 2 6 0 01 22 33 44 55 10 111110`) did not finish inside this review — it is the
    obvious next run, and given that $C_5$ took only 4 minutes it is probably within reach.
* **Fact 3** ($2^{8m^2}+1$): fine, though the "$+1$" is unnecessary and the dead state is
  already inside the subset construction.
* **§7 class counts are inconsistent.** Re-running the authors' own canonicalisation:
  admissible $(\delta,\tau)$ pairs / canonical classes are $8/4$ ($m{=}2$),
  $436/109$ ($m{=}3$), $53520/4460$ ($m{=}4$). The table reports $8$ and $436$ (pair
  counts) but $4460$ (class count) — three numbers, two different units.
* **§5's non-abelian comparison is generator-dependent.** Exactly one generating set per
  group is reported; $\mathbb Z_6$ at $k=3$ has several inequivalent choices of
  $(a_1,a_2)$ and the $991$ vs $473$ gap is not shown to be robust to them.

### Status
| claim | verdict |
|---|---|
| Facts 1–2, (2.1) cone decomposition, Lemma 1 (containment + monotonicity part) | **PROVED** |
| **Lemma 5 / Corollary 6** (abelian nesting $\Rightarrow$ $\mathcal I=\lvert G\rvert^{O(1)}$) | **PROVED** — but about $\mathcal I$, not about $\lvert FE\rvert$ |
| Lemma 7 (products) | **WRONG AS WRITTEN**; repairable via $m_i\le m(T)$ |
| Lemma 3 (alignment) | **HEURISTIC**, vacuous hypothesis, informal proof |
| Lemma 1 "image exactly $R_n(p_0)$" | **FALSE** (counterexample above); unused |
| "codings of group automata: dead, by proof" | **OVERCLAIM** — no bound on $\lvert FE_{T_p}\rvert$ exists |
| all measured tables (§3, §4, §5, §7) | **REPRODUCED** exactly |
| exhaustive maxima $264$ ($m{=}3$), $1152$ ($m{=}4$) | **TRUE**, and now actually exhaustive (this review) |
| interval-image Conjecture, §8 (N)/(S)/(I) | **PLAUSIBLE, UNPROVED**; and §8's Conjecture contradicts its own route (I) |
| the model's fidelity | **MIXED** — measured here: median within-cell Spearman $+0.6$ vs $\lvert FE\rvert$ on 385 random morphisms (good), but exactly zero resolution on the coding axis, which is the axis §5 studies (bad) |
| "no exponential family found" | **TRUE and honestly reported** |

---

## 4. What is actually PROVED across the three documents

**Proved, unconditionally, and new:**
1. $|FE_{G_p}|_{\mathrm{msd}}\ge p^3$ for all $p\ge3$ (family Thm 4.4) — with the
   $O(p^3)$ construction, $|FE_{G_p}|_{\mathrm{msd}}=\Theta(m^3)$. First infinite DFAO
   family with $|FE|$ pinned to a matching order, against a literature bound of $2^{9m^2}$.
2. Parity rigidity for $G_p$ (family Lemma 3.3): two positions of opposite parity have
   longest common extension $\le3$. Clean, and the engine of everything else in that file.
3. The complete characterisation of $FE_{G_p}$ by four numbers $(s(i)-s(j),\alpha,\beta,L)$
   (family Prop 3.4 + Cor 3.5) and of $FE_{T_p}$ by blocks (Prop 5.1).
4. The locality / head–middle–tail factorisation and the $\Lambda$ reduction
   (upper Lemmas 3.1, 5.1, Thms 4.4, 5.4) — correct, and Theorem 6.1's critical-exponent
   collapses are correct and genuinely explain a measured feature (long runs).
5. Lemma 5 / Cor 6 of `proof-lower.md`, as a theorem about interval images of abelian
   translation automata.

**Plausible, unproved:** $\Lambda=\mathrm{poly}(m)$; $|FE_{T_p}|$ polynomial (no upper bound
anywhere); the interval-image conjecture; the exact laws $p^3+8$ / $2p^2+3p+12$ for all $p$;
the exhaustive-maximum ratios as evidence against exponential growth — and *weakened* by
this review, since the corrected $m=5$ point ($C_5=3846$, not $\ge2415$) turns the ratio
sequence $17.6,4.36,2.10$ into $17.6,4.36,3.34$ and the local exponents $7.1,5.1,3.3$ into
$7.1,5.1,5.4$.

**Wrong / overclaimed (all fixable in prose):**
* family: "the lossy-coding lead is closed" — circular (level 0 = the original problem).
* family §4.1/§4.2: "unconditionally" attached to a 15-point polynomial fit.
* lower Lemma 7: implication stated backwards.
* lower §5 heading "dead, by proof", and the model credential in §2.
* lower Lemma 1's "exactly $R_n(p_0)$".
* lower §7: "exhaustive over all DFAOs" was binary-coding only (now repaired).

## 5. Open items I would prioritise

1. `proof-lower.md` §7's own recommendation is right: $C_5,C_6,C_7$. The extremal-family
   claim currently rests on three points and the $m=5$ sample champion is a *different*
   automaton (`0 02 33 41 24 10 / 00001`, $2415$, which I confirmed). **$C_5$ is now
   measured: $3846$ msd** (238 s, 5 GB, cap $10^5$; lsd killed at 900 s). It beats the
   sample champion, so the family claim survives — but it also flattens §7's ratio
   argument (see above). $C_6$, $C_7$ are the decisive next points.
2. $|FE_{T_p}|$ at $p=9,10,12$ — the only thing that separates $44p^3$ from $3p^4$ and hence
   decides whether the coding mechanism costs $O(1)$ or $\Theta(p)$. I retried: $p=8$
   reproduces ($11988$, 704 s, cap $5\cdot10^4$, 6 GB) but **$p=9$ still exits on the
   allocator budget at 6 GB** (`rc 3`), so G4's diagnosis is confirmed and the gap is real.
   It will need either a construction that exploits Prop 5.1 directly or a bigger ceiling —
   note that the *proved* Prop 5.1 gives an $O(1)$-per-block test, so a bespoke builder for
   this family looks far more promising than raising memory.
3. An upper bound of *any* kind for $T_p$. Until one exists, the phrase "the lead is closed"
   should not appear in any of the three files.

---

## Appendix — independent checks run for this review

All brute forces below were written from scratch for the review and evaluate $FE$ from its
definition (digit sums / raw comparison), never through the authors' characterisations.

| check | scope | result |
|---|---|---|
| family Prop 3.4 | $p=2..7$, $i,j,l<70$ (2 058 000 triples) | 0 mismatches |
| family Lemma 3.3 | $p=2..9$, $a,b<300$, opposite parity | 0 failures |
| family Cor 3.5 (3 clauses + $L$ formula) | $p=2..7$, $i,j<200$ | 0 failures |
| family Thm 4.4 (rebuilt prefixes + 3 suffix families) | $p=3..8$ | $27,64,125,216,343,512=p^3$ distinct response vectors; key $(D,\rho,\rho')$ recovered |
| family Prop 5.1 | $p=2..7$, $i,j,l<48$ | 0 mismatches |
| family exact laws (engine, msd/lsd) | $p=3,7,11,13,16$ | $35,351,1339,2205,4104=p^3{+}8$; $39,131,287,389,572=2p^2{+}3p{+}12$ |
| family §1 coding aside (engine) | $p=3..7$, all singleton targets | identical $|FE|$: $190,698,1877,3971,7243$ |
| upper: `proof-upper-check.py` | as shipped | 0 violations, all five examples |
| **new: upper checks on 5 adversarial DFAOs** | $C_4$ champion, exactly-2-ones, ruler $r{=}4$, $\mathbb Z_4$ faithful + singleton; $I,J,L<10$, $s\le4$ | 0 violations in every cell |
| **new: model predictiveness** | $\mathcal T$ vs $\lvert FE\rvert$, within-cell Spearman over `results/blowup.json` (385 morphisms) | $+0.34\dots+0.91$, median $+0.6$, positive in all 12 cells |
| **new: $C_5$ resolved** | msd, cap $10^5$ (238 s) and cap $2\cdot10^4$ (245 s) | **3846 states**, both runs agree; lsd killed at 900 s; Python reference builder caps out at $3\cdot10^6$ subsets — `proof-lower.md`'s censored cell |
| **new: $T_p$ retry** | $p=8,9$ msd, 6 GB, cap $5\cdot10^4$ | $p{=}8$: $11988$ (704 s); $p{=}9$: allocator budget exceeded (`rc 3`) — G4 confirmed open |
| lower §3 ruler family (engine) | $r=2..8$ | $8,34,113,305,712,1471,2751$ — exact match |
| lower §4 $D_m$ (engine) | $M=5..10$ | $71,113,171,241,319,405$ — exact match |
| lower §7 binary exhaust re-run (engine) | $m=3$, 109 classes | max $264$ — exact match |
| lower §7 champions (engine) | $m=4$, $m=5$ | $1152$, $2415$ — exact match |
| **new: non-binary exhaust** (engine) | $m=3$: 71 classes; $m=4$: 5329 classes, 0 failures | max $50$ / $492$ — binary codings still win, so $264$ and $1152$ are the true maxima over all output alphabets |
| lower §5 model coding-blindness | code inspection of `ii_states(k, trans)` | $\tau$ is not an argument; $\mathcal I,\mathcal T$ identical for $G_p$ and $T_p$ |
| lower §5 $\mathcal I(\mathbb Z_c)$ | $c=2..21$ | $3,7,15,31,61,113,197,325,511,\dots$ — exact match; tracker $\mathcal T=16,60,176,435,\dots\sim c^5$, vs $|FE_{G_p}|\sim p^3$ |
| lower §7 class bookkeeping | authors' canonicalisation re-run | pairs/classes $=8/4$, $436/109$, $53520/4460$ |

---

## Appendix B — relation to the target as stated

Khodier's Chapter 8 (thesis text, lines 2389–2400 of the `pdftotext` extract) asks for a
*characterization of the $k$-automatic sequences with exponential blowup*, says "currently
no such class of examples is known", and states the belief that the relationship
$|FE|$ vs $m$ **is** exponential. The $2^{9m^2}$ figure is attached to the $\forall u,v$
reformulation, not to the $\forall t$ form used throughout this repo.

Measured against that:

* **None of the three documents answers Open Problem 1(A).** `proof-lower.md` says so
  outright.
* The strongest contribution is in the *opposite* direction: `proof-family.md` exhibits the
  first infinite DFAO family whose $|FE|$ is pinned to a matching order — $\Theta(m^3)$,
  and exactly $m^3+8$ for $3\le m\le24$ — which is evidence against the stated belief for
  that class. I did not check the wider literature for prior exact families; the thesis
  itself claims none is known.
* Aspect (B) (construction strategy) is where this repo's real edge lies, and it is
  documented in `bench/README.md`, not in these three files.

---

## Appendix C — reproducing this review

    cd /Users/andrew/maths
    .venv/bin/python paper/proof-upper-check.py                 # 0 violations (2 min)

    # independent brute forces (no engine, no authors' code)
    .venv/bin/python paper/verdict-checks/fam_check.py          # family L3.3, P3.4, C3.5
    .venv/bin/python paper/verdict-checks/thm44.py              # family Thm 4.4 -> p^3
    .venv/bin/python paper/verdict-checks/prop51.py             # family Prop 5.1

    # exhaustive maximisation over NON-binary codings (the gap in fe_exhaust4.py)
    .venv/bin/python paper/verdict-checks/exh_nonbin.py 3       # max 50
    .venv/bin/python paper/verdict-checks/exh_nonbin.py 4       # max 492  (216 s, 5329 classes)

    # the censored extremal automaton, now resolved
    printf 'mode msd\ndef T 2 5 0 01 22 33 44 10 11110\nlet FE(i,j,l) A t. t < l => T[i+t] = T[j+t]\n'
    # via explore/engine.py, cap=100000, AM_MEM_MB=5000  ->  states=3846, 238 s

The four check scripts are committed under `paper/verdict-checks/`.
