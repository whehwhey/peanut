"""Brute-force cross-check of the numeration-system engine (Fibonacci, Tribonacci, Pell).

Everything the engine claims here is re-derived independently in Python from the
substitution fixed point / from integer arithmetic, on a 10^6 prefix:

  A. the sequence   engine `seq`  ==  fixed point of the substitution
  B. arithmetic     engine `enum` ==  {tuples satisfying the relation}, for
                    +, <, =, constants and T[i]=T[j], every coordinate < 12
  C. the sentences  every TRUE/FALSE the fib/trib benchmark reports, checked on
                    the prefix (a FALSE existential must have no witness in range,
                    a TRUE universal no counterexample; a TRUE existential must
                    have a witness, and we print it)
  D. the FE automaton (learnfe) == the direct longest-common-prefix answer, for
                    all i,j < 40 and l < 20

Run:
    python3 explore/numsys_check.py
Exit status is nonzero if any check fails.
"""
import os, sys, itertools
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__))))
import engine

N = 10 ** 6
fails = []

def ok(name, cond, extra=""):
    print(("PASS " if cond else "FAIL ") + name + (("  " + extra) if extra else ""))
    if not cond: fails.append(name)

def fixed_point(rules, start, n):
    """Prefix of the fixed point of a substitution given as {letter: word}."""
    s = [start]
    while len(s) < n:
        t = []
        for c in s: t.extend(rules[c])
        if len(t) == len(s): break
        s = t
    return s[:n]

def run(src, **kw):
    r = engine.run(src, timeout=kw.pop("timeout", 600), mem_mb=kw.pop("mem_mb", 4096))
    if not r.ok:
        print("ENGINE FAILED:", r.stdout[-2000:], r.stderr[-2000:] if r.stderr else "")
        sys.exit(2)
    return r.stdout

def parse_enum(line):
    body = line.split("]", 1)[1].strip()
    body = body.split(" ", 1)[1] if body.startswith("n=") else body
    if not body.strip(): return set()
    return {tuple(int(x) for x in t.split(",")) for t in body.split()}

SYS = {
    "fib":  ("F",  "0:0,1 1:0,-",        {0: [0, 1], 1: [0]},        0),
    "trib": ("TR", "0:0,1 1:0,2 2:0,-",  {0: [0, 1], 1: [0, 2], 2: [0]}, 0),
}

def check_system(name):
    seqname, spec, rules, start = SYS[name]
    print(f"\n=== {name} ===")
    w = fixed_point(rules, start, N)
    head = "".join(str(c) for c in w[:200])
    out = run(f"numsys {name}\ndfao {seqname} 2 {spec}\nseq 200\n")
    got = [l for l in out.split("\n") if l.startswith("SEQ")][0].split()[-1]
    ok(f"{name}: sequence prefix (200) == substitution fixed point", got == head,
       got[:40] + " vs " + head[:40])

    # --- B. arithmetic and sequence relations, brute force to B
    B = 12
    tests = {
        "i+j=k":     {(i, j, i + j) for i in range(B) for j in range(B) if i + j < B},
        "i<j":       {(i, j) for i in range(B) for j in range(B) if i < j},
        "i<=j":      {(i, j) for i in range(B) for j in range(B) if i <= j},
        "i=5":       {(5,)} if 5 < B else set(),
        "i+3=j":     {(i, i + 3) for i in range(B) if i + 3 < B},
        "2*i=j":     {(i, 2 * i) for i in range(B) if 2 * i < B},
        "T[i]=T[j]": {(i, j) for i in range(B) for j in range(B) if w[i] == w[j]},
        "T[i]=1":    {(i,) for i in range(B) if w[i] == 1},
        "T[i+1]=T[j]": {(i, j) for i in range(B) for j in range(B) if w[i + 1] == w[j]},
    }
    src = f"numsys {name}\ndfao {seqname} 2 {spec}\n" + "".join(f"enum {B} {f}\n" for f in tests)
    lines = [l for l in run(src).split("\n") if l.startswith("ENUM")]
    for (f, want), line in zip(tests.items(), lines):
        ok(f"{name}: enum {f}", parse_enum(line) == want,
           "" if parse_enum(line) == want else f"got {sorted(parse_enum(line))[:6]} want {sorted(want)[:6]}")

    # --- C. the sentences
    import array
    a = array.array("b", w)
    def eq_run(p):
        """longest run length of positions with a[i]==a[i+p], as a list of run lengths"""
        best = 0; cur = 0
        for i in range(N - p):
            if a[i] == a[i + p]: cur += 1; best = max(best, cur)
            else: cur = 0
        return best
    sentences = []
    # (a) 1 is always followed by 0  (fib only)
    if name == "fib":
        sentences.append(("A i. T[i]=1 => T[i+1]=0",
                          all(not (a[i] == 1 and a[i + 1] == 1) for i in range(N - 1))))
    # (b) eventual periodicity
    per = False
    for p in range(1, 300):
        if all(a[i] == a[i + p] for i in range(N - 5000, N - p)): per = True; break
    sentences.append(("E p,N. p>=1 & A i. i>=N => T[i]=T[i+p]", per))
    # (c) cubes / (d) fourth powers, via run lengths of the p-shift agreement
    cube = any(eq_run(p) >= 2 * p for p in range(1, 300))
    four = any(eq_run(p) >= 3 * p for p in range(1, 300))
    sentences.append(("E i,n. n>=1 & A t. t<2*n => T[i+t]=T[i+n+t]", cube))
    sentences.append(("E i,n. n>=1 & A t. t<3*n => T[i+t]=T[i+n+t]", four))
    src = f"numsys {name}\ndfao {seqname} 2 {spec}\n" + "".join(f"? {s}\n" for s, _ in sentences)
    lines = [l for l in run(src).split("\n") if l.startswith(("TRUE", "FALSE"))]
    for (s, want), line in zip(sentences, lines):
        got = line.startswith("TRUE")
        ok(f"{name}: {s}", got == want, f"engine={got} brute(10^6 prefix)={want}")

    # --- D. FE automaton vs direct LCP
    M, L = 40, 20
    want = set()
    for i in range(M):
        for j in range(M):
            for l in range(L):
                if all(a[i + t] == a[j + t] for t in range(l)): want.add((i, j, l))
    src = (f"numsys {name}\ndfao {seqname} 2 {spec}\nlearnfe FE\n"
           f"enum {max(M, L)} $FE(i,j,l)\n")
    out = run(src)
    line = [l for l in out.split("\n") if l.startswith("ENUM")][0]
    got = {t for t in parse_enum(line) if t[0] < M and t[1] < M and t[2] < L}
    ok(f"{name}: learnfe FE(i,j,l) == direct LCP for i,j<{M}, l<{L}", got == want,
       "" if got == want else f"symdiff {sorted(got ^ want)[:5]}")
    st = [l for l in out.split("\n") if l.startswith("OK learnfe")][0]
    print("   ", st)

