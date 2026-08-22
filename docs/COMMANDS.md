# COMMANDS - stdin command reference

Peanut's engine (`engine/target/release/peanut`, crate name `automatheus`) reads a
line-oriented script from stdin and prints one or more `OK`/`ERR`/other lines per
command. This file enumerates every command exactly as implemented in
`engine/src/main.rs`, plus the formula grammar (`engine/src/logic.rs`), the resource
env vars, and worked examples. Always launch the engine through
`explore/engine.py` (`docs/GUARD.md`, `docs/PYTHON-API.md`) - never invoke the
binary directly from a script.

Blank lines and lines starting with `#` are ignored. A script that does not end
with `quit` has one appended automatically by `explore/engine.py`.

## Session model

The engine holds three pieces of mutable state:

- `cur : Option<Dfao>` - the *current sequence*, set by `def` or `dfao`. Most
  commands need one and print `ERR no sequence` if none is set.
- `defs : Defs` - a `HashMap<String, (Vec<String>, Dfa)>` of named predicates bound
  by `let` or `learnfe`, callable from later formulas as `$NAME(args)`. **`def`
  clears `defs`** - a new sequence starts every named predicate over.
- the active **numeration system** (`numsys`), `None` for built-in base `k`.
  Switching it clears both of the above, since a base-`k` sequence means nothing
  under Zeckendorf digits.

## Commands

### `mode msd|lsd`

Sets the active digit order for every automaton built afterwards (affects
`exists`'s zero-closure, `dfa`/`enum`'s word encoding, and word length in
`witness`/`learnfe`). Global, not per-sequence.

```
> mode lsd
OK mode lsd
> mode msd
OK mode msd
```

### `numsys NAME` / `numsys off` / `numsys`

Switches the session's numeration system to `NAME`, loading its validity and
addition automata (and an optional comparison automaton) from
`engine/numeration/`. `off` (or `base`/`none`) returns to built-in base `k`; with
no argument, reports the current system. Clears the current sequence and all
`let`/`learnfe` defs. Full design note and file format: `docs/NUMERATION.md`.

Shipped systems: `fib` (Zeckendorf), `trib`, `pell`. The files are Walnut's
"Custom Bases" text format, so Walnut's own `msd_fib_addition.txt` &c. can be
dropped in unchanged (`AM_WALNUT_BASES` puts their directory on the search path).

```
> numsys fib
OK numsys fib digits=2 valid=3 add=17 lt=lexicographic weights=1,2,3,5,8,13,21,34,...
> numsys off
OK numsys base-k (built in)
```

`valid`/`add` are state counts including the dead state; `lt` is `lexicographic`
unless a `NAME_less_than.txt` was found; `weights` is `U_l` = the number of valid
words of length `l` = the value of a leading 1 followed by `l` zeros.

Loading runs a **self-check** and refuses an incoherent system: for `n <= 200`,
`value(rep(n)) == n`; for `x,y <= 20`, the adder accepts `(x,y,x+y)`, rejects
`(x,y,x+y+1)`, and the comparison automaton agrees with `<`. Walnut's own
`msd_fib`, `msd_trib`, `msd_pell`, `msd_pisot4`, `msd_kim` and `msd_nara` pass;
`msd_neg_fib` is refused (a negative base is not radix-ordered, and values here
are ranks), as are `msd_tib` and `msd_ns` (different representation convention).

Under a numeration system every automaton the compiler builds is conjoined with
"each track is a valid representation", and `pow(t)` is refused (the `V_k`
predicate is a base-`k` notion):

```
> ? pow(i) & i<10
ERR pow() is base-k only; no V_k predicate is defined for numeration system "fib" ...
```

**Known gap - arithmetic-only queries still need a current sequence.** `?`,
`witness`, `enum` and `finite` all require a current sequence (`ERR no sequence`)
even when the formula names no sequence term, so a purely arithmetic fact under a
numeration system errors:

```
> numsys fib
> ? A x,y. x+y=y+x
ERR no sequence
```

Workaround: define any throwaway sequence first (`dfao D 2 0:0,1 1:0,-`) - it is
never referenced, so it does not affect the verdict. A future release should let
sequence-free formulas compile without a DFAO.

Failure: `ERR numsys no validity automaton for "xyz" (looked for ...)`;
`ERR numsys <file>: <parse error>`.

### `dfao NAME D o0:t0,t1,.. o1:.. ..`  /  `dfao NAME @file`

