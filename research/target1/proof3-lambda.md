# $\Lambda$ is superpolynomial: an explicit family, and what survives

**Status: the question is ANSWERED, negatively.** Definition 5.3 of
`paper/proof-upper.md` defines a morphism-only invariant $\Lambda(T)$ and Theorem 5.4
proves $|\mathrm{FE}_{\mathrm{msd}}(T)|\le m^4+m^6+m^8\Lambda(T)^2$, leaving
"$\Lambda=\mathrm{poly}(m)$?" as the sole obstruction to an unconditional polynomial
bound. This document constructs an explicit family of **binary, $2$-automatic**
sequences with

$$\Lambda(T)\;\ge\;\prod_{i\le g}(1+p_i)\;=\;\exp\bigl((1+o(1))\sqrt{m\log m}\bigr),$$

$p_i$ the $i$-th odd prime and $m$ the size of the minimal DFAO. So $\Lambda$ is not
bounded by any polynomial in $m$, Theorem 5.4 can never be made unconditional as
stated, and Open Problem 1(A) cannot be closed along that route.

Three further contributions, in decreasing order of how much they matter:

1. **The repair (§6).** Theorem 5.4's proof never needs the full intersection closure:
   it needs only the pairs $(A^+,A^-)$ that *actually occur* at some prefix $(I,J,L)$.
   Calling that count $\Lambda^\ast(T)$ gives the strictly stronger and still
   unconditional
   $$|\mathrm{FE}_{\mathrm{msd}}(T)|\ \le\ m^4+m^6+m^8\,\Lambda^\ast(T),\qquad
     \Lambda^\ast\le\Lambda^2,$$
   and the counterexample **does not** touch $\Lambda^\ast$: on the same family
   $\Lambda^\ast$ is measured at $38,259,297$ for $m=5,11,19$ while $\Lambda$ is
   $53,816,15583$. The open problem should be restated for $\Lambda^\ast$.
2. **Structure (§2).** $\mathrm{Sh}_{s,\varepsilon}\subseteq Q^3$ is the join of two
   *binary* relations on $Q$: $\mathrm{Sh}_{s,\varepsilon}=\{(u,a,b):(u,a)\in
   A_{s,\varepsilon},\ (b,u)\in A_{s,k^s-\varepsilon}\}$ where
   $A_{s,h}=\{(p,q):B_s(p)[0{:}k^s{-}h]=B_s(q)[h{:}k^s]\}$. The triple structure of
   Definition 4.1 in `proof-upper.md` is an artifact of the bookkeeping; everything about
   $\Lambda$ is the arithmetic of *block alignments*. This is what made the
   counterexample findable.
3. **Invariance (§3).** $\Lambda$ is unchanged by the Myhill–Nerode quotient, so it may
   be computed on any DFAO for $T$ and one never has to prove minimality of a
   construction.

Everything below is machine-checked by `paper/proof3-lambda-check.py` (brute force over
explicit blocks; no engine, no automata library); all checks report 0 violations, and
Theorem 4.4 is additionally verified *head-on* by enumerating the intersections it
claims are distinct. The $|\mathrm{FE}|$ figures come from the engine (`learnfe`,
cross-checked at $m\le11$ by a direct `let FE(i,j,l) A t. (t<l) => T[i+t]=T[j+t]`
subset construction).

---

## 1. Setting

Notation is that of `paper/proof-upper.md` §1 and §5, repeated here so this file stands
alone. Fix $k\ge2$; let $T$ be $k$-automatic with minimal msd DFAO
$M=(Q,\Sigma_k,\delta,q_0,\tau)$, $m=|Q|$, $\delta(q_0,0)=q_0$, and for $q\in Q$, $s\ge0$
let
$$B_s(q):=\bigl(\tau(\delta(q,\mathrm{rep}_s(y)))\bigr)_{0\le y<k^s}\in\Delta^{k^s}$$
be the length-$k^s$ **block** of $q$ (Definition 1.1 there). Recall
$B_s(q)=B_{s-1}(\delta(q,0))\cdots B_{s-1}(\delta(q,k-1))$ (Lemma 1.3 there).

**Definition 1.1** (`proof-upper.md` Def. 4.1). For $s\ge0$, $0\le\varepsilon<k^s$,
$$\mathrm{Sh}_{s,\varepsilon}:=\bigl\{(u,a,b)\in Q^3:\
B_s(u)=(B_s(a)B_s(b))[\varepsilon:\varepsilon+k^s]\bigr\}.$$

