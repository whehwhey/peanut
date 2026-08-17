# TARGET 1 — equality-of-factors automaton blowup (Khodier 2026, Open Problem 1)

## The problem, verbatim
Mazen Khodier, *New Methods for Analyzing the Properties of Automatic Sequences*, PhD
thesis, U. Waterloo, Jan 2026 (docs/khodier2026-thesis.pdf), Chapter 8:

> **Open Problem 1.** Give a characterization for the k-automatic sequences that have an
> exponential blowup in size when constructing the "equality of factors" automaton.
> Currently no such class of examples is known.
> This issue has two distinct aspects. The first concerns the size of the minimal
> automaton for the equality of factors predicate relative to the size of the morphism.
> We believe that this relationship is exponential, but currently, no class of examples
> is known that proves this behavior. The second aspect involves determining the most
> efficient strategy for constructing this automaton [...]

Upper bound stated there: |FE| <= 2^(9 m^2) for the formulation
`A u,v. (u>=i & u<i+n & u+j=v+i) => T[u]=T[v]`. Anecdote: Tribonacci FE has 26 states but
the direct Walnut construction peaked at 323,831,403 intermediate states / 300 GB.

    FE(i,j,l) := A t. t < l => T[i+t] = T[j+t]

So the target has two halves: (A) is *minimal* |FE| exponential in the DFAO size m — find
a family; (B) construct FE without the intermediate blowup.  The earlier session's
"sweep, don't solve" framing is right for (A): the reason no family is known is that the
tools die before examples accumulate.

## Sweep (explore/blowup.py, results/blowup.json)
480 admissible random k-uniform morphisms + binary coding from PRISM, k in {2,3},
m in 2..7, 40 per cell (dedup leaves fewer at m=2).  |FE| = minimal DFA states, msd.
Failures (memory budget / 90 s) re-run in lsd (explore/blowup_retry.py).

     k m    n cens   min  q1  med  q3   max   med/m^3
     2 2    8    0     3    7    8   15    15    0.94
     2 3   33    0     9   34   51   96   218    1.89
     2 4   24    0    24   61  161  420   549    2.52
     2 5   39    0    97  265  345  483  3067    2.76
     2 6   39    1   201  473  627  976  2124    2.90
     2 7   34    6   200  667  844 1078  1574    2.46
     3 2   28    0     3    7   17   22    38    2.12
     3 3   40    0    10   64  105  235   989    3.89
     3 4   40    0   135  218  330  537   903    5.16
     3 5   38    2   112  300  415  505  1604    3.32
     3 6   36    4   311  525  740  838  1786    3.42
     3 7   26   14   521  721  812 1015  2359    2.37
    (cens = failed in BOTH digit orders at 1.5-3 GB / 90-120 s; being retried)

Fit on medians, m=2..7: k=2 median ~ m^3.8 (SSE 0.20) vs 2.49^m (SSE 1.33);
k=3 median ~ m^3.1 (SSE 0.52) vs 2.07^m (SSE 1.61).  log2(max)/m FALLS with m.

**Reading.**  For random morphisms the *typical* minimal FE is polynomial, ~2.5-4 m^3,
and there is no sign of exponential growth in the bulk.  This is evidence about the
typical case only.  27/480 (5.6%) are censored, all at m>=5, and an exponential family
would live exactly in that tail -- so this is "not observed", not "does not exist".

## Finding 1: most "blowups" are digit-order artefacts
Of 47 memory-budget failures in msd, 45 finish in lsd with modest minimal size
(64-989 states for k=3, m=3/4).  Every k=3, m=3 and m=4 failure was an artefact.  The
intermediate blowup and the final size are decoupled -- exactly Khodier's Tribonacci
anecdote, reproduced at scale.  Half (B) of the problem is real; half (A) is not visible
at m<=7 in the random ensemble.