Loads an **explicit** DFA-with-output over a `D`-letter digit alphabet as the
current sequence: state `q` is given as `output:target,target,…` (one target per
digit, `-` meaning dead), and state 0 is the start state. This is how a sequence
that is not the fixed point of a `k`-uniform morphism enters the engine - every
Fibonacci-, Tribonacci- or Pell-automatic word - and it is a convenient way to
type any automatic sequence directly. `D` must match the active numeration
system's digit count. Clears all `let`/`learnfe` defs.

`@file` reads a Walnut word-automaton file instead (`Word Automata Library/F.txt`
loads as-is, number-system header line included).

```
> numsys fib
> dfao F 2 0:0,1 1:0,-
OK dfao F k=2 states=3 lsd_states=4 ns=fib mode=msd
> seq 20
SEQ n=20 k=2 01001010010010100101
```

`explore/morphic_to_dfao.py` prints the `dfao` line for a Pisot substitution
(Fibonacci `0->01, 1->0`; Tribonacci `0->01, 1->02, 2->0`) by the Dumont–Thomas
construction, after checking it against the substitution's fixed point on `10^5`
terms.

Failure: `ERR dfao usage: ...`; `ERR dfao state q: N transitions, expected D`;
`ERR dfao numeration system fib has 2 digits, got 3`;
`ERR dfao state 0 must loop on digit 0 (leading zeros must not change the value)`.

### `def NAME k m start w0 .. w_{m-1} coding`

Defines the current sequence `T` as the output of a k-uniform morphism on an
m-letter alphabet, applied to `start`, read through `coding`. `wA` is the length-k
image of letter `A` (digit string, each digit `< m`); `coding` is a length-m digit
string mapping each letter to its output symbol. The morphism must be prolongable
at the start letter (`w_start[0] == start`). Clears all `let`/`learnfe` defs.

```
> def T 2 2 0 01 10 01
OK def T k=2 states=2 lsd_states=2 mode=msd
```

Failure modes: `usage: def name k m start w0..w_{m-1} coding`; `bad k`/`bad m`/
`bad start`; `expected N words + coding, got M`; `word A has length L, expected k`;
`word A has a letter >= m`; `coding length != m`; `not prolongable at start letter`.

### `seq [n]`

Prints the first `n` terms (default 60) of the current sequence as a digit string.

```
> seq 20
SEQ n=20 k=2 01101001100101101001
```

### `export NAME`

Dumps the automaton for `NAME` as one line of JSON, for the Peanut GUI. `NAME` is
either `T`/the sequence's own `def` name (the DFAO) or any predicate bound by
`let`/`learnfe`. Format (see `engine/src/export.rs`):

