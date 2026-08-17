# Referee verdict on `docs/ATTACK-1.md` (shifted Thue–Morse state complexity)

Adversarial read, 2026-08-17, to the standard of `paper/proof-verdict.md` /
`paper/proof3-verdict.md`. Every load-bearing lemma re-derived by hand; every finite
claim re-checked with code **written from scratch for this review**
(`paper/verdict-attack1/`), never through the authors' scripts; the primary sources
(arXiv:2512.10017v5, the Biswas manuscript, Moradi's thesis, OEIS A382296/A382298)
fetched and read. The whole 262 144-term table was recomputed independently.

## One-line verdict

**Theorems 1, 2, 3 and the corollary algorithm are CORRECT, and Theorem 1 is a genuine
new sharpening of the published construction. The data is exactly reproducible — I
rebuilt all 262 144 values twice, from two constructions that do not use Theorem 1, and
they agree to the last digit with the note, with the OEIS b-file on `c ≤ 12000`, and with
the engine on `c ≤ 400`. Open Problem 2 is *not* solved and the note says so. One
sub-claim inside F4 is false; two others are misstated; and the note's §5 ledger omits
the two published results (MRS Thm 9, Thm 19) that Theorem 1 actually supersedes — one of
which I find to be false as published.**

## Status of every claim

| claim | verdict |
|---|---|
| Thm 1(i) offset-closure formula for `N(c)` | **PROVED** (re-derived; 22 118 values checked) |
| Thm 1(ii) minimal msd DFAO, transitions, output, `a(c)=|Q_c|` | **PROVED** (re-derived; DFAO run on 48 000 inputs, 0 errors) |
| Thm 2 `a(c)` even | **PROVED** (re-derived; 0 odd in 262 144) |
| Thm 3 `a(c) ≥ a(⌊c/2⌋)` | **PROVED** (re-derived; 0 violations in 262 144) |
| Corollary: `O(a(c)·log c)` algorithm, 12 001 terms in 2.1 s | **PROVED** + timing reproduced (2.05 s user) |
| `results/attack1_a.txt.gz`, `c < 262144` | **MACHINE-VERIFIED** (independently recomputed in full) |
| identity `(*)`, morphism, digit concatenation, Biswas adjacency, locality failure | **MACHINE-VERIFIED** (my own 502-sentence engine battery, all TRUE) |
| F1 block minima `= a(2^m) = 2F_{m+3}`, unique | **PLAUSIBLE** (m ≤ 17 exhaustive; m = 18 adversarially probed here) |
| F2 `κ(d)` family laws | **PLAUSIBLE** (independently verified to `r = 24`, 267 instances) |
| F3 2-kernel rank `2^{e+1}−1`, hence not 2-regular | **PLAUSIBLE** (rank reproduced mod two primes; non-regularity unproved) |
| F4 `max_{c<2^m} a(c) ~ C·1.919^m` | **PLAUSIBLE** (ratios reproduced) |
| F4 "argmax converges digit-by-digit to a fixed infinite word" | **WRONG** — contradicted by the note's own data |
| F1 "with the optimal constant" (2.342) | **misstated** — the data supports 3.708 |
| §4 "engine limitation found" | **stale** — fixed in commit `63ad97e` |
| Open Problem 2 solved? | **No** — and correctly not claimed |

---

## 0. Primary sources (all fetched and read for this review)

| source | what it actually says |
|---|---|
| arXiv:2512.10017 **v5, 2 Apr 2026** (Moradi–Rampersad–Shallit) | §4.2 **Open Problem 2** verbatim: *"What is a good formula for the exact number of states in the minimal automaton generating `(t(i+c))_{i≥0}` with msd-first input, as a function of c?"* — exactly as the note quotes it. **Thm 20**: `> c^0.694` states for infinitely many `c` (proof does `c = 2^n`, `F_{n+1}` inequivalent words). **Thm 19**: `a(c) ≤ (10/3)c` for all `c ≥ 1`. **Thm 9(b)**: an automaton with `ρ_{t'}(c+1) = Θ(c)` states (window `[h'(p),…,h'(p+c)]`), explicitly *not* minimal. |
| OEIS **A382296** (`#52 Aug 10 2026`) | b-file `n = 0..12000` by Shallit; formula line *"It appears… a(2^n)=2F(n+3)"* + *"This conjecture is proved. See Biswas link. — Ranadeep Biswas, Aug 02 2026"*. |
| **Biswas**, *State Complexity of Thue-Morse Shifts by Powers of Two* (github.com/rnbguy/proofs, 5 pp.) | Thm 2.1: `A_r` computes `w ↦ t(V(w)+2^r)`, has exactly `2F_{r+3}` states, and is minimal. §1: *"They pose the exact state complexity for arbitrary c as Open Problem 2. This manuscript determines the exact value for the power-of-two subfamily."* Power-of-two **only**. |
| Moradi, MSc thesis, Waterloo 2026 (p. 43 / Problem 2) | restates Problem 2 verbatim; no formula, no partial result beyond Thm 20. |
| citation search | Semantic Scholar lists **one** citing paper: Moradi–Popoli–Shallit–Vukusic, *State Complexity of Shifts of the Fibonacci Word*, arXiv:2603.18858 (2026) — proves `O(log c)` for the **Fibonacci** word, does not touch base-2 Thue–Morse. arXiv:2603.21645 (Fibonacci-automatic) likewise not. arXiv full-text search for "Thue-Morse" + "state complexity" returns nothing after 2019. |

**Conclusion: the target is live.** General `c` is open; the `c = 2^r` line is closed by
Biswas (Aug 2026); the note's §0 status table is accurate.

---

## 1. Theorems 1–3 — re-derived by hand

### Theorem 1(i)
Composing the two maps `f_b(n) = ⌊(n+b)/2⌋` in any order gives
`f_{b_L}∘⋯∘f_{b_1}(c) = ⌊(c+e)/2^L⌋` with `e = Σ b_i 2^{i-1}` ranging over `[0,2^L)`.
Writing `c = q_L·2^L + r_L`, the value is `q_L` or `q_L+1`, and `q_L+1` is attained iff
`r_L ≥ 1`, i.e. iff `L > v_2(c)`. Hence exactly the stated set, `max N(c) = c`,
`|N(c)| ≤ 2ℓ+1`. **Correct.** Machine check: closure vs formula for all `c ≤ 20000`, all
`2^k`, `2^k±1` up to `k = 39`, and 2 000 random 40-bit `c` — **0 mismatches / 22 118**;
`max |N(c)| = 29` for `c < 20000` (`= 2ℓ+1`, so the bound is tight).

### Theorem 1(ii)
`V_c(2p+b)[n] = t(2p+b+n)`; put `n+b = 2m+s`, then `t(2(p+m)+s) = t(p+m) ⊕ s` with
`m = (n+b)>>1 ∈ N(c)` by closure. Output `V[c] = t(p+c)`; `V_c(0)·0 = V_c(0)` handles
leading zeros. Distinguishability: with `s + r_L = ε·2^L + s'`,
`t(p·2^L + s + c) = t(p+q_L+ε) ⊕ t(s')` — the identity `(*)`, correct because
`t(x·2^L+y) = t(x) ⊕ t(y)` for `y < 2^L`. Coordinate `q_L` is reached with `s = 0`;
coordinate `q_L+1` needs `s ≥ 2^L − r_L`, which exists exactly when `r_L > 0`, i.e.
`L > v` — precisely the coordinates Thm 1(i) says are present. Every coordinate of `N(c)`
is of one of these two shapes, so equal vectors ⇒ equivalent prefixes and distinct vectors
⇒ distinguishable. **Minimality is genuinely proved.** No gap.

*Machine check of the theorem as stated* (`verdict-attack1/`, my code): built the DFAO
literally from the statement for `c ∈ {0,1,2,3,5,7,8,11,37,64,100,255,300,1000,1023,2000}`
and ran it on 48 000 inputs (with random leading zeros): **0 output mismatches**, and the
reachable-state counts equal my independent counts.

### Theorem 2
`¬` commutes with the transition because the gather is coordinate-wise and the flip mask
`(n+b) mod 2` is constant: `(¬V·b)[n] = ¬V[(n+b)>>1] ⊕ ((n+b)&1) = ¬((V·b)[n])`.
`V_c(2^K) = ¬V_c(0)` for `2^K > c = max N(c)`, so `Q_c` is `¬`-closed and `¬` is
fixed-point free. **Correct.** 0 odd values in 262 144.

### Theorem 3
`⌊c/2⌋ ∈ N(c)` and `N` is a closure, so `N(⌊c/2⌋) ⊆ N(c)` (verified for all
`c < 20000`); coordinate restriction is a well-defined surjection `Q_c ↠ Q_{⌊c/2⌋}`.
**Correct.** 0 violations in 262 144.

### Free corollary the note misses
Thm 3 iterated + Biswas gives an **unconditional** general-`c` bound: writing
`c = 2^m + r` and `s = ` bit-length of `r`, `⌊c/2^s⌋ = 2^{m-s}`, so

```
a(2^m + r)  ≥  a(2^{m-s})  =  2 F_{m-s+3}          (unconditional, m ≥ s)
```

0 violations on the table. It is the only *proved* lower bound for general `c` anywhere in
the note's orbit, and it is worth stating even though it is weak (median slack factor
2.2·10⁴; it only bites when `r ≪ c`). It does **not** give `c^{0.694}` in general — F1
stays a conjecture.