**Definition 1.2** (`proof-upper.md` Def. 5.2, 5.3). $\mathcal S(T):=\{\mathrm{Sh}_{s,\varepsilon}\}
\subseteq2^{Q^3}$ (equivalently the orbit of $\mathrm{Sh}_{0,0}$ under the operators
$\Phi_d$), $\gamma(T):=|\mathcal S(T)|$; for $t\in Q^3$, $P_t:=\{X\in\mathcal S(T):t\in X\}$;
and
$$\Lambda(T):=\Bigl|\bigl\{\ \mathcal S_{\mathcal T}\ :\ \mathcal T\subseteq Q^3\ \bigr\}\Bigr|,
\qquad
\mathcal S_{\mathcal T}:=\bigcap_{t\in\mathcal T}P_t=\{X\in\mathcal S(T):\mathcal T\subseteq X\}$$
(the empty intersection being $\mathcal S(T)$).

Theorem 5.4 of `proof-upper.md` bounds $|\mathrm{FE}_{\mathrm{msd}}(T)|$ by
$m^4+m^6+m^8\Lambda^2$, because for $L\ge2$ the residual of the prefix $(I,J,L)$ is
determined by $8$ DFAO states together with the middle language $\Theta_{I,J,L}$, and
$\Theta_{I,J,L}$ is determined by the pair
$$A^{+}_{I,J,L}=\mathcal S_{\mathcal T^{+}_{I,J,L}},\qquad
  A^{-}_{I,J,L}=\mathcal S_{\mathcal T^{-}_{I,J,L}},$$
$$\mathcal T^{+}_{I,J,L}=\{(q_{I+c},q_{J+c},q_{J+c+1}):1\le c\le L-1\},\quad
  \mathcal T^{-}_{I,J,L}=\{(q_{I+c},q_{J+c-1},q_{J+c}):1\le c\le L-1\}.$$

---

## 2. $\mathrm{Sh}$ is a pair of binary alignment relations

**Definition 2.1 (alignment relation).** For $s\ge0$ and $0\le h\le k^s$,
$$A_{s,h}\ :=\ \bigl\{(p,q)\in Q^2\ :\ B_s(p)[0:k^s-h]=B_s(q)[h:k^s]\bigr\}.$$
So $A_{s,0}=\{(p,q):B_s(p)=B_s(q)\}$ and $A_{s,k^s}=Q^2$ (empty comparison).
Read: *the block of $p$ agrees with the block of $q$ shifted left by $h$, as far as the
block of $q$ goes*.

**Lemma 2.2 (pair form).** For all $s\ge0$ and $0\le\varepsilon<k^s$,
$$\mathrm{Sh}_{s,\varepsilon}\;=\;\bigl\{(u,a,b)\in Q^3\ :\ (u,a)\in A_{s,\varepsilon}
\ \text{ and }\ (b,u)\in A_{s,\,k^s-\varepsilon}\bigr\}.$$

*Proof.* Write $N=k^s$. The defining condition is $B_s(u)[y]=(B_s(a)B_s(b))[\varepsilon+y]$
for all $0\le y<N$. Split at $\varepsilon+y=N$.
For $y<N-\varepsilon$ the right side is $B_s(a)[\varepsilon+y]$, so those $y$ say exactly
$B_s(u)[0:N-\varepsilon]=B_s(a)[\varepsilon:N]$, i.e. $(u,a)\in A_{s,\varepsilon}$.
For $y\ge N-\varepsilon$ put $z=y-(N-\varepsilon)\in[0,\varepsilon)$; the right side is
$B_s(b)[z]$, so those $y$ say exactly $B_s(b)[0:\varepsilon]=B_s(u)[N-\varepsilon:N]$,
which is $(b,u)\in A_{s,N-\varepsilon}$ (Definition 2.1 with $h=N-\varepsilon$, whose
comparison length is $N-h=\varepsilon$). $\square$

*Machine check.* `[Lemma A]` in `proof3-lambda-check.py`: $0$ violations over all
$(s,\varepsilon)$ with $s\le4$ ($s\le3$ for $k=3$) on the five stock DFAOs of
`proof-upper-check.py`, and $s\le5$ on every member of the family of §4.

**Corollary 2.3.** Write $\pi_{12}(u,a,b)=(u,a)$ and $\pi_{31}(u,a,b)=(b,u)$. For every
$\mathcal T\subseteq Q^3$,
$$\mathcal T\subseteq\mathrm{Sh}_{s,\varepsilon}\iff
\pi_{12}(\mathcal T)\subseteq A_{s,\varepsilon}\ \text{ and }\
\pi_{31}(\mathcal T)\subseteq A_{s,k^s-\varepsilon}.$$
Consequently, with $\alpha(T):=|\{A_{s,h}\}|$ and $\Lambda_2(T)$ the size of the
intersection closure of the up-sets $\{A\in\{A_{s,h}\}:R\subseteq A\}$, $R\subseteq Q^2$:
$$\gamma\le\alpha^2,\qquad \Lambda\le\Lambda_2^{\,2}.$$
(For the first: $\mathrm{Sh}_{s,\varepsilon}$ is determined by the ordered pair
$(A_{s,\varepsilon},A_{s,k^s-\varepsilon})$. For the second: by the displayed
equivalence, $\mathcal S_{\mathcal T}$ is determined by the pair of up-sets
$U_{\pi_{12}(\mathcal T)},U_{\pi_{31}(\mathcal T)}$, and each up-set lies in the
closure counted by $\Lambda_2$.)
So $\Lambda$ is not really about triples: it is the closure arithmetic of a family of
**binary relations on $Q$**, indexed by a scale $s$ and a shift $h$. The construction in
§4 exploits exactly this — it manufactures many distinct $A_{s,h}$ by making the shift
$h$ visible modulo many coprime numbers at once.

