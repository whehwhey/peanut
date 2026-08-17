# Can residuals multiply across $k$-adic levels?

*Target 1 (Khodier 2026, Open Problem 1). This note settles the sharpest open objection in
`paper/proof-verdict.md` §2, "the second unstated hole":*

> The claimed ceiling $O(p)\cdot\max_\kappa|FE_{\chi_\kappa\circ G_p}|$ assumes the levels
> *add*. In msd the automaton cannot know $\kappa=v_2(j-i)$ from a prefix — it is a low-bit
> quantity — so a residual may have to record behaviour at *every* level simultaneously,
> which permits a product $\prod_\kappa$, not a sum. Nothing in §5 excludes that, and a
> product of $p$ polynomials is exactly the shape an exponential family would have.

**Verdict: the product mechanism does not exist.**
In the *derivative* formulation of "level" (Def. 2.2) this is proved unconditionally for
every $k$-automatic sequence — not just for codings of $\mathbb Z_p$ — by Theorem 3. In the
*slice* formulation (partition the suffix space by $\kappa=v_k(j'-i')$) it is proved for
every **zero-fixed** DFAO by Theorem 4, which covers every group automaton and therefore the
entire class the objection is about; for non-zero-fixed DFAOs the slice formulation is closed
only by a measurement (Remark 2.3a: the number of *distinct* level problems is $O(m)$, not
$\Omega(m)$ independent ones).

The reason is neither of the two the referee guessed. It is not that the head/middle/tail
factorisation of `proof-upper` Thm 4.4 pins the level (it does not: $\Theta_{I,J,L}$ is
exactly the object that records all levels at once, and §6 below shows the associated
counting device $\Lambda$ really is **exponential** for the singleton coding of
$\mathbb Z_p$ — correcting `proof-upper` §7.3). It is not, in general, that the levels are
nested either (they are, but only for zero-fixed automata, Lemma 2). It is that

> **the level-$\kappa$ problem is a $\mathbf 0^\kappa$-derivative of the level-$0$
> problem** (Theorem 3). Consequently the partitions of prefix space induced by the levels
> form a *chain* under refinement, with level $0$ — the original predicate — finest. A
> chain has no product structure: the joint information over all levels equals the
> information at level $0$ alone.

**Companion notes written in parallel with this one** (both untracked at the time of
writing): `paper/proof3-lambda.md` independently proves that $\Lambda$ of `proof-upper`
Def. 5.3 is superpolynomial, via an explicit CRT family, and supplies the repair
$\Lambda^\ast$ (count only the pairs $(A^{+},A^{-})$ that actually occur);
`paper/proof3-singleton.md` proves $|FE_{T_p}|_{\mathrm{msd}}\ge p^4$. Both are used below
and both supersede parts of what this note found independently; where they do, it is said.

Four by-products:

* $|FE_{T^{(\kappa)}}|_{\mathrm{msd}}$ is **non-increasing in $\kappa$** and $\le|FE_T|$
  (Cor. 3.2) — this *proves* the monotone column of `proof-family` §5.4, which was
  reported there as data.
