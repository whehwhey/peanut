#!/usr/bin/env python3
"""Peanut MCP server -- drive the Peanut decision engine from any MCP-aware agent.

This is an OPEN, community MCP server for the Peanut engine. It is NOT an
Anthropic-official product; it just speaks the Model Context Protocol so that
Claude Code, Claude Desktop or any other MCP client can call the engine natively
over stdio.

Peanut decides first-order statements over k-automatic sequences (and the
Fibonacci / Tribonacci / Pell numeration systems). See docs/COMMANDS.md for the
full command language and docs/PYTHON-API.md for the runner these tools wrap.

Every tool here goes through explore/engine.py (`engine.run`), so all three
resource guards apply: the counting allocator budget (AM_MEM_MB), the runner's
admission control + RSS watchdog, and the system memguard. No tool ever launches
the binary directly. Errors -- timeouts, memory-budget exits, engine ERR lines,
parse failures -- are returned as structured fields, never raised to the client.

Transport: stdio. Run with:  python mcp/server.py
"""
import os
import sys
import json

# explore/engine.py is the ONLY sanctioned way to launch the engine.
_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(_ROOT, "explore"))
import engine  # noqa: E402

# The MCP SDK renamed the high-level server class from FastMCP (1.x) to MCPServer
# (2.x). Import whichever this environment has; both expose .tool() and .run().
try:
    from mcp.server.fastmcp import FastMCP as _Server  # SDK 1.x
except Exception:  # pragma: no cover - depends on installed SDK
    from mcp.server.mcpserver import MCPServer as _Server  # SDK 2.x

# ---------------------------------------------------------------------------
# Resource caps. Overridable by env, but always clamped to a sane ceiling so a
# runaway request cannot ask for an unbounded engine.
# ---------------------------------------------------------------------------
DEFAULT_TIMEOUT_S = int(os.environ.get("PEANUT_MCP_TIMEOUT_S", "60"))
MAX_TIMEOUT_S = int(os.environ.get("PEANUT_MCP_MAX_TIMEOUT_S", "600"))
DEFAULT_MEM_MB = int(os.environ.get("PEANUT_MCP_MEM_MB", "1536"))
MAX_MEM_MB = int(os.environ.get("PEANUT_MCP_MAX_MEM_MB", "8192"))
MAX_TERMS = 4000  # cap for peanut_sequence's n


def _clamp_timeout(t):
    if t is None:
        return DEFAULT_TIMEOUT_S
    try:
        t = int(t)
    except Exception:
        return DEFAULT_TIMEOUT_S
    return max(1, min(t, MAX_TIMEOUT_S))


def _clamp_mem(m):
    if m is None:
        return DEFAULT_MEM_MB
    try:
        m = int(m)
    except Exception:
        return DEFAULT_MEM_MB
    return max(64, min(m, MAX_MEM_MB))


def _num(line, key):
    """Extract `key=<int>` from a line, or None."""
    tok = key + "="
    i = line.find(tok)
    if i < 0:
        return None
    tail = line[i + len(tok):].split()[0] if line[i + len(tok):].split() else ""
    try:
        return int(tail)
    except Exception:
        return None


def _err_lines(out):
    return [l for l in out.split("\n") if l.startswith("ERR") or l.startswith("WERR")]


def _run(script, timeout_s, mem_mb):
    """Run a script through the guarded runner and return (Result, base_dict).

    base_dict carries the run-level facts every tool reports; the Result lets a
    tool parse its own stdout lines. Never raises for engine-side failures.
    """
    r = engine.run(script, timeout=timeout_s, mem_mb=mem_mb)
    errs = _err_lines(r.stdout)
    base = {
        "ok": bool(r.ok and not errs),
        "timed_out": bool(r.timed_out),
        "memory_budget_exceeded": bool(r.budget),
        "killed": bool(r.killed),
        "return_code": r.rc,
        "elapsed_s": round(r.secs, 3),
        "engine_errors": errs,
        "stdout": r.stdout,
    }
    # Turn known failure modes into a single human-readable `error` string, or None.
    if r.timed_out:
        base["error"] = f"engine timed out after {timeout_s}s"
    elif r.budget:
        base["error"] = f"memory budget exceeded (cap {mem_mb} MB)"
    elif r.killed:
        base["error"] = f"engine killed by signal (rc={r.rc})"
    elif errs:
        base["error"] = errs[0]
    else:
        base["error"] = None
    return r, base


