"""Resource-aware runner for the Peanut engine.  USE THIS -- never call the engine
with a raw ThreadPoolExecutor(max_workers=12) again.

Three guards, from the inside out:
  1. engine   AM_MEM_MB   -- counting allocator, process exits 3 at the budget
  2. runner   this file   -- admission control from *actual* free RAM, RSS watchdog,
                             children killed on exit, no orphans
  3. system   explore/memguard.sh -- independent process, kills engines under pressure

Streaming:  run_stream(src, on_event, ...) is run() plus AM_PROGRESS=1 and a callback
per stderr progress event.  Same admission control, same watchdog, same cleanup --
the GUI (gui/serve.py) uses it so that a live job is still inside all three guards.

    from engine import run, pool
    out = run("mode msd\ndef T 2 2 0 01 10 01\n? A i. T[i]=T[i]\n")
    results = pool(jobs, fn)              # fn(job) -> result, fn calls run() inside

Environment overrides:  AM_MEM_MB (per-engine budget, default 1536)
                        AM_WORKERS (hard cap on parallelism)
                        AM_FLOOR_MB (system free-RAM floor, default 6144)

Portability (2026-08-19):  runs on macOS and on Windows (the "peanut-rig" Dell).  The
binary is `peanut.exe` on win32; the mac-only probes (`vm_stat`, `ps`, the
`kern.memorystatus_vm_pressure_level` sysctl) are never invoked off darwin -- there
psutil is the single source of truth and the pressure level is a constant 1 (normal),
so admission control degrades to the free-RAM floor alone.  macOS behaviour is
unchanged.

Reads/writes: no results/*.json artifacts referenced directly (see code for any docs/ or in-memory-only use).

Run:
    python3 explore/engine.py
"""
import os, sys, time, subprocess, threading, atexit, signal
from concurrent.futures import ThreadPoolExecutor

try:
    import psutil
except ImportError:                                   # system python3 has no psutil
    # Minimal stand-in over vm_stat / ps.  The guards must work from any interpreter --
    # gui/serve.py is run with plain `python3` -- and a missing package is not a reason
    # to launch engines with no admission control or no RSS watchdog.
    class _VM:
        def __init__(self, available): self.available = available
    class _MI:
        def __init__(self, rss): self.rss = rss
    class _Proc:
        def __init__(self, pid): self.pid = pid
        def memory_info(self):
            if sys.platform != "darwin":
                # no `ps -o rss=` to lean on: report 0 so the watchdog never kills on a
                # number it cannot measure (the engine's own AM_MEM_MB budget still holds)
                return _MI(0)
            try:
                out = subprocess.run(["ps", "-o", "rss=", "-p", str(self.pid)],
                                     capture_output=True, text=True, timeout=2).stdout.strip()
            except Exception:
                raise _NoSuchProcess()
            if not out: raise _NoSuchProcess()
            return _MI(int(out.split()[0]) * 1024)
    class _NoSuchProcess(Exception): pass
    class _psutil:
        NoSuchProcess = _NoSuchProcess
        AccessDenied = PermissionError
        Process = _Proc
        @staticmethod
        def virtual_memory():
            if sys.platform != "darwin":
                return _VM(1 << 40)                    # unknown: do not block
            try:
                out = subprocess.run(["vm_stat"], capture_output=True, text=True, timeout=2).stdout
            except Exception:
                return _VM(1 << 40)                    # unknown: do not block
            ps_, free = 4096, 0
            for line in out.split("\n"):
                if "page size of" in line:
                    ps_ = int(line.split("page size of")[1].split()[0])
                for tag in ("Pages free", "Pages inactive", "Pages speculative"):
                    if line.startswith(tag + ":"):
                        free += int(line.split(":")[1].strip().rstrip("."))
            return _VM(free * ps_)
    psutil = _psutil

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
IS_WIN = sys.platform.startswith("win")
IS_MAC = sys.platform == "darwin"
_EXE = ".exe" if IS_WIN else ""
# renamed automatheus -> peanut 2026-08-17; .exe on Windows
ENGINE = os.path.join(ROOT, "engine", "target", "release", "peanut" + _EXE)
if not os.path.exists(ENGINE):                                # fall back to the old name
    _old = os.path.join(ROOT, "engine", "target", "release", "automatheus" + _EXE)
    if os.path.exists(_old): ENGINE = _old
if "AM_ENGINE" in os.environ: ENGINE = os.environ["AM_ENGINE"]

MEM_MB   = int(os.environ.get("AM_MEM_MB", "1536"))   # per-engine allocator budget
def _total_mb():
    """Total physical RAM in MB.  psutil when present; else sysctl (mac) or
    /proc/meminfo (linux); else 'unknown' (1<<20 MB) so the clamp never fires."""
    try:
        t = psutil.virtual_memory().total                 # real psutil only; shim has none
        if t and t < (1 << 50):
            return t >> 20
    except Exception:
        pass
    try:
        if IS_MAC:
            out = subprocess.run(["sysctl", "-n", "hw.memsize"],
                                 capture_output=True, text=True, timeout=2).stdout.strip()
            if out:
                return int(out) >> 20
        elif not IS_WIN:
            with open("/proc/meminfo") as f:
                for line in f:
                    if line.startswith("MemTotal:"):
                        return int(line.split()[1]) >> 10   # kB -> MB
    except Exception:
        pass
    return 1 << 20                                          # unknown: do not clamp
