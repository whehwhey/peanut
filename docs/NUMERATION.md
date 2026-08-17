# NUMERATION — addable numeration systems (Fibonacci, Tribonacci, Pell, ...)

Peanut decides first-order sentences over `<N, +, <, T>` where positions are written
in a **numeration system**, not necessarily base `k`. Base `k` stays the built-in
fast path; anything else is four automata loaded from a file.

This note records (1) what Walnut hardcodes and what it loads, (2) what we do
instead, (3) the file format, (4) what has been machine-checked.

## 1. How Walnut does it

`walnut7/src/main/java/Automata/NumberSystem.java` (Mousavi 2016, Nicol 2025). A
number system is a name like `msd_fib`, split into `msd|lsd` + a base token, plus
four automata:

| object | where it comes from |
|---|---|
| addition `(x,y,z) : x+y=z` | `Custom Bases/<name>_addition.txt`, else the built-in base-n / base-(-n) adder (2–3 states) |
| comparison `(x,y) : x<y` | `Custom Bases/<name>_less_than.txt`, else **lexicographic** msd order (`lexicographicLessThan`) |
| equality | always digit-string equality, built in |
| valid representations | `Custom Bases/<name>.txt`; if the file is absent the flag `flagUseAllRepresentations` goes false and nothing is restricted |

Three things are hardcoded and everything else is data:

* **base n and base −n** adders/comparators are generated in Java
  (`baseNadditionAutomaton`, `baseNegNAddition`, `baseNegNLessThan`).
* **lsd is msd reversed**: `loadAutomatonOrNull` falls back to the complementary
  file and calls `AutomatonLogicalOps.reverse`.
* **constants**: `constant(n)` is built by binary recursion
  `n = floor(n/2) + ceil(n/2)` through the adder, with two quantifications per
  level, memoised in `constantsDynamicTable`. `multiplication(n)`/`division(n)`
  likewise.

The load-bearing invariant is validity restriction. `Automaton.applyAllRepresentations()`
conjoins the validity automaton onto **every arithmetic track**, and Walnut calls it
after every boolean operation (`AutomatonLogicalOps.totalizeCrossProduct`, i.e. all of
`&`, `|`, `=>`, `<=>`), after every `not()`, and after `setAlphabet`, `rightQuotient`,
`Star`, `Concat`. Quantification is projection followed by
`fixLeadingZerosProblem`/`fixTrailingZerosProblem` (`AutomatonQuantification.quantify`).

Two details of the text format are easy to get wrong and both bite:

* the **initial state is the first state declared in the file, not state 0** —
  `msd_trib_addition.txt` opens with `78 1` (`AutomatonReader.java`, `q0 = pair[0]`
  on the first declaration);
* a missing transition means *dead*, so the automata are partial.

Walnut has no general index → digits routine for custom bases (its `Ostrowski`
package is a separate special case), which is why custom-base word automata are
supplied as files rather than derived.

## 2. What Peanut does instead

`engine/src/numsys.rs` is the whole numeration layer; `base.rs`, `dfa.rs`, `dfao.rs`,
`logic.rs`, `learn.rs` consult it.

**A numeration system is** `{ name, digit alphabet {0..D-1}, validity DFA (msd,
leading zeros allowed), addition DFA over 3 tracks, comparison DFA (loaded, else msd
lexicographic) }`, each kept in *both* digit orders — the lsd forms are the msd forms
put through `Dfa::reverse_determinize`, so `mode lsd` costs nothing extra and remains
the independent oracle it is for base `k`.

**Values are ranks, not weighted sums.** We never assume a weight sequence. From the
validity DFA we build `cnt[q][l] = #{accepted words of length l from q}` and define
the value of a valid msd word as its rank in the radix (length, then lexicographic)
ordering of the validity language. Then

* `rep(n)` = unrank at the shortest length with `cnt[q0][L] > n`,
* `value(w)` = rank, `None` if `w` is not accepted,
* `succ(w)` = the next word of the same width, from the same table,
* `U_l = cnt[q0][l]` recovers the classical weights (1,2,3,5,8,… for Zeckendorf,
  1,2,4,7,13,… for Tribonacci, 1,2,5,12,29,… for Pell) as a *consequence*.

