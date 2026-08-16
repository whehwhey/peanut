# LEARNFE — building the equality-of-factors automaton by guess-and-verify

`learnfe NAME` constructs the minimal DFA for

    FE(i,j,l)  :=  A t. t < l => T[i+t] = T[j+t]

for the current sequence `T` **without ever building the intermediate automaton that
the direct construction needs**, and registers it under `NAME` exactly as `let` would.
This is Khodier's "self-verifying predicate" idea (thesis §8, Open Problem 1, aspect B)
made into an engine command.

    def T 2 6 0 05 23 44 42 51 10 000010     # tail-c: `let FE` dies at 6 GB, Walnut OOMs
    learnfe FE
    OK learnfe FE(i,j,l) states=1382 iters=203 eqs=139 ces=1381 mqs=5992969 ... ms=15086
    ? A i,j. $FE(i,j,3) => ...               # usable like any other `let` predicate

Two new commands:

| command | meaning |
|---|---|
| `witness <formula>` | one satisfying assignment of the free variables — the SHORTEST accepted word — or `NONE`. Closed formulas degenerate to `TRUE`/`FALSE`. |
| `learnfe NAME` | build `FE(i,j,l)` as above and bind it to `NAME` with parameters `(i,j,l)`. |

---

## 1. Why the direct construction blows up, and what replaces it

`A t. t < l => T[i+t]=T[j+t]` is compiled as `~ E t. (t < l & T[i+t] != T[j+t])`.
Projecting `t` away turns a 4-track DFA into an NFA whose determinisation is the
pathology: Walnut peaks at 3.2e8 intermediate states / 300 GB on Tribonacci, whose
final answer has 26 states.  This engine's adaptive ladder (small forward cap →
Brzozowski → big forward cap → Brzozowski) rescues most cases, but not all: see
`results/blowup_residue3.log`.

Guess-and-verify sidesteps the projection entirely.  We produce a *candidate*
automaton `H` by active learning against a concrete oracle, and then check `H` against
a recurrence that only FE can satisfy.  No universal quantifier over `t` is ever
compiled.

## 2. The recurrence has a unique solution

**Claim.**  Let `H : N^3 -> {T,F}` satisfy, for all i, j, l,

    (R)   H(i,j,l)  <=>  ( l = 0  or  ( T[i] = T[j]  and  H(i+1, j+1, l-1) ) ).

Then `H = FE`.

*Proof.*  Induction on `l`, uniformly in `(i,j)`.

*Base `l = 0`.*  The right-hand side of (R) is true, so `H(i,j,0)` is true for every
`i,j`; and `FE(i,j,0) = "A t. t < 0 => ..."` is vacuously true.  They agree.

*Step.*  Let `l >= 1` and suppose `H(i',j',l-1) = FE(i',j',l-1)` for **all** `i',j'`.
The disjunct `l = 0` of (R) is false, so

    H(i,j,l) = ( T[i]=T[j]  and  H(i+1,j+1,l-1) )
             = ( T[i]=T[j]  and  FE(i+1,j+1,l-1) )                (induction hypothesis)
             = ( T[i]=T[j]  and  A t. t < l-1 => T[i+1+t] = T[j+1+t] )
             = ( A t. t < l => T[i+t] = T[j+t] )                  (split off t = 0)
             = FE(i,j,l).

Every `l` is reached, so `H = FE`.  ∎

FE itself satisfies (R) — the same "split off `t=0`" computation read backwards — so
(R) has **exactly one** solution.

**Consequence (why this is called self-verifying).**  Any candidate that passes the (R)
check *is* FE, regardless of how it was obtained.  The learner may be heuristic and its
membership oracle may even be wrong on some inputs: a bad guess costs iterations, never
correctness.  The only failure mode is non-termination, which the engine reports
(`ERR learnfe no progress ...` / `gave up after N iterations`) rather than hiding.

### How the check is actually run

To stay inside the engine's Presburger fragment we avoid the subtraction `l-1` by
reindexing, and split (R) into two sentences:

    (C1)  A i,j.    H(i,j,0)
    (C2)  A i,j,l.  H(i,j,l+1) <=> ( T[i]=T[j] & H(i+1,j+1,l) )

