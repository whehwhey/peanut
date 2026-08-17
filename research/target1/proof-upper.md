# Upper bounds for the equality-of-factors automaton

**Status: PARTIAL.** I do not prove a polynomial-in-$m$ upper bound in general, and I do
not believe the argument below can be pushed to one without new input. What is proved:

1. an exact **locality** theorem — the Myhill–Nerode residual of a prefix $(I,J,L)$ is a
   function of the $2(L+2)$ DFAO states $q_{I},\dots,q_{I+L+1},q_J,\dots,q_{J+L+1}$
   (Lemma 3.1);
2. an **unconditional polynomial bound** $m^4+m^6$ on the number of states reachable by
   prefixes with $L\le 1$ (Corollary 3.2);
3. a **head–middle–tail factorisation** (Theorem 4.3): for $L\ge2$ the residual is
   determined by exactly **8** DFAO states plus one further datum $\Theta$, the *middle
   language*, and $\Theta$ depends on $(I,J,L)$ only through two subsets of $Q^3$;
4. a finite, morphism-only invariant $\Lambda(T)$ that bounds the number of middle
   languages, giving the **main bound**
   $$|\mathrm{FE}_{\mathrm{msd}}(T)|\;\le\;m^4+m^6+m^8\,\Lambda(T)^2 ,$$
   so that $|\mathrm{FE}|$ is polynomial in $m$ **whenever $\Lambda$ is** (Theorem 5.4);
   $\Lambda$ is computable from the morphism alone in time polynomial in $m^3$ and
   $\Lambda$, without ever constructing $\mathrm{FE}$;
5. two **unconditional collapses under bounded critical exponent** (Theorem 6.1).

What is *not* proved: that $\Lambda(T)=\mathrm{poly}(m)$. Empirically $\Lambda$ is small
(§7): median $\Lambda$ over random minimal DFAOs is $4,11,20,29,60,52,88$ for
$k=2$, $m=2..8$, and $\Lambda$ is *constant* $=4$ for the whole generalised-Thue–Morse
family with faithful output but grows like $p^{2.7}$ for the same family under a lossy
binary coding — which is exactly the coding effect flagged as the current lead in
`docs/TARGET1.md`. So Open Problem 1(A) is reduced, in the upper-bound direction, to a
finite question about a small explicit automaton attached to the morphism.

Everything below is machine-checked by `paper/proof-upper-check.py` (brute force over an
explicit prefix of $T$; no engine, no automata library). Every check reports 0 violations.

---

## 1. Setting

Fix $k\ge2$. Let $T=(T[n])_{n\ge0}$ be $k$-automatic over a finite output alphabet
$\Delta$, and let
$$M=(Q,\Sigma_k,\delta,q_0,\tau),\qquad \Sigma_k=\{0,\dots,k-1\},\ \ \tau:Q\to\Delta,$$
be its **minimal msd DFAO**: $T[n]=\tau(\delta(q_0,\mathrm{rep}_k(n)))$, where
$\mathrm{rep}_k(n)$ is the base-$k$ representation of $n$, most significant digit first.
Put $m=|Q|$. Minimality forces every state reachable and no two states equivalent; the
msd convention forces
$$\delta(q_0,0)=q_0, \tag{1.1}$$
so leading zeros are harmless and $\delta(q_0,w)$ is well defined for any $w\in\Sigma_k^*$
denoting $n$. Write
$$q_x:=\delta(q_0,\mathrm{rep}_k(x))\qquad(x\ge0),\qquad q_0\text{ for }x=0 .$$
For $s\ge0$ and $0\le y<k^s$ let $\mathrm{rep}_s(y)\in\Sigma_k^s$ be the $s$-digit
representation of $y$ (with leading zeros).

**Definition 1.1 (blocks).** For $q\in Q$ and $s\ge0$ let
$$B_s(q)\;:=\;\bigl(\tau(\delta(q,\mathrm{rep}_s(y)))\bigr)_{0\le y<k^s}\ \in\ \Delta^{k^s}.$$

**Lemma 1.2 (block factorisation).** For all $x\ge0$, $s\ge0$, $0\le y<k^s$,
$$T[xk^s+y]\;=\;B_s(q_x)[y].$$
Consequently $T = B_s(q_0)\,B_s(q_1)\,B_s(q_2)\cdots$ for every $s$.

