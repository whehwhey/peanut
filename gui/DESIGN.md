# Peanut GUI — design

Every visual decision in `gui/static/style.css` comes from one number:

    PEANUT_DESIGN_SEED = 653658211

The seed is expanded with **splitmix64** and consumed in a fixed order. Nothing is
hand-picked afterwards; if you change a token in the stylesheet, change the draw that
produced it here too, or the seed stops meaning anything.

## The generator

```python
M = (1 << 64) - 1
class SplitMix:
    def __init__(s, seed): s.x = seed & M
    def next(s):
        s.x = (s.x + 0x9E3779B97F4A7C15) & M
        z = s.x
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & M
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & M
        return (z ^ (z >> 31)) & M
    def pick(s, k): return s.next() % k

r = SplitMix(653658211)
```

## The draws, in order

| # | draw | candidates | result |
|---|---|---|---|
| 1 | base hue | `pick(360)` | **248** — indigo |
| 2 | scheme | analogous / split-complement / triad | **1 = split-complement**, arms at 248+150 = **38** and 248+210 = **98** |
| 3 | ground | ink-on-light / light-on-deep / duotone deep | **0 = ink on light** (with a dark-mode inversion) |
| 4 | display face | Superclarendon, Didot, Optima, Futura, Iowan Old Style, American Typewriter, Copperplate, Bodoni 72 | **American Typewriter** |
| 5 | body face | Charter, Palatino, Avenir Next, Verdana, Georgia, Iowan Old Style | **Charter** |
| 6 | mono face | Menlo, Courier New, Monaco, SF Mono | **Menlo** |
| 7 | type scale | 1.200, 1.250, 1.333, 1.414, 1.500 | **1.25** |
| 8 | corner radius | 0, 2, 3, 6, 10 | **10 px** |
| 9 | layout | 0 = rail + full-bleed surface, 1 = top tabs, 2 = split, 3 = single column | **0** |
| 10 | tile shape | flat square, rounded, dot, notched | **3 = notched** |
| 11 | signature | 0 = the tape, 1 = automaton constellation, 2 = digit rain, 3 = proof ledger, 4 = morphism tree | **0 = the tape** |
| 12 | saturation | `40 + pick(45)` | **53** |

Candidate lists are restricted to typefaces present on both macOS and iOS, because the
page is served over the LAN and read on a phone as often as on a laptop, and it loads no
webfonts (no CDN, no network beyond the host).

## What the draws became

**Colour.** Hue 248 is the whole interface: paper `hsl(248 30% 96%)`, ink
`hsl(248 42% 11%)`, structure `hsl(248 53% 46%)` at the drawn saturation. The two
split-complement arms carry meaning, not decoration:

| role | hue | used for |
|---|---|---|
| structure | 248 | chrome, edges, the FE heatmap, **FALSE** |
| attention | 38 | the start state, witness brackets, painted positions, the memory gauge when hot |
| assent | 98 | **TRUE**, accepting states |
| fault | 14 | errors and budget kills only — the one hue outside the scheme, so a genuine failure never reads as part of the furniture |

**FALSE is indigo, not red.** In this tool a refutation is a theorem: "Thue–Morse is not
square-free" is as much a result as its opposite. Only an *error* — a parse failure, a
memory-budget kill — gets the fault colour.

Dark mode inverts the ground and lifts the three arms; it is a `prefers-color-scheme`
block over the same tokens, so there is exactly one palette to maintain.

**Type.** American Typewriter sets the wordmark, view titles and verdicts, and nothing
else — a slab typewriter face is right for a machine that prints proofs, and wrong for
body text. Charter carries prose. Menlo carries every formula, digit, state id and
number, which is most of the interface: in a tool about base-`k` digits, the monospace
is not a "code font", it is the subject matter. Scale 1.25 from 13 px: 13 / 16 / 20 /
25 / 32, with 10–11 px letterspaced mono for eyebrows.

**Radius 10 with a slab face** is the risk in this design, and the reason it works is the
mascot: a peanut is a rounded shell, so buttons are capsules and panels are soft
rectangles, set against type with hard slab serifs. Shell and typewriter.

**The tape (signature draw 0).** A single strip of T runs across the top of every view
and does four jobs at once: it shows the sequence, it takes clicks to select a position,
it paints the positions a one-variable predicate satisfies, and it draws witness
brackets under the tiles they cover. Each tile is a notched capsule (draw 10) — the notch
is the punched-tape reference, and it disappears below 9 px per tile, where it would only
be noise. Tile colour walks the three seed hues and their midpoints
(248, 38, 98, 268, 18, 118, 208, 78) with lightness stepping down every fourth letter, so
an eight-letter output alphabet stays legible in greyscale too.

**Structure devices.** No 01 / 02 / 03 numbering: the views are a workbench, not a
sequence. What each view header carries instead is the engine command it drives —
`seq N`, `export`, `? · let · witness · enum · finite · learnfe`, `fe_map`,
`def T k m start …`, `AM_PROGRESS=1`. That is a true statement about the view and it
teaches the command language for free.

## The mascot

`gui/static/logo.svg`, `mascot-sprite.svg` and `favicon.svg` are drawn in a separate pass
of this project. The palette they arrived with sits inside the seed's scheme without any
adjustment — the mascot's ink `#2E1D49` is hue 263 (inside the analogous band around
248), and its sparkle green `#69A234` is hue 88 against the drawn arm at 98 — so the two
were left exactly as drawn. If the sprite is ever missing, the page falls back to a
two-circle outline shell and says nothing about it; the app must not depend on art.

The mascot reacts only in the Live view, and only on a settled result: happy on TRUE,
oops on FALSE or a failed run, thinking while a job is in flight. One 500 ms bob,
suppressed under `prefers-reduced-motion`. That is the only animation in the interface.

## Quality floor

Responsive from 360 px to desktop (the rail becomes a bottom tab bar at 900 px), visible
`:focus-visible` outlines in the attention hue, `prefers-reduced-motion` respected, dark
mode complete, and no horizontal page scroll at any width — the last one is checked by
`gui/static/selftest.js` on every run, at both the real viewport and a forced 360 px root.
