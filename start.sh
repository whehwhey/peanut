#!/usr/bin/env bash
# Peanut quick start: builds the engine if needed, starts the web GUI, opens your browser.
#   ./start.sh            # default port 7373
#   ./start.sh --port 8080
# Requirements: Rust (cargo) and Python 3.9+. Nothing else — the GUI is stdlib-only.
set -euo pipefail
cd "$(dirname "$0")"
PORT=7373
while [ $# -gt 0 ]; do case "$1" in --port) PORT="$2"; shift 2;; *) shift;; esac; done

say() { printf '\033[1;35m[peanut]\033[0m %s\n' "$*"; }
need() { command -v "$1" >/dev/null 2>&1; }

if ! need cargo; then
  say "Rust is not installed. Install it with:  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  say "then re-run ./start.sh"; exit 1
fi
if ! need python3; then say "python3 is required (3.9+)."; exit 1; fi

BIN=engine/target/release/peanut
if [ ! -x "$BIN" ] || [ -n "$(find engine/src -newer "$BIN" -name '*.rs' 2>/dev/null | head -1)" ]; then
  say "building the engine (first time takes a minute)..."
  (cd engine && cargo build --release 2>&1 | grep -E "^(error|warning: unused)|Finished" || true)
  [ -x "$BIN" ] || { say "build failed — see output above"; exit 1; }
fi
say "engine: $BIN"
printf 'def T 2 2 0 01 10 01\n? E i,n. n>=1 & A t. t<n => T[i+t]=T[i+n+t]\nquit\n' | "$BIN" | tail -1 | sed 's/^/[peanut] smoke test: /'

# pick a free port if the requested one is taken
while lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; do say "port $PORT busy, trying $((PORT+1))"; PORT=$((PORT+1)); done

URL="http://127.0.0.1:$PORT"
say "starting the GUI on $URL  (Ctrl-C to stop)"
( # open the browser once the server answers
  for i in $(seq 1 60); do
    if curl -s -o /dev/null "$URL/api/health" 2>/dev/null; then
      if need open; then open "$URL"; elif need xdg-open; then xdg-open "$URL" >/dev/null 2>&1; fi
      break
    fi; sleep 0.5
  done ) &
exec python3 gui/serve.py --port "$PORT"
