"""Antichain evaluation of closed sentences (engine/src/antichain.rs, AM_ANTICHAIN=1)
against the default path, on bench/panel.json.

Two suites of CLOSED sentences (the only shape the antichain path ever fires on):

  direct   the closed '?' scripts of the GUI library (gui/serve.py), which write the
           equality-of-factors predicate out inline, so their cost is dominated by the
           INNER quantifier and the outer block is cheap either way.
  fe       `learnfe FE` first, then the same statements phrased over $FE -- now the
           outermost block IS the work, and it is what the antichain replaces.

Each row is run twice, in two separate engine processes (same binary, the flag off and
on), one query per process, with `mem` before and after the query so the allocator
high-water mark is attributable.  Verdicts must match; bench/antichain_results.json
holds the raw rows.

Run:
    python3 bench/antichain_bench.py fe          # the suite that moves
    python3 bench/antichain_bench.py direct
    python3 bench/antichain_bench.py both
"""
import os, sys, json, time
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "explore"))
from engine import run, pool                                       # noqa: E402

PANEL = dict(json.load(open(os.path.join(ROOT, "bench", "panel.json"))))

PAL = "(A t,u. t+u+1 = {n} => T[{i}+t] = T[{i}+u])"
BORD = ("(E b,j. b >= 1 & b < {n} & j + b = {i} + {n} & "
        "(A t. t < b => T[{i}+t] = T[j+t]))")
RS = "let RS(i,n) E j. (A t. t < n => T[i+t] = T[j+t]) & T[i+n] != T[j+n]\n"
PELT = ("let factorEq(i,n,m) A t. t < i => T[n+t] = T[m+t]\n"
        "let isRS(i,n) E p,q. $factorEq(i,n,p) & $factorEq(i,n,q) & T[p+i] != T[q+i]\n"
        "let extRS2(i,j,n) i<j & (E m1,m2. $isRS(j,m1) & $isRS(j,m2) & $factorEq(i,m1,m2) "
        "& $factorEq(i,n,m1) & T[m1+i] != T[m2+i])\n")

# ------------------------------------------------------------------ direct suite
QUERIES = [
    ("cube-free",     "", "? ~ E i,n. n>=1 & (A t. t < 2*n => T[i+t] = T[i+n+t])"),
    ("overlap-free",  "", "? ~ E i,n. n>=1 & (A t. t <= n => T[i+t] = T[i+n+t])"),
    ("4-power-free",  "", "? ~ E i,n. n>=1 & (A t. t < 3*n => T[i+t] = T[i+n+t])"),
    ("crit-7/3",      "", "? E i,n,L. n>=1 & 3*L >= 7*n & (A t. t+n < L => T[i+t] = T[i+n+t])"),
    ("crit-3/1",      "", "? E i,n,L. n>=1 & 1*L >= 3*n & (A t. t+n < L => T[i+t] = T[i+n+t])"),
    ("has-pal",       "", "? E i,n. n>=3 & " + PAL.format(i="i", n="n")),
    ("arb-pal",       "", "? A n. E i,m. m >= n & " + PAL.format(i="i", n="m")),
    ("unbordered",    "", "? A n. n>=1 => E i. ~" + BORD.format(i="i", n="n")),
    ("rs-count",      RS, "? A n. E i. $RS(i,n)"),
    ("recurrent",     "", "? A i,n,N. E j. j >= N & (A t. t < n => T[i+t] = T[j+t])"),
    ("mirror",        "", "? A i,n. E j. (A t,u. t+u+1 = n => T[i+t] = T[j+u])"),
    ("ap3",           "", "? A n. E i. (A t. t < n => T[i+3*t] = 0)"),
    ("peltomaki",  PELT,  "? E i,j,n. $extRS2(i,j,n)"),
]

# ------------------------------------------------------------------ FE suite
# Run after `learnfe FE`, so FE(i,j,l) ("the length-l factors at i and j are equal") is
# a ready-made 3-track automaton and every remaining quantifier is an outer one.
LADDER = [(2, 1), (7, 3), (5, 2), (8, 3), (3, 1), (7, 2), (4, 1)]
FE_QUERIES = [
    ("fe-cube",       "? E i,n. n>=1 & $FE(i,i+n,2*n)"),
    ("fe-4power",     "? E i,n. n>=1 & $FE(i,i+n,3*n)"),
    ("fe-recur-N",    "? A i,n,N. E j. j >= N & $FE(i,j,n)"),
    ("fe-recur",      "? A i,n. E j. j > i & $FE(i,j,n)"),
    ("fe-repeat",     "? A n. E i,j. i<j & $FE(i,j,n)"),
    ("fe-unique",     "? A n. E i. A j. $FE(i,j,n) => j=i"),
    ("fe-rext",       "? A i,n. E j. i<j & $FE(i,j,n) & ~$FE(i,j,n+1)"),
] + [(f"fe-crit-{a}/{b}",
      f"? E i,n,L. n>=1 & {b}*L + {b}*n >= {a}*n & $FE(i,i+n,L)") for a, b in LADDER]