**Lemma 2.4 (descent for $A$; not used below, but it is how $A_{s,h}$ is computed).**
Let $s\ge1$, $0\le h\le k^s$, and write $h=dk^{s-1}+h'$ with $0\le h'<k^{s-1}$. If $d=k$
(i.e. $h=k^s$) then $A_{s,h}=Q^2$. Otherwise
$$A_{s,h}=\Bigl\{(p,q):\bigl(\delta(p,k{-}1{-}d),\delta(q,k{-}1)\bigr)\in A_{s-1,h'}
\ \wedge\ \forall r<k{-}1{-}d:\ \bigl(\delta(p,r),\delta(q,r{+}d),\delta(q,r{+}d{+}1)\bigr)
\in\mathrm{Sh}_{s-1,h'}\Bigr\}.$$

*Proof.* Put $K=k^{s-1}$ and $y=rK+y'$, $0\le y'<K$. Then $B_s(p)[y]=B_{s-1}(\delta(p,r))[y']$
and $h+y=(r+d)K+(h'+y')$ with $h'+y'<2K$, so $B_s(q)[h+y]=
\bigl(B_{s-1}(\delta(q,r{+}d))B_{s-1}(\delta(q,r{+}d{+}1))\bigr)[h'+y']$ whenever the
second block is needed. The range $0\le y<k^s-h=(k-d)K-h'$ is: chunks
$r=0,\dots,k-2-d$ in full, and chunk $r=k-1-d$ truncated to $y'<K-h'$. A full chunk $r$
says $B_{s-1}(\delta(p,r))=\bigl(B_{s-1}(\delta(q,r{+}d))B_{s-1}(\delta(q,r{+}d{+}1))\bigr)
[h':h'+K]$, i.e. the $\mathrm{Sh}_{s-1,h'}$ condition (and $r+d+1\le k-1$, so
$\delta(q,r{+}d{+}1)$ is defined); the truncated chunk says
$B_{s-1}(\delta(p,k{-}1{-}d))[0:K-h']=B_{s-1}(\delta(q,k{-}1))[h':K]$, i.e. the
$A_{s-1,h'}$ condition. $\square$

*Machine check.* `[Lemma 2.4 descent]`: $0$ violations, all $s\le4$, all $h\le k^s$, five
stock DFAOs and every member of the family of §4.

---

## 3. $\Lambda$ is a quotient invariant

**Lemma 3.1.** Let $N=(Q_N,\Sigma_k,\delta,q_0,\tau)$ be *any* DFAO for $T$ with all
states reachable and $\delta(q_0,0)=q_0$, and let $M=N/{\sim}$ be its Myhill–Nerode
quotient (the minimal DFAO), with projection $\pi:Q_N\to Q$. Then
$$\gamma(N)=\gamma(M)\quad\text{and}\quad\Lambda(N)=\Lambda(M).$$

*Proof.* $q\sim q'$ iff $\tau(\delta(q,w))=\tau(\delta(q',w))$ for all $w$, iff
$B_s(q)=B_s(q')$ for all $s$; and $B_s^N(q)=B_s^M(\pi q)$. Since
$\mathrm{Sh}_{s,\varepsilon}$ is defined by equalities between blocks,
$\mathrm{Sh}^N_{s,\varepsilon}=(\pi^3)^{-1}\bigl(\mathrm{Sh}^M_{s,\varepsilon}\bigr)$.
As $\pi$ is onto, $X\mapsto(\pi^3)^{-1}(X)$ is injective, so
$\mathcal S(N)=(\pi^3)^{-1}\mathcal S(M)$ and $\gamma(N)=\gamma(M)$. For
$\mathcal T\subseteq Q_N^3$ and $X\subseteq Q^3$ we have
$\mathcal T\subseteq(\pi^3)^{-1}(X)\iff\pi^3(\mathcal T)\subseteq X$, hence
$\mathcal S_{\mathcal T}(N)$ corresponds to $\mathcal S_{\pi^3(\mathcal T)}(M)$ under the
same bijection, and $\pi^3(\mathcal T)$ runs over all subsets of $Q^3$ as $\mathcal T$
runs over all subsets of $Q_N^3$. The two closures are therefore in bijection. $\square$

*Machine check.* `[Lemma B]`: for each of the five stock DFAOs, cloning one state (and
rerouting one incoming edge to the clone) leaves $\Lambda$ unchanged: $4,17,28,13,40$
before and after.

Lemma 3.1 is a convenience: the automaton of §4 need not be proved minimal for the lower
bound on $\Lambda$ to be a lower bound for the *minimal* DFAO of the same sequence. (In
fact it is minimal for every case computed — reported by the checker — but nothing
depends on that.)

---

## 4. The CRT family

The idea. By Lemma 2.2, $\Lambda$ counts how finely the family of alignment relations
$A_{s,h}$ can resolve a subset of $Q^2$. Take a purely periodic gadget of period $p$: it
costs $p$ states, and its alignment relation sees the shift $h$ **modulo $p$**. Put $g$
gadgets with pairwise coprime periods side by side inside one automaton: the state count
is the *sum* $\sum p_i$, but the shifts they jointly resolve is the *product*
$\prod p_i$, by CRT. Sum versus product of pairwise coprime numbers is exactly the gap
between $m$ and $e^{\sqrt{m\log m}}$.

**Definition 4.1 (the automaton $C_{p_1,\dots,p_g}$).** Fix $k=2$ and odd primes
$p_1<\dots<p_g$. States:
* routers $r_0,r_1,\dots,r_g$;
* gadget states $(i,x)$ for $1\le i\le g$, $x\in\mathbb Z_{p_i}$.

Transitions:
$$\delta(r_0,0)=r_0,\quad\delta(r_0,1)=r_1;\qquad
\delta(r_j,1)=(j,0),\quad
\delta(r_j,0)=\begin{cases}r_{j+1}&j<g\\ (g,0)&j=g\end{cases}\ (1\le j\le g);$$
$$\delta\bigl((i,x),b\bigr)=\bigl(i,\ (2x+b)\bmod p_i\bigr)\quad(b\in\{0,1\}).$$
Output: $\tau((i,x))=[\,x=0\,]$, $\tau(r_j)=0$; so $\Delta=\{0,1\}$. The initial state is
$q_0=r_0$ and $\delta(q_0,0)=q_0$, as required. Every state is reachable, so
$$m\ \le\ |Q|\ =\ 1+g+\sum_{i\le g}p_i .$$
Write $T=T_{p_1,\dots,p_g}$ for the resulting binary $2$-automatic sequence. (Reading
msd, an integer $n$ with binary expansion $1\,0^{a}1x$ is routed to gadget $a+1$ and then
run through that gadget's odometer; so $T$ is a concatenation of windows of the periodic
sequences $[\,n\equiv0\bmod p_i\,]$, laid out in blocks of exponentially varying length.)

**Lemma 4.2 (blocks).** For all $i\le g$, $x\in\mathbb Z_{p_i}$, $s\ge0$, $0\le y<2^s$:
$$B_s\bigl((i,x)\bigr)[y]=\bigl[\ x\cdot2^s+y\equiv0\ (\mathrm{mod}\ p_i)\ \bigr].$$

*Proof.* Induction on $s$: reading $\mathrm{rep}_s(y)=d_1\cdots d_s$ from $(i,x)$ lands in
$\bigl(i,(x2^s+y)\bmod p_i\bigr)$, because each digit performs $x\mapsto2x+d$; apply
$\tau$. $\square$

*Machine check.* `[block formula]`: $0$ violations, $s\le8$, all gadgets, $g\le3$.

**Lemma 4.3 (one triple $=$ one congruence).** For $1\le i\le g$ and $c\in\mathbb Z_{p_i}$ put
$$t_{i,c}\ :=\ \bigl((i,c),\,(i,0),\,(i,1)\bigr)\in Q^3 .$$
Let $s\ge0$ and $0\le\varepsilon<2^s$. Then

**(a)** if $\varepsilon\equiv c\,2^s\pmod{p_i}$ then $t_{i,c}\in\mathrm{Sh}_{s,\varepsilon}$;

**(b)** if $p_i\le\varepsilon\le2^s-p_i$ and $t_{i,c}\in\mathrm{Sh}_{s,\varepsilon}$ then
$\varepsilon\equiv c\,2^s\pmod{p_i}$.

*Proof.* By Lemma 2.2, $t_{i,c}\in\mathrm{Sh}_{s,\varepsilon}$ iff both
$$(\mathrm I)\ \ \bigl((i,c),(i,0)\bigr)\in A_{s,\varepsilon},\qquad
(\mathrm{II})\ \ \bigl((i,1),(i,c)\bigr)\in A_{s,2^s-\varepsilon}.$$
Write $p=p_i$, $N=2^s$. By Lemma 4.2, (I) says
$[\,cN+y\equiv0\,]=[\,\varepsilon+y\equiv0\,]$ (mod $p$) for all $0\le y<N-\varepsilon$,
i.e. the two arithmetic progressions $\{y<N-\varepsilon: y\equiv-cN\}$ and
$\{y<N-\varepsilon:y\equiv-\varepsilon\}$ coincide. If $\varepsilon\equiv cN$ this holds
for every $s,\varepsilon$; if the comparison length $N-\varepsilon$ is $\ge p$ both
progressions are non-empty, and equality forces their first elements to agree, i.e.
$\varepsilon\equiv cN\ (p)$.
Similarly (II) says $[\,N+z\equiv0\,]=[\,cN+(N-\varepsilon)+z\equiv0\,]$ for all
$0\le z<\varepsilon$; it holds whenever $-N\equiv\varepsilon-(c+1)N$, i.e.
$\varepsilon\equiv cN\ (p)$, and if $\varepsilon\ge p$ it forces that congruence.
So both conditions are implied by $\varepsilon\equiv cN\ (p)$ — giving (a) — and, in the
stated range, each of them forces it — giving (b). $\square$

*Machine check.* `[triples]`: $0$ violations over all $3\le s\le9$, all
$\max_ip_i\le\varepsilon\le2^s-\max_ip_i$, all $i\le g$, all $c<p_i$, for
$g=1,2,3$ (24 525 instances).

**Theorem 4.4 (main).** $\displaystyle \Lambda\bigl(T_{p_1,\dots,p_g}\bigr)\ \ge\
\prod_{i=1}^{g}(1+p_i).$

*Proof.* Index the intersections by pairs $(I,c)$ where $I\subseteq\{1,\dots,g\}$ and
$c=(c_i)_{i\in I}\in\prod_{i\in I}\mathbb Z_{p_i}$; there are exactly
$\sum_{I}\prod_{i\in I}p_i=\prod_i(1+p_i)$ of them. Put
$$\mathcal T_{I,c}:=\{\,t_{i,c_i}\ :\ i\in I\,\}\subseteq Q^3,\qquad
  \mathcal S_{I,c}:=\mathcal S_{\mathcal T_{I,c}}=\bigcap_{i\in I}P_{t_{i,c_i}} .$$
Each $\mathcal S_{I,c}$ lies in the intersection closure of Definition 1.2, so it
suffices to show $(I,c)\mapsto\mathcal S_{I,c}$ is injective.

Let $(I,c)\ne(I',c')$. Swapping the two if necessary, there is $i\in I$ with either
$i\notin I'$, or $i\in I'$ and $c_i\ne c'_i$. Put $P:=\prod_{j\le g}p_j$ and consider the
system of congruences on $\varepsilon$
$$\varepsilon\equiv c'_j2^s\ (p_j)\ \ (j\in I'),\qquad\text{and, if }i\notin I',\quad
\varepsilon\equiv(c_i+1)2^s\ (p_i).$$
The moduli are pairwise coprime, so by CRT the solution set is a full residue class
modulo some divisor of $P$; choose $s$ with $2^s>2P+2p_g$ and choose $\varepsilon$ in
that class with $p_g\le\varepsilon\le2^s-p_g$. Every solution satisfies
$\varepsilon\not\equiv c_i2^s\ (p_i)$: if $i\notin I'$ because
$(c_i+1)2^s\not\equiv c_i2^s$ ($2$ is invertible mod $p_i$), and if $i\in I'$ because
$c'_i2^s\not\equiv c_i2^s$ as $c'_i\ne c_i$.

Set $X:=\mathrm{Sh}_{s,\varepsilon}\in\mathcal S(T)$. By Lemma 4.3(a),
$t_{j,c'_j}\in X$ for every $j\in I'$, so $X\in\mathcal S_{I',c'}$. By Lemma 4.3(b) —
applicable since $p_i\le\varepsilon\le2^s-p_i$ — $t_{i,c_i}\notin X$, so
$X\notin\mathcal S_{I,c}$. Hence $\mathcal S_{I,c}\ne\mathcal S_{I',c'}$. $\square$

*Machine check.* `[Thm 4.4]`: the $\prod(1+p_i)$ intersections were enumerated and found
pairwise distinct — $4/4$ for $g=1$, $24/24$ for $g=2$, $192/192$ for $g=3$.

**Corollary 4.5 (superpolynomial).** Let $p_i$ be the $i$-th odd prime, $C_g:=
T_{p_1,\dots,p_g}$, $m_g$ the size of its minimal DFAO. Then $m_g\le1+g+\sum_{i\le g}p_i$
and
$$\log\Lambda(C_g)\ \ge\ \sum_{i\le g}\log(1+p_i)\ \ge\ \theta(p_g)\ =\ (1+o(1))\,\sqrt{m_g\log m_g},$$
i.e. $\Lambda(C_g)\ge\exp\bigl((1+o(1))\sqrt{m_g\log m_g}\bigr)$ (logs natural): for every
polynomial $P$ there is a binary $2$-automatic sequence with $\Lambda(T)>P(m)$.

*Proof.* $\sum_{i\le g}p_i=(1+o(1))\tfrac12g^2\ln g$ and $p_g=(1+o(1))g\ln g$ and
$\theta(p_g)=(1+o(1))p_g$ (prime number theorem). From $m=(1+o(1))\tfrac12g^2\ln g$ we get
$\ln m=(2+o(1))\ln g$, hence $g=(1+o(1))\,2\sqrt{m/\ln m}$ and
$p_g=(1+o(1))g\ln g=(1+o(1))\sqrt{m\ln m}$. $\square$

The convergence is slow, as it must be for a $\exp\sqrt{\cdot}$ bound; the *proved* lower
bound overtakes $m^3$ only at $g=6$:

| $g$ | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| $\vert Q\vert=1{+}g{+}\sum p_i$ | 5 | 11 | 19 | 31 | 45 | 63 | 83 | 107 |
| $\prod(1+p_i)$ (proved $\le\Lambda$) | 4 | 24 | 192 | 2304 | 32256 | 580608 | $1.2\cdot10^7$ | $2.8\cdot10^8$ |
| $m^3$ | 125 | 1331 | 6859 | 29791 | 91125 | 250047 | 571787 | $1.2\cdot10^6$ |
| $\Lambda$ **measured** | 53 | 816 | 15583 | — | — | — | — | — |

The measured $\Lambda$ is far above the proved bound and its growth factor is *rising*
($\times15.4$, $\times19.1$), so Theorem 4.4 is not tight; $\exp(c\sqrt{m\log m})$ is what
this construction *proves*, not necessarily what the family does.

**Remark 4.6 (nothing is smuggled in).** $k=2$ is fixed, the output alphabet is
$\{0,1\}$ (no faithful/lossy coding trick — see `proof-family.md`), the automaton is
minimal for every case computed, and by Lemma 3.1 minimality is not needed anyway. The
mechanism is not "large $k$", not "large $\Delta$", and not unbounded critical exponent
per se: it is that pairwise coprime periods cost a *sum* of states and resolve a
*product* of shifts.

**Remark 4.7 (the one-modulus control).** The same checker computes $\Lambda$ for the
single-gadget automaton $\mathbb Z_P$ ($m=P$ states, $\delta(x,b)=2x+b$, $\tau=[x{=}0]$),
whose alignment relations see only *one* modulus: $\Lambda=6805$ at $P=15$ and
$58396$ at $P=21$, i.e. $\log\Lambda/\log m=3.26$ then $3.61$. The exponent is rising
there too, so $\Lambda$ may well be superpolynomial for reasons beyond the CRT
mechanism — but for $\mathbb Z_P$ the growth comes from the short-comparison regime
($\varepsilon<P$ or $k^s-\varepsilon<P$, where Lemma 4.3(b)'s forcing fails) and I have
no proof. Theorem 4.4 is the statement that does not depend on unexplained data.

**Remark 4.8 (how big can $\Lambda$ get?).** Pairwise coprime moduli $q_1<\dots<q_g$ with
$\sum q_i\le m$ have $\prod q_i\le e^{(1+o(1))\sqrt{m\log m}}$, so this mechanism cannot
be pushed past $\exp(\sqrt{m\log m})$. Whether $\Lambda(T)$ can be $2^{\Omega(m)}$ — the
trivial ceiling is $2^{\min(\gamma,m^3)}$ — is open. Note the shift-match automaton
$\mathcal S(T)$ may have far more states than $M$ ($\gamma=817$ at $m=19$ above), so
a small DFAO could in principle resolve shifts modulo something exponential in $m$.

---

## 5. Consequences for `proof-upper.md`

1. **Theorem 5.4 is permanently conditional.** Corollary 5.5 there ("if
   $\Lambda=O(m^c)$ then $|\mathrm{FE}|=O(m^{8+2c})$") is true but has no unconditional
   instance: there is no polynomial bound on $\Lambda$ to plug in. The honest remark in
   §8.1 of that document ("$\Lambda=\mathrm{poly}(m)$ is not proved and I see no route to
   it") stands, but the status of the statement "$\Lambda=\mathrm{poly}(m)$" is upgraded
   from *not proved* to **false**.
2. **$\Lambda$ is not a lower-bound proxy either.** For the family, $|\mathrm{FE}|$ stays
   tiny while $\Lambda$ explodes:

   | family | $m$ | $\gamma$ | $\Lambda$ | $\Lambda^\ast$ (sampled, §6) | $\vert\mathrm{FE}_{\mathrm{msd}}\vert$ | bound $m^4{+}m^6{+}m^8\Lambda^2$ |
   |---|---|---|---|---|---|---|
   | $T_{3}$ | 5 | 27 | 53 | 38 | **168** | $1.1\cdot10^9$ |
   | $T_{3,5}$ | 11 | 130 | 816 | 259 | **1479** | $1.4\cdot10^{14}$ |
   | $T_{3,5,7}$ | 19 | 817 | 15583 | 297 | **2828** | $4.1\cdot10^{18}$ |

   ($|\mathrm{FE}|$ from the engine, msd, dead state included. For $m=5$ and $m=11$ the
   `learnfe` counts $168$ and $1479$ are reproduced exactly by a direct
   `let FE(i,j,l) A t. (t<l) => T[i+t]=T[j+t]` build — subset construction, no learner;
   at $m=19$ the direct build was killed by the memory watchdog after 109 s, so $2828$
   rests on `learnfe` alone, which reported `capped_lcp=103`.) $|\mathrm{FE}|$ tracks $m^3$
   here while $\Lambda$ passes $m^3$ already at $m=19$. This is the sharpest instance of
   the one-way-ness recorded in `proof-verdict.md` §1 ("$\Lambda$ is not monotone in
   $|\mathrm{FE}|$ across families").
3. **What is untouched.** Lemma 3.1, Corollary 3.2, Proposition 4.3, Theorem 4.4,
   Lemma 5.1 and Theorem 6.1 of `proof-upper.md` are unaffected; so is Corollary 5.6 ($\Lambda$ as a cheap, engine-free *heuristic*),
   and so is §7.3's observation that $\Lambda$ separates the faithful from the lossy
   coding of the same DFAO. What dies is the use of $\Lambda$ as a *bound*.

---

## 6. The repair: count the pairs that actually occur

Theorem 5.4's proof passes through the intersection closure only to have a place to put
$A^{\pm}_{I,J,L}$. The closure is enormously bigger than the set of pairs that occur.

**Definition 6.1.** $\displaystyle \Lambda^\ast(T):=\Bigl|\bigl\{\,
(A^{+}_{I,J,L},\,A^{-}_{I,J,L})\ :\ I,J\ge0,\ L\ge2\,\bigr\}\Bigr|$, with $A^{\pm}$ as in
§1. Trivially $\Lambda^\ast\le\Lambda^2$.

**Theorem 6.2.** $\ |\mathrm{FE}_{\mathrm{msd}}(T)|\ \le\ m^4+m^6+m^8\,\Lambda^\ast(T).$

*Proof.* Verbatim the proof of Theorem 5.4 of `proof-upper.md`: prefixes with $L\le1$
contribute $\le m^4+m^6$ (Corollary 3.2 there); for $L\ge2$ the residual is a function of
$8$ DFAO states and $\Theta_{I,J,L}$ (Theorem 4.4 there), and $\Theta_{I,J,L}$ is a
function of the pair $(A^{+}_{I,J,L},A^{-}_{I,J,L})$ (Definition 4.2 there, using that
$(s,\varepsilon)\mapsto\mathrm{Sh}_{s,\varepsilon}$ depends on $T$ only). The number of
such pairs is $\Lambda^\ast$ by definition, instead of being bounded by $\Lambda\cdot\Lambda$. $\square$

This is strictly stronger (one factor of $\Lambda$ saved) and, unlike Theorem 5.4, it is
not refuted by §4:

| $T$ | $m$ | $\Lambda$ | $\Lambda^2$ | $\Lambda^\ast$ (sampled) |
|---|---|---|---|---|
| Thue–Morse | 2 | 4 | 16 | 6 |
| `01 20 22 / 101` | 3 | 17 | 289 | 50 |
| `01 23 00 32 / 0011` | 4 | 28 | 784 | 85 |
| `012 201 120 / 011` (k=3) | 3 | 13 | 169 | 26 |
| `01 23 34 10 42 / 10010` | 5 | 40 | 1600 | 124 |
| $T_3$ | 5 | 53 | 2809 | 38 |
| $T_{3,5}$ | 11 | 816 | 665856 | 259 |
| $T_{3,5,7}$ | 19 | 15583 | $2.4\cdot10^8$ | 297 |

($\Lambda^\ast$ over $I,J<60$, $2\le L<60$ for the stock examples — 205 320 prefixes. For
the family: $I,J<40$, $2\le L<40$ (60 800 prefixes) plus up to 7 695 large prefixes at
scale $2^{16}$, chosen to sit inside, and to straddle, every gadget region, with
$|I-J|$ ranging over three full periods $\prod p_i$. These are samples, not exhaustive
counts, so they are upper-bounded observations of a quantity defined over all
$(I,J,L)$.)

**Why the counterexample misses $\Lambda^\ast$.** $\mathcal T^{\pm}_{I,J,L}$ is not an
arbitrary subset of $Q^3$: it is the set of triples visited by a *walk of consecutive
integers*, i.e. by the pair odometer $c\mapsto(q_{I+c},q_{J+c})$. In $C_{p_1,\dots,p_g}$
the gadgets occupy disjoint intervals of $n$ (the routing is by the top bits), so a
window either
* stays inside one gadget's interval, in which case every triple it collects is
  $\bigl((i,\alpha{+}c),(i,\beta{+}c),(i,\beta{+}c{+}1)\bigr)$ and all of them impose the
  *same* single congruence $\varepsilon\equiv(\alpha-\beta)2^s\ (p_i)$, so only
  $\sum_ip_i=O(m)$ congruence classes are available to such windows, never the CRT
  product $\prod_ip_i$ (short-scale effects can add more classes, but they are not
  multiplicative across gadgets); or
* straddles a boundary, in which case it also collects "mixed" triples whose components
  live in different gadgets, and the intersection collapses (measured: over a window of
  length $12\,000$ at scale $2^{14}$ spanning all gadget regions, the number of distinct
  $(A^{+},A^{-})$ over $|I-J|<2\prod p_i$ is **3**, not $\prod p_i$).

The CRT product is available to the closure but not to any single walk. So the family
separates $\Lambda$ from $\Lambda^\ast$ — which is the useful content of §4 for the
program: **$\Lambda^\ast$, not $\Lambda$, is the invariant Open Problem 1(A) reduces to.**

**Restatement of the open problem.** Is $\Lambda^\ast(T)=\mathrm{poly}(m)$ for all
$k$-automatic $T$? By Theorem 6.2 a yes gives $|\mathrm{FE}|=\mathrm{poly}(m)$
unconditionally. Note $\Lambda^\ast$ is not obviously easier: $\Theta_{I,J,L}\ne\emptyset$
means $T$ has a repetition of length $\asymp L k^s$ at distance $\asymp|I-J|k^s$, so
$\Lambda^\ast$ counts *repetition profiles*, which is close to what $|\mathrm{FE}|$
counts. Theorem 6.1 of `proof-upper.md` (bounded critical exponent) is exactly a
statement that most profiles are empty. The right object to study is the set of subsets
of $Q\times Q$ realisable as vertex sets of walks in the pair odometer.

---

## 7. Gaps

1. **The real question is untouched.** $\Lambda^\ast=\mathrm{poly}(m)$ is neither proved
   nor refuted here, and by §6 it is the only version of the invariant that could still
   settle Open Problem 1(A). This document closes a route, it does not open one.
2. **$\Lambda^\ast$ on the CRT family is measured, not proved.** The claim "$O(m)$
   congruences per single-gadget window" in §6 is a proof sketch for the in-region case
   only; the straddling case is empirical ($3$ classes observed). A complete proof that
   $\Lambda^\ast(C_g)=\mathrm{poly}(m)$ is not given.
3. **Strength of the blow-up.** Theorem 4.4 gives $\exp(\Theta(\sqrt{m\log m}))$, not
   $2^{\Omega(m)}$; see Remark 4.8. Also the measured $\Lambda$ grows visibly faster than
   the proved bound, so the true growth rate for this family is unknown.
4. **$|\mathrm{FE}|$ for the family** is measured at $g\le3$ only ($m\le19$); the $g=4$
   ($m=31$) build did not finish inside the 900 s engine budget, and at $m=19$ only
   `learnfe` (which capped 103 LCP walks) completed — the independent direct construction
   was killed by the memory watchdog. Three points are not a growth law:
   "$|\mathrm{FE}|\approx m^3$ here" is an observation, not a claim.
5. **msd only**, as in `proof-upper.md`. Nothing here says anything about lsd.
6. **Finite-range machine checks.** Lemmas 2.2, 2.4, 4.2, 4.3 have proofs; the checks are
   brute force (scales $s\le9$, $g\le3$, five stock DFAOs) and can only refute. Theorem 4.4's
   conclusion is verified exactly for $g\le3$.

---

## 8. Files

* `paper/proof3-lambda.md` — this document.
* `paper/proof3-lambda-check.py` — all machine checks: Lemma 2.2, Lemma 2.4, Lemma 3.1,
  Lemma 4.2, Lemma 4.3, and Theorem 4.4 head-on, plus $\gamma$/$\Lambda$ for the family
  and for the one-modulus control $\mathbb Z_P$. `python3 paper/proof3-lambda-check.py`
  (~5 min; `--big` adds $g=4$ and $P\le45$ and did not finish in an hour).
* `paper/proof-upper.md` — Definitions 4.1, 4.2, 5.2, 5.3 and Theorems 4.4, 5.4 referred
  to throughout.
* `paper/proof-verdict.md` — the referee report whose "weakest step" for `proof-upper.md`
  (Definition 5.3 + Theorem 5.4 as a bound) this document resolves.
