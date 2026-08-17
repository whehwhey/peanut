# Adversarial search for an exponential equality-of-factors family

*Target 1 — Khodier 2026, Open Problem 1, the **lower-bound** side.*

**Status on the claim ("an exponential family exists"): `failed`.** No exponential family
was produced, and none of the seven mechanisms tried behaved exponentially. The
alternative deliverable — a report of which structural mechanisms are ruled out and why —
is complete. Specifically: (a) a combinatorial model of $|FE|$ that needs no logic engine
and reproduces the measured exponents (§2); (b) three *proofs* (Lemmas 3, 5, 7) that named
mechanisms cannot give exponential growth; (c) three further mechanisms killed by
measurement; (d) the first *exact* maxima $\max_{|Q|=m}|FE|$ for $k=2$, $m\le4$, together
with the extremal family they belong to (§7); and (e) a sharp statement of the only
structural route this analysis leaves open (§8).

All numbers below are minimal msd DFA state counts including the dead state (the
`automatheus` convention; Walnut reports one fewer). Every family was measured twice where
possible: with the Rust engine and with `explore/fe_py.py` / `explore/fe_fast.py`, two
independent from-scratch Python constructions written for this note and validated against
the engine on ten inputs (`15, 8, 17, 71, 113, 171, 241, 190, 698, 113`).

---

## 1. Setting

Fix $k \ge 2$. A $k$-automatic sequence $T$ is given by a msd DFAO
$A = (Q,\Sigma_k,\delta,q_0,\tau)$, $\Sigma_k=\{0,\dots,k-1\}$, $|Q| = m$,
$\delta(q_0,0)=q_0$ (prolongability of the generating $k$-uniform morphism), and
$T[n]=\tau(\delta^*(q_0,\langle n\rangle_k))$ with the expansion most significant digit
first. Here $m$ is "the size of the morphism".

$$FE(i,j,l) \;:=\; \forall t\,\big(t<l \Rightarrow T[i+t]=T[j+t]\big),$$

read msd with the three tracks zero-padded to a common length. Khodier's Open Problem 1
asks for a class of $T$ with $|FE|$ exponential in $m$, and states the belief that the
relationship *is* exponential.

**Fact 1.** $FE(i,j,l)\iff l\le \mathrm{LCE}(i,j)$, the longest common extension.

**Fact 2 (window form).** With $d=|j-i|$ and $M_d=\{x: T[x]\neq T[x+d]\}$,
$$FE(i,j,l)\iff [\min(i,j),\ \min(i,j)+l)\ \cap\ M_d=\emptyset .$$
So $FE$ says: *a window of length $l$ avoids the mismatch set at offset $d$*.

**Fact 3 (explicit upper bound).** The construction in `explore/fe_py.py` uses the NFA
with state space $(\text{carry}_1,\text{carry}_2,\text{cmp},q_u,q_v)\in 2\times2\times2\times Q\times Q$,
so $|FE| \le 2^{8m^2}+1$, slightly sharper than the $2^{9m^2}$ quoted in the thesis for the
$\forall u,v$ formulation.

---

## 2. The interval-image model

Fact 2 says the entire difficulty is *tracking which states a moving window can reach*.

**Definition (interval image).** Let $\mathcal B=(P,\gamma,p_0)$ be a DFA over $\Sigma_k$
(no output). For $n\ge0$ and words $A\le B\in\Sigma_k^n$ (compared as base-$k$ integers,
leading zeros allowed),
$$\mathrm{Img}_n(A,B)=\{\gamma^*(p_0,X): X\in\Sigma_k^n,\ A\le X\le B\}.$$
Let $\mathcal I(\mathcal B)=\#\{\mathrm{Img}_n(A,B)\}$ over all $n$ and all $A\le B$, and let
$\mathcal T(\mathcal B)$ be the number of reachable states of the online *tracker* that computes
$\mathrm{Img}$ while reading $(A,B)$ msd: states $(\mathrm{eq},u)$ and
$(\mathrm{sp},u,v,S)$ with $u,v$ the states on the two endpoint prefixes and $S$ the set of
states reached by strings strictly between them ($\mathcal B$ is written $\mathcal B$ to keep it apart from the endpoint $B$). Implementation:
`explore/interval_img.py`.