## Finding 2: what predicts |FE| within a size class (explore/blowup_features.py)
Spearman rho of within-(k,m) residual log|FE| against prefix features (n=349):
subword-complexity growth p(32)/p(16) +0.45, max run length +0.25, right-special
count +0.20, p(32) +0.19; letter frequency, recurrence gap ~0.  Eventually periodic
sequences (8) sit 1.75 nats below their class median.  So: FE is large when the
sequence has *both* rich factor structure and long runs.

## Lead: thin sets
The sweep champion (|FE|=3067, k=2, m=5) is `0->01 1->43 2->30 3->33 4->24`, coding
10010: state 3 is a SINK, so T is 1 except on the thin set S of positions whose binary
prefixes avoid state 3.  FE then compares shifted configurations of a sparse set --
distance comparisons, which is where automata multiply states.  Hand-built families
under test (explore/thin_families.py): T_c = [n has exactly c ones in base 2] (m=c+2),
"at most c ones", and the base-3 analogue.  If |FE| grows like a^c for these, that is a
candidate class for Open Problem 1(A) -- to be proved, not just measured.

## Structured families (explore/thin_families*.py, struct_families.py, gtm_full.py)
All |FE| minimal, msd unless stated.  "-" = engine failed at that budget.

    exactly c ones, base 2 (m=c+2)   msd  52  183   -  1042 2008 3463        ~c^3
                                     lsd  41  242 1133 4227 13461 38504      ratios 5.9 4.7 3.7 3.2 2.9
    at most c ones, base 2           msd  51  108   -   327  502  730        ~c^2.5
    contains 1^r in binary (m=r+1)   msd   7   91   -    -   998 1713 2700 4004   ratios ~1.5
    contains 10^(r-1)                msd  13   54   -    -   587 1026 1651   poly
    s_2(n) mod p, FULL output (m=p)  msd  15   35   -   133  224  351  520  737   ~p^2.5
                                     lsd  22   39  56    77  102  131  164  201   exactly quadratic (2nd diff const)
    s_3(n) mod p, FULL output        msd   4   37  27   137   66  355    -  741   poly, parity structure
    [s_2(n) mod p != 0], BINARY code msd  15  190 698  1877   -    -    -    -    ratios 12.7 3.7 2.7
                                     lsd  22  656 6154  ...  (running)

Verdicts. Thin/counting sets: polynomial.  Pattern-containment (sink automata,
the sweep champion's type): polynomial.  Generalised Thue-Morse with full output:
polynomial, and my "FE forces i = j mod 2^p, hence 2^p msd states" heuristic is refuted
by the numbers -- the congruence is forced globally along the whole window, so the msd
automaton never needs a sliding window.  Retract.

The one striking effect: **coding**.  Collapsing s_2 mod p from p letters to the binary
[s_2 != 0 mod p] keeps the DFAO at p states but multiplies |FE| by ~100 (lsd, p=4: 6154 vs
56).  Whatever exponential class exists, it is not "big automaton" but "lossy coding of a
group automaton": the FE automaton must track the pair (s_2(i+t), s_2(j+t)) in Z_p^2
rather than the difference in Z_p.  This is the current lead.

## Construction (aspect B): the ladder, and Walnut like-for-like
Measured across every failure of the day: a SMALL forward subset cap (50k) that fails
fast into Brzozowski reverse-first determinization beats a large forward cap by orders
of magnitude (exact-c=3: 482 states in 0.0 s vs 6 GB blown; [s2=a mod 5]: 30 MB vs
3.3 GB).  This is now the engine default (AM_CAP0 -> Brzozowski -> AM_CAP -> Brzozowski).
Walnut 8-dev at the same 6 GB fails 7 of 9 hard cases that ours answers; see bench/README.md.
Singleton family to p=8: 190 698 1877 3971 7243 11988. The $3p^4$ fit is loose, not
tight -- at p=3 it predicts 243 against the measured 190 (28% error), and the exponent is
unsettled between cubic and quartic (see paper/proof-verdict.md). The result actually
proved is $|FE|=\Theta(p^3)$ for the FULL-output GTM family $G_p$ (identity coding), not
this singleton (lossy-coding) family.