*Proof.* $\mathrm{rep}_k(xk^s+y)=\mathrm{rep}_k(x)\,\mathrm{rep}_s(y)$ up to leading
zeros, which are absorbed by (1.1); apply $\delta$ and $\tau$. $\square$

**Lemma 1.3 (block refinement).** $B_s(q)=B_{s-1}(\delta(q,0))\,B_{s-1}(\delta(q,1))\cdots
B_{s-1}(\delta(q,k-1))$ for $s\ge1$.

*Proof.* Write $y=rk^{s-1}+y'$ with $r\in\Sigma_k$, $0\le y'<k^{s-1}$; then
$\mathrm{rep}_s(y)=r\,\mathrm{rep}_{s-1}(y')$, so
$B_s(q)[y]=\tau(\delta(\delta(q,r),\mathrm{rep}_{s-1}(y')))=B_{s-1}(\delta(q,r))[y']$. $\square$

**The predicate.** $\ \mathrm{FE}(i,j,l)\ :\Longleftrightarrow\ \forall t<l:\ T[i+t]=T[j+t]$,
i.e. $T[i..i+l)=T[j..j+l)$. Its msd automaton reads the three tracks $i,j,l$ in parallel
over the alphabet $\Sigma_k^3$, all three padded with leading zeros to a common length.
Write $|\mathrm{FE}_{\mathrm{msd}}(T)|$ for the number of states of the minimal complete
msd DFA. (This is the count reported by `engine`, which includes the dead state; it is
Walnut's count $+1$ — see `bench/README.md`.)

**Definition 1.4 (mismatch distance).**
$D(i,j):=\min\{t\ge0: T[i+t]\ne T[j+t]\}\in\mathbb N\cup\{\infty\}$. Then
$$\mathrm{FE}(i,j,l)\iff l\le D(i,j). \tag{1.2}$$

---

## 2. Residuals

A word $w\in(\Sigma_k^3)^r$ read by the automaton is exactly a triple of $r$-digit
strings, i.e. a triple $(I,J,L)\in[0,k^r)^3$. Two words with the same triple of values
(differing only by leading zeros) have the same residual, and every $(I,J,L)\in\mathbb N^3$
occurs. Hence

$$|\mathrm{FE}_{\mathrm{msd}}(T)| \;=\; \#\bigl\{\,R_{I,J,L}\ :\ (I,J,L)\in\mathbb N^3\,\bigr\}, \tag{2.1}$$

where the **residual** of $(I,J,L)$ is the language over $\Sigma_k^3$
$$R_{I,J,L}\;:=\;\bigl\{(i',j',l')_s\ :\ s\ge0,\ 0\le i',j',l'<k^s,\
\mathrm{FE}(Ik^s+i',\,Jk^s+j',\,Lk^s+l')\bigr\},$$
a triple of $s$-digit strings being appended to the prefix.

**Lemma 2.1 (clamped form).** Let $\mathrm{cl}_s(x)=\min(\max(x,0),k^s)$. Then
$R_{I,J,L}=R_{I_2,J_2,L_2}$ iff for every $s\ge0$ and every $i',j'\in[0,k^s)$
$$\mathrm{cl}_s\!\bigl(D(Ik^s+i',Jk^s+j')-Lk^s\bigr)
=\mathrm{cl}_s\!\bigl(D(I_2k^s+i',J_2k^s+j')-L_2k^s\bigr).$$

*Proof.* By (1.2) the set of accepted $l'$ for fixed $(s,i',j')$ is
$\{l'<k^s: Lk^s+l'\le D\}$, an initial segment of $[0,k^s)$ of length exactly
$\mathrm{cl}_s(D-Lk^s)$. $\square$

So the whole problem is: how much information about $D$ near the diagonal scale $Lk^s$
must be remembered.

---

## 3. Locality: only $2(L+2)$ DFAO states matter

**Lemma 3.1 (window locality).** Fix $(I,J,L)$, $s\ge0$ and $i',j',l'\in[0,k^s)$, and put
$i=Ik^s+i'$, $j=Jk^s+j'$, $l=Lk^s+l'$. Then
$$[i,i+l)\subseteq[Ik^s,(I+L+2)k^s),\qquad [j,j+l)\subseteq[Jk^s,(J+L+2)k^s).$$
Consequently $\mathrm{FE}(i,j,l)$ is a function of
$$\bigl(u_0,\dots,u_{L+1};\,v_0,\dots,v_{L+1}\bigr)\quad\text{and}\quad (s,i',j',l'),$$
where $u_c:=q_{I+c}$ and $v_c:=q_{J+c}$, and therefore $R_{I,J,L}$ is a function of
$(u_0,\dots,u_{L+1},v_0,\dots,v_{L+1})$ alone (with $L$ fixed).

*Proof.* $i+l=(I+L)k^s+i'+l'<(I+L+2)k^s$; same for $j$. By Lemma 1.2 the content of
$T$ on $[(I+c)k^s,(I+c+1)k^s)$ is $B_s(u_c)$, and $B_s$ is a function of the state. All
positions compared lie in blocks $I..I+L+1$ resp. $J..J+L+1$. $\square$

**Corollary 3.2 (unconditional polynomial bound for $L\le1$).**
$$\#\{R_{I,J,0}\}\le m^4,\qquad \#\{R_{I,J,1}\}\le m^6 .$$
Explicitly, $R_{I,J,0}$ is a function of $(u_0,u_1,v_0,v_1)$ and $R_{I,J,1}$ is a
function of $(u_0,u_1,u_2,v_0,v_1,v_2)$.

*Proof.* Lemma 3.1 with $L=0$ ($L+2=2$ blocks per side) and $L=1$ ($3$ blocks). $\square$

*Machine check.* `[Cor 3.2]` lines of `proof-upper-check.py`: 0 violations on all five
worked examples, over all prefixes $I,J,L<10$ and all suffixes of length $\le4$
($\le3$ for $k=3$).

Lemma 3.1 is sharp in the sense that the number of DFAO states it needs grows with $L$;
the content of §4 is that only 8 of them are needed once $L\ge2$, the rest entering
through a single aggregated predicate.

---

## 4. Head–middle–tail factorisation

Throughout this section $L\ge2$. Fix $s$, $i',j',l'\in[0,k^s)$ and set
$$e:=j'-i'\in(-k^s,k^s),\qquad i=Ik^s+i',\ j=Jk^s+j',\ l=Lk^s+l'.$$
Partition $[0,l)$ at the block boundaries of the **$i$-side**:

| region | $x$-range | $i$-side positions | $i$-blocks |
|---|---|---|---|
| head   | $[0,\;k^s-i')$              | $[i,\,(I+1)k^s)$            | $I$ |
| middle | $[k^s-i',\;Lk^s-i')$        | $[(I+1)k^s,\,(I+L)k^s)$     | $I+1,\dots,I+L-1$ (full) |
| tail   | $[Lk^s-i',\;l)$             | $[(I+L)k^s,\,i+l)$          | $I+L,\ I+L+1$ |

(The middle is non-empty and has length exactly $(L-1)k^s\ge k^s$ because $L\ge2$.)
Since $j+x=i+x+e$ with $|e|<k^s$, and $j+l<(J+L+2)k^s$, the corresponding **$j$-side**
positions lie in blocks

* head: $J,\,J+1$;
* middle block $c$ $(1\le c\le L-1)$: $J+c,\,J+c+1$ if $e\ge0$; $J+c-1,\,J+c$ if $e<0$;
* tail: $J+L-1,\,J+L,\,J+L+1$.

**Definition 4.1 (shift-match relation).** For $s\ge0$ and $0\le\varepsilon<k^s$ put
$$\mathrm{Sh}_{s,\varepsilon}\;:=\;\bigl\{(u,a,b)\in Q^3\ :\
B_s(u)=\bigl(B_s(a)B_s(b)\bigr)[\varepsilon\,:\,\varepsilon+k^s]\bigr\}\ \subseteq\ Q^3 .$$
(For $\varepsilon=0$ this is $\{(u,a,b):B_s(u)=B_s(a)\}$, i.e. $u\sim_s a$ where $\sim_s$
is "same length-$k^s$ block".)

**Definition 4.2 (middle triple sets and middle language).** For $L\ge2$,
$$\mathcal T^{+}_{I,J,L}:=\{(u_c,v_c,v_{c+1}):1\le c\le L-1\},\qquad
  \mathcal T^{-}_{I,J,L}:=\{(u_c,v_{c-1},v_c):1\le c\le L-1\}\ \subseteq Q^3,$$
$$\Theta_{I,J,L}:=\Bigl\{(s,e): s\ge0,\ |e|<k^s,\
\begin{cases}\mathcal T^{+}\subseteq \mathrm{Sh}_{s,e} & e\ge0\\[2pt]
\mathcal T^{-}\subseteq \mathrm{Sh}_{s,k^s+e} & e<0\end{cases}\Bigr\}.$$

**Proposition 4.3 (the middle condition).** For $L\ge2$, all $s$ and all $|e|<k^s$,
$$T\bigl[(I+1)k^s+x\bigr]=T\bigl[(J+1)k^s+x+e\bigr]\ \ \text{for all }0\le x<(L-1)k^s
\qquad\Longleftrightarrow\qquad (s,e)\in\Theta_{I,J,L}.$$

*Proof.* The left side says: for every $c\in[1,L-1]$, the block $B_s(u_c)$ (the content of
$i$-blocks $I+c$) equals the length-$k^s$ window of $T$ starting at $(J+c)k^s+e$. If
$e\ge0$ that window lies inside blocks $J+c,J+c+1$ at offset $e$, i.e. it is
$(B_s(v_c)B_s(v_{c+1}))[e:e+k^s]$; so the condition for this $c$ is
$(u_c,v_c,v_{c+1})\in\mathrm{Sh}_{s,e}$. If $e<0$ the window starts at
$(J+c-1)k^s+(k^s+e)$, giving $(u_c,v_{c-1},v_c)\in\mathrm{Sh}_{s,k^s+e}$. Conjoin over
$c$. $\square$

*Machine check.* `[Prop 4.3]` (printed as `Prop 4.2` in the script): 0 mismatches out of
45 600 (resp. 60 800) $(I,J,L,s,e)$ instances per example.

**Theorem 4.4 (endpoint + middle).** For $L\ge2$ the residual $R_{I,J,L}$ is a function of
$$\underbrace{\bigl(u_0,\;u_L,\;u_{L+1},\;v_0,\;v_1,\;v_{L-1},\;v_L,\;v_{L+1}\bigr)}_{8\text{ DFAO states}}
\quad\text{and}\quad \Theta_{I,J,L}.$$

*Proof.* Fix $(s,i',j',l')$. By the head/middle/tail table, $\mathrm{FE}(i,j,l)$ is the
conjunction of three conditions.

*Head.* Compares $B_s(u_0)[i'..k^s)$ with the length $k^s-i'$ window of $T$ starting at
$Jk^s+j'$, which lies in blocks $J,J+1$; so the head condition is a function of
$(u_0,v_0,v_1)$ and $(s,i',j')$.

*Middle.* By Proposition 4.3 it is $[(s,e)\in\Theta_{I,J,L}]$, a function of $\Theta$ and
$(s,e)=(s,j'-i')$.

*Tail.* Compares $T$ on $[(I+L)k^s,(I+L)k^s+i'+l')$, which lies in blocks $I+L,I+L+1$
(content $B_s(u_L)B_s(u_{L+1})$), against $T$ on
$[(J+L)k^s+e,(J+L)k^s+e+i'+l')$, which lies in blocks $J+L-1,J+L,J+L+1$ (content
$B_s(v_{L-1})B_s(v_L)B_s(v_{L+1})$); so it is a function of
$(u_L,u_{L+1},v_{L-1},v_L,v_{L+1})$ and $(s,i',l',e)$.

All three conditions are determined by the listed 8 states, $\Theta$, and the suffix
$(i',j',l')$ itself, which is the argument of the residual. $\square$

*Machine check.* `[Thm 4.3]` line of the script (the 8-tuple above): 0 violations on all
five examples. Dropping $v_{L-1}$ from the tuple produces genuine violations, so 8 is not
obviously reducible; the middle can be dropped only in favour of $\Theta$, not of a bit.

---

## 5. The shift-match automaton, and the main bound

The point of Definition 4.2 is that $\Theta$ depends on $(I,J,L)$ only through the two
subsets $\mathcal T^\pm\subseteq Q^3$, and on $T$ only through the family
$\{\mathrm{Sh}_{s,\varepsilon}\}$. That family is itself the state set of a small
automaton.

**Lemma 5.1 (descent).** $\mathrm{Sh}_{0,0}=\{(u,a,b):\tau(u)=\tau(a)\}$, and for $s\ge1$,
writing $\varepsilon=d\,k^{s-1}+\varepsilon'$ with $d\in\Sigma_k$, $0\le\varepsilon'<k^{s-1}$,
$$\mathrm{Sh}_{s,\varepsilon}\;=\;\Phi_d\bigl(\mathrm{Sh}_{s-1,\varepsilon'}\bigr),$$
where, for $X\subseteq Q^3$ and $d\in\Sigma_k$,
$$\Phi_d(X):=\Bigl\{(u,a,b)\ :\ \forall r\in\Sigma_k,\
\bigl(\delta(u,r),\,Z_{d+r},\,Z_{d+r+1}\bigr)\in X\Bigr\},\qquad
Z_i:=\begin{cases}\delta(a,i)&0\le i<k\\ \delta(b,i-k)&k\le i<2k.\end{cases}$$

*Proof.* Put $K=k^{s-1}$. By Lemma 1.3, $B_s(a)B_s(b)=Z_0Z_1\cdots Z_{2k-1}$ read as a
concatenation of $2k$ words $B_{s-1}(Z_i)$ of length $K$ each, and
$B_s(u)=B_{s-1}(\delta(u,0))\cdots B_{s-1}(\delta(u,k-1))$. The window
$[\varepsilon,\varepsilon+kK)$ of the former splits into $k$ chunks of length $K$, the
$r$-th starting at $dK+\varepsilon'+rK$, i.e. at offset $\varepsilon'$ inside the chunk of
index $d+r$; that chunk equals
$(B_{s-1}(Z_{d+r})B_{s-1}(Z_{d+r+1}))[\varepsilon':\varepsilon'+K]$ (note
$d+r+1\le 2k-1$). Equating chunk $r$ with $B_{s-1}(\delta(u,r))$ gives exactly
$(\delta(u,r),Z_{d+r},Z_{d+r+1})\in\mathrm{Sh}_{s-1,\varepsilon'}$. $\square$

*Machine check.* `[Lem 5.1]`: 0 violations, all $(s,\varepsilon)$ with $s\le4$, five
examples.

**Definition 5.2.** The **shift-match automaton** of $T$ is
$$\mathcal S(T):=\{\Phi_{d_1}\Phi_{d_2}\cdots\Phi_{d_s}(\mathrm{Sh}_{0,0})\ :\ s\ge0,\ d_i\in\Sigma_k\}
\ \subseteq\ 2^{Q^3},$$
the orbit of $\mathrm{Sh}_{0,0}$ under the $k$ monotone operators $\Phi_d$; put
$\gamma(T):=|\mathcal S(T)|\le 2^{m^3}$. By Lemma 5.1,
$\mathrm{Sh}_{s,\varepsilon}\in\mathcal S(T)$ for every $s,\varepsilon$, and the map
$(s,\varepsilon)\mapsto\mathrm{Sh}_{s,\varepsilon}$ is computed by reading
$\mathrm{rep}_s(\varepsilon)$.

**Definition 5.3.** For $t\in Q^3$ let $P_t:=\{X\in\mathcal S(T):t\in X\}$ (a "principal
up-set"). Let
$$\Lambda(T):=\bigl|\ \{\textstyle\bigcap_{t\in\mathcal T}P_t\ :\ \mathcal T\subseteq Q^3\}\ \bigr|$$
be the size of the intersection-closure of the $P_t$ inside $2^{\mathcal S(T)}$
(including the empty intersection $\mathcal S(T)$). Trivially
$\Lambda\le\min(2^{\gamma},2^{m^3})$.

**Theorem 5.4 (main bound).**
$$\boxed{\ |\mathrm{FE}_{\mathrm{msd}}(T)|\ \le\ m^4+m^6+m^8\,\Lambda(T)^2\ }$$

*Proof.* By (2.1) it suffices to count residuals. Prefixes with $L=0$ contribute at most
$m^4$ and with $L=1$ at most $m^6$ (Corollary 3.2). For $L\ge2$, Theorem 4.4 says the
residual is determined by 8 DFAO states ($\le m^8$ choices) together with
$\Theta_{I,J,L}$. By Definition 4.2, $\Theta$ is determined by the pair of sets
$$A^{+}:=\{X\in\mathcal S(T):\mathcal T^{+}\subseteq X\}=\bigcap_{t\in\mathcal T^{+}}P_t,
\qquad A^{-}:=\bigcap_{t\in\mathcal T^{-}}P_t,$$
because $\mathrm{Sh}_{s,\varepsilon}\in\mathcal S(T)$ and the containment
$\mathcal T^{\pm}\subseteq\mathrm{Sh}_{s,\varepsilon}$ is precisely
$\mathrm{Sh}_{s,\varepsilon}\in A^{\pm}$, while $(s,\varepsilon)\mapsto
\mathrm{Sh}_{s,\varepsilon}$ depends on $T$ only. Both $A^{\pm}$ lie in the
intersection-closure, so there are at most $\Lambda^2$ possible $\Theta$. $\square$

**Corollary 5.5.** If $\Lambda(T)=O(m^{c})$ then $|\mathrm{FE}_{\mathrm{msd}}(T)|=O(m^{8+2c})$.
In particular a class of $k$-automatic sequences with exponential $|\mathrm{FE}|$ must have
superpolynomial $\Lambda$.

**Corollary 5.6 (cheap predictor; relevant to Open Problem 1(B)).** $\gamma$ and $\Lambda$
are computable directly from the morphism: $\Phi_d$ costs $O(km^3)$, the orbit costs
$O(k\gamma m^3)$, and the closure costs $O(\Lambda m^3\gamma)$ set operations. No
automaton for $\mathrm{FE}$ is built. This gives an a-priori size estimate before paying
for the construction.

**Honest remark on the general case.** Since $\Lambda\le2^{m^3}$, Theorem 5.4 yields only
$2^{O(m^3)}$ unconditionally, which is *worse* than the $2^{9m^2}$ bound quoted by
Khodier. Theorem 5.4 is not an improvement on the general exponential bound; its value is
that it isolates the entire obstruction in one small, computable object.

---

## 6. Collapses under bounded critical exponent

Let $E(T)=\sup\{\alpha: T\text{ has a factor of exponent }\alpha\}$ (exponent of a word of
length $n$ with period $p$ is $n/p$). Suppose $E(T)\le\rho<\infty$.

**Theorem 6.1.** Assume $E(T)\le\rho$.

**(i)** For every $I$ and every $L\ge\lceil\rho\rceil$,
$$R_{I,I,L}=\{(i',j',l')_s:\ i'=j'\}.$$
In particular *all* prefixes with $I=J$ and $L\ge\lceil\rho\rceil$ collapse to a **single**
state of the FE automaton.

**(ii)** If $I\ne J$ and $L>(\rho-1)\bigl(|I-J|+1\bigr)$ then $R_{I,J,L}=\emptyset$: the
prefix is dead.

*Proof.* Let $\Delta=|i-j|$ with $i=Ik^s+i'$, $j=Jk^s+j'$, $l=Lk^s+l'$. If
$\mathrm{FE}(i,j,l)$ holds with $l\ge1$ and $\Delta>0$, then $T$ restricted to
$[\min(i,j),\max(i,j)+l)$ has period $\Delta$ and length $l+\Delta$, hence exponent
$(l+\Delta)/\Delta\le\rho$, i.e.
$$l\le(\rho-1)\Delta. \tag{6.1}$$
(i) Here $I=J$, so $\Delta=|i'-j'|<k^s$, while $l\ge Lk^s\ge\rho k^s>(\rho-1)\Delta$
unless $\Delta=0$; so (6.1) forces $\Delta=0$, i.e. $i'=j'$, and conversely $i'=j'$ gives
$i=j$ and $\mathrm{FE}$ trivially. (For $s=0$, $i'=j'=0$ anyway.)
(ii) $\Delta=|(I-J)k^s+(i'-j')|\le(|I-J|+1)k^s$ and $l\ge Lk^s$, so (6.1) gives
$L\le(\rho-1)(|I-J|+1)$; if that fails, no suffix is accepted. $\square$

*Machine check.* Thue–Morse ($E=2$): $R_{I,I,L}=\{i'=j'\}$ for all $L\ge2$, 0 violations
out of 374 480 instances. The same test on sequences of unbounded critical exponent
(e.g. the $m=3$, $m=4$ examples, which contain arbitrarily long constant runs) fails, as
it must.

**Reading.** Theorem 6.1 says the *entire* diagonal region of the FE automaton collapses
to one state as soon as the sequence is power-bounded, and a large region dies. This is a
proof-side explanation of `docs/TARGET1.md` Finding 2, where max run length was the second
strongest predictor ($\rho=+0.25$) of residual $\log|\mathrm{FE}|$ within a size class:
long runs are exactly unbounded critical exponent, which is exactly what disables Theorem
6.1. It also says a candidate exponential family for Open Problem 1(A) must have unbounded
critical exponent — which the current best candidates (thin sets "exactly $c$ ones", long
constant runs) do.

Theorem 6.1 does **not** give a polynomial bound: prefixes with $|I-J|$ and
$L\asymp\rho|I-J|$ both large survive it, and for those the middle language is still
needed.

---

## 7. What $\Lambda$ actually does

All numbers from `paper/proof-upper-check.py` (and `--sweep`).

### 7.1 Worked examples (msd, $|\mathrm{FE}|$ from `engine`, includes dead state)

| $T$ | $k$ | $m$ | $\gamma$ | $\Lambda$ | $\#\Theta$ observed | $\lvert\mathrm{FE}\rvert$ | bound $m^4{+}m^6{+}m^8\Lambda^2$ |
|---|---|---|---|---|---|---|---|
| Thue–Morse | 2 | 2 | 3 | 4 | 4 | 15 | $4.2\cdot10^3$ |
| `01 20 22 / 101` | 2 | 3 | 14 | 17 | 34 | 123 | $1.9\cdot10^6$ |
| `01 23 00 32 / 0011` | 2 | 4 | 15 | 28 | 31 | 315 | $5.1\cdot10^7$ |
| `012 201 120 / 011` | 3 | 3 | 7 | 13 | 15 | 101 | $1.1\cdot10^6$ |
| `01 23 34 10 42 / 10010` | 2 | 5 | 15 | 40 | 39 | 671 | $6.3\cdot10^8$ |

The bound is very lossy (the $m^8$ endpoint factor is nowhere near tight) but polynomial
in $m$ for fixed $\Lambda$-growth.

### 7.2 $\Lambda$ on random minimal DFAOs (25 per cell)

```
  k  m   gamma med/max   Lambda med/max
  2  2       3     3          4      4
  2  3       6    11         11     23
  2  4      13    20         27     88
  2  5      13    26         34    210
  2  6      13    31         46    251
  2  7      14    28         55    262
  2  8      17    29         88    403
  3  2       3     4          4      6
  3  3       7    10         10     16
  3  4       9    16         15     40
  3  5       8    16         17     38
  3  6      10    17         22     71
  3  7      10    19         23     68
  3  8      14    31         34    111
```
Median $\Lambda$ grows roughly like $m^{1.7\text{--}2}$; max like $m^{3}$. Over the 385
morphisms of `results/blowup.json`, Spearman correlation of $\Lambda$ against the measured
$|\mathrm{FE}|$ *within* each $(k,m)$ cell is positive in all 12 cells
($+0.60,+0.72,+0.01,+0.32,+0.62,+0.47$ for $k=2$, $m=2..7$;
$+0.35,+0.21,+0.61,+0.42,+0.48,+0.75$ for $k=3$) — median $\approx+0.5$.

### 7.3 $\Lambda$ explains the coding effect

`docs/TARGET1.md` reports that collapsing $s_2(n)\bmod p$ from $p$ output letters to the
binary $[s_2\not\equiv0]$ keeps the DFAO at $p$ states but multiplies $|\mathrm{FE}|$ by
$\sim100$, and calls this "the current lead". $\Lambda$ reproduces that split exactly:

| family (DFAO: $\delta(q,0)=q$, $\delta(q,1)=q+1 \bmod p$) | $p$: | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|
| faithful output $\tau=\mathrm{id}$ | $\gamma$ | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| | $\Lambda$ | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| singleton coding $\tau=[q{=}0]$ | $\gamma$ | 3 | 7 | 13 | 23 | 37 | 55 | 77 | 103 |
| | $\Lambda$ | 4 | 11 | 28 | 58 | 106 | 181 | 300 | 496 |

$\Lambda$ is **constant** for the faithful family (whose $|\mathrm{FE}|$ is measured
polynomial — exactly quadratic in lsd) and grows like $p^{2.7}$ for the lossily coded
family (whose $|\mathrm{FE}|\approx3p^4$). Likewise for "exactly $c$ ones in base 2"
($m=c+2$): $\Lambda=12,30,66,129,235,417$ for $c=1..6$, against measured
$|\mathrm{FE}|=52,183,-,1042,2008,3463$. In all three families the growth of $\Lambda$
tracks the growth of $|\mathrm{FE}|$, and in all three both are polynomial.

So: no known family has superpolynomial $\Lambda$, and by Theorem 5.4 no family with
polynomial $\Lambda$ can have exponential $|\mathrm{FE}|$.

---

## 8. Gaps — what is not proved

1. **The main gap.** $\Lambda(T)=\mathrm{poly}(m)$ is *not* proved and I see no route to
   it. $\Lambda$ is the size of the intersection-closure of $\le m^3$ subsets of an
   $\gamma$-element set, so a priori $\Lambda\le 2^{\min(\gamma,m^3)}$; nothing in the
   argument prevents $\gamma$ itself from being exponential, since $\mathcal S(T)$ is the
   reachable set of a subset-valued dynamical system and the operators $\Phi_d$ are not
   monotone along the orbit (already $\mathrm{Sh}_{0,0}$ and $\Phi_0(\mathrm{Sh}_{0,0})$
   are $\subseteq$-incomparable in general: $\mathrm{Sh}_{0,0}$ is "$u\sim_0a$" and
   $\Phi_0(\mathrm{Sh}_{0,0})$ is "$u\sim_1a$", and $\sim_s$ is not monotone in $s$).
   Consequently Theorem 5.4 does **not** improve the unconditional $2^{9m^2}$ bound.
2. **Direction of the reduction.** Theorem 5.4 is one-way. Large $\Lambda$ does *not*
   imply large $|\mathrm{FE}|$ — the $m^8$ endpoint factor and the $\Lambda^2$ factor are
   both gross over-counts (see §7.1: the bound overshoots by 2–6 orders of magnitude).
   So $\Lambda$ cannot be used directly to *construct* an exponential family; it can only
   rule regions out.
3. **The $m^8$ factor is not tight.** Machine evidence: the 8-tuple is necessary in the
   sense that dropping $v_{L-1}$ breaks it, but the number of *reachable*
   (8-tuple, $\Theta$) pairs is far below $m^8\Lambda^2$ (e.g. 267 vs $4\cdot10^3$ for
   Thue–Morse). A tighter count would have to exploit which 8-tuples co-occur with which
   $\Theta$, i.e. the joint reachability of the odometer trajectories
   $c\mapsto(q_{I+c},q_{J+c})$ — not attempted here.
4. **$L\le1$ constants.** $m^4$ and $m^6$ are the crude counts from Lemma 3.1; the
   measured class counts (e.g. 16 and 36 for Thue–Morse, i.e. exactly $m^4$ and $m^6$)
   suggest they are close to tight as *parametrisations* but the reachable sets are
   smaller.
5. **Critical exponent.** Theorem 6.1 requires $E(T)<\infty$, which excludes exactly the
   families that look most dangerous (thin sets, long runs). For those the theorem is
   vacuous and the middle language carries everything.
6. **Verification scope.** The machine checks are brute force over prefixes
   $I,J,L<10$ and suffix lengths $s\le4$ ($s\le3$ for $k=3$) on five DFAOs with
   $m\le5$. They can only refute, not prove; all the statements above have proofs, and the
   checks are there because the head/middle/tail bookkeeping is easy to get wrong (an
   earlier 8-tuple omitting $v_{L-1}$ *was* wrong and the check caught it).
7. **lsd.** Everything here is msd. The lsd automaton has a different (sometimes much
   larger, sometimes much smaller) size and is not covered.

## 9. Files

* `paper/proof-upper.md` — this document.
* `paper/proof-upper-check.py` — all machine checks; `python3 paper/proof-upper-check.py`
  (add `--sweep` for §7.2, `--families` for §7.3). Self-contained, no engine, ~2 minutes.