**Why this is the right object.** Reading $(i,j,l)$ msd, after $r$ digits the automaton
knows prefixes $(I,J,L)$; the still-live positions $x=i+t$, $t<l$, have $r$-digit prefixes
filling out the interval $[I,I+L]$ up to a carry, and the only thing the future can depend
on is the set of states these prefixes reach in the *correlation automaton* of $T$ (states
$(\text{carry},q_x,q_{x+d})$, at most $8m^2$ of them). Hence, heuristically,
$$|FE|\ \approx\ \mathcal T(\text{correlation automaton of }T).$$
The model has two known infidelities (the offset $d$ is read simultaneously rather than
fixed; tracker states may be $FE$-indistinguishable), so it is used here only to separate
*polynomial* from *exponential*. On that question it is faithful: §5 shows it reproduces
the empirical $3p^4$ law of `docs/TARGET1.md` and explains where the $4$ comes from.

**Cone decomposition.** Let $d_0$ be the first position where $A,B$ differ (if none,
$\mathrm{Img}=\{\gamma^*(A)\}$). Put $u_d=\gamma^*(p_0,A_{<d})$, $v_d=\gamma^*(p_0,B_{<d})$,
and let $R_r(q)=\{\gamma^*(q,w): w\in\Sigma_k^r\}$ be the *sphere of radius $r$*. Then

$$
\mathrm{Img}_n(A,B)=\{\gamma^*(A),\gamma^*(B)\}
\cup\!\!\!\bigcup_{a_{d_0}<c<b_{d_0}}\!\!\! R_{n-d_0-1}(\gamma(u_{d_0},c))
\cup\bigcup_{d>d_0}\bigcup_{c>a_d} R_{n-d-1}(\gamma(u_d,c))
\cup\bigcup_{d>d_0}\bigcup_{c<b_d} R_{n-d-1}(\gamma(v_d,c)).
\tag{2.1}
$$

**Lemma 1 (saturation: wide windows are cheap).** $\mathrm{Img}_n(A,B)\subseteq R_n(p_0)$.
If $\gamma(p_0,0)=p_0$ — which zero-stability gives for the DFAO and which the correlation
automaton inherits on the all-zero letter — the spheres from the start state increase,
$R_r(p_0)\subseteq R_{r+1}(p_0)$, so $R_r(p_0)$ is constant for $r\ge|P|$. Any window wide
enough to contain a full cone of depth $\le n-|P|$ therefore has image exactly $R_n(p_0)$.

**Lemma 2 (radius = digit position).** The radii carrying a nonempty cone in (2.1) are,
up to the usual odometer borrow, the positions of the nonzero base-$k$ digits of $B-A$:
each cone of radius $r$ accounts for $k^{r}$ of the $B-A+1$ words in the interval. The adversary's only freedom is *which
radii occur* and *where the cone roots sit*.

Together: an exponential family needs many distinguishable *narrow*-window images, i.e.
spheres that grow slowly enough not to saturate but fast enough not to be trivial.

---

## 3. Mechanism 1 — unbounded congruences are **free** in msd

> *"sequences where factor equality forces congruences (mod $k^r$) but with unbounded $r$
> relative to $m$"*

**Lemma 3 (alignment).** Suppose $FE(i,j,l)\iff i\equiv j \pmod{k^{f(l)}}$ for a monotone
$f$ with $f(l)\le \log_k l + C$. Then $|FE|=O(k^{C}m)$.

*Proof.* msd pads all three tracks to one length $n$, so the leading nonzero digit of $l$
occurs at position $n-\lceil\log_k l\rceil$ and is seen exactly when it occurs. The
automaton therefore knows at each digit position, up to an additive constant $C$, whether
it is inside the last $f(l)$ positions, and can enforce $i_p=j_p$ from there on; buffering
$C$ digits of slack costs $k^{C}$ states. $\square$

msd *aligns scales for free*. This is why the earlier "$FE$ forces $i\equiv j \bmod 2^p$,
hence $2^p$ states" heuristic (retracted in `docs/TARGET1.md`) had to fail.

