# Peanut MCP server

An **open** [Model Context Protocol](https://modelcontextprotocol.io) server that
lets any MCP-aware agent (Claude Code, Claude Desktop, or anything else that
speaks MCP) drive the Peanut decision engine natively over stdio.

This is a community server for Peanut. It is **not** an Anthropic-official product;
it simply implements the MCP protocol so an agent can call the engine as a set of
tools instead of shelling out.

Peanut decides first-order statements over k-automatic sequences and the
Fibonacci / Tribonacci / Pell numeration systems. The engine's command language is
documented in `docs/COMMANDS.md`; the resource-guarded runner these tools wrap is
`explore/engine.py` (see `docs/PYTHON-API.md` and `docs/GUARD.md`).

## Tools

Every tool returns a structured object. Failures (timeouts, memory-budget exits,
engine `ERR` lines, parse problems) come back as fields (`ok`, `error`, ...),
never as raised exceptions.

| tool | what it does |
|---|---|
| `peanut_decide(script, timeout_s?, mem_mb?)` | Run a raw Peanut command script and return the parsed result (verdict, states, peak, ms, full stdout, per-line list, engine errors). The escape hatch for anything below plus `let`/`enum`/`dfa`/`finite`/`transduce`/Walnut mode. |
| `peanut_prove(sequence_def, sentence, timeout_s?, mem_mb?)` | Decide a **closed** sentence over a named sequence. Returns `TRUE`/`FALSE` and the decision automaton's state count. |
| `peanut_witness(sequence_def, formula, timeout_s?, mem_mb?)` | Return one satisfying assignment for an open formula (the shortest accepted word), or `NONE`. |
| `peanut_fe(sequence_def, timeout_s?, mem_mb?)` | Build the equality-of-factors automaton `FE(i,j,l)` via `learnfe` and return its minimal state count plus learner counters. |
| `peanut_sequence(sequence_def, n?, timeout_s?, mem_mb?)` | First `n` terms of a named sequence as a digit string. |
| `peanut_export(sequence_def, name?, timeout_s?, mem_mb?)` | Export an automaton (the sequence's DFAO, or a `let`/`learnfe` predicate) as parsed JSON. |

### `sequence_def`

Every tool except `peanut_decide` takes a `sequence_def`: one or more engine
commands that establish the current sequence. Inside formulas the sequence is
always named `T`. Examples:

- `def T 2 2 0 01 10 01` - Thue-Morse (base 2, the fixed point of `0->01, 1->10`).
- `def T 2 2 0 01 00 01` - the period-doubling sequence.
- `numsys fib\ndfao F 2 0:0,1 1:0,-` - the Fibonacci word (Zeckendorf digits).

To export or reason about a derived predicate, include its `let`/`learnfe` in the
`sequence_def`, e.g. `def T 2 2 0 01 10 01\nlet EQ(i,j) T[i]=T[j]`.

## Resource limits

The server always launches the engine through `explore/engine.py`, so all three
Peanut guards apply: the counting-allocator budget (`AM_MEM_MB`), the runner's
admission control + RSS watchdog, and the system memguard. Per-call `timeout_s`
and `mem_mb` are clamped to sane ceilings. Defaults and caps (env-overridable):

| env var | default | meaning |
|---|---|---|
| `PEANUT_MCP_TIMEOUT_S` | 60 | default wall-clock timeout per call |
| `PEANUT_MCP_MAX_TIMEOUT_S` | 600 | hard ceiling on `timeout_s` |
| `PEANUT_MCP_MEM_MB` | 1536 | default per-engine allocator budget |
| `PEANUT_MCP_MAX_MEM_MB` | 8192 | hard ceiling on `mem_mb` |

`AM_FLOOR_MB` (free-RAM floor for admission control) is honoured from the
environment as usual; on a small machine set it low (e.g. `AM_FLOOR_MB=1000`).

## Install

The engine binary must be built first (`engine/target/release/peanut`):

```
cargo build --release --manifest-path engine/Cargo.toml
```

Then install the server's Python dependencies (the official MCP SDK + psutil):

```
python -m venv .venv && . .venv/bin/activate
pip install -r mcp/requirements.txt
```

## Register with Claude Code

Use the built-in `claude mcp add`. Point it at the Python interpreter that has the
dependencies installed and at `mcp/server.py` (use absolute paths):

```
claude mcp add peanut -- /ABS/PATH/.venv/bin/python /ABS/PATH/mcp/server.py
```

For example, from this repo on the author's machine:

```
claude mcp add peanut -- /path/to/venv/bin/python /path/to/peanut/mcp/server.py
```

Verify it connected:

```
claude mcp list
```

To scope it to a single project instead of your user config, add `-s project`
(writes a `.mcp.json` next to your project), or `-s local` for just this machine.

## Register with Claude Desktop (config snippet)

Add an entry under `mcpServers` in `claude_desktop_config.json`
(macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "peanut": {
      "command": "/ABS/PATH/.venv/bin/python",
      "args": ["/ABS/PATH/mcp/server.py"],
      "env": { "AM_FLOOR_MB": "1000" }
    }
  }
}
```

The same block works as a `.mcp.json` for project-scoped Claude Code registration.

## Smoke test

`mcp/smoke_test.py` starts the server as a real stdio MCP subprocess, lists the
tools, and exercises each one. The headline check calls `peanut_decide` on a
Thue-Morse square query and asserts `TRUE`:

```
AM_FLOOR_MB=1000 .venv/bin/python mcp/smoke_test.py
```

Expected tail:

```
peanut_decide square: TRUE states= 1 ms= 1
...
SMOKE TEST PASSED
```
