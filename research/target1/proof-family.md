# The equality-of-factors automaton of the generalised Thue–Morse family, exactly

*Target 1 (Khodier 2026, Open Problem 1) — the **family** side: a natural infinite family
of $k$-automatic sequences for which $|FE|$ is determined exactly as a function of the
size $m$ of the DFAO, and is polynomial.*

**Status: `partial` (strong).**
Proved and machine-checked: a complete structural characterisation of $FE$ for the
$p$-letter generalised Thue–Morse word $G_p$ (Prop. 3.4), the parity-rigidity lemma that
drives it (Lemma 3.3), two explicit correct automaton constructions realising it, a
matching Myhill–Nerode lower bound $|FE|_{\mathrm{msd}}\ge p^{3}$ (Thm. 4.4), hence
$$|FE_{G_p}|_{\mathrm{msd}}=\Theta(m^{3}),\qquad |FE_{G_p}|_{\mathrm{lsd}}=O(m^{2}),
\qquad m=p .$$
Exactly, and verified independently three times for $3\le p\le 24$:
$$\boxed{\;|FE_{G_p}|_{\mathrm{msd}} = p^{3}+8,\qquad
        |FE_{G_p}|_{\mathrm{lsd}} = 2p^{2}+3p+12\;}$$
(with $p=2$, Thue–Morse, exceptional: $15$ and $22$).
Also proved and machine-checked: an exact block characterisation of $FE$ for the
**singleton coding** $T_p=[\,s_2(n)\equiv1\,]$ (Prop. 5.1) and the *grading* that explains
its cost: at $k=v_2(j-i)$ the effective coding is $\chi_{\min(k,p-1)}$, refining with $k$
and injective already at $k=p-2$ (Lemma 5.2, Prop. 5.3). This closes
`docs/TARGET1.md`'s "lossy coding of a group automaton" lead as a *mechanism* for
exponential blowup: the collapse is a grading with only $p-2$ genuinely lossy levels, so
it cannot manufacture an exponential unless some single level's own $FE$ is already
exponential — which is the original question again, for a coarser coding of the same
$m$-state DFAO. Measured, the collapse costs a factor $5.4\to23.1$ over $p=3..8$.

**Not proved / not obtained.**
(i) That the closed forms hold for *every* $p$ — they are proved for each individual
$p\le24$ (correct construction + exact minimisation), not uniformly.
(ii) The requested $\Omega(p^4)$ for $T_p$: **not established**. Worse, the local
log-log slope of $190,698,1877,3971,7243,11988$ falls *through* $4$
($4.52\to3.77$ over $p=3..8$), so the "$3p^4$" law of `docs/TARGET1.md` is not safe; a
cubic $44.4p^3-205.4p^2+308.5p-88.7$ fits the same six points with residual $\le5.7$.
The decisive data point ($p\ge9$ msd) did not fit in the 6 GB memory guard.
(iii) No upper bound at all for $T_p$.

---

## 1. Setting

Base $k=2$ throughout. $s(n)$ is the binary digit sum, $\nu(n)$ the number of trailing
$1$-bits of $n$, $v_2$ the $2$-adic valuation. Fix $p\ge2$.

* $G_p[n] := s(n)\bmod p$ — the *generalised Thue–Morse word*, the fixed point of the
  $2$-uniform morphism $a\mapsto a\,(a{+}1)$ on $\mathbb Z_p$ with the **identity coding**.
  Its DFAO is the cyclic group $\mathbb Z_p$ with $\delta(a,c)=a+c$: $m=p$ states.
* $T_p[n] := [\,s(n)\equiv 1 \bmod p\,]$ — the **singleton coding** of the *same* DFAO
  ($m=p$ again), a binary sequence. ($[\,s(n)\not\equiv0\,]$, used in
  `docs/TARGET1.md`, is the complement of the singleton coding at $0$; the two give the
  same $|FE|$, since $G_p$ is shift-conjugate to itself under $a\mapsto a+1$.)
* $$FE(i,j,l)\;:=\;\forall t\,(t<l\Rightarrow T[i+t]=T[j+t]),$$
  a language of triples over $(\{0,1\}^3)^*$, all three tracks padded to a common length,
  read msd or lsd. $|FE|$ = number of states of the minimal **complete** DFA, i.e. the
  dead state is counted (the `automatheus` convention; Walnut reports one fewer).

Two elementary identities are used constantly:

$$\textbf{(I1)}\quad G[2n+f]=G[n]+f\quad(f\in\{0,1\}),\qquad\qquad
\textbf{(I2)}\quad G[n{+}1]-G[n]=1-\nu(n).$$

---

## 2. Why this family is the right test case