- **`kind:"dfa"`** (a `let`/`learnfe` predicate): `name`, `k`, `mode`, `vars`
  (formula's free variables), `params` (the bound parameter list), `alpha`
  (`k^tracks`), `nstates`, `shown` (states actually emitted), `truncated`,
  `initial` (always 0), `accepting` (state indices), `labels` (per-symbol digit
  tuple, one per track, in the automaton's sorted variable order), `trans`
  (state -> symbol -> state; `-1` for a target beyond the truncation cap).
- **`kind:"dfao"`** (the sequence itself): `name`, `k`, `mode`, `nstates`, `shown`,
  `truncated`, `initial`, `out` (per-state output letter), `trans`
  (state -> digit -> state), plus an `lsd` sub-object with the same shape for the
  lsd-order automaton of the same sequence.

Large automata are truncated to `AM_EXPORT_MAX` states (default 4000); truncated
out-of-range transitions are written as `-1` and `truncated:true` is set.

```
> export T
EXPORT {"kind":"dfao","name":"T","k":2,"mode":"msd","ns":"base","nstates":2,"shown":2,"truncated":false,"initial":0,"out":[0,1],"trans":[[0,1],[1,0]],"lsd":{"nstates":2,"shown":2,"out":[0,1],"trans":[[0,1],[1,0]]}}
```

Failure: `ERR no sequence`; `ERR export: no such predicate "NAME" (have: T, ...)`.

### `fe_map i0 j0 size L`

A `size x size` grid of `FE(i,j,L)` for `i in [i0, i0+size)`, `j in [j0, j0+size)`,
computed by a *direct* LCP walk through the DFAO (`learn::Oracle::fe`) - no
automaton is built. This is the ground truth an `FE`/`learnfe` automaton is
supposed to encode, and is what the GUI's heatmap draws for comparison. `size` is
capped at 512. Uses `AM_LEARN_LCP` (or `L+1` if larger) as the oracle's step cap.

```
> fe_map 0 0 4 3
FEMAP i0=0 j0=0 size=4 l=3 ms=0 rows=1000,0100,0010,0001
```

Failure: `ERR no sequence`; `ERR usage: fe_map i0 j0 size L` (fewer than 4
parseable numeric args).

### `pic NAME W H [i0 j0 [scale]]`

A rectangle of the `(i, j)` plane as one line. `NAME` is either a predicate bound by
`let`/`learnfe` with **exactly two free variables**, or `T`/the sequence's own `def`
name, in which case the cell is the *output letter* of the addition table `T[i+j]`
rather than a truth value.

Geometry: `W` is the width in cells (the `j` axis, across), `H` the height (the `i`
axis, down), `scale` the step in both (default 1). Cell `(r, c)` is the point
`i = i0 + r*scale`, `j = j0 + c*scale`, and rows are printed `i`-major - `H` rows of
`W` cells, the same orientation as `fe_map`. `W*H` is capped at `2^20`.

Nothing is constructed: each cell is one run of the already-compiled DFA on the
two-track base-`k` digit string of the pair in the active digit order (for `T`, one
walk of the DFAO), so the cost is `W*H*digits` transitions and no extra memory. The
picture's axes follow the predicate's *declared parameter order*, not the automaton's
sorted variable order - `pic P` is always `P(i, j)` with the first parameter down.

Each row is either `W` hex digits, or - when that is shorter - a run-length form
`~<hex><count>.<hex><count>…`; rows are joined with `,`. `vals` is the number of
distinct cell values the picture can take (2 for a predicate).

```
> let P(i,j) T[i]=T[j]
OK let P(i,j) states=2 peak=2 ms=0
> pic P 8 8
PIC 8 8 i0=0 j0=0 scale=1 vals=2 ms=0 rows=10010110,01101001,01101001,10010110,01101001,10010110,10010110,01101001
> pic T 8 8
PIC 8 8 i0=0 j0=0 scale=1 vals=2 ms=0 rows=01101001,11010011,10100110,...
> pic P 8 8 0 0 3
PIC 8 8 i0=0 j0=0 scale=3 vals=2 ms=0 rows=~17.01,~17.01,...
```

Failure: `ERR no sequence`; `ERR usage: pic NAME W H [i0 j0 [scale]]`;
`ERR pic: W and H must be positive`; `ERR pic: N cells exceeds the cap of 1048576`;
`ERR pic: no such predicate "NAME" (have: T, ...)`;
`ERR pic: NAME has M free variables [...], need exactly 2`.

### `? formula` (alias `eval`)

Compiles `formula` (see grammar below) and evaluates it.

- **Closed formula** (no free variables): prints `TRUE` or `FALSE`, the automaton's
  state count, the peak subset-construction size (`peak`, reset per query), and
  elapsed ms.
- **Open formula**: prints `OPEN`, the free variables in sorted order, state count,
  whether the language is nonempty, elapsed ms, and up to 14 witnessing tuples
  (base up to 12) via `Dfa::enumerate`.

```
> ? A i. T[i]=T[i]
TRUE states=1 peak=1 ms=0 :: A i. T[i]=T[i]
> ? T[i]=1
OPEN vars=[i] states=2 nonempty=true ms=0 witnesses=(1) (2) (4) (7) (8) (11) (13) (14) (16) (19) (21) (22) :: T[i]=1
```

### `witness formula`

Like `?` but returns one concrete satisfying assignment instead of enumerating -
specifically the assignment decoded from the **shortest** accepted word (BFS from
the start state), or `NONE` if the language is empty. A closed formula degenerates
to `TRUE`/`FALSE`.

A closed formula reports `TRUE`/`FALSE`, not a witness tuple: witnesses are the
assignment to the *free* variables, so quantify nothing you want reported. To get
a position back, leave the index free (`witness T[i]=1`), do not bind it
(`witness E i. T[i]=1` is closed and prints `TRUE`).

```
> witness T[i]=1
WITNESS i=1 states=2 len=1 ms=0 :: T[i]=1
> witness E i. T[i]=1
TRUE states=1 ms=0 :: E i. T[i]=1
> witness A i. T[i]=2
FALSE states=1 ms=0 :: A i. T[i]=2
```

### `let NAME(p1,p2,..) formula`

Compiles `formula`, checks that every named parameter appears (a parameter absent
from the body is legal and gets cylindrified in - unconstrained), errors if the
body has any variable *not* in the parameter list, and registers the resulting DFA
under `NAME` in `defs` with that parameter order. Later formulas call it as
`$NAME(a1,a2,...)`.

```
> let EQ(i,j) T[i]=T[j]
OK let EQ(i,j) states=2 peak=2 ms=0
> ? A i. $EQ(i,i)
TRUE states=1 peak=5 ms=0 :: A i. $EQ(i,i)
```

Error: `ERR $NAME body has unbound variables [...] not in the parameter list`.

**Auto-`learnfe` (`AM_AUTOLEARN`, default on).** If `formula` is *exactly* one of the
self-verifying predicate shapes the learner knows -

| shape | body (subtraction-free `let` form) |
|---|---|
| FE | `A t. t < l => T[i+t] = T[j+t]` |
| rev | `A t,u. (t<l & t+u+1=l) => T[i+t] = T[j+u]` |
| period | `A t. t+p < l => T[i+t] = T[i+t+p]` |
| border | `(b<=l) & (A t,u. (t<b & u+b=l) => T[i+t] = T[i+u+t])` |

up to renaming and reordering the parameters - then `let` first *probes* the ordinary
determinization ladder on its two cheap rungs (forward `AM_CAP0`, Brzozowski `4·AM_CAP0`).
If they succeed the ladder answer is returned unchanged (`via=ladder`); if they fail -
the hard "tail" cases - the construction is handed to `learn_pred` (`via=learnfe`), which
is *proved* language-equal to the predicate and returns the **same minimal DFA** the
ladder would, but wins the blow-up cases (tail-c: `let FE` needs 448 s / 2818 MB of
direct determinization and dies under the default budget; auto-`learnfe` answers in
~16 s / ~230 MB). A near-miss body (`t<=l`, a shifted index, `E` instead of `A`, `!=`
instead of `=`, …) is **not** a shape and is compiled by the ordinary ladder, unchanged.

```
> let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]          # thue-morse
OK let FE(i,j,l) states=15 peak=531 ms=1 via=ladder
> let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]          # tail-c (hard)
OK let FE(i,j,l) states=1382 peak=8767 ms=16058 via=learnfe kind=fe eqs=139 ces=1381 mqs=5992969
```

`AM_AUTOLEARN=0` forces the pure ladder on every `let` (no shape detection, no `via=`
suffix), which the benchmarks use to time the two paths separately. This never changes a
verdict or a minimal state count: it only chooses *how* the automaton is built. See
`docs/LEARNFE.md` §10.

### `learnfe NAME`

Builds `FE(i,j,l) := A t. t < l => T[i+t] = T[j+t]` for the current sequence by
Khodier-style guess-and-verify active learning (never compiling the direct
universally-quantified formula) and registers it under `NAME` with parameters
`(i,j,l)`, exactly as `let` would. See `docs/LEARNFE.md` for the full construction,
correctness proof and benchmark table.

```
> learnfe FE
OK learnfe FE(i,j,l) states=15 iters=2 eqs=1 ces=14 mqs=42502 steps=... peak=63 ms=34
> ? A i,j,l. $FE(i,j,l) => (l=0 | T[i]=T[j])
TRUE ...
```

Output fields: `states` (minimal DFA size), `iters` (learner rounds), `eqs`
(equivalence queries run), `ces` (counterexamples processed), `mqs` (membership
queries to the LCP oracle), `steps` (oracle work units), `peak` (max subset-
construction size seen during verification), `ms` (wall time), and - only when
nonzero - `capped_lcp=N`, the count of oracle queries that hit the `AM_LEARN_LCP`
step cap and were answered as `LCP = infinity` (harmless by the self-verification
argument; see LEARNFE.md §3). Failure: `ERR learnfe no progress ...` /
`ERR learnfe gave up after N iterations` - reported, not hidden, and the only
failure mode (correctness cannot be wrong, only non-termination).

### `learn NAME <kind>` / `learn NAME (v..) init:.. step:..`

The same guess-and-verify construction for other self-verifying predicate classes.
Registers `$NAME` like `let`, and takes the same `AM_LEARN_*` env vars as `learnfe`.
Built-in kinds:

| kind | parameters | predicate |
|---|---|---|
| `fe` | `(i,j,l)` | `A t<l. T[i+t] = T[j+t]` - identical to `learnfe` |
| `rev` | `(i,j,l)` | `A t<l. T[i+t] = T[j+l-1-t]` (so `$R(i,i,n)` = "palindrome") |
| `period` | `(i,l,p)` | `A t. t+p<l => T[i+t] = T[i+t+p]` |
| `border` | `(i,l,b)` | `b<=l & A t<b. T[i+t] = T[i+l-b+t]` |

```
> def T 2 2 0 01 10 01
> learn RV rev
OK learn RV(i,j,l) kind=rev states=31 iters=3 eqs=1 ces=30 mqs=80537 steps=69606 peak=93 ms=43
> ? A n. E i. $RV(i,i,n)                 # does Thue-Morse have a palindrome of every length?
TRUE states=1 peak=... ms=2
```

A custom class is given as its own recurrence - `learn NAME (v1,..,vn) [on:v]
init:PHI0 step:PHI1` - and the learner verifies the result against exactly that
recurrence, so a wrong recurrence is a rejected automaton, not a wrong answer.
Full syntax, the uniqueness proof for each recurrence, and the panel benchmarks
(`learn` finishes 12 panel cases where the direct `let` construction exhausts
6 GB or 150 s, and agrees with it on all 45 where both finish):
`docs/LEARN.md`. Why it is sound at all: `docs/LEARNFE.md`.

### `enum B formula`

Lists every accepted tuple with every coordinate `< B` (default 20 if `B` is
unparseable), by brute-force enumeration of all `B^n` tuples run against the
compiled DFA (not `Dfa::enumerate`'s BFS sampler - this is exhaustive up to `B`).
A closed formula (0 free variables) prints `CLOSED TRUE`/`CLOSED FALSE` instead.

```
> enum 8 T[i]=1
ENUM vars=[i] n=4 1 2 4 7
```

### `dfa formula`

Compiles `formula` and dumps its automaton: state count, digit order/base, every
state's transition row (`q<n>` or `q<n>*` if accepting, arrows indexed by symbol),
and up to 40 sample accepted tuples (max length 12) from `Dfa::enumerate`.

```
> dfa T[i]=1
DFA vars=[i] states=2 (msd base 2, padding allowed)
  q0  -> [0 1]
  q1* -> [1 0]
  members: 1 2 4 7 8 11 13 14 16 19 21 22 ...
```

### `finite formula`

For a one-free-variable formula, decides whether the set of **values** is finite.
The analysis runs on the *pad quotient* (`Dfa::pad_quotient`), not the raw
compiled automaton: a value has infinitely many padded representations (leading
zeros in msd, trailing zeros in lsd, plus any numeration-system validity slack),
so the raw automaton always contains the padding cycle and a naive cycle test
would report `INFINITE` for every nonempty set. The quotient keeps exactly one
canonical word per value - under the active numeration system as well as base-k -
after which finiteness is "no cycle on any start-to-accept path among useful
states (those that can reach acceptance)". Prints `EMPTY`, `INFINITE states=N`,
or `FINITE size=S max=M states=N` (`M` from `Dfa::enumerate(200000, 40)` over the
finite set - the intended use is proving statements like "P(n) <= c only finitely
often" by exhibiting the largest exception).

```
> finite T[i]=1 & i<10
FINITE size=5 max=8 states=7 :: T[i]=1 & i<10
```

### `mem`

Reports the counting allocator's current and peak live-byte usage (see
`docs/GUARD.md` §1 / `engine/src/membudget.rs`). Put it right after a `let`/
`learnfe`/`?` to measure that command's footprint.

```
> mem
OK mem live=12MB peak=88MB
```

### `quit` / `exit`

Ends the session (breaks the stdin read loop). `explore/engine.py` appends `quit`
automatically if a script doesn't end with it.

### Unrecognized command

```
> foo
ERR unknown command "foo"
```

### `transducer NAME @file`  /  `transducer NAME D q0:t/o,t/o,.. ..`

Loads a **finite-state transducer** -- a deterministic, 1-uniform, all-states-final
machine that rewrites a sequence letter by letter while carrying a state
(`y_n = sigma(p_n, x_n)`, `p_{n+1} = tau(p_n, x_n)`). `@file` is Walnut's
`Transducer Library` format verbatim, so `RUNSUM2.txt` &c. load unchanged; the
inline form gives one `target/output` pair per input letter of `{0..D-1}`. State
0 (the first declared) is the initial state; the machine must be total.
Transducers live in their own namespace and are not cleared by `def`/`dfao`.

```
> transducer RUNSUM2 @walnut7/Transducer Library/RUNSUM2.txt
OK transducer RUNSUM2 states=2 alphabet=[0, 1]
> transducer XOR 2 0:1/0,2/0 1:1/0,2/1 2:1/1,2/0
OK transducer XOR states=3 alphabet=[0, 1]
```

Failure: `ERR transducer <parse error>`; `ERR transducer state q has no transition
on input a (a transducer must be total)`.

### `transduce NEW TRANS SEQ`

Applies transducer `TRANS` to the current sequence (`SEQ` must be `T` or the
sequence's own name) by the Dekking (1994) construction, and makes the result the
new current sequence under the name `NEW`. Clears all `let`/`learnfe` defs, like
`def`. Works under any active numeration system: the DFAO is crossed with the
validity automaton first, so only digits the system allows are morphism edges and
only *accepting* nodes carry a letter. Full note: `docs/BREADTH.md`.

```
> def T 2 2 0 01 10 01
> transduce NEWT RUNSUM2 T
OK transduce NEWT states=8 lsd_states=7 from=T via=RUNSUM2 ms=0
> seq 30
SEQ n=30 k=2 010011101110010011100100010011
```

Failure: `ERR no sequence`; `ERR transduce: no transducer "X"`;
`ERR transduce sequence letter L is outside the transducer's input alphabet [..]`.

### `negbase K`

Generates the numeration system **base -K** (digits `{0..K-1}`, place values
`(-K)^i`) and writes `msd_neg_K.txt`, `msd_neg_K_addition.txt` and
`msd_neg_K_less_than.txt` into `engine/numeration/`. `numsys neg_K` then loads it
(and generates the same automata in memory if no file is on the search path).

Every integer, negative ones included, has exactly one representation, so
**`msd_neg_K` quantifies over Z**:

```
> negbase 2
OK negbase -2 wrote .../msd_neg_2.txt .../msd_neg_2_addition.txt .../msd_neg_2_less_than.txt
> numsys neg_2
OK numsys neg_2 digits=2 valid=1 add=4 lt=loaded weights=1,-2,4,-8,16,-32,64,-128,...
> ? E x . x+1 = 0
TRUE states=1 peak=12 ms=0 :: E x . x+1 = 0
```

Caveat: `witness`/`enum`/`?` decode a tuple back to numbers through
`numsys::decode_word`, which is unsigned, so **negative witnesses are skipped**
in the displayed list. Truth values and automata are unaffected. See
`docs/BREADTH.md` §2.

### `ost NAME [preperiod] [period]`

Generates the **Ostrowski numeration system** of the quadratic irrational
`alpha = [0; preperiod, bar(period)]` -- validity and adder -- into
`engine/numeration/msd_NAME.txt` and `msd_NAME_addition.txt`, same syntax and same
continued-fraction normalisation as Walnut's `ost`. `numsys NAME` then loads it.

```
> ost ostpell [] [2]
OK ost ostpell valid=3 add=12 weights=1,2,5,12,29,70,169,408,985 wrote ...
> ost ost13 [0 3 1] [1 2]
OK ost ost13 valid=12 add=61 weights=1,3,4,7,18,25,68,93,254 wrote ...
```

`[] [1]` reproduces Zeckendorf and `[] [2]` reproduces Pell (`alpha = sqrt(2)-1`).
Both automata are language-identical to Walnut's; `docs/BREADTH.md` §3 has the
construction and the verification.

Failure: `ERR ost the period cannot be empty`; `ERR ost all continued-fraction
partial quotients must be positive`; `ERR ost Ostrowski adder: the carry cap N binds`.

### `walnut` / `walnut on` / `walnut off`

Toggles Walnut-compatibility mode: while it is on, every line is parsed and run as
a **Walnut** command (`eval`/`def`/`reg`/`morphism`/…, `Word Automata Library/`
lookups, `?msd_k`/`?msd_fib`/… number-system prefixes) instead of the native
grammar above, and results print as `WOK`/`WERR` lines. A line whose first word is
a Walnut command *and* contains `?msd_`/`?lsd_` turns the mode on automatically, so
a Walnut script can be piped in with no preamble; `quit`/`exit` end the session as
usual regardless of mode. Full design note, grammar and the differential-test
results against real Walnut: `docs/WALNUT-COMPAT.md`.

```
> walnut
OK walnut on root=/path/to/walnut
> eval sq "?msd_fib Ei,n (n>=1) & (Aj (j<n) => F[i+j]=F[i+j+n])":
WOK eval sq states=1 vars=[] verdict=TRUE ms=41
> walnut off
OK walnut off root=/path/to/walnut
```

## Formula grammar

Sentences are parsed by `engine/src/logic.rs` (`Parser`/`Ast`) over the current
sequence, named `T` inside formulas regardless of the `def` name.

**Quantifiers**: `A v1,v2,... . body` / `E v1,v2,... . body` (also spelled
`forall`/`exists`); the `.` separator is optional. Multiple comma-separated
variables are sugar for nested quantifiers of the same kind.

**Boolean connectives**, standard precedence (loosest to tightest):
`<=>`  `=>`  `|`  `&`  `~` / `!` (unary not). Parentheses group; `true`/`false`
are literals.

**Arithmetic terms** (`Lin`, linear forms over `N`): variables, non-negative
integer literals, `+`, `-` (as long as the compiled form's coefficients are
non-negative after normalization - negative terms are rejected at compile time
with `"negative index"`/`"negative argument"`/`"negative argument to pow"`),
`n*v` / `v*n` scaling, parenthesization.

**Comparisons**: `t1 REL t2` for `REL` in `= != ~= < <= > >=` (`~=` is an alias
for `!=`) between two arithmetic terms.

**Sequence terms**: `NAME[t]` where `t` is an arithmetic term and `NAME` is the
name the sequence was defined or loaded under. For `def T ...` that is `T`; for a
`dfao`-loaded sequence it is the DFAO's own name (`RS[...]`, `W[...]`), and using
`T[...]` there parses `T` as a free variable and fails with "expected relation".
One sequence name per formula. Two forms, `=`/`!=` only:
- `T[t] = a` / `T[t] != a` - compare against a numeral output letter `a`.
- `T[t1] = T[t2]` / `T[t1] != T[t2]` - compare two positions.

**`pow(t)`**: true iff the value of `t` is an exact power of `k` (the current
sequence's base), via `base::power_of_k`.

**Named-predicate calls**: `$NAME(t1,t2,...)` - invokes a `let`/`learnfe`
definition, substituting arithmetic terms for its parameters (arguments must be
non-negative; arity must match).

## Environment variables

| var | default | meaning |
|---|---|---|
| `AM_MEM_MB` | 2048 | Hard allocator budget (MB) for the whole process - `engine/src/membudget.rs`. On breach: `ERR memory budget exceeded (N MB)` to stdout, matching line to stderr, exit code 3. |
| `AM_PAR` | `min(8, cores-2)` | Worker threads for the frontier-parallel subset construction (`engine/src/det_par.rs`). **On by default since 2026-08-19.** `AM_PAR=1` restores the pre-2026-08-19 single-threaded reference path; `AM_PAR=1 AM_FAST=1` the serial flat core. Benchmarks and the correctness gate: `bench/SPEED-ROUND6.md`, `bench/DETPAR-RESULTS.md`. Set it to 1 in a harness that already runs several engines at once - each engine builds its own pool. |
| `AM_FAST` | implied by `AM_PAR` | Route determinization/minimization through the flat core without the thread pool. Only meaningful together with `AM_PAR=1`. |
| `AM_FAST_VERIFY` | unset | Build every `exists`/`minimize`/`zero_closure` both the old way and the flat way and assert the results are equal element by element (~2x slower; development gate). |
| `AM_ANTICHAIN` | on | Answer **closed** sentences by NFA emptiness / antichain universality instead of determinizing (`engine/src/antichain.rs`, `docs/ANTICHAIN.md`). **On by default since 2026-08-19**; `AM_ANTICHAIN=0` turns it off. Never fires on a formula with free variables, so `let`/`dfa`/`enum` are unaffected. Tuning: `AM_AC_CAP`, `AM_AC_WORK`, `AM_AC_SIM`, `AM_AC_DEBUG`. |
| `AM_STRATEGY` | `off` | `bdd` = always try the symbolic (MONA-style, decision-diagram) projection first; `auto` = try it only when the alphabet is large enough to pay for it; `off` = the explicit ladder. **Off by default**: on the equality-of-factors panel it is a wash, but on large product alphabets it is worth up to 18x - see `bench/BDD-RESULTS.md` and use `auto` for wide formulas. Tuning: `AM_BDD_CAP`, `AM_BDD_NODES`, `AM_BDD_MINALPHA`, `AM_BDD_PROBE`, `AM_BDD_DEBUG`. |
| `AM_LAZY_CLOSED` | unset | Answer the closed sentence left by projecting the last variable by NFA reachability rather than determinizing. Correct and gated, worth under 1 % now that the antichain handles closed sentences first; off by default. |
| `AM_AUTOLEARN` | on | Auto-`learnfe`: when a `let` body is exactly a self-verifying shape (FE/rev/period/border), probe the ladder's two cheap rungs and hand off to `learn_pred` (`via=learnfe`) if they fail (`engine/src/autolearn.rs`). Answer-identical to the pure ladder on every case the ladder finishes; wins the tail cases. `AM_AUTOLEARN=0` forces the pure ladder on every `let` (no `via=` suffix). Gate: `tools/fuzz_autolearn.py`. |
| `AM_CAP0` | 50000 | First (cheap) forward subset-construction cap tried by every `exists` - `engine/src/dfa.rs`. Also the probe cap for auto-`learnfe` (see `AM_AUTOLEARN`). |
| `AM_CAP` | 3000000 | Last-resort forward subset-construction cap, tried after Brzozowski(cap0*4) fails. |
| `AM_LEARN_LCP` | 2^22 (4194304) | Step cap on the LCP membership-oracle walk inside `learnfe`; a pair surviving the cap is treated as `LCP = infinity` (harmless, see LEARNFE.md). |
| `AM_LEARN_LCP_MAX` | 2^26 (67108864) | Ceiling `AM_LEARN_LCP` is allowed to escalate to when the learner detects a stall and retries with a larger cap. |
| `AM_LEARN_WITNESS` | 256 | Max shortest-counterexample witnesses harvested per round from `Dfa::bfs_tree`/`word_to`. |
| `AM_LEARN_DIGITS` | (see `learn.rs`) | Max digit-length used by the boundary-sampling counterexample search. |
| `AM_LEARN_SAMPLES` | (see `learn.rs`) | Number of random boundary-sampling probes run before each equivalence query. |
| `AM_LEARN_ITERS` | (see `learn.rs`) | Cap on learner rounds before giving up with `ERR learnfe gave up after N iterations`. |
| `AM_LEARN_PROBE` | (see `learn.rs`) | Size of the magnitude-preserving local-neighbourhood crawl around each counterexample. |
| `AM_LEARN_DEBUG` | unset | If set, print learner progress (stall/cap-raise events) to stderr. |
| `AM_DEBUG` | unset | If set, `logic.rs` traces each `exists`/`forall` compilation step (vars, state count) to stderr. |
| `AM_DEBUG2` | unset | If set, `dfa.rs` traces the determinization-ladder branch taken and resulting state counts to stderr. |
| `AM_PROGRESS` | unset (0) | If set to a nonempty value other than `0`, the engine emits structured progress events - one JSON object per line - to **stderr** during `?`/`eval`/`let`/`witness`/`enum`/`dfa`/`finite`/`learnfe`: `phase` (compile-stage start, e.g. forward/brzozowski/minimize/learn/verify), `subsets` (subset-construction tick, ~every 50k subsets, with live MB), `states` (an automaton just built), `mem`, `learn` (one per learner equivalence query), and `done` (end of the top-level command, with live/peak MB). stdout is untouched - every existing script's line-protocol parsing keeps working. Cost when off: one relaxed atomic load per call site, no formatting, no allocation, no syscall. |
| `AM_NUMSYS_DIR` | unset | Extra directory searched first for numeration-system files (`docs/NUMERATION.md`). |
| `AM_WALNUT_BASES` | unset | Directory searched last - point it at a Walnut checkout's `Custom Bases` to use Walnut's own adders. |
| `AM_EXPORT_MAX` | 4000 | Max states written by `export` before truncating (`truncated:true`, out-of-range transitions as `-1`). |

The Python runner (`explore/engine.py`) adds its own env vars for process-level
resource control - `AM_WORKERS`, `AM_FLOOR_MB` - documented in `docs/GUARD.md` and
`docs/PYTHON-API.md`; these govern the runner, not the engine binary.

This file tracks `engine/src/main.rs` exactly, including the GUI-support commands
(`export`, `fe_map`) and the `AM_PROGRESS` diagnostic channel. If a command or env
var you need is missing from both the source and this file, it does not exist yet:
extend `main.rs`/the relevant module and update this file in the same change (see
`CONTRIBUTING.md`).

## A full session

```
mode msd
def T 2 2 0 01 10 01
seq 16
let EQ(i,j) T[i]=T[j]
? A i. $EQ(i,i)
witness E i,j. i!=j & $EQ(i,j)
learnfe FE
? A i,j. $FE(i,j,0)
fe_map 0 0 4 3
export FE
enum 10 T[i]=1
finite T[i]=1 & i<10
mem
quit
```

```
OK mode msd
OK def T k=2 states=2 lsd_states=2 mode=msd
SEQ n=16 k=2 0110100110010110
OK let EQ(i,j) states=2 peak=2 ms=0
TRUE states=1 peak=5 ms=0 :: A i. $EQ(i,i)
TRUE states=... ms=... :: E i,j. i!=j & $EQ(i,j)
OK learnfe FE(i,j,l) states=15 iters=2 eqs=1 ces=14 mqs=42502 steps=... peak=63 ms=34
TRUE states=... peak=... ms=... :: A i,j. $FE(i,j,0)
FEMAP i0=0 j0=0 size=4 l=3 ms=0 rows=1000,0100,0010,0001
EXPORT {"kind":"dfa","name":"FE","k":2,"mode":"msd","vars":["i","j","l"],"params":["i","j","l"],...}
ENUM vars=[i] n=5 1 2 4 7 8
FINITE size=5 max=8 states=7 :: T[i]=1 & i<10
OK mem live=..MB peak=..MB
```