---

## 2. The data — recomputed from scratch, twice

Two implementations written for this review, neither using Theorem 1:

* **Route A** (`paper/verdict-attack1/routeA.c`) — explicit **lsd** DFAO from a
  ripple-carry adder plus a running parity accumulator (states `(j,carry,parity)`,
  `≤ 4(ℓ+1)`), then the msd minimal automaton as residual vectors
  `g_u(s) = out(δ*(s, rev u))` restricted to the lsd-reachable set. Myhill–Nerode classes
  by construction.
* **Route C** (`paper/verdict-attack1/routeC.c`) — no automaton theory at all: BFS over
  prefix values `p → 2p, 2p+1`, states identified by the raw response table
  `(t(p·2^L+s+c) : L ≤ ℓ+3, s < 2^L)` computed by popcount, dedup on the full table.

| check | range | result |
|---|---|---|
| route A vs the note's `results/attack1_a.txt.gz` | **all `c < 262144`** | **identical** |
| route C vs the note's table | `c ≤ 2000` | identical (max BFS depth 13) |
| route C stability in suffix length (`ℓ+1` vs `ℓ+5`) | `c ≤ 59`, `1000 ≤ c < 1010` | identical |
| route A vs OEIS b-file `b382296.txt` (fetched today) | `c ≤ 12000` | identical |
| note's table vs b-file | `c ≤ 12000` | identical |
| **engine**, `mode msd; def T 2 2 0 01 10 01; dfa T[i+c]=1` (my own driver) | `c ≤ 400` | identical, 401/401 |
| engine, `mode lsd` | `c ≤ 40` | identical, and equals **A382298** term-for-term |
| the note's `attack1_states.c` vs route A | `c ≤ 12000`, plus 60 random `c ∈ [2^18,2^19)` | identical |