(C2) is (R) at every `l >= 1`; (C1) is (R) at `l = 0`; together they are (R).

Neither is compiled with `forall`, which would run a complement / subset-construction /
complement sandwich and reintroduce exactly the blowup we are avoiding.  Both have the
shape `A vars. Phi`, so the engine compiles only the **open** formula `Phi` over
`(i,j,l)` and asks whether the resulting (trimmed, minimised) DFA has a non-accepting
state.  Since every triple of naturals has a base-`k` representation and every
constituent automaton is value-based, "all states accepting" is exactly "the sentence
holds".  Cost: two `iff`-products of automata that are *equal languages* once `H` is
right, so the reachable product stays O(|H|), not O(|H|^2).

### Turning a violated recurrence into a counterexample

A witness `(i,j,l)` of `~(C2)` is a point where the recurrence fails, not directly a
point where `H` differs from FE.  But FE satisfies the recurrence, so if both
`H(i,j,l+1) = FE(i,j,l+1)` and `H(i+1,j+1,l) = FE(i+1,j+1,l)` held, the recurrence
would hold at `(i,j,l)`.  Hence at least one of `(i,j,l+1)`, `(i+1,j+1,l)` is a genuine
counterexample; both are tested with the membership oracle and whichever differs is fed
to the learner.  A witness `(i,j)` of `~(C1)` gives `(i,j,0)` directly.

Witnesses come from `Dfa::bfs_tree` / `word_to`: breadth-first search from the start
state yields a **shortest** word reaching each rejecting state.  Short counterexamples
are exactly what a learner wants, and one is harvested per rejecting state (up to
`AM_LEARN_WITNESS`, default 256) rather than one per round.

## 3. Membership oracle

    FE(i,j,l)  <=>  l <= LCP(i,j)

where `LCP(i,j)` is the length of the longest common prefix of the suffixes of `T` at
`i` and at `j`.  `LCP` is computed by walking two base-`k` counters through the DFAO in
lockstep and stopping at the first mismatch.  Each counter caches the DFAO state along
its msd digit path, so `n -> n+1` is O(1) amortised: no prefix array is materialised
(memory is O(log n), not O(n)), and positions of any size are reachable.  Results are
memoised per unordered pair `(i,j)` together with the cap they were computed under.

A hard cap `AM_LEARN_LCP` (default 2^22 steps) bounds the work per pair; a pair that
survives the cap is treated as `LCP = infinity`.  That is genuinely wrong for
eventually periodic `T` — and harmless, by §2.  The count of such answers is reported
as `capped_lcp=` when nonzero (tail-c: 42 of 6.0M queries; the result still verified).

## 4. Learner

Kearns–Vazirani discrimination tree over the `k^3`-letter track alphabet.

* **Alphabet / representation.**  Exactly the engine's normal convention: variables in
  sorted order `[i,j,l]`, symbol = `d_i + d_j*k + d_l*k^2`, words read in the active
  digit order (`mode msd` / `mode lsd`), padding zeros allowed.  Zero-robustness is
  automatic: the oracle answers on *values*, so the residual after `0^m` equals the
  residual after `ε` and minimisation gives `δ(q0,0) = q0`.  The result is a plain
  `Dfa` registered in `Defs` under `(["i","j","l"], dfa)`, so `$NAME(a,b,c)` works
  everywhere a `let` predicate does.
* **Counterexample handling.**  Rivest–Schapire binary search over the `n+1` values
  `α(p) = MQ(access(δ_H(q0, w[0..p])) · w[p..])`; `α(0) = FE(w)` and `α(n) = H(w)`
  differ, so some adjacent pair differs, found in `O(log |w|)` queries.  Each
  counterexample adds exactly one state, so the tree never exceeds the minimal DFA.
* **Incremental hypothesis.**  Splitting a leaf re-sifts only the transitions that
  pointed at it — one query each, against the new suffix — plus `k^3` sifts for the new
  state.  Rebuilding the whole hypothesis after every split (textbook KV) would cost
  `O(N^2 · k^3 · depth)` queries; this costs `O(N · k^3 · depth)` in total.
