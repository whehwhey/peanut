"""Substitution fixed point  ->  DFAO over a non-uniform numeration system.

For a k-uniform morphism the engine's `def` command already does this.  For a
*Pisot* substitution such as Fibonacci (0 -> 01, 1 -> 0) the fixed point is not
k-automatic at all; it is U-automatic, where U is the numeration system whose
weights are U_i = |sigma^i(a)| (1,2,3,5,8,... for Fibonacci -- Zeckendorf).

Construction (Dumont-Thomas).  A position n is located in sigma^L(a) by peeling
one level at a time: with the current letter b, the digit d says "skip the first
d letters of sigma(b), then descend into the (d+1)-st".  Descending is legal only
if every skipped letter has the same subtree size as a, i.e. only if
sigma(b)[0..d) is a^d -- which is exactly the admissibility condition of the
numeration system (Zeckendorf's "no 11" is "you cannot skip twice in a row").
So the DFAO is simply

    states  = letters,          delta(b, d) = sigma(b)[d]   (dead if illegal)
    output  = the coding of the letter,          start = a

and the digit alphabet is max_b |sigma(b)|.

Nothing here is taken on trust.  The script checks
  1. the skip condition above (else it refuses -- the construction does not apply),
  2. |sigma^i(a)| == the weight sequence of the shipped numeration system,
  3. the accepted language == the shipped validity automaton (both directions,
     as automata, not by sampling),
  4. DFAO(rep(n)) == fixedpoint[n] for n < 10^5,
and prints the `dfao` command line for the engine.

Run:
    python3 explore/morphic_to_dfao.py                 # fib, trib, and check all
    python3 explore/morphic_to_dfao.py "0->01,1->0" fib
"""
import os, sys, itertools
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "explore"))
from gen_numsys import Valid, SYSTEMS, read_walnut          # rank/unrank + file reader

N_CHECK = 10 ** 5

def parse_sub(s):
    """'0->01,1->0' -> {0: [0,1], 1: [0]}"""
    out = {}
    for part in s.split(","):
        a, b = part.split("->")
        out[int(a.strip())] = [int(c) for c in b.strip()]
    return out

def fixed_point(sub, a, n):
    s = [a]
    while len(s) < n:
        t = []
        for c in s: t.extend(sub[c])
        if len(t) == len(s): break
        s = t
    return s[:n]

def build(sub, a=0, coding=None):
    """Dumont-Thomas DFAO.  Returns (D, nstates, trans, out) with `None` for dead."""
    letters = sorted(sub)
    assert sub[a][0] == a, "substitution is not prolongable at the start letter"
    D = max(len(sub[b]) for b in letters)
    trans = {}
    for b in letters:
        w = sub[b]
        for d in range(D):
            if d < len(w) and all(w[t] == a for t in range(d)):
                trans[(b, d)] = w[d]
    out = {b: (coding[b] if coding else b) for b in letters}
    return D, len(letters), trans, out

def language_equal(A, B, D):
    """Equality of two 1-track DFA languages (state sets, accept sets, partial trans)."""
    (na, acca, tra), (nb, accb, trb) = A, B
    da, db = na, nb
    def ta(q, d): return tra.get((q, d), da) if q != da else da
    def tb(q, d): return trb.get((q, d), db) if q != db else db
    seen = {(0, 0)}; stack = [(0, 0)]
    while stack:
        p, q = stack.pop()
        if ((p in acca and p != da) != (q in accb and q != db)): return False, (p, q)
        for d in range(D):
            n = (ta(p, d), tb(q, d))
            if n not in seen: seen.add(n); stack.append(n)
    return True, None

def check(name, subs, a=0, coding=None):
    print(f"\n=== {name}: sigma = " + ", ".join(f"{b}->{''.join(map(str, w))}" for b, w in sorted(subs.items())))
    D, n, trans, out = build(subs, a, coding)
    spec = SYSTEMS[name]
    assert D == spec["digits"], f"digit alphabet {D} != numeration system's {spec['digits']}"
    V = Valid(spec)

    # 1/2. weights
    x = [a]
    lens = [1]
    for i in range(1, 30):
        y = []
        for c in x: y.extend(subs[c])
        x = y; lens.append(len(x))
    want = [V.weight(i) for i in range(30)]
    assert lens == want, f"|sigma^i(a)| = {lens[:8]} != numeration weights {want[:8]}"
    print(f"  weights |sigma^i(a)| = {lens[:8]}... == numeration system weights  OK")

    # 3. accepted language == validity automaton
    dfao_lang = (n, set(range(n)), trans)            # every letter-state is accepting
    val_lang = (V.n, V.acc, V.tr)
    eq, wit = language_equal(dfao_lang, val_lang, D)
    assert eq, f"DFAO domain != validity language (witness state pair {wit})"
    print(f"  domain of the DFAO == validity language of {name}  OK")

    # 4. values
    w = fixed_point(subs, a, N_CHECK)
    bad = 0
    for i in range(N_CHECK):
        q = 0
        for d in V.rep(i):
            q = trans.get((q, d))
            if q is None: break
        v = None if q is None else out[q]
        if v != w[i]:
            bad += 1
            if bad < 4: print(f"  MISMATCH n={i}: dfao={v} fixedpoint={w[i]}")
    assert bad == 0, f"{bad} mismatches in the first {N_CHECK} terms"
    print(f"  DFAO(rep(n)) == fixed point for all n < {N_CHECK}  OK")

    cmd = " ".join(f"{out[b]}:" + ",".join(str(trans.get((b, d), '-')) for d in range(D))
                   for b in range(n))
    print(f"  engine:  numsys {name}\n           dfao X {D} {cmd}")
    return cmd

if __name__ == "__main__":
    if len(sys.argv) >= 3:
        check(sys.argv[2], parse_sub(sys.argv[1]))
    else:
        check("fib",  {0: [0, 1], 1: [0]})
        check("trib", {0: [0, 1], 1: [0, 2], 2: [0]})
        print("\nall constructions verified")
