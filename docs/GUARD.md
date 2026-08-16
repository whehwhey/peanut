# GUARD — why the machine will not crash again

On 2026-08-16 the Mac hard-crashed mid-session. Cause: `explore/blowup.py` ran **12
engine processes in parallel, each allowed 4,000,000 subsets** (AM_CAP) with no memory
limit anywhere. A single FE query on a 3-state sequence peaks at ~500 MB; large-m
sequences run to many GB each. macOS does not OOM-kill — it compresses, swaps, and
eventually falls over. Three independent guards now stand between a sweep and the kernel.

## 1. Engine: hard allocator budget (`AM_MEM_MB`, default 2048)
`engine/src/membudget.rs` is a counting `#[global_allocator]`. When live bytes exceed
the budget the process prints `ERR memory budget exceeded (N MB)`, exits with status 3.
No unwinding, no partial state — a clean, cheap failure at a known ceiling.
`mem` command prints `OK mem live=..MB peak=..MB` — put it after a `let` to measure.

## 2. Runner: `explore/engine.py` — the ONLY way to launch the engine from Python
    import engine
    r = engine.run(src, timeout=60, cap=2_000_000)   # Result: .ok .stdout .timed_out .budget .rc
    res = engine.pool(jobs, fn)                       # resource-aware map
- workers = min(cpu-2, (free_RAM - FLOOR) / (MEM_MB+128), AM_WORKERS)
- admission control: a job does not launch until free RAM > FLOOR + MEM_MB and the
  kernel pressure level is normal
- watchdog thread: kills any child over 1.5×budget RSS; under critical pressure kills the
  biggest child
- all children are killed at exit / SIGINT — no orphans
- `run()` never raises: timeouts, budget exits and kills all come back as a `Result`
- `run_stream(src, on_event=...)` is `run()` plus `AM_PROGRESS=1`: the same admission
  control and watchdog, with the engine's stderr progress events delivered live.  This is
  how `gui/serve.py` runs jobs, so the GUI is inside the guards rather than beside them.
- `psutil` is used when present and falls back to `vm_stat` / `ps` when it is not, so the
  guards behave identically under the repo venv and under a bare `python3`
Every sweep script in `explore/` has been ported to it. Do not add
`ThreadPoolExecutor(max_workers=12)` + `subprocess.run([ENGINE]...)` again.
Defaults: AM_MEM_MB=1536, AM_FLOOR_MB=6144. Override via env.

## 3. System: `explore/memguard.sh` — LaunchAgent `com.andrew.maths.memguard`
Runs always (survives reboot, KeepAlive). Every 2 s: kills any engine process over
AM_KILL_MB=6144 RSS (last resort -- ABOVE any runner budget); if kernel pressure >= warn or free RAM < 3072 MB, kills the biggest
engine **that is over AM_PRESSURE_MIN_MB=384** (2026-08-17). Covers engines launched from anywhere — shell, Walnut comparisons, ad hoc.
The binary was renamed `automatheus` -> `peanut` on 2026-08-17; the guard matches **both**
names (`$3 ~ /(peanut|automatheus)$/`), since old checkouts and the compatibility symlink
`engine/target/release/automatheus -> peanut` still produce the old process name.
The LaunchAgent plist itself names only `explore/memguard.sh`, so it needed no change.
`target/` is git-ignored, so after a fresh build recreate the compatibility symlink for
any script still hard-coding the old path:
`ln -sf peanut engine/target/release/automatheus`.

The size floor on the pressure rule matters: without it, any large non-engine process on
the machine (another session's 4 GB Python job, say) pins the kernel at pressure=warn and
the guard then kills every engine within 2 s of launch, including 12 MB ones — which
relieves nothing and makes the repo unusable. The hard AM_KILL_MB ceiling is unchanged,
so the actual protection is untouched.
Log: `results/memguard.log`.  Reload: `launchctl kickstart -k gui/$(id -u)/com.andrew.maths.memguard`.

## Reading a sweep
A `FAIL budget` line means the automaton genuinely exceeded the ceiling — for Target 1
that IS the datum (the blowup), not a failure of the harness.
