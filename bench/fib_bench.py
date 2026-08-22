"""Peanut vs Walnut 8-dev on Fibonacci / Tribonacci / Pell numeration systems.

Same sentences, same machine, same 6 GB ceiling on both sides.  For each query we
record the final automaton size, the largest intermediate automaton either tool
built (`peak` here, the max of Walnut's per-step "N states" log lines there), and
wall-clock ms.  Writes bench/fib.md.

Run:
    python3 bench/fib_bench.py            # everything
    python3 bench/fib_bench.py fib        # one system
"""
import os, re, sys, time, subprocess
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "explore"))
import engine

# Walnut is optional and used only for the side-by-side comparison; set
# WALNUT_HOME to your Walnut checkout, else this benchmark runs Peanut only.
W = os.environ.get("WALNUT_HOME", os.path.join(ROOT, "walnut7"))
JAVA = os.environ.get("JAVA", "java")
TIMEOUT = 900

# name -> (peanut numsys, peanut dfao spec, walnut base, walnut word automaton)
SYS = {
    "fib":  ("fib",  "dfao F 2 0:0,1 1:0,-",       "msd_fib",  "F"),
    "trib": ("trib", "dfao TR 2 0:0,1 1:0,2 2:0,-", "msd_trib", "TR"),
}

# label, peanut formula, walnut formula (with W = the word automaton name)
QUERIES = [
    ("no 11 (fib only)",
     "A i. T[i]=1 => T[i+1]=0",
     "Ai (W[i]=@1) => (W[i+1]=@0)"),
    ("eventually periodic",
     "E p,N. p>=1 & A i. i>=N => T[i]=T[i+p]",
     "Ep,n (p>=1) & (Ai (i>=n) => (W[i]=W[i+p]))"),
    ("cube exists",
     "E i,n. n>=1 & A t. t<2*n => T[i+t]=T[i+n+t]",
     "Ei,n (n>=1) & (At (t<2*n) => (W[i+t]=W[i+n+t]))"),
    ("4th power exists",
     "E i,n. n>=1 & A t. t<3*n => T[i+t]=T[i+n+t]",
     "Ei,n (n>=1) & (At (t<3*n) => (W[i+t]=W[i+n+t]))"),
    ("palindrome of every length",
     "A n. E i. A t,u. (t<n & t+u+1=n) => T[i+t]=T[i+u]",
     "An Ei At,u ((t<n) & (t+u+1=n)) => (W[i+t]=W[i+u])"),
    ("FE(i,j,l) [direct]",
     "let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]",
     "At (t<l) => (W[i+t]=W[j+t])"),
    ("FE(i,j,l) [learnfe]", "learnfe FE", None),
]

def walnut(base, word, formula, defname):
    if formula is None: return None
    f = formula.replace("W[", word + "[")
    src = f'def {defname} "?{base} {f}":\nexit;\n'
    t0 = time.time()
    try:
        r = subprocess.run([JAVA, "-Xmx6g", "-jar", "target/Walnut-all.jar"],
                           input=src, capture_output=True, text=True, timeout=TIMEOUT, cwd=W)
    except subprocess.TimeoutExpired:
        return {"states": "timeout", "peak": "-", "ms": TIMEOUT * 1000}
    out = r.stdout + r.stderr
    secs = time.time() - t0
    sizes = [int(x) for x in re.findall(r":(\d+) states", out)]
    tot = re.findall(r"Total computation time: (\d+)ms", out)
    verdict = "TRUE" if re.search(r"^TRUE", out, re.M) else ("FALSE" if re.search(r"^FALSE", out, re.M) else "")
    if "OutOfMemory" in out:
        return {"states": "OOM", "peak": max(sizes) if sizes else "-", "ms": int(secs * 1000), "verdict": ""}
    return {"states": sizes[-1] if sizes else "?", "peak": max(sizes) if sizes else "-",
            "ms": int(tot[-1]) if tot else int(secs * 1000), "verdict": verdict,
            "wall_ms": int(secs * 1000)}

def peanut(ns, dfao, formula):
    if formula.startswith("let ") or formula.startswith("learnfe"):
        src = f"numsys {ns}\n{dfao}\n{formula}\n"
    else:
        src = f"numsys {ns}\n{dfao}\n? {formula}\n"
    t0 = time.time()
    r = engine.run(src, timeout=TIMEOUT, mem_mb=6144)
    wall = int((time.time() - t0) * 1000)
    if not r.ok:
        return {"states": "FAIL", "peak": "-", "ms": wall, "verdict": "",
                "note": ("budget" if r.budget else "timeout" if r.timed_out else "err")}
    for line in r.stdout.split("\n"):
        if line.startswith(("TRUE", "FALSE")):
            m = re.search(r"states=(\d+) peak=(\d+) ms=(\d+)", line)
            return {"states": int(m.group(1)), "peak": int(m.group(2)), "ms": int(m.group(3)),
                    "verdict": line.split()[0], "wall_ms": wall}
        if line.startswith("OK let ") or line.startswith("OK learnfe "):
            st = int(re.search(r"states=(\d+)", line).group(1))
            ms = int(re.search(r"ms=(\d+)", line).group(1))
            pk = re.search(r"peak=(\d+)", line)
            return {"states": st, "peak": int(pk.group(1)) if pk else "-", "ms": ms,
                    "verdict": "", "wall_ms": wall}
    return {"states": "?", "peak": "-", "ms": wall, "verdict": "", "note": r.stdout[-200:]}

def main():
    which = [a for a in sys.argv[1:] if a in SYS] or list(SYS)
    rows = []
    for name in which:
        ns, dfao, base, word = SYS[name]
        for i, (label, pf, wf) in enumerate(QUERIES):
            if name != "fib" and label.endswith("(fib only)"): continue
            p = peanut(ns, dfao, pf)
            if label == "FE(i,j,l) [learnfe]":
                w = None
            else:
                w = walnut(base, word, wf, f"q{name}{i}")
            rows.append((name, label, p, w))
            print(f"{name:5} {label:26} ours {str(p['states']):>7} peak {str(p['peak']):>9} "
                  f"{p['ms']:>7}ms {p.get('verdict',''):5}"
                  + (f"  | walnut {str(w['states']):>7} peak {str(w['peak']):>9} {w['ms']:>7}ms "
                     f"{w.get('verdict',''):5}" if w else "  | walnut n/a"), flush=True)
    return rows

if __name__ == "__main__":
    main()