mcp = _Server(
    name="peanut",
    version="0.1.0",
    instructions=(
        "Peanut decides first-order statements over k-automatic sequences and the "
        "Fibonacci/Tribonacci/Pell numeration systems. Define a sequence with a "
        "`sequence_def` (engine commands that set the current sequence, e.g. "
        "'def T 2 2 0 01 10 01' for Thue-Morse, or 'numsys fib\\ndfao F 2 0:0,1 1:0,-' "
        "for the Fibonacci word), then prove sentences, find witnesses, or export "
        "automata. The sequence is always named T inside formulas. See "
        "docs/COMMANDS.md for the full grammar."
    ),
)


# ---------------------------------------------------------------------------
# Tools
# ---------------------------------------------------------------------------

@mcp.tool(
    description=(
        "Run a raw Peanut command script (docs/COMMANDS.md) and return the parsed "
        "result. Use this for anything the specialized tools do not cover: multi-step "
        "sessions, `let`/`enum`/`dfa`/`finite`/`transduce`, Walnut-compat mode, etc.\n\n"
        "The script is a newline-separated list of engine commands; `quit` is appended "
        "automatically. Inside formulas the current sequence is always named T.\n\n"
        "Returns: ok, verdict (TRUE/FALSE/OPEN/null from the last decidable line), "
        "states, peak, ms, the full stdout, the per-line list, any engine ERR lines, "
        "and error (null on success). Timeouts and memory-budget exits come back as "
        "structured fields, never as exceptions.\n\n"
        "Example script (does Thue-Morse contain a square?):\n"
        "  def T 2 2 0 01 10 01\n"
        "  ? E i. E n. n>=1 & (A t. t<n => T[i+t]=T[i+n+t])\n"
        "-> verdict TRUE."
    )
)
def peanut_decide(script: str, timeout_s: int = DEFAULT_TIMEOUT_S,
                  mem_mb: int = DEFAULT_MEM_MB) -> dict:
    to, mm = _clamp_timeout(timeout_s), _clamp_mem(mem_mb)
    if not script or not script.strip():
        return {"ok": False, "error": "empty script", "verdict": None,
                "states": None, "peak": None, "ms": None, "lines": [], "stdout": ""}
    r, base = _run(script, to, mm)
    lines = [l for l in r.stdout.split("\n") if l.strip()]
    # Verdict / metrics from the last TRUE/FALSE/OPEN/CLOSED line, if any.
    verdict, states, peak, ms = None, None, None, None
    for l in lines:
        if l.startswith(("TRUE", "FALSE", "OPEN")):
            verdict = "OPEN" if l.startswith("OPEN") else l.split()[0]
            states = _num(l, "states")
            peak = _num(l, "peak")
            ms = _num(l, "ms")
        elif l.startswith("CLOSED"):
            verdict = l.split()[1] if len(l.split()) > 1 else None
    base.update({"verdict": verdict, "states": states, "peak": peak, "ms": ms,
                 "lines": lines})
    return base


@mcp.tool(
    description=(
        "Decide a CLOSED first-order sentence over a named sequence. Returns "
        "TRUE/FALSE plus the decision automaton's state count.\n\n"
        "sequence_def: engine command(s) that set the current sequence. Examples:\n"
        "  'def T 2 2 0 01 10 01'                     (Thue-Morse, base 2)\n"
        "  'def T 3 2 0 012 021 01'                   (period-doubling style, base 3)\n"
        "  'numsys fib\\ndfao F 2 0:0,1 1:0,-'         (Fibonacci word)\n"
        "sentence: a closed formula (no free variables), sequence named T. Examples:\n"
        "  'A i. T[i]=T[i]'                            -> TRUE\n"
        "  'E i. E n. n>=1 & (A t. t<n => T[i+t]=T[i+n+t])'  (has a square?) -> TRUE\n\n"
        "If the formula has free variables the result is verdict OPEN and an error "
        "explaining it is not closed; use peanut_witness for open formulas."
    )
)
def peanut_prove(sequence_def: str, sentence: str,
                 timeout_s: int = DEFAULT_TIMEOUT_S,
                 mem_mb: int = DEFAULT_MEM_MB) -> dict:
    to, mm = _clamp_timeout(timeout_s), _clamp_mem(mem_mb)
    if not sequence_def or not sequence_def.strip():
        return {"ok": False, "error": "empty sequence_def", "verdict": None, "states": None}
    if not sentence or not sentence.strip():
        return {"ok": False, "error": "empty sentence", "verdict": None, "states": None}
    script = sequence_def.rstrip() + "\n? " + sentence.strip() + "\n"
    r, base = _run(script, to, mm)
    verdict, states, peak, ms = None, None, None, None
    for l in r.stdout.split("\n"):
        if l.startswith(("TRUE", "FALSE")):
            verdict, states, peak, ms = l.split()[0], _num(l, "states"), _num(l, "peak"), _num(l, "ms")
        elif l.startswith("OPEN"):
            verdict, states, ms = "OPEN", _num(l, "states"), _num(l, "ms")
            if base["error"] is None:
                base["error"] = "sentence is not closed (has free variables); use peanut_witness"
                base["ok"] = False
    if verdict is None and base["error"] is None:
        base["error"] = "no verdict produced; check the sequence_def and sentence"
        base["ok"] = False
    base.update({"verdict": verdict, "states": states, "peak": peak, "ms": ms})
    return base


