"""Bench and cross-check for the generalised `learn` command (docs/LEARN.md).

Subcommands
    panel    every panel sequence x {fe,rev,period,border}: learned vs the direct
             `let` construction -- state counts, times, peak MB, and the engine's own
             `A vars. $L(..) <=> $D(..)` equivalence check
    trib     the same four classes on Tribonacci (`numsys trib`) and Fibonacci
    fuzz     N random closed formulas over the learned predicate vs the same formula
             over the direct predicate: verdicts must be identical
    brute    pure-Python brute force over a morphism prefix vs `enum B $L(..)`
    ladder   end-to-end: palindromes-of-every-length and the critical-exponent ladder,
             direct predicates vs learned ones

Writes results/learn_*.json.  Run:  python3 explore/learn_bench.py panel
"""
import sys, os, json, time, random, itertools
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import engine

# Set AM_ENGINE to benchmark a specific engine binary (e.g. a fixed build) instead
# of the default engine/target/release/peanut.
if os.environ.get("AM_ENGINE"):
    engine.ENGINE = os.environ["AM_ENGINE"]

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PANEL = json.load(open(os.path.join(ROOT, "bench/panel.json")))
RES = os.path.join(ROOT, "results")

# kind -> (params, direct `let` body).  These are the textbook definitions, written
# with an extra bounded variable wherever the natural form would need a subtraction
# (the compiler refuses negative indices).
KINDS = {
    "fe":     (("i", "j", "l"), "A t. t<l => T[i+t]=T[j+t]"),
    "rev":    (("i", "j", "l"), "A t,u. (t<l & t+u+1=l) => T[i+t]=T[j+u]"),
    "period": (("i", "l", "p"), "A t. t+p<l => T[i+t]=T[i+t+p]"),
    "border": (("i", "l", "b"), "(b<=l) & (A t,u. (t<b & u+b=l) => T[i+t]=T[i+u+t])"),
}

TRIB = "numsys trib\ndfao TR 2 0:0,1 1:0,2 2:0,-"
FIB  = "numsys fib\ndfao F 2 0:0,1 1:0,-"


def field(line, key, cast=int):
    for tok in line.split():
        if tok.startswith(key + "="):
            try: return cast(tok.split("=", 1)[1])
            except ValueError: return None
    return None


def one(prefix, script, timeout, mem):
    r = engine.run(prefix + "\n" + script + "\nmem\n", timeout=timeout, mem_mb=mem)
    return r


def learn_run(prefix, kind, timeout=600, mem=4096):
    """`learn L <kind>` alone: states, ms, peak MB."""
    r = one(prefix, "learn L %s" % kind, timeout, mem)
    ok = [l for l in r.stdout.split("\n") if l.startswith("OK learn ")]
    mem_l = [l for l in r.stdout.split("\n") if l.startswith("OK mem")]
    if not ok:
        return {"states": None, "ms": None, "peak_mb": None,
                "err": ("timeout" if r.timed_out else
                        "budget" if r.budget else
                        next((l for l in r.stdout.split("\n") if l.startswith("ERR")), "rc=%s" % r.rc))}
    return {"states": field(ok[0], "states"), "ms": field(ok[0], "ms"),
            "eqs": field(ok[0], "eqs"), "mqs": field(ok[0], "mqs"),
            "peak_mb": field(mem_l[0], "peak", lambda x: int(x.rstrip("MB"))) if mem_l else None,
            "secs": round(r.secs, 2)}


def direct_run(prefix, kind, timeout=600, mem=4096):
    """`let D(..) <body>` alone: states, ms, peak MB."""
    params, body = KINDS[kind]
    r = one(prefix, "let D(%s) %s" % (",".join(params), body), timeout, mem)
    ok = [l for l in r.stdout.split("\n") if l.startswith("OK let ")]
    mem_l = [l for l in r.stdout.split("\n") if l.startswith("OK mem")]
    if not ok:
        return {"states": None, "ms": None, "peak_mb": None,
                "err": ("timeout" if r.timed_out else "budget" if r.budget else
                        next((l for l in r.stdout.split("\n") if l.startswith("ERR")), "rc=%s" % r.rc))}
    return {"states": field(ok[0], "states"), "ms": field(ok[0], "ms"),
            "peak_mb": field(mem_l[0], "peak", lambda x: int(x.rstrip("MB"))) if mem_l else None,
            "secs": round(r.secs, 2)}


