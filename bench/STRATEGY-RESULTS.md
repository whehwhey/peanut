# Walnut strategies vs Peanut — the corrected comparison (2026-08-18)

> **Peanut's column is pre-2026-08-19 defaults.** The Walnut columns are unchanged and
> still current; Peanut's defaults changed on 2026-08-19 (parallel determinization on,
> antichain closed-sentence evaluation on). The re-measured head-to-head is
> `bench/SPEED-ROUND6.md`, "Final defaults".

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
3. **`learnfe` (a reimplementation of Khodier's self-verifying predicates, which Nicol is building
   into Walnut 8) is the largest single gap**: Tribonacci FE in 0.07 s / <1 MB against 62 s for
   the best Walnut strategy. That gap will close when Walnut 8 ships the same construction.
4. Correctness cross-check stands: every state count agrees to the dead state.

Peanut's case is therefore: an independent implementation that agrees with Walnut everywhere
both finish; sane defaults that need no strategy tuning; a scriptable, memory-guarded runner
for large sweeps; and a GUI. It is not a faster algorithm than Walnut 7/8. Thanks to John Nicol
for the correction.