# never launch if free RAM below this.  On a small machine a 6 GB floor would
# starve every job forever, so clamp the default to 40% of total RAM; an explicit
# AM_FLOOR_MB is still honoured as the requested value but never above 40% total.
FLOOR_MB = min(int(os.environ.get("AM_FLOOR_MB", "6144")), int(_total_mb() * 0.4))

_children = {}            # pid -> (Popen, kill_mb)
_lock = threading.Lock()
_stats = {"launched": 0, "killed_rss": 0, "killed_pressure": 0, "budget_exit": 0,
          "timeouts": 0, "waited_s": 0.0}

def free_mb():
    return psutil.virtual_memory().available >> 20

def pressure_level():
    """macOS kern.memorystatus_vm_pressure_level: 1 normal, 2 warn, 4 critical.

    Off darwin there is no such notion (Windows/Linux reclaim rather than signal), so we
    report 1 = normal without shelling out; admission control then rests on the free-RAM
    floor, which is the guard that actually matters on a 64 GB rig."""
    if not IS_MAC:
        return 1
    try:
        return int(subprocess.run(["sysctl", "-n", "kern.memorystatus_vm_pressure_level"],
                                  capture_output=True, text=True, timeout=2).stdout.strip() or 1)
    except Exception:
        return 1

def default_workers(requested=None):
    cpu = max(1, (os.cpu_count() or 4) - 2)
    by_mem = max(1, (free_mb() - FLOOR_MB) // (MEM_MB + 128))
    w = min(cpu, by_mem, requested or cpu)
    if "AM_WORKERS" in os.environ: w = min(w, int(os.environ["AM_WORKERS"]))
    return max(1, w)

def _admit():
    """Block until the system can afford another engine.  Never starve forever: after
    a long wait we re-check with a lower bar in case memory accounting is pessimistic."""
    t0 = time.time(); warned = False
    while True:
        f = free_mb(); p = pressure_level()
        if f > FLOOR_MB + MEM_MB and p <= 1: break
        if not warned and time.time() - t0 > 5:
            print(f"[engine] waiting for RAM: free={f}MB floor={FLOOR_MB}MB pressure={p}", file=sys.stderr, flush=True)
            warned = True
        time.sleep(0.5)
    _stats["waited_s"] += time.time() - t0

def admit_status(mem_mb=None):
    """Single non-blocking admission check.  Returns None if an engine can launch
    right now, else a short 'free=XMB floor=YMB' string explaining the wait.  The GUI
    uses this to answer 503 instead of hanging a request forever on a starved box."""
    need = (mem_mb or MEM_MB)
    f = free_mb(); p = pressure_level()
    if f > FLOOR_MB + need and p <= 1:
        return None
    return f"free={f}MB floor={FLOOR_MB}MB need={need}MB pressure={p}"

def _watchdog():
    while True:
        time.sleep(1.0)
        with _lock: procs = list(_children.values())
        if not procs: continue
        procs, kills = [p for p, _ in procs], {p.pid: k for p, k in procs}
        pressure = pressure_level()
        biggest = None; big_rss = 0
        for p in procs:
            try:
                rss = psutil.Process(p.pid).memory_info().rss >> 20
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
            if rss > big_rss: big_rss, biggest = rss, p
            if rss > kills[p.pid]:
                _stats["killed_rss"] += 1
                print(f"[engine] killing pid {p.pid}: rss {rss}MB > {kills[p.pid]}MB", file=sys.stderr, flush=True)
                _kill(p)
        if pressure >= 4 and biggest is not None:
            _stats["killed_pressure"] += 1
            print(f"[engine] SYSTEM MEMORY CRITICAL: killing biggest engine pid {biggest.pid} ({big_rss}MB)", file=sys.stderr, flush=True)
            _kill(biggest)

def _kill(p):
    try: p.kill()
    except Exception: pass

def _cleanup():
    with _lock: procs = [p for p, _ in _children.values()]
    for p in procs: _kill(p)

atexit.register(_cleanup)
# SIGINT/SIGTERM exist everywhere Python runs; SIGHUP and friends do not exist on
# Windows, so the set is built by name and anything missing is simply skipped.
for _name in ("SIGINT", "SIGTERM", "SIGHUP", "SIGBREAK"):
    _sig = getattr(signal, _name, None)
    if _sig is None: continue
    try:
        signal.signal(_sig, lambda s, f: (_cleanup(), sys.exit(128 + s)))
    except (ValueError, OSError, RuntimeError, AttributeError):
        pass  # not the main thread, or the platform will not let us handle it
threading.Thread(target=_watchdog, daemon=True).start()

class Result:
    __slots__ = ("stdout", "stderr", "rc", "timed_out", "budget", "killed", "secs")
    def __init__(self, stdout, stderr, rc, timed_out, secs):
        self.stdout, self.stderr, self.rc, self.timed_out, self.secs = stdout, stderr, rc, timed_out, secs
        self.budget = rc == 3 or "memory budget exceeded" in stdout
        self.killed = rc is not None and rc < 0
    @property
    def ok(self): return self.rc == 0 and not self.timed_out
    def lines(self, prefix): return [l for l in self.stdout.split("\n") if l.startswith(prefix)]
    def verdict(self):
        l = self.lines(("TRUE", "FALSE"))
        return "1" if l and l[0].startswith("TRUE") else "0" if l else "?"

def run(src, timeout=60, mem_mb=None, cap=None, env=None):
    """Run one engine session on `src` (a full script including quit).  Always returns a
    Result; never raises on timeout / budget / kill."""
    if not src.rstrip().endswith("quit"): src = src.rstrip() + "\nquit\n"
    e = {**os.environ, "AM_MEM_MB": str(mem_mb or MEM_MB)}
    if cap: e["AM_CAP"] = str(cap)
    if env: e.update(env)
    _admit()
    t0 = time.time()
    p = subprocess.Popen([ENGINE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE, text=True, encoding="utf-8",
                         errors="replace", env=e)
    kill_mb = int((mem_mb or MEM_MB) * 1.5) + 256
    with _lock: _children[p.pid] = (p, kill_mb); _stats["launched"] += 1
    timed_out = False
    try:
        out, err = p.communicate(src, timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True; _stats["timeouts"] += 1
        _kill(p)
        try: out, err = p.communicate(timeout=5)
        except Exception: out, err = "", ""
    finally:
        with _lock: _children.pop(p.pid, None)
    r = Result(out or "", err or "", p.returncode, timed_out, time.time() - t0)
    if r.budget: _stats["budget_exit"] += 1
    return r

def run_stream(src, on_event=None, timeout=600, mem_mb=None, cap=None, env=None,
               on_line=None, on_spawn=None):
    """run() with the engine's progress stream switched on (AM_PROGRESS=1).

    `on_event(dict)` is called for every structured stderr event as it arrives, and
    `on_line(str)` for every stdout line.  Everything else -- admission control, the RSS
    watchdog, kill-on-exit -- is identical to run(), which is the whole point: the GUI
    must not become a fourth way to launch an unguarded engine.
    """
    import json as _json
    if not src.rstrip().endswith("quit"): src = src.rstrip() + "\nquit\n"
    e = {**os.environ, "AM_MEM_MB": str(mem_mb or MEM_MB), "AM_PROGRESS": "1"}
    if cap: e["AM_CAP"] = str(cap)
    if env: e.update(env)
    _admit()
    t0 = time.time()
    p = subprocess.Popen([ENGINE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE, text=True, encoding="utf-8",
                         errors="replace", bufsize=1, env=e)
    kill_mb = int((mem_mb or MEM_MB) * 1.5) + 256
    with _lock: _children[p.pid] = (p, kill_mb); _stats["launched"] += 1
    if on_spawn:
        try: on_spawn(p)          # lets a caller cancel the job (gui/serve.py does)
        except Exception: pass
    out_buf, err_buf = [], []

    def pump_out():
        for line in p.stdout:
            out_buf.append(line)
            if on_line:
                try: on_line(line.rstrip("\n"))
                except Exception: pass

    def pump_err():
        for line in p.stderr:
            err_buf.append(line)
            line = line.strip()
            if not line: continue
            if line.startswith("{") and on_event:
                try: ev = _json.loads(line)
                except Exception: continue
                try: on_event(ev)
                except Exception: pass

    to = threading.Thread(target=pump_out, daemon=True)
    te = threading.Thread(target=pump_err, daemon=True)
    to.start(); te.start()
    try:
        p.stdin.write(src); p.stdin.flush(); p.stdin.close()
    except Exception:
        pass
    timed_out = False
    try:
        p.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True; _stats["timeouts"] += 1
        _kill(p)
        try: p.wait(timeout=5)
        except Exception: pass
    finally:
        with _lock: _children.pop(p.pid, None)
    to.join(timeout=5); te.join(timeout=5)
    r = Result("".join(out_buf), "".join(err_buf), p.returncode, timed_out, time.time() - t0)
    if r.budget: _stats["budget_exit"] += 1
    return r


def pool(jobs, fn, workers=None, label=None):
    """Map fn over jobs with resource-aware parallelism.  fn should call run()."""
    jobs = list(jobs)
    w = default_workers(workers)
    loud = len(jobs) >= 20
    if loud: print(f"[engine] {label or 'pool'}: {len(jobs)} jobs, {w} workers "
          f"(free={free_mb()}MB, budget={MEM_MB}MB/engine, floor={FLOOR_MB}MB)", file=sys.stderr, flush=True)
    t0 = time.time()
    with ThreadPoolExecutor(max_workers=w) as ex:
        res = list(ex.map(fn, jobs))
    if loud: print(f"[engine] done in {time.time()-t0:.0f}s: {_stats}", file=sys.stderr, flush=True)
    return res

def stats(): return dict(_stats)