* $|FE_{\tau\circ M}|_{\mathrm{msd}}\ \ge\ |FE_{\text{faithful}}|_{\mathrm{msd}}$ for
  **every** coding $\tau$ of every zero-fixed minimal DFAO $M$ (Cor. 3.4) — a lossy coding
  never costs less than the identity coding. Specialised to $\mathbb Z_p$ this gives
  $|FE_{\pi\circ G_p}|\ge p^3$ for every $\pi$, which was the first lower bound for $T_p$
  when found (`proof-family` §5.5: "neither $\Omega(p^4)$ nor even $\Omega(p^3)$ is
  proved"); it is now **superseded for $T_p$** by `proof3-singleton.md` Thm 4.1
  ($\ge p^4$). What survives as new is the *uniformity*: one line, every coding, every
  zero-fixed automaton.
* **`proof-upper` §7.3 is wrong on its own example**: $\Lambda$ for the singleton coding of
  $\mathbb Z_p$ is $(\tfrac12+o(1))2^{p}$, measured to $p=18$, not "$p^{2.7}$" (§6). That
  $\Lambda$ is superpolynomial *somewhere* is `proof3-lambda.md`'s theorem, proved there
  by an explicit CRT construction; the point here is complementary and cheaper — the
  witness was already sitting in `proof-upper` §7.3's own table, one column past where the
  table stops. This is the one place where the referee's product *is* real — in the
  bookkeeping — and §6 measures how far it is from being realised
  (`proof3-lambda.md`'s $\Lambda^\ast$).
* Zero-fixed $k=2$ DFAOs have a **canonical form** with only $m(2^{m-1}-1)$ members, so the
  class in which the referee's mechanism lives can be enumerated exactly. Done for
  $m\le6$: the maximum of $|FE|$ is attained by $\mathbb Z_m$ with the singleton coding, at
  the maximal level depth $m-2$, and equals $15,190,698,1877,3971$ (§7.4). At $k=2$ every
  group automaton is $\mathbb Z_m$, so this settles the referee's class at $k=2$ by
  exhaustion.

**What is *not* settled.** No upper bound on $|FE_{T_p}|$ follows, and none is claimed;
gap G3 of `proof-family` stays open. What is closed is the *mechanism*: the grading by
$v_k(j-i)$ can never manufacture an exponential, so it is not a route to an answer to Open
Problem 1(A), in either direction. All the remaining freedom is at level $0$, i.e. in the
original problem.

---

## 1. Setting

Notation follows `paper/proof-upper.md` §1 exactly. Fix $k\ge2$; let $T$ be $k$-automatic
over a finite alphabet $\Delta$ with **minimal msd DFAO**
$M=(Q,\Sigma_k,\delta,q_0,\tau)$, $m=|Q|$, $\delta(q_0,0)=q_0$, and
$q_n:=\delta(q_0,\mathrm{rep}_k(n))$, so $T[n]=\tau(q_n)$.

**Blocks.** $B_\kappa(q):=\bigl(\tau(\delta(q,\mathrm{rep}_\kappa(y)))\bigr)_{0\le y<k^\kappa}
\in\Delta^{k^\kappa}$; $B_0(q)=\tau(q)$. Lemma 1.2 of `proof-upper`:
$$T[xk^\kappa+y]=B_\kappa(q_x)[y]\qquad(0\le y<k^\kappa). \tag{1.1}$$

**The predicate and its residuals.**
$FE(i,j,l)\iff\forall t<l:T[i+t]=T[j+t]$, read msd on three zero-padded tracks over
$\Sigma_k^3$. Writing $\mathbf 0:=(0,0,0)\in\Sigma_k^3$, the residual of the prefix
$(I,J,L)\in\mathbb N^3$ is
$$R^{T}_{I,J,L}:=\bigl\{(i',j',l')_s\ :\ s\ge0,\ 0\le i',j',l'<k^s,\
FE_T(Ik^s+i',\,Jk^s+j',\,Lk^s+l')\bigr\},$$
and (`proof-upper` (2.1)) $|FE_{\mathrm{msd}}(T)|=\#\{R^T_{I,J,L}\}$, the count of the
minimal **complete** DFA including the dead state (the empty residual).

**Zero-fixed automata.** Call $M$ *zero-fixed* if $\delta(q,0)=q$ for **every** $q\in Q$
(not merely $q=q_0$). Equivalently, the defining $k$-uniform morphism
$A\mapsto w_A$ satisfies $w_A[0]=A$ for every letter $A$ — it is prolongable at every
letter. Every *group automaton* $Q=H$, $\delta(h,c)=h\,a_c$ with $a_0=e$ (which
zero-stability forces) is zero-fixed. This is exactly the class the referee's objection is
about.

---

## 2. What a "level" is

$\kappa=v_k(j-i)$ is a property of the *low* digits of $j-i$, so in msd it is unknown to the
automaton until the very end of the word. The following makes the level-$\kappa$ subproblem
an object in its own right.

**Definition 2.1 (level equivalence and level coding).** For $\kappa\ge0$ put
$$q\approx_\kappa q'\ :\Longleftrightarrow\ B_\kappa(q)=B_\kappa(q'),\qquad
T^{(\kappa)}[n]:=[\,q_n\,]_{\approx_\kappa}.$$
$\approx_0=\ker\tau$, so $T^{(0)}=T$ (up to renaming output letters). Each $T^{(\kappa)}$ is
a **coding of the same DFAO $M$** — only the output map changes — so all the $T^{(\kappa)}$
have at most $m$ states.

Note the equivalences are defined by blocks of length *exactly* $k^\kappa$, **not** by the
Moore/Hopcroft refinement $\bigcap_{r\le\kappa}$. The distinction matters: for a general
DFAO $\approx_\kappa$ is not monotone in $\kappa$ (`proof-upper` §8.1 notes this), and an
earlier draft of Theorem 1 stated with the Moore chain is false — the machine check in §7
below caught it.

**Definition 2.2 (dilation).** $FE^{[\kappa]}_T(i,j,l):=FE_T(k^\kappa i,\ k^\kappa j,\ k^\kappa l)$.

If $v_k(j-i)=0$ then $v_k(k^\kappa j-k^\kappa i)=\kappa$ exactly, so
$\{FE^{[\kappa]}\}_{\kappa\ge0}$ sweeps the $k$-adic levels; $FE^{[\kappa]}$ is the
restriction of $FE_T$ to the $k^\kappa$-aligned configurations of level $\ge\kappa$.

### Theorem 1 (level realisation)

*For every $k$-automatic $T$, every $\kappa\ge0$ and all $i,j,l\ge0$,*
$$FE_T\bigl(k^\kappa i,\ k^\kappa j,\ k^\kappa l\bigr)\ \Longleftrightarrow\
\forall t<l:\ q_{i+t}\approx_\kappa q_{j+t}\ \Longleftrightarrow\
FE_{T^{(\kappa)}}(i,j,l).$$

*Proof.* Every $t<k^\kappa l$ is uniquely $t=k^\kappa t'+c$ with $t'<l$, $0\le c<k^\kappa$,
and then $k^\kappa i+t=k^\kappa(i+t')+c$. By (1.1),
$T[k^\kappa i+t]=B_\kappa(q_{i+t'})[c]$ and $T[k^\kappa j+t]=B_\kappa(q_{j+t'})[c]$.
So $FE_T(k^\kappa i,k^\kappa j,k^\kappa l)$ holds iff for every $t'<l$ the two length-$k^\kappa$
words $B_\kappa(q_{i+t'})$ and $B_\kappa(q_{j+t'})$ agree, i.e. iff
$q_{i+t'}\approx_\kappa q_{j+t'}$. The second equivalence is Definition 2.1. $\square$

**Machine check.** `0 violations / 140 516 tests` over 97 DFAOs (7 singleton codings of
$\mathbb Z_p$, $p\le8$; 60 random $k\in\{2,3\}$, $m\le6$; 30 random zero-fixed), $\kappa\le6$,
$l<15$, positions $<6000$. Script §9.

### Lemma 2 (nesting — zero-fixed automata only)

*Let $M$ be zero-fixed. Then $\approx_{\kappa+1}\ \subseteq\ \approx_{\kappa}$ for all
$\kappa$: the level equivalences form a **refining chain**. Writing $\mathrm{md}(M)$ for the
least $\kappa$ with $\approx_{\kappa+1}=\approx_\kappa$, we have $\mathrm{md}(M)\le m-1$, and
$\le m-2$ whenever $\tau$ is non-constant (if $\tau$ is constant then $FE\equiv\mathrm{true}$
and $|FE|=1$), and
$\approx_{\mathrm{md}}=\bigcap_\kappa\approx_\kappa$ is the Myhill–Nerode equivalence of
$M$. If $M$ is minimal for $T$, $\approx_{\mathrm{md}}$ is equality and
$T^{(\mathrm{md})}[n]=q_n$ is the **faithful** sequence.*

*Proof.* Lemma 1.3 of `proof-upper` gives
$B_{\kappa+1}(q)=B_\kappa(\delta(q,0))B_\kappa(\delta(q,1))\cdots B_\kappa(\delta(q,k-1))$.
Zero-fixedness makes the first factor $B_\kappa(q)$, so $B_\kappa(q)$ is a prefix of
$B_{\kappa+1}(q)$; hence $B_{\kappa+1}(q)=B_{\kappa+1}(q')\Rightarrow B_\kappa(q)=B_\kappa(q')$.
The recursion $q\approx_{\kappa+1}q'\iff\forall d\,(\delta(q,d)\approx_\kappa\delta(q',d))$
shows $\approx_{\kappa+1}$ is a function of $\approx_\kappa$, so once two consecutive terms
agree the chain is constant; a strictly refining chain of partitions of an $m$-set starting
from $\ge2$ classes has at most $m-2$ strict steps. The limit is the coarsest congruence
refining $\ker\tau$, i.e. Myhill–Nerode. $\square$

**Zero-fixedness is necessary.** Take $k=2$, $Q=\{0,1,2\}$, $\delta(0,\cdot)=(0,1)$,
$\delta(1,\cdot)=(2,2)$, $\delta(2,\cdot)=(2,2)$, $\tau=(0,1,0)$. Then
$\approx_0=\{\{0,2\},\{1\}\}$ but $B_1(0)=\tau(0)\tau(1)=01$, $B_1(1)=B_1(2)=00$, so
$\approx_1=\{\{0\},\{1,2\}\}$ — incomparable with $\approx_0$.
**Measured:** nesting held for **46/46** zero-fixed random DFAOs and only **21/51**
non-zero-fixed ones (§7).

**Remark 2.3a (how many distinct levels there are, in general).** $\approx_{\kappa+1}$ is a
function of $\approx_\kappa$ alone (the recursion in Lemma 2's proof), so
$\kappa\mapsto\ \approx_\kappa$ is the orbit of a deterministic map on the finite lattice of
partitions of $Q$: it is **eventually periodic**, with transient $+$ period
$\rho(M)\le B(m)$ a priori. For zero-fixed $M$ the orbit is a chain and $\rho\le m-1$
(Lemma 2). In general it is not a chain, but it is still short — sampled over
$3\cdot10^5$ random DFAOs per cell:

| $m$ | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|
| max $\rho$, $k=2$ | 2 | 4 | 8 | 14 | 17 | 24 | 22 |
| max period, $k=2$ | 1 | 1 | 4 | 6 | 6 | 6 | 6 |
| max $\rho$, $k=3$ | 2 | 4 | 8 | 12 | 12 | 11 | 10 |

($m\le4$ is exhaustive over all $m^{km}2^m$ DFAOs, not sampled, and gives the same maxima.)
So the number of *distinct* level problems is $O(m)$ in practice for every DFAO, not just
the zero-fixed ones — a second, independent reason why "$\prod$ over $\Omega(m)$ levels" has
nothing to multiply.

### Proposition 2.3 (identification with `proof-family` Prop. 5.3)

*Let $M=\mathbb Z_p$, $\delta(a,c)=a+c$, $k=2$, and let $\pi:\mathbb Z_p\to\Delta$ be any
coding, $T=\pi\circ G_p$. Then $q\approx_\kappa q'$ iff
$\pi^{(\kappa)}(q)=\pi^{(\kappa)}(q')$, where $\pi^{(\kappa)}(\mu):=(\pi(\mu),\pi(\mu+1),\dots,\pi(\mu+\kappa))$
is the length-$(\kappa+1)$ window coding. For the singleton coding $\pi=[\mu\equiv1]$ this is
exactly $\chi_{\min(\kappa,p-1)}$ of `proof-family` Prop. 5.3, with
$V_\kappa=\{1,0,-1,\dots,1-\kappa\}$.*

*Proof.* $B_\kappa(\mu)=(\pi(\mu+s_2(y)))_{y<2^\kappa}$ and $\{s_2(y):y<2^\kappa\}=\{0,\dots,\kappa\}$,
each value attained; so $B_\kappa(\mu)$ and $\pi^{(\kappa)}(\mu)$ determine each other.
For $\pi=[\mu\equiv1]$, $\pi^{(\kappa)}(\mu)$ records whether $\mu+c\equiv1$ for some
$0\le c\le\kappa$ and, if so, which $c$ — i.e. it is injective on
$V_\kappa=\{1-c\}_{c\le\kappa}$ and constant off it. $\square$

So Definition 2.1 is the general form of `proof-family`'s grading, for arbitrary DFAOs and
arbitrary codings, and it is stated *without* the "fully covered blocks" restriction the
referee flagged (Theorem 1 is an identity on the aligned family; see §5 for the general,
non-aligned, configurations).

---

## 3. The levels do not multiply

**Definition 3.1 (the product hypothesis, precisely).** The mechanism under suspicion is:
*there are $r=\Omega(m)$ levels; the residual must record one state per level; those states
are independent functions of the prefix; hence*
$$|FE_T|\ =\ \#\Bigl\{\bigl(R^{T^{(0)}}_{I,J,L},\ R^{T^{(1)}}_{I,J,L},\dots,
R^{T^{(r)}}_{I,J,L}\bigr)\ :\ (I,J,L)\Bigr\}\ =\ \prod_{\kappa=0}^{r} n_\kappa,
\qquad n_\kappa:=|FE_{T^{(\kappa)}}| .$$
With $n_\kappa\ge2$ and $r=\Omega(m)$ this is $2^{\Omega(m)}$. (For $M=\mathbb Z_p$ with the
singleton coding, Prop. 2.3 says $T^{(\kappa)}=\chi_\kappa\circ G_p$ and $r=p-2$, so this is
literally the referee's "$\prod_\kappa$ of $p$ polynomials".)

### Theorem 3 (levels are derivatives, not coordinates)

*For every $k$-automatic $T$, every prefix $(I,J,L)$ and every $\kappa\ge0$,*
$$R^{T^{(\kappa)}}_{I,J,L}\ =\ \bigl\{\,w\in(\Sigma_k^3)^*\ :\ w\,\mathbf 0^{\kappa}\in R^{T}_{I,J,L}\,\bigr\}
\tag{3.1}$$
*— the right quotient of the level-$0$ residual by $\mathbf 0^\kappa$. Consequently:*

1. *$R^{T}_{I,J,L}$ determines $R^{T^{(\kappa)}}_{I,J,L}$ for **all** $\kappa$ simultaneously;*
2. *$|FE_{T^{(\kappa)}}|_{\mathrm{msd}}\le|FE_{T}|_{\mathrm{msd}}$ for every $\kappa$;*
3. *letting $\Pi_\kappa$ be the partition of prefix space $\mathbb N^3$ induced by
   $(I,J,L)\mapsto R^{T^{(\kappa)}}_{I,J,L}$, the family $\{\Pi_\kappa\}_{\kappa\ge0}$ is a
   **chain**: $\Pi_0$ refines $\Pi_1$ refines $\Pi_2\cdots$. Hence*
   $$\#\Bigl\{\bigl(R^{T^{(0)}},R^{T^{(1)}},\dots\bigr)_{I,J,L}\Bigr\}\ =\ |\Pi_0|\ =\ |FE_T| ,$$
   *not $\prod_\kappa n_\kappa$. **The product hypothesis 3.1 is false for every $T$**: it can
   hold only in the degenerate case $n_\kappa=1$ for all $\kappa\ge1$, i.e. when every level
   above $0$ is trivial.*

*Proof.* By Theorem 1, $w=(i',j',l')_s\in R^{T^{(\kappa)}}_{I,J,L}$ iff
$FE_{T^{(\kappa)}}(Ik^s+i',Jk^s+j',Lk^s+l')$ iff
$FE_T\bigl(k^\kappa(Ik^s+i'),k^\kappa(Jk^s+j'),k^\kappa(Lk^s+l')\bigr)$.
Now $k^\kappa(Xk^s+x')=Xk^{s+\kappa}+x'k^\kappa$, whose $(s+\kappa)$-digit suffix after the
prefix $X$ is $\mathrm{rep}_s(x')\,0^\kappa$. Componentwise on the three tracks this is
$w\mathbf 0^\kappa$, giving (3.1).

(1) is immediate from (3.1). (2): (3.1) exhibits $R^{T^{(\kappa)}}_{I,J,L}$ as the image of
$R^{T}_{I,J,L}$ under a map that does not depend on $(I,J,L)$, so the number of values it
takes is at most the number of values of $R^T_{I,J,L}$; by `proof-upper` (2.1) those counts
are $|FE_{T^{(\kappa)}}|$ and $|FE_T|$. (Empty residual $\mapsto$ empty residual, so the
dead state is handled consistently.)

(3): $\Pi_\kappa$ is the partition into fibres of $(I,J,L)\mapsto R^{T^{(\kappa)}}$. By (1)
each such map factors through $(I,J,L)\mapsto R^T=R^{T^{(0)}}$, so every $\Pi_0$-block lies
inside a $\Pi_\kappa$-block; i.e. $\Pi_0$ refines $\Pi_\kappa$. Applying the same argument
to the sequence $T^{(\kappa)}$ in place of $T$ — legitimate because
$\bigl(T^{(\kappa)}\bigr)^{(1)}=T^{(\kappa+1)}$, since
$q\approx^{T^{(\kappa)}}_1q'\iff\forall d\,(\delta(q,d)\approx_\kappa\delta(q',d))
\iff q\approx_{\kappa+1}q'$ — gives that $\Pi_\kappa$ refines $\Pi_{\kappa+1}$. A meet of a
chain of partitions is its finest member, $\Pi_0$. $\square$

**Remark 3.1a (lsd).** The referee's objection is explicitly msd ("the automaton cannot
know $\kappa$ from a prefix"). In lsd it is even weaker. Multiplying by $k^\kappa$ prepends
$\kappa$ zero-triples to the lsd word, so the lsd residual of $FE_{T^{(\kappa)}}$ at prefix
$w$ equals the lsd residual of $FE_T$ at prefix $\mathbf 0^\kappa w$; the set of level-$\kappa$
residuals is literally a **subset** of the set of level-$0$ residuals, and
$|FE_{T^{(\kappa)}}|_{\mathrm{lsd}}\le|FE_T|_{\mathrm{lsd}}$ as well. Both digit orders are
covered; the msd/lsd asymmetry is irrelevant to the levels question.

### Corollary 3.2 (monotone level sizes — proves the `proof-family` §5.4 column)

*$|FE_{T^{(\kappa)}}|_{\mathrm{msd}}$ is non-increasing in $\kappa$, for every $T$ and every
$k$; no zero-fixedness needed.*

*Proof.* Theorem 3(2) applied to $T^{(\kappa)}$ with $\kappa=1$, using
$(T^{(\kappa)})^{(1)}=T^{(\kappa+1)}$. $\square$

`proof-family` §5.4 reports "the sizes decrease in $\kappa$" as an observation on four rows
of measured data. It is a theorem. **Machine check:** 26/26 DFAOs, engine, §7.

### Corollary 3.3 (the ceiling is an identity, and the referee's first objection subsumes the second)

*$\max_\kappa|FE_{T^{(\kappa)}}|=|FE_{T^{(0)}}|=|FE_T|$.*

So `proof-family` §5.3's ceiling "$O(p)\cdot\max_\kappa|FE_{\chi_\kappa\circ G_p}|$" is
**true** — the referee was right to doubt the *derivation* (the levels do not obviously
add), but the conclusion holds for a stronger reason than addition, namely
$\max_\kappa=\kappa{=}0$. It is also **completely vacuous**, since the maximum is the
quantity being bounded. This is exactly the circularity the referee identified as "weakest
step 1". The "second unstated hole" is therefore not a second hole: there is no product,
because a chain of partitions has no product structure. `proof-family`'s grading argument
is vacuous, not wrong.

### Corollary 3.4 (a proved lower bound for the singleton coding)

*Let $M$ be zero-fixed and minimal for $T=\tau\circ M$. Then*
$$|FE_{T}|_{\mathrm{msd}}\ \ge\ |FE_{\mathrm{faithful}}|_{\mathrm{msd}},$$
*where $\mathrm{faithful}$ is the sequence $n\mapsto q_n$ (identity coding of $M$). In
particular, for $M=\mathbb Z_p$ and any coding $\pi$,*
$$|FE_{\pi\circ G_p}|_{\mathrm{msd}}\ \ge\ |FE_{G_p}|_{\mathrm{msd}}\ \ge\ p^{3}\quad(p\ge3),
\qquad\text{and}\qquad |FE_{T_p}|_{\mathrm{msd}}\ \ge\ p^{3}+8\ \ (3\le p\le24).$$

*Proof.* Lemma 2 gives $T^{(\mathrm{md})}=\mathrm{faithful}$; Theorem 3(2) gives the
inequality. The $\mathbb Z_p$ instances use `proof-family` Thm 4.4 ($\ge p^3$, proved) and
its machine-verified exact law $p^3+8$ for $p\le24$. $\square$

`proof-family` §5.5 concludes "**neither $\Omega(p^4)$ nor even $\Omega(p^3)$ is proved here
for $T_p$**". Corollary 3.4 proves $\Omega(p^3)$, in one line, for **every** coding of
$\mathbb Z_p$ simultaneously — and, in the same line, for every coding of every zero-fixed
minimal DFAO. For $T_p$ specifically it is superseded by `proof3-singleton.md` Thm 4.1,
which proves $\ge p^4$ by an explicit fooling set; that argument is specific to $T_p$,
whereas Cor. 3.4 is uniform and free.

*Scope warning.* Cor. 3.4 compares a coding only with its **own Moore tower**. It is
tempting to read it as "finer codings are cheaper" in general; that reading is **false**.
Exhaustively over all codings of $\mathbb Z_4$ there are $31$ strict refinement pairs and
$7$ of them go the wrong way — e.g. `0012` refines `0011` but gives $387$ against $258$
(§7.3). So Cor. 3.4 is *consistent with* the referee's non-binary exhaustive finding that
finer codings lose ($50$ vs $264$ at $m=3$, $492$ vs $1152$ at $m=4$) but does not imply
it; those champions are not zero-fixed (`0 01 22 33 10 / 1110` has $\delta(1,0)=2$), so
Lemma 2 does not even apply to them.

### 3.5 Why the referee's two guesses were not the reason

* **"the head/middle/tail factorisation pins the level".** It does not. In `proof-upper`
  Thm 4.4 the middle condition is $[(s,e)\in\Theta_{I,J,L}]$ and $\Theta$ is a set of pairs
  $(s,e)$ ranging over *all* $e$, hence over all levels $v_k(e)$ at once. $\Theta$ is
  precisely the "record every level simultaneously" object, and §6 shows the device that
  counts it, $\Lambda$, is genuinely exponential. Theorem 4.4 is therefore neutral on this
  question.
* **"levels are nested".** They are, but only for zero-fixed automata (Lemma 2), and
  nesting alone would not settle it: a decreasing chain of $r$ languages drawn from a pool
  of $n$ still has $\binom{n}{r}$ possibilities. The decisive fact is stronger — each level
  is a *function* of the previous one (Theorem 3), so the whole tuple is determined by its
  first component.

---

## 4. What the theorem does and does not exclude

**Excluded.** Any argument of the form "there are $\Theta(m)$ levels, each carrying
$\ge2$ independent residual states, therefore $|FE|\ge c^{m}$". Any construction that tries
to build an exponential family by making the level-$\kappa$ behaviour an independently
tunable function of the prefix. The level index contributes **nothing**: all information at
levels $\ge1$ is a $\mathbf 0$-derivative of level $0$.

**Not excluded.** That $|FE_T|$ is exponential for some family — but if so, the witness must
already be visible at level $0$, i.e. in the ordinary residual count, and the grading gives
no help in exhibiting it. In particular:

* no upper bound on $|FE_{T_p}|$ follows (`proof-family` G3 open);
* lower bounds are unaffected: Cor. 3.4 gives $\Omega(p^3)$ for every coding uniformly, and
  `proof3-singleton.md` Thm 4.1 gives $\Omega(p^4)$ for $T_p$ by other means;
* for **non-zero-fixed** DFAOs the *slice* formulation of the objection (§5) is closed only
  by the measurement of Remark 2.3a, not by a theorem. Every group automaton is zero-fixed,
  so this gap is outside the class the referee's objection names.

---

## 5. The non-aligned configurations, and the threshold theorem

Theorem 3 is about the *dilation* family $\{(k^\kappa i,k^\kappa j,k^\kappa l)\}$. A sceptic can
reasonably read the referee's objection the other way: partition the suffix space by
$\kappa=v_k(j'-i')$ and ask whether the restrictions of the residual to those parts can be
independent. Level-$\kappa$ configurations that are not $k^\kappa$-aligned are not in the
dilation family, so Theorem 3 does not cover them. This section closes that reading too.

**Proposition 5.1 (general level decomposition).** *Let $i<j$, $l\ge1$, $d=j-i$,
$\kappa=v_k(d)$, $d'=d/k^\kappa$. Write $i=Ak^\kappa+r_0$ with $0\le r_0<k^\kappa$ and
$i+l=A_1k^\kappa+r_1$ with $0\le r_1<k^\kappa$. Then $FE_T(i,j,l)$ holds iff*
$$\underbrace{B_\kappa(q_{A})[r_0:]=B_\kappa(q_{A+d'})[r_0:]}_{\text{head}}
\ \wedge\ \underbrace{\forall a\in(A,A_1):\ q_a\approx_\kappa q_{a+d'}}_{\text{interior}}
\ \wedge\ \underbrace{B_\kappa(q_{A_1})[:r_1]=B_\kappa(q_{A_1+d'})[:r_1]}_{\text{tail}}$$
*(head absent when $r_0=0$, tail when $r_1=0$; if $A=A_1$ the two merge into the single
condition $B_\kappa(q_A)[r_0:r_1]=B_\kappa(q_{A+d'})[r_0:r_1]$).*

*Proof.* $n\in[ak^\kappa,(a+1)k^\kappa)$ has $n+d=(a+d')k^\kappa+(n-ak^\kappa)$, so by (1.1)
$T[n]=B_\kappa(q_a)[n-ak^\kappa]$ and $T[n+d]=B_\kappa(q_{a+d'})[n-ak^\kappa]$. Split
$[i,i+l)$ at the multiples of $k^\kappa$. $\square$

This is `proof-family` Prop. 5.1 with $\mathbb Z_p$ and the singleton coding stripped out.
Its interior is Theorem 1. Its head and tail are *partial*-block equalities — the part
`proof-verdict` correctly calls "outside the grading entirely". They are not, however,
outside the **chain**:

### Proposition 5.2 (partial blocks reduce to the same chain, at lower levels)

*Let $0\le r<k^\kappa$ have $\kappa$-digit base-$k$ expansion $r=r_1\cdots r_\kappa$. Then for
all $q,q'\in Q$*
$$B_\kappa(q)[r:]=B_\kappa(q')[r:]\iff
\tau\bigl(\delta(q,r)\bigr)=\tau\bigl(\delta(q',r)\bigr)\ \wedge
\bigwedge_{t=1}^{\kappa}\ \bigwedge_{c=r_t+1}^{k-1}\
\delta(q,w_{t,c})\ \approx_{\kappa-t}\ \delta(q',w_{t,c}),$$
$$B_\kappa(q)[:r]=B_\kappa(q')[:r]\iff
\bigwedge_{t=1}^{\kappa}\ \bigwedge_{c=0}^{r_t-1}\
\delta(q,w_{t,c})\ \approx_{\kappa-t}\ \delta(q',w_{t,c}),
\qquad w_{t,c}:=r_1\cdots r_{t-1}\,c .$$
*(Here $\delta(q,w)$ is the run of $M$ from $q$ on the digit string $w$, and
$\delta(q,r)$ means the run on the full string $r_1\cdots r_\kappa$.)*

*Proof.* $[r,k^\kappa)=\{r\}\ \sqcup\ \bigsqcup_{t=1}^{\kappa}\bigsqcup_{c>r_t}
\{y:\ y_1\cdots y_{t-1}=r_1\cdots r_{t-1},\ y_t=c\}$, and the $(t,c)$ piece is the aligned
block of size $k^{\kappa-t}$ rooted at $\delta(\cdot,w_{t,c})$; equality of $T$ on an aligned
block of size $k^{\kappa-t}$ is $\approx_{\kappa-t}$. Dually for $[0,r)$. $\square$

**Machine check.** $0$ violations $/\,600$ (300 random DFAOs, $k\in\{2,3\}$, $m\le5$,
$\kappa\le4$, random $r$, both head and tail forms). Script §9.

### Theorem 4 (the level axis is a threshold, not a vector)

*Let $M$ be zero-fixed and minimal. Every atom occurring anywhere in the level
decomposition of $FE_T$ — interior, head or tail, at any level — has the form
$u\approx_{\kappa'}v$ for a pair of states $u,v\in Q$ and some $\kappa'\ge0$ (Prop. 5.1 +
Prop. 5.2). By Lemma 2 the family $\{\approx_{\kappa'}\}_{\kappa'\ge0}$ is a chain, so the
truth values of **all** these atoms for a fixed pair $(u,v)$, across all levels
simultaneously, are determined by the single integer*
$$\mathrm{sep}(u,v):=\min\{\kappa'\ :\ u\not\approx_{\kappa'}v\}\ \in\
\{0,1,\dots,\mathrm{md}(M)\}\cup\{\infty\},\qquad \mathrm{md}(M)\le m-2 ,$$
*which takes at most $m$ values ($\infty$ iff $u=v$). Hence recording "behaviour at every
$k$-adic level simultaneously" for a state pair costs $\lceil\log_2 m\rceil$ bits, not $m$
bits: the level axis contributes a factor $\le m$ per state pair, never $c^{m}$.*

The remaining, genuinely large, axis is the **pair** axis: a residual may have to know
$\mathrm{sep}(u,v)$ for many pairs at once, i.e. a partial function $Q^2\to\{0,\dots,m-1\}$.
But that axis is *coding-free and level-free* — it is exactly the reachable-$(\mathcal T^+,
\mathcal T^-)$ bookkeeping of `proof-upper` Thm 4.4 — and §6 shows both that its a-priori
size is exponential and that the part of it actually realised is not. Whatever an
exponential family costs, it does not get it from the levels.

Theorem 4 is the quantitative form of the answer, and it is the one that matches the
referee's phrasing most directly. Theorem 3 is the stronger, hypothesis-free form: even
without nesting, the level partitions of prefix space are a chain and carry no product.

## 6. Where a product really does appear — and why it is bookkeeping

`proof-upper` §7.3 states:

> "$\Lambda$ is **constant** for the faithful family and grows like $p^{2.7}$ for the
> lossily coded family … So: no known family has superpolynomial $\Lambda$."

**That is false, and this is the sharpest correction in this note.** $\Lambda$ for the
singleton coding of $\mathbb Z_p$ is exponential.

Recomputed with `paper/proof-upper-check.py`'s own `orbit` and `Lambda`
(DFAO $\delta(q,0)=q$, $\delta(q,1)=q+1$, $\tau=[q{=}0]$, $k=2$, msd):

| $p$ | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| $\gamma=\lvert\mathcal S(T)\rvert$ | 3 | 7 | 13 | 23 | 37 | 55 | 77 | 103 | 133 | 167 | 205 | 247 | 293 | 343 | 397 | 455 | 517 |
| $\Lambda$ | 4 | 11 | 28 | 58 | 106 | 181 | 300 | 496 | 834 | 1443 | 2580 | 4758 | 9002 | 17361 | 33932 | 66908 | 132674 |
| $\Lambda/2^{p}$ | 1.00 | 1.38 | 1.75 | 1.81 | 1.66 | 1.41 | 1.17 | .969 | .814 | .705 | .630 | .581 | .549 | .530 | .518 | .510 | .506 |

$\gamma=2p^2-8p+13$ exactly for $4\le p\le18$ (quadratic, third difference $0$), but
$$\Lambda\ =\ \bigl(\tfrac12+o(1)\bigr)\,2^{p},$$
with $\Lambda/2^{p-1}=1.41,\,1.26,\,1.16,\,1.10,\,1.06,\,1.04,\,1.02,\,1.01$ for $p=11..18$ —
monotonically approaching $1$ from above. The *local* log–log slope of $\Lambda$ is
$3.26,\,3.31,\,3.47,\,3.78,\,4.27,\,4.93,\,5.75,\,6.68,\,7.65,\,8.61,\,9.52,\,10.4,\,11.2,\,12.0$
for $p=4{:}5,\dots,17{:}18$ — **rising steadily and without sign of a ceiling**, the signature of exponential growth.
`proof-upper`'s "$p^{2.7}$" is a global fit over $p\le9$, i.e. over exactly the range in which
$\Lambda/2^p$ is still *falling* and the exponential has not yet taken over. Its table stops one
column before the crossover.

Consequences:

1. **`proof-upper` Cor. 5.5 is not applicable to any coding of $\mathbb Z_p$.** The bound
   $m^4+m^6+m^8\Lambda^2$ evaluates to $\approx 4^{p}p^{8}$ for $T_p$, against a measured
   $|FE_{T_p}|$ of at most $\sim10^4$. `proof-upper` §7.3's closing sentence — "no known
   family has superpolynomial $\Lambda$, and by Theorem 5.4 no family with polynomial
   $\Lambda$ can have exponential $|FE|$" — must be withdrawn: a family with
   superpolynomial $\Lambda$ was already in the paper.
2. **$\Lambda$ is exactly the referee's product.** $\Lambda$ counts the intersection-closure
   of the principal up-sets $P_t$, $t\in Q^3$, inside $\mathcal S(T)$ — i.e. the number of
   *a priori conceivable* records "which $\mathrm{Sh}_{s,\varepsilon}$, over all $(s,\varepsilon)$
   and hence all levels, contain $\mathcal T$". It is exponential because a closure of
   $p^3$ subsets of a $2p^2$-element set can be. So the product-over-levels is real **in
   the bookkeeping**.
3. **It is not realised.** The residual only ever sees $\Theta_{I,J,L}$ for *reachable*
   $(\mathcal T^+,\mathcal T^-)$, i.e. sets of triples along the joint odometer trajectory
   $c\mapsto(q_{I+c},q_{J+c},q_{J+c+1})$. The count of pairs
   $(A^+,A^-)=(\bigcap_{t\in\mathcal T^+}P_t,\bigcap_{t\in\mathcal T^-}P_t)$ that actually
   occur is `proof3-lambda.md`'s $\Lambda^\ast$, and that note's Theorem 6.2 gives the
   repaired, still-unconditional bound $|FE_{\mathrm{msd}}|\le m^4+m^6+m^8\Lambda^\ast$.
   Measured here for the singleton $\mathbb Z_p$ (random $I,J<2^{60}$, $L\le60$,
   $\approx 10^6$ prefixes per $p$):

   | $p$ | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
   |---|---|---|---|---|---|---|---|---|---|---|
   | $\Lambda^\ast$ (sampled) | 29 | 84 | 170 | 283 | 400 | 515 | 665 | 779 | 875 | 937 |
   | a priori $\Lambda^2$ | 121 | 784 | 3364 | 11236 | 32761 | $9.0{\cdot}10^4$ | $2.5{\cdot}10^5$ | $7.0{\cdot}10^5$ | $2.1{\cdot}10^6$ | $6.7{\cdot}10^6$ |
   | $\lvert FE_{T_p}\rvert$ | 190 | 698 | 1877 | 3971 | 7243 | 11988 | — | — | — | — |

   $\Lambda^\ast$ grows polynomially (local log–log slope falling
   $3.70,3.16,2.79,2.24,1.89,2.17,1.50,1.22,0.79$ over $p=3{:}4,\dots,11{:}12$) while the a priori count grows like $4^p$. This is a *lower bound* on the
   realised count (sampling), so the polynomial behaviour is measured, not proved.

So §6 and §3 fit together: the product exists in the over-counting device $\Lambda$ (which
is level-blind and reachability-blind), and Theorem 3 says it can never be realised by
actual residuals.

---

## 7. Measurements

All engine runs: msd, `explore/engine.py`, `AM_CAP=50000`, `AM_MEM_MB` $\le4096$,
timeout $\le1200$ s.

### 7.1 Theorem 1 and Lemma 2 (brute force, no engine)

| check | scope | result |
|---|---|---|
| Theorem 1 | 97 DFAOs ($\mathbb Z_p$ singleton $p\le8$; 60 random $k\in\{2,3\},m\le6$; 30 random zero-fixed), $\kappa\le6$, $l<15$, positions $<6000$ | **0 violations / 140 516** |
| Lemma 2 nesting | same DFAOs, $\kappa\le6$ | **46/46** zero-fixed hold; **21/51** non-zero-fixed hold |
| Theorem 1 with the *Moore* ($\le\kappa$) chain instead of $\approx_\kappa$ | 47 DFAOs, same regime | **901 violations / 70 500** — the exact-$\kappa$ blocks are the right object, the Moore chain is not |
| Proposition 5.2 (partial blocks $=$ conjunction of aligned $\approx_{\kappa'}$ atoms), head and tail forms | 300 random DFAOs, $k\in\{2,3\}$, $m\le5$, $\kappa\le4$, random $r$ | **0 violations / 600** |

### 7.2 Corollary 3.2 (engine): $|FE_{T^{(\kappa)}}|$, $\kappa=0..5$

| DFAO | $\kappa=0$ | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| $\mathbb Z_3$ singleton | **190** | 35 | 35 | 35 | 35 | 35 |
| $\mathbb Z_4$ singleton | **698** | 387 | 72 | 72 | 72 | 72 |
| $\mathbb Z_5$ singleton | **1877** | 1118 | 769 | 133 | 133 | 133 |
| $\mathbb Z_6$ singleton | **3971** | 2607 | 1799 | 1453 | 224 | 224 |

The stabilised value is $p^3+8$ in every row ($35,72,133,224$) — Lemma 2's
$T^{(\mathrm{md})}=G_p$, confirmed. The $\kappa=1,2,3$ entries reproduce
`proof-family` §5.4's $\chi_\kappa$ table exactly, and $\mathbb Z_4$'s $\kappa=1$ value
$387$ is new. Over 26 DFAOs (4 above + 14 random + 8 random zero-fixed):
**monotone non-increasing in 26/26, $\le$ level $0$ in 26/26, 0 violations.**

### 7.3 Is the singleton the worst coding of $\mathbb Z_p$? (engine, exhaustive)

Exhaustive over all output partitions up to the cyclic symmetry $a\mapsto a+1$
($p\le5$: all partitions; $p=6,7$: all $2$-block partitions).

| $p$ | coding classes tested | $\max_\tau\lvert FE\rvert$ | argmax | 2nd best |
|---|---|---|---|---|
| 3 | 2 (all) | **190** | singleton `001` | `012` (faithful) 35 |
| 4 | 6 (all) | **698** | singleton `0001` | `0012` 387 |
| 5 | 11 (all) | **1877** | singleton `00001` | `00011` 1276 |
| 6 | 7 (2-block) | **3971** | singleton `000001` | `000011` 2949 |
| 7 | 9 (2-block) | **7243** | singleton `0000001` | `0000011` 5739 |

The **singleton coding maximises $|FE|$** over *all* codings for $p\le5$ and over all
$2$-block codings for $p=6,7$, so the $T_p$ row of `proof-family` §5.4 is the true worst case for this transition
structure and no cleverer coding does better. Its growth,
$$190,\ 698,\ 1877,\ 3971,\ 7243,\ 11988\qquad(p=3..8),$$
has $\log_2$ increments $1.88,\,1.43,\,1.08,\,0.87,\,0.73$ — **falling monotonically**.
An exponential family requires the increments to be bounded below by a positive constant;
here they are more than halving over five steps. (Two amusing degenerate rows: at $p=6$ the
coding `001001` gives exactly $190=|FE_{T_3}|$ and `010101` gives $15=|FE_{\mathrm{TM}}|$ —
periodic codings factor through the quotient group.)

**Refinement is *not* monotone in general.** Corollary 3.2's monotonicity is along the Moore
tower only, and that restriction is necessary: exhaustively over all $p=4$ codings there are
$31$ strict refinement pairs and **7 violate** $|FE_{\text{finer}}|\le|FE_{\text{coarser}}|$
— e.g. `0012` (classes $\{0,1\},\{2\},\{3\}$) refines `0011` ($\{0,1\},\{2,3\}$) but gives
$387>258$. At $p=5$: $255$ pairs, $0$ violations. So Theorem 3 is doing real work; "finer
codings are cheaper" is false as a general principle.

### 7.4 The extremal zero-fixed DFAO (exhaustive)

Zero-fixed $k=2$ DFAOs with all $m$ states reachable have a **canonical form**: reachability
from $q_0$ under $\delta(\cdot,0)=\mathrm{id}$ forces the states to be the forward orbit of
$q_0$ under $\delta(\cdot,1)$, so after relabelling
$$\delta(i,0)=i,\qquad \delta(i,1)=i+1\ (i<m-1),\qquad \delta(m-1,1)=j\in\{0,\dots,m-1\}.$$
There are therefore only $m\cdot(2^m-2)$ of them, and modulo output complementation
$m(2^{m-1}-1)$ — small enough to enumerate exactly. $j=0$ is $\mathbb Z_m$; the level tower
is deepest ($\mathrm{md}=m-2$) exactly in this class. This is the *entire* universe in which
Lemma 2's nesting holds and in which the referee's mechanism could live.

**Result (engine, exhaustive).** For every $m$ tested, the maximum is attained by
$j=0$ — i.e. by $\mathbb Z_m$ — with the **singleton** coding, and at the maximal possible
level depth $\mathrm{md}=m-2$:

| $m$ | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|
| zero-fixed DFAOs (mod complement) | 2 | 9 | 28 | 75 | 186 |
| $\max\lvert FE\rvert_{\mathrm{msd}}$ | **15** | **190** | **698** | **1877** | **3971** |
| argmax | $\mathbb Z_2$ sing. | $\mathbb Z_3$ sing. | $\mathbb Z_4$ sing. | $\mathbb Z_5$ sing. | $\mathbb Z_6$ sing. |
| $\mathrm{md}$ of argmax / $m-2$ | 0/0 | 1/1 | 2/2 | 3/3 | 4/4 |

Two consequences.

* **At $k=2$ every group automaton *is* $\mathbb Z_m$.** Zero-stability forces $a_0=e$, and
  reachability of all states from $q_0=e$ forces $\langle a_1\rangle=H$, so $H$ is cyclic
  and $a_1$ a generator. Non-abelian group automata require $k\ge3$ (as `proof-lower` §5
  notes). Hence §7.3 plus this table **exhaust the referee's mechanism at $k=2$**: the
  extremal object is $T_m$, and its growth is the falling-increment sequence
  $15,190,698,1877,3971,7243,11988$.
* The zero-fixed class is strictly larger than the group automata ($j\ne0$ gives
  $\rho$-shaped automata with a tail), and the group automaton still wins. So the extra
  freedom of a tail buys nothing, and the deepest level tower is also the largest $|FE|$ —
  which is exactly what one would expect if levels *did* multiply. They do not: the growth
  is polynomial anyway, as Theorem 3 requires.

### 7.5 Candidate families designed to make levels multiply

The four candidate structures named in `proof-verdict` §5, all with the singleton coding
(the worst coding, by §7.3). Non-abelian group automata need $k\ge3$; the $k=3$ cyclic and
$\mathbb Z_2\times\mathbb Z_n$ rows are the controls.

| family | $m$ | 4 | 6 | 8 | 10 | 12 |
|---|---|---|---|---|---|---|
| $\mathbb Z_m$, $k=3$ (abelian, one counter) | | 158 | 991 | 3283 | timeout | — |
| $\mathbb Z_2\times\mathbb Z_{m/2}$, $k=3$ (**two independent counters**) | | 196 | 979 | 2902 | timeout | — |
| $D_{m/2}$, $k=3$ (**non-abelian**; $D_3=S_3$) | | — | 473 | 904 | 1493 | n/a |
| $\mathbb Z_{2^r}$, $k=2$ (**$2$-group, deepest tower**) | | 698 | — | 11988 | — | — |

($\mathbb Z_4,\mathbb Z_6,\mathbb Z_8,D_3,D_4,D_5$ reproduce `proof-lower` §5's table exactly:
$158,991,3283,473,904,1493$. $D_6$ was not run — the engine's digit-string `def` syntax
tops out at $m\le10$ for this encoding. The two $m=10$ timeouts are at $600$–$700$ s,
$4$ GB.)

Local log–log slopes in $m$:

* $\mathbb Z_m$ at $k=3$: $\ 4.53,\ 4.16$ — falling;
* $\mathbb Z_2\times\mathbb Z_{m/2}$: $\ 3.97,\ 3.78$ — falling, and **below** the single
  cyclic counter at $m=8$ ($2902<3283$). Making the group a product of two independent
  counters — the most natural way to try to give each level its own register — *reduces*
  $|FE|$;
* $D_{m/2}$: $\ 2.25,\ 2.25$ — flat at $\approx2$, i.e. quadratic, and every value is far
  below the abelian group of the same order ($473$ vs $991$, $904$ vs $3283$). Non-abelian
  is worse, confirming `proof-lower` §5;
* $\mathbb Z_{2^r}$ at $k=2$: $15,698,11988$ at $m=2,4,8$, i.e. $\log_2$ increments per
  doubling of $m$ of $5.54$ then $4.10$ — falling. This is the sub-family of $T_p$ with the
  deepest possible level tower relative to $\log m$, and it grows no faster than the rest.

An exponential family needs $\log|FE|$ to grow **linearly in $m$** with a positive slope.
In every row here $\log|FE|/\log m$ is flat or falling, i.e. every family is polynomial in
$m$ with a *decreasing* apparent degree — the opposite signature. No candidate produced a
constant $\log_2|FE|$ increment per unit $m$, which is what an exponential family requires.

**Staircase codings combined.** The referee's last suggestion — combine the $\chi_\kappa$ —
**collapses by Lemma 2**. The $\chi_\kappa$ are a *chain*, so their join is simply the
finest one, and the engine confirms the identity to the digit:

| join | coding | $\lvert FE\rvert$ | equals |
|---|---|---|---|
| $\mathbb Z_5$: $\chi_0\vee\chi_1$ | `01222` | 1118 | $\lvert FE_{\chi_1}\rvert=1118$ |
| $\mathbb Z_5$: $\chi_0\vee\chi_2$, $\chi_1\vee\chi_2$ | `01223` | 769 | $\lvert FE_{\chi_2}\rvert=769$ |
| $\mathbb Z_6$: $\chi_0\vee\chi_1$ | `012222` | 2607 | $\lvert FE_{\chi_1}\rvert=2607$ |
| $\mathbb Z_6$: $\chi_0\vee\chi_2$, $\chi_1\vee\chi_2$ | `012223` | 1799 | $\lvert FE_{\chi_2}\rvert=1799$ |
| $\mathbb Z_7$: $\chi_0\vee\chi_1$ | `0122222` | 5128 | $\lvert FE_{\chi_1}\rvert=5128$ |
| $\mathbb Z_7$: $\chi_0\vee\chi_2$, $\chi_1\vee\chi_2$ | `0122223` | 3765 | $\lvert FE_{\chi_2}\rvert=3765$ |

Every join equals the finer of its two factors. Combining levels is not a construction;
it is a no-op. This is the referee's mechanism, tried directly, returning nothing.

---

## 8. Verdict

| claim | status |
|---|---|
| **Theorem 1** (level realisation: $FE_T(k^\kappa i,k^\kappa j,k^\kappa l)=FE_{T^{(\kappa)}}(i,j,l)$) | **PROVED** + machine-checked (0/140 516) |
| **Lemma 2** (level equivalences nest for zero-fixed DFAOs; $\mathrm{md}\le m-2$; $T^{(\mathrm{md})}$ faithful) | **PROVED** + machine-checked; counterexample given for non-zero-fixed |
| **Prop. 2.3** ($\approx_\kappa=\ker\pi^{(\kappa)}$ for codings of $\mathbb Z_p$; $=\chi_\kappa$ for the singleton) | **PROVED** |
| **Theorem 3** ($R^{T^{(\kappa)}}=R^T/\mathbf 0^\kappa$; level partitions form a chain; **no product over levels**) | **PROVED**, unconditionally, for every $k$-automatic $T$ |
| **Cor. 3.2** ($\lvert FE_{T^{(\kappa)}}\rvert$ non-increasing in $\kappa$) | **PROVED** + machine-checked (26/26) |
| **Remark 3.1a** (the same containment holds in lsd) | **PROVED** |
| **Cor. 3.4** ($\lvert FE_{\pi\circ G_p}\rvert\ge\lvert FE_{G_p}\rvert\ge p^3$; $\ge p^3+8$ for $p\le24$) | **PROVED** (modulo `proof-family` Thm 4.4, which `proof-verdict` independently confirmed) |
| **Prop. 5.1** (general level decomposition: head $+$ interior $+$ tail) | **PROVED** |
| **Prop. 5.2** (partial blocks are conjunctions of aligned $\approx_{\kappa'}$ atoms at *lower* levels) | **PROVED** + machine-checked (0/600) |
| **Theorem 4** (all level information about a state pair is one threshold $\mathrm{sep}(u,v)\le m-1$; zero-fixed minimal $M$) | **PROVED** |
| general "finer coding $\Rightarrow$ smaller $\lvert FE\rvert$" (outside the Moore tower) | **FALSE** — 7 counterexamples among the 31 refinement pairs at $p=4$ |
| `proof-upper` §7.3 "$\Lambda\sim p^{2.7}$, no family has superpolynomial $\Lambda$" | **REFUTED** — $\Lambda=(\tfrac12+o(1))2^p$, measured to $p=18$ |
| $\Lambda^\ast$ polynomial while $\Lambda^2$ is $4^p$, for singleton $\mathbb Z_p$ | **MEASURED** (sampled), not proved; $\Lambda^\ast$ and its repaired bound are `proof3-lambda.md` Def. 6.1 / Thm 6.2 |
| $\max_\tau\lvert FE_{\tau\circ\mathbb Z_p}\rvert$ attained at the singleton coding ($p\le7$) | **MEASURED**, exhaustive over codings |
| $\max\lvert FE\rvert$ over **all** zero-fixed $k=2$ DFAOs $=\lvert FE_{T_m}\rvert$, $m\le6$ | **MEASURED**, exhaustive over the class |
| $\mathbb Z_p\times\mathbb Z_q$, $\mathbb Z_{2^r}$, $D_n$/$S_3$, joins of $\chi_\kappa$: any exponential growth | **NO** — all measured polynomial with falling apparent degree; joins of $\chi_\kappa$ collapse to the finer factor |
| upper bound on $\lvert FE_{T_p}\rvert$ | **STILL OPEN** (`proof-family` G3) |
| $\Omega(p^4)$ for $T_p$ | **PROVED ELSEWHERE** — `proof3-singleton.md` Thm 4.1; Cor. 3.4 here gives only $\Omega(p^3)$, but for every coding of every zero-fixed $M$ |

**Honest gaps.**

1. Theorem 3 kills the mechanism, not the possibility. It says the grading by $v_k(j-i)$
   contributes no multiplicity; it says nothing about how large level $0$ can be. An
   exponential family, if one exists, must be exhibited at level $0$.
2. Proposition 5.1's head and tail partial blocks are genuinely outside Theorem 1. The
   argument in §5 that they cannot multiply is a counting observation (one boundary datum
   per configuration, not one per level), not a bound on their cost.
3. §6's $\Lambda^\ast$ table is sampled over $\approx10^6$ prefixes with $I,J<2^{60}$ and
   $L\le60$; larger $L$ can only add classes. It is an observed plateau, not a proof that
   $\Lambda^\ast$ is polynomial. Proving $\Lambda^\ast(T_p)=p^{O(1)}$ would give the first
   upper bound for $T_p$ via `proof3-lambda.md` Thm 6.2 — and note that
   $(\mathcal T^+,\mathcal T^-)$ is **coding-independent** (it depends on $\delta$ and
   $(I,J,L)$ only), so such a bound would cover *every* coding of $\mathbb Z_p$ at once.
   This is the most promising remaining route to G3 that this note found, and it is the
   same target `proof3-lambda.md` §7 names.
4. Cor. 3.4 requires $M$ zero-fixed **and** minimal for $\tau\circ M$. If $\tau$ is so lossy
   that $M$ is not minimal for $\tau\circ M$, the bound is against the faithful sequence of
   the minimal quotient instead.
5. §7.4's exhaustion of the zero-fixed class reaches $m\le6$ only; $m=7$ (448 automata,
   $\mathbb Z_7$ alone costs $143$ s) did not finish inside this session. §7.3's coding
   exhaustion is complete for $p\le5$ and $2$-block-only for $p=6,7$. §7.5 has two $m=10$
   timeouts and no $D_6$ (an encoding limit of the engine's `def` digit-string syntax at
   $m>10$, not a resource limit).
6. Remark 2.3a's $O(m)$ bound on the number of distinct level problems is *sampled*
   ($3\cdot10^5$ random DFAOs per cell, exhaustive only for $m\le4$). A proof that the
   partition-refinement orbit has length $\mathrm{poly}(m)$ for every DFAO would upgrade
   Theorem 4 to the non-zero-fixed case. I did not attempt it.

## 9. Reproduction

    cd /Users/andrew/maths
    # Theorem 1 + Lemma 2, brute force, no engine (~1 min)
    .venv/bin/python paper/verdict-checks/levels_check.py
    # Proposition 5.2, brute force (~5 s)
    .venv/bin/python paper/verdict-checks/levels_boundary.py
    # Corollary 3.2, engine (~10 min)
    .venv/bin/python paper/verdict-checks/levels_propP.py
    # Remark 2.3a: how many distinct level equivalences (~3 min)
    .venv/bin/python paper/verdict-checks/levels_orbit.py
    # Lambda / gamma table of section 6 (~1 min to p=16; p=17,18 add ~6 min)
    .venv/bin/python paper/verdict-checks/levels_lambda.py
    # realised (A+,A-) counts of section 6 (~1 min)
    .venv/bin/python paper/verdict-checks/levels_realised.py
    # sections 7.3, 7.4, 7.5 (engine, hours)
    .venv/bin/python paper/verdict-checks/levels_codmax.py
    .venv/bin/python paper/verdict-checks/levels_zerofixed.py
    .venv/bin/python paper/verdict-checks/levels_families.py
