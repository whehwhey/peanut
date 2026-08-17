# Referee verdict — numeration systems (`docs/NUMERATION.md`, `bench/fib.md`, `engine/numeration/*`)

Adversarial read, 2026-08-17, repo at `4beb34e`, engine `engine/target/release/peanut`
(built 2026-08-17), Walnut `walnut7` v8.0-alpha (`target/Walnut-all.jar`, OpenJDK 26,
`-Xmx6g`), macOS 26.5 / arm64 / 24 GB.

Everything below was re-derived by code **written from scratch for this review**
(`paper/verdict-numeration/`): my own greedy Zeckendorf / Tribonacci / Pell converter, my
own substitution fixed points, my own parser and simulator for the Walnut "Custom Bases"
text format. Nothing routes through `explore/gen_numsys.py`, `explore/numsys_check.py` or
`explore/morphic_to_dfao.py` — those were run once, separately, only to confirm they still
pass and that the shipped files regenerate byte-identically. The engine was only ever
launched through `explore/engine.py`. The primary source (`docs/khodier2026-thesis.pdf`)
was extracted and read.

## One-line verdict

**The numeration layer is CORRECT and `bench/fib.md` is exactly reproducible — every
Walnut state count and peak in the table reproduced to the digit, every Peanut number
reproduced to the digit, all nine TRUE/FALSE verdicts confirmed by independent brute
force on a 10^6 prefix, and the headline Tribonacci FE automaton confirmed three ways
(brute force on 1.6 x 10^6 triples, minimality, and language equivalence with Walnut's own
26-state automaton). Two claims in the write-ups are false as stated — bench's "ours =
Walnut + 1 wherever both finish ... a free independent check of every number here" (it
holds in one row of eleven) and `NUMERATION.md`'s "`mode lsd` costs nothing extra and
remains the independent oracle" (lsd exhausts 6 GB on four Tribonacci queries msd answers
in ≤ 4.3 s) — one documented refusal does not exist (`def` under a numeration system is
accepted, and aborts the process when `k` mismatches the digit count), and the headline
fairness row that the caveats gesture at was never measured: with Khodier's own
reformulated predicate Walnut builds the 26-state Tribonacci FE in 10.2 s with peak
18 853, not OOM.**

## Status of every claim

| claim | verdict |
|---|---|
| `value(rep(n)) == n`, greedy = canonical, `n < 2*10^5` (fib, trib, pell) | **MACHINE-VERIFIED** (mine) |
| validity DFA language = greedy reps; lex rank within a length = numeric value | **MACHINE-VERIFIED** (all words `\|w\| ≤ 12`) |
| `U_l = #{valid words of length l}` = 1,2,3,5,8… / 1,2,4,7,13… / 1,2,5,12,29… | **MACHINE-VERIFIED** |
| adder accepts `(x,y,x+y)` and nothing else | **MACHINE-VERIFIED** (10^5 random pairs + 10^5 negatives per system, per source; plus exhaustive) |
| our adders ≡ Walnut's `msd_{fib,trib,pell}_addition.txt` on valid triples | **MACHINE-VERIFIED** (product BFS, mine, independent of `gen_numsys.py`) |
| our validity DFAs ≡ Walnut's `msd_{fib,trib,pell}.txt` | **MACHINE-VERIFIED** (product BFS) |
| `dfao F/TR` = the Fibonacci / Tribonacci word | **MACHINE-VERIFIED** (10^6 terms, from my Zeckendorf digits) |
| all 9 TRUE/FALSE in `bench/fib.md` | **CONFIRMED** by brute force on 10^6 (see §3 for the logical status of each) |
| Walnut column of `bench/fib.md` (states, peak, OOM) | **REPRODUCED EXACTLY**, 11/11 rows |
| Peanut column of `bench/fib.md` (states, peak) | **REPRODUCED EXACTLY**, 13/13 rows |
| Tribonacci FE = 27 = 26 + dead, and 26 is Khodier's minimal automaton | **CONFIRMED** (thesis p. 3, p. 47; and I rebuilt Walnut's 26-state automaton and proved it equivalent) |
| `learnfe` FE (msd 12/27, lsd 14/29) = the true FE relation | **MACHINE-VERIFIED** (1.6 x 10^6 triples each) |
| compiled DFAs accept exactly `valid tracks ∧ relation` (incl. `=>` and `~`) | **MACHINE-VERIFIED** (15.1 x 10^6 words exhaustively) |
| Walnut base files `msd_pisot4/kim/nara` load; `neg_fib/tib/ns` refused | **CONFIRMED** |
| `explore/gen_numsys.py` reproduces `engine/numeration/*` byte-identically | **CONFIRMED** (`git status` clean after a run) |
| bench: "ours = Walnut + 1 wherever both finish — a free independent check of every number here" | **FALSE** — true in 1 row of 11 |
| bench: "Every TRUE/FALSE above is independently confirmed … (`explore/numsys_check.py`)" | **OVERSTATED** — the two palindrome rows are not in that script (they are true; I checked them) |
| bench: Walnut's own base files "give the numbers above unchanged" | **NEARLY** — one peak changes, 50 → 41 |
| NUMERATION.md: "`def` … is likewise a base-k notion" (implying refusal) | **FALSE** — accepted; aborts (SIGABRT) on a digit-count mismatch |
| NUMERATION.md: "`mode lsd` costs nothing extra and remains the independent oracle" | **FALSE for Tribonacci** — 4 of 5 queries exceed 6 GB |
| NUMERATION.md §4 example `dfao WF @"…/F.txt"` | **DOES NOT WORK** — quotes are not stripped |
| known-gaps ledger (`pic` wrong under numsys, negative outputs, `enum` needs a sequence, refused systems) | **HONEST** — every item reproduced as described |