@mcp.tool(
    description=(
        "Find ONE satisfying assignment for a formula over a named sequence, or NONE "
        "if the language is empty. Uses the shortest accepted word.\n\n"
        "sequence_def: as in peanut_prove.\n"
        "formula: leave the variable(s) you want reported FREE (do not quantify them). "
        "Examples (Thue-Morse, sequence named T):\n"
        "  'T[i]=1'            -> assignment {i: 1}\n"
        "  'T[i]!=T[j] & i<j'  -> a concrete (i, j) pair\n\n"
        "Returns: satisfiable (bool), assignment (dict var->value or null), states, "
        "word_len, ms. A closed formula degenerates to verdict TRUE/FALSE with no "
        "assignment."
    )
)
def peanut_witness(sequence_def: str, formula: str,
                   timeout_s: int = DEFAULT_TIMEOUT_S,
                   mem_mb: int = DEFAULT_MEM_MB) -> dict:
    to, mm = _clamp_timeout(timeout_s), _clamp_mem(mem_mb)
    if not sequence_def or not sequence_def.strip():
        return {"ok": False, "error": "empty sequence_def", "satisfiable": None, "assignment": None}
    if not formula or not formula.strip():
        return {"ok": False, "error": "empty formula", "satisfiable": None, "assignment": None}
    script = sequence_def.rstrip() + "\nwitness " + formula.strip() + "\n"
    r, base = _run(script, to, mm)
    satisfiable, assignment, verdict, states, wlen, ms = None, None, None, None, None, None
    for l in r.stdout.split("\n"):
        if l.startswith("WITNESS"):
            satisfiable = True
            states, wlen, ms = _num(l, "states"), _num(l, "len"), _num(l, "ms")
            body = l[len("WITNESS"):].split("::")[0].strip()
            assignment = {}
            _reserved = {"states", "len", "ms", "peak"}
            for tok in body.split():
                k, _, v = tok.partition("=")
                if k in _reserved:
                    break  # metrics trail the assignment; stop here
                if "=" in tok and not any(c in tok for c in "<>!"):
                    try:
                        assignment[k] = int(v)
                    except Exception:
                        assignment[k] = v
        elif l.startswith("NONE"):
            satisfiable = False
            states, ms = _num(l, "states"), _num(l, "ms")
        elif l.startswith(("TRUE", "FALSE")):
            verdict = l.split()[0]
            satisfiable = (verdict == "TRUE")
            states, ms = _num(l, "states"), _num(l, "ms")
    if satisfiable is None and base["error"] is None:
        base["error"] = "no witness line produced; check the sequence_def and formula"
        base["ok"] = False
    base.update({"satisfiable": satisfiable, "assignment": assignment,
                 "verdict": verdict, "states": states, "word_len": wlen, "ms": ms})
    return base