def equiv_run(prefix, kind, timeout=900, mem=4096):
    """Both, plus the engine's own proof that they are the same predicate."""
    params, body = KINDS[kind]
    v = ",".join(params)
    script = ("learn L %s\nlet D(%s) %s\n? A %s. $L(%s) <=> $D(%s)" %
              (kind, v, body, v, v, v))
    r = one(prefix, script, timeout, mem)
    verdict = next((l.split()[0] for l in r.stdout.split("\n")
                    if l.startswith(("TRUE", "FALSE"))), None)
    return verdict


LEARN_TIMEOUT = int(os.environ.get("AM_LEARN_TIMEOUT", "600"))
DIRECT_TIMEOUT = int(os.environ.get("AM_DIRECT_TIMEOUT", "240"))


def cmd_panel(argv):
    kinds = argv or list(KINDS)
    jobs = [(name, defline, k) for name, defline in PANEL for k in kinds]
    lock = __import__("threading").Lock()
    done = [0]

    def fn(job):
        name, defline, k = job
        out = {"seq": name, "kind": k}
        out["learn"] = learn_run(defline, k, timeout=LEARN_TIMEOUT, mem=3072)
        out["direct"] = direct_run(defline, k, timeout=DIRECT_TIMEOUT, mem=4096)
        if out["learn"]["states"] and out["direct"]["states"]:
            out["same_size"] = out["learn"]["states"] == out["direct"]["states"]
            out["equiv"] = equiv_run(defline, k,
                                     timeout=LEARN_TIMEOUT + DIRECT_TIMEOUT, mem=4096)
        with lock:
            done[0] += 1
            print("[%2d/%d] %-16s %-7s learn=%-8s direct=%-10s %s" % (
                done[0], len(jobs), name, k,
                out["learn"]["states"] or out["learn"].get("err"),
                out["direct"]["states"] or out["direct"].get("err"),
                out.get("equiv") or ""), flush=True)
        return out

    res = engine.pool(jobs, fn, label="learn panel")
    path = os.path.join(RES, "learn_panel.json")
    json.dump(res, open(path, "w"), indent=1)
    print(json.dumps(res, indent=1)[:200], "...")
    hdr = "%-16s %-7s %8s %8s %8s %9s %8s %6s" % (
        "sequence", "kind", "learn", "direct", "same", "learn ms", "let ms", "equiv")
    print(hdr); print("-" * len(hdr))
    bad = 0
    for r in res:
        ls, ds = r["learn"]["states"], r["direct"]["states"]
        same = "-" if not (ls and ds) else ("yes" if r.get("same_size") else "NO")
        if ls and ds and not r.get("same_size"): bad += 1
        if r.get("equiv") not in (None, "TRUE"): bad += 1
        print("%-16s %-7s %8s %8s %8s %9s %8s %6s" % (
            r["seq"], r["kind"], ls or r["learn"].get("err"), ds or r["direct"].get("err"),
            same, r["learn"]["ms"], r["direct"]["ms"], r.get("equiv") or "-"))
    print("\nmismatches:", bad, " ->", path)


def cmd_trib(argv):
    rows = []
    for label, prefix in (("tribonacci", TRIB), ("fibonacci", FIB)):
        for k in (argv or list(KINDS)):
            row = {"seq": label, "kind": k,
                   "learn": learn_run(prefix, k, timeout=900, mem=6144),
                   "direct": direct_run(prefix, k, timeout=900, mem=6144)}
            if row["learn"]["states"] and row["direct"]["states"]:
                row["same_size"] = row["learn"]["states"] == row["direct"]["states"]
                row["equiv"] = equiv_run(prefix, k, timeout=900, mem=6144)
            rows.append(row)
            print("%-11s %-7s learn=%-6s direct=%-8s same=%-5s learn_ms=%-7s let_ms=%-8s %s" % (
                label, k, row["learn"]["states"] or row["learn"].get("err"),
                row["direct"]["states"] or row["direct"].get("err"),
                row.get("same_size"), row["learn"]["ms"], row["direct"]["ms"],
                row.get("equiv") or ""), flush=True)
    path = os.path.join(RES, "learn_numsys.json")
    json.dump(rows, open(path, "w"), indent=1)
    print("->", path)


# ---------------------------------------------------------------- fuzz