---

## 1. Numeration arithmetic (`paper/verdict-numeration/check1_validity.py`, `check2_adder.py`, `check3_equiv_walnut.py`)

My converter is greedy-from-the-top against weights `1,2,3,5,8,…` / `1,2,4,7,13,…` /
`1,2,5,12,29,…`, with no reference to the engine. Results:

* round trip and canonicity for every `n < 200 000` in all three systems;
* the shipped validity DFA accepts exactly the valid words: for all `|w| ≤ 12`, the
  lexicographic rank of `w` inside its own length class equals its greedy value, the count
  per length equals `U_l`, and the empty word (the representation of 0) is accepted. This
  is the load-bearing invariant of the "values are ranks" design (`NUMERATION.md` §2) and
  it holds. Note the doc's `weights=1,2,3,5,…` line counts the empty word as `U_0 = 1`;
  reading it as "words of length 1" is off by one and cost me one false alarm.
* the adder: **100 000 random `(x,y)` with `x,y` up to 10^15, plus 100 000 negatives
  `z = x+y+d`, for each of the six adder files** (ours and Walnut's, three systems) —
  0 misses, 0 false accepts; plus exhaustive `x,y < 120` and exhaustive `x,y < 40,
  z < 120` (`z` accepted iff `z = x+y`);
* **product-BFS equivalence** (not sampling): our adder and Walnut's accept the same
  language of valid triples, in all three systems, and likewise the validity DFAs. This
  independently confirms the strongest claim in `NUMERATION.md` §4.

Ours is bigger than Walnut's on `fib` (16 + dead vs 7 + dead) and on `trib` (43 + dead vs
149 + dead — here ours is *smaller*); the sizes differ, the languages do not.

## 2. The words (`check4_words.py`)

From the substitutions `0->01,1->0` and `0->01,1->02,2->0`, 10^6 terms each. The DFAO
lines used in the benchmark (`dfao F 2 0:0,1 1:0,-`, `dfao TR 2 0:0,1 1:0,2 2:0,-`),
simulated by me over my own Zeckendorf/Tribonacci digits, reproduce those 10^6 terms
exactly; the engine's own `seq 100000` agrees; Walnut's `Word Automata Library/F.txt` and
`TR.txt` agree for `n < 2*10^5`. The prefixes printed in `bench/fib.md` are correct, and
the Tribonacci prefix matches the thesis (`010201001020101020100102…`).

## 3. Every TRUE/FALSE, brute-forced (`check5_verdicts.py`)

10^6-term prefixes, numpy. A decision procedure proves these; brute force can only refute
or corroborate, so the logical status of each row is stated.