def parse(out):
    verdict, ms, states = "?", -1, -1
    mems = []
    for l in out.split("\n"):
        if l.startswith("TRUE") or l.startswith("FALSE"):
            f = l.split()
            verdict = f[0]
            for t in f:
                if t.startswith("ms="): ms = int(t[3:])
                if t.startswith("states="): states = int(t[7:])
        elif l.startswith("OPEN") or l.startswith("ERR"):
            verdict = l.split()[0] if verdict == "?" else verdict
        elif l.startswith("OK mem"):
            mems.append(int(l.split("peak=")[1].split("MB")[0]))
    return verdict, ms, states, mems


def job(a):
    suite, seq, name, script, ac, timeout, mem = a
    pre = "learnfe FE\n" if suite == "fe" else ""
    src = f"mode msd\n{PANEL[seq]}\n{pre}mem\n{script}\nmem\n"
    t0 = time.time()
    r = run(src, timeout=timeout, mem_mb=mem, env={"AM_ANTICHAIN": "1"} if ac else None)
    v, ms, st, mems = parse(r.stdout)
    if r.timed_out: v = "TIMEOUT"
    elif r.budget: v = "BUDGET"
    return dict(suite=suite, seq=seq, q=name, ac=ac, verdict=v, ms=ms, states=st,
                mem_pre=mems[0] if mems else -1, mem_post=mems[1] if len(mems) > 1 else -1,
                secs=round(time.time() - t0, 2))


def build(suites, seqs):
    """AC_QUERIES=name,name restricts to those query ids (for serial re-measurement
    of the rows that move, without the machine running a dozen engines at once)."""
    only = os.environ.get("AC_QUERIES")
    only = set(only.split(",")) if only else None
    jobs = []
    for s in seqs:
        if "direct" in suites:
            for n, pre, q in QUERIES:
                if only is None or n in only: jobs.append(("direct", s, n, pre + q))
        if "fe" in suites:
            for n, q in FE_QUERIES:
                if only is None or n in only: jobs.append(("fe", s, n, q))
    return jobs


def main():
    which = sys.argv[1] if len(sys.argv) > 1 else "both"
    suites = ["direct", "fe"] if which == "both" else [which]
    seqs = sys.argv[2].split(",") if len(sys.argv) > 2 else list(PANEL)
    seqs = [s for s in seqs if s in PANEL]
    timeout = int(os.environ.get("AC_TIMEOUT", "300"))
    mem = int(os.environ.get("AC_MEM", "6144"))
    jobs = [(su, s, n, q, ac, timeout, mem)
            for (su, s, n, q) in build(suites, seqs) for ac in (False, True)]
    res = pool(jobs, job, label="antichain")
    by = {}
    for r in res:
        by.setdefault((r["suite"], r["seq"], r["q"]), {})[r["ac"]] = r
    bad, incomparable = [], []
    print(f"{'suite':7s} {'sequence':16s} {'query':14s} "
          f"{'base ms':>9s} {'ac ms':>8s} {'base MB':>8s} {'ac MB':>7s} "
          f"{'base s':>7s} {'ac s':>6s}  verdict")
    for (su, s, q), d in sorted(by.items()):
        b, a = d.get(False), d.get(True)
        if not b or not a: continue
        # a run with no verdict line was cut short (budget, kill); it is not a
        # disagreement, it is a missing measurement
        if b["verdict"] not in ("TRUE", "FALSE") or a["verdict"] not in ("TRUE", "FALSE"):
            incomparable.append((su, s, q, b["verdict"], a["verdict"]))
        elif b["verdict"] != a["verdict"]:
            bad.append((su, s, q, b["verdict"], a["verdict"]))
        print(f"{su:7s} {s:16s} {q:14s} {b['ms']:9d} {a['ms']:8d} "
              f"{b['mem_post']:8d} {a['mem_post']:7d} {b['secs']:7.1f} {a['secs']:6.1f}  "
              f"{b['verdict']}{'' if b['verdict'] == a['verdict'] else ' / ' + a['verdict']}")
    print()
    print("DISAGREEMENTS: " + (str(bad) if bad else "none"))
    print("INCOMPARABLE (a side produced no verdict): " +
          (str(incomparable) if incomparable else "none"))
    print("note: `base s` / `ac s` are whole-process wall times and include the "
          "`learnfe FE` that precedes every fe-suite query; `ms` is the query alone.")
    json.dump(res, open(os.path.join(ROOT, "bench", "antichain_results.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