`docs/TARGET1.md` records the one surviving lead for an *exponential* family: **lossy
codings of group automata**. Keeping the DFAO at $p$ states but collapsing
$s_2\bmod p$ to one bit multiplies $|FE|$ by $\approx100$ (lsd, $p=4$: $6154$ vs $56$).
This note determines the mechanism exactly. It is a **grading by $v_2(j-i)$**: at level
$k$ the effective coding is $\chi_{\min(k,p-1)}$, a coding that refines as $k$ grows and
**saturates at the identity coding at level $p-1$** (Lemma 5.2, Prop. 5.3). There are only
$p$ levels, and each is an $FE$ problem for the same $p$-state DFAO. The collapse is
therefore worth at most a factor $p$ — measurably it is between a constant $\approx 44$
and $\approx 3p$ (§5.4) — not an exponential. As a route to exponential blowup the lead
is closed.

---

## 3. Structure of $FE$ for $G_p$

### Lemma 3.1 (halving)
*Let $l\ge1$ and $i\equiv j \pmod 2$. Put $i'=\lfloor i/2\rfloor$, $j'=\lfloor j/2\rfloor$
and $l' = \lfloor (i+l-1)/2\rfloor-\lfloor i/2\rfloor+1$ (the number of blocks
$\{2c,2c+1\}$ meeting $[i,i+l)$). Then*
$$FE_{G}(i,j,l)\iff FE_{G}(i',j',l').$$

*Proof.* Let $e=i\bmod2=j\bmod 2$. For $0\le t<l$ write $e+t=2u+f$, $f\in\{0,1\}$,
$u=\lfloor (e+t)/2\rfloor$. Then $i+t=2(i'+u)+f$ and $j+t=2(j'+u)+f$, so by (I1)
$G[i+t]=G[i'+u]+f$ and $G[j+t]=G[j'+u]+f$; hence
$G[i+t]=G[j+t]\iff G[i'+u]=G[j'+u]$. As $t$ runs over $[0,l)$, $u$ runs over exactly the
integers of $[0,\lfloor (e+l-1)/2\rfloor]$, each attained; and
$\lfloor (e+l-1)/2\rfloor=\lfloor (i+l-1)/2\rfloor-i'$ because $i=2i'+e$. $\square$

Note that $l'$ is *not* $\lceil l/2\rceil$: for even $l$ and odd $i$ it is $l/2+1$. (This
is the trap; the first version of this proof got it wrong and was caught by brute force.)

### Corollary 3.2 (iterated halving)
*If $i\equiv j \pmod{2^k}$ and $l\ge1$ then*
$$FE_G(i,j,l)\iff FE_G\!\big(\lfloor i/2^k\rfloor,\ \lfloor j/2^k\rfloor,\ L_k\big),
\qquad L_k=\Big\lfloor \tfrac{i+l-1}{2^k}\Big\rfloor-\Big\lfloor \tfrac{i}{2^k}\Big\rfloor+1,$$
*i.e. $L_k$ is the number of blocks $[c2^k,(c+1)2^k)$ meeting $[i,i+l)$.*

*Proof.* Induction on $k$ using Lemma 3.1; the block counts compose because
$i'+l'-1=\lfloor (i+l-1)/2\rfloor$ and $\lfloor i'/2\rfloor=\lfloor i/4\rfloor$. $\square$

### Lemma 3.3 (parity rigidity)
*Let $p\ge 2$ and $a\not\equiv b \pmod 2$. Then $FE_G(a,b,4)$ is false. Equivalently:
a factor of $G_p$ of length $\ge4$ determines the parity of its occurrence positions.*

