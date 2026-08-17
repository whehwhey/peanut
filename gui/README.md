# Peanut GUI

A local web front end for the engine. Standard library only — no build step, no CDN, no
dependencies to install.

```
python3 gui/serve.py            # http://0.0.0.0:7373, prints the LAN URL
python3 gui/serve.py --port 8080
```

It prints something like

```
  Peanut
  local   http://127.0.0.1:7373
  LAN     http://192.168.1.24:7373
  engine  /Users/andrew/maths/engine/target/release/peanut   budget 1536 MB/job
```

so you can open it on a phone on the same network. Build the engine first if it is
missing: `cd engine && cargo build --release`.

## Views

| view | what it does | engine commands |
|---|---|---|
| Sequence | the tape of T, pan/zoom/click; a position's base-`k` digits and its path through the DFAO; find every occurrence of the factor starting there | `seq`, `enum` |
| Automaton | the DFAO, and the automaton of any predicate you have built, laid out in BFS layers with hover detail | `export` |
| Playground | a script editor with a library of examples over 26 sequences; verdicts, witnesses, states, ms, peak MB | `?`, `let`, `witness`, `enum`, `dfa`, `finite`, `learnfe` |
| FE heatmap | FE(i, j, L) as a grid walked directly through the DFAO — no automaton involved | `fe_map` |
| Shapes | three pictures of the sequence: a 2-variable predicate over the (i, j) plane, the sequence as a turtle path, the sequence folded into a square | `pic`, `seq` |
| Morphism sandbox | edit a k-uniform morphism and coding, watch the fixed point regrow, roll a random admissible one, run the battery | `def`, then the battery |
| Live compute | the phase the engine is in (running / complete / skipped), subsets built, states, memory against the budget, elapsed, a stop button while it runs and a verdict chip with `Done in X s` when it finishes | any job, with `AM_PROGRESS=1` |

Results talk to each other: a `witness` draws brackets on the tape, a one-variable `enum`
paints the positions that satisfy it, `let`/`learnfe` register a predicate the Automaton
view can then draw, and a cell tapped in a Shapes picture selects `i` on the tape.

## Shapes

Three tabs, all of them pictures of things the engine already knows.

**Predicate picture** draws any two-variable predicate over a window of the plane: one
run of its compiled automaton per cell (`pic`), so a 128×128 picture is 16 384 decisions
and nothing is constructed that a `?` would not construct anyway. Drag to pan, wheel or
pinch to zoom (the step between sampled points grows, so zooming out covers more plane
without more cells), tap a cell to read `(i, j)`, its base-`k` digits and the values of
`T` there. The menu ships with the agreement table `T[i] = T[j]`, the addition table
`T[i+j]` in output colours, `FE(i, j, L)` at a fixed length (both directly and via
`learnfe`), `pow(i+j)`, and two carry-free pictures:

- **Sierpiński** — `C(i+j, i)` is odd exactly when binary addition of `i` and `j` carries
  nowhere (Kummer). "No carries" is not directly sayable in this logic, so the shape
  brings its own sequence, `T[n] = s₂(n) mod 10`, and says
  `s₂(i) + s₂(j) ≡ s₂(i+j)` as the 100-term disjunction over the possible letter pairs.
  Sound while the carry count stays below 10, i.e. for `i, j < 512`.
- **Pascal mod 3** — the same construction in base 3 over `s₃(n) mod 7`, valid for
  `i, j < 729`.

Or pick *custom* and write your own body for `P(i, j)`, with optional setup lines
(`let`, `learnfe`) run before it in the same session.

**Turtle** walks the sequence: one step forward per symbol, then a turn by that symbol's
angle. Presets are angles, not sequences — ±90° (the paperfolding sequence draws the
dragon curve), 60°/−120° (Koch-like on Thue–Morse), ±120°, ±29° — so any sequence can be
put through any rule. Draw animates with a speed control and fits the finished path to
the canvas; `prefers-reduced-motion` draws it in one frame.

**Square** lays the first `N²` terms out row-major (or boustrophedon) and colours them by
symbol, which is the quickest way to see a sequence's self-similarity.

## Safety

Every engine process is launched through `explore/engine.py` — the same runner the sweeps
use — so the GUI is inside all three guards from `docs/GUARD.md`: admission control on
free RAM before a job starts, an RSS watchdog, `AM_MEM_MB` inside the engine, and the
system LaunchAgent above everything. The per-job budget and timeout are set in the
playground; there is no path in the GUI that runs an engine outside the runner.

The server binds `0.0.0.0`, which is the point (phones), and it runs whatever formula
script it is sent. Anyone on your network can use it. Don't expose it to the internet.

## API

| endpoint | |
|---|---|
| `GET /api/library` | sequences and formula examples |
| `GET /api/health` | engine path, free RAM, budget |
| `POST /api/run` | `{script, timeout, mem_mb, cap}` → parsed result |
| `POST /api/job` | same body, returns `{job}` and runs it in the background |
| `GET /api/stream/<job>` | server-sent events: progress, stdout lines, final result |
| `POST /api/cancel/<job>` | kill it |
| `GET /api/seq?def=&n=&mode=` | first `n` symbols |
| `GET /api/export?def=&name=&pre=&mode=` | one automaton as JSON |
| `GET /api/femap?def=&i0=&j0=&size=&l=&mode=` | the FE grid |
| `GET /api/pic?def=&name=&pre=&w=&h=&i0=&j0=&scale=&mode=` | one predicate picture, rows expanded |

`def=` is always a whole `def T k m start …` line; `pre=` is the script that builds the
predicate you are asking to export.

## Checking it works

```
python3 gui/serve.py &
curl -s localhost:7373/api/health
curl -s -X POST localhost:7373/api/run \
     -d '{"script":"def T 2 2 0 01 10 01\n? ~ E i,n. n>=1 & (A t. t <= n => T[i+t] = T[i+n+t])"}'
```

End to end, in a real browser, headless:

```
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  --headless=new --disable-gpu --virtual-time-budget=200000 \
  --dump-dom 'http://localhost:7373/?selftest=1' | grep -A20 'selftest-out'
```

`gui/static/selftest.js` drives the real page — library, tape, position card, DFAO export,
an overlap-free query, a witness, an `enum` paint, the FE grid, a random morphism, a
streaming `let FE` job and a `learnfe` job (checking the live panel's DONE state and which
phases ran, were skipped, or are still pending), the predicate automaton, the three Shapes
pictures against independently computed ground truth (`T[i]=T[j]` against the tape,
Sierpiński against `i & j == 0`, the turtle's Thue–Morse bounding box), and the responsive
layout at 360 px across every view — and prints one PASS/FAIL line each. Add `&show=1` to
see the report on screen in a screenshot, `&end=<view>` to leave the app on a particular
view (`&end=shapes/picture` picks a Shapes tab too).

## Design

`gui/DESIGN.md` — every colour, typeface and measurement is derived from
`PEANUT_DESIGN_SEED = 653658211`, with the draw order written out.

---

Peanut · Andrew Hingston · MIT