| row | claim | brute force | status |
|---|---|---|---|
| fib `T[i]=1 => T[i+1]=0` | TRUE | 0 counterexamples in 10^6 | corroborated (universal) |
| fib eventually periodic | FALSE | for every `p ≤ 20 000` there is a mismatch at index ≥ 971 143 | refutes every witness with `p ≤ 20 000` |
| trib eventually periodic | FALSE | same, mismatch at index ≥ 962 415 | ditto |
| fib cube | TRUE | witness: period 3 at position 5 | **witness exhibited** |
| trib cube | TRUE | witness: period 7 at position 37 | **witness exhibited** |
| fib 4th power | FALSE | **every** period `p ≤ 250 000` searched over the whole 10^6 prefix, none reaches exponent 4; max exponent 3.6094 (`p = 233`) | strong: exhaustive on the prefix (and 3.6094 ≈ 2+φ = 3.618, the known critical exponent) |
| trib 4th power | FALSE | same; max exponent 3.1861 (`p = 274`) | strong (≈ the known 3.19) |
| fib palindrome of every length | TRUE | Manacher on 2*10^5: every `n ≤ 121 391` occurs | corroborated |
| trib palindrome of every length | TRUE | every `n ≤ 133 922` occurs | corroborated |

9/9 agree with the engine and, where Walnut finishes, with Walnut. The exhaustive
4th-power search uses a sound stride-`⌈p/2⌉` sampling prefilter (a run of `3p` matches
cannot hide from it), so no period is skipped.

`explore/numsys_check.py` does **not** cover the two palindrome rows, contrary to
`bench/fib.md`'s "Every TRUE/FALSE above is independently confirmed by brute force …
(`explore/numsys_check.py`)". They are true; the claim of coverage was not.

## 4. Walnut re-run (`check6_walnut_fib.py`, `check8_walnut_trib.py`)

Same jar, same heap, same formulas as `bench/fib_bench.py`.

    Fibonacci            bench (states/peak)   mine        Tribonacci        bench       mine
    no 11                     1 / 3            1 / 3       ev. periodic      1 / 431     1 / 431
    eventually periodic       1 / 41           1 / 41      cube            OOM / 14034 OOM / 14034
    cube                      6 / 190          6 / 190     4th power       OOM / 25922 OOM / 25922
    4th power                 1 / 330          1 / 330     palindrome      OOM / 31536 OOM / 31536
    palindrome                1 / 201          1 / 201     FE direct       OOM / 10533 OOM / 10533
    FE direct                11 / 153         11 / 153

