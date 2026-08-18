"""Benchmark + differential harness for the SYMBOLIC (BDD/MTKDD) strategy.

Owned by the symbolic-strategy work (engine/src/symbolic.rs).  Runs a fixed case
list through the engine twice -- once with the default explicit ladder, once with
`AM_STRATEGY=bdd` -- and records, per case: wall seconds, the minimal state count
of every predicate built, the engine's own peak live-bytes figure (`mem`), and a
canonical fingerprint of every exported automaton so that "same answer" means
"same language", not just "same number of states".

Canonical fingerprint: the exported DFA is relabelled by a breadth-first walk from
state 0 taking symbols in index order, which is a canonical form for a *minimal*
DFA -- two minimal DFAs have equal fingerprints iff they accept the same language.

Usage:
    python3 bench/bdd_bench.py base   out.json [caseset]   # explicit ladder
    python3 bench/bdd_bench.py bdd    out.json [caseset]   # AM_STRATEGY=bdd
    python3 bench/bdd_bench.py cmp    a.json b.json        # compare two runs
caseset in {all, panel, fe2, pelt, trib, many, quick}.
"""
import os, sys, json, time, hashlib

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "explore"))
import engine

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PANEL = json.load(open(os.path.join(ROOT, "bench", "panel.json")))

FE   = "let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]"
FE2  = "let FE2(i,j,n) A u,v. (u>=i & u<i+n & u+j=v+i) => T[u]=T[v]"
PELT = ["let factorEq(i,n,m) A t. t<i => T[n+t]=T[m+t]",
        "let isRS(i,n) E p,q. $factorEq(i,n,p) & $factorEq(i,n,q) & T[p+i]!=T[q+i]",
        "let extRS2(i,j,n) i<j & (E m1,m2. $isRS(j,m1) & $isRS(j,m2) & $factorEq(i,m1,m2) & $factorEq(i,n,m1) & T[m1+i]!=T[m2+i])",
        "let final(i,j) E n. $extRS2(i,j,n)"]

# three-free-variable queries over a base-k sequence: large product alphabets
MANY = ["let SQ(i,n) (n>=1) & A t. t<n => T[i+t]=T[i+n+t]",
        "let CU(i,n) (n>=1) & A t. t<2*n => T[i+t]=T[i+n+t]",
        "let OCC(i,j,n) A t. (t<n) => (T[i+t]=T[j+t])",
        "let RSP(i,n) E j. (A t. t<n => T[i+t]=T[j+t]) & T[i+n]!=T[j+n]",
        "let PAL(i,n) A t,u. ((t<n) & (t+u+1=n)) => T[i+t]=T[i+u]"]

TRIB = ["numsys trib",
        "dfao TR 3 @" + os.path.join(ROOT, "walnut7", "Word Automata Library", "TR.txt")]


def panel_def(name):
    for n, d in PANEL:
        if n == name: return d
    raise KeyError(name)


