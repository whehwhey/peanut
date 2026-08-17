# Referee verdict on `paper/proof3-lambda.md`, `proof3-singleton.md`, `proof3-levels.md`

Adversarial read, 2026-08-17, to the same standard as `paper/proof-verdict.md`. Every
load-bearing lemma was re-derived by hand; every finite claim was re-checked by a brute
force **written from scratch for this review** (`paper/verdict3-checks/`), never through
the authors' scripts and, where possible, never through the engine either. Where a
document reports an engine number I re-ran it from a fresh `def` line. Three mathematical
statements break, all three in the same file, plus one wrong claim about the engine.

## One-line verdict per file

* **`proof3-singleton.md` — CORRECT, AND THE BEST OF THE THREE.** Theorem 4.1
  ($|FE_{T_p}|_{\mathrm{msd}}\ge p^4$) is a genuine new unconditional theorem; I rebuilt
  the prefixes and all three suffix families from the text and evaluated $FE$ from digit
  sums, getting $p^4$ distinct response vectors with **0** collisions for $p=3..7$ *and*
  the stated behaviour of every individual suffix. Lemma 1.2, Prop 3.1 and Thm 2.4 all
  survive independent brute force; an independent from-scratch DFA construction returns
  $15$ and $190$. It settles the lower half of gap G4 and refutes `proof-family.md`
  §5.4(b). No error found. Two prose defects and an over-confident constant.
* **`proof3-lambda.md` — CORRECT; closes a route, opens none.** Every number reproduces
  exactly from an independent implementation of Definitions 4.1/5.2/5.3 of
  `proof-upper.md`: $\gamma=27,130,817$, $\Lambda=53,816,15583$, the $\prod(1+p_i)$
  intersections pairwise distinct, $|FE|=168,1479,2828$, and the control $\Lambda(\mathbb
  Z_{15})=6805$, $\Lambda(\mathbb Z_{21})=58396$. $\Lambda$ is genuinely superpolynomial.
  The one soft spot is §6: the $\Lambda^\ast$ table is an unconverged *sample* and I beat
  its $m=19$ entry by $1.5\times$ with a wider search.
* **`proof3-levels.md` — CORE THEOREM CORRECT, THREE STATEMENTS FALSE.** Theorem 1,
  Lemma 2, Theorem 3, Prop 5.2, the $\Lambda$ table of §6 and the coding exhaustion of
  §7.3 all reproduce exactly. But **Corollary 3.4's "in particular" is false** (engine
  counterexample: coding `001001` of $\mathbb Z_6$ gives $190<224=|FE_{G_6}|$),
  **Proposition 5.1 is false as written** (the "head absent when $r_0=0$" parenthetical
  drops a conjunct — 1003 counterexamples, smallest is Thue–Morse $i{=}177,j{=}188,l{=}2$),
  and **§6's "Cor. 5.5 is not applicable to any coding of $\mathbb Z_p$" is false**
  ($\Lambda=4$ for the *faithful* coding at every $p$, so Cor. 5.5 does apply there and
  yields $|FE_{G_p}|=O(m^8)$ unconditionally). All three are repairable in prose; the
  note's own conclusions survive all three repairs.

## Ranking

1. **`proof3-singleton.md`** — the only file with a new theorem that moves a measured
   quantity: $\Omega(m^4)$ for a concrete infinite family, up from $\Omega(m^3)$. Fully
   reconstructed and confirmed.
2. **`proof3-lambda.md`** — a new theorem too, but a negative one: it kills the only route
   in the repo to an unconditional polynomial bound. Impeccably verified; the proposed
   replacement $\Lambda^\ast$ is honest but its supporting numbers are soft.
3. **`proof3-levels.md`** — the most useful *survey* (two exhaustions, one over all
   codings of $\mathbb Z_p$, one over all zero-fixed $k{=}2$ DFAOs) wrapped around a
   ten-line theorem, and it carries all three of the false statements found in this
   review. Its headline resolution is by **vacuity**, which it says in §3.3 but not in the
   abstract.

---

## 0. The bottom line the task asks for

> **Is $|FE|=\mathrm{poly}(m)$ now PROVED?  No. No upper bound is proved anywhere in the
> three notes; the main route to one is now closed by theorem; and the proved lower bound
> has risen from $m^3$ to $m^4$.**

* **No upper bound of any kind** on $|FE_{T_p}|$, or on $|FE|$ in general, is proved
  anywhere in the three documents. The only new upper bound is `proof3-lambda.md`
  Theorem 6.2, $|FE_{\mathrm{msd}}|\le m^4+m^6+m^8\Lambda^\ast$, whose parameter
  $\Lambda^\ast$ has no proved bound better than $\Lambda^2\le2^{2m^3}$.