TEMPLATES = [
    "A {v0},{v1}. ({v0}<{B} & {v1}<{B}) => ($P({a}) => $P({b}))",
    "E {v0},{v1}. {v0}<{B} & {v1}<{B} & $P({a})",
    "A {v0}. {v0}<{B} => (E {v1}. {v1}<{B} & $P({a}))",
    "E {v0}. {v0}<{B} & (A {v1}. {v1}<{B} => ~$P({a}))",
    "A {v0},{v1}. ({v0}<{B} & {v1}<{B} & $P({a})) => $P({b})",
]


def rand_formula(rng, kind):
    params, _ = KINDS[kind]
    B = rng.choice([4, 6, 8, 12])
    v0, v1 = "x", "y"
    def args():
        out = []
        for _ in params:
            c = rng.randrange(5)
            out.append(rng.choice([v0, v1, str(rng.randrange(B)),
                                   "%s+%d" % (rng.choice([v0, v1]), rng.randrange(4))])
                       if c else str(rng.randrange(B)))
        return ",".join(out)
    t = rng.choice(TEMPLATES)
    return t.format(v0=v0, v1=v1, B=B, a=args(), b=args())


def cmd_fuzz(argv):
    n = int(argv[0]) if argv else 200
    rng = random.Random(653658211)
    jobs = []
    for i in range(n):
        name, defline = rng.choice(PANEL)
        kind = rng.choice(list(KINDS))
        jobs.append((i, name, defline, kind, rand_formula(rng, kind)))

    def fn(job):
        i, name, defline, kind, f = job
        params, body = KINDS[kind]
        v = ",".join(params)
        sl = "learn P %s\n? %s" % (kind, f)
        sd = "let P(%s) %s\n? %s" % (v, body, f)
        rl = engine.run(defline + "\n" + sl + "\nquit\n", timeout=300, mem_mb=3072)
        rd = engine.run(defline + "\n" + sd + "\nquit\n", timeout=300, mem_mb=3072)
        vl = next((l.split()[0] for l in rl.stdout.split("\n") if l.startswith(("TRUE", "FALSE"))), None)
        vd = next((l.split()[0] for l in rd.stdout.split("\n") if l.startswith(("TRUE", "FALSE"))), None)
        return {"i": i, "seq": name, "kind": kind, "formula": f,
                "learned": vl, "direct": vd,
                "agree": (vl == vd) if (vl and vd) else None}

    res = engine.pool(jobs, fn, label="learn fuzz")
    agree = sum(1 for r in res if r["agree"] is True)
    dis = [r for r in res if r["agree"] is False]
    inc = [r for r in res if r["agree"] is None]
    path = os.path.join(RES, "learn_fuzz.json")
    json.dump(res, open(path, "w"), indent=1)
    print("agree %d / disagree %d / incomplete %d  (of %d)" % (agree, len(dis), len(inc), len(res)))
    for r in dis[:10]: print("  DISAGREE", r)
    for r in inc[:5]: print("  incomplete", r["seq"], r["kind"], r["formula"])
    print("->", path)


# ---------------------------------------------------------------- brute force

def morphism_prefix(defline, n):
    """The fixed point of the `def` line's morphism, as a list of coded letters.
    Written from the definition, sharing no code with the engine."""
    p = defline.split()
    k, m, start = int(p[2]), int(p[3]), int(p[4])
    words = [[int(c) for c in p[5 + a]] for a in range(m)]
    coding = [int(c) for c in p[5 + m]]
    s = [start]
    while len(s) < n:
        s = [b for a in s for b in words[a]]
    return [coding[a] for a in s[:n]]


def brute(kind, w, B):
    out = set()
    if kind == "fe":
        for i, j, l in itertools.product(range(B), range(B), range(B)):
            if all(w[i + t] == w[j + t] for t in range(l)): out.add((i, j, l))
    elif kind == "rev":
        for i, j, l in itertools.product(range(B), range(B), range(B)):
            if all(w[i + t] == w[j + l - 1 - t] for t in range(l)): out.add((i, j, l))
    elif kind == "period":
        for i, l, p in itertools.product(range(B), range(B), range(B)):
            if all(w[i + t] == w[i + t + p] for t in range(max(0, l - p))): out.add((i, l, p))
    elif kind == "border":
        for i, l, b in itertools.product(range(B), range(B), range(B)):
            if b <= l and all(w[i + t] == w[i + l - b + t] for t in range(b)): out.add((i, l, b))
    return out


