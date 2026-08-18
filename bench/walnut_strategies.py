"""Re-run the FE-panel and Tribonacci hard cases through Walnut 8-dev with every
determinization strategy Walnut 7.0+ offers (John Nicol's correction: our earlier
BENCHMARKS.md compared only against Walnut's SC default).

Strategies (see walnut7/Help Documentation/Commands/Metacommands/[strategy].txt and
walnut7/src/main/java/Automata/FA/DeterminizationStrategies.java):
    SC          subset construction (default, already measured -- not rerun here)
    BRZ         Brzozowski (reverse, determinize, minimize, reverse, determinize)
    CCL         OTF convexity-closure-lattice
    CCLS        OTF convexity-closure-lattice with simulation
    BRZ-CCL     Brzozowski + CCL
    BRZ-CCLS    Brzozowski + CCLS

Syntax: "[strategy * X]" immediately before a command, command must end in "::"
(single ":" silently ignores the metacommand -- Prover only calls
MetaCommands.parseMetaCommands when printDetails is set, which requires "::").
Placed on the *last* line only (the "def"/"eval" that does the quantifier
elimination), so the earlier morphism/promote/image lines that build the word
automaton (a DFAO) still use SC -- non-SC strategies refuse DFAOs
("DeterminizationStrategies.java": "DFAOs are not supported for non-SC strategies").

Hard cases rerun:
    - panel.json rows that OOM'd or timed out under SC (bench/README.md):
      prism-1, single3, single4, single5, single6, tail-a, tail-b, tail-c
      (single3..6 are printed as "[s2=a mod 3..6]" in bench/README.md)
    - fib.md Tribonacci rows that OOM'd under SC: cube exists, 4th power exists,
      palindrome of every length, FE(i,j,l) direct

Runs with bounded parallelism (several JVMs at once -- 18 cores / 24 GB on this
machine, -Xmx6g each) to keep total wall-clock down; each run is still an
independent, unmodified Walnut invocation with its own 900 s ceiling.

Run:
    python3 bench/walnut_strategies.py both       # everything, writes .jsonl as it goes
    python3 bench/walnut_strategies.py panel
    python3 bench/walnut_strategies.py trib
"""
import os, sys, json, subprocess, time, re
from concurrent.futures import ThreadPoolExecutor, as_completed

ROOT = "/Users/andrew/maths"
W = os.path.join(ROOT, "walnut7")
JAVA = "/opt/homebrew/opt/openjdk/bin/java"
TIMEOUT = 900  # 15 min, same ceiling as the original benchmarks
WORKERS = 3

STRATEGIES = ["BRZ", "CCL", "CCLS", "BRZ-CCL", "BRZ-CCLS"]

PANEL = json.load(open(os.path.join(ROOT, "bench", "panel.json")))
PANEL_HARD = {"prism-1", "single3", "single4", "single5", "single6",
              "tail-a", "tail-b", "tail-c"}

TRIB_HARD = [
    ("cube exists",
     "Ei,n (n>=1) & (At (t<2*n) => (TR[i+t]=TR[i+n+t]))"),
    ("4th power exists",
     "Ei,n (n>=1) & (At (t<3*n) => (TR[i+t]=TR[i+n+t]))"),
    ("palindrome of every length",
     "An Ei At,u ((t<n) & (t+u+1=n)) => (TR[i+t]=TR[i+u])"),
    ("FE(i,j,l) [direct]",
     "At (t<l) => (TR[i+t]=TR[j+t])"),
]


def parse_def(d):
    p = d.split()
    k, m = int(p[2]), int(p[3])
    w = p[5:5 + m]
    c = p[5 + m]
    return k, m, w, c


