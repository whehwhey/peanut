# ANTICHAIN — evaluating closed sentences without determinizing the outer block

`engine/src/antichain.rs`. **On by default since 2026-08-19**
(`bench/SPEED-ROUND6.md`, "Final defaults"); `AM_ANTICHAIN=0` restores the
pre-2026-08-19 path. The measurements below were taken while it was still opt-in,
so "base" there is what `AM_ANTICHAIN=0` gives now.
Hook: five lines in `logic::compile_str`, one `mod` line in `main.rs`.

## What it changes

The default pipeline turns every quantifier into a determinization: `Dfa::exists`
subset-constructs the projected NFA, and `Dfa::forall` is
`complement().exists().complement()`. For a **closed** sentence (`?` returning
TRUE/FALSE) the outermost block does not need an automaton, only an answer, and two
classical constructions give the answer without a subset construction:

| outermost block | question | algorithm | cost |
|---|---|---|---|
| `E x1..xn. phi` | is `phi`'s language nonempty? | one BFS (`Dfa::is_nonempty`) | linear in the body automaton |
| `A x1..xm. (g => E y1..yp. psi)` | is `L(G)` contained in `L(N)`, `N` the projection of `psi` along the `y`s? | NFA **universality** by the antichain of De Wulf–Doyen–Henzinger–Raskin (CAV 2006), optionally refined by a forward simulation preorder (Abdulla–Chen–Holík–Mayr–Vojnar, TACAS 2010) | the subsumption-minimal reachable subsets only |

Everything **below** the outermost block is compiled by the ordinary compiler, so
whether this pays depends entirely on whether the outer block is where the work is.

### The rewrites that get a sentence into one of those two shapes

Polarity is threaded through the evaluator instead of being pushed into the formula,
so all of these are recognised:

```
~E x. b                  ->  not nonempty(b)
~A x. b                  ->  nonempty(~b)
A x. (g => ~z)           ->  not nonempty(g & z)        no universality question at all
A x. (g | h)             ->  A x. (~g => h)
E x. (p & E y. q)        ->  nonempty over block [x,y] of (p & q)     (existentials hoisted)
A x. (g => (E y. p & E z. q))  ->  universality of proj_{y,z}(p & q)
```

Hoisting an existential out of a conjunction is refused when it would capture a name
already bound in the block, and when the extra track would push the product's alphabet
past `AM_AC_ALPHA` (default `2^16`) — every hoisted track multiplies the alphabet by
`k`.

### Padding

A projected NFA is not yet the automaton the engine's `exists` produces: the padding
convention has to be re-established. `close_padding` does it at the NFA level, which is
where it is nearly free:

* **msd** (padding = leading zeros): `L'' = {w : exists m, 0^m w in L}` — close the
  *initial* set forward under the all-zero symbol.
* **lsd** (padding = trailing zeros): `L'' = {w : exists m, w 0^m in L}` — close the
  *accepting* set backward under the all-zero symbol.

Projecting several tracks at once and closing once is the same language as projecting
them one at a time and closing after each: both are
`{u : exists m, exists y of length |u|+m with (u 0^m, y) accepted}`.

### Guard and validity

`A x. phi` in this engine means "for every tuple of *valid representations*": `forall`
is `complement().exists().complement()` and `Dfa::complement` re-restricts to valid
words (`numsys::restrict`). The antichain therefore answers `L(G) ⊆ L(N)` with
`G` = the guard conjoined with the validity language of every kept track
(`Dfa::constant(k, keep, true)`), not `Sigma* ⊆ L(N)`. Search states are pairs
`(guard state, subset)`, and only subsets are compared for subsumption — pairs with
different guard states are incomparable.

### The simulation ladder

Simulation shrinks the antichain sharply when it matters (tail-a, `A i,n,N. E j. j>=N &
FE(i,j,n)`, a 2246-state NFA: **691 antichain elements without it, 1 with it**) but its
naive greatest-fixpoint costs more than the whole search does when the search was
already small (20 ms -> 338 ms on that same query). So it is a ladder, in the style of
the determinization ladder in `dfa.rs`:

1. plain subset-inclusion antichain, capped at `AM_AC_SIM_TRIGGER` (default 5 000) elements
2. if that cap is hit and `|Q|^2 * alpha <= AM_AC_SIMWORK` (default 5e7), compute the
   simulation and re-run with the full cap