def cmd_brute(argv):
    B = int(argv[0]) if argv else 12
    names = argv[1:] or ["thue-morse", "rudin-shapiro", "tail-b", "tail-c", "prism-1"]
    path0 = os.path.join(RES, "learn_brute.json")
    try: rows = [r for r in json.load(open(path0)) if not (r["seq"] in names and r.get("B") == B)]
    except Exception: rows = []
    for name, defline in PANEL:
        if name not in names: continue
        w = morphism_prefix(defline, 4 * B + 8)
        for kind in KINDS:
            params, _ = KINDS[kind]
            r = engine.run("%s\nlearn L %s\nenum %d $L(%s)\nquit\n" %
                           (defline, kind, B, ",".join(params)), timeout=900, mem_mb=4096)
            line = next((l for l in r.stdout.split("\n") if l.startswith("ENUM")), None)
            if line is None:
                rows.append({"seq": name, "kind": kind, "B": B, "ok": None,
                             "err": next((l for l in r.stdout.split("\n") if l.startswith("ERR")), "timeout")})
                print("  %-14s %-7s NO RESULT" % (name, kind), flush=True)
                json.dump(rows, open(path0, "w"), indent=1); continue
            # ENUM lists coordinates in the automaton's canonical (sorted) track
            # order, which is not the parameter order for e.g. border (b,i,l).
            order = line.split()[1][len("vars=["):-1].split(",")
            perm = [order.index(v) for v in params]
            got = set()
            for tok in line.split()[3:]:
                vals = [int(x) for x in tok.split(",")]
                got.add(tuple(vals[q] for q in perm))
            want = brute(kind, w, B)
            ok = got == want
            rows.append({"seq": name, "kind": kind, "B": B, "ok": ok,
                         "n_engine": len(got), "n_brute": len(want),
                         "missing": len(want - got), "extra": len(got - want)})
            print("  %-14s %-7s %s  (%d tuples, B=%d)" %
                  (name, kind, "PASS" if ok else "FAIL", len(want), B), flush=True)
            json.dump(rows, open(path0, "w"), indent=1)
    json.dump(rows, open(path0, "w"), indent=1)
    print("->", path0, " failures:", sum(1 for r in rows if r["ok"] is not True))


# ---------------------------------------------------------------- end-to-end ladders

# The two ladders these predicate classes exist for.
#
#   palindrome of every length     A n. E i. REV(i,i,n)
#   critical exponent              E i,n,l. n>=1 & b*l >= a*n & PER(i,l,n)
#
# For an integer exponent the length variable is not needed at all (l = a*n), which
# saves a whole track; for a fraction it is.  Exponents with larger numerators are
# not in the ladder because the *arithmetic* -- `lin_auto` folding a copies of n and
# b copies of l -- exhausts 6 GB on Tribonacci for every variant including the
# learned one (measured: 10/3 and 13/4 both die, 7/2 finishes).  That is a cost of
# the multiplication, not of the predicate, so a ladder that includes them measures
# the wrong thing.
EXPONENTS = [(2, 1), (5, 2), (3, 1), (7, 2), (4, 1)]

PREDS = {
    "learned": {"build": ["learn RV rev"], "buildp": ["learn PR period"],
                "pal": "$RV(i,i,n)", "per": "$PR(i,{L},n)"},
    "let":     {"build": ["let RVD(i,j,l) %s" % KINDS["rev"][1]],
                "buildp": ["let PRD(i,l,p) %s" % KINDS["period"][1]],
                "pal": "$RVD(i,i,n)", "per": "$PRD(i,{L},n)"},
    "inline":  {"build": [], "buildp": [],
                "pal": "A t,u. (t<n & t+u+1=n) => T[i+t]=T[i+u]",
                "per": "A t. t+n<{L} => T[i+t]=T[i+t+n]"},
}


def exp_queries(pred):
    out = []
    for a, b in EXPONENTS:
        if b == 1:
            out.append(("exp>=%d" % a, "E i,n. n>=1 & (%s)" % pred["per"].format(L="%d*n" % a)))
        else:
            out.append(("exp>=%d/%d" % (a, b),
                        "E i,n,l. n>=1 & %d*l >= %d*n & (%s)" % (b, a, pred["per"].format(L="l"))))
    return out


