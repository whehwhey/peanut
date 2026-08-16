# PYTHON-API — `explore/engine.py`

`explore/engine.py` is the **only** sanctioned way to launch the Peanut engine
(`engine/target/release/peanut`) from Python. It wraps the raw subprocess in three
layers of resource control; see `docs/GUARD.md` for why (the machine hard-crashed
once from unguarded `ThreadPoolExecutor` + `subprocess.run` parallelism). Every
sweep script in `explore/` uses it. Requires `psutil`.

```python
import sys; sys.path.insert(0, "explore")   # or run from repo root with explore/ on path
import engine

r = engine.run("mode msd\ndef T 2 2 0 01 10 01\n? A i. T[i]=T[i]\n")
print(r.ok, r.stdout)

results = engine.pool(jobs, worker_fn)      # resource-aware parallel map
```

## `run(src, timeout=60, mem_mb=None, cap=None, env=None) -> Result`

Runs one engine session on `src` — a full stdin script. `quit` is appended
automatically if the script doesn't already end with it.

- `timeout` — wall-clock seconds before the child is killed and `Result.timed_out`
  is set.
- `mem_mb` — per-process `AM_MEM_MB` override (defaults to the module-level
  `MEM_MB`, itself `AM_MEM_MB` env or 1536).
- `cap` — sets `AM_CAP` for this run.
- `env` — dict merged into the child's environment last (overrides `mem_mb`/`cap`
  if it sets the same keys).

`run()` **never raises**. Timeouts, memory-budget exits (`rc == 3`), and watchdog
kills all come back as a normal `Result`, never an exception.

Before launching, `run()` blocks in `_admit()` until free system RAM exceeds
`AM_FLOOR_MB + mem_mb` *and* macOS kernel memory-pressure is `normal` (not
`warn`/`critical`) — admission control, the second of the three guard layers. A
background watchdog thread checks every running child once a second and kills
anything over `1.5 * mem_mb + 256` MB RSS, or (under critical system pressure) the
single largest running child. All children are registered for cleanup on process
exit and on `SIGINT`/`SIGTERM` — no orphaned engine processes survive the Python
process.

### `Result`

```python
class Result:
    stdout: str
    stderr: str
    rc: int | None          # process return code (None if killed before exit)
    timed_out: bool
    secs: float              # wall time
    budget: bool              # rc == 3 or "memory budget exceeded" in stdout
    killed: bool               # rc is not None and rc < 0 (killed by signal)

    @property
    def ok(self) -> bool: ...   # rc == 0 and not timed_out

    def lines(self, prefix) -> list[str]:
        """Lines of stdout starting with `prefix` (str or tuple of str)."""

    def verdict(self) -> str:
        """'1' if a TRUE line was seen, '0' if a FALSE line, else '?'."""
```

Typical pattern:

```python
r = engine.run(src, timeout=90, mem_mb=1536)
if r.budget:
    ...  # genuine resource exhaustion, not a bug -- see docs/GUARD.md "Reading a sweep"
elif r.timed_out:
    ...
elif not r.ok:
    print("engine error:", r.stdout, r.stderr)
else:
    states = int(r.lines("OK let")[0].split("states=")[1].split()[0])
```

## `pool(jobs, fn, workers=None, label=None) -> list`

Resource-aware `concurrent.futures.ThreadPoolExecutor` map. `fn` takes one job and
is expected to call `engine.run(...)` internally (so admission control and the
watchdog cover every launch it makes). Worker count is
`min(cpu_count - 2, (free_MB - AM_FLOOR_MB) / (AM_MEM_MB + 128), AM_WORKERS)`,
computed once at pool start via `default_workers()`. Jobs with `len(jobs) >= 20`
print a one-line banner (job count, worker count, free/budget/floor MB) to stderr
before starting and a summary (elapsed, `engine.stats()`) after.

```python
def one(job):
    src = build_script(job)
    return job, engine.run(src, timeout=90)

results = engine.pool(jobs, one, label="blowup sweep")
```

## `stats() -> dict`

Cumulative counters since import: `launched`, `killed_rss`, `killed_pressure`,
`budget_exit`, `timeouts`, `waited_s` (total time spent blocked in admission
control). Useful at the end of a sweep to sanity-check that guard layers actually
fired when expected.

## Env vars this module reads

| var | default | meaning |
|---|---|---|
| `AM_MEM_MB` | 1536 | Default per-engine `AM_MEM_MB`, used by `run()` when no `mem_mb` arg is given and by `pool()`'s worker-count math. |
| `AM_FLOOR_MB` | 6144 | System free-RAM floor: `run()` will not launch (and `pool()` will not size workers) below `AM_FLOOR_MB + MEM_MB` free. |
| `AM_WORKERS` | (unset = cpu-based) | Hard ceiling on `pool()` parallelism, applied after the RAM-based and CPU-based bounds. |

Everything the *engine binary itself* reads (`AM_MEM_MB` again, `AM_CAP0`,
`AM_CAP`, `AM_LEARN_*`, `AM_DEBUG*`) is documented in `docs/COMMANDS.md`.

## What NOT to do

Do not `subprocess.run([ENGINE], ...)` directly, do not build your own
`ThreadPoolExecutor(max_workers=N)` around raw subprocess calls, and do not launch
the engine from a shell loop outside Python's guard. `explore/memguard.sh` (system
layer, `docs/GUARD.md` §3) is a last-resort backstop, not a substitute for using
`engine.run`/`engine.pool`.
