# TARGET 1 — final uncensored table, random ensemble

Merge of every pass over the 2026-08-16/17 random-morphism sweep (k in {2,3}, m in
2..7, up to 40 admissible sequences per cell): `results/blowup.json` (msd first pass),
`results/blowup_retry.json` (lsd rescue of every msd FAIL), `results/blowup_residue.json`
(Khodier's FE2 reformulation, msd+lsd, on the 27 double-failures), `results/blowup_residue2.json`
(msd-only FE2/FE second pass — **did not finish**, recovered from its .log: 18/25 rows),
`results/blowup_residue3.json` (msd-only FE, forced small-AM_CAP ladder — **did not
finish**, recovered from its .log: 22/25 rows; stalled >4.5h on the last 3, RAM-starved
by unrelated concurrent processes on the machine, not by the query itself — task budget
of 90 min exceeded, proceeding with what's there per instructions).

Built by `explore/final_table.py`. Raw merged data: `results/final_table.json`.

For each sequence, one number is kept: its **msd** `|FE|`, preferred over lsd because
msd/lsd automaton sizes for the same predicate genuinely differ (Finding 1 in
`docs/TARGET1.md`). `FE2` is Khodier's alternative formula for the *same* predicate
(`FE2(i,j,n) := A u,v. (u>=i & u<i+n & u+j=v+i) => T[u]=T[v]`, equal to `FE(i,j,l)` under
`v=t+j, l=n`), so an `FE2_msd` success counts as a bona fide msd `|FE|`. A sequence with
no msd value from any formula/cap but an lsd value is **lsd-only** (listed separately,
not in the quartiles below — not comparable). A sequence with no value anywhere is
**still-censored**.

    msd=356  lsd-only=45  still-censored=11   (out of 412 sequences total)

## Per (k, m)

     k  m    n  cens    min     q1    med     q3    max  med/m^3  log2(max)/m
     2  2    8     0      3      6      8     10     15     0.94       1.95
     2  3   33     0      9     34     51     96    218     1.89       2.59
     2  4   24     0     24     58    161    404    549     2.52       2.28
     2  5   39     0     97    260    340    403    924     2.72       1.97
     2  6   40     1    201    474    608    966   2124     2.81       1.84
     2  7   40     0    200    685    882   1134   1770     2.57       1.54
     3  2   28     0      3      7     17     22     38     2.12       2.62
     3  3   40     0     10     50     72    100    190     2.69       2.52
     3  4   40     0    135    216    250    348    532     3.91       2.26
     3  5   40     0    112    328    424    509   1604     3.40       2.13
     3  6   40     0    311    540    752    872   2356     3.48       1.87
     3  7   40    10    521    722    847   1085   3480     2.47       1.68

`n` = sequences dispatched in that cell (union of first-pass successes and failures).
`cens` = still-censored count. `min/q1/med/q3/max` over the msd-valued sequences only.
All 11 still-censored sequences sit in `k=2,m=6` (1) and `k=3,m=7` (10) — exactly the
tail flagged in `docs/TARGET1.md`, now smaller (11 vs the original 27) but not empty:
`k=3,m=7` remains the hardest cell in the ensemble by a wide margin (only 30/40 resolved
to msd).

## lsd-only sequences (45)

No msd value in any formula/cap; value shown is `|FE|` in **lsd**, not directly
comparable to the table above. All in cells `k=2,m=5` (9), `k=2,m=6` (1), `k=3,m=3`
(15), `k=3,m=4` (18), `k=3,m=5` (2) — see `results/final_table.json` for the full list
with def lines (also printed by `explore/final_table.py`). Notably `def T 2 5 0 01 43 30
33 24 10010` (the old sweep champion, `|FE|_lsd=3067`) is lsd-only: its msd construction
has never completed in any pass.

## Still-censored sequences (11)

No value in any mode, formula, or cap, after 5 rescue passes:

    k=2 m=6  def T 2 6 0 05 23 44 42 51 10 000010
    k=3 m=7  def T 3 7 0 001 036 664 412 153 131 230 1101101
    k=3 m=7  def T 3 7 0 004 334 114 653 155 301 245 0011011
    k=3 m=7  def T 3 7 0 013 622 453 124 333 203 521 0011000
    k=3 m=7  def T 3 7 0 020 412 341 404 625 512 153 0101111
    k=3 m=7  def T 3 7 0 031 525 352 166 266 240 645 1011010
    k=3 m=7  def T 3 7 0 034 552 341 662 310 243 154 0111101
    k=3 m=7  def T 3 7 0 055 342 461 416 216 014 625 0001000
    k=3 m=7  def T 3 7 0 056 343 462 501 220 250 010 1010110
    k=3 m=7  def T 3 7 0 065 435 536 164 340 214 656 0110101
    k=3 m=7  def T 3 7 0 065 630 536 161 341 241 461 0011010

Three of these (`020 412...`, `031 525...`, `034 552...`) are specifically the ones
`blowup_residue3.py` never got to finish/retry due to machine RAM contention from
unrelated processes, not a confirmed hard failure at the forced-small-cap ladder — they
remain worth another attempt.

## Top-10 largest |FE|

     |FE|   mode      source              def
     3480   msd       residue3.cap50000   def T 3 7 0 044 513 202 604 520 421 061 1111110
     3067   lsd-only  retry.lsd           def T 2 5 0 01 43 30 33 24 10010
     2359   msd       pass1               def T 3 7 0 063 261 501 542 663 140 516 1101111
     2356   msd       residue2.FE2        def T 3 6 0 031 121 500 252 342 440 111110
     2124   msd       pass1               def T 2 6 0 02 30 42 15 35 51 111011
     2079   msd       residue3.cap50000   def T 3 7 0 020 240 556 610 454 130 152 0010000
     1808   lsd-only  retry.lsd           def T 2 6 0 05 24 34 31 11 44 101100
     1786   msd       pass1               def T 3 6 0 032 550 502 243 304 132 010000
     1770   msd       residue2.FE2        def T 2 7 0 04 64 61 46 12 32 50 1101110
     1714   msd       pass1               def T 2 6 0 04 40 14 12 35 13 111011

The new champion, `|FE|=3480` (k=3, m=7), only surfaced because of the small-AM_CAP-then-
Brzozowski ladder in `blowup_residue3.py` — first-pass msd and every other formulation
failed on it. It surpasses the previous champion (3067, and that one is lsd-only, so not
even msd-comparable), meaning the msd record for this ensemble is now 3480.

## Fits: log(median) vs log(m) and vs m, per k

    k=2: m=[2,3,4,5,6,7]  median=[7.5, 51, 161.0, 339.5, 607.5, 882.0]
      power law:   median ~ 0.666 * m^3.81    SSE(log-log)   = 0.18
      exponential: median ~ 2.44  * 2.496^m   SSE(log-linear)= 1.26

    k=3: m=[2,3,4,5,6,7]  median=[17.0, 72.5, 250, 424.5, 752.0, 847.0]
      power law:   median ~ 2.11  * m^3.23    SSE(log-log)   = 0.21
      exponential: median ~ 6.42  * 2.168^m   SSE(log-linear)= 1.07

Power law fits an order of magnitude better than exponential in both cases (SSE ~0.2 vs
~1.1-1.3, on log-residuals), consistent with `docs/TARGET1.md`'s reading: for random
morphisms the typical minimal `|FE|` is polynomial (~ m^3.2 to m^3.8), with no bulk sign
of exponential growth through m=7. The unresolved tail (`k=3,m=7`: only 30/40 sequences
resolved, still-censored is the whole 10-sequence residue) is exactly where an
exponential family would have to live, and remains the open lead.
