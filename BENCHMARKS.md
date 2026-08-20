# Benchmarks — Peanut vs Walnut 8-dev

## Current defaults (2026-08-19)

Peanut's defaults changed on 2026-08-19: frontier-parallel bitset determinization
(`AM_PAR = min(8, cores-2)`) and antichain evaluation of closed sentences
(`AM_ANTICHAIN`) are now on. Every Peanut number below this section predates that and
is the *old* default. Re-measured, quiet machine (18-core Apple Silicon / 24 GB), one
engine process at a time, 6 GB budget on both engines, msd, `bench/defaults_bench.py`:

    equality of factors, base k          Peanut  Peanut       best Walnut 8-dev
    let FE(i,j,l) A t. t<l => T[i+t]=T[j+t]
    case      states   old default    NEW default     strategy    seconds
    prism-1      467      34.05 s        2.71 s       CCLS          90.8
    single3      190       0.010 s       0.004 s      BRZ-CCL        0.4
    single4      698       0.150 s       0.022 s      BRZ-CCL        3.8
    single5     1877       2.06 s        0.219 s      BRZ-CCL       67
    single6     3971      18.02 s        1.66 s       BRZ-CCLS     897
    tail-a      1165     140.4 s        14.49 s       CCLS          66
    tail-b      1000     169.1 s        19.41 s       CCLS         163
    tail-c      1382     no answer*    191.1 s        CCLS          10.6
    tail-c      1382      14.99 s       16.10 s   (auto-`learnfe`, now the default path)

    Tribonacci (numsys trib)
    FE(i,j,l)     27       3.08 s        0.349 s      BRZ-CCLS      62
    FE via learnfe 27       0.074 s       0.082 s     BRZ-CCLS      62

`*` the old default is killed at 6 GB on tail-c's direct construction; the new one
finishes it in 191 s / 2818 MB, at the same 1382 states the learner reports.

**tail-c is now automatic.** Since the `AM_AUTOLEARN` change (default on), an ordinary
`let FE(i,j,l) A t. t<l => T[i+t]=T[j+t]` detects the equality-of-factors shape and, when
the ladder cannot build it cheaply, hands off to the `learnfe` guess-and-verify path by
itself — the user no longer has to know the `learnfe` command to get the 16 s answer
(`docs/LEARNFE.md` §10). `AM_AUTOLEARN=0` forces the direct ladder (191 s here, and no
answer at all under the default 2048 MB budget).

**Where Walnut is still faster on this machine: tail-c.** On this 6 GB Mac measurement
Walnut's CCLS answers it in 10.6 s vs Peanut's 16.10 s (1.5x), the only row Peanut does
not win here. On the 32 GB rig the ranking flips — Walnut CCLS is 28.5 s there against the
same ~16 s learn path — so tail-c is a Peanut win by default on the rig; see the
`bench/RIG-BENCH-32GB.md` cross-check below. Both are true and both are reported; the honest
note stays that the winning path is the *learn* construction (learned, then proved
language-equal to FE), not direct determinization.

Everything else in the table Peanut now answers faster than the best Walnut strategy
for that case — 4.6x on tail-a, 8.4x on tail-b, 33x on prism-1, 540x on single6, 178x
on Tribonacci FE by the ladder and 756x by the learner. Read that with the two caveats
below intact: **Walnut answers every one of these cases too** once the right
`[strategy]` is chosen per query (John Nicol's correction, which the section below
records in full), and `learnfe` is a reimplementation of Mazen Khodier's self-verifying
predicates, which Walnut 8 is adopting. The difference this table measures is mostly
one of defaults — Peanut picks a construction per subproblem and needs no per-query
tuning — not of algorithms.

State counts still agree with Walnut's to the dead state on every row where both
finish, which is the correctness cross-check behind all of the above.

---

# 32 GB cross-check: does more RAM change who wins?  (`bench/RIG-BENCH-32GB.md`)

The head-to-head above is a 6 GB Mac. To rule out "Peanut only wins because Walnut ran
out of memory", the full equality-of-factors panel and four Tribonacci queries were rerun
on the peanut-rig (i9-14900, 64 GB) at `AM_MEM_MB=32768` and `java -Xmx32g` on both sides,
one process at a time. The complete per-case and per-strategy tables, with every
measurement, are in **[`bench/RIG-BENCH-32GB.md`](bench/RIG-BENCH-32GB.md)**.

Reading, in one line: **more RAM does not change who wins.** Walnut's `SC`/`CCL` turn
their 6 GB OOMs into 1800 s timeouts rather than answers, and its `CCLS`/`BRZ-CCLS` — the
strategies that do answer — are, if anything, *slower* on the rig than on the Mac (the
i9's weaker single-thread clock), so the extra memory buys no Walnut win on any case
Peanut also answers. Peanut's own numbers are memory-ceiling-independent (identical state
counts and near-identical peak MB at 6 GB and 32 GB).