* **Cheap equivalence first.**  Equivalence queries build automata and are the expensive
  part, so before each one we run two pure-membership searches:
  1. *boundary sampling* — random `(i,j)` (uniform, near-diagonal, digit-perturbed) with
     `l` drawn from `{0, 1, LCP-1, LCP, LCP+1, random}`.  The language changes exactly at
     `l = LCP(i,j)`, so this is where counterexamples live;
  2. *local probe* — errors cluster, so from every counterexample found we crawl a
     bounded neighbourhood (`l ± 3`, `(i±1, j±1)`, `(i/k, j/k)`) keeping every
     disagreement.  The neighbourhood is deliberately magnitude-preserving: enlarging
     `i,j,l` lengthens words, long words become long distinguishing suffixes, and those
     make every later sift an expensive long LCP walk.  A first version that also
     crawled the `k`-ary children `(ki+a, kj+b, kl+c)` was measured and abandoned for
     exactly that reason: on `[s2 != 0 mod 5]` it had not finished after 150 s.

  Effect on `[s2 != 0 mod 5]` (1877 states): 205 equivalence queries / 29.3 s with no
  local probe, 146 / 19.9 s with the magnitude-preserving one, no completion in 150 s
  with the magnitude-growing one.

## 5. Memory

Everything runs under the existing counting allocator (`AM_MEM_MB`, `engine/src/membudget.rs`).
The learner's own footprint is negligible — the LCP memo plus `N·k^3` transitions — and
the peaks reported below are the verification automata.  Measured peaks: 50 MB
(tail-c, 1382 states), 50 MB (single5, 1877 states), 312 MB (prism-1, only 467 states but
over a 64-letter alphabet), 627 MB (single6, 3971 states).  Compare the 6144 MB budgets
exhausted by the direct construction on tail-c in both digit orders.

## 6. Results

`states` are minimal DFA states including the dead state (this repo's convention;
Walnut reports one fewer).  msd unless stated.  Machine: M-series Mac, release build,
shared with an unrelated 4 GB job, so times are upper bounds.

### 6.1 Agreement with the direct construction

Every case where `let FE` finishes.  `A i,j,l. $FE(i,j,l) <=> $G(i,j,l)` was checked
inside the engine and returned `TRUE` in every row.

| sequence | `let FE` | `learnfe` | same | `let` ms | learn ms | eqs | mqs | peak MB |
|---|---|---|---|---|---|---|---|---|
| thue-morse | 15 | 15 | true | 0 | 34 | 1 | 42502 | 0 |
| period-doubling | 8 | 8 | true | 0 | 236 | 1 | 39258 | 0 |
| rudin-shapiro | 68 | 68 | true | 127 | 188 | 2 | 175954 | 88 |
| paperfolding | 44 | 44 | true | 1 | 559 | 5 | 186091 | 1 |
| cantor | 17 | 17 | true | 0 | 1229 | 1 | 87222 | 1 |
| mephisto | 14 | 14 | true | 2 | 64 | 1 | 41619 | 1 |
| prism-a | 24 | 24 | true | 0 | 445 | 2 | 126887 | 0 |
| prism-d | 82 | 82 | true | 486 | 495 | 3 | 223626 | 132 |
| champion-m5 | 199 | 199 | true | 42 | 10845 | 10 | 545521 | 6 |
| k3m3-artefact-a | 216 | 216 | true | 54 | 2287 | 11 | 732790 | 21 |
| k3m3-artefact-b | 71 | 71 | true | 35 | 497 | 4 | 265244 | 6 |
| single3 `[s2!=0 mod 3]` | 190 | 190 | true | 12 | 363 | 9 | 410824 | 6 |
| single4 `[s2!=0 mod 4]` | 698 | 698 | true | 170 | 4767 | 43 | 1990616 | 12 |
| single5 `[s2!=0 mod 5]` | 1877 | 1877 | true | 2469 | 21833 | 146 | 7122707 | 50 |
| single6 `[s2!=0 mod 6]` | 3971 | 3971 | true | 24591 | 190326 | 406 | 16653361 | 627 |
| prism-1 (k=4, m=6) | 467 | 467 | true | 38594 | 17002 | 25 | 1693633 | 312 |
| rudin-shapiro (**lsd**) | 99 | 99 | true | 2 | 324 | 3 | 181082 | 1 |