**Measured — the ruler family.** $T_r[n]=[\,\nu_2(n+1)\equiv0 \bmod r\,]$; msd DFAO
$\delta(q,1)=q{+}1 \bmod r$, $\delta(q,0)=0$, $m=r$ states, minimal, zero-stable. Factor
equality here really does force $i\equiv j$ modulo a power of two growing with $l$.

| $m=r$ | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|
| $\lvert FE\rvert$ | 8 | 34 | 113 | 305 | 712 | 1471 | 2751 | 4755 |
| ratio | | 4.25 | 3.32 | 2.70 | 2.33 | 2.07 | 1.87 | 1.73 |
| local exponent $a$ in $r^a$ | | 3.6 | 4.2 | 4.1 | 4.0 | 4.3 | 4.5 | 4.6 |

Ratios fall monotonically; the local exponent sits between 3.6 and 4.6 over an eightfold
range of $r$. Polynomial, degree $\approx 4$. (`results/lower_batch1.log`,
`results/lower_ruler.log`.)

---

## 4. Mechanism 2 — reverse (lsd) blowup does **not** transfer

The most attractive hidden exponential inside an automatic sequence is the gap between its
msd and lsd automata: the minimal lsd DFAO can be $2^{m}$ while the msd DFAO has $m$
states. Since addition is an lsd operation and $FE$ is made of additions, one expects msd
$FE$ to have to pay for the reverse.

**Family $D_m$.** $T[n]$ = the $m$-th most significant base-2 digit of $n$ (0 if $n$ has
fewer than $m$ digits). msd DFAO: $q_0$ absorbing leading zeros, a counter
$c_1,\dots,c_{m-1}$, two sinks — $M:=m+2$ states, minimal, zero-stable. Minimal lsd DFAO:
exactly $2^{m}$ (reading from the low end you must remember the last $m$ digits).

| $M$ (msd states) | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|
| minimal lsd DFAO | 8 | 16 | 32 | 64 | 128 | 256 |
| $\lvert FE\rvert$ | 71 | 113 | 171 | 241 | 319 | 405 |

Second differences $16,12,8,8$: sub-quadratic, local exponent $\approx 2.3$ and falling.
An exponential gap in the digit-order complexity of $T$ buys **nothing** in $FE$.
Confirmed independently by the Rust engine (`results/lower_batch1.log`) and by
`explore/fe_py.py`. In the model the reason is Lemma 1: the spheres of $D_m$ saturate
after $m$ steps.

**The transition-monoid variant.** `full` family, $k=3$: digit 0 = transposition
$(1\,2)$ (fixing the start state), digit 1 = the $m$-cycle, digit 2 = a collapse; the
transition monoid is the full transformation monoid, $m^m$ elements. Measured
$|FE| = 113,\,301,\,460$ at $m=3,4,5$ — indistinguishable from the random-ensemble medians
$105,\,330,\,415$ of `docs/TARGET1.md`. Across the 385 sweep automata of
`results/blowup.json` the within-$(k,m)$ Spearman correlation of $|FE|$ with the minimal
lsd DFAO size is only $+0.19\dots+0.44$, and the ensemble never produced a reverse size
above 82 at $m=7$ — random morphisms simply do not explore this axis, which is why the
family above had to be built by hand.

---

## 5. Mechanism 3 — codings of group automata: dead, by proof

`docs/TARGET1.md` leaves one lead standing: *lossy codings of group automata*. Collapsing
$s_2 \bmod p$ from $p$ letters to the binary $[s_2\not\equiv0]$ keeps $m=p$ and multiplies
$|FE|$ by $\approx100$; the sizes $190,698,1877,3971,7243,11988$ fit $3p^4$ within 2.5%.
Is the exponent 4 an artefact of $\mathbb Z_p$, or a law?

**Definition.** A *translation automaton* over a finite group $G$ is $P=G$,
$\gamma(x,c)=x\,a_c$. Zero-stability forces $a_0=e$; hence for $k=2$ the transition monoid
is the cyclic group $\langle a_1\rangle$, and $k\ge3$ is needed for a non-abelian one.
Write $S_r=\{a_{c_1}\cdots a_{c_r}\}$, so $R_r(x)=xS_r$ and $S_{r+s}=S_rS_s$.