Timing: the note's C program does `c ≤ 12000` in **2.05 s** user on this machine — the
claimed 2.1 s is right. (My route A takes 7.9 s; it is doing more work per `c`.)

---

## 3. Machine proofs of the lemmas — my own battery, 502 sentences

`paper/verdict-attack1/lem.py` (my sentences, freshly compiled, not the note's 48):

| block | sentences | result |
|---|---|---|
| morphism `T[2n]=T[n]`, `T[2n+1]≠T[n]` | 2 | TRUE |
| digit concatenation `T[2^L x + s] ≡ T[x] ⊕ T[s]`, `L = 1…11`, `s ∈ {0,1,2^L−1}` | 33 | TRUE |
| **the identity `(*)` for every `s < 2^L`**, `(c,L) ∈ {(5,1),(5,2),(11,3),(37,4),(300,5),(1000,7),(2000,8)}` | 446 | TRUE |
| Biswas adjacency `T[p]≠T[p+2^j] ∨ T[p]≠T[p+2^{j+1}]`, `j = 0…12` | 13 | TRUE |
| locality failure `∃p. T[p]=T[p+q]=T[p+2q]`, `q ∈ {3,5,6,7,9,11,12,13}` | 8 | TRUE |

**502 / 502 TRUE, 0 FALSE, 0 ERR, 42 s.** The note checked `(*)` at three `s` values per
`(c,L)`; I checked **all** of them. The load-bearing identity of Theorem 1 is
machine-confirmed exhaustively in that range.

---

## 4. F1–F4, independently

### F1 — holds, and one block further than the note went
My table reproduces the note's §2 block table exactly (min, argmin, `2F_{m+3}`, max,
argmax, ratio, mean `a/c`, all `m = 1…17`), with unique minimiser `c = 2^m` in every
block.