The three targets named in the task — thue-morse 15, rudin-shapiro 68, prism-1 467 —
and the singleton family 190 / 698 / 1877 / 3971 reproduce exactly.

Independent check, not using the engine's own `let FE` and not using any automaton
machinery: brute force in pure Python over a morphism-generated prefix, enumerating
`FE(i,j,l)` for all `i,j,l < B` and diffing against `enum B $G(i,j,l)`.

* rudin-shapiro, B = 12: 416/416 tuples identical, in **both** msd and lsd;
* **tail-c**, B = 14: 826/826 tuples identical;
* **tail-b**, B = 11: 325/325 tuples identical.

The last two matter most: they are the results with no `let FE` to compare against.
(tail-b's 1000 also matches the value `bench/README.md` records for the one censored
sequence the direct construction did eventually manage, at 6 GB and 168 s.)

### 6.2 The cases the direct construction cannot do

These are the sequences on which `let FE` failed in **both** digit orders at a 6 GB
budget and failed again under the small-cap Brzozowski retry (`results/blowup_residue.json`,
`results/blowup_residue3.log`) — the "censored tail" of `docs/TARGET1.md`.  There is no
`let FE` number to compare against; correctness rests on `learnfe`'s own recurrence
check (§2), plus the pure-Python brute-force diffs in §6.1 for tail-b and tail-c.

Budget here: `AM_MEM_MB=2500`, 360 s wall clock, 3 concurrent engines, on a machine
simultaneously running an unrelated multi-gigabyte job.  Read the dashes as "not at this
budget on this machine", not as "impossible".

| case | k | m | `learnfe` states | s | eqs | mqs | peak MB |
|---|---|---|---|---|---|---|---|
| tail-c | 2 | 6 | **1382** | 14.2 | 139 | 5992969 | 50 |
| tail-b | 3 | 5 | **1000** | 25.7 | 61 | 3164860 | 179 |
| c-3.5b | 3 | 5 | — | timeout 360 s | | | |
| c-3.7a | 3 | 7 | — | timeout 360 s | | | |
| c-3.7b | 3 | 7 | — | timeout 360 s | | | |
| c-3.7c | 3 | 7 | — | timeout 360 s | | | |
| c-3.7d | 3 | 7 | **1147** | 44.4 | 80 | 4602591 | 109 |
| c-3.7e | 3 | 7 | — | timeout 360 s | | | |
| c-3.7f | 3 | 7 | — | timeout 360 s | | | |
| c-3.7g | 3 | 7 | — | timeout 360 s | | | |
| c-3.7h | 3 | 7 | **781** | 23.2 | 70 | 3846241 | 80 |
| c-3.7i | 3 | 7 | — | timeout 360 s | | | |
| c-3.7j | 3 | 7 | — | RSS watchdog at 4 GB | | | |

Definitions, in the order above:

```
tail-c   def T 2 6 0 05 23 44 42 51 10 000010
tail-b   def T 3 5 0 014 421 120 202 323 01100
c-3.5b   def T 3 5 0 044 230 312 401 141 00111
c-3.7a   def T 3 7 0 001 036 664 412 153 131 230 1101101
c-3.7b   def T 3 7 0 004 334 114 653 155 301 245 0011011
c-3.7c   def T 3 7 0 013 622 453 124 333 203 521 0011000
c-3.7d   def T 3 7 0 020 412 341 404 625 512 153 0101111
c-3.7e   def T 3 7 0 031 525 352 166 266 240 645 1011010
c-3.7f   def T 3 7 0 034 552 341 662 310 243 154 0111101
c-3.7g   def T 3 7 0 055 342 461 416 216 014 625 0001000
c-3.7h   def T 3 7 0 056 343 462 501 220 250 010 1010110
c-3.7i   def T 3 7 0 065 435 536 164 340 214 656 0110101
c-3.7j   def T 3 7 0 065 630 536 161 341 241 461 0011010
```


**4 of 13 previously-uncomputable FE automata now have a value**, and it is a *verified*
value.  The nine that remain are all k=3 — a 27-letter track alphabet, and k^6 = 729-letter
intermediates inside the equivalence check, so each equivalence query costs an order of
magnitude more than at k=2.  c-3.7j died on the runner's 4 GB RSS watchdog at 198 s
rather than on the clock.  Nothing here shows they are out of reach; only that they need
more than six minutes and 2.5 GB on a contended machine.