* **The main route is now provably dead.** `proof-upper.md` Cor. 5.5 ("$\Lambda=O(m^c)
  \Rightarrow|FE|=O(m^{8+2c})$") has *no unconditional instance*: `proof3-lambda.md`
  Thm 4.4 exhibits binary $2$-automatic sequences with
  $\Lambda\ge\prod_{i\le g}(1+p_i)=\exp((1+o(1))\sqrt{m\log m})$. I verified the
  construction, the two lemmas it rests on, and the conclusion head-on. Status of
  "$\Lambda=\mathrm{poly}(m)$" goes from OPEN to **FALSE**.
* **The proved lower bound rises.** `proof3-singleton.md` Thm 4.1 gives
  $|FE_{T_p}|_{\mathrm{msd}}\ge p^4=m^4$, superseding `proof-family.md`'s $\Omega(m^3)$.
  So any true polynomial law has exponent $\ge4$; with the measured
  $|FE_{T_p}|/p^4\approx3$ the honest reading of the whole $T_p$ line is
  $\Theta(m^4)$, **conjecturally**.
* **The level grading is neutral.** `proof3-levels.md` Thm 3 proves the level residuals
  form a chain, so the $k$-adic grading can neither manufacture an exponential nor bound
  one. `proof-family.md` §5.3's ceiling is true and vacuous (Cor. 3.3), which is exactly
  the circularity `proof-verdict.md` §2 called out; the "second unstated hole" (a product
  over levels) is closed.
* **One unconditional polynomial upper bound does exist, and none of the three notes says
  so.** I recomputed $\Lambda(\mathbb Z_p^{\text{faithful}})=4$, $\gamma=3$ for every
  $p=3..11$ — the same constant already printed in `proof-upper.md` §7.3's own table.
  Plugging $c=0$ into `proof-upper.md` Cor. 5.5 gives
  $|FE_{G_p}|_{\mathrm{msd}}\le m^4+m^6+16m^8=O(m^8)$ **unconditionally, by a general
  theorem** — the only instance in the repo where Theorem 5.4 produces a polynomial bound
  for an infinite family. It is far weaker than `proof-family.md`'s family-specific
  $\Theta(m^3)$, but it flatly contradicts `proof3-levels.md` §6's claim that Cor. 5.5 is
  inapplicable to any coding of $\mathbb Z_p$, and the inference is worth stating in
  `proof-upper.md` §7.

---

## 1. `proof3-lambda.md` — correct; the route is closed

### Re-derived by hand
* **Lemma 2.2 (pair form).** Split at $\varepsilon+y=N$; the $y\ge N-\varepsilon$ half
  gives $B_s(b)[0{:}\varepsilon]=B_s(u)[N{-}\varepsilon{:}N]$, whose comparison length is
  $N-h=\varepsilon$ with $h=N-\varepsilon$. Correct.
* **Lemma 2.4 (descent for $A$).** Chunk arithmetic checked: $h+y=(r+d)K+(h'+y')$ with
  $h'+y'<2K$, range $0\le y<(k-d)K-h'$ = chunks $0..k-2-d$ full plus chunk $k-1-d$
  truncated at $K-h'$. Correct.
* **Corollary 2.3** ($\gamma\le\alpha^2$, $\Lambda\le\Lambda_2^2$). Correct, though
  $\Lambda_2$ is defined loosely; not load-bearing.
* **Lemma 3.1 (quotient invariance).** $X\mapsto(\pi^3)^{-1}X$ is injective because $\pi$
  is onto, and $\pi^3(\mathcal T)$ runs over all subsets of $Q^3$. Correct, and it is what
  lets the construction skip a minimality proof.
* **Lemma 4.2 (blocks).** Correct; the odometer $x\mapsto2x+d$ gives
  $B_s((i,x))[y]=[x2^s+y\equiv0]$.
* **Lemma 4.3.** Re-derived both halves from Lemma 2.2. (I) is
  $[cN+y\equiv0]=[\varepsilon+y\equiv0]$ on $y<N-\varepsilon$, forced when
  $N-\varepsilon\ge p$; (II) is $[N+z\equiv0]=[cN+(N-\varepsilon)+z\equiv0]$ on
  $z<\varepsilon$, forced when $\varepsilon\ge p$. So (b)'s hypothesis
  $p\le\varepsilon\le N-p$ is exactly right. Correct.
* **Theorem 4.4.** The CRT step is sound: the moduli are distinct primes; the solution
  class has modulus dividing $P=\prod p_j$; $2^s>2P+2p_g$ makes
  $[p_g,2^s-p_g]$ longer than $2P$, so it contains a full class; $2$ is invertible mod
  $p_i$, so $(c_i+1)2^s\not\equiv c_i2^s$. Both branches of the case split are covered.
  **Correct.**
* **Corollary 4.5.** Re-derived: $m=(1+o(1))\tfrac12g^2\ln g\Rightarrow\ln m=(2+o(1))\ln g
  \Rightarrow g=(1+o(1))2\sqrt{m/\ln m}\Rightarrow p_g=(1+o(1))g\ln g=(1+o(1))\sqrt{m\ln m}$,
  and $\sum_{i\le g}\ln(1+p_i)\ge\theta(p_g)-\ln2=(1+o(1))p_g$. Correct. The table
  ($|Q|=5,11,19,31,45,63,83,107$; $\prod(1+p_i)=4,24,192,2304,32256,580608,\dots$;
  crossover with $m^3$ at $g=6$) is arithmetically right.
* **Theorem 6.2 (the $\Lambda^\ast$ repair).** Verbatim `proof-upper.md` Thm 5.4 with
  "$\le\Lambda\cdot\Lambda$ possible pairs" replaced by "$\Lambda^\ast$ pairs by
  definition". Correct, and strictly stronger.

### Independently machine-verified (my code, `paper/verdict3-checks/lam3*.py`)
| check | scope | result |
|---|---|---|
| Lemma 2.2 (blocks $\to$ triples, both sides recomputed) | 3 stock DFAOs + $T_3$, $T_{3,5}$; all $(s,\varepsilon)$, $s\le4$ | **0 / 245** |
| `proof-upper` Lemma 5.1 (descent) | 4 DFAOs, all $(s,\varepsilon)$, $s\le4$ | **0 / 214** |
| Lemma 4.2 | $g\le3$, all gadget states, $s\le8$ | **0 / 13 286** |
| Lemma 4.3(a) and (b) | $g\le3$, $3\le s\le8$, all $\varepsilon$, all $(i,c)$ | **0 / 13 104** each |
| $|Q|$ = reachable = **minimal** | $g=1,2,3$ | $5,11,19$ — the construction is minimal, so Lemma 3.1 is not even needed |
| $\gamma$, $\Lambda$ from Defs 5.2/5.3 | $g=1,2,3$ | $27/53$, $130/816$, $817/15583$ — **exact match** |
| **Theorem 4.4 head-on** | $g=1,2,3$ | $4/4$, $24/24$, $192/192$ intersections pairwise distinct |
| Remark 4.7 control $\mathbb Z_P$, $\delta(x,b)=2x{+}b$ | $P=9,15,21$ | $\Lambda=2224,6805,58396$ — **exact match** |
| $|FE|$ (engine, fresh `def`) | $g=1,2,3$ | $168$, $1479$, $2828$ — **exact match** |

### Weakest step 1 — §6's $\Lambda^\ast$ table is an unconverged sample, and I beat it
The table's headline ("$\Lambda^\ast$ is $38,259,297$ while $\Lambda$ is $53,816,15583$")
is the file's argument that the counterexample "does not touch $\Lambda^\ast$". Both
numbers are lower bounds obtained by sampling $(I,J,L)$. I re-sampled with prefixes
deliberately placed at and across the gadget boundaries $2^e+2^{e-i}$ ($5\le e\le40$,
$1\le i\le5$), $|I-J|\le80$, $L\le200$, plus $1.2\cdot10^5$ random prefixes at four scales:

| $g$ | $m$ | authors' $\Lambda^\ast$ | mine |
|---|---|---|---|
| 1 | 5 | 38 | **38** (exact agreement) |
| 2 | 11 | 259 | 213 (their small-range sweep is better here) |
| 3 | 19 | 297 | **448** |

So the $m=19$ entry is wrong by $1.5\times$ and the sequence $38,259,297$ — which reads as
"$\Lambda^\ast$ has plateaued" — is an artefact of the sampling. $38,259,448$ does not
plateau. The *qualitative* separation survives easily ($448\ll15583\ll2.4\cdot10^8$), and
§7 gap 2 already labels this measured-not-proved, but the table should carry $\ge$ signs
and the $g=3$ entry should be corrected.

### Weakest step 2 — small wording faults
* §6: "these are samples, not exhaustive counts, so they are **upper-bounded**
  observations of a quantity defined over all $(I,J,L)$" — backwards. A sample of the size
  of a set is a **lower** bound. (§7 gap 2 gets it right.)
* §5.2: "$|FE|$ tracks $m^3$ here" — $168,1479,2828$ against $m^3=125,1331,6859$; the last
  point is $0.41m^3$, and three points with $\log$–$\log$ slopes $2.75,\,2.53$ do not
  identify a cubic. It is fine as an observation but is stated a shade too firmly (the
  document does say so in gap 4).
* §1: "leaving $\Lambda=\mathrm{poly}(m)$ as the **sole** obstruction" — sole obstruction
  *along that route*, not in general.
* The $m=19$ value $|FE|=2828$ came from `learnfe` with `capped_lcp=103`. I reproduced it
  from a fresh `def` line with the same tool, so the number is confirmed but not
  cross-implementation; the same caveat the document states.

### Status
| claim | verdict |
|---|---|
| Lemma 2.2, Lemma 2.4, Cor 2.3, Lemma 3.1, Lemma 4.2, Lemma 4.3 | **PROVED** (hand proof correct + independently brute-forced) |
| **Thm 4.4: $\Lambda(T_{p_1..p_g})\ge\prod(1+p_i)$** | **PROVED** — independently reconstructed; verified head-on at $g\le3$ |
| **Cor 4.5: $\Lambda$ superpolynomial; $\Lambda=\mathrm{poly}(m)$ is FALSE** | **PROVED** |
| Thm 6.2 ($|FE|\le m^4+m^6+m^8\Lambda^\ast$) | **PROVED**, and strictly stronger than Thm 5.4 |
| $\gamma,\Lambda,|FE|$ of the family; $\Lambda(\mathbb Z_{15}),\Lambda(\mathbb Z_{21})$ | **MACHINE-VERIFIED**, reproduced exactly and independently |
| $\Lambda^\ast$ small on the family ($38,259,297$) | **MEASURED, AND THE $g{=}3$ NUMBER IS WRONG** — $\ge448$ (this review); the separation from $\Lambda$ survives |
| $\Lambda^\ast=\mathrm{poly}(m)$ in general | **OPEN** — correctly flagged, and admitted to be close to circular |
| $\Lambda=2^{\Omega(m)}$ (Remark 4.8) | **OPEN** |

---

## 2. `proof3-singleton.md` — the real theorem; no error found

### Re-derived by hand
* **Lemma 1.2.** Immediate and correct.
* **Lemma 2.3 / Theorem 2.4.** The split $t=\Theta2^S+\tau$, the carry indicators
  $c(\tau),c'(\tau)$ and (D1) give $s(i+t)=s(I+\Theta+c)+x(\tau)$; the clause becomes
  $[\sigma=1{-}x]=[\sigma'=1{-}y]$, i.e. $\Phi_\Sigma(1{-}x,1{-}y)$ with
  $\Sigma=\Sigma^<$ (all $\Theta<\Lambda$) or $\Sigma^=$ (the $\Theta=\Lambda$ term,
  present iff $\tau<l_{lo}$). Correct, and $\Sigma^<_{c,c'}=\Pi(I{+}c,I{+}c{+}\Lambda{-}1;
  (J{-}I)+(c'{-}c))$ checks out.
* **Proposition 3.1.** All four cases re-derived. Correct.
* **Theorem 4.1.** I checked every step. $x_{a,\rho}=0^{P-e-\rho-1}1^e01^\rho$ with
  $e=(a-\rho)\bmod p$ has $s\equiv a$, $\nu=\rho$ (the explicit $0$ makes the trailing run
  exact) and $e+\rho+1\le2p-1\le P$; $(a,\rho)\mapsto(A_0,A_1)$ is a bijection onto
  $\mathbb Z_p^2$. Family 1: row $1{-}A_0$ meets the rejected set in $p-1\ge2$ cells and
  every other row in exactly $1$, so the residual determines $(A_0,B_0)$ — **and this is
  the only place $p\ge3$ is used, sharply** ($|FE_{T_2}|=15<16$). Family 2: the four
  choices ($w$, $v$, $\nu$, $S$) are all satisfiable as claimed, $t=0$ is neutral by the
  choice of $S$ and $v$, the $i$-side crosses $2^S$ for every $t\ge1$ and the $j$-side
  never does, $s(n2^m+r)=s(n)+s(r)$, and $\{s(r):r<2^m\}=\{0..m\}$, giving
  $u\Theta_m\in L_p\iff A_1\notin V_m$. The separation argument via
  $m_1=\min\{m:A_1\in V_m\}=(1-A_1)\bmod p$ (with $\infty$ iff $A_1=2$) is correct, and
  the two keys share $(A_0,B_0)$ so the same suffix serves both. **Correct.**
* **Corollary 4.3.** Follows. `proof-family.md` §5.4 reading (b) is refuted.

### Independently machine-verified (`paper/verdict3-checks/sing3*.py`, `phichk.py`)
| check | scope | result |
|---|---|---|
| Lemma 1.2 | $p=2..7$, $i<60$, $j-i<60$, $l<40$ | **0 / 842 400** |
| Prop 3.1 | $p=2..6$, $u<40$, $v-u<25$, $\eta<40$ | **0 / 200 000** |
| Thm 2.4 ($\Phi$ refines Nerode) | $p=2,3,4$; $26\times26\times14$ prefixes, suffixes to length 3 | **0** classes with two residuals; $\Phi$-class count at $p{=}2$ is $104$, matching the paper |
| **Thm 4.1, rebuilt from the text, $FE$ from digit sums** | $p=3..7$ | $81,256,625,1296,2401$ distinct response vectors, **0 collisions**; and every Family-2/3 suffix asserted to compute $[A_1\notin V_m]$ / $[B_1\notin V_m]$ did so in **every** instance |
| **from-scratch DFA** (NFA for the negation with guessed $t$ and guessed msd carries $\to$ subset $\to$ complement $\to$ Moore) | $p=2,3$ | $15$, $190$ — matches the engine and the authors' $\Phi$-builder; $p=4$ exhausted memory in this review |
| Lemma 5.1 rigidity, **wider search** ($d$ odd up to $2^{p+3}$, i.e. $8\times$ the claimed maximum; $i<2^{22}$) | $p=2..7$ | $3,7,15,31,63,127=2^p-1$; maximiser $d=2^p-1$ and $i=249,1009,4065,16321=\mathrm{val}(1^{p+1}0^{p-2}1)$ — **exact match** |
| interval images $\{s(q)\bmod p\}$ (§3(a)) | $p=2..9$ | $3,7,15,31,61,113,197,325$ — **exact match**, 4th difference $2$ |

### Weakest step 1 — Lemma 5.1's $p=8,9,10$ rows are not evidence
The scan is "all odd $d\le1023$". The claimed maximiser is $d=2^p-1$, which at $p=10$ is
$1023$ — the last value scanned. So for $p=10$ the search could not have found a larger
$d$, and for $p=9$ it had one bit of headroom. The rows $p\le7$ are solid (I re-ran them
with $8\times$ the range); $p=8,9,10$ should be labelled as confirmations of attainment
only, not as maxima. The upper bound $l\le2^p-1$ is unproved, as the document says (H3).

### Weakest step 2 — Conjecture 4.4's constant is not supported
$|FE_{T_p}|/p^4=2.35,\,2.73,\,3.00,\,3.06,\,3.02,\,2.93$ for $p=3..8$: it rises to $p=6$
and then **falls**. "Flat at $\approx3$" is generous, and a $\Theta(p^4)$ sequence with a
negative lower-order part (the document's own explanation for the falling log-log slope)
would have a *monotonically rising* ratio. The exponent $4$ is proved from below; the
constant, and the matching $O(p^4)$, are not indicated by six points that turn over.

### Weakest step 3 — §3's $\#\Pi_p$ prose is self-contradictory
"$\#\Pi_p\ \ge\ 15,463,18512,\dots$ … the $p=3,4$ entries are stable under further range
increases (and reproduce $44,486$ after adding the empty set)". $463\ne44$: the first
number counts $\Pi$-sets, the second counts $\Phi_\Pi$-images, and the sentence reads as
though they are the same quantity. Two lines, two units.

### Other, smaller
* §3's headline "a polynomial bound on $\#\Pi_p$ gives the **first upper bound of any
  kind** for $T_p$" is right, but (3.1) is loose by a measured factor $\approx p^4$ (H5),
  so even a tight $\#\Pi_p$ would give $p^{O(1)}$ with a bad exponent. Fine as stated.
* §6's table of failed runs is honest and matches `proof-verdict.md`'s own $p=9$ failure.
* §7's "$O(p^c)$ construction as requested — NOT DELIVERED" is the correct self-assessment.

### Status
| claim | verdict |
|---|---|
| Lemma 1.2, Lemma 2.3, **Theorem 2.4**, Proposition 3.1 | **PROVED** (hand proof correct + independently brute-forced) |
| **Theorem 4.1: $\lvert FE_{T_p}\rvert_{\mathrm{msd}}\ge p^4$ for $p\ge3$** | **PROVED** — independently reconstructed from the text and confirmed from the definition of $FE$ for $p=3..7$ |
| **Corollary 4.3: $\lvert FE_{T_p}\rvert\ne O(p^3)$; `proof-family` §5.4(b) refuted** | **PROVED** |
| $\Phi$-DFA reproduces $15,190,698,1877,3971,7243$ | **MACHINE-VERIFIED**; I reproduced $15,190$ by a third, independent pipeline |
| $\#\Pi_p=p^{O(1)}$, hence $\lvert FE_{T_p}\rvert=p^{O(1)}$ | **OPEN** — cleanly reduced, and the 1-dimensional case is a theorem |
| $\lvert FE_{T_p}\rvert=\Theta(p^4)$ with $c\approx2.9$ (Conj 4.4) | **PLAUSIBLE for the exponent, UNSUPPORTED for the constant** |
| Lemma 5.1 ($=2^p-1$) | **MEASURED**; solid for $p\le7$ (re-run at $8\times$ range), **not evidence at $p=8,9,10$** (search boundary); upper bound unproved |
| $\lvert FE_{T_9}\rvert,\lvert FE_{T_{10}}\rvert$ | **NOT OBTAINED** — and, as the document correctly argues, no longer needed |

---

## 3. `proof3-levels.md` — right theorem, three false statements

### Re-derived by hand, and correct
* **Theorem 1.** $t=k^\kappa t'+c$, $k^\kappa i+t=k^\kappa(i+t')+c$, then (1.1). Correct.
* **Lemma 2.** Zero-fixedness makes $B_\kappa(q)$ a prefix of $B_{\kappa+1}(q)$; the
  recursion makes $\approx_{\kappa+1}$ a function of $\approx_\kappa$, so the chain is
  eventually constant after $\le m-2$ strict refinements and its limit is Myhill–Nerode.
  Correct, and the non-zero-fixed counterexample given is genuine.
* **Prop 2.3.** $\{s_2(y):y<2^\kappa\}=\{0..\kappa\}$, so $B_\kappa(\mu)$ and
  $\pi^{(\kappa)}(\mu)$ determine each other. Correct.
* **Theorem 3.** $k^\kappa(Xk^s+x')=Xk^{s+\kappa}+x'k^\kappa$, whose length-$(s+\kappa)$
  suffix is $w\mathbf 0^\kappa$. Hence (3.1); (1),(2),(3) follow, using
  $(T^{(\kappa)})^{(1)}=T^{(\kappa+1)}$ which I checked
  ($B_1^{T^{(\kappa)}}(q)=([\delta(q,d)]_{\approx_\kappa})_d$ and
  $B_{\kappa+1}(q)=B_\kappa(\delta(q,0))\cdots$). **Correct.**
* **Cor 3.2, Cor 3.3, Remark 3.1a.** Correct.
* **Theorem 4.** Given Lemma 2's chain, all atoms $u\approx_{\kappa'}v$ for fixed $(u,v)$
  are determined by $\mathrm{sep}(u,v)\in\{0..m{-}2\}\cup\{\infty\}$. Correct.
* **Prop 5.2.** The decomposition of $[r,k^\kappa)$ into $\{r\}$ and the $(t,c)$ aligned
  blocks is right. Correct.
* **§7.4's canonical form.** $\delta(\cdot,0)=\mathrm{id}$ forces $Q$ to be the forward
  orbit of $q_0$ under $\delta(\cdot,1)$, giving $m(2^m-2)$ non-constant DFAOs and
  $m(2^{m-1}-1)=2,9,28,75,186$ modulo complementation. Correct. "At $k=2$ every group
  automaton is $\mathbb Z_m$" is correct.

### Independently machine-verified (`paper/verdict3-checks/lev3.py`, `lamcod.py`, engine)
| check | scope | result |
|---|---|---|
| Theorem 1 | 97 DFAOs (7 singleton $\mathbb Z_p$; 60 random $k\in\{2,3\}$, $m\le6$; 30 random zero-fixed), $\kappa\le6$ | **0 / 17 430** |
| Lemma 2 nesting | same pool | zero-fixed **47/47**; non-zero-fixed **18/50** |
| **Theorem 3, identity (3.1)** | 25 DFAOs $\times$ 6 prefixes $\times$ $\kappa=1,2$, residuals to suffix length 2 | **0 / 300** |
| Prop 5.2 (head and tail forms) | 45 DFAOs, $\kappa\le4$, all $r$, all state pairs | **0 / 253 920** |
| §6 $\gamma$ and $\Lambda$ for singleton $\mathbb Z_p$ | $p=2..15$, my own orbit + closure | $3,7,13,23,37,\dots,343$ and $4,11,28,58,106,\dots,17361$ — **exact match**; $\gamma=2p^2-8p+13$ for $4\le p\le15$ |
| §7.3 at $p=4$ and $p=5$, **exhaustive over all $15$ resp. $52$ set partitions** (not just up to rotation) | engine | $p{=}4$: max $698$ at the four singleton codings, then $387,258,153,72$ (faithful), $15$, $1$. $p{=}5$: max $1877$ at the five singletons, then $1276$ (`00011`), $1118$ ($\chi_1$), $1070,880,838,833,769$ ($\chi_2$), $736,683$, $133$ (faithful), $1$. The singleton **is** the worst coding, and §7.2's $\chi_1,\chi_2$ values $1118,769$ reappear exactly |
| $|FE|$ spot checks (fresh `def`, engine) | $\mathbb Z_6$ faithful/singleton, $\mathbb Z_4$ faithful/`0011`/`0012` | $224$, $3971$, $72$, $258$, $387$ — **exact match** |
| §7.2 / §7.5 spot checks (engine) | $\mathbb Z_5$ $\chi_1$ `01222`, $\chi_2$ `01223`; $\mathbb Z_4$, $\mathbb Z_6$ singleton at $k=3$; $D_3=S_3$ at $k=3$ | $1118$, $769$, $158$, $991$, $473$ — **exact match** |

### Defect 1 (real) — **Corollary 3.4's "in particular" is false**
The corollary is proved under "$M$ zero-fixed **and minimal for** $T=\tau\circ M$", and
that is correct. But the displayed specialisation drops the hypothesis:

> *In particular, for $M=\mathbb Z_p$ and **any** coding $\pi$,
> $|FE_{\pi\circ G_p}|_{\mathrm{msd}}\ge|FE_{G_p}|_{\mathrm{msd}}\ge p^3$.*

Engine counterexamples, all at $p=6$ where $|FE_{G_6}|=224$:

| coding $\pi$ of $\mathbb Z_6$ | $\lvert FE_{\pi\circ G_6}\rvert_{\mathrm{msd}}$ |
|---|---|
| `001001` | **190** |
| `012012` | **35** |
| `010101` | **15** |
| `000000` | **1** |

Each is $<224$. The reason is exactly the one gap 4 gives: $\mathbb Z_6$ is not minimal for
these codings (they factor through $\mathbb Z_3$, $\mathbb Z_3$, $\mathbb Z_2$, the trivial
group). The abstract's bullet — "$|FE_{\tau\circ M}|\ge|FE_{\text{faithful}}|$ for
**every** coding $\tau$ of every zero-fixed minimal DFAO" — is fine, but the $\mathbb Z_p$
sentence and the "one line, every coding" claim beside it are not. And the document
*reports the number 190 itself* in §7.3 ("at $p=6$ the coding `001001` gives exactly
$190=|FE_{T_3}|$") four sections later, without noticing it contradicts Cor. 3.4.
**Repair:** state it as $|FE_T|\ge|FE_{\mathrm{faithful\ sequence\ of\ the\ minimal\
DFAO\ of\ }T}|$, or add "for every $\pi$ such that $\mathbb Z_p$ is minimal for
$\pi\circ G_p$". Nothing else in the note depends on the broken form.

### Defect 2 (real) — **Proposition 5.1 is false as written**
The parenthetical "*head absent when $r_0=0$*" deletes a conjunct: when $r_0=0$ the head
condition $B_\kappa(q_A)[0{:}]=B_\kappa(q_{A+d'})[0{:}]$ is the *full-block* condition
$q_A\approx_\kappa q_{A+d'}$, and the interior as written ranges over the **open**
interval $a\in(A,A_1)$, so block $A$ is covered by nothing.

Brute force over 45 DFAOs $\times$ 3 000 random $(i,d,l)$:

| reading | violations |
|---|---|
| literal (head absent when $r_0=0$, interior $(A,A_1)$) | **1003 / 135 000** |
| repaired (head always a conjunct; equivalently interior $[A,A_1)$ when $r_0=0$) | **0 / 135 000** |

Smallest counterexample: Thue–Morse ($k=2$, $\delta=[[0,1],[1,0]]$, $\tau=(0,1)$),
$i=177$, $j=188$, $l=2$. Here $d=11$ is odd so $\kappa=0$, $K=1$, $r_0=r_1=0$, $A=177$,
$A_1=179$: the literal statement checks only $a=178$ and never compares $T[177]$ with
$T[188]$. (Indeed at $\kappa=0$ the literal form always drops one of the $l$ positions.)
Note §7.1's machine-check table lists **Prop 5.2** but **not Prop 5.1** — this is precisely
the claim that was never checked. Theorem 4 is unaffected: the dropped atom is itself of
the form $u\approx_\kappa v$, which is what Theorem 4 needs.

### Defect 3 (real) — **§6 consequence 1 is false**
> "**`proof-upper` Cor. 5.5 is not applicable to any coding of $\mathbb Z_p$.**"

I computed $\Lambda$ for four codings of $\mathbb Z_p$ ($\delta(q,0)=q$, $\delta(q,1)=q+1$)
with my own orbit/closure code:

| coding | $p=3$ | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 |
|---|---|---|---|---|---|---|---|---|---|
| faithful | **4** | **4** | **4** | **4** | **4** | **4** | **4** | **4** | **4** |
| singleton | 11 | 28 | 58 | 106 | 181 | 300 | 496 | 834 | 1443 |
| 2-block $0^{p-2}11$ | 11 | 12 | 30 | 59 | 103 | 166 | 254 | 377 | 552 |

$\Lambda\equiv4$ for the faithful coding, so Cor. 5.5 applies with $c=0$ and gives
$|FE_{G_p}|=O(m^8)$ **unconditionally**. The two-block coding is also small and looks
polynomial. What is exponential is the **singleton** coding specifically, which is what
§6's own table is about. Worse, the $\Lambda=4$ row is printed in `proof-upper.md` §7.3
itself — the very table §6 sets out to correct — so this is not a subtle oversight but a
misreading of the source. **Repair:** "not applicable to the singleton coding of
$\mathbb Z_p$". This matters beyond wording: the corrected statement contains the only
place in the repo where `proof-upper.md` Thm 5.4 delivers a polynomial bound for an
infinite family, and §6 as written asserts the opposite.

### Weakest step — the headline resolves the objection by vacuity
"**Verdict: the product mechanism does not exist**" is true, and Theorem 3 proves it. But
the mechanism dissolves because Cor. 3.3 shows $\max_\kappa|FE_{T^{(\kappa)}}|=|FE_T|$,
i.e. `proof-family` §5.3's ceiling bounds $|FE_{T_p}|$ by itself — which is exactly
`proof-verdict.md`'s "weakest step 1" (the circularity), not a new resolution of the
"second unstated hole". §3.3 says this plainly ("`proof-family`'s grading argument is
vacuous, not wrong"); the abstract does not, and a reader of the abstract alone would
think a gap had been repaired rather than shown empty. Theorem 3 itself is a ten-line
observation (level $\kappa$ is the right quotient of level $0$ by $\mathbf 0^\kappa$);
correct and worth recording, but the note's framing oversells its weight.

### Smaller points
* **§7.5's encoding claim is wrong.** "the engine's digit-string `def` syntax tops out at
  $m\le10$ for this encoding" — the parser is `c as u8 - b'0'`
  (`engine/src/main.rs:36,41`), so `:`, `;`, `<`, `=`, … encode letters $10,11,12,\dots$.
  I used exactly this to run the $m=11$ and $m=19$ CRT automata through `def`+`learnfe`
  (getting $1479$ and $2828$), after a control on $\mathbb Z_6$ that returned $224$. So
  $D_6$ and the other $m>10$ rows of §7.5 were reachable; the missing rows are a
  resource/effort limit, not an encoding limit.
* **Definition 3.1** already writes $|FE_T|$ as the tuple count, so Theorem 3(3) refutes
  the equality with $\prod_\kappa n_\kappa$, not a hypothesis anyone would defend. The
  useful content is (2) and the chain, not the refutation.
* §7.3's "6 (all)" coding classes at $p=4$: there are 7 rotation classes of set partitions
  of $\mathbb Z_4$ (I enumerated all 15 partitions); presumably the constant coding was
  excluded. Cosmetic.
* Remark 2.3a ($O(m)$ distinct level problems), the $\Lambda^\ast$ table, §7.3–§7.5 are all
  measurements and are labelled as such.

### Status
| claim | verdict |
|---|---|
| **Theorem 1** (level realisation) | **PROVED** + independently machine-checked (0/17 430) |
| **Lemma 2** (nesting for zero-fixed; $\mathrm{md}\le m-2$) | **PROVED** + machine-checked; counterexample for non-zero-fixed is genuine |
| **Prop 2.3** ($\approx_\kappa$ = window coding; $=\chi_\kappa$ for the singleton) | **PROVED** |
| **Theorem 3** (levels are $\mathbf 0^\kappa$-derivatives; the partitions are a chain; no product) | **PROVED**, unconditionally, and independently machine-checked |
| **Cor 3.2** (monotone in $\kappa$), **Cor 3.3**, **Remark 3.1a** | **PROVED** |
| **Cor 3.4**, general form ($M$ zero-fixed *and minimal* for $\tau\circ M$) | **PROVED** |
| **Cor 3.4, the "in particular" for any coding of $\mathbb Z_p$** | **WRONG** — `001001` at $p=6$ gives $190<224$ (also `012012` $35$, `010101` $15$, constant $1$) |
| **Prop 5.1** as displayed ("head absent when $r_0=0$") | **WRONG** — 1003/135 000 counterexamples; repaired form is correct (0/135 000) |
| **Prop 5.2** (partial blocks) | **PROVED** + machine-checked (0/253 920) |
| **Theorem 4** (level axis is one threshold $\mathrm{sep}(u,v)$) | **PROVED** (unaffected by the Prop 5.1 defect) |
| §6: $\Lambda$ for singleton $\mathbb Z_p$ is $(\tfrac12+o(1))2^p$, refuting `proof-upper` §7.3's "$p^{2.7}$" | **MEASURED and reproduced exactly to $p=15$**; the refutation of the fit is certain, the asymptotic $\tfrac12 2^p$ is not proved (the *proved* superpolynomiality is `proof3-lambda` Thm 4.4) |
| §6 consequence 1: "Cor. 5.5 is not applicable to any coding of $\mathbb Z_p$" | **WRONG** — $\Lambda\equiv4$ for the faithful coding, giving $\lvert FE_{G_p}\rvert=O(m^8)$ unconditionally |
| §7.3 singleton is the worst coding ($p\le5$ all, $p=6,7$ 2-block) | **MEASURED**; I re-ran $p=4$ and $p=5$ exhaustively over all $15$ and $52$ set partitions — confirmed, with the same runners-up ($387$, $1276$) |
| §7.4 max over zero-fixed $k{=}2$ DFAOs, $m\le6$ | **MEASURED**, exhaustive over the class; canonical form correct |
| §7.5 "no candidate family is exponential" | **MEASURED**, and the $\chi_\kappa$-join collapse is a theorem (Lemma 2), not just data |
| §7.5 "`def` tops out at $m\le10$" | **WRONG** — the parser accepts letters $\ge10$ as `:`, `;`, …; I ran $m=19$ |
| upper bound on $\lvert FE_{T_p}\rvert$ | **STILL OPEN** |

---

## 4. What is actually PROVED across the three documents

**Proved, unconditionally, and new:**
1. $|FE_{T_p}|_{\mathrm{msd}}\ge p^4$ for all $p\ge3$ (`proof3-singleton` Thm 4.1). This is
   the first $\Omega(m^4)$ lower bound for an explicit infinite family anywhere in this
   repo, it settles the lower half of gap G4, and it refutes the cubic reading of
   `proof-family` §5.4(b).
2. $\Lambda(T)\ge\prod_{i\le g}(1+p_i)=\exp((1+o(1))\sqrt{m\log m})$ for an explicit family
   of **binary $2$-automatic** sequences (`proof3-lambda` Thm 4.4 + Cor 4.5). Hence
   $\Lambda=\mathrm{poly}(m)$ is **false**, and `proof-upper` Thm 5.4 has no unconditional
   polynomial instance.
3. $|FE_{\mathrm{msd}}(T)|\le m^4+m^6+m^8\Lambda^\ast(T)$ (`proof3-lambda` Thm 6.2),
   strictly stronger than Thm 5.4 and untouched by (2).
4. $R^{T^{(\kappa)}}_{I,J,L}=R^{T}_{I,J,L}/\mathbf 0^\kappa$; the level partitions of
   prefix space form a chain; the $k$-adic grading carries no product
   (`proof3-levels` Thm 3), with Cor 3.2 (monotonicity, previously reported as data) and
   Theorem 4 (the level axis is a single threshold per state pair) as corollaries.
5. The exact residual invariant $\Phi$ for $T_p$ (`proof3-singleton` Thm 2.4) and the
   2-adic recursion for $\Pi$ (Prop 3.1) — a correct explicit finite-state description,
   reproducing the engine at $p\le7$.
6. $\Lambda$ is a Myhill–Nerode quotient invariant (`proof3-lambda` Lemma 3.1) and
   $\mathrm{Sh}$ is the join of two binary alignment relations (Lemma 2.2).

**Found in this review, not in any of the three:** $\Lambda\equiv4$ for the faithful
coding of $\mathbb Z_p$ at every $p$ (the row is already in `proof-upper` §7.3; only the
inference is new), so `proof-upper` Cor. 5.5 gives $|FE_{G_p}|=O(m^8)$ unconditionally —
the only instance in the repo where Thm 5.4 delivers a polynomial bound for an infinite
family, and a direct refutation of `proof3-levels` §6 consequence 1.

**Plausible, unproved:** $\Lambda^\ast=\mathrm{poly}(m)$ (and hence any upper bound at all
for $T_p$); $\#\Pi_p=p^{O(1)}$; $|FE_{T_p}|=\Theta(p^4)$; $\Lambda=(\tfrac12+o(1))2^p$ for
the singleton $\mathbb Z_p$; the rigidity constant $2^p-1$; $\Lambda^\ast$ polynomial on
the CRT family (and its measured value at $m=19$ is $\ge448$, not $297$).

**Wrong / overclaimed (all fixable in prose):**
* levels Cor 3.4's "in particular, for any coding of $\mathbb Z_p$" — false, four
  counterexamples at $p=6$.
* levels Prop 5.1's "head absent when $r_0=0$" — false, 1003 counterexamples.
* levels §6 "Cor. 5.5 is not applicable to any coding of $\mathbb Z_p$" — false for the
  faithful coding.
* levels §7.5 "`def` tops out at $m\le10$" — false.
* lambda §6's $\Lambda^\ast$ table — an unconverged sample; $g=3$ is $\ge448$, not $297$;
  and "upper-bounded observations" should read "lower bounds".
* singleton Conjecture 4.4's constant, and Lemma 5.1's $p\ge8$ rows (search boundary).
* singleton §3's $\#\Pi_p$ / $\#\Phi_\Pi$ conflation.

---

## 5. Is $|FE|=\mathrm{poly}(m)$ proved?

**No.** Explicitly:

* There is **no upper bound**, polynomial or otherwise, on $|FE|$ for a general
  $k$-automatic sequence in these documents beyond what was already in the literature
  ($2^{9m^2}$) and in `proof-upper.md` ($2^{O(m^3)}$).
* There is **no upper bound at all** for $T_p$ — gap G3 of `proof-family.md` is untouched.
  Both notes that could have attacked it (`proof3-singleton` §3, `proof3-levels` §6/gap 3)
  reduce it to a counting question ($\#\Pi_p$, or $\Lambda^\ast$) and stop.
* The conditional bound `proof-upper` Cor. 5.5 is now known to have **no unconditional
  instance in general** (`proof3-lambda` Thm 4.4), although it does apply, with $c=0$, to
  the faithful coding of $\mathbb Z_p$ (this review), giving $|FE_{G_p}|=O(m^8)$.
* The proved **lower** bound has risen to $m^4$ (`proof3-singleton` Thm 4.1), so any true
  polynomial law has exponent $\ge4$. The best current guess, unproved in both directions
  for the general case, is $\Theta(m^4)$ for the $T_p$ family and no exponential family
  known anywhere.

If one wants the shortest honest summary: **the polynomial conjecture is still open; the
main route to it is now closed by theorem; the evidence for it is one family more, and one
exponent higher, than before.**

---

## 6. Open items I would prioritise

1. **$\Lambda^\ast$, exactly, on the CRT family.** My $448$ at $m=19$ shows the sampled
   table is not converged. Either prove `proof3-lambda` §6's in-region sketch (the
   straddling case is the hard half) or compute $\Lambda^\ast$ exhaustively over a
   certified-complete family of prefixes. Until then "the counterexample does not touch
   $\Lambda^\ast$" is a conjecture supported by a sample I already beat.
2. **The $\mathbb Z_p^2$ nesting lemma** (`proof3-singleton` H1). This is the single most
   valuable open step in the repo: it would give $\#\Pi_p=p^{O(1)}$, hence the first upper
   bound of any kind for $T_p$, and the one-dimensional case is already a theorem
   (`proof-lower` Cor. 6). Prop 3.1 supplies the recursion.
3. **$|FE|$ for the CRT family at $g=4$** ($m=31$) and $\Lambda$ at $g=4$. Three points
   ($168,1479,2828$) do not identify a growth law, and the family is the only place where
   $\Lambda$ and $|FE|$ are known to diverge sharply.
4. **Is $\Lambda$ ever $2^{\Omega(m)}$?** (`proof3-lambda` Rem 4.8.) The measured singleton
   $\mathbb Z_p$ says yes ($\Lambda\approx2^{p-1}$ to $p=18$) but nothing is proved beyond
   $\exp(\sqrt{m\log m})$. A proof would also settle whether the $\Lambda^\ast$ route can
   ever be more than bookkeeping.
5. **Fix the three false statements in `proof3-levels.md`** before any of it is quoted:
   Cor 3.4's "in particular", Prop 5.1's $r_0=0$ case, and §6's Cor. 5.5 claim (and drop
   the "`def` tops out at $m\le10$" remark in §7.5, which is also false). Add
   Prop 5.1 to the §7.1 machine-check table — it is the one proposition in that file that
   was never checked, and it is the one that is wrong.

---

## Appendix A — independent checks run for this review

All scripts are committed under `paper/verdict3-checks/` and were written from scratch:
blocks, $\mathrm{Sh}$, $\Phi_d$, $\gamma$, $\Lambda$, $\Lambda^\ast$, $\Pi$, $\Phi$, the
level codings, the residuals and the $T_p$ DFA are all reimplemented from the definitions.

| check | scope | result |
|---|---|---|
| lambda Lemma 2.2 | 5 DFAOs, all $(s,\varepsilon)$, $s\le4$ | 0 / 245 |
| `proof-upper` Lemma 5.1 (descent) | 4 DFAOs, $s\le4$ | 0 / 214 |
| lambda Lemma 4.2 | $g\le3$, $s\le8$ | 0 / 13 286 |
| lambda Lemma 4.3(a),(b) | $g\le3$, $3\le s\le8$, all $\varepsilon,i,c$ | 0 / 13 104 |
| lambda $\gamma$, $\Lambda$, minimality | $g=1,2,3$ | $27/53$, $130/816$, $817/15583$; $m=5,11,19$ minimal — exact match |
| **lambda Thm 4.4 head-on** | $g=1,2,3$ | $4/4$, $24/24$, $192/192$ distinct |
| lambda Rem 4.7 control $\mathbb Z_P$ | $P=9,15,21$ | $\Lambda=2224,6805,58396$ — exact match |
| **new: adversarial $\Lambda^\ast$** | CRT $g\le3$; boundary-straddling prefixes at scales $2^5..2^{40}$, $L\le200$, $1.2\cdot10^5$ random | $38$, $213$, **$448$** — the paper's $g{=}3$ entry ($297$) is beaten by $1.5\times$ |
| singleton Lemma 1.2 | $p=2..7$, $i<60$, $j{-}i<60$, $l<40$ | 0 / 842 400 |
| singleton Prop 3.1 | $p=2..6$ | 0 / 200 000 |
| singleton Thm 2.4 | $p=2,3,4$, $26^2\times14$ prefixes | 0 classes with two residuals; $104$ $\Phi$-classes at $p=2$ |
| **singleton Thm 4.1 rebuilt, $FE$ from digit sums** | $p=3..7$ | $81,256,625,1296,2401$ distinct, 0 collisions; every suffix-family assertion held |
| **new: from-scratch DFA for $L_p$** (NFA + subset + complement + Moore) | $p=2,3$ | $15$, $190$ |
| singleton Lemma 5.1, **8$\times$ wider $d$-range** | $p=2..7$, odd $d<2^{p+3}$, $i<2^{22}$ | $2^p-1$ with $d=2^p-1$, $i=\mathrm{val}(1^{p+1}0^{p-2}1)$ — exact match |
| singleton §3(a) interval images | $p=2..9$ | $3,7,15,31,61,113,197,325$ — exact match |
| levels Theorem 1 | 97 DFAOs, $\kappa\le6$ | 0 / 17 430 |
| levels Lemma 2 | same pool | zero-fixed 47/47, non-zero-fixed 18/50 |
| **levels Theorem 3, identity (3.1)** | 25 DFAOs, $\kappa=1,2$ | 0 / 300 |
| levels Prop 5.2 | 45 DFAOs, $\kappa\le4$ | 0 / 253 920 |
| **new: levels Prop 5.1 as written** | 45 DFAOs, 135 000 random $(i,d,l)$ | **1003 violations**; repaired form 0 |
| levels §6 $\gamma$, $\Lambda$ (singleton $\mathbb Z_p$) | $p=2..15$ | exact match; $\gamma=2p^2-8p+13$ |
| **new: $\Lambda$ across codings of $\mathbb Z_p$** | faithful / singleton / 2-block, $p=3..11$ | faithful $\Lambda\equiv4$ — refutes levels §6 consequence 1 |
| **new: levels Cor 3.4 counterexamples** (engine) | codings of $\mathbb Z_6$ | `001001` $190$, `012012` $35$, `010101` $15$, const $1$ — all $<224=\lvert FE_{G_6}\rvert$ |
| levels §7.3 at $p=4$, **all 15 partitions** (engine) | exhaustive | max $698$ (singleton), then $387,258,153,72,15,1$ — claim confirmed |
| lambda §5 $\lvert FE\rvert$ of the CRT family (engine, fresh `def`) | $g=1,2,3$ | $168$, $1479$, $2828$ — exact match |
| levels §7.3 at $p=5$, all 52 partitions (engine) | exhaustive | max $1877$ (singleton), 2nd $1276$ (`00011`), $\chi_1=1118$, $\chi_2=769$, faithful $133$ — exact match |
| levels §7.2 / §7.5 spot checks (engine) | $\mathbb Z_5\ \chi_1,\chi_2$; $\mathbb Z_4,\mathbb Z_6$ at $k=3$; $D_3$ at $k=3$ | $1118,769,158,991,473$ — exact match |
| engine `def` letters $\ge10$ | control $\mathbb Z_6$ $\to224$; CRT $m=11,19$ | works — refutes levels §7.5's encoding claim |

## Appendix B — reproducing this review

    cd /Users/andrew/maths
    .venv/bin/python paper/verdict3-checks/lam3.py        # Lemma 2.2, descent, minimality
    .venv/bin/python paper/verdict3-checks/lam3b.py       # Lemma 4.2, 4.3, gamma/Lambda, Thm 4.4
    .venv/bin/python paper/verdict3-checks/lam3c.py       # levels sec 6 Lambda table, p<=15
    .venv/bin/python paper/verdict3-checks/lamcod.py      # Lambda per coding of Z_p (faithful == 4)
    .venv/bin/python paper/verdict3-checks/lamstar.py     # adversarial Lambda* (448 at m=19)
    .venv/bin/python paper/verdict3-checks/sing3.py       # Lemma 1.2, Prop 3.1, Theorem 4.1
    .venv/bin/python paper/verdict3-checks/phichk.py      # Theorem 2.4
    .venv/bin/python paper/verdict3-checks/sing3dfa.py 4  # from-scratch DFA: 15, 190
    .venv/bin/python paper/verdict3-checks/misc3.py       # Remark 4.7 control + Lemma 5.1 wide scan
    .venv/bin/python paper/verdict3-checks/lev3.py        # Theorem 1, Lemma 2, Theorem 3, Prop 5.2
    .venv/bin/python paper/verdict3-checks/prop51chk.py   # Prop 5.1: 1003 violations as written
    .venv/bin/python paper/verdict3-checks/eng3.py        # engine: codings of Z_p (Cor 3.4 breaks)
    .venv/bin/python paper/verdict3-checks/eng3b.py       # engine: CRT family 168/1479/2828
    .venv/bin/python paper/verdict3-checks/codmax3.py     # engine: all partitions of Z_4, Z_5
    .venv/bin/python paper/verdict3-checks/eng3c.py       # engine: chi_1/chi_2, k=3 families, D_3