Two caveats carried over intact from that file:

- **Walnut/Peanut cross-check: no disagreements.** Every Walnut strategy that runs to
  completion on the rig agrees with Peanut: on the equality-of-factors panel a completed
  Walnut count equals Peanut minus one (Walnut does not count the dead state — e.g. all
  completing strategies return 466 on `prism-1`, matching Peanut's 467), and on the
  closed Tribonacci sentences the TRUE/FALSE verdicts match. An earlier revision here
  flagged a `prism-1` `BRZ` = 1058 cell as a suspected Walnut bug; that was a mistake in
  our own benchmark harness, which misread an incomplete (killed-early) BRZ run's
  intermediate NFA size as a final count. BRZ on `prism-1` is a *timeout* in the 1800 s
  window and returns 466 when run to completion. There is no Walnut bug — see
  `bench/walnut-bug/ISSUE.md`.
- **tail-c honesty.** On the rig, tail-c is a Peanut win by default (~16 s auto-`learnfe`
  vs Walnut `CCLS` 28.5 s), the opposite of the 6 GB Mac row above. The winning path is
  the *learn* construction (proved language-equal to FE), not the direct determinization
  (447 s / 2818 MB on the rig, and no answer under the default budget).

---

# The 2026-08-18 per-strategy comparison (Peanut numbers = old defaults)


John Nicol (Walnut's developer) pointed out that our first benchmark compared Peanut only
against Walnut's DEFAULT determinization (plain subset construction), although Walnut 7+ ships
Brzozowski and on-the-fly (OTF) strategies via `[strategy N NAME]`. He was right; this table
re-runs every hard case under all six Walnut strategies (6 GB heap, 15-min ceiling, same machine).
Syntax and details: bench/WALNUT-STRATEGIES.md. Raw rows: bench/walnut_strategies_results.json.

## Equality of factors, base k (states / seconds; ours count the dead state, so ours = Walnut+1)

    case       Peanut         W default    BRZ           CCL        CCLS          BRZ-CCL       BRZ-CCLS      best Walnut
    prism-1    467   38.6s    OOM          466  791s     timeout    466   90.8s   466  309s     466  330s     CCLS   90.8s
    single3    190    1.5s    timeout      189  0.5s     189 3.4s   189    0.7s   189  0.4s     189  0.5s     BRZ-CCL 0.4s
    single4    698    0.2s    OOM          697  5.7s     timeout    697    4.8s   697  3.8s     697  4.0s     BRZ-CCL 3.8s
    single5   1877    2.1s    OOM         1876  90s      timeout   1876   95s    1876  67s     1876  72s      BRZ-CCL 67s
    single6   3971   20.9s    OOM          timeout       timeout    timeout      timeout      3970  897s     BRZ-CCLS 897s
    tail-a    1165  139s      OOM          OOM           timeout   1164   66s     OOM          OOM           CCLS   66s
    tail-b    1000  168s      OOM          timeout       timeout    999  163s     timeout      timeout       CCLS  163s
    tail-c    1382   14.2s    OOM          OOM           timeout   1381   10.6s   OOM          OOM           CCLS   10.6s

## Tribonacci (numsys trib), states / seconds

    query                        Peanut (ladder / learnfe)   W default   best Walnut
    cube exists                  9      <5s                  OOM         BRZ-CCLS 167s
    4th power exists             1      <5s                  OOM         BRZ-CCL  177s
    palindrome of every length   1      <5s                  OOM         BRZ-CCL   86s
    FE(i,j,l)                    27  3.1s / 0.07s            OOM         BRZ-CCLS  62s

## Fair reading

With the right strategy chosen per query, **Walnut 8-dev answers every case in this table** —
the "Walnut OOMs where Peanut answers" claim in our first benchmark was an artefact of running
Walnut's default strategy only, and is withdrawn. What remains true:

1. **No single Walnut strategy works everywhere.** CCLS is the only strategy that finishes the
   sweep tail (tail-a/b/c) but fails single6 and every Tribonacci query; BRZ variants finish the
   singleton family and Tribonacci but OOM on the tail. Walnut's own help says "you may need to
   try them all". Peanut's one default (small forward cap -> Brzozowski -> escalate) finished
   seven of the eight without a per-query choice; the eighth (tail-c) needed `learnfe`. That is
   a defaults/ergonomics difference, not an algorithmic one — the algorithms are the same
   family, and Walnut had them first.
2. **Speed against Walnut's best strategy is split on the base-k panel**: Peanut faster on
   prism-1, single4, single5, single6 (2-40x); Walnut's best faster on single3 (0.4 s vs 1.5 s),
   tail-a (66 s vs 139 s), tail-b (163 s vs 168 s) and tail-c (10.6 s vs 14.2 s). On the four
   Tribonacci queries Peanut is faster on all four (learnfe on FE: 0.07 s vs 62 s).
3. **`learnfe` (a reimplementation of Khodier's self-verifying predicates, arXiv:2507.19717 /
   Khodier 2026 thesis) is the largest single gap**: Tribonacci FE in 0.07 s / <1 MB against 62 s for
   the best Walnut strategy. This is a construction advantage, not an engine one; any prover adopting the same self-verifying-predicate method would close it.
4. Correctness cross-check stands: every state count agrees to the dead state.

Peanut's case is therefore: an independent implementation that agrees with Walnut everywhere
both finish; sane defaults that need no strategy tuning; a scriptable, memory-guarded runner
for large sweeps; and a GUI. It is not a faster algorithm than Walnut 7/8. Thanks to John Nicol
for the correction.


---

# Appendix: the original default-strategy tables (superseded by the section above)


Reproducible with `bench/walnut_fe.py` (FE panel) and the numeration harness described in docs/NUMERATION.md. Both tools at 6 GB, same machine (Apple Silicon, 26 GB). Peanut counts the dead state, so Peanut = Walnut + 1 whenever both finish — an independent cross-check of every state count.

## Equality of factors, base k

    sequence            ours  states   s  |  Walnut states     s
    thue-morse                15  0.0   |     14        0.3
    period-doubling            8  0.0   |      7        0.2
    rudin-shapiro             68  0.1   |     67        0.4
    paperfolding              44  0.0   |     43        0.2
    cantor                    17  0.0   |     16        0.2
    mephisto                  14  0.0   |     13        0.3
    prism-a                   24  0.0   |     23        0.3
    prism-d                   82  0.5   |     81        0.9
    champion-m5              199  0.0   |    198        6.1
    k3m3-artefact-b           71  0.0   |     70       18.0
    k3m3-artefact-a          216  0.0   |    215      444.5
    prism-1  (k=4,m=6)       467 38.6   |    OOM      251.5
    [s2=a mod 3]             190  1.5   |    timeout  900
    [s2=a mod 4]             698  0.2   |    OOM      313
    [s2=a mod 5]            1877  2.1   |    OOM       92
    [s2=a mod 6]            3971 20.9   |    OOM       53
    tail-a (k=2,m=7)        1165  139   |    OOM       77
    tail-b (k=3,m=5)        1000  168   |    OOM      119
    tail-c (k=2,m=6)        fail  535   |    OOM      164

Reading.  On easy inputs the two are equivalent (both instant, same automaton).  On the
FE-hard inputs -- group automata with lossy codings, PRISM-1, the sweep tail -- Walnut
8-dev at 6 GB fails on 7 of 9 and needs 7 minutes on an eighth, while ours answers 8 of
9 in 0-170 s at the same memory.  The difference is not raw speed; it is construction
strategy: reverse-first (Brzozowski) determinization with a small forward cap, and the
lsd/msd switch.  Both are things Walnut could adopt (it has `reverse`); the point is that
they are the *default* here.

Fairness caveats, stated plainly: Walnut got its default forward msd construction with no
expert reformulation (Khodier's self-verifying predicates would rescue some of these);
a bigger heap would rescue some more; and this is Walnut on the predicate its own
authors document as the pathological one.  It is a benchmark of one predicate family, not
of the tools.

## Fibonacci / Tribonacci numeration

### (from bench/fib.md)

Same sentences, same machine (M-series mac, OpenJDK 26), same 6 GB ceiling on both
sides: Walnut `java -Xmx6g -jar target/Walnut-all.jar`, Peanut `AM_MEM_MB=6144`
through `explore/engine.py`. Walnut base `?msd_fib` / `?msd_trib` with its own
`Word Automata Library/F.txt` and `TR.txt`; Peanut `numsys fib` / `numsys trib`
with the same automata typed as `dfao` (identical transition tables — see
`explore/morphic_to_dfao.py`, which derives them from the substitution and checks
them against the fixed point on `10^5` terms).

Reproduce: `python3 bench/fib_bench.py` (writes the numbers below),
`python3 explore/numsys_check.py` (brute-force cross-check of every verdict).

`states` = final automaton (ours counts the dead state, so ours = Walnut + 1
wherever both finish — a free independent check of every number here).
`peak` = largest intermediate automaton built: for us the largest subset actually
constructed anywhere in the compile; for Walnut the largest it *logged*, which is
the largest **completed** step — the determinization that kills it never prints.
`ms` = each tool's own reported computation time.

## Fibonacci (Zeckendorf), `?msd_fib`, `F = 0100101001001010010100...`

    query                                 ours                    Walnut 8-dev
                               states   peak      ms      states   peak      ms   verdict
    T[i]=1 => T[i+1]=0              1     50       0           1      3      10   TRUE  (agree)
    eventually periodic             1    102       0           1     41      15   FALSE (agree)
    a cube occurs                   1   1152       2           6    190      40   TRUE  (agree)
    a 4th power occurs              1   1942       3           1    330      53   FALSE (agree)
    palindrome of every length      1    317       2           1    201      44   TRUE  (agree)
    FE(i,j,l)  direct              12    541       1          11    153      36   (open)
    FE(i,j,l)  learnfe             12     94      35           --  no equivalent  --

## Tribonacci, `?msd_trib`, `TR = 0102010010201010201001...`

    query                                 ours                    Walnut 8-dev
                               states   peak      ms      states   peak      ms   verdict
    eventually periodic             1  84119     113           1    431      65   FALSE (agree)
    a cube occurs                   1  50001    1150         OOM  14034  101082   TRUE  (ours)
    a 4th power occurs              1  50001    4261         OOM  25922   83830   FALSE (ours)
    palindrome of every length      1  50002     654         OOM  31536   97483   TRUE  (ours)
    FE(i,j,l)  direct              27  50007    3057         OOM  10533  167069   (open)
    FE(i,j,l)  learnfe             27    346      70           --  no equivalent  --

## Reading

**Fibonacci is a draw.** Both tools answer every sentence in tens of milliseconds
and agree on every verdict; the automata are the same size (ours = Walnut + 1 for
the dead state). Nothing here needs a new construction strategy.

**Tribonacci is where it separates.** At 6 GB, Walnut 8-dev exhausts the heap on
four of the five Tribonacci queries after 1.4–2.8 minutes each; Peanut answers
all five, the slowest in 4.3 s. The difference is the same one measured for base `k`
in `bench/README.md` and `docs/TARGET1.md` — a small forward subset-construction
cap that fails fast into reverse-first (Brzozowski) determinization. Every
Tribonacci `peak` of `50001`/`50002` is literally that: the forward attempt hit
`AM_CAP0 = 50000` and the reverse pass then finished in a fraction of a second.
Walnut has `reverse`; it just does not use it here.

**The headline is FE on Tribonacci.** `FE(i,j,l) := A t. t<l => T[i+t]=T[j+t]` is
Khodier's Open Problem 1 (2026 Waterloo thesis, ch. 8): the direct construction is
documented to peak at `3.2 x 10^8` states / ~300 GB in Walnut, while the minimal
answer has **26** states. Our numbers:

    Tribonacci FE      states   peak intermediate   time      memory
    Walnut 8-dev, 6 GB    OOM   >= 10533 (logged)   167 s     out of memory
    Peanut, ladder         27   50007               3.1 s     133 MB
    Peanut, learnfe        27   346                 0.070 s   < 1 MB

27 = 26 + the dead state, i.e. **exactly Khodier's minimal automaton**, reached
in 70 ms with a largest intermediate of 346 states — seven orders of magnitude
below the documented blow-up, and without ever compiling the universally
quantified formula (`docs/LEARNFE.md`). The Fibonacci FE is 12 = 11 + 1 states,
which Walnut also reaches, in 36 ms.

**Where Walnut wins:** "eventually periodic" on Tribonacci. Walnut's intermediates
stay at 431 states; ours peaks at 84 119 (still 116 ms). Our compiler folds
`i >= N => T[i]=T[i+p]` differently and pays for it; the answer agrees.

## Fairness caveats, stated plainly

* Walnut got its default msd forward construction with no expert reformulation.
  Khodier's self-verifying predicates and Walnut's `reverse`/`split` commands would
  rescue some of these rows, as would a larger heap.
* Walnut's own `Custom Bases` files can be used directly instead of ours
  (`AM_WALNUT_BASES=".../Custom Bases"`), and Walnut's own `F.txt` / `TR.txt` word
  automata load with `dfao NAME @file`; both give the numbers above unchanged.
* Both tools are given the *same* automata for the word and for the numeration
  system: our `fib`/`trib` adders are generated by `explore/gen_numsys.py` and are
  machine-checked to be **language-equivalent to Walnut's own**
  `msd_fib_addition.txt` / `msd_trib_addition.txt` restricted to valid
  representations, so no advantage hides in the arithmetic.
* Peanut counts the dead state; Walnut does not.
* Every TRUE/FALSE above is independently confirmed by brute force on a `10^6`
  prefix of the word (`explore/numsys_check.py`), and the `learnfe` FE automaton
  is confirmed against the direct longest-common-prefix computation for all
  `i,j < 40`, `l < 20`.
