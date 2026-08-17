# Peanut GUI — test sweep

How to drive the whole GUI headlessly, what a run actually proves, and the record of the
last sweep: what was found, what was fixed.

## Running the selftest

The selftest (`gui/static/selftest.js`, loaded only when the page is opened with
`?selftest=1`) drives the real page end-to-end — no mocks, no fixtures, real engine calls
over the same API the UI uses — and writes a plain-text PASS/FAIL report into a hidden
`<pre id="selftest-out">`, so a single headless-Chrome DOM dump is enough to see whether
the GUI actually works:

```
python3 gui/serve.py &                      # if not already running on :7373

chrome="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
"$chrome" --headless=new --disable-gpu --virtual-time-budget=45000 \
          --dump-dom 'http://localhost:7373/?selftest=1' | grep -o 'PASS[^<]*\|FAIL[^<]*'
```

25 checks as of this sweep: library load, the playground library click filling the
editor, tape/position/DFAO/query/witness/enum plumbing, the FE heatmap, the morphism
sandbox roll (admissibility, then restoring Thue–Morse), a streaming job's phase bar
(`let FE` — forward only; `learnfe FE` — learn *and* verify light up, Brzozowski stays
skipped), the Live panel's DONE state and verdict chip, all three Shapes pictures
(predicate picture including an *independently computed* Sierpiński ground truth, the
turtle including an *independently computed* Thue–Morse bounding box, the square), a
360px-root layout check across all 7 views (+3 Shapes panes), a control-height check at
phone width, and "the page never scrolls sideways."

Add `&show=1` to render the report on-screen instead of hidden — useful for reading it
back out of a screenshot instead of a DOM dump.

## The screenshot sweep

Every view (`&end=<view>`, see `selftest.js`) at five widths, both edges of the phone/
tablet/desktop range plus the two commonest phones:

```
chrome --headless=new --disable-gpu --hide-scrollbars --window-size=WxH \
       --virtual-time-budget=20000 --screenshot=out.png \
       'http://localhost:7373/?selftest=1&end=<view>'
```

Views: `sequence`, `automaton`, `playground`, `femap`, `shapes/picture`, `shapes/turtle`,
`shapes/square`, `morphism`, `live`. Widths: 360×800, 390×844, 768×1024, 1280×900,
1920×1080.

### The 360/390 trap — read this before trusting a narrow screenshot

**This Chrome build's headless renderer will not lay out a real page below roughly
500–600 CSS px, no matter what `--window-size` asks for.** `--screenshot` still writes an
image of the exact requested pixel size, so a screenshot asked for at 360×800 is a
360-pixel-wide **crop of a wider page** — everything past the crop edge is simply gone
from the image, which reads exactly like a horizontal-overflow bug (text cut mid-word,
panel borders that never close, a button sliced in half) even when the real CSS is fine
at a real 360px viewport. `window.innerWidth` is silently wrong too (pinned at `756`
regardless of the requested size), so a script-based check of the viewport doesn't catch
it either — only the fact that the *painted* layout still matches whatever width was
requested (confirmed by comparing a plain colored test page's fill against the image
bounds, and by the app's own 900px rail→tabs breakpoint firing correctly at 1280 and
900 but not at anything requested below the floor).

`selftest.js` already knew this — its 360px-root check constrains
`document.documentElement.style.width` instead of trusting the viewport, with the
comment *"Headless Chrome will not lay out below ~500 CSS px."* The screenshot sweep
needs the same workaround, because a screenshot has no such fallback built in:

1. Open the page at a **floor-safe** width (900×H works for every view here).
2. Pass `&w=<px>` on top of `&end=<view>` — `selftest.js` sets
   `document.documentElement.style.width = '<px>px'` before leaving the app on that view,
   which reproduces the real media queries and flex/grid math at that width from a wider,
   real viewport.
3. Screenshot at the floor-safe window size, then crop the image to `<px>` wide with
   PIL/ImageMagick/etc. — the app renders flush against the left edge, so a crop is exact.

368×844 and 390×800 don't need this (they're above the floor) and were shot directly.

```python
# after step 2/3 above, e.g. window-size=900x1000, &w=360:
Image.open(raw).crop((0, 0, 360, 1000)).save(f'{width}x{height}_{view}.png')
```

