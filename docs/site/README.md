# Peanut — in-browser WASM playground

This directory is a self-contained static site: the Peanut engine compiled to
WebAssembly plus the existing front end. It runs **entirely in the visitor's
browser** — no server, no install, no network beyond loading these files. Drop
it on GitHub Pages and anyone can drive the engine from a link.

It is the same UI as the local server GUI (`gui/static`). One code path talks to
an `EngineTransport`; when a local Python server is present the transport uses
`fetch('/api/…')`, and when the page is served statically it detects the missing
server and swaps in the wasm engine (`wasm-backend.js` → `peanut.js` →
`peanut_bg.wasm`). Nothing else in the app changes.

## Preview locally

    cd docs/site
    python3 -m http.server 8000
    # open http://127.0.0.1:8000/

(You must serve over http — opening `index.html` as a `file://` URL fails because
the browser will not `fetch` the wasm/library files from disk.)

The page detects "no server" and runs the wasm engine. To instead exercise the
*server* transport, run the real GUI: `python3 gui/serve.py`.

## The wasm blob — size, honestly

| file            | bytes    | over the wire (gzip) |
|-----------------|----------|----------------------|
| `peanut_bg.wasm`| ~910 KB  | ~290 KB              |
| `peanut.js`     | ~7 KB    | ~3 KB                |
| `app.js`        | ~85 KB   | ~20 KB               |
| everything else | ~60 KB   | —                    |

Cold load to interactive is ~80 ms on a mid laptop. GitHub Pages serves gzip
automatically, so the real download is ~300 KB — well under the 5 MB budget in
the spec. The blob is **not** run through `wasm-opt -Oz` (binaryen was not
installed in the build environment); doing so would shave perhaps another
100–150 KB off the raw size. See `bench/WASM-NOTES.md`.

## What works in the browser vs what needs the local server

**Works fully client-side (verified headless):**

- The **playground**: type a formula, run it, get TRUE/FALSE with a witness,
  `enum`, `dfa`, `finite`, `let`, `learnfe`, `export`, `seq`, `pic`, `fe_map`.
- The **sequence tape** (`seq`), position cards, paint/brackets.
- The **automaton view** (from `export`), including predicates rebuilt from their
  defining script (e.g. `FE` → 15 states for Thue–Morse, in-browser).
- The **FE heatmap** (`fe_map`).
- **Shapes / pictures** (`pic`): the agreement table, Sierpiński by carry-free
  addition, etc.
- **Turtle** and **square** renderings.
- The **morphism sandbox** preview.
- The full **library** of sequences and examples.

**Needs the local server (`gui/serve.py`) — degrades gracefully here:**

- The **live phase bar and mid-run counters**. There is no SSE/streaming in the
  static build: a "Run (streamed)" click runs the job to completion on the main
  thread and then shows the final result and verdict (the panel still settles
  into a proper DONE state). The per-phase bar and live subset/state/memory
  counters are a server feature — the browser has no worker-streamed progress.
- **Cancellation** mid-run (the run is synchronous; every library *classic* is
  sub-second, so this rarely matters).
- **Numeration systems, negbase, Ostrowski, transducer/dfao from `@file`**: these
  read files, which the wasm build has no filesystem for. Only base-k sequences
  and the inline `def`/`dfao` forms exist in the playground.

**Slow client-side (single-threaded):** the native engine uses `AM_PAR` threads;
wasm is serial (no threads in `wasm32-unknown-unknown`). Every *classical* demo
is milliseconds. The deliberately-hard library entries are not: `prism-1`
(|FE| = 467) takes ~24 s serially in-browser, and the `tail-*` cases will either
crawl or trip the in-browser memory budget (default 768 MB). That is expected and
honest — the playground is for the shareable sub-second cases; the heavy hunts
belong on the native binary.

## Enabling GitHub Pages (for Andrew, after review)

This is on branch `wasm-playground` and Pages is **not** enabled. Two ways to turn
it on:

**Option A — Deploy from a branch (simplest).**
Settings → Pages → Build and deployment → Source: *Deploy from a branch* →
Branch: `wasm-playground`, Folder: `/docs`. Save. The site appears at
`https://<user>.github.io/<repo>/site/` (all asset paths are relative, so the
`/site/` subpath is fine). Note this also publishes the rest of `docs/`, which is
already public.

**Option B — GitHub Actions (publishes only this folder at the site root).**
Settings → Pages → Source: *GitHub Actions*. Then run the included workflow
`.github/workflows/pages-wasm.yml` (Actions tab → "Deploy wasm playground" →
Run workflow). It uploads `docs/site/` as the Pages artifact, so the site is at
`https://<user>.github.io/<repo>/`. The workflow is **manual-dispatch only** — it
never runs on push, so nothing deploys until you ask.

After merging to `main`, the same `/docs` or Actions choice applies.

## Rebuilding the wasm bundle

See `bench/WASM-NOTES.md` for the exact toolchain and commands.
