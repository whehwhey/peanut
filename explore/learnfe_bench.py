"""learnfe vs `let FE`: agreement panel + the censored tail.

    python3 explore/learnfe_bench.py panel      -> results/learnfe_panel.json
    python3 explore/learnfe_bench.py censored   -> results/learnfe_censored.json

Panel rows run BOTH constructions in one engine session and then ask the engine itself
whether they agree (`A i,j,l. $FE(i,j,l) <=> $G(i,j,l)`), so "same size" is backed by a
proof of equal language, not just by matching integers.  Censored rows are the
sequences on which `let FE` failed in both digit orders at 6 GB
(results/blowup_residue.json / blowup_residue3.log); there is no `let FE` to compare
against, and correctness rests on learnfe's own recurrence check -- see docs/LEARNFE.md.

Always launched through explore/engine.py (see docs/GUARD.md).  Note that engine.py's
admission control waits for kernel memory pressure to be normal, so this will sit idle
while some other large job owns the machine.

Reads/writes: results/blowup_residue.json, results/blowup_residue3.log, results/learnfe_censored.json, results/learnfe_panel.json

Run:
    python3 explore/learnfe_bench.py [args - see __main__ / argv handling below]
"""
import os, sys, json, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import engine

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# censored tail: `let FE` failed msd AND lsd at 6 GB, and again under the small-cap
# Brzozowski retry (results/blowup_residue3.log)
CENSORED = [
    ("tail-c",  "def T 2 6 0 05 23 44 42 51 10 000010"),
    ("tail-b",  "def T 3 5 0 014 421 120 202 323 01100"),
    ("c-3.5b",  "def T 3 5 0 044 230 312 401 141 00111"),
    ("c-3.7a",  "def T 3 7 0 001 036 664 412 153 131 230 1101101"),
    ("c-3.7b",  "def T 3 7 0 004 334 114 653 155 301 245 0011011"),
    ("c-3.7c",  "def T 3 7 0 013 622 453 124 333 203 521 0011000"),
    ("c-3.7d",  "def T 3 7 0 020 412 341 404 625 512 153 0101111"),
    ("c-3.7e",  "def T 3 7 0 031 525 352 166 266 240 645 1011010"),
    ("c-3.7f",  "def T 3 7 0 034 552 341 662 310 243 154 0111101"),
    ("c-3.7g",  "def T 3 7 0 055 342 461 416 216 014 625 0001000"),
    ("c-3.7h",  "def T 3 7 0 056 343 462 501 220 250 010 1010110"),
    ("c-3.7i",  "def T 3 7 0 065 435 536 164 340 214 656 0110101"),
    ("c-3.7j",  "def T 3 7 0 065 630 536 161 341 241 461 0011010"),
]

def parse(out, row):
    for l in out.split("\n"):
        if l.startswith("OK let FE"):
            row["let_states"] = int(l.split("states=")[1].split()[0])
            row["let_ms"] = int(l.split("ms=")[1].split()[0])
        if l.startswith("OK learnfe"):
            f = dict(x.split("=", 1) for x in l.split() if "=" in x)
            row.update(learn_states=int(f["states"]), eqs=int(f["eqs"]), iters=int(f["iters"]),
                       ces=int(f["ces"]), mqs=int(f["mqs"]), learn_ms=int(f["ms"]),
                       capped=int(f.get("capped_lcp", 0)))
        if l.startswith(("TRUE", "FALSE")) and "<=>" in l:
            row["agree"] = l.split()[0]
        if l.startswith("OK mem"):
            row["peak_mb"] = int(l.split("peak=")[1].split("MB")[0])
        if l.startswith("ERR"):
            row.setdefault("err", []).append(l)
    return row

def go(name, d, mode="msd", compare=True, timeout=1800, mem=3000, tries=6):
    src = f"mode {mode}\n{d}\n"
    if compare: src += "let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]\n"
    src += "learnfe G\n"
    if compare: src += "? A i,j,l. $FE(i,j,l) <=> $G(i,j,l)\n"
    src += "mem\n"
    for _ in range(tries):
        r = engine.run(src, timeout=timeout, mem_mb=mem)
        if r.rc == 0 or r.timed_out or r.budget: break
        time.sleep(3)                     # killed by the system watchdog; retry
    row = dict(name=name, mode=mode, secs=round(r.secs, 1), rc=r.rc, timed_out=r.timed_out)
    return parse(r.stdout, row)

def main():
    which = sys.argv[1] if len(sys.argv) > 1 else "panel"
    if which == "panel":
        panel = json.load(open(f"{ROOT}/bench/panel.json"))
        jobs = [(n, d, "msd", True) for n, d in panel
                if n not in {"tail-a", "tail-b", "tail-c"}]
        jobs.append(("rudin-shapiro", dict(panel)["rudin-shapiro"], "lsd", True))
        out_path = f"{ROOT}/results/learnfe_panel.json"
    else:
        jobs = [(n, d, "msd", False) for n, d in CENSORED]
        out_path = f"{ROOT}/results/learnfe_censored.json"
    rows = []
    for name, d, mode, cmp_ in jobs:
        row = go(name, d, mode, cmp_)
        row["def"] = d
        print(json.dumps(row), flush=True)
        rows.append(row)
        json.dump(rows, open(out_path, "w"), indent=0)

if __name__ == "__main__":
    main()