3. otherwise re-run plain with the full cap

`AM_AC_SIM=off|auto|on` overrides the ladder.

## Flags

| flag | default | meaning |
|---|---|---|
| `AM_ANTICHAIN` | **on** | `AM_ANTICHAIN=0` disables this module; any other value or none leaves it on |
| `AM_AC_DEBUG=1` | off | one stderr line per block: shape, sizes, antichain width |
| `AM_AC_CAP` | 200 000 | antichain elements before giving up |
| `AM_AC_WORK` | 4 000 000 | subsumption tests, shared across all attempts of one block |
| `AM_AC_SIM` | `auto` | `off` / `auto` / `on` |
| `AM_AC_SIM_TRIGGER` | 5 000 | elements that trigger computing the simulation |
| `AM_AC_SIMWORK` | 8 000 000 | `\|Q\|^2 * alpha` ceiling for affording a simulation |
| `AM_AC_ALPHA` | 65 536 | working-alphabet ceiling for hoisting existentials |

## Giving up cheaply

Two different give-up paths, and the difference matters more than the antichain itself:

* **Before compiling anything** — an unrecognised shape, a shadowed binder — the module
  returns "not my shape" and `compile_str` falls through to the ordinary compiler. Cost:
  a walk over the AST.
* **After the body has been compiled**, abstaining would be a disaster: the caller
  restarts from the AST and compiles the body a *second* time, and the body is normally
  where all the time goes. Measured on prism-1 `unbordered`
  (`A n. n>=1 => E i. ~(E b,j. ...)`, whose antichain does blow past its budget):
  40.8 s by default, **87.9 s** with an abstain-and-recompile fallback. So `ev_forall`
  never abstains once it has compiled: if the antichain runs out of budget it finishes
  the job from the NFA already in hand, replaying `dfa.rs`'s determinization ladder
  (`AM_CAP0` forward -> Brzozowski -> `AM_CAP` forward -> Brzozowski) and answering by
  product reachability. Same query, same build, with that fallback: **42.2 s**, identical
  `peak=74327` and 312 MB — a 3 % overhead for the abandoned antichain attempt.

The module therefore only ever answers correctly or hands the question back before doing
real work — it never approximates.

## Which shapes benefit

**They benefit when the outermost block is where the work is.** That is a narrow but
real class, and it is exactly the class the repo's own hard cases fall into *once the
inner equality-of-factors predicate is available as an automaton* (`learnfe FE`):

* **wins**: `A i,n,N. E j. ...FE...` (uniform recurrence), `A n. E i,j. ...`,
  `A i,n. E j. ...`, and any `E`-block whose body is already built. These are the
  "border", "right-special" and "recurrence" shapes that `docs/FUZZ.md` records as the
  ones that exhaust Walnut.
* **no change**: every sentence whose cost is an *inner* quantifier. On prism-1 the
  same recurrence statement written out inline
  (`A i,n,N. E j. j>=N & (A t. t<n => T[i+t]=T[j+t])`) spends ~99 % of its time building
  `FE` under the `A t`, which this module does not touch — the outer block it does
  replace was a rounding error to begin with. Which of the two blocks dominates is a
  property of the sequence, not of the sentence: the same inline statement on single5
  goes from "exhausts 6 GB" to 2.1 s, because there the inner `A t` is small and the
  outer projection is the blowup.
* **never fires**: open formulas (`dfa`, `enum`, `witness` on a formula with free
  variables), `let` bodies, and any sentence whose outer block is not one of the shapes
  above. Their state counts are unchanged by construction and this is checked, not
  assumed (`tools/antichain_gate.py`, `library` suite).

## Measurements

See `bench/ANTICHAIN-RESULTS.md`.

## Gate

`python3 tools/antichain_gate.py` runs three suites through the engine three times —
flag off, flag on, and the pre-change binary `engine/target/release/peanut_old` — and
requires the replies to be identical once `ms=` and `peak=` are removed:

* `library` — every script of the GUI library (`gui/serve.py`) over every sequence in
  `bench/panel.json`, in msd and lsd. Includes the *open* formulas as a control.
* `fuzz` — the ten formula templates of `tools/fuzz_walnut.py` over PRISM sequences.
* `fe` — `learnfe` plus the closed FE sentences and critical-exponent ladders of
  `bench/antichain_bench.py`.