*Adversarial probe at `m = 18`* (beyond the note's data): `a(2^18) = 21892 = 2F_21`; I
computed `a(2^18+d)` for **all** `d < 4096` and for **300 random** `c ∈ [2^18, 2^19)`.
Minimum over all 4 395 competitors is **35 422 = 2F_22** at `c = 2^18+1`. **F1 survives.**
The probe is not blind: sorting block 17 shows the 40 smallest values sit at offsets
`0,1,2,4,3,8,5,16,9,32,17,64,33,65,128,…` — sparse low bits — so `d < 4096` covers the
dangerous region.

*A sharper conjecture that would prove F1* (new here). Let
`P(c) = #{ (t(p+⌊c/2^L⌋))_{L=0..ℓ} : p ≥ 0 }` — the projection of `Q_c` onto the **descending
chain only** (`ℓ+1` coordinates, not all of `N(c)`). Since chain elements are coordinates
of `V_c`, `P(c) ≤ a(c)`. Measured (`paper/verdict-attack1/proj.py`):

```
P(c) ≥ 2F_{m+3} for every 2 ≤ c < 4096
equality exactly at c = 2^m  (checked over 2 ≤ c < 2048)
min P(c)/a(c) = 0.569       (so P is a strictly smaller object than a)
```

So **F1 reduces to a statement about one chain `c, ⌊c/2⌋, ⌊c/4⌋, …, 1, 0`** — literally
Biswas's setting with the chain `2^r,…,1,0` replaced by an arbitrary one, and his
`E_{r+1}` ("no adjacent zeros") count `F_{r+3}` conjecturally the minimum over all chains.
That is a much smaller target than F1 as stated, and the note does not notice it.

### F1's constant is understated, and "optimal" is the wrong word
`2.342 = 2φ²/√5` is what F1 yields at the *top* of a block. The sequence itself does much
better: `min_{1 ≤ c < 262144} a(c)/c^{0.694242} = 3.7082` (at `c = 2`), and along the
minimisers `a(2^m)/2^{0.694242m} → 2φ³/√5 = 3.7889`. So the honest reading of the data is
`a(c) ≥ 3.708·c^{0.694242}` on the whole range — 58 % stronger than the note's own
headline — and the phrase "with the optimal constant" is false as written (2.342 is
optimal only as a *consequence of F1*, not for `a`).

### F2 — verified independently to `r = 24`
Recomputed `κ(d)` and `r0(d)` from my table: **every entry of the note's table matches**
(`κ(−1,1,2,3,4,5,8,9,16,17,32,33,64,65) = −6,0,2,8,6,20,16,42,36,82,76,154,152,282`;
`r0 = 1,1,2,3,3,4,4,5,8,8,10,10,12,12`). Extending with route A to `c = 2^24 + d`
(97 fresh values, largest `c = 16 777 281`): **267 instances of
`a(2^r+d) = 2F_{r+4}+κ(d)`, 0 violations.** The "exactly these `d`" claim survives 20
further `d` values (`−6…−2, 6,7,10…15,18,20,34,66, −9,−17,−33`): none has a constant tail.
`a(2^r+1) = a(2^{r+1})` confirmed for `r ≤ 16`.

C-finiteness: `a(2^r+d)` for `d ∈ {−1,2,3,4}` satisfies `x_n = 2x_{n−1} − x_{n−3}` for
`r` past the transient (the note should say *eventually*: `d = 3` and `d = 4` fail the
recurrence at `r = 5` because `r = 2` is pre-asymptotic), and so does `d = −2` even though
its offset from `2F_{r+4}` **grows** — that family is Fibonacci-like with a different
coefficient, which the note does not explain. With 19 terms (I computed
`a(3·2^r), a(5·2^r)` out to `c = 1 310 720`) the minimal orders are **7 for both
`3·2^r` and `5·2^r`**, exactly as claimed; `7·2^r` is undetermined at 16 terms.

### F3 — rank reproduced exactly
My own Gauss elimination (`kernel.py`, `kernel_np.py`), rows `n ↦ a(2^e n+r)`:

```
e ≤ 6 : 127 rows × 2000 cols  rank 127   (mod 2^31−1 and mod 10^9+7 — full)
e ≤ 8 : 511 rows × 1000 cols  rank 511   (mod 2^31−1 — full)
e ≤ 9 : 1023 rows ×  500 cols  rank 500  (column-limited; the table cannot decide e = 9)
```

Full mod-`p` rank *proves* full rank over `Q`, so the note's "exact over `Q` at `e ≤ 6`,
mod `p` at `e ≤ 8`" is if anything over-cautious. The inference to "not 2-regular" remains
evidence only, and the note says so.

### F4 — the growth rate is fine, the "limit word" is not
Block-max ratios reproduce (`1.9000, 1.9131, 1.9203, 1.9175, 1.9192` for `m = 13…17`), as
does the decaying mean `a(c)/c`.

**Defect (real).** "The maximising `c` in each block converges digit-by-digit to a fixed
infinite binary word beginning `1 1 1 0 1 1 1 1 1 0 1 1 1 0 0 0 1 1`" is **false on the
note's own data**. The quoted word is simply the `m = 17` argmax. Longest common prefix of
consecutive block argmaxima:

```
m      : 2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17
lcp(m-1,m): 2  2  2  4  6  2  7  2  9 11 11  7 10 15 14 11
```

It is not monotone: it collapses from 11 (`m = 12`) to **7** (`m = 13`), and again from 15
(`m = 15`) to 14 to 11. There is no digit-by-digit convergence in the range computed; at
best the argmax words share a `111011111`-ish head. Repair: state the argmax table and
drop the limit-word sentence, or say "the leading digits are stable but the agreement is
not monotone".

---

## 5. Two things wrong outside the note, found while checking it

1. **MRS Theorem 19 (arXiv:2512.10017 v5) is false as stated.** It asserts
   `a(c) ≤ (10/3)c` for **all** `c ≥ 1`, but `a(1) = 4 > 10/3` — and `a(1) = 4` is in
   Shallit's own b-file. The proof splits `c = 2^p + r'` into `0 < r' ≤ 2^{p-1}` and
   `2^{p-1} < r' ≤ 2^p` and never covers `r' = 0`; `c = 1` is the unique failure in
   `c < 262144`. The bound is otherwise correct and **exactly tight at `c = 3`**
   (`a(3) = 10 = (10/3)·3`); `max_{c ≥ 100} a(c)/c = 2.3867` at `c = 181`. Worth reporting
   upstream, and worth a line in the note's ledger, which never mentions Thm 19 at all.
2. **The note's "engine limitation found" is stale.** The `assert!(c < 100_000)` it
   describes was real at the commit where ATTACK-1 was written, but it is now fixed
   (commit `63ad97e`: `MAX_CONSTANT = 10^12`, returns `Err`). With the current binary the
   failure mode for `c = 150000` is a clean `ERR memory budget exceeded`, not a process
   abort. The paragraph should be rewritten as "was fixed", not "worth turning into a
   returned error".

---

## 6. Attribution — the ledger is honest but incomplete

Correctly credited: A382296/b-file, Open Problem 2, Biswas's `2F_{r+3}`, MRS Thm 20,
A382298. Missing: **MRS Theorem 9(b) and Theorem 19**, which are the prior art Theorem 1
supersedes. Thm 9(b) gives an automaton for `t(i+c)` with `ρ_{t'}(c+1) = Θ(c)` states
(window of `c+1` interior values) and is explicitly non-minimal in the paper; Thm 19 turns
it into `a(c) ≤ (10/3)c`. Theorem 1 replaces the `c+1` window by the `≤ 2ℓ+1` offsets of
`N(c)` **and** proves minimality. That is a real gain (`Θ(c) → Θ(log c)` coordinates, plus
exactness) and stating it against Thm 9/19 makes the contribution *more* visible, not less.
I found no source containing Theorem 1 for general `c`; relative to the literature I can
see, it is new.

## 7. Weakest steps

1. **F1 and F2 are the note's real content beyond Theorem 1, and neither is proved.** F2
   in particular looks close: the difference vectors stabilise, and the note itself says a
   per-`d` induction in `r` would do it. Nobody set it up.
2. **F3's headline ("no good formula of transfer-matrix type") is an inference from a rank
   computation, not a theorem**, and it is doing a lot of rhetorical work — it is the
   note's explanation for why Open Problem 2 has no clean answer. A proof of
   non-2-regularity is a separate open problem, as §5 admits.
3. **Nothing in the note constrains `a(c)` from above for general `c`** beyond the
   published `(10/3)c`. The observed `c^{0.9405}` is a five-point ratio fit.

## 8. Bottom line

Open Problem 2 is **not** solved and the note does not claim it is. What it does deliver —
a proved, minimal, `O(log c)`-coordinate automaton for **every** `c`, an exact
`O(a(c)·log c)` algorithm, a 21× table extension that reproduces bit-for-bit under two
independent reimplementations, and four sharp conjectures — is real, and the honest ledger
is accurate apart from the F4 limit-word sentence and the "optimal constant" phrasing.
The most promising unclaimed lead in the material is the chain projection of §4: F1 is
equivalent to a Fibonacci-minimality statement about the single chain
`c, ⌊c/2⌋, …, 1, 0`, verified here for all `c < 4096` with equality exactly at the powers
of two.

---

## Appendix — checks run for this review

| file | what it does |
|---|---|
| `paper/verdict-attack1/routeA.c` | lsd adder DFAO + residual-vector msd minimisation; `a(c)` for any `c` |
| `paper/verdict-attack1/routeC.c` | raw Myhill–Nerode brute force over response tables |
| `paper/verdict-attack1/eng.py` | fresh engine driver (`dfa T[i+c]=1`), msd and lsd |
| `paper/verdict-attack1/lem.py` | 502-sentence engine battery (morphism, concatenation, `(*)`, adjacency, locality) |
| `paper/verdict-attack1/analyse.py` | evenness, Thm 3, ratio bound, block table, `κ` laws |
| `paper/verdict-attack1/kernel.py`, `kernel_np.py` | 2-kernel rank mod `p`, exact Gauss |
| `paper/verdict-attack1/proj.py` | chain-projection experiment `P(c)` vs `2F_{m+3}` |
| `paper/verdict-attack1/block18.txt` | `a(2^18+d)`, `d < 4096`, + 300 random `c ∈ [2^18,2^19)` (F1 probe) |
| `paper/verdict-attack1/fam_r24.txt` | `a(2^r+d)` for the 14 constant families, `r ≤ 24` |

```sh
cc -O3 -o /tmp/rA paper/verdict-attack1/routeA.c && /tmp/rA 0 262144 > /tmp/a.txt
cc -O3 -o /tmp/rC paper/verdict-attack1/routeC.c && /tmp/rC 0 2001 3
python3 paper/verdict-attack1/eng.py 0 400 msd
python3 paper/verdict-attack1/lem.py
python3 paper/verdict-attack1/analyse.py /tmp/a.txt
python3 paper/verdict-attack1/kernel_np.py /tmp/a.txt 8 1000
python3 paper/verdict-attack1/proj.py 2 4096
```