### 6.3 The learned automaton is a first-class predicate

`learnfe` binds its result exactly like `let`, so tail-c — which had no FE automaton at
all before — now answers questions instantly:

    def T 2 6 0 05 23 44 42 51 10 000010
    learnfe G
    OK learnfe G(i,j,l) states=1382 ... ms=16081 capped_lcp=42
    witness i < j & $G(i,j,60)
    WITNESS i=0 j=192 states=620 len=8 ms=8
    ? A i,j. (i<j & $G(i,j,60)) => j > i+3
    TRUE states=1 peak=2764 ms=7
    finite E j. j > i & $G(i,j,200)
    INFINITE states=1

So: the length-60 factor at position 0 recurs at position 192 (a shortest-word witness,
i.e. one with the fewest base-2 digits — not necessarily the numerically smallest pair);
no two occurrences of a common length-60 factor start within 3 of each other; and for
every `i` there are infinitely many `j > i` sharing a length-200 factor.  Each answer
took single-digit milliseconds on top of the 16 s construction.

## 7. Knobs

| env | default | meaning |
|---|---|---|
| `AM_LEARN_LCP` | 2^22 | LCP step cap per pair.  Too small and the capped language stops being small-regular, so the hypothesis grows past the true state count; too large and eventually periodic sequences waste time. |
| `AM_LEARN_LCP_MAX` | 2^26 | ceiling for automatic escalation on a stall |
| `AM_LEARN_SAMPLES` | 4000 | triples drawn per boundary-sampling round |
| `AM_LEARN_PROBE` | 2000 | counterexamples harvested per local-probe crawl |
| `AM_LEARN_WITNESS` | 256 | rejecting states harvested per equivalence query |
| `AM_LEARN_DIGITS` | 22 / 14 / 11 / 9 for k=2/3/4/other | max digits of sampled values |
| `AM_LEARN_ITERS` | 20000 | iteration ceiling before giving up |
| `AM_LEARN_DEBUG` | unset | per-round trace on stderr |
| `AM_MEM_MB` | 2048 | unchanged: the counting allocator still bounds the whole process |

## 8. What this does and does not settle

It moves aspect (B) of Open Problem 1 forward in the practical sense.  Every FE the
direct construction can build, `learnfe` builds identically (17/17, each equality proved
inside the engine).  Four sequences that the direct construction could not build at all
at 6 GB in either digit order now have verified FE automata — tail-c 1382, tail-b 1000,
c-3.7d 1147, c-3.7h 781 — at 50–180 MB and 14–45 s.  Because the construction is
verified rather than trusted, those sizes are exactly as sound as the direct ones: they
are the minimal DFA of a language *proved* equal to FE by a recurrence with a unique
solution.  Nine of the thirteen censored sequences remain open at the budget used here,
all of them k=3.

It says nothing about aspect (A).  `learnfe` makes more data points *available*; the
sizes it produced are all polynomial-looking, consistent with `docs/TARGET1.md`.

Two honest limits:

* `learnfe` is not uniformly faster.  On easy inputs the direct construction wins by
  orders of magnitude (thue-morse: 0 ms vs 32 ms).  Its running time scales with the
  *answer* size (roughly `N` counterexamples, `N/13` equivalence queries), not with the
  intermediate blowup — which is exactly why it wins where the direct method dies, and
  loses where the direct method is fine.  A sensible driver runs `let FE` first with a
  small cap and falls back to `learnfe`.
* The membership oracle assumes `LCP = infinity` past its step cap.  This cannot make a
  reported automaton wrong (§2), but it can make the learner chase a non-regular target
  and stall; the engine detects the stall, raises the cap, and relearns.

## 9. Files

* `engine/src/learn.rs` — oracle, discrimination-tree learner, verifier, driver
* `engine/src/dfa.rs` — `bfs_tree`, `word_to`, `shortest_word` (witness extraction)
* `engine/src/main.rs` — `witness` and `learnfe` commands
* `explore/learnfe_bench.py` — reproduces both tables (`panel` / `censored`)
* `results/learnfe_panel.json` — agreement table, raw
* `results/learnfe_censored.json` — censored-tail table, raw
