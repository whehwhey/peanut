#!/usr/bin/env python3
"""Final-defaults benchmark: the 2026-08-19 defaults against the pre-2026-08-19 ones.

Three configurations of ONE binary (engine/target/release/peanut), run back to back
per case, one engine process at a time:

    new    no environment at all       -- AM_PAR = min(8, cores-2), antichain ON
    old    AM_PAR=1 AM_ANTICHAIN=0     -- exactly the pre-2026-08-19 default path
    auto   AM_STRATEGY=auto            -- new defaults plus the symbolic rung
                                         (the configuration this round decides on)

Recorded per (case, config): the engine's own `ms=` for the query, the engine's own
allocator high-water mark from `mem`, and the minimal state count -- which must be
identical across configurations or the run is a blocker.

Reads : bench/panel.json.   Writes: results/defaults_bench.json.

Run:
    python3 bench/defaults_bench.py fe      results/defaults_bench.json   # FE panel
    python3 bench/defaults_bench.py closed  results/defaults_closed.json  # A..E shapes
    python3 bench/defaults_bench.py trib    results/defaults_trib.json    # Tribonacci
    python3 bench/defaults_bench.py table   results/defaults_bench.json
"""
import json, os, re, sys, time

ROOT = "/Users/andrew/maths"
sys.path.insert(0, os.path.join(ROOT, "explore"))
import engine                                              # noqa: E402

PANEL = json.load(open(os.path.join(ROOT, "bench/panel.json")))
SEQ = dict(PANEL)

CONFIGS = [("new", {}),
           ("old", {"AM_PAR": "1", "AM_ANTICHAIN": "0"}),
           ("auto", {"AM_STRATEGY": "auto"})]

FE = "let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]"
LET_LINE = re.compile(r"^OK let (\w+)\(.*?\) states=(\d+) peak=(\d+) ms=(\d+)")
LEARN_LINE = re.compile(r"^OK learn(?:fe)? (\w+)\(.*?\) states=(\d+).*?ms=(\d+)")
Q_LINE = re.compile(r"^(TRUE|FALSE) states=(\d+) peak=(\d+) ms=(\d+)")
MEM_LINE = re.compile(r"^OK mem live=(\d+)MB peak=(\d+)MB")


def parse(r):
    """-> dict(states, ms, mb, verdict, ok)"""
    out = {"states": None, "ms": None, "mb": None, "verdict": None, "ok": False}
    for line in r.stdout.split("\n"):
        m = LET_LINE.match(line) or LEARN_LINE.match(line)
        if m:
            g = m.groups()
            out["states"] = int(g[1]); out["ms"] = int(g[-1]); out["ok"] = True
        m = Q_LINE.match(line)
        if m:
            out["verdict"] = m.group(1); out["states"] = int(m.group(2))
            out["ms"] = int(m.group(4)); out["ok"] = True
        m = MEM_LINE.match(line)
        if m:
            out["mb"] = int(m.group(2))
    if not out["ok"]:
        out["tail"] = (r.stdout[-300:] + " | " + r.stderr[-300:]).strip()
        out["timeout"] = r.timed_out
        out["budget"] = getattr(r, "budget", None)
    return out


def one(src, env, timeout, mem_mb=6144):
    t0 = time.time()
    r = engine.run(src, timeout=timeout, mem_mb=mem_mb, env=env)
    d = parse(r)
    d["wall"] = round(time.time() - t0, 2)
    return d


# ------------------------------------------------------------------ suites

def suite_fe(cases, timeout):
    rows = []
    for name in cases:
        defn = SEQ[name]
        for label, env in CONFIGS:
            src = "mode msd\n%s\n%s\nmem\n" % (defn, FE)
            d = one(src, env, timeout)
            d.update(case=name, config=label, query="let FE")
            rows.append(d)
            print("[fe] %-18s %-5s %s" % (name, label, fmt(d)), flush=True)
    return rows


def suite_learnfe(cases, timeout):
    rows = []
    for name in cases:
        defn = SEQ[name]
        for label, env in CONFIGS:
            src = "mode msd\n%s\nlearnfe FE\nmem\n" % defn
            d = one(src, env, timeout)
            d.update(case=name, config=label, query="learnfe FE")
            rows.append(d)
            print("[learnfe] %-14s %-5s %s" % (name, label, fmt(d)), flush=True)
    return rows


CLOSED = [("fe-recur-N", "? A i,n,N. E j. j >= N & $FE(i,j,n)"),
          ("fe-recur",   "? A i,n. E j. j > i & $FE(i,j,n)"),
          ("fe-samelen", "? A n. E i,j. i<j & $FE(i,j,n)"),
          ("fe-cube",    "? E i,n. n>=1 & $FE(i,i+n,2*n)")]