def run_walnut(src, timeout=TIMEOUT):
    t0 = time.time()
    try:
        r = subprocess.run([JAVA, "-Xmx6g", "-jar", "target/Walnut-all.jar"],
                            input=src, capture_output=True, text=True,
                            timeout=timeout, cwd=W)
        out = r.stdout + r.stderr
        secs = time.time() - t0
    except subprocess.TimeoutExpired:
        return {"states": "timeout", "s": round(time.time() - t0, 1)}
    verdict = "TRUE" if re.search(r"^TRUE\s*$", out, re.M) else \
              ("FALSE" if re.search(r"^FALSE\s*$", out, re.M) else "")
    if "OutOfMemoryError" in out:
        sizes = [int(x) for x in re.findall(r":\s*(\d+) states", out)]
        return {"states": "OOM", "peak": max(sizes) if sizes else "-", "s": round(secs, 1)}
    sizes = [int(x) for x in re.findall(r":\s*(\d+) states", out)]
    tot = re.findall(r"Total computation time: (\d+)ms", out)
    final = sizes[-1] if sizes else None
    if final is None:
        return {"states": "error", "s": round(secs, 1), "verdict": verdict, "tail": out[-400:]}
    return {"states": final, "peak": max(sizes), "s": round(secs, 1),
            "ms": int(tot[-1]) if tot else None, "verdict": verdict}


def panel_task(name, d, strategy):
    k, m, w, c = parse_def(d)
    morph = " ".join(f"{a}->{w[a]}" for a in range(m))
    cod = " ".join(f"{a}->{c[a]}" for a in range(m))
    tag = "S" + re.sub(r'\W', '', name)
    src = (f'morphism mf{tag} "{morph}";\n'
           f'promote PW{tag} mf{tag};\n'
           f'morphism cd{tag} "{cod}";\n'
           f'image {tag} cd{tag} PW{tag};\n'
           f'[strategy * {strategy}]def fe{tag} "?msd_{k} At (t<l) => {tag}[i+t]={tag}[j+t]"::\n'
           f'exit;\n')
    row = {"kind": "panel", "name": name, "strategy": strategy}
    row.update(run_walnut(src))
    return row


def trib_task(label, formula, strategy):
    tag = "T" + re.sub(r'\W', '', label)
    src = f'[strategy * {strategy}]def {tag} "?msd_trib {formula}"::\nexit;\n'
    row = {"kind": "trib", "name": label, "strategy": strategy}
    row.update(run_walnut(src))
    return row


def main(which):
    jobs = []
    if which in ("panel", "both"):
        for name, d in PANEL:
            if name in PANEL_HARD:
                for strat in STRATEGIES:
                    jobs.append(("panel", name, d, strat))
    if which in ("trib", "both"):
        for label, formula in TRIB_HARD:
            for strat in STRATEGIES:
                jobs.append(("trib", label, formula, strat))

    outpath = os.path.join(ROOT, "bench", "walnut_strategies_results.jsonl")
    # RESUME: skip (name, strategy) pairs already recorded in bench/walnut_strategies.log
    done = set()
    logp = os.path.join(ROOT, "bench", "walnut_strategies.log")
    if os.path.exists(logp):
        import ast
        for line in open(logp):
            try:
                r = ast.literal_eval(line)
                done.add((r.get("name") or r.get("label"), r["strategy"]))
            except Exception:
                pass
    jobs = [j for j in jobs if (j[1], j[3]) not in done]
    print(f"{len(jobs)} jobs remaining ({len(done)} already done)", flush=True)
    results = []
    with ThreadPoolExecutor(max_workers=WORKERS) as ex, open(outpath, "a") as f:
        futs = {}
        for kind, a, b, strat in jobs:
            if kind == "panel":
                fut = ex.submit(panel_task, a, b, strat)
            else:
                fut = ex.submit(trib_task, a, b, strat)
            futs[fut] = (kind, a, strat)
        for fut in as_completed(futs):
            row = fut.result()
            print(row, flush=True)
            f.write(json.dumps(row) + "\n")
            f.flush()
            results.append(row)
    json.dump(results, open(os.path.join(ROOT, "bench", "walnut_strategies_results.json"), "w"), indent=1)


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "both")