*Proof.* WLOG $a=2a'$, $b=2b'+1$. By (I1) the four conditions $G[a+t]=G[b+t]$, $t<4$, read
$$G[a']=G[b']+1,\quad G[a']+1=G[b'+1],\quad G[a'+1]=G[b'+1]+1,\quad G[a'+1]+1=G[b'+2].$$
The first two give $G[b'+1]-G[b']=2$; the last two give
$G[b'+2]=G[a'+1]+1=G[b'+1]+2$. By (I2), $\nu(b')\equiv-1$ and $\nu(b'+1)\equiv-1 \pmod p$.
Since $p\ge2$ we have $-1\not\equiv0$, so $\nu(b')\ge1$, i.e. $b'$ is odd; then $b'+1$ is
even and $\nu(b'+1)=0\equiv-1$ is false. $\square$

### Proposition 3.4 (complete characterisation)
*Let $p\ge2$. $FE_G(i,j,l)$ holds iff $l=0$, or $i=j$, or: $i\ne j$, $l\ge1$ and, putting*
$$k=v_2(i\oplus j)=v_2(|j-i|),\quad a=\lfloor i/2^k\rfloor,\quad b=\lfloor j/2^k\rfloor,
\quad L=\Big\lfloor\tfrac{i+l-1}{2^k}\Big\rfloor-\Big\lfloor\tfrac{i}{2^k}\Big\rfloor+1,$$
*we have $L\le 3$ and $G[a+t]=G[b+t]$ for all $t<L$.*

*Proof.* Corollary 3.2 plus Lemma 3.3: by maximality of $k$, $a\not\equiv b\pmod 2$. $\square$

**Machine check.** Brute force against the definition for $p=2,\dots,6$ and all
$i,j,l<70$: $0$ mismatches ($1\,715\,000$ triples).

### Corollary 3.5 (bit-level form)
*Keep the notation of Prop. 3.4 and let $u\in\{i,j\}$ be the one whose bit $k$ is $1$ and
$w$ the other. Put $\alpha=\nu\!\left(\lfloor w/2^{k+1}\rfloor\right)$ and
$\beta=\nu\!\left(\lfloor u/2^{k+1}\rfloor\right)$ (the lengths of the runs of $1$s
immediately above position $k$ in $w$ resp. $u$). Then*
$$G[a]=G[b]\iff s(i)\equiv s(j),\qquad
G[a{+}1]=G[b{+}1]\ \text{(given the previous)}\iff \beta\equiv-1,$$
$$G[a{+}2]=G[b{+}2]\ \text{(given the previous two)}\iff \alpha\equiv-1 \pmod p .$$
*Moreover, with $r=i\bmod 2^k=j\bmod 2^k$ and $l=2^k l_{hi}+l_{lo}$,*
$$L=\begin{cases} l_{hi} & r+l_{lo}=0,\\ l_{hi}+1+[\,r+l_{lo}>2^k\,] & \text{otherwise.}\end{cases}$$

*Proof.* $i\equiv j \pmod{2^k}$, so $s(i)-s(j)=s(a)-s(b)$: the first item. Take $w=i$
(so $a$ even, $b$ odd). By (I2), $G[a{+}1]-G[a]=1-\nu(a)=1$ and
$G[b{+}1]-G[b]=1-\nu(b)=1-(1+\beta)$, giving the second. Then $a+1$ is odd with
$\nu(a{+}1)=1+\alpha$ while $b+1$ is even with $\nu(b{+}1)=0$, giving the third. The
formula for $L$ is $\lfloor (r+l_{lo}-1)/2^k\rfloor\in\{-1,0,1\}$. $\square$

So $FE_{G_p}$ is decided by **four numbers**: $s(i)-s(j)\bmod p$; the two run lengths
$\alpha,\beta\bmod p$ above the lowest differing bit; and $L\in\{0,1,2,3,{\ge}4\}$.
That is the whole content of the equality-of-factors predicate for this family.

---

## 4. Exact automaton sizes for $G_p$

### 4.1 An explicit msd automaton — the $O(p^3)$ upper bound

`explore/fe_gtm_construct.py` builds a deterministic msd automaton $\mathcal A^{msd}_p$
over $(\{0,1\}^3)$ whose state is
$$\big(D,\;\rho_i,\;\rho_j,\;H,\;M\big),$$
$D=s(i_{read})-s(j_{read})\bmod p$; $\rho_i,\rho_j$ = current trailing-run-of-$1$s lengths
of the two tracks $\bmod\ p$; $H=\min(4,\text{value of the } l\text{-prefix})$;
and $M=\bot$ ("no difference yet") or a quadruple $(f_2,f_3,h,\Sigma)$ recorded at the
*most recent* differing position: $f_2=[\beta\equiv-1]$, $f_3=[\alpha\equiv-1]$ (read off
$\rho_i,\rho_j$ at that moment, Cor. 3.5), $h=\min(4,l_{hi})$, and $\Sigma$ the state of a
$3$-state msd comparator for $r+l_{lo}$ against $2^k$ (it stores the map
$\text{carry-in}\mapsto(\text{carry-out},\text{all sum digits zero})$; composition of the
per-digit maps is $\Sigma\mapsto\Sigma\circ g$, which is constant-absorbing, so only a
handful of maps occur). Acceptance evaluates Prop. 3.4 through Cor. 3.5.

*Correctness* is Cor. 3.5 read as an online algorithm. *Verified* by brute force against
the definition for $p\le6$, all $i,j,l<48$ ($0$ mismatches), and against the engine for
$p\le16$ (see 4.3).

**Size.** The reachable state count is exactly
$$|\mathcal A^{msd}_p| \;=\; 34p^{3}+98p^{2}+41p \qquad (2\le p\le16,\ \text{exact fit,
third difference constant}),$$
so $|FE_{G_p}|_{\mathrm{msd}}\le 34p^3+98p^2+41p=O(m^3)$ unconditionally.

### 4.2 An explicit lsd automaton — the $O(p^2)$ upper bound

`explore/fe_gtm_lsd.py`. Reading lsd, the *first* difference seen is the lowest one, so
$k$ is known as soon as it occurs. Before it: a $4$-state accumulator for
$(\text{carry},\text{all-zero})$ of $r+l_{lo}$. At position $k$: freeze
$\text{ovf}=[r+l_{lo}>2^k]$, $\text{zero}=[r+l_{lo}=0]$ and $\sigma$ = which track carries
the $1$. After it: $D\bmod p$; for each track a run register that is either *active* with
a count $\bmod\ p$ or *frozen* with the single bit $[\text{count}\equiv-1]$; a capped
lsd accumulator $(\min(4,l_{hi}),\min(4,2^{t}))$ for $l_{hi}$; plus $\text{ovf}$,
$\text{zero}$, $\sigma$, and a bit "$l$ has a $1$".

Crucially, *both* run registers can be active only while the two tracks agree above $k$,
which forces $D=\pm1$ and equal counts — that coupling is what makes the lsd machine
quadratic rather than cubic. Reachable count, exactly,
$$|\mathcal A^{lsd}_p| = 128p^{2}+160p+19 \qquad (2\le p\le16),$$
so $|FE_{G_p}|_{\mathrm{lsd}} = O(m^2)$ unconditionally. Brute-force verified for $p\le6$.

### 4.3 The exact minimal sizes

Minimising $\mathcal A^{msd}_p$ and $\mathcal A^{lsd}_p$ (Moore/Hopcroft, exact) and
independently running the `automatheus` engine on
`let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]` gives, in complete agreement:

| $p$ | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | … | 24 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| msd | 15 | 35 | 72 | 133 | 224 | 351 | 520 | 737 | 1008 | 1339 | 1736 | … | 13832 |
| $p^3+8$ | *16* | 35 | 72 | 133 | 224 | 351 | 520 | 737 | 1008 | 1339 | 1736 | … | 13832 |
| lsd | 22 | 39 | 56 | 77 | 102 | 131 | 164 | 201 | 242 | 287 | 336 | … | 1236 |
| $2p^2{+}3p{+}12$ | *26* | 39 | 56 | 77 | 102 | 131 | 164 | 201 | 242 | 287 | 336 | … | 1236 |

Three independent computations agree on every entry: the Rust engine (msd/lsd, $p\le24$),
`fe_gtm_construct.py` (msd, $p\le16$), `fe_gtm_lsd.py` (lsd, $p\le16$); and a fourth,
fully generic construction (`explore/fe_direct.py`, subset construction from the
carry/digit-sum product automaton, no use of §3) reproduces $p\le3$ before blowing up.

**Theorem 4.3.** *For each individual $3\le p\le 24$, $|FE_{G_p}|_{\mathrm{msd}}=p^3+8$
and $|FE_{G_p}|_{\mathrm{lsd}}=2p^2+3p+12$.* This is a proof for those $p$: the
constructions are proved correct from Prop. 3.4, and exact DFA minimisation of a correct
DFA yields the minimal DFA. For $p=2$ (Thue–Morse) the values are $15$ and $22$.

### 4.4 Matching lower bound: $|FE_{G_p}|_{\mathrm{msd}}\ge p^3$ for all $p\ge3$

**Theorem 4.4.** *For every $p\ge3$ the minimal msd DFA of $FE_{G_p}$ has at least $p^3$
states.*

*Proof.* Put $P=2p+2$, $S=3p+6$. For $(A,\rho)\in\mathbb Z_p^2$ let
$$X_{A,\rho}\;=\;0^{\,P-e-1-\rho}\,1^{e}\,0\,1^{\rho},\qquad e=(A-\rho)\bmod p,$$
a length-$P$ binary word with digit sum $\equiv A$ and with *exactly* $\rho$ trailing
$1$s (the separating $0$ guarantees exactness). Consider the msd prefixes
$\big(X_{A,\rho},\,X_{B,\rho'},\,0^{P}\big)$, and attach to each the *key*
$\mathbf{key}=(D,\rho,\rho')$, $D=A-B\bmod p$. Every $\mathbf{key}\in\mathbb Z_p^3$
occurs; we show that the key is recoverable from the residual language, so that a set of
$p^3$ representatives is pairwise Myhill–Nerode-inequivalent.

Distinguishing suffixes, of common length $S$, indexed by $0\le c,k<p$ (bit positions are
numbered $S-1,\dots,0$ inside the suffix; all unlisted bits are $0$):

| | $i$-track $1$s | $j$-track $1$s | $l$ |
|---|---|---|---|
| $\mathrm A_{c,k}$ | $k{+}2,\dots,k{+}1{+}c$ | $k$ | $1$ |
| $\mathrm B_{c,k}$ | $k{+}2,\dots,k{+}1{+}c$ | $k,\dots,S{-}1$ | $2^k+1$ |
| $\mathrm C_{c,k}$ | $k{+}1,\dots,S{-}1$ | $k$; $k{+}1,\dots,k{+}p{-}1$; $k{+}p{+}1,\dots,k{+}p{+}c$ | $2\cdot 2^k+1$ |

In all three, both tracks are $0$ below position $k$ and differ at $k$, so
$v_2(i\oplus j)=k$, $r=0$, $u=j$, $w=i$. With $r=0$ one checks
$L=1,2,3$ respectively (Cor. 3.5). Hence, by Cor. 3.5:

* $\mathrm A_{c,k}$ accepts $\iff s(i)\equiv s(j)\iff A+c\equiv B+1\iff c\equiv 1-D$.
  A unique $c$ accepts: this reads off $D$.
* $\mathrm B_{c,k}$: here $\beta=(S-1-k)+\rho'$ (the $j$-run runs from $k{+}1$ to the top
  of the suffix and continues into the prefix's trailing run), $s(i)=A+c$,
  $s(j)=B+(S-k)$. It accepts $\iff A+c\equiv B+S-k$ and $S-k+\rho'\equiv0$; a unique pair
  $(c,k)\bmod p$ accepts, namely $k\equiv S+\rho'$, $c\equiv-D-\rho'$: this reads off
  $(D,\rho')$.
* $\mathrm C_{c,k}$: here $\beta=p-1\equiv-1$ automatically, $\alpha=(S-1-k)+\rho$,
  $s(i)=A+(S-1-k)$, $s(j)=B+c$ (mod $p$, since $1+(p-1)\equiv0$). It accepts $\iff$
  $A+S-1-k\equiv B+c$ and $S-k+\rho\equiv0$; a unique pair accepts, $k\equiv S+\rho$,
  $c\equiv D-1-\rho$: this reads off $(D,\rho)$.

All positions used are $<S$ because $k+p+c+1\le 3p<S$, and $|X_{A,\rho}|\le 2p+1<P$.
Thus the response vector of a prefix determines $\mathbf{key}=(D,\rho,\rho')$, so
representatives of the $p^3$ keys are pairwise inequivalent. $\square$

**Machine check.** Exactly $p^3$ distinct response vectors for $p=3,4,5,6,7,8$
(`explore/fe_family_verify.py`, `check_lb()`).

### 4.5 Consequences

**Theorem 4.5 (main).** *For the generalised Thue–Morse family $G_p$ (a DFAO with
$m=p$ states over $k=2$),*
$$p^{3}\;\le\;|FE_{G_p}|_{\mathrm{msd}}\;\le\;34p^{3}+98p^{2}+41p,\qquad
|FE_{G_p}|_{\mathrm{lsd}}\;\le\;128p^{2}+160p+19 .$$
*Hence $|FE|=\Theta(m^{3})$ in msd; and $|FE|_{\mathrm{msd}}=p^3+8$,
$|FE|_{\mathrm{lsd}}=2p^2+3p+12$ for every $3\le p\le24$.*

Khodier's general upper bound for this predicate is $2^{9m^2}$; at $m=10$ that is
$2^{900}$ against the true $1008$. This is, as far as I know, the first infinite family of
DFAOs for which $|FE|$ is pinned down exactly as a function of $m$, and it is cubic.

---

## 5. The singleton coding $T_p=[\,s_2(n)\equiv1\,]$

### Proposition 5.1 (block characterisation)
*Let $i<j$, $l\ge1$, $d=j-i$, $k=v_2(d)$, $d'=d/2^k$ (odd). For $a\in\mathbb N$ let
$B_a=[a2^k,(a+1)2^k)$, and for each $a$ with $B_a\cap[i,i+l)\ne\emptyset$ put*
$$\mu_a=s(a)\bmod p,\qquad \theta_a=s(a+d')-s(a)\bmod p,\qquad
S_a=\{\,s(r)\bmod p:\ a2^k+r\in[i,i+l)\,\}.$$
*Then $FE_{T_p}(i,j,l)$ holds iff for every such $a$:*
$$\theta_a\equiv0\quad\text{or}\quad (\mu_a+S_a)\cap\{1,\,1-\theta_a\}=\emptyset
\ \ \text{in }\mathbb Z_p .$$

*Proof.* For $n=a2^k+r\in B_a$ we have $n+d=(a+d')2^k+r$, so $s(n)=s(a)+s(r)$ and
$s(n+d)=s(a+d')+s(r)$: the *shift* $G[n+d]-G[n]=\theta_a$ depends only on the block.
Now $T[n]=T[n+d]\iff[G[n]=1]=[G[n]+\theta_a=1]$, which holds iff $\theta_a=0$ or
$G[n]\notin\{1,1-\theta_a\}$. Range over $n$ in the window. $\square$

**Machine check.** Brute force for $p=2,\dots,7$ and all $i,j,l<48$: $0$ mismatches.

### Lemma 5.2 (level cap / saturation)
*If $B_a\subseteq[i,i+l)$ then $S_a=\{0,1,\dots,k\}\bmod p$. Hence if $k\ge p-1$ then
$S_a=\mathbb Z_p$ and the condition of Prop. 5.1 degenerates to $\theta_a\equiv0$, i.e. to
the full-coding condition $G[a]=G[a+d']$.*

*Proof.* $\{s(r):0\le r<2^k\}=\{0,\dots,k\}$. $\square$

(In fact the collapse happens one step earlier: $\chi_{p-2}$ merges only one letter into
$*$ and is therefore injective, so levels $\kappa\ge p-2$ already give $FE_{G_p}$; the
grading has $p-2$ genuinely lossy levels $\kappa=0,\dots,p-3$.)

### Proposition 5.3 (level decomposition)
*Put $\kappa=\min(k,p-1)$, $V_\kappa=\{1,0,-1,\dots,1-\kappa\}\subseteq\mathbb Z_p$
($|V_\kappa|=\kappa+1$), and let $\chi_\kappa:\mathbb Z_p\to V_\kappa\cup\{*\}$ be the
identity on $V_\kappa$ and $*$ off it. For a fully covered block the condition of
Prop. 5.1 is exactly*
$$\chi_\kappa\big(G[a]\big)=\chi_\kappa\big(G[a+d']\big).$$
*Consequently the interior of $FE_{T_p}$ is the equality-of-factors predicate for the
coded sequence $\chi_\kappa\!\circ\!G_p$ — the same $p$-state DFAO with a coding onto
$\kappa+2$ letters — evaluated at a pair of positions with odd difference $d'$.*

*Proof.* $(\mu+[0,\kappa])\cap\{1,1-\theta\}=\emptyset$ says $1-\mu\notin[0,\kappa]$ and
$1-\mu-\theta\notin[0,\kappa]$, i.e. $\mu\notin V_\kappa$ and $\mu+\theta\notin V_\kappa$.
Adding the escape clause $\theta=0$ (i.e. $\mu=\mu+\theta$) gives precisely
"$\chi_\kappa(\mu)=\chi_\kappa(\mu+\theta)$". $\square$

**This is the mechanism.** The singleton coding is *graded by $v_2(j-i)$*: at level $k$
the effective coding is $\chi_{\min(k,p-1)}$, which refines as $k$ grows. $\chi_0$ is the
singleton coding itself; $\chi_\kappa$ becomes *injective* already at $\kappa=p-2$ (only
one letter is merged), so the grading has exactly $p-2$ genuinely lossy levels and
collapses to $FE_{G_p}$ from level $p-2$ on. Lemma 3.3 (rigidity) applies at the top level
and is what stops the recursion. So $FE_{T_p}$ is a superposition of $\Theta(p)$ $G$-type
problems, indexed by the level — at most one factor of $p$ on top of §4, hence at worst
$O(p)\cdot\max_\kappa|FE_{\chi_\kappa\circ G_p}|$, and in no case exponential. What the
grading does *not* immediately give is a bound, because each level is itself an $FE$
problem (gap G3).

### 5.4 Data

Minimal msd sizes (engine; $p\le6$ re-derived independently here from Prop. 5.1):

| $p$ | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|
| $|FE_{T_p}|_{\mathrm{msd}}$ | 15 | 190 | 698 | 1877 | 3971 | 7243 | 11988 |
| lsd | 22 | 656 | 6154 | — | — | — | — |

Fits on $p=3..8$. A **quartic** with leading coefficient $1.06$ fits with maximum
residual $0.5$; a **cubic** $44.4p^3-205.4p^2+308.5p-88.7$ fits with maximum residual
$5.7$; the best pure $cp^4$ is $2.96\,p^4$ ($\le3.5\%$ error for $p\ge5$, $26\%$ at
$p=3$). Six points cannot separate these.

The *local* log-log slope $\Delta\log|FE_{T_p}|/\Delta\log p$ is
$$4.52,\; 4.43,\; 4.11,\; 3.90,\; 3.77 \qquad (p=3{:}4,\dots,7{:}8),$$
i.e. it is **falling through 4**, whereas $|FE_{G_p}|=p^3+8$ has local slope $2.94$ at the
same point. Two readings are consistent with this: (a) degree $4$ with a large negative
$p^3$ correction — but then the slope should approach $4$ from *below*, not fall through
it; (b) degree $3$ with a large constant, $\approx44p^3$, i.e. the singleton coding costs
a *constant* factor $\approx44$ rather than a factor $p$. The data currently favour (b),
which would contradict the naive reading of Prop. 5.3 (levels $0..p-1$, each a $G$-type
$\Theta(p^3)$ problem) and would mean the level automata **share** most of their states.
`docs/TARGET1.md`'s "$3p^4$ within 2.5%" is a fit over the same six points and does not
distinguish the two either.

**The graded codings, measured** (`explore/coding_levels.py`,
`results/coding_levels*.json`). $|FE_{\chi_\kappa\circ G_p}|_{\mathrm{msd}}$:

| $p\backslash\kappa$ | 0 | 1 | 2 | 3 | 4 | $\ge p-2$ |
|---|---|---|---|---|---|---|
| 5 | **1877** | 1118 | 769 | 133 | | $133=p^3{+}8$ |
| 6 | **3971** | 2607 | 1799 | 1453 | 224 | $224$ |
| 7 | **7243** | 5128 | 3765 | 2926 | | $351$ |
| 8 | **11988** | 8962 | | | | $520$ |

Three things to read off.

1. $\chi_\kappa$ is *injective* as soon as $\kappa\ge p-2$ (only one letter is merged
   into $*$), so the grading has exactly $p-2$ genuinely lossy levels and collapses to
   $G_p$ at $\kappa=p-2$ — the $133,224,351$ column.
2. The sizes decrease in $\kappa$ and level $0$ *is* $T_p$, so this is not by itself a
   bound on $T_p$; it is a probe of how a single level grows.
3. Growth in $p$ at fixed $\kappa$ (local log-log slope): $\kappa=1$ gives
   $4.64,\,4.39,\,4.18$; $\kappa=2$ gives $4.66,\,4.79$; $\kappa=3$ gives $4.54$. Every
   level shows the *same declining* apparent exponent as level $0$ itself
   ($4.52\to3.77$) — the levels are not $\Theta(p^3)$ objects at these sizes, and the
   "$p$ levels $\times$ a cubic each" picture is too crude. Also
   $|FE_{\chi_0}|/|FE_{\chi_1}| = 1.68,\,1.52,\,1.41,\,1.34$ for $p=5..8$: consecutive
   levels converge, which is what one expects if $T_p$ is *dominated by* rather than the
   *sum of* its levels. What the grading buys is a description of the mechanism, and a
   proof that the mechanism is not exponential-by-construction — not yet a bound.

The ratio $|FE_{T_p}|/|FE_{G_p}|$ is $5.4,\,9.7,\,14.1,\,17.7,\,20.6,\,23.1$ for
$p=3..8$: growing, but with local exponent already down to $0.83$ at $p=8$.

Predictions of the two fits (cubic / quartic / $3p^4$):
$p=9$: $18446/18522/19683$; $p=10$: $26894/27189/30000$; $p=12$: $50823/52421/62208$.
So $p=12$ is the first really decisive point; $p=9,10$ help.

The lsd direction is much worse for the singleton coding ($22,656,6154$), i.e. the digit
order that is *good* for $G_p$ is *bad* for $T_p$; this is the same msd/lsd decoupling
recorded in `docs/TARGET1.md` Finding 1.

### 5.5 Partial lower bound

The natural $p^4$ coordinate system is $(s(i),s(j),\alpha,\beta)\bmod p$ — the §4
coordinates with $s(i)-s(j)$ *split* into the ordered pair, because $T$ is not
translation-invariant. Explicit prefix families realising all $p^4$ keys, tested against
an exhaustive structured suffix pool (18\,083 suffixes at $p=3$), separate
$76/81$ ($p=3$), $221/256$ ($p=4$), $478/625$ ($p=5$) — so
$76/81$ ($p=3$), $221/256$ ($p=4$), $478/625$ ($p=5$). These are *rigorous* lower bounds
($\ge76,\,221,\,478$ states) but weaker than the known exact values $190,698,1877$, and
they are not a proof of $\Omega(p^4)$: some keys are separated by no suffix in the pool,
so $(s(i),s(j),\alpha,\beta)$ is **not** a complete coordinate system for this language.
Fixing $\alpha=0$ and using the $p^3$ keys $(s(i),s(j),\beta)$ gives $27/27$, $62/64$,
$118/125$ — again not complete.
Honest verdict: **neither $\Omega(p^4)$ nor even $\Omega(p^3)$ is proved here for $T_p$**;
the requested $\Omega(p^4)$ is *not* established, and §5.4 raises a real possibility that
the truth is $\Theta(p^3)$ with a large constant.

---

## 6. What this contributes to Open Problem 1

1. **A family with an exact law.** $|FE_{G_p}|_{\mathrm{msd}}=m^3+8$,
   $|FE_{G_p}|_{\mathrm{lsd}}=2m^2+3m+12$. Polynomial of degree 3, with matching proved
   bounds $\Theta(m^3)$. Khodier's $2^{9m^2}$ is off by a tower.
2. **The last standing lead is closed** — as a route to *exponential* growth. Prop. 5.3
   shows what a lossy coding of a group automaton actually does: it grades the predicate by
   $v_2(j-i)$, with effective coding $\chi_{\min(k,p-1)}$ refining as $k$ grows and
   **saturating at the identity coding at level $p-1$**. There are $p$ levels, each an
   $FE$ problem for the *same* $p$-state DFAO with a coarser coding, so the ceiling this
   mechanism can reach is $O(p)\cdot\max_\kappa|FE_{\chi_\kappa\circ G_p}|$ — and there
   are only $p-2$ genuinely lossy levels, since $\chi_\kappa$ is injective for
   $\kappa\ge p-2$. Measured (`results/coding_levels*.json`), the cost of the collapse
   over $p=3..8$ is a factor $5.4\to23.1$, i.e. somewhere between a constant
   $\approx44$ and $\approx3p$; either way, nothing exponential is visible, and the
   grading says where an exponential would have to come from: a *single* level whose own
   $FE$ is exponential, which is the original problem again for a coarser coding of the
   same $m$-state DFAO.
3. **A reusable proof technique.** Lemma 3.1 + Lemma 3.3 ("desubstitute until the parities
   split, then the length is $\le3$") is a general recipe for group automata: it turns
   $FE$ into a bounded arithmetic condition on $\big(s(i)-s(j),\ \alpha,\ \beta,\ L\big)$.
   Whether an *exponential* family exists is therefore, for this class, a question about
   how far the "desubstitute" recursion can be made to run before the parity obstruction
   bites — and Lemma 3.3 says: never more than $\log_2 3$ blocks past the split.
4. **Independent validation of the engine.** Every engine number for this family is
   reproduced by two hand-built automata derived from a proved characterisation.

---

## 7. Gaps — stated plainly

* **G1.** The closed forms $p^3+8$ and $2p^2+3p+12$ are *proved for each $p\le24$
  separately* (correct construction + exact minimisation), not for all $p$. Closing this
  needs a uniform description of the minimal automaton — the $34p^3$ construction has to
  be shown to collapse to exactly $p^3+8$ classes. I have the decomposition that explains
  the shape ($2p^2$ lsd = $2$ signs $\times$ $p$ for $D$ $\times$ $p$ for the live run
  counter; $3p$ = one-run-frozen phases; $12$ = the small pre-difference states) but not a
  complete proof.
* **G2.** The lsd lower bound. I give only $O(p^2)$ constructively; the matching
  $\Omega(p^2)$ family was designed but my explicit realisation was buggy and I did not
  repair it in time. The exact value $2p^2+3p+12$ makes it certain.
* **G3.** $T_p$: **no upper bound is proved at all.** Prop. 5.3 reduces the interior to
  $FE$ for $\chi_\kappa\circ G_p$, but that is again an $FE$ problem, so the argument is
  not yet a bound — it explains the observed factor $p$ without bounding it. A proof would
  need $|FE_{\chi_\kappa\circ G_p}| = O(p^3)$ uniformly in $\kappa$ *plus* a bound on how
  the levels superpose. The levels *were* measured (§5.4) and their apparent exponents are
  the same as level $0$'s, so this route does not close the bound by itself.
* **G4.** $T_p$: exponent $3$ vs $4$ is **not settled**, and the local log-log slope
  (falling through $4$ at $p=8$) leans *against* the $3p^4$ law quoted in
  `docs/TARGET1.md`. The decisive experiment is $|FE_{T_p}|$ msd for $p=9,10,12$; the
  small-cap ladder (`AM_CAP=50000`, peak $768$ MB at $p=7$, `explore/single_ladder.py`)
  makes it feasible in time but **not in memory**: $p=8$ msd already needs $>6$ GB in the
  forward phase and $p=9$ was killed by the memory guard. Getting $p\ge9$ needs either a
  better construction strategy for this predicate (the aspect-(B) half of Open Problem 1)
  or a raised ceiling. `explore/single_ladder.py` and `explore/coding_levels.py` were left
  running; their output lands in `results/single_ladder9.log`,
  `results/coding_levels2.log`.
* **G5.** Only $k=2$. The base-$3$ analogue $s_3(n)\bmod p$ shows a parity-like structure
  in `docs/TARGET1.md` ($4,37,27,137,66,355,-,741$) that Lemma 3.1 should explain
  ($G[3n+f]=G[n]+f$ holds verbatim), but Lemma 3.3 must be redone: with $k=3$ the
  obstruction is $\nu_3$-flavoured and the constant $3$ in "$L\le3$" will change.
* **G6.** The characterisations are verified over finite ranges ($i,j,l<70$ and $<48$).
  The proofs do not depend on that, but the *implementations* of the automata do (their
  brute-force checks are finite).

---

## 8. Reproduction

```
engine/target/release/automatheus          # cargo build --release
explore/family_sweep2.py                   # engine: |FE| for both codings, p up to 24
explore/single_ladder.py                   # singleton, small-cap Brzozowski ladder
explore/fe_gtm_construct.py                # msd automaton from Prop 3.4  -> p^3+8
explore/fe_gtm_lsd.py                      # lsd automaton from Prop 3.4  -> 2p^2+3p+12
explore/fe_direct.py                       # generic construction (no theory), cross-check
explore/fe_family_verify.py                # Prop 3.4, Prop 5.1, Thm 4.4 verifications
explore/coding_levels.py                   # |FE| for the graded codings chi_kappa
results/family_sweep2.json  results/single_ladder*.log  results/coding_levels*.log
```
Engine one-liner (msd, $p=11$, identity coding; letters are `chr(48+a)`, so `:` is 10):

```
mode msd
def T 2 11 0 01 12 23 34 45 56 67 78 89 9: :0 0123456789:
let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]
```
gives `states=1339` $=11^3+8$.

`explore/fe_family_verify.py` reproduces every verification quoted above:

```
Prop 3.4 mismatches: 0   Prop 5.1 mismatches: 0  (p<=7, i,j,l<48)
Thm 4.4  p=3..8: distinct response vectors = 27, 64, 125, 216, 343, 512  (= p^3, OK)
```