def run_ladder(prefix, build, queries, timeout, mem):
    t0 = time.time()
    script = list(build) + ["? " + q for _, q in queries]
    r = engine.run(prefix + "\n" + "\n".join(script) + "\nmem\nquit\n", timeout=timeout, mem_mb=mem)
    v = [l.split()[0] for l in r.stdout.split("\n") if l.startswith(("TRUE", "FALSE"))]
    peak = field(next((l for l in r.stdout.split("\n") if l.startswith("OK mem")), ""),
                 "peak", lambda x: int(x.rstrip("MB")))
    return {"secs": round(time.time() - t0, 1), "verdicts": v, "peak_mb": peak,
            "timeout": r.timed_out, "budget": r.budget, "killed": r.killed,
            "n_expected": len(queries), "stdout": r.stdout}


def cmd_ladder(argv):
    cases = [("tribonacci", TRIB), ("prism-1", dict(PANEL)["prism-1"])]
    rows = []
    for label, prefix in cases:
        for ladder in ("palindrome", "exponent"):
            row = {"case": label, "ladder": ladder}
            for variant, pred in PREDS.items():
                if ladder == "palindrome":
                    qs = [("palindrome-of-every-length", "A n. E i. (%s)" % pred["pal"])]
                    build = pred["build"]
                else:
                    qs = exp_queries(pred)
                    build = pred["buildp"]
                row["names"] = [n for n, _ in qs]
                row[variant] = run_ladder(prefix, build, qs, timeout=1800, mem=6144)
                r = row[variant]
                print("%-11s %-10s %-8s %6.1fs %s%s" % (
                    label, ladder, variant, r["secs"], r["verdicts"],
                    " INCOMPLETE" if len(r["verdicts"]) < r["n_expected"] else ""), flush=True)
            rows.append(row)
            json.dump(rows, open(os.path.join(RES, "learn_ladder.json"), "w"), indent=1)
    print("->", os.path.join(RES, "learn_ladder.json"))


# ---------------------------------------------------------------- regression

def cmd_regress(argv):
    """`learnfe` on two engine binaries, field by field.  Usage:

        git archive HEAD engine | tar -x -C /tmp/base
        (cd /tmp/base && CARGO_TARGET_DIR=/tmp/base/target cargo build --release \
                               --manifest-path engine/Cargo.toml)
        python3 explore/learn_bench.py regress /tmp/base/target/release/peanut \
                                               engine/target/release/peanut

    Compares every field of the `OK learnfe` line except ms/peak (timing and allocator
    noise), on the whole panel plus Tribonacci and Fibonacci.  This is the gate for
    "the FE path did not change"."""
    import subprocess
    if len(argv) < 2: print(cmd_regress.__doc__); return
    old_bin, new_bin = argv[0], argv[1]
    cases = list(PANEL) + [("tribonacci", TRIB), ("fibonacci", FIB)]

    def run(binary, src, to=1800):
        e = dict(os.environ); e["AM_MEM_MB"] = os.environ.get("AM_MEM_MB", "4096")
        return subprocess.run([binary], input=src + "\nquit\n", capture_output=True,
                              text=True, timeout=to, env=e).stdout

    bad = n = 0
    for name, d in cases:
        src = d + "\nlearnfe FE"
        try:
            a, b = run(old_bin, src), run(new_bin, src)
        except subprocess.TimeoutExpired:
            print("%-16s TIMEOUT" % name); continue
        ga = [l for l in a.split("\n") if l.startswith("OK learnfe")]
        gb = [l for l in b.split("\n") if l.startswith("OK learnfe")]
        if not ga or not gb:
            print("%-16s old=%s new=%s" % (name, ga or "-", gb or "-")); continue
        f = lambda l: {k: v for k, v in (t.split("=", 1) for t in l.split() if "=" in t)
                       if k not in ("ms", "peak")}
        n += 1
        if f(ga[0]) != f(gb[0]):
            bad += 1; print("DIFF %-16s\n  old %s\n  new %s" % (name, ga[0], gb[0]))
        else:
            print("same %-16s %s" % (name, " ".join(
                t for t in ga[0].split() if t.startswith(("states=", "eqs=", "ces=", "mqs=", "iters=")))),
                flush=True)
    print("compared %d, differences %d" % (n, bad))