@mcp.tool(
    description=(
        "Build the equality-of-factors automaton FE(i,j,l) := (A t<l. T[i+t]=T[j+t]) "
        "for a named sequence, by Khodier-style guess-and-verify active learning "
        "(the `learnfe` command). Returns the minimal DFA's state count and the "
        "learner's work counters.\n\n"
        "sequence_def: as in peanut_prove. This is the classic 'do two length-l "
        "factors starting at i and j agree?' predicate underlying many combinatorics-"
        "on-words results.\n\n"
        "Returns: states (minimal DFA size), iters, eqs (equivalence queries), "
        "ces (counterexamples), mqs (membership queries), ms. Example: Thue-Morse "
        "('def T 2 2 0 01 10 01') gives states=15."
    )
)
def peanut_fe(sequence_def: str, timeout_s: int = 300,
              mem_mb: int = DEFAULT_MEM_MB) -> dict:
    to, mm = _clamp_timeout(timeout_s), _clamp_mem(mem_mb)
    if not sequence_def or not sequence_def.strip():
        return {"ok": False, "error": "empty sequence_def", "states": None}
    script = sequence_def.rstrip() + "\nlearnfe FE\n"
    r, base = _run(script, to, mm)
    fields = {"states": None, "iters": None, "eqs": None, "ces": None,
              "mqs": None, "ms": None}
    for l in r.stdout.split("\n"):
        if l.startswith("OK learnfe"):
            for k in fields:
                fields[k] = _num(l, k)
    if fields["states"] is None and base["error"] is None:
        base["error"] = "learnfe produced no state count; check the sequence_def"
        base["ok"] = False
    base.update(fields)
    return base


@mcp.tool(
    description=(
        "Return the first n terms of a named sequence as a digit string.\n\n"
        "sequence_def: as in peanut_prove.\n"
        "n: number of terms (default 60, capped at 4000).\n\n"
        "Examples:\n"
        "  sequence_def='def T 2 2 0 01 10 01', n=20 -> '01101001100101101001' (Thue-Morse)\n"
        "  sequence_def='numsys fib\\ndfao F 2 0:0,1 1:0,-', n=20 -> Fibonacci word.\n\n"
        "Returns: terms (string), n, k (base/digit count)."
    )
)
def peanut_sequence(sequence_def: str, n: int = 60,
                    timeout_s: int = DEFAULT_TIMEOUT_S,
                    mem_mb: int = DEFAULT_MEM_MB) -> dict:
    to, mm = _clamp_timeout(timeout_s), _clamp_mem(mem_mb)
    if not sequence_def or not sequence_def.strip():
        return {"ok": False, "error": "empty sequence_def", "terms": None}
    try:
        n = int(n)
    except Exception:
        n = 60
    n = max(1, min(n, MAX_TERMS))
    script = sequence_def.rstrip() + f"\nseq {n}\n"
    r, base = _run(script, to, mm)
    terms, k = None, None
    for l in r.stdout.split("\n"):
        if l.startswith("SEQ"):
            k = _num(l, "k")
            parts = l.split()
            terms = parts[-1] if parts else None
    if terms is None and base["error"] is None:
        base["error"] = "no SEQ line produced; check the sequence_def"
        base["ok"] = False
    base.update({"terms": terms, "n": n, "k": k})
    return base


@mcp.tool(
    description=(
        "Export an automaton as JSON (the format the Peanut GUI consumes; see "
        "engine/src/export.rs). Use it to inspect states, transitions and outputs "
        "programmatically.\n\n"
        "sequence_def: as in peanut_prove. If you want to export a `let`/`learnfe` "
        "predicate rather than the sequence itself, include its definition in "
        "sequence_def (e.g. 'def T 2 2 0 01 10 01\\nlet EQ(i,j) T[i]=T[j]').\n"
        "name: which automaton to dump -- 'T' (or the sequence's def name) for the "
        "DFAO of the sequence, or the name of any let/learnfe predicate.\n\n"
        "Returns: automaton (parsed JSON object; kind 'dfao' for the sequence, 'dfa' "
        "for a predicate). Large automata are truncated at AM_EXPORT_MAX states."
    )
)
def peanut_export(sequence_def: str, name: str = "T",
                  timeout_s: int = DEFAULT_TIMEOUT_S,
                  mem_mb: int = DEFAULT_MEM_MB) -> dict:
    to, mm = _clamp_timeout(timeout_s), _clamp_mem(mem_mb)
    if not sequence_def or not sequence_def.strip():
        return {"ok": False, "error": "empty sequence_def", "automaton": None}
    name = (name or "T").strip()
    script = sequence_def.rstrip() + f"\nexport {name}\n"
    r, base = _run(script, to, mm)
    automaton = None
    for l in r.stdout.split("\n"):
        if l.startswith("EXPORT "):
            try:
                automaton = json.loads(l[len("EXPORT "):])
            except Exception as e:
                base["error"] = f"failed to parse EXPORT JSON: {e}"
                base["ok"] = False
    if automaton is None and base["error"] is None:
        base["error"] = f"no automaton exported for name '{name}'; check the sequence_def"
        base["ok"] = False
    base.update({"automaton": automaton, "name": name})
    return base


if __name__ == "__main__":
    mcp.run(transport="stdio")