**Every Walnut number in `bench/fib.md` reproduces exactly**, including all four OOMs
(94–205 s each here; the bench's times are in the same range). Peanut likewise: all
thirteen `states`/`peak` values reproduce to the digit (`check7_peanut_bench.py`), and
`AM_CAP0=20000/50000/200000` moves the Tribonacci peak to `20001/50001/200000`, confirming
the bench's own reading that those peaks are the ladder's cap, not an intrinsic size.

**But the sentence in the bench's preamble is wrong.** "ours counts the dead state, so
ours = Walnut + 1 wherever both finish — a free independent check of every number here"
holds in exactly one of the eleven rows where both finish (FE on Fibonacci, 12 vs 11). In
the five Fibonacci sentence rows both report 1 except "a cube occurs", where Walnut
reports 6 and Peanut 1. The reason is that the two tools' `states` mean different things
for a closed sentence: Peanut minimises to the trivial automaton, while Walnut's number is
its last logged step (for the cube query, the 6-state automaton left after `E i,n`
projection, which it then evaluates to TRUE). So the "free independent check" does not
exist for the sentence rows, and the `states` columns there are not comparable.

## 5. The FE headline (`check9`, `check10`, `check12`)

Primary source, read directly: thesis §1.2 p. 3 and §5 p. 47 — Tribonacci `EqFac` "the
largest intermediate automaton has 323,831,403 states (!), while the final result has only
26 states … Execution required over 300 GB of RAM and took 432,831,386 ms over several
days." The bench's "3.2 x 10^8 states / ~300 GB" and "26" are accurate quotations. The
Thue–Morse figure quoted in the thesis (14 states) also checks out: Peanut gives 15 = 14 +
dead in base 2.

Verified here:

* Peanut's msd FE automata agree with a brute-force longest-common-prefix computation:
  the direct ones (12 fib, 27 trib) on 864 000 triples each (`i,j < 120`, `l < 60`), the
  `learnfe` ones on 1 575 000 each (`i,j < 150`, `l < 70`); the lsd ones (14 fib direct,
  14 fib learnfe, 29 trib learnfe) on 500 000 each;
* both are **minimal** complete DFAs (my own Moore partition refinement), so dropping the
  dead state gives 11 and **26**;
* Walnut's own 11-state Fibonacci FE is equivalent to Peanut's 12-state one (product BFS
  over valid triples);
* I rebuilt Walnut's Tribonacci FE with Khodier's reformulated predicate from p. 47,
  `Au,v (u>=i & u<i+n & u+j=v+i) => TR[u]=TR[v]`: **26 states, peak 18 853, 10.2 s** here
  (thesis: 18 853, 16 307 ms — reproduced). It is equivalent to Peanut's 27-state
  automaton, and to brute force.

That last row is the one fairness datum the benchmark is missing. `bench/fib.md` says
"Khodier's self-verifying predicates and Walnut's `reverse`/`split` commands would rescue
some of these rows" but does not measure it, and the reading section states the comparison
as `OOM` vs `70 ms`. Measured, the honest comparison for the Tribonacci FE is:

    Walnut, naive EqFac, 6 GB      OOM after 205 s        peak ≥ 10 533
    Walnut, thesis reformulation   26 states, 10.2 s      peak 18 853
    Peanut, ladder                 27 states, 3.1 s       peak 50 007  (= AM_CAP0 + 7)
    Peanut, learnfe                27 states, 0.070 s     peak 346

`learnfe` still wins by ~145x in time and ~54x in peak against the best published Walnut
formulation, and it needs no reformulation — which is the actual claim worth making. The
"seven orders of magnitude below the documented blow-up" line is fair only against the
naive predicate (346 vs 3.2 x 10^8), and should say so.

## 6. Soundness of the compiler under a numeration system (`check11_semantics.py`)

Exhaustive over *all* digit words, not samples: for `i+1=j` and `i<j => T[i]=T[j]` (fib,
2 tracks, `|w| ≤ 8`), `~(T[i]=1)` (fib, `|w| ≤ 12`) and `i+j=k` (pell, 3 tracks, `|w| ≤ 5`),
the compiled DFA accepts a word **iff** every track is a valid representation *and* the
relation holds of the values — 15 083 741 words, 0 discrepancies. That covers the `=>` and
`~` paths, i.e. exactly the places where `dfa.rs` re-restricts validity, and is the check
that the "restrict only after complement and after `op(false,false)=true` products"
optimisation is safe.

## 7. Defects (`check13_defects.py` reproduces all of them)

**D1 (medium; robustness + a false doc claim).** `def` is not refused under a numeration
system. `numsys fib` + `def T 2 2 0 01 10 01` is accepted and `seq 10` prints `0111010010`
— *not* Thue–Morse (`0110100110`), but the Thue–Morse automaton read over Zeckendorf
digits. That object is well defined (prolongability forces `delta(q0,0)=q0`, so leading
zeros are harmless) and nothing unsound follows, but `docs/NUMERATION.md` §2 says `def` "is
likewise a base-k notion" alongside a `pow()` refusal, and `docs/COMMANDS.md` says
switching system clears the sequence "since a base-`k` sequence means nothing under
Zeckendorf digits". Worse, unlike `dfao` (`ERR dfao numeration system fib has 2 digits, got
3`), `def` never compares `k` with the system's digit count:

    numsys pell
    def T 2 2 0 01 10 01
    seq 12
    -> thread 'main' panicked at src/dfao.rs:69:62: index out of bounds: the len is 4 but the index is 4
    (process abort, rc = -6; same for `?`, `enum`, any command that touches the DFAO)

Given commit `63ad97e` ("convert reachable panics to ERR"), this is a reachable panic that
should be an `ERR`. Fix: in `def`, refuse when a numeration system is active (as the docs
claim) or at minimum when `k != numsys.digits`.

**D2 (low; a verification claim that does not verify).** The bench's "ours = Walnut + 1
wherever both finish — a free independent check of every number here". See §4. Suggested
replacement: state it for the FE rows only, and note that for closed sentences the two
`states` columns measure different things.

**D3 (low).** `bench/fib.md`: "Every TRUE/FALSE above is independently confirmed by brute
force on a 10^6 prefix (`explore/numsys_check.py`)" — the two palindrome rows are absent
from that script. They are true (§3); add them, or qualify the sentence.

**D4 (cosmetic).** `docs/NUMERATION.md` §4 shows `dfao WF @".../Word Automata Library/F.txt"`.
Quotes are not stripped, so that exact line fails with `No such file or directory`. The
unquoted form (spaces and all — the rest of the line is the path) works.

**D5 (low; a false claim about the oracle).** `docs/NUMERATION.md` §2: "the lsd forms are
the msd forms put through `Dfa::reverse_determinize`, so `mode lsd` costs nothing extra and
remains the independent oracle it is for base `k`." True for load; false for use. At the
bench's own 6 GB ceiling, `mode lsd` on Tribonacci exceeds the budget on the cube, 4th
power and palindrome sentences and on FE-direct — all of which msd answers in ≤ 4.3 s —
so the lsd cross-check that exists for base `k` and for Fibonacci (all verdicts agree,
FE = 14 states, verified correct here) is not available for the Tribonacci rows. `learnfe`
does survive lsd (29 states, verified correct). This is a performance fact, not a
correctness one, but the sentence should be weakened.

**D6 (trivial).** "Walnut's own `Custom Bases` files … give the numbers above unchanged":
one number changes, the Fibonacci `no 11` peak, 50 → 41, because Walnut's `fib` adder has
7 states to our 16. Verdicts and final sizes are unchanged in all 13 rows.

Documented gaps re-tested and confirmed exactly as the ledger describes: `pic` under
`numsys fib` disagrees with `fe_map` ground truth (row 0: `10101000` vs `10110101`);
`dfao` of a negative-output Walnut automaton is refused (`ERR dfao output -1 out of
range`, under `numsys pell` where `X3.txt` belongs); `enum` without a sequence is `ERR no
sequence`; `msd_neg_fib`, `msd_tib`, `msd_ns` are refused with the quoted messages, and
`msd_pisot4`, `msd_kim`, `msd_nara` load and pass the self-check.

## 8. Ledger

**Known (prior art, correctly attributed):** the abstract-numeration-system / rank
definition (Lecomte–Rigo); Walnut's custom-base file format and its `msd_*` automata
(Mousavi 2016, Nicol 2025); the Dumont–Thomas construction; Khodier's `EqFac` blow-up
figures (26 states, 323 831 403, 300 GB) and his reformulated predicate; the critical
exponents of the Fibonacci and Tribonacci words (matched to 4 digits by my brute force).

**New here (and verified):** a generated, machine-checked adder for an arbitrary addable
system (language-equivalent to Walnut's, product-checked); values as ranks, so no weight
sequence is ever typed in; direct single-word constants; the "restrict validity only after
`complement` and after `op(false,false)` products" optimisation, exhaustively sound on
15.1 x 10^6 words; `learnfe` under a numeration system, giving the minimal Tribonacci FE
(26 + dead) in 70 ms with a 346-state peak — 145x faster and 54x smaller in peak than the
best published Walnut formulation, and the only route that needs no expert reformulation.

**Failed / not established:** "ours = Walnut + 1 wherever both finish" (false, D2); "`mode
lsd` … remains the independent oracle" (false on Tribonacci, D5); "every TRUE/FALSE is
confirmed by `numsys_check.py`" (two rows missing, D3); "Walnut's own base files give the
numbers unchanged" (one peak differs, D6); `def` refusal under a numeration system (does
not exist, and panics, D1). The four Tribonacci OOM rows are *not* evidence that Walnut
cannot do those queries — only that its default msd forward construction cannot at 6 GB;
the bench says so in its caveats, and the FE row now has a measured counter-example (§5).

**Machine-verified (this review):** 200 000 rep/value round trips x 3 systems; all valid
words to length 12 x 3 systems; 1 200 000 random adder runs across 6 automata plus 1 238 400 exhaustive ones; 6 product-BFS
equivalences; 2 x 10^6 word terms; 9 verdicts on 10^6 prefixes with an exhaustive
period search to 250 000; 5 348 000 FE triples across nine automata (seven Peanut, two Walnut) in the shipped
scripts, plus 3 150 000 more at `i,j < 150`, `l < 70` for the two msd `learnfe` automata;
2 minimality proofs;
15 083 741 exhaustive semantics words; 11 Walnut runs; every Peanut row of `bench/fib.md` re-run in both digit orders.

## 9. Files

* `paper/verdict-numeration/refcore.py` — my converter, words, Walnut-format parser
* `check1_validity.py` `check2_adder.py` `check3_equiv_walnut.py` — §1
* `check4_words.py` — §2
* `check5_verdicts.py` — §3
* `check6_walnut_fib.py` `check8_walnut_trib.py` `check7_peanut_bench.py` — §4
* `check0_export_fe.py` `check9_fe_bruteforce.py` `check10_fe_vs_walnut.py`
  `check12_fe_trib_vs_walnut.py` — §5 (`walnut_fib_fe_11states.txt`,
  `walnut_trib_fe_26states.txt` are Walnut's own outputs, kept for reproducibility)
* `check11_semantics.py` — §6
* `check13_defects.py` — §7