def cases(which):
    out = []
    panel_names = [n for n, _ in PANEL]
    if which == "recon":
        for n in ["single3", "single4", "k3m3-artefact-a", "prism-d"]:
            out.append(("FE/" + n, [panel_def(n), FE], ["FE"]))
        for n in ["cantor", "mephisto", "prism-a", "prism-d", "single3", "single4",
                  "k3m3-artefact-a", "k3m3-artefact-b", "tail-b"]:
            out.append(("FE2/" + n, [panel_def(n), FE2], ["FE2"]))
        for n in ["cantor", "paperfolding", "rudin-shapiro", "prism-d", "mephisto"]:
            out.append(("pelt/" + n, [panel_def(n)] + PELT, ["factorEq", "isRS", "extRS2", "final"]))
        for n in ["cantor", "prism-d", "single3", "tail-b", "prism-1"]:
            for q in MANY:
                nm = q.split()[1].split("(")[0]
                out.append(("%s/%s" % (nm, n), [panel_def(n), q], [nm]))
        return out
    if which in ("all", "panel", "quick"):
        names = panel_names if which != "quick" else ["thue-morse", "cantor", "prism-a", "single3"]
        for n in names:
            out.append(("FE/" + n, [panel_def(n), FE], ["FE"]))
    if which in ("all", "fe2", "quick"):
        names = (["thue-morse", "period-doubling", "rudin-shapiro", "paperfolding", "cantor",
                  "mephisto", "prism-a", "prism-d", "single3", "single4", "k3m3-artefact-a",
                  "k3m3-artefact-b", "champion-m5"] if which != "quick" else ["thue-morse", "cantor"])
        for n in names:
            out.append(("FE2/" + n, [panel_def(n), FE2], ["FE2"]))
    if which in ("all", "pelt", "quick"):
        names = (["thue-morse", "period-doubling", "cantor", "paperfolding", "rudin-shapiro",
                  "prism-d", "mephisto", "single3"] if which != "quick" else ["thue-morse"])
        for n in names:
            out.append(("pelt/" + n, [panel_def(n)] + PELT, ["factorEq", "isRS", "extRS2", "final"]))
    if which in ("all", "many", "quick"):
        names = (["thue-morse", "cantor", "prism-d", "single3", "prism-1",
                  "k3m3-artefact-a", "champion-m5"] if which != "quick" else ["thue-morse"])
        for n in names:
            for q in MANY:
                nm = q.split()[1].split("(")[0]
                out.append(("%s/%s" % (nm, n), [panel_def(n), q], [nm]))
    return out


def canon(ex):
    """Canonical fingerprint of an exported DFA (BFS relabel from state 0)."""
    if ex.get("kind") != "dfa": return None
    if ex.get("truncated"): return "TRUNCATED"
    tr, acc = ex["trans"], set(ex["accepting"])
    order, seen = [0], {0: 0}
    i = 0
    while i < len(order):
        s = order[i]
        for d in tr[s]:
            if d not in seen:
                seen[d] = len(order); order.append(d)
        i += 1
    rows = []
    for s in order:
        rows.append((1 if s in acc else 0, tuple(seen[d] for d in tr[s])))
    h = hashlib.sha256(repr((ex["k"], ex["vars"], ex["alpha"], rows)).encode()).hexdigest()[:16]
    return "%d:%s" % (len(order), h)


def run_case(name, lines, exports, tag, timeout, mem_mb, binary=None, extra_env=None):
    src = "mode msd\n" + "\n".join(lines) + "\n"
    for e in exports: src += "export %s\n" % e
    src += "mem\n"
    if binary: engine.ENGINE = binary
    env = {"AM_EXPORT_MAX": "200000"}
    if extra_env: env.update(extra_env)
    t0 = time.time()
    r = engine.run(src, mem_mb=mem_mb, timeout=timeout, env=env)
    secs = time.time() - t0
    rec = {"case": name, "tag": tag, "secs": round(secs, 2), "rc": r.rc,
           "timed_out": r.timed_out, "budget": r.budget, "states": {}, "ms": {}, "canon": {}}
    for l in r.stdout.split("\n"):
        if l.startswith("OK let ") or l.startswith("OK learnfe "):
            head = l.split()[2]
            nm = head.split("(")[0]
            for tok in l.split():
                if tok.startswith("states="): rec["states"][nm] = int(tok[7:])
                if tok.startswith("ms="): rec["ms"][nm] = int(tok[3:])
        if l.startswith("EXPORT "):
            try:
                ex = json.loads(l[7:])
                rec["canon"][ex["name"]] = canon(ex)
            except Exception as e:
                rec["canon"]["?"] = "parse-error " + str(e)
        if l.startswith("OK mem "):
            rec["peak_mb"] = int(l.split("peak=")[1].split("MB")[0])
        if l.startswith("ERR"):
            rec.setdefault("errs", []).append(l)
    if r.stderr and "bdd" in tag:
        rec["bdd_note"] = [x for x in r.stderr.split("\n") if "symbolic" in x][:8]
    return rec