**Lemma 5 (nesting).** Let $G$ be abelian and suppose that for every
$0\le\alpha\le k-2$ and every $1\le c'\le k-1$ there is a digit $c$ with $\alpha<c\le k-1$
and $a_\alpha+a_{c'}-a_c\in\{a_0,\dots,a_{k-1}\}$. Then for $A$-side cones at depths
$d<d'$,
$$R_{n-d'-1}\big(\gamma(u_{d'},c')\big)\ \subseteq\ R_{n-d-1}\big(\gamma(u_d,c)\big).$$
The hypothesis holds (i) whenever $k=2$, and (ii) whenever the digit actions are *linear*,
$a_c = c\,g$ — which covers $s_k(n)\bmod p$ and every generalised Thue–Morse sequence.

*Proof.* Write $\alpha=a_d$ (the $A$-digit at the shallower cone depth). Iterating the
transitions from depth $d$ to depth $d'$,
$u_{d'}\in u_d + a_\alpha + S_{d'-d-1}$, so
$$\gamma(u_{d'},c')+S_{n-d'-1}\ \subseteq\ u_d+a_\alpha+a_{c'}+S_{d'-d-1}+S_{n-d'-1}
= u_d+\big(a_\alpha+a_{c'}\big)+S_{n-d-2}.$$
Choose $c$ with $a_\alpha+a_{c'}-a_c\in\{a_0,\dots,a_{k-1}\}$ and $c>\alpha$; then
$a_\alpha+a_{c'}\in a_c+S_1$, so the right-hand side is contained in
$u_d+a_c+S_1+S_{n-d-2}=u_d+a_c+S_{n-d-1}=R_{n-d-1}(\gamma(u_d,c))$.
Commutativity is what lets $a_{c'}$ be collected together with $a_\alpha$ out of the
middle of the sum; without it the two are separated by $S_{d'-d-1}$ and the containment
fails.
For $k=2$: a cone exists only when $\alpha=0$, and then $c=c'=1$ and
$a_0+a_1-a_1=a_0$ is a digit action. For $a_c=cg$: take $c=\alpha+1\le k-1$; then
$a_\alpha+a_{c'}-a_{\alpha+1}=(c'-1)g=a_{c'-1}$. $\square$

**Corollary 6.** Under the hypothesis of Lemma 5, on each side of the split all cones are
contained in the shallowest one, so every interval image is
$$\{\gamma^*(A),\gamma^*(B)\}\ \cup\ (g_A+S_{r_A})\ \cup\ (g_B+S_{r_B})\ \cup
\!\!\bigcup_{a_{d_0}<c<b_{d_0}}\!\!(u_{d_0}+a_c+S_{r_0})$$
— at most $k$ spheres and two points; for $k=2$ exactly *two spheres and two points*.
The chain $\{S_r-r\,a_0\}_r$ is non-decreasing ($S_{r+1}\supseteq S_r+a_0$), so it takes at
most $|G|$ values and $\{S_r\}_r$ has at most $|G|\cdot\mathrm{ord}(a_0)\le|G|^2$ members
(just $|G|$ when $a_0=e$, which zero-stability forces for a DFAO). An image is therefore
determined by $O(1)$ pairs (group element, sphere index), giving
$\mathcal I = |G|^{O(1)}$. **Polynomial in the number of states.**

**Measured.** $G=\mathbb Z_c$, $k=2$, actions $(0,1)$ (i.e. $s_2\bmod c$):
$$\mathcal I(c)=3,7,15,31,61,113,197,325,511,771,1123,1587,2185,2941,3881,5033,6427,8095,10071,12391$$
for $c=2..21$. The **fourth** difference is constantly $2$: $\mathcal I$ is a *quartic
polynomial* in $c$ for $c\ge4$ — the same order as the number of "unions of two arcs"
($\sim c^4/8$), and the same exponent 4 that the engine measures for $|FE|$ on this family. The first four values equal
$2^c-1$: small $\mathbb Z_c$ do realise *every* subset; the exponential behaviour is real
but dies at $c\approx6$, when Lemma 1 takes over.

**Non-abelian does not help.** Lemma 5 needs commutativity, so non-abelian group automata
are the designed counter-candidate. They are *worse*:

| $m$ | group ($k=3$, digit 0 = identity, singleton coding) | $\mathcal I$ | $\lvert FE\rvert$ |
|---|---|---|---|
| 3 | $\mathbb Z_3$ | 7 | 239 |
| 4 | $\mathbb Z_4$ | 13 | 158 |
| 5 | $\mathbb Z_5$ | 31 | 1579 |
| 6 | $\mathbb Z_6$ | 46 | **991** |
| 6 | $S_3=D_3=\mathrm{AGL}(1,3)$ (non-abelian) | 45 | 473 |
| 8 | $\mathbb Z_8$ | 141 | **3283** |
| 8 | $D_4$ (non-abelian) | 129 | 904 |
| 10 | $D_5$ (non-abelian) | 241 | 1493 |

At every size where both are available the non-abelian group is the *smaller* of the two:
$473$ vs $991$ at $m=6$, $904$ vs $3283$ at $m=8$. Over the whole dihedral family $D_n$ ($c=2n$ states) the interval-image count is
$45,129,241,379,547,745,973,1231,1519$ for $n=3..11$: second differences constant at
$\approx30$, i.e. **quadratic** in $c$ — an order *below* the abelian $c^4$. Reflections
mix the Cayley ball too quickly, so the spheres saturate sooner and Lemma 1 bites harder.
$\mathrm{AGL}(1,5)$ ($c=20$) gives $10420$, the same order as $\mathbb Z_{20}$'s $7221$.

**The one place the hypothesis of Lemma 5 genuinely fails** is *abelian but non-linear*
digit actions, $k\ge3$, $a=(0,1,s)$ with $s\neq2$. These are measurably richer than the
linear ones — and still polynomial:

| $c$ | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| linear $s=2$ | 46 | 113 | 141 | 325 | 361 | 771 | 793 | 1587 | 1548 | 2941 | 2761 | 5033 | 4591 |
| best $s$ | 60 | 127 | 241 | 412 | 726 | 1134 | 1735 | 2588 | 3711 | 5131 | 6945 | 9249 | 11881 |

$\log$–$\log$ slope of the second row over $c=6..18$: $4.81$, essentially constant
($\log_2$ increments $1.1,0.9,0.8,0.8,0.6,0.6,0.6,0.5,0.5,0.4,0.4,0.4$ — falling, not
constant, so not exponential). Note $c=7$ hits $127=2^7-1$: again, *all* subsets are
realised at small $c$, and the collapse to a polynomial happens only once the spheres
saturate. (`explore/interval_img.py`, `explore/ii_groups.py`.)

**Hill-climbing over all DFAs barely beats the arcs.** `explore/ii_search.py` /
`ii_search2.py` maximise $\mathcal I$ over $c$-state DFAs, from random starts and *seeded
from the abelian arc automaton*. Random restarts never even reach the cyclic value
(at $c=14$: best found $2154$ vs the cyclic $2185$). Seeded ($k=2$), cyclic $\to$ best:

| $c$ | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 15 | 16 | 17 |
|---|---|---|---|---|---|---|---|---|---|---|
| cyclic | 61 | 113 | 197 | 325 | 511 | 771 | 1123 | 2941 | 3881 | 5033 |
| best found | 63 | 117 | 204 | 325 | 511 | 792 | 1123 | 3070 | 4910 | 6191 |

The margin grows with $c$ (up to $+27\%$ at $c=16$) but the *growth rate* does not: the
$\log_2$ increments of the second row are $0.9,0.8,0.67,0.66,0.63,0.50,\dots,0.68,0.34$,
falling, and the local exponent between $c=16$ and $17$ is $3.8$. Nothing found by search
grows faster than a low-degree polynomial, and the abelian arc automaton is within a small
constant factor of the best known — which is what makes Corollary 6 informative rather
than a special case.

---

## 6. Mechanism 4 — products and direct sums

**Lemma 7 (products are useless).** For $T=(T_1,T_2)$ over the product alphabet,
$m(T)\le m_1m_2$ and $FE_T=FE_{T_1}\wedge FE_{T_2}$, so $|FE_T|\le|FE_1||FE_2|$. Both sides
multiply: if $|FE_i|\le Cm_i^{a}$ then $|FE_T|\le C^2m(T)^{a}$. Polynomiality is preserved,
and no leverage is gained. Same for every Boolean combination.

The interesting version is the one where $m$ is **additive**: a *direct sum*, $k=3$, with
$\delta(q_0,0)=q_0$, $\delta(q_0,1)$ = start of $A_1$, $\delta(q_0,2)$ = start of $A_2$, so
$m=1+m_1+m_2$. Does $|FE|$ multiply? It does not: $FE$ splits over the four branch pairs
and is **additive**.

| construction ($A_i$ = $s_3\bmod p_i$, singleton coding) | $m$ | $\lvert FE\rvert$ |
|---|---|---|
| single $p=2$ | 3 | 35 |
| single $p=3$ | 4 | 284 |
| single $p=4$ | 5 | 275 |
| sum $2\oplus2$ | 5 | 35 |
| sum $2\oplus3$ | 6 | 652 |
| sum $3\oplus3$ | 7 | 284 |
| sum $3\oplus4$ | 8 | 1088 |

$3\oplus4$ would need $284\times275=78{,}100$ if $FE$ multiplied; it gives $1088\approx
2(284+275)$ — the two component predicates plus the two cross predicates, i.e. **additive**.
(The two rows with equal components, $2\oplus2$ and $3\oplus3$, reproduce the single-component
value exactly: identical branches carry no extra information, and the state count does not
even grow with $m$.) Direct sums are ruled out. (`results/lower_batch1.log`.)

---

## 7. The maximum over all $m$-state DFAOs

The families above are hand-picked; the real question is $\max_{|Q|=m}|FE|$. This was
computed **exhaustively** for $k=2$, $m\le4$, over all minimal zero-stable DFAOs up to
state relabelling and coding complement (`explore/fe_exhaust4.py`, engine-evaluated,
0 failures), and by large random sampling / hill-climbing above.

| $k$ | $m$ | mode | $\max\lvert FE\rvert$ | ratio | witness |
|---|---|---|---|---|---|
| 2 | 2 | exhaustive, 8 classes | 15 | | `def T 2 2 0 01 10 10` (Thue–Morse) |
| 2 | 3 | exhaustive, 436 classes | 264 | 17.6 | `def T 2 3 0 01 22 10 110` |
| 2 | 4 | exhaustive, 4460 classes | **1152** | 4.36 | `def T 2 4 0 01 22 33 10 1110` |
| 2 | 5 | random sample, 1200 classes | $\ge$ **2415** | $\ge2.10$ | `def T 2 5 0 02 33 41 24 10 00001` |

The ratios fall hard: $17.6 \to 4.36 \to (\ge)2.10$. An exponential law $\max=C\lambda^m$
requires a *constant* ratio; a power law $\max=Cm^{a}$ predicts ratios $(1+1/m)^{a}$, i.e.
local exponents $7.1$, $5.1$, $(\ge)3.3$ — falling, which is what a polynomial with
lower-order terms looks like. The $m\le4$ rows are *exact maxima over the whole class*,
not sample maxima. The $m=5$ row is a lower bound from 1200 sampled classes out of
$\approx1.2$M (2 of the 1200 were censored by the 2 GB / 180 s budget and could be larger);
even if the true $m=5$ maximum were three times the sampled one the ratio would still be
below $4.36$.

**The extremal family.** The exhaustive witnesses at $m=2,3,4$ are not three unrelated
automata — they are the first three members of one family $C_M$ ($k=2$):

$$\delta(0,0)=0,\quad \delta(0,1)=1,\quad \delta(i,\cdot)=i+1\ (1\le i\le M-2),\quad
\delta(M-1,0)=1,\ \delta(M-1,1)=0,\quad \tau(q)=[\,q\neq M-1\,].$$

$C_2$ **is Thue–Morse**. So the maximiser of $|FE|$ over the whole class, at every size where
the maximum is known, is "Thue–Morse with a longer counter": a mod-$(M-1)$ counter that
advances on every digit and is reset only by the digit $1$ arriving in the last counter
state. It is not a group automaton (the reset is digit-dependent), which is consistent
with §5. Measured:

| $M$ | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|
| $\lvert FE\rvert$ | 15 | 264 | 1152 | censored | censored | censored |

$C_5$ is where the *construction* becomes hard: msd, forward caps 5k/50k/400k, 3–5 GB,
ten minutes — no answer (`results/fe_champ.log`, `results/fe_champ_m5.log`). That is
Khodier's intermediate blowup landing exactly on the extremal family, which is either a
coincidence or the whole story of Open Problem 1's second half. **This is the single most
promising object found here and the obvious next step**: if any family is exponential, the
exhaustive evidence says it should be this one, and settling $C_5, C_6, C_7$ (Walnut's
`reverse`, a better ladder, or lsd-then-reverse) is the highest-value follow-up in this
note.

For orientation, the exhaustive maximum at $m=3$ (264) is five times the random-ensemble
median (51) of `docs/TARGET1.md` and 1.2 times its sample maximum (218); at $m=4$ the
exhaustive maximum (1152) is 2.1 times the sample maximum (549). Random sampling
under-reports the extreme by a small factor, so the sweep conclusions of
`docs/TARGET1.md` survive the correction.

---

## 8. What an exponential family would have to look like

Collecting §2–§7, in the interval-image picture an exponential family must satisfy all of:

* **(N) Non-nesting.** The cones of (2.1) must not nest. By Lemma 5 this rules out
  commutative transition structure at $k=2$, and all *linear* abelian structure at every
  $k$ (so: every generalised Thue–Morse sequence, with any coding). By §5,
  non-commutativity is not sufficient — it typically *accelerates* saturation.
* **(S) Slow saturation.** $|R_r(q)|$ must stay well below $|P|$ for $r$ up to $\sim|P|$,
  or Lemma 1 collapses every wide window. But $|R_r(q)|\le B$ for all $r$ forces
  $\mathrm{Img}\subseteq R_n(p_0)$ with $|R_n(p_0)|\le B$ and hence at most $2^{B}$ images
  in total. So one needs the intermediate regime $|R_r(q)|\approx r$: spheres that are
  proper, linearly growing arcs.
* **(I) Independence.** With $|R_r|\approx r$ and $\sum_{r\in D}r\lesssim|P|$ the adversary
  can afford $|D|\approx\sqrt{|P|}$ independent radii (Lemma 2: the radii are the nonzero
  digit positions of the window length), so at most $2^{\Theta(\sqrt{|P|})}$ images. With
  $|P|=\Theta(m^2)$ for the correlation automaton, $2^{\Theta(\sqrt{m^2})}=2^{\Theta(m)}$
  — **exponential in $m$, and the only route this analysis leaves open.**

The target is therefore sharp: *an automatic sequence whose correlation automaton has
linearly growing, non-nesting spheres.* Every family measured here fails (N) (abelian
arcs) or fails (S) (dihedral, full-monoid, $D_m$, thin sets, pattern containment).

A byproduct worth stating on its own, because it is engine-free and checkable:

> **Conjecture (interval images).** For every DFA with $c$ states over a $k$-letter
> alphabet, the number of distinct interval images is $c^{O(1)}$; the data say
> $\max_{|P|=c}\mathcal I \approx c^{4}$–$c^{5}$, attained by the abelian arc automaton.

Modulo the fidelity of the model, this conjecture implies $|FE| = m^{O(1)}$, i.e. that
Khodier's Open Problem 1 has a negative answer.

---

## 9. Gaps — what is *not* established

1. **No lower-bound theorem, and no impossibility theorem.** Nothing here forbids an
   exponential family. §8 is a list of necessary conditions inside a heuristic model, not a
   proof.
2. **Lemma 5's scope.** Proved for translation automata over abelian groups, and only under
   the digit hypothesis — which holds for all $k$ in the linear case and for $k=2$ in
   general, but *not* for general abelian actions at $k\ge3$. That is exactly the case
   whose data (§5, last table) grows fastest, and it is only measured to $c=16$.
3. **The interval-image model is a heuristic.** It fixes the offset track $d$ that $FE$
   reads simultaneously, and it ignores $FE$-indistinguishability, so $\mathcal T$ neither
   upper- nor lower-bounds $|FE|$ formally. Its only credential is that it predicts the one
   exponent that is independently known (quartic, for group automata with lossy codings).
4. **(I) is a ceiling, not a construction.** $2^{\Theta(\sqrt{|P|})}$ is what arc-counting
   *permits*. No automaton realising it is exhibited, and hill-climbing over general
   $c$-state DFAs never beat the abelian $\Theta(c^4)$.
5. **The maximisation is small.** Exhaustive only up to $m=4$, $k=2$ (4460 classes); $m=5$
   would be $\approx1.2$M classes. Three exact points ($m=2,3,4$) is a short curve on which
   to reject an exponential, even though the ratios fall by a factor of four across it. The
   Python hill-climb is a weak optimiser and its objective silently discards candidates that
   blow the *forward subset* cap — precisely where Khodier's intermediate blowup lives; that
   is why the exhaustive pass was run through the Rust engine (with its Brzozowski ladder)
   instead, where 0 of 4460 evaluations failed.
6. **Censoring.** The Rust engine shared a memory-constrained machine for part of the
   session and the system guard killed several runs (`FE=KILLED` in `results/lower_*.log`);
   those cells are censored, not resolved. All Python-side numbers are unaffected.
7. **$k$ is held small.** Everything is $k\in\{2,3\}$. A family in which $k$ grows with $m$
   was not attempted, and the problem statement does not obviously forbid it.

---

## Artefacts

| file | what |
|---|---|
| `explore/fe_py.py` | independent Python construction of the minimal msd $FE$ DFA (reference) |
| `explore/fe_fast.py` | bit-mask version, + Brzozowski double-reversal route |
| `explore/fe_max.py`, `explore/fe_grow.py` | Python-side exhaustive / hill-climbing maximisation of $\lvert FE\rvert$ |
| `explore/fe_exhaust4.py` | **exhaustive** maximisation up to state relabelling + coding complement, engine-evaluated |
| `explore/fe_sample5.py` | large random sample of the DFAO class, engine-evaluated |
| `explore/interval_img.py` | interval-image tracker, $\mathcal I$ and $\mathcal T$ |
| `explore/ii_groups.py`, `explore/ii_search.py`, `explore/ii_search2.py` | group families and maximisation of $\mathcal I$ |
| `explore/lower_fam.py`, `explore/lower_monoid.py`, `explore/lower_nonab.py` | the $D_m$, full-monoid, and group-coding DFAO families |
| `explore/lower_batch.py`, `explore/lower_run.sh` | engine drivers (the `.sh` is a sequential single-engine fallback used while the machine was under memory pressure) |
| `results/fe_exhaust_m4.{log,json}`, `results/fe_sample_k2m5.{log,json}` | the maxima |
| `results/fe_champ_family.txt`, `results/fe_champ.log`, `results/fe_champ_m5.log` | the extremal family $C_M$ and its (partly censored) measurements |
| `results/ii_cyc.log`, `results/ii_groups.log`, `results/ii_nonlin.log`, `results/ii_seeded.log`, `results/ii_big.log` | interval-image counts |
| `results/lower_batch1.{log,json}`, `results/lower_dig.log`, `results/lower_full.log`, `results/lower_nonab.log`, `results/lower_dsum.log`, `results/lower_ruler.log` | measurements |

---

## Reproduce

    cd /Users/andrew/maths
    .venv/bin/python explore/fe_py.py                      # validate the Python FE builder
    .venv/bin/python -u explore/fe_exhaust4.py 4           # exact max |FE|, k=2 m=4  (118 s)
    .venv/bin/python -u explore/fe_sample5.py 2 5 8000     # sampled max, k=2 m=5
    .venv/bin/python -u explore/interval_img.py cyc        # I(Z_c): the quartic
    .venv/bin/python -u explore/ii_groups.py               # dihedral / affine / Heisenberg
    .venv/bin/python -u explore/ii_search2.py              # seeded hill-climb on I
    .venv/bin/python -u explore/lower_batch.py <defs> <out.json>   # engine |FE| batch