Screenshots live in `results/gui-sweep/` (gitignore-friendly; not committed — the repo
already had a handful checked in from an earlier session, left as-is rather than
untracked mid-sweep).

## What this sweep found

Two real defects, both confirmed at a genuine 360px viewport via the crop method above
(not artifacts of the floor), both fixed:

### 1. `.view-head p` and `.grid2` children had no `min-width: 0`

**Symptom** (only visible once the floor trap above is worked around): at a true 360px
viewport, the Sequence view's intro paragraph sat *beside* the heading instead of
wrapping below it and ran off the right edge mid-word; the `SEQUENCE`/`DIGITS` header
pair didn't stack; the "Find every occurrence" button overflowed its row instead of
wrapping; several panel borders never closed. This reproduced from static HTML+CSS
alone (no JS), and identically in the Sequence, Playground, and Morphism views — every
view using `.grid2` or the shared `.view-head p`.

**Cause**: flexbox and grid items default to `min-width: auto` — they refuse to shrink
below their own content's natural minimum width, even when a `max-width` or an explicit
`flex-basis` says otherwise. `.grid2`'s track sizing was already correctly
`minmax(0, 1fr)`, but that only constrains the *track*; the *item* sitting in it (a
`.panel` containing a nowrap button row) still wouldn't shrink below its own min-content
width, so it overflowed the track instead of wrapping its contents. `.view-head`'s
`<p>` had the same problem as a flex item. The codebase already had the fix for exactly
this class of bug on `.body`'s grid (`.body > * { min-width: 0; }`, with a comment
explaining why) — `.grid2` and `.view-head p` just never got it.

**Fix** (`gui/static/style.css`):
```css
.view-head p { ...; min-width: 0; flex-basis: 100%; }
.grid2 > * { min-width: 0; }
```

Before / after at a genuine 360px viewport (Sequence view):

| before | after |
|---|---|
| header doesn't stack, paragraph beside the title runs off-screen, "Find every occurrence" overflows, a panel border never closes | header stacks, paragraph wraps under the title in its own full-width row, the button wraps to a new line inside its panel, every panel border closes within the viewport |

### 2. FE heatmap info pill went stale on sequence switch

**Symptom**: switch sequences (e.g. via the morphism sandbox's "roll", which restores
Thue–Morse when it's done) while the FE heatmap has a picture loaded, then open FE
heatmap — the canvas correctly reverts to "Press Draw to walk the grid," but the pill
next to Draw still reads the *previous* sequence's stats (`32×32 at L=3 · 172 agreeing
pairs · 0 ms`), directly contradicting the empty canvas beside it.

**Cause**: `selectSequence()` resets `Heat.data` and redraws the canvas, but never
touched `#feInfo`. The equivalent reset for the Shapes view (`Shapes.reset()`) already
clears `picInfo`/`turInfo`/`sqInfo` correctly — the FE heatmap was the one place this
convention wasn't followed.

**Fix** (`gui/static/app.js`, in `selectSequence`):
```js
Heat.data = null;
Heat.draw();
$('feInfo').textContent = '—';   // added
```

### Selftest coverage added

`playground library click fills the editor` — clicks the first `.libitem` in the
library panel and asserts the editor textarea is no longer empty. Not previously
checked; the task's explicit "library click fills the editor" requirement.

### Checked and clean

Every view at every width, plus dark-mode-default rendering (this Chrome build defaults
to dark; the design has no separate light-mode requirement to test against here):
favicon/logo present, no illegible contrast, no empty state that reads as broken (all
say something specific — "Press Draw," "No tile selected yet," "the session starts
with…"), Shapes turtle renders a non-empty path (paperfolding dragon, 512 steps, real
bounding box), Live reaches a DONE state with a colored verdict chip and `Done in Xs`,
morphism `roll` produces an admissible definition every time it's checked. The rail
correctly becomes a bottom tab bar under 900px and back to a sidebar above it, confirmed
independently of the floor issue by the breakpoint firing correctly at 1280 vs staying a
sidebar only above 900 in the crop-corrected 360/390 shots. No control was found
stretched out of shape at phone width (the `select` in a column-direction `label.field`
bug the selftest's own comment warns about did not recur).