def main():
    mode = sys.argv[1]
    if mode == "cmp":
        a = {r["case"]: r for r in json.load(open(sys.argv[2]))}
        b = {r["case"]: r for r in json.load(open(sys.argv[3]))}
        bad = 0
        print("%-26s %10s %10s %8s  %s" % ("case", "A secs", "B secs", "A/B", "states / canon"))
        for c in a:
            if c not in b: continue
            x, y = a[c], b[c]
            same = x["states"] == y["states"] and x["canon"] == y["canon"]
            if not same: bad += 1
            r = (x["secs"] / y["secs"]) if y["secs"] else 0
            print("%-26s %10.2f %10.2f %8.2f  %s %s" % (
                c, x["secs"], y["secs"], r, "OK" if same else "MISMATCH",
                "" if same else "%s vs %s / %s vs %s" % (x["states"], y["states"], x["canon"], y["canon"])))
        print("mismatches:", bad)
        return
    if mode == "abc":
        # Interleaved: every case is run under all three configurations back to back, so
        # a busy machine costs all three the same.  Ratios from this file are fair even
        # when the absolute seconds are inflated by other load.
        out_path = sys.argv[2]
        which = sys.argv[3] if len(sys.argv) > 3 else "all"
        timeout = int(os.environ.get("BDD_TIMEOUT", "900"))
        mem_mb = int(os.environ.get("BDD_MEM", "6144"))
        binary = os.environ.get("BDD_BIN")
        recs = []
        for name, lines, exports in cases(which):
            row = {"case": name}
            for tag, env in (("base", None), ("bdd", {"AM_STRATEGY": "bdd"}),
                             ("auto", {"AM_STRATEGY": "auto"})):
                r = run_case(name, lines, exports, tag, timeout, mem_mb, binary, env)
                row[tag] = r
            row["same"] = (row["base"]["states"] == row["bdd"]["states"] == row["auto"]["states"]
                           and row["base"]["canon"] == row["bdd"]["canon"] == row["auto"]["canon"])
            recs.append(row)
            print(json.dumps(row), flush=True)
            json.dump(recs, open(out_path, "w"), indent=1)
        return
    if mode == "table":
        # The engine's own `ms=` (summed over the predicates a case builds) is the
        # timing of record: wall `secs` also contains the runner's RAM-admission wait,
        # which on a machine running other jobs can dominate.
        recs = json.load(open(sys.argv[2]))
        print("%-24s %9s %9s %9s %7s %7s   %8s %8s %8s  %s" % (
            "case", "base s", "bdd s", "auto s", "b/bdd", "b/auto",
            "base MB", "bdd MB", "auto MB", "states"))
        for r in recs:
            g = lambda t, f, d=0: r[t].get(f, d)
            ms = lambda t: sum(r[t].get("ms", {}).values()) / 1000.0 or g(t, "secs")
            rb = (ms("base") / ms("bdd")) if ms("bdd") else 0
            ra = (ms("base") / ms("auto")) if ms("auto") else 0
            st = ",".join("%s=%s" % (k, v) for k, v in sorted(g("base", "states", {}).items()))
            print("%-24s %9.2f %9.2f %9.2f %7.2f %7.2f   %8s %8s %8s  %s%s" % (
                r["case"], ms("base"), ms("bdd"), ms("auto"), rb, ra,
                g("base", "peak_mb", "-"), g("bdd", "peak_mb", "-"), g("auto", "peak_mb", "-"),
                st, "" if r["same"] else "   <<< MISMATCH"))
        print("mismatches:", sum(0 if r["same"] else 1 for r in recs), "of", len(recs))
        return
    out_path = sys.argv[2]
    which = sys.argv[3] if len(sys.argv) > 3 else "all"
    timeout = int(os.environ.get("BDD_TIMEOUT", "900"))
    mem_mb = int(os.environ.get("BDD_MEM", "6144"))
    binary = os.environ.get("BDD_BIN")
    env = {"AM_STRATEGY": mode} if mode in ("bdd", "auto") else None
    recs = []
    for name, lines, exports in cases(which):
        rec = run_case(name, lines, exports, mode, timeout, mem_mb, binary, env)
        recs.append(rec)
        print(json.dumps(rec), flush=True)
        json.dump(recs, open(out_path, "w"), indent=1)


if __name__ == "__main__":
    main()
