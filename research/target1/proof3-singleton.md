# The equality-of-factors automaton of the singleton coding $T_p=[\,s_2(n)\equiv1\,]$

*Target 1 (Khodier 2026, Open Problem 1) — the singleton family. Companion to
`paper/proof-family.md` (which settles the identity coding $G_p$ at $\Theta(m^3)$) and to
the referee report `paper/proof-verdict.md`, whose open item G4 ("$T_p$: exponent 3 vs 4 is
not settled") is what this note attacks.*

**Status: `partial`.**

**Proved and new.**

1. An *exact* reformulation of $FE_{T_p}$ as an avoidance condition on a **pair set**
   (Lemma 1.2), and an exact description of the Myhill–Nerode residual of an msd prefix by
   an eight-component invariant $\Phi$ (Theorem 2.4). This is a genuine explicit
   finite-state description: quotienting by $\Phi$ *is* a DFA for $FE_{T_p}$, and it was
   implemented and minimised, reproducing the engine exactly for $p\le6$
   ($15,190,698,1877,3971$) and independently reproducing $|FE_{T_7}|=7243$.
2. **Theorem 4.1 (main).** $\;|FE_{T_p}|_{\mathrm{msd}}\ \ge\ p^{4}$ for every $p\ge3$.
   Proof by an explicit $p^4$-element fooling set with three explicit suffix families;
   machine-verified **from the definition of $FE$** (digit sums, no automaton) for
   $p=3,4,5,6,7$: $81,256,625,1296,2401$ pairwise distinct response vectors, $0$
   collisions.
3. **Corollary 4.3 — G4 is settled on the lower side.** $|FE_{T_p}|=\Omega(p^4)$, so the
   *cubic* reading of `proof-family.md` §5.4(b) ("$\approx 44p^3$, i.e. the singleton
   coding costs a constant factor") is **false**. Against $|FE_{G_p}|=p^3+8$, the lossy
   coding costs a factor $\Omega(p)$, not $O(1)$. With the measured values the ratio $|FE_{T_p}|/p^4$ is
   $2.35,\,2.73,\,3.00,\,3.06,\,3.02,\,2.93$ for $p=3..8$, so `docs/TARGET1.md`'s "$3p^4$"
   law is right in its exponent (from below) and the truth is conjecturally
   $|FE_{T_p}|_{\mathrm{msd}}=\Theta(p^4)$ with constant $\approx3$.
4. **Lemma 5.1 (singleton rigidity), measured.** $\max\{l:FE_{T_p}(i,i+d,l),\ d\text{ odd}\}
   = 2^{p}-1$, attained at $d=2^p-1$, $i=\mathrm{val}(1^{p+1}0^{p-2}1)$ ($p\ge3$). Contrast
   `proof-family.md` Lemma 3.3, where the same quantity for the *identity* coding is
   $\le 3$. This is the precise reason the $G_p$ proof technique does not transfer, and it
   rules out any "bounded window past the parity split" construction for $T_p$.

**Not obtained.**

* The requested $O(p^c)$ *upper* construction. The explicit description of §2 is correct
  but its state count is measured $\approx p^{8}$ (§3), and I cannot prove any polynomial
  bound on it. What §3 *does* give is a reduction of the whole upper-bound problem (G3) to
  one clean, self-contained counting question — **how many sets
  $\Pi(u,v;\eta)=\{(s(q),s(q+\eta))\bmod p:\ u\le q\le v\}$ are there?** — which is the
  two-dimensional analogue of the interval-image count that `proof-lower.md` Cor. 6
  *proves* is $p^{O(1)}$ in one dimension (its structure theorem gives $O(p^6)$; the
  measured truth is $\sim p^4/12$).
* $p=9,10$ measurements. Both engine routes fail on memory (§6): `learnfe` is killed above
  $9.5$ GB at $p=7,8,9$; the direct `let FE` ladder likewise. My independent builder
  reproduces $p\le7$ but its state count grows too fast for $p\ge8$.

Everything below is base $k=2$, $s(\cdot)$ = binary digit sum, $\nu(\cdot)$ = number of
trailing $1$-bits, $G_p[n]=s(n)\bmod p$, $T_p[n]=[\,s(n)\equiv1\bmod p\,]$, and
$$FE(i,j,l)\ :=\ \forall t\,(t<l\Rightarrow T_p[i+t]=T_p[j+t]).$$

---

## 0. Conventions

Words are over $\Sigma_3=\{0,1\}^3$; a word of length $N$ encodes the triple $(i,j,l)$ in
**msd** order with leading zeros allowed, so the language

$$L_p\ :=\ \{\,w\in\Sigma_3^*\ :\ FE(i(w),j(w),l(w))\,\}$$

is closed under adding and (as long as the value is unchanged) removing leading $000$
letters. $|FE_{T_p}|_{\mathrm{msd}}$ is the number of states of the minimal **complete**
DFA of $L_p$ (dead state counted — the `automatheus`/`peanut` convention; Walnut reports
one fewer).

Two facts used throughout, both immediate:

$$\textbf{(D1)}\quad s(x2^S+y)=s(x)+s(y)\ \ (0\le y<2^S),\qquad
\textbf{(D2)}\quad s(n+1)=s(n)+1-\nu(n).$$

**Independent reference implementation.** `explore/sing/sub1.py` / `sub2.py` build the DFA
of $L_p$ from scratch — a $8p^2$-state NFA for the *negated* witness
$\exists t\,(t<l\wedge T[i+t]\ne T[j+t])$ (state: the two required carries, the $t<l$
comparator, and the two running digit sums $\bmod\ p$), started from the $0^\infty$-limit
subset so that the language is padding-invariant, then determinised, complemented and
minimised. It returns
$$15,\ 190,\ 698 \qquad (p=2,3,4),$$
i.e. exactly the engine's numbers. Every claim below that is called "machine-checked"
was checked against *this* pipeline or against the definition of $FE$ directly, never
against the engine alone.

---

## 1. $FE_{T_p}$ as a pair-set avoidance condition

**Definition 1.1 (pair set).** For $u\le v$ and $\eta\in\mathbb Z$ with $u+\eta\ge0$ put

$$\Pi(u,v;\eta)\ :=\ \{\,(\,s(q)\bmod p,\ s(q+\eta)\bmod p\,)\ :\ u\le q\le v\,\}\ \subseteq\
\mathbb Z_p^2 ,$$

and $\Pi(u,v;\eta)=\emptyset$ when $u>v$. Let

$$\mathcal B\ :=\ \{(\alpha,\beta)\in\mathbb Z_p^2:\ [\alpha=1]\ne[\beta=1]\}$$

(the "bad" set: a cross of $2(p-1)$ cells through $(1,1)$, with $(1,1)$ removed).

**Lemma 1.2.** For $l\ge1$ and $i\le j$,
$$FE(i,j,l)\iff \Pi(i,\,i+l-1;\,j-i)\cap\mathcal B=\emptyset .$$

*Proof.* $T[n]=[s(n)\equiv1]$, so $T[q]=T[q+d]$ fails exactly when
$(s(q),s(q+d))\in\mathcal B$; range over $q=i+t$, $t<l$. $\square$

This is `proof-family.md` Prop. 5.1 in a coordinate-free form: Prop. 5.1 is what one gets
by splitting $\Pi$ along the blocks $[a2^k,(a+1)2^k)$, $k=v_2(j-i)$.

---

## 2. The residual of an msd prefix: an exact eight-component invariant

Fix a prefix $w\in\Sigma_3^h$ with values $(I,J,\Lambda)$ and a suffix $z\in\Sigma_3^S$
with values $(i_{lo},j_{lo},l_{lo})$. Then $wz$ has values
$$i=I2^S+i_{lo},\qquad j=J2^S+j_{lo},\qquad l=\Lambda 2^S+l_{lo}.$$

**Definition 2.1.** For $c,c'\in\{0,1\}$ put
$$\Sigma^{<}_{c,c'}(w):=\Pi\big(I+c,\ I+c+\Lambda-1;\ (J-I)+(c'-c)\big)
=\{(s(I{+}\Theta{+}c),s(J{+}\Theta{+}c')): 0\le\Theta<\Lambda\},$$
$$\Sigma^{=}_{c,c'}(w):=\{(\,s(I+\Lambda+c),\ s(J+\Lambda+c')\,)\}\quad(\text{a single pair}).$$

**Definition 2.2.** For $\Sigma\subseteq\mathbb Z_p^2$ and $(\alpha,\beta)\in\mathbb Z_p^2$
let
$$\Phi_\Sigma(\alpha,\beta)\ :=\ \big[\ \forall(\sigma,\sigma')\in\Sigma:\
[\sigma=\alpha]=[\sigma'=\beta]\ \big]
\ =\ \big[\ \Sigma\cap\big(\text{row }\alpha\ \cup\ \text{col }\beta\big)\subseteq\{(\alpha,\beta)\}\ \big].$$
Equivalently $\Phi_\Sigma(\alpha,\beta)=1$ iff $(\Sigma+(1-\alpha,1-\beta))\cap\mathcal B=\emptyset$.

**Lemma 2.3 (splitting).** Write every $t\in[0,l)$ uniquely as $t=\Theta2^S+\tau$ with
$0\le\tau<2^S$ and $0\le\Theta\le\Lambda$, subject to $\Theta<\Lambda$, or $\Theta=\Lambda$
and $\tau<l_{lo}$. For each $\tau$ set
$$c(\tau)=[i_{lo}+\tau\ge2^S],\quad x(\tau)=s\big((i_{lo}+\tau)\bmod 2^S\big),\quad
c'(\tau)=[j_{lo}+\tau\ge2^S],\quad y(\tau)=s\big((j_{lo}+\tau)\bmod 2^S\big).$$
Then
$$wz\in L_p\iff \forall\tau\in[0,2^S):\quad
\Phi_{\Sigma^{<}_{c(\tau),c'(\tau)}(w)}\big(1{-}x(\tau),\,1{-}y(\tau)\big)
\ \ \wedge\ \ \Big(\tau<l_{lo}\Rightarrow
\Phi_{\Sigma^{=}_{c(\tau),c'(\tau)}(w)}\big(1{-}x(\tau),\,1{-}y(\tau)\big)\Big).$$

*Proof.* $i+t=(I+\Theta+c(\tau))2^S+\big((i_{lo}+\tau)\bmod2^S\big)$, so by (D1)
$s(i+t)=s(I+\Theta+c(\tau))+x(\tau)$, and likewise $s(j+t)=s(J+\Theta+c'(\tau))+y(\tau)$.
Hence the clause "$[s(i+t)\equiv1]=[s(j+t)\equiv1]$" reads
"$[\sigma=1-x(\tau)]=[\sigma'=1-y(\tau)]$" for the pair
$(\sigma,\sigma')=(s(I{+}\Theta{+}c),s(J{+}\Theta{+}c'))$. Quantifying over the admissible
$\Theta$ at fixed $\tau$ gives exactly the two $\Phi$'s of Definition 2.2 evaluated at
$(1-x(\tau),1-y(\tau))$: the $\Theta<\Lambda$ part is $\Sigma^{<}_{c,c'}$, the
$\Theta=\Lambda$ part (present iff $\tau<l_{lo}$) is $\Sigma^{=}_{c,c'}$. $\square$

**Theorem 2.4 (explicit finite-state description).** Let
$$\Phi(w)\ :=\ \Big(\ \Phi_{\Sigma^{<}_{c,c'}(w)},\ \Phi_{\Sigma^{=}_{c,c'}(w)}\ \Big)_{c,c'\in\{0,1\}}
\ \in\ \big(\{0,1\}^{\mathbb Z_p^2}\big)^{8}.$$
If $\Phi(w)=\Phi(w')$ then $w$ and $w'$ are Myhill–Nerode equivalent for $L_p$.
Consequently the quotient of $\Sigma_3^*$ by $\Phi$ carries a well-defined DFA for $L_p$
(transitions realised on any representative), and
$$|FE_{T_p}|_{\mathrm{msd}}\ \le\ \#\{\Phi(w):w\in\Sigma_3^*\}.$$

*Proof.* Immediate from Lemma 2.3: the right-hand side there depends on $w$ only through
$\Phi(w)$. For the quotient: replacing a state by a Myhill–Nerode-equivalent one preserves
the accepted language, so redirecting each transition to the representative of the
successor's $\Phi$-class yields a DFA accepting $L_p$. $\square$

**Structure of a single component.** Write $\mathrm{Row}(\Sigma),\mathrm{Col}(\Sigma)$ for
the two projections and
$\mathrm{Iso}(\Sigma)=\{(\alpha,\beta)\in\Sigma:\ \Sigma\cap(\text{row}\,\alpha\cup\text{col}\,\beta)=\{(\alpha,\beta)\}\}$.
Then directly from Definition 2.2,
$$\Phi_\Sigma\ =\ \big(\mathbb Z_p\setminus\mathrm{Row}\big)\times\big(\mathbb Z_p\setminus\mathrm{Col}\big)\ \cup\ \mathrm{Iso},$$
and $\mathrm{Iso}$ is a partial injection $\mathrm{Row}\to\mathrm{Col}$. For
$\Sigma=\Pi(u,v;\eta)$ this says: $\Phi_\Sigma(\alpha,\beta)=1$ iff the level set
$\{q\in[u,v]:s(q)\equiv\alpha\}$ coincides with $\{q\in[u,v]:s(q+\eta)\equiv\beta\}$.
At $(\alpha,\beta)=(1,1)$ this is exactly Lemma 1.2.

**Machine check.** `explore/sing/phidfa.py` implements exactly this: subset construction
with dedup key $\Phi$, then exact Moore minimisation. Output:

| $p$ | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|
| $\Phi$-states | 104 | 4905 | 45440 | 257900 | 1131588 | 4058229 |
| minimised | **15** | **190** | **698** | **1877** | **3971** | **7243** |
| wall clock | 0.0 s | 0.5 s | 7 s | 55 s | 337 s | 1615 s |

The minimised row is the engine's row exactly, including $p=7$ (which the engine reaches
only through its direct ladder — `learnfe` runs out of memory there, §6). Theorem 2.4's
description is therefore correct; it is
just far from tight (the successive $\Phi$-state ratios are
$47,\,9.3,\,5.7,\,4.4,\,3.6$, i.e. log-log slopes $7.7,\,7.8,\,8.1,\,8.3$).

---

## 3. Why this description is not (yet) an $O(p^c)$ construction

By Theorem 2.4 and the structure paragraph, $\Phi(w)$ is determined by the four sets
$\Sigma^{<}_{c,c'}(w)$ (each of the form $\Pi(u,v;\eta)$) together with four points of
$\mathbb Z_p^2$; and the four $\Sigma^{=}$ singletons are determined by the four residues
$s(I{+}\Lambda),s(I{+}\Lambda{+}1),s(J{+}\Lambda),s(J{+}\Lambda{+}1)$. Hence

$$|FE_{T_p}|_{\mathrm{msd}}\ \le\ p^{4}\cdot\big(\#\Pi_p\big)^{4},
\qquad \#\Pi_p:=\#\{\Pi(u,v;\eta)\ :\ u\le v,\ \eta\in\mathbb Z\}. \tag{3.1}$$

So a polynomial bound on $\#\Pi_p$ gives the **first upper bound of any kind** for $T_p$
(gap G3 of `proof-family.md`). Negative $\eta$ costs nothing: transposing coordinates gives
$\Pi(u,v;-\eta)=\Pi(u{-}\eta,v{-}\eta;\eta)^{\mathsf T}$, so restricting to $\eta\ge0$ changes
$\#\Pi_p$ by at most a factor $2$. Two things are known about this quantity.

**(a) The one-dimensional case is proved polynomial.** Taking only the first coordinate,
$\mathrm{Row}\,\Pi(u,v;\eta)=\{s(q)\bmod p:q\in[u,v]\}$ is exactly the *interval image* of
the translation automaton $\mathbb Z_p$ studied in `proof-lower.md` §5. Its Corollary 6
(certified **PROVED** by the referee) says every such image is *two spheres and two
points*, $\{s(u),s(v)\}\cup(g_A+S_{r_A})\cup(g_B+S_{r_B})$ with $S_r=\{0,\dots,r\}$, hence
there are $O(p^6)$ of them (polynomial, which is all that is needed). I re-measured the
count directly
(`explore/sing/dsets.py`): $3,7,15,31,61,113,197$ for $p=2..8$ — identical to
`proof-lower.md`'s $\mathcal I(\mathbb Z_p)$, fourth difference constantly $2$, i.e.
$\sim p^4/12$.

> *Correction to a reading in this repo.* At $p\le5$ these counts are $2^p-1$, and I first
> mis-read them as exponential. They are not: $\mathcal I(\mathbb Z_p)$ is a quartic
> polynomial and the coincidence dies at $p=6$ ($61$ vs $63$), exactly as
> `proof-lower.md` §5 already says.

**(b) The two-dimensional case is open, and is the whole gap.** Two measurements. First,
the number of *distinct single-component invariants* $\Phi_{\Sigma}$, counted exactly over
the reachable states of the $\Phi$-quotient of §2 (`explore/sing/comp.py` — this count is
authoritative, it enumerates every reachable component):

| $p$ | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|
| distinct $\Phi_{\Sigma^{<}}$ | 4 | 44 | 486 | 3957 | 20012 |
| distinct $\Phi_{\Sigma^{=}}$ | 4 | 9 | 16 | 25 | 36 |
| distinct $\mathrm{Row}$ | 3 | 7 | 15 | 31 | 61 |

($\Phi_{\Sigma^{=}}$ is exactly $p^2$, as it must be: a singleton pair.) Second, a direct
enumeration of the sets themselves (`explore/sing/pisets.py`, $\eta\le 2^{p+3}$,
$u\le2^{p+4}$) gives $\#\Pi_p\ \ge\ 15,\ 463,\ 18512,\ 77717,\ 364143$ for $p=2..6$;
these are **lower bounds** — the $p=3,4$ entries are stable under further range increases
(and reproduce $44,486$ after adding the empty set), the $p\ge5$ entries are not yet.

Both look polynomial, and the joint count of Theorem 2.4 is measured
$104,\,4905,\,45440,\,257900,\,1131588,\,4058229$ for $p=2..7$ with successive log-log
slopes $7.7,\,7.8,\,8.1,\,8.3$ — i.e. $\approx p^{8}$, which via Theorem 2.4 would give
$|FE_{T_p}|=O(p^8)$. But I have no proof of any of it. What I can prove is the recursion
that such a proof will run on:

**Proposition 3.1 (2-adic recursion for $\Pi$).** For $\eta\ge0$ let
$\eta_0=\lfloor\eta/2\rfloor$, and let $[u_0,v_0]$, $[u_1,v_1]$ be the images of the even
resp. odd elements of $[u,v]$ under $q\mapsto\lfloor q/2\rfloor$ (both are integer
intervals). Then
$$\Pi(u,v;\eta)=
\begin{cases}
\big(\Pi(u_0,v_0;\eta_0)+(0,0)\big)\ \cup\ \big(\Pi(u_1,v_1;\eta_0)+(1,1)\big), & \eta \text{ even},\\[2pt]
\big(\Pi(u_0,v_0;\eta_0)+(0,1)\big)\ \cup\ \big(\Pi(u_1,v_1;\eta_0+1)+(1,0)\big), & \eta \text{ odd}.
\end{cases}$$

*Proof.* $q=2q_0$: $s(q)=s(q_0)$; if $\eta=2\eta_0$ then $q+\eta=2(q_0+\eta_0)$ and
$s(q+\eta)=s(q_0+\eta_0)$; if $\eta=2\eta_0+1$ then $q+\eta=2(q_0+\eta_0)+1$ and
$s(q+\eta)=s(q_0+\eta_0)+1$. $q=2q_0+1$: $s(q)=s(q_0)+1$; if $\eta$ even,
$q+\eta=2(q_0+\eta_0)+1$ and $s(q+\eta)=s(q_0+\eta_0)+1$; if $\eta$ odd,
$q+\eta=2(q_0+\eta_0+1)$ and $s(q+\eta)=s(q_0+\eta_0+1)$. $\square$

The shift parameter at recursion depth $t$ always lies in
$\{\lfloor\eta/2^t\rfloor,\lfloor\eta/2^t\rfloor+1\}$ — two consecutive integers, never
more — and the intervals at depth $t$ are $[\lfloor u/2^t\rfloor+O(1),\lfloor
v/2^t\rfloor+O(1)]$. So $\Pi(u,v;\eta)$ is a union of $O(\log\eta)$ translates of a
bounded family, with translations in $\mathbb Z_p^2$. **The missing step is the nesting
lemma** — the $\mathbb Z_p^2$ analogue of `proof-lower.md` Lemma 5 — showing that only
$O(1)$ (or $O(p)$) of the depths contribute, which would collapse the union to $O(1)$
"spheres and points" and give $\#\Pi_p=p^{O(1)}$, hence by (3.1)
$|FE_{T_p}|_{\mathrm{msd}}=p^{O(1)}$.

**Honest statement.** No upper bound for $|FE_{T_p}|$ is proved here either. What is new is
that the problem now has one, purely combinatorial, self-contained form: *bound the number
of sets $\{(s(q),s(q+\eta))\bmod p:u\le q\le v\}$*. The one-dimensional version of exactly
this question is already a theorem.

---

## 4. The lower bound: $|FE_{T_p}|_{\mathrm{msd}}\ge p^4$

Throughout this section $p\ge3$ and $V_m:=\{1-x:0\le x\le m\}\subseteq\mathbb Z_p$, so
$V_0=\{1\}\subsetneq V_1\subsetneq\dots\subsetneq V_{p-2}=\mathbb Z_p\setminus\{2\}$ and
$V_m\setminus V_{m-1}=\{1-m\}$.

**The prefixes.** Put $P:=2p+2$. For $(a,\rho)\in\mathbb Z_p\times\{0,\dots,p-1\}$ let
$$x_{a,\rho}\ :=\ 0^{\,P-e-\rho-1}\,1^{e}\,0\,1^{\rho}\in\{0,1\}^P,\qquad e:=(a-\rho)\bmod p,$$
(well defined: $e+\rho+1\le 2p-1\le P$) and let $I_{a,\rho}$ be its value. The explicit
separating $0$ makes the trailing run exact, so
$$s(I_{a,\rho})=e+\rho\equiv a,\qquad \nu(I_{a,\rho})=\rho,\qquad\text{hence by (D2)}\quad
s(I_{a,\rho}+1)\equiv a+1-\rho .$$
Define the length-$P$ prefixes
$$u_{a,\rho,b,\tau}\ :=\ \big(x_{a,\rho},\ x_{b,\tau},\ 0^{P}\big)\in\Sigma_3^P ,$$
so that $I=I_{a,\rho}$, $J=I_{b,\tau}$, $\Lambda=0$. Write
$$A_0:=a,\quad A_1:=a+1-\rho,\quad B_0:=b,\quad B_1:=b+1-\tau \pmod p .$$
$(a,\rho)\mapsto(A_0,A_1)$ is a bijection $\mathbb Z_p\times\{0,\dots,p-1\}\to\mathbb Z_p^2$
(invert by $\rho=A_0+1-A_1$), and likewise for $(b,\tau)$. There are $p^4$ prefixes.

**Reading off a suffix.** Let $z\in\Sigma_3^S$ have values $(i_{lo},j_{lo},l_{lo})$. Since
$\Lambda=0$, $u_{a,\rho,b,\tau}z$ encodes $(i,j,l)=(I2^S+i_{lo},\,J2^S+j_{lo},\,l_{lo})$ and,
by (D1), for $0\le t<l_{lo}$
$$s(i+t)=\begin{cases}A_0+s(i_{lo}+t)&i_{lo}+t<2^S\\ A_1+s(i_{lo}+t-2^S)&i_{lo}+t\ge2^S\end{cases}
\pmod p, \tag{4.0}$$
and symmetrically for $j$ with $B_0,B_1$. So membership of $u z$ in $L_p$ is a statement
about $(A_0,A_1,B_0,B_1)$ only.

### Family 1 — recovers $(A_0,B_0)$

Let $S:=p$. For $x,y\in\{0,\dots,p-1\}$ let $\Xi_{x,y}$ be the suffix with
$i_{lo}=2^{x}-1$, $j_{lo}=2^{y}-1$, $l_{lo}=1$ (all $<2^S$). Only $t=0$ occurs and neither
side crosses $2^S$, so by (4.0)
$$u\,\Xi_{x,y}\in L_p\iff[A_0+x\equiv1]=[B_0+y\equiv1]\iff [x\equiv1{-}A_0]=[y\equiv1{-}B_0].$$
The rejected set is
$N=\big(\{1{-}A_0\}\times(\mathbb Z_p\setminus\{1{-}B_0\})\big)\cup\big((\mathbb Z_p\setminus\{1{-}A_0\})\times\{1{-}B_0\}\big)$.
Row $1{-}A_0$ meets $N$ in $p-1\ge2$ cells, every other row in exactly one; symmetrically
for columns. Hence the residual determines $(A_0,B_0)$.
*(This is the only place $p\ge3$ is used, and it is used sharply: at $p=2$ the keys
$(A_0,B_0)$ and $(A_0{+}1,B_0{+}1)$ give the same $N$, which is why
$|FE_{T_2}|=15<16=2^4$.)*

### Family 2 — recovers $A_1$, given $(A_0,B_0)$

Fix $m\in\{0,\dots,p-2\}$ and choose

* $w\in\mathbb Z_p$ with $w\notin\{1-B_0-x:0\le x\le m\}$ (that set has $m+1\le p-1$
  elements, so $w$ exists);
* $v\in\mathbb Z_p$ with $v\ne 1-B_0-m$;
* $\nu:=(v+1-w)\bmod p\in\{0,\dots,p-1\}$ and $M:=\mathrm{val}\big(1^{(v-\nu)\bmod p}\,0\,1^{\nu}\big)$,
  so $s(M)\equiv v$, $\nu(M)=\nu$ and, by (D2), $s(M+1)\equiv v+1-\nu=w$. Put $n:=M+1\ge1$;
* $S$ any integer with $(n+1)2^{m}<2^{S}$ **and** $S\not\equiv 1-A_0\pmod p$ (both are
  satisfiable: fix the residue of $S$ and take it large).

Let $\Theta_m$ be the suffix of length $S$ with
$$i_{lo}=2^{S}-1,\qquad j_{lo}=n2^{m}-1,\qquad l_{lo}=2^{m}+1 .$$
Then $t$ runs over $\{0,1,\dots,2^m\}$ and:

* $t=0$. $i_{lo}<2^S$, and $s(2^S-1)=S$, so the $i$-value is $A_0+S$. Also
  $j_{lo}=M2^m+(2^m-1)$, so $s(j_{lo})=s(M)+m\equiv v+m$ and the $j$-value is $B_0+v+m$.
  Both $[A_0+S\equiv1]$ and $[B_0+v+m\equiv1]$ are **false** by the choice of $S$ and $v$,
  so this clause holds identically.
* $t=1+r$ with $0\le r\le 2^m-1$. Now $i_{lo}+t=2^S+r\ge2^S$, so by (4.0) the $i$-value is
  $A_1+s(r)$; and $j_{lo}+t=n2^m+r<2^S$ with $s(n2^m+r)=s(n)+s(r)\equiv w+s(r)$, so the
  $j$-value is $B_0+w+s(r)$. The latter is $\ne1$ for every $r$, because
  $s(r)\in\{0,\dots,m\}$ and $w\notin\{1-B_0-x:0\le x\le m\}$.

Since $\{s(r):0\le r\le2^m-1\}=\{0,1,\dots,m\}$, the whole conjunction reduces to
$$u\,\Theta_m\in L_p\iff \forall x\in\{0,\dots,m\}:\ A_1+x\not\equiv 1
\iff A_1\notin V_m .$$

### Family 3 — recovers $B_1$, given $(A_0,B_0)$

Identical with the two tracks exchanged ($L_p$ is symmetric in $i\leftrightarrow j$): take
$j_{lo}=2^S-1$, $i_{lo}=n'2^m-1$ with $s(n')\equiv w'\notin\{1-A_0-x:0\le x\le m\}$,
$s(n'-1)\equiv v'\ne 1-A_0-m$, and $S\not\equiv 1-B_0$. Then
$u\,\Theta'_m\in L_p\iff B_1\notin V_m$.

### Theorem 4.1

**Theorem 4.1.** *For every $p\ge3$, the minimal msd DFA of $FE_{T_p}$ has at least $p^4$
states.*

*Proof.* Take two distinct keys $(A_0,A_1,B_0,B_1)\ne(A_0',A_1',B_0',B_1')$.
If $(A_0,B_0)\ne(A_0',B_0')$, Family 1 separates them (the rejected sets $N,N'$ differ, so
some $\Xi_{x,y}$ separates). Otherwise $(A_0,B_0)=(A_0',B_0')$ and we may build Families 2
and 3 for this common pair. If $A_1\ne A_1'$, put
$m_1=\min\{m\le p-2: A_1\in V_m\}$ ($=\infty$ if $A_1=2$) and likewise $m_1'$; these differ
because $V_m\setminus V_{m-1}=\{1-m\}$, and $m:=\min(m_1,m_1')\le p-2$ gives a $\Theta_m$
accepted for exactly one of the two. If $A_1=A_1'$ then $B_1\ne B_1'$ and Family 3 does the
same. So the $p^4$ prefixes $u_{a,\rho,b,\tau}$ are pairwise Myhill–Nerode inequivalent.
$\square$

**Machine check (independent of every automaton in this repo).**
`explore/sing/lbproof.py` builds the prefixes and the three suffix families literally as
above and evaluates $FE$ **from its definition** (binary digit sums of $i+t$, $j+t$). The
number of distinct response vectors over the $p^4$ keys is

| $p$ | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|
| keys $p^4$ | 81 | 256 | 625 | 1296 | 2401 |
| distinct response vectors | **81** | **256** | **625** | **1296** | **2401** |
| collisions | 0 | 0 | 0 | 0 | 0 |

A second, independent check (`explore/sing/lb.py`) pushes the same $p^4$ prefixes through
the *minimised* DFA of §2 and finds $81/256/625$ distinct **states** at $p=3,4,5$.

**Remark 4.2 (how much is left on the table).** Allowing the $l$-track prefix to be
non-zero enlarges the fooling set: with $\Lambda\in\{0,1\}$ one reaches
$139/452/1095$ distinct states at $p=3,4,5$ (i.e. $1.7$–$1.8\,p^4$), and with
$\Lambda<8$, $145/494/1221$ ($\approx1.95\,p^4$), against the true $190/698/1877$
($\approx3\,p^4$). So the constant $1$ in Theorem 4.1 is not optimal; the exponent
apparently is.

**Corollary 4.3 (G4, lower half).**
$$|FE_{T_p}|_{\mathrm{msd}}\ \ge\ p^4 = m^4 .$$
In particular $|FE_{T_p}|\ne O(p^3)$: the reading (b) of `proof-family.md` §5.4 — "degree
$3$ with a large constant, $\approx44p^3$, i.e. the singleton coding costs a *constant*
factor" — is **refuted**, as is the cubic fit $44.4p^3-205.4p^2+308.5p-88.7$ as an
asymptotic law. Relative to `proof-family.md` Thm 4.5 ($|FE_{G_p}|_{\mathrm{msd}}=p^3+8$),
the lossy singleton coding of the *same* $p$-state DFAO costs at least a factor
$\Theta(p)$, matching the naive reading of the level grading (Prop. 5.3 there) rather than
contradicting it.

Measured against the data:

| $p$ | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|
| $\lvert FE_{T_p}\rvert_{\mathrm{msd}}$ | 190 | 698 | 1877 | 3971 | 7243 | 11988 |
| $p^4$ (Thm 4.1) | 81 | 256 | 625 | 1296 | 2401 | 4096 |
| ratio | 2.35 | 2.73 | 3.00 | 3.06 | 3.02 | 2.93 |

The ratio is flat at $\approx3$ over the whole range, which is why the *local* log-log
slope of the raw sequence ($4.52\to3.77$, the observation that made `proof-family.md` G4
suspect a cubic) is a finite-size artefact of a $\Theta(p^4)$ sequence with a large
negative lower-order part, not evidence for degree $3$.

**Conjecture 4.4.** $|FE_{T_p}|_{\mathrm{msd}}=\Theta(p^4)$, with
$|FE_{T_p}|/p^4\to c\approx 2.9$.

---

## 5. Singleton rigidity: why the $G_p$ technique does not transfer

`proof-family.md` Lemma 3.3 is the engine of the whole $G_p$ analysis: for the *identity*
coding, two positions of opposite parity have longest common extension $\le3$, so after
desubstituting to the lowest differing bit only $L\le3$ blocks ever matter and the whole
predicate collapses to four residues. The corresponding constant for the singleton coding
is exponential.

**Lemma 5.1 (measured).** For $2\le p\le10$,
$$\max\{\,l\ :\ \exists i,\ d\ \text{odd},\ FE_{T_p}(i,i+d,l)\,\}\ =\ 2^{p}-1 ,$$
the maximum being attained at $d=2^p-1$ and, for $p\ge3$, at $i=\mathrm{val}(1^{p+1}0^{p-2}1)$ — e.g.
$p=4$: $d=15$, $i=249=\mathtt{11111001}_2$, $l=15$; $p=8$: $d=255$, $i=65409$, $l=255$.

*Evidence.* `explore/sing/maxrun.py` scans all odd $d\le1023$ and all $i<2^{21}$ for
$p=2,\dots,10$ and returns $3,7,15,31,63,127,255,511,1023$ — exactly $2^p-1$, with the
maximiser $d=2^p-1$ in every case. `explore/sing/rigid.py` confirms the stated witness
$i=\mathrm{val}(1^{p+1}0^{p-2}1)$ for $3\le p\le8$. The upper bound $l\le2^p-1$ is **not
proved**; the lower bound is a finite verification for each listed $p$.

*Why $2^p$ is the natural scale.* Any interval of $2^{p-1}$ consecutive integers that is
$2^{p-1}$-aligned has digit sums $s(a)+\{0,\dots,p-1\}=\mathbb Z_p$, so the $1$-positions of
$T_p$ have gaps $<2^{p}$; a window can be free of the coding's single distinguished letter
on both sides for at most that long.

**Consequence.** Any construction of the `proof-family.md` §4.1 type — "desubstitute to the
lowest differing bit, then only $O(1)$ blocks matter" — is impossible for $T_p$: at level
$k=0$ the number of relevant blocks is $\Theta(2^p)$. This is the structural reason the
upper bound is hard, and it is consistent with §3: the state has to summarise an
exponentially long window, and it does so through the *set* $\Pi$, which is (conjecturally)
of polynomial variety even though the window is not of polynomial length.

---

## 6. Attempted verification at $p=9,10$

Both routes named in the task fail on this machine (24 GB, 17.6 GB free at launch), which
confirms and extends the referee's finding.

| run | budget | outcome |
|---|---|---|
| `learnfe FE`, $p=6$ | 4 GB | **OK, states $=3971$**, 182 s (`iters=554 eqs=406 mqs=1.67e7`) |
| `learnfe FE`, $p=7$ | 4 GB | `ERR memory budget exceeded`, 54 s |
| `learnfe FE`, $p=7,8,9$ | 6 GB | killed by the RSS watchdog ($\approx9.5$ GB), 49 s / 29 s / 24 s |
| direct `let FE`, $p=9$, `AM_CAP=50000` | 6 GB | killed by the watchdog, 430 s (same failure mode as the referee's $p=9$ run) |
| direct `let FE`, $p=10$, `AM_CAP=50000` | 6 GB | killed by the watchdog, 446 s |
| own builder (§2), $p\le6$ | — | $15,190,698,1877,3971$ in $0.0/0.5/7/55/337$ s |
| own builder, $p=7$ | 3.2 GB peak | **$|FE_{T_7}|=7243$**, $4\,058\,229$ $\Phi$-states, 1615 s — an independent reproduction of the engine's direct-ladder value |
| own builder, $p\ge8$ | — | projected $\ge1.3\times10^7$ $\Phi$-states, out of reach in Python |

So $|FE_{T_9}|,|FE_{T_{10}}|$ remain unmeasured. **This no longer matters for the question
that was asked.** The two competing fits of `proof-family.md` §5.4 predict
$18446$ (cubic) vs $18522$ (quartic) at $p=9$ and $26894$ vs $27189$ at $p=10$ — they are
$0.4\%$ and $1\%$ apart, so neither point could ever have separated them. Theorem 4.1
separates them outright, without any new measurement: no cubic can bound a sequence that
is $\ge p^4$.

---

## 7. What this settles, and what it does not

| claim | status |
|---|---|
| Lemma 1.2 (pair-set reformulation of $FE_{T_p}$) | **PROVED** (and equals `proof-family.md` Prop. 5.1 after block splitting) |
| Lemma 2.3 / Theorem 2.4 (residual determined by the 8-tuple $\Phi$; explicit DFA) | **PROVED**; implemented, minimises to the engine's $15,190,698,1877,3971,7243$ for $p\le7$ |
| Proposition 3.1 (2-adic recursion for $\Pi$) | **PROVED** |
| $\#\Pi_p=p^{O(1)}$, hence $\lvert FE_{T_p}\rvert=p^{O(1)}$ | **OPEN** — reduced to a 2-dimensional interval-image count; the 1-dimensional case is `proof-lower.md` Cor. 6, $p^{O(1)}$ (measured $\sim p^4/12$) |
| **Theorem 4.1: $\lvert FE_{T_p}\rvert_{\mathrm{msd}}\ge p^4$, $p\ge3$** | **PROVED**; machine-checked from the definition for $p=3..7$ |
| Corollary 4.3: $\lvert FE_{T_p}\rvert\ne O(p^3)$; `proof-family.md` §5.4 reading (b) refuted | **PROVED** |
| $\lvert FE_{T_p}\rvert=\Theta(p^4)$ | **CONJECTURE** (ratio $\approx3$, flat over $p=3..8$); needs the §3 upper bound |
| Lemma 5.1: rigidity constant $=2^p-1$ | **MEASURED** ($p\le10$); attainment verified, upper bound unproved |
| $\lvert FE_{T_9}\rvert,\lvert FE_{T_{10}}\rvert$ | **NOT OBTAINED** (both engine routes OOM; §6) |
| $O(p^c)$ construction as requested | **NOT DELIVERED** — §2 is explicit and correct but measured $\approx p^8$, with no proof of any polynomial bound |

**Gaps, stated plainly.**

* **H1.** No upper bound. §3 reduces it to $\#\Pi_p=p^{O(1)}$ and supplies the recursion
  (Prop. 3.1); the missing ingredient is the $\mathbb Z_p^2$ nesting lemma. I regard this
  as the single most valuable next step in the whole $T_p$ line.
* **H2.** The constant in Theorem 4.1 is $1$, the truth is $\approx3$ (Remark 4.2). A
  fooling set including $\Lambda\ne0$ prefixes should give $\ge2p^4$; I did not write the
  suffix families for it.
* **H3.** Lemma 5.1's upper bound $l\le2^p-1$ is measured, not proved. The residue-pair
  dual of Prop. 3.1 is the right recursion: writing $Q(\alpha,\beta;W;\eta)$ for
  "$[s(q)\equiv\alpha]=[s(q+\eta)\equiv\beta]$ on $W$", halving splits $Q(\alpha,\beta;W;\eta)$
  into $Q(\alpha,\beta{-}1;\cdot;\lfloor\eta/2\rfloor)$ and
  $Q(\alpha{-}1,\beta;\cdot;\lfloor\eta/2\rfloor{+}1)$ when $\eta$ is odd, and into
  $Q(\alpha,\beta;\cdot;\eta/2)$ and $Q(\alpha{-}1,\beta{-}1;\cdot;\eta/2)$ when $\eta$ is even
  — so $\beta-\alpha$ moves by $\pm1$ exactly at the odd steps and each step halves $|W|$.
  After $p$ odd steps the pair has wrapped, which is why the scale is $2^p$; turning that
  into a contradiction is what I did not finish.
* **H4.** Everything is msd, $k=2$. The lsd direction for $T_p$ ($22,656,6154$ for
  $p=2,3,4$) is untouched here.
* **H5.** Theorem 2.4's bound (3.1) is loose by a measured factor $\approx p^4$
  ($1131588$ vs $3971$ at $p=6$): the $\Phi$-invariant is *sufficient* for Nerode
  equivalence but far from necessary, because the query points $(x,y)$ that a suffix can
  actually present are themselves of the restricted form $\Pi$. Exploiting that would both
  tighten (3.1) and make the builder reach $p\ge8$.

---

## 8. Reproduction

```
cd /Users/andrew/maths
.venv/bin/python explore/sing/sub2.py 4        # from-scratch DFA: 15, 190, 698
.venv/bin/python explore/sing/phidfa.py 2 7    # Phi-quotient + Moore: 15,190,698,1877,3971,7243
.venv/bin/python explore/sing/lbproof.py 3 7   # Theorem 4.1, FE from the definition
.venv/bin/python explore/sing/lb.py            # p^4 keys -> p^4 distinct minimal-DFA states
.venv/bin/python explore/sing/lb2.py           # Remark 4.2 (Lambda != 0 families)
.venv/bin/python explore/sing/maxrun.py        # Lemma 5.1 (rigidity constant 2^p-1)
.venv/bin/python explore/sing/rigid.py         # the extremal witness 1^{p+1} 0^{p-2} 1
.venv/bin/python explore/sing/dsets.py         # #interval images = I(Z_p) = 3,7,15,31,61,113,197
.venv/bin/python explore/sing/pisets.py        # #Pi_p and #Phi_Pi
.venv/bin/python explore/sing/comp.py          # per-component counts (Row / Iso / Phi)
.venv/bin/python explore/sing/lfe.py 3 4 5     # engine learnfe: 190, 698, 1877
```

Engine one-liner (msd, $p=5$ singleton coding):

```
mode msd
def T 2 5 0 01 12 23 34 40 01000
learnfe FE
```
gives `states=1877`.

Logs: `results/singleton_learnfe.log`, `results/singleton_big2.log`,
`results/singleton_phidfa7.log`, `results/singleton_lb67.log`,
`results/singleton_pisets.log`, `results/singleton_comp6.log`.