def suite_closed(cases, timeout):
    rows = []
    for name in cases:
        defn = SEQ[name]
        for qname, q in CLOSED:
            for label, env in CONFIGS:
                src = "mode msd\n%s\nlearnfe FE\n%s\nmem\n" % (defn, q)
                d = one(src, env, timeout)
                d.update(case=name, config=label, query=qname)
                rows.append(d)
                print("[closed] %-12s %-11s %-5s %s" % (name, qname, label, fmt(d)),
                      flush=True)
    return rows


def suite_trib(timeout):
    rows = []
    head = "numsys trib\ndfao TR 2 0:0,1 1:0,2 2:0,-\n"
    for qname, q in [("let FE", FE),
                     ("learnfe FE", "learnfe FE")]:
        for label, env in CONFIGS:
            src = head + q + "\nmem\n"
            d = one(src, env, timeout)
            d.update(case="tribonacci", config=label, query=qname)
            rows.append(d)
            print("[trib] %-11s %-5s %s" % (qname, label, fmt(d)), flush=True)
    return rows


ATTRIB_CONFIGS = [("par1+ac0", {"AM_PAR": "1", "AM_ANTICHAIN": "0"}),
                  ("par1+ac1", {"AM_PAR": "1"}),
                  ("par8+ac0", {"AM_ANTICHAIN": "0"}),
                  ("par8+ac1", {})]


def suite_attrib(timeout):
    """Which of the two new defaults owns a regression? 2x2 over AM_PAR / AM_ANTICHAIN."""
    rows = []
    cases = [("prism-1", "fe-cube"), ("prism-1", "fe-recur"), ("prism-1", "fe-recur-N"),
             ("tail-a", "fe-cube"), ("tail-b", "fe-cube"), ("single4", "fe-cube")]
    qs = dict(CLOSED)
    for rep in range(3):
        for name, qname in cases:
            for label, env in ATTRIB_CONFIGS:
                src = "mode msd\n%s\nlearnfe FE\n%s\nmem\n" % (SEQ[name], qs[qname])
                d = one(src, env, timeout)
                d.update(case=name, config=label, query=qname, rep=rep)
                rows.append(d)
                print("[attrib] %-9s %-11s %-9s rep%d %s" % (name, qname, label, rep, fmt(d)),
                      flush=True)
    return rows


def fmt(d):
    if not d["ok"]:
        return "NO ANSWER (%s) wall=%ss" % ("timeout" if d.get("timeout") else "budget/err",
                                            d["wall"])
    return "states=%s ms=%s MB=%s%s" % (d["states"], d["ms"], d["mb"],
                                        "" if d["verdict"] is None else " " + d["verdict"])


HARD = ["prism-1", "single3", "single4", "single5", "single6", "tail-a", "tail-b", "tail-c"]
EASY = [n for n, _ in PANEL if n not in HARD]


def main():
    what = sys.argv[1] if len(sys.argv) > 1 else "fe"
    out = sys.argv[2] if len(sys.argv) > 2 else "results/defaults_%s.json" % what
    out = out if os.path.isabs(out) else os.path.join(ROOT, out)
    if what == "table":
        table(json.load(open(out)))
        return
    rows = []
    if what == "fe":
        rows = suite_fe(HARD, 900)
    elif what == "easy":
        rows = suite_fe(EASY, 300)
    elif what == "learnfe":
        rows = suite_learnfe(["tail-c"], 300)
    elif what == "closed":
        rows = suite_closed(["single4", "single5", "tail-a", "tail-b", "tail-c", "prism-1"], 600)
    elif what == "trib":
        rows = suite_trib(600)
    elif what == "attrib":
        rows = suite_attrib(600)
    else:
        print(__doc__); sys.exit(2)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    json.dump(rows, open(out, "w"), indent=1)
    print("wrote", out)
    table(rows)


def table(rows):
    keys = []
    for r in rows:
        k = (r["case"], r["query"])
        if k not in keys: keys.append(k)
    print("%-18s %-11s %10s %10s %10s   %s" % ("case", "query", "old s/MB", "new s/MB",
                                               "auto s/MB", "states"))
    for k in keys:
        cell = {r["config"]: r for r in rows if (r["case"], r["query"]) == k}
        def c(x):
            r = cell.get(x)
            if r is None: return "-"
            if not r["ok"]: return "FAIL"
            return "%.3g/%s" % (r["ms"] / 1000.0, r["mb"])
        st = {r["states"] for r in cell.values() if r["ok"]}
        print("%-18s %-11s %10s %10s %10s   %s" % (k[0], k[1], c("old"), c("new"),
                                                   c("auto"), st))


if __name__ == "__main__":
    main()