This is the abstract-numeration-system definition (Lecomte–Rigo). It costs no
per-system code, and it is what lets `seq`, `enum`, `witness`, `pic`, `fe_map` and —
crucially — the `learnfe` LCP oracle convert index ↔ digits in a system whose weights
nobody typed in. The oracle's counter steps `n -> n+1` by `succ`, amortised O(1), so
`learnfe` is as cheap on Zeckendorf as on base 2.

**Constants are direct.** `x = c` is the single-word recognizer of `rep(c)` padded
with `0*`: `O(|rep(c)|)` states, one automaton, no adder recursion and no
quantification (contrast Walnut's `constant()` above).

**Validity is restricted exactly where it can matter.** Every leaf — adder,
comparison, equality, constant, `T[x]=a`, `T[x]=T[y]`, and the cylindrification
`Dfa::constant(..., true)` — is conjoined with validity on each of its tracks. Given
that, an invalid word is rejected by *both* operands of every product, so the product
accepts it iff `op(false,false)` does. Hence `dfa.rs` re-restricts only after
`complement()` and after a product whose `op(false,false)` is true — that is, after
`=>` and `<=>`, but never after `&` or `|`, where Walnut restricts unconditionally.
Existential projection preserves the restriction on the surviving tracks, and
`zero_closure` preserves it because `delta(q0, 0) = q0` is checked at load time.

**`pow(t)`** (the `V_k` predicate) is base-k only and is refused with an explicit
error under a numeration system; `def` (k-uniform morphism) is likewise a base-k
notion, so a Fibonacci-automatic word enters through `dfao` (below).

## 3. Files

Walnut's "Custom Bases" text format, verbatim, so Walnut's own files drop in
unchanged (point `AM_WALNUT_BASES` at their directory, or copy them in):

```
{0, 1} {0, 1} {0, 1}      <- one alphabet per track; {0,..,D-1} only

0 1                       <- state, output (nonzero = accepting); FIRST = initial
0 0 0 -> 0                <- one digit per track, '*' = wildcard; missing = dead
0 0 1 -> 1
```

Shipped in `engine/numeration/`, generated **and machine-checked** by
`explore/gen_numsys.py` (not copied from Walnut, which is GPL):

| system | validity | addition | weights |
|---|---|---|---|
| `fib` | 2 states, no factor `11` | 16 states | 1,2,3,5,8,13,… |
| `trib` | 3 states, no factor `111` | 43 states | 1,2,4,7,13,24,… |
| `pell` | 2 states, a `2` must be followed by `0` | 12 states | 1,2,5,12,29,… |

Search path for `numsys NAME`: `$AM_NUMSYS_DIR`, `engine/numeration/` (relative to
the binary and to the cwd), then `$AM_WALNUT_BASES`; within each, `NAME.txt` then
`msd_NAME.txt`, and likewise `NAME_addition.txt` / `NAME_less_than.txt`.

**Load-time self-check.** Installing a system is refused unless, for `n <= 200`,
`value(rep(n)) == n`, and for `x,y <= 20`, the addition automaton accepts
`(x,y,x+y)`, rejects `(x,y,x+y+1)`, and the comparison automaton agrees with `<`.
That is what makes "drop Walnut's files in and go" safe: values here are *ranks*
and comparison defaults to lexicographic, so a system that is not radix-ordered
would otherwise be silently wrong everywhere. Pointing `AM_WALNUT_BASES` at a
Walnut checkout's `Custom Bases`:

    msd_fib, msd_trib, msd_pell, msd_pisot4, msd_kim, msd_nara   load and pass
    msd_neg_fib   refused: "comparison automaton disagrees with 0 < 2"  (negative base)
    msd_tib, msd_ns   refused: their addition automaton and their validity automaton
                      use a different representation convention from the greedy one
                      ("addition automaton accepts 0 + 2 = 3" / "rejects 1 + 4 = 5")

Walnut's word automata load too: `dfao WF @".../Word Automata Library/F.txt"`
gives the same Fibonacci word, and the same 12-state FE, as the hand-typed table.

The adder is *constructed*, not transcribed. Reading msd with `r` digits still to
come, the running difference `D = val(x prefix) + val(y prefix) - val(z prefix)` is
written in the basis `(U_r, …, U_{r+d-1})` as an integer vector `e`; a digit triple
with `s = a+b-c` gives

    e'_j = [j>=1] e_{j-1} + e_{d-1} * a_{d-j} + [j==0] s

from `U_{r-1+d} = a_1 U_{r-2+d} + … + a_d U_{r-1}`. Start `e = 0`, accept when
`sum_j e_j U_j = 0`, prune by `max_j |e_j| > B`, trim to reachable ∧ co-reachable,
minimise. `B = 12` and `B = 24` give the same automaton for all three systems, which
is the pruning's soundness check.

## 4. Machine-verified

`explore/gen_numsys.py` (run it; it refuses to write a file that fails):

* `value(rep(n)) == n` and `rep(n) ==` the greedy expansion, for all `n < 2*10^5`
  (and again, cheaply, at every `numsys` load — see the self-check above);
* the valid words of each length, in lexicographic order, have values `0,1,2,…` —
  i.e. radix order **is** numeric order, which is what makes the lexicographic
  comparison automaton correct;
* the adder accepts `(x,y,x+y)` and rejects `(x,y,z)` for `z != x+y`, exhaustively
  for `x,y < 400` (and `x,y<60, z<200`), plus 20 000 random pairs up to `10^9`;
* **each generated adder is language-equivalent to Walnut's own
  `msd_{fib,trib,pell}_addition.txt`**, as languages restricted to valid
  representations on all three tracks (product + reachability, not sampling).

`explore/morphic_to_dfao.py`: the Dumont–Thomas DFAO for a Pisot substitution
(`0->01, 1->0`; `0->01, 1->02, 2->0`) has `|sigma^i(a)|` equal to the numeration
weights, has domain equal to the validity language (checked as automata), and
reproduces the fixed point for all `n < 10^5`.

`explore/numsys_check.py`: the engine's `enum` for `+`, `<`, `<=`, constants,
`2*i=j`, `T[i]=1`, `T[i]=T[j]`, `T[i+1]=T[j]` matches brute force for every tuple
below 12; every TRUE/FALSE in `bench/fib.md` matches brute force on a `10^6` prefix;
and the `learnfe` FE automaton matches the direct longest-common-prefix answer for
all `i,j < 40`, `l < 20`.

## 5. Known gaps (honest ledger)

* **`pic` is not numeration-aware.** `engine/src/picture.rs` builds each cell's word
  from plain base-`k` digits (`digits`/`width_of`), so a picture drawn under
  `numsys fib` addresses the wrong cells. The fix is to route it through
  `numsys::encode_word`, as `enum`, `witness` and `learnfe` already are.
* **Negative output letters are not supported**: `Dfao` outputs are `u8`, so Walnut
  word automata with outputs like `-1` (`X3.txt`, `C_alpha.txt`) are refused with
  `ERR dfao output -1 out of range`. The formula grammar only compares against
  non-negative numerals anyway.
* **Every command still needs a sequence.** Pure arithmetic over a numeration system
  (`enum 12 i+j=k`) requires some `dfao` to have been loaded first, because `?`,
  `enum`, `dfa`, `finite`, `let` all fetch `cur` before compiling.
* **Systems we refuse**: `msd_neg_fib` (negative base -- radix order is not numeric
  order, and values here are ranks), `msd_tib` and `msd_ns` (their addition automaton
  does not agree with the greedy representations their validity automaton describes,
  under our convention that the empty word represents 0). Both are caught by the
  load-time self-check rather than producing wrong theorems.
* **The Pell word.** The Dumont-Thomas construction of `explore/morphic_to_dfao.py`
  applies to a substitution only when its admissible digit strings coincide with the
  greedy ones; for `0 -> 01, 1 -> 001` (weights 1,2,5,12,29, the Pell weights) they do
  not, and the script says so rather than emitting a wrong DFAO. Pell-automatic words
  still work when given directly as a `dfao` -- `explore/numsys_check.py` runs one
  (Walnut's `R2`) and checks 400 terms against an independent Python rank/unrank.

## 6. Adding a system

1. Write (or copy from Walnut) `NAME.txt` and `NAME_addition.txt` into
   `engine/numeration/`.
2. `numsys NAME` — the loader checks `delta(q0,0) = q0`, that the empty word is
   accepted (the representation of 0), that all tracks share one alphabet
   `{0..D-1}`, and that the adder has three tracks.
3. Load a sequence with `dfao` (see `docs/COMMANDS.md`), e.g. from
   `explore/morphic_to_dfao.py`.
4. Everything else — `?`, `let`, `witness`, `enum`, `finite`, `learnfe`, `pic`,
   `export`, both digit orders — works unchanged.