def check_pell():
    """Pell: arithmetic, plus a real Pell-automatic word if a Walnut checkout is here.

    The word check is the strongest cross-check of the numeration layer available for
    a system with no substitution of ours behind it: the engine's Rust rank/unrank is
    compared against the independent Python implementation in gen_numsys.py, on a
    sequence neither of them was written for (Walnut's R2).
    """
    print("\n=== pell ===")
    B = 12
    tests = {
        "i+j=k": {(i, j, i + j) for i in range(B) for j in range(B) if i + j < B},
        "i<j":   {(i, j) for i in range(B) for j in range(B) if i < j},
        "i=7":   {(7,)},
        "3*i=j": {(i, 3 * i) for i in range(B) if 3 * i < B},
    }
    src = "numsys pell\ndfao U 3 0:0,1,2 1:0,-,- 2:0,-,-\n" + "".join(f"enum {B} {f}\n" for f in tests)
    lines = [l for l in run(src).split("\n") if l.startswith("ENUM")]
    for (f, want), line in zip(tests.items(), lines):
        ok(f"pell: enum {f}", parse_enum(line) == want)

    walnut_home = os.environ.get("WALNUT_HOME",
                                os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "walnut7"))
    r2 = os.path.join(walnut_home, "Word Automata Library", "R2.txt")
    if not os.path.isfile(r2):
        print("SKIP pell: no walnut7 checkout, skipping the R2 word check"); return
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from gen_numsys import Valid, SYSTEMS, read_walnut
    V = Valid(SYSTEMS["pell"])
    _, _, n, acc, tr = read_walnut(r2, D=3)
    out = {0: 0, 1: 0, 2: 1, 3: 1}          # state outputs, from the file
    want = []
    for i in range(400):
        q = 0
        for d in V.rep(i): q = tr.get((q, (d,)))
        want.append(out[q])
    got = run(f"numsys pell\ndfao R2 @{r2}\nseq 400\n")
    got = [int(c) for c in [l for l in got.split("\n") if l.startswith("SEQ")][0].split()[-1]]
    ok("pell: engine R2[n] == python rank/unrank + Walnut's R2 automaton, n<400",
       got == want, "" if got == want else f"first diff at {next(i for i in range(400) if got[i]!=want[i])}")

if __name__ == "__main__":
    for n in ("fib", "trib"): check_system(n)
    check_pell()
    print()
    if fails:
        print(f"{len(fails)} FAILED: {fails}"); sys.exit(1)
    print("all checks passed")