# ---------------------------------------------------------------- report

def cmd_report(argv):
    """Render the markdown tables for docs/LEARN.md from results/learn_*.json."""
    def load(n):
        try: return json.load(open(os.path.join(RES, n)))
        except Exception: return None

    p = load("learn_panel.json")
    if p:
        print("### panel\n")
        for kind in KINDS:
            rows = [r for r in p if r["kind"] == kind]
            if not rows: continue
            print("\n**%s**\n" % kind)
            print("| sequence | learned | `let` | same | learn s | `let` s | learn MB | `let` MB | proved equal |")
            print("|---|---|---|---|---|---|---|---|---|")
            for r in rows:
                l, d = r["learn"], r["direct"]
                print("| %s | %s | %s | %s | %s | %s | %s | %s | %s |" % (
                    r["seq"], l["states"] or "—", d["states"] or ("**%s**" % d.get("err")),
                    "yes" if r.get("same_size") else ("—" if not (l["states"] and d["states"]) else "**NO**"),
                    ("%.1f" % (l["ms"] / 1000)) if l["ms"] is not None else "—",
                    ("%.1f" % (d["ms"] / 1000)) if d["ms"] is not None else "—",
                    l.get("peak_mb", "—"), d.get("peak_mb", "—"), r.get("equiv") or "—"))
    n = load("learn_numsys.json")
    if n:
        print("\n### numeration systems\n")
        print("| sequence | class | learned | `let` | same | learn s | `let` s | proved equal |")
        print("|---|---|---|---|---|---|---|---|")
        for r in n:
            l, d = r["learn"], r["direct"]
            print("| %s | %s | %s | %s | %s | %s | %s | %s |" % (
                r["seq"], r["kind"], l["states"] or "—", d["states"] or d.get("err"),
                "yes" if r.get("same_size") else "—",
                ("%.2f" % (l["ms"] / 1000)) if l["ms"] is not None else "—",
                ("%.2f" % (d["ms"] / 1000)) if d["ms"] is not None else "—",
                r.get("equiv") or "—"))
    b = load("learn_brute.json")
    if b:
        print("\n### brute force\n")
        for r in b:
            print("* %-14s %-7s %s (%s tuples, coords < %s)" %
                  (r["seq"], r["kind"], "PASS" if r["ok"] else "FAIL",
                   r.get("n_brute"), r.get("B")))
    f = load("learn_fuzz.json")
    if f:
        ag = sum(1 for r in f if r["agree"] is True)
        di = sum(1 for r in f if r["agree"] is False)
        ic = sum(1 for r in f if r["agree"] is None)
        print("\n### fuzz\n\nagree %d / disagree %d / incomplete %d of %d" % (ag, di, ic, len(f)))
    la = load("learn_ladder.json")
    if la:
        print("\n### ladders\n")
        print("| case | ladder | learned s | `let` s | inline s | learned MB | `let` MB | inline MB |")
        print("|---|---|---|---|---|---|---|---|")
        for r in la:
            def c(v):
                x = r[v]
                bad = "" if len(x["verdicts"]) == x["n_expected"] else "*"
                return "%.1f%s" % (x["secs"], bad)
            print("| %s | %s | %s | %s | %s | %s | %s | %s |" % (
                r["case"], r["ladder"], c("learned"), c("let"), c("inline"),
                r["learned"]["peak_mb"], r["let"]["peak_mb"], r["inline"]["peak_mb"]))
        for r in la:
            print("\n**%s / %s**\n" % (r["case"], r["ladder"]))
            print("| query | learned | `let` | inline |")
            print("|---|---|---|---|")
            for i, nm in enumerate(r["names"]):
                g = lambda v: r[v]["verdicts"][i] if i < len(r[v]["verdicts"]) else "—"
                print("| %s | %s | %s | %s |" % (nm, g("learned"), g("let"), g("inline")))


if __name__ == "__main__":
    cmds = {"panel": cmd_panel, "trib": cmd_trib, "fuzz": cmd_fuzz,
            "brute": cmd_brute, "ladder": cmd_ladder, "report": cmd_report,
            "regress": cmd_regress}
    if len(sys.argv) < 2 or sys.argv[1] not in cmds:
        print(__doc__); sys.exit(1)
    cmds[sys.argv[1]](sys.argv[2:])
