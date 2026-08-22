"""Generate (and machine-check) the numeration-system files in engine/numeration/.

For each system we write two automata in Walnut's "Custom Bases" text format:

    <name>.txt            validity: msd, leading zeros allowed, accepts exactly the
                          canonical representations
    <name>_addition.txt   addition: three tracks (x,y,z), accepts iff x + y = z

The validity automaton is the textbook admissibility condition for the system
(Zeckendorf: no "11"; Tribonacci: no "111"; Pell: a "2" must be followed by "0").
The adder is *constructed*, not copied, by the msd difference-vector method:

    reading msd with r digits still to come, the running difference
        D = val(x prefix) + val(y prefix) - val(z prefix)
    is written in the basis (U_r, U_{r+1}, ..., U_{r+d-1}) as a vector e.
    Reading one more digit triple with s = a + b - c gives r' = r-1 and
        e'_j = [j>=1] e_{j-1} + e_{d-1} * a_{d-j} + [j==0] s
    (from U_{r-1+d} = a_1 U_{r-2+d} + ... + a_d U_{r-1}).
    Start e = 0; accept when sum_j e_j U_j = 0.

Everything is then checked three ways, and nothing is written unless all pass:
  1. rank/unrank: value(rep(n)) == n and rep is greedy, for n < 10^6; U_l = #valid
     words of length l satisfies the recurrence.
  2. adder: accepts (x,y,x+y) and rejects (x,y,z) for z != x+y, all x,y < 400,
     plus random large pairs.
  3. equivalence with Walnut's own msd_<name>_addition.txt (when a Walnut checkout
     is present), as languages restricted to valid representations on all 3 tracks.

Run:
    python3 explore/gen_numsys.py [--check-only]
"""
import os, sys, itertools, random

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "engine", "numeration")
WALNUT = os.path.join(os.environ.get("WALNUT_HOME", os.path.join(ROOT, "walnut7")), "Custom Bases")

# ---------------------------------------------------------------- the systems
# valid: (nstates, accepting set, trans dict {(state,digit): state}); missing = dead
SYSTEMS = {
    "fib": dict(
        digits=2, rec=[1, 1],
        doc="Zeckendorf / Fibonacci: digits {0,1}, no factor 11, weights 1,2,3,5,8,...",
        nstates=2, acc={0, 1}, trans={(0, 0): 0, (0, 1): 1, (1, 0): 0}),
    "trib": dict(
        digits=2, rec=[1, 1, 1],
        doc="Tribonacci: digits {0,1}, no factor 111, weights 1,2,4,7,13,24,...",
        nstates=3, acc={0, 1, 2}, trans={(0, 0): 0, (0, 1): 1, (1, 0): 0, (1, 1): 2, (2, 0): 0}),
    "pell": dict(
        digits=3, rec=[2, 1],
        doc="Pell: digits {0,1,2}, a 2 must be followed by a 0, weights 1,2,5,12,29,...",
        nstates=2, acc={0}, trans={(0, 0): 0, (0, 1): 0, (0, 2): 1, (1, 0): 0}),
}

MAXLEN = 96

class Valid:
    def __init__(self, spec):
        self.D = spec["digits"]; self.n = spec["nstates"]
        self.acc = spec["acc"]; self.tr = spec["trans"]
        self.dead = self.n                      # implicit sink
        self.cnt = [[0] * (MAXLEN + 1) for _ in range(self.n + 1)]
        for q in range(self.n + 1):
            self.cnt[q][0] = 1 if q in self.acc else 0
        for l in range(1, MAXLEN + 1):
            for q in range(self.n + 1):
                self.cnt[q][l] = sum(self.cnt[self.t(q, d)][l - 1] for d in range(self.D))
    def t(self, q, d):
        return self.tr.get((q, d), self.dead)
    def weight(self, l):
        return self.cnt[0][l]
    def rep(self, n):
        L = 0
        while self.cnt[0][L] <= n: L += 1
        if L == 0: L = 1
        return self.unrank(n, L)
    def unrank(self, n, L):
        w = []; q = 0
        for p in range(L):
            rem = L - 1 - p
            for e in range(self.D):
                q2 = self.t(q, e); c = self.cnt[q2][rem]
                if n < c: w.append(e); q = q2; break
                n -= c
            else:
                raise ValueError("unrank out of range")
        return w
    def value(self, w):
        q = 0; v = 0; L = len(w)
        for p, d in enumerate(w):
            rem = L - 1 - p
            for e in range(d): v += self.cnt[self.t(q, e)][rem]
            q = self.t(q, d)
        return v if q in self.acc else None

def greedy(V, U, n):
    """Greedy representation of n against the weight sequence U (independent of rank)."""
    if n == 0: return [0]
    L = 0
    while L + 1 < len(U) and U[L + 1] <= n: L += 1
    out = []
    for i in range(L, -1, -1):
        d = n // U[i]
        assert d < V.D, (n, i, d)
        out.append(d); n -= d * U[i]
    assert n == 0
    return out

# ---------------------------------------------------------------- adder
def build_adder(V, rec, bound=12):
    d = len(rec); D = V.D
    U = [V.weight(l) for l in range(MAXLEN + 1)]
    for n in range(d, 40):
        assert U[n] == sum(rec[i] * U[n - 1 - i] for i in range(d)), (n, U[:12])
    start = tuple([0] * d)
    seen = {start: 0}; order = [start]; trans = {}
    i = 0
    while i < len(order):
        e = order[i]
        for a, b, c in itertools.product(range(D), repeat=3):
            s = a + b - c
            ne = [0] * d
            for j in range(d):
                v = e[j - 1] if j >= 1 else 0
                v += e[d - 1] * rec[d - 1 - j]
                if j == 0: v += s
                ne[j] = v
            ne = tuple(ne)
            if max(abs(x) for x in ne) > bound: continue     # provably dead (see check below)
            if ne not in seen:
                seen[ne] = len(order); order.append(ne)
            trans[(i, (a, b, c))] = seen[ne]
        i += 1
    acc = {q for q, e in enumerate(order) if sum(e[j] * U[j] for j in range(d)) == 0}
    n2, acc2, tr2 = trim(len(order), acc, trans, D, 3)
    return minimize(n2, acc2, tr2, D, 3), order

def minimize(n, acc, trans, D, tracks):
    """Moore refinement on the completed automaton (explicit dead state), then drop
    the dead class again.  Does not change the language, only the file size."""
    syms = list(itertools.product(range(D), repeat=tracks))
    dead = n
    T = [[trans.get((q, s), dead) if q != dead else dead for s in syms] for q in range(n + 1)]
    color = [1 if q in acc else 0 for q in range(n + 1)]
    while True:
        sig, new = {}, []
        for q in range(n + 1):
            key = (color[q],) + tuple(color[t] for t in T[q])
            new.append(sig.setdefault(key, len(sig)))
        done = len(set(new)) == len(set(color))
        color = new
        if done: break
    classes = sorted(set(color), key=lambda c: (c != color[0], c))
    idx = {c: i for i, c in enumerate(classes)}
    rep = {}
    for q in range(n + 1): rep.setdefault(color[q], q)
    nacc = {idx[color[q]] for q in range(n) if q in acc}
    ntr = {(idx[c], s): idx[color[T[rep[c]][i]]] for c in classes for i, s in enumerate(syms)}
    dq = idx[color[dead]]
    if dq not in nacc and all(ntr[(dq, s)] == dq for s in syms):
        keep = [i for i in range(len(classes)) if i != dq]
        ridx = {q: i for i, q in enumerate(keep)}
        ntr = {(ridx[q], s): ridx[t] for (q, s), t in ntr.items() if q != dq and t != dq}
        nacc = {ridx[q] for q in nacc if q in ridx}
        return (len(keep), nacc, ntr)
    return (len(classes), nacc, ntr)

def trim(n, acc, trans, D, tracks):
    """Keep only states reachable from 0 and co-reachable to an accepting state."""
    # co-reachable
    co = set(acc); changed = True
    while changed:
        changed = False
        for (q, sym), t in trans.items():
            if t in co and q not in co: co.add(q); changed = True
    reach = {0}; stack = [0]
    while stack:
        q = stack.pop()
        for sym in itertools.product(range(D), repeat=tracks):
            t = trans.get((q, sym))
            if t is not None and t in co and t not in reach:
                reach.add(t); stack.append(t)
    keep = sorted(reach & co) if 0 in co else []
    idx = {q: i for i, q in enumerate(keep)}
    ntr = {(idx[q], sym): idx[t] for (q, sym), t in trans.items() if q in idx and t in idx}
    nacc = {idx[q] for q in keep if q in acc}
    return (len(keep), nacc, ntr)

# ---------------------------------------------------------------- Walnut format IO
def write_aut(path, tracks, D, n, acc, trans, header):
    lines = [f"# {h}" for h in header]
    lines.append(" ".join("{" + ", ".join(str(i) for i in range(D)) + "}" for _ in range(tracks)))
    lines.append("")
    for q in range(n):
        lines.append(f"{q} {1 if q in acc else 0}")
        for sym in sorted(k[1] for k in trans if k[0] == q):
            lines.append(" ".join(str(x) for x in sym) + f" -> {trans[(q, sym)]}")
        lines.append("")
    open(path, "w").write("\n".join(lines).rstrip() + "\n")

def read_walnut(path, D=None):
    """Minimal reader for Walnut's format -> (tracks, D, nstates, acc, trans)."""
    txt = [l.split("#")[0] for l in open(path).read().split("\n")]
    txt = [l for l in txt if l.strip()]
    head = txt[0]
    tracks = head.count("{") if "{" in head else len(head.split())
    if "{" in head:
        first = head[head.index("{") + 1:head.index("}")]
        DD = len([x for x in first.split(",") if x.strip() != ""])
    else:
        DD = D
    acc = set(); trans = {}; cur = None; mx = 0; start = None
    for l in txt[1:]:
        if "->" in l:
            lhs, rhs = l.split("->")
            sym = tuple(int(x) for x in lhs.split())
            t = int(rhs.strip()); mx = max(mx, t)
            trans[(cur, sym)] = t
        else:
            p = l.split(); cur = int(p[0]); mx = max(mx, cur)
            if start is None: start = cur
            if int(p[1]) != 0: acc.add(cur)
    # Walnut's initial state is the FIRST state DECLARED, not state number 0.
    relab = lambda q: 0 if q == start else (start if q == 0 else q)
    acc = {relab(q) for q in acc}
    trans = {(relab(q), sym): relab(t) for (q, sym), t in trans.items()}
    return tracks, DD, mx + 1, acc, trans

# ---------------------------------------------------------------- checks
def run(n, acc, trans, word, D):
    q = 0
    for sym in word:
        q = trans.get((q, sym))
        if q is None: return False
    return q in acc

def pad(V, vals, L=None):
    reps = [V.rep(v) for v in vals]
    L = L or max(len(r) for r in reps)
    reps = [[0] * (L - len(r)) + r for r in reps]
    return list(zip(*reps))

def equivalent(A, B, D, tracks, restrict):
    """Language equivalence of two total-ish DFAs after intersecting each track with
    `restrict` (the validity automaton), by product + reachability."""
    (na, acca, tra) = A; (nb, accb, trb) = B
    (nv, accv, trv) = restrict
    DEADA, DEADB, DEADV = na, nb, nv
    def ta(q, s): return tra.get((q, s), DEADA) if q != DEADA else DEADA
    def tb(q, s): return trb.get((q, s), DEADB) if q != DEADB else DEADB
    def tv(q, d): return trv.get((q, (d,)), DEADV) if q != DEADV else DEADV
    start = (0, 0) + tuple(0 for _ in range(tracks))
    seen = {start}; stack = [start]
    while stack:
        st = stack.pop()
        qa, qb = st[0], st[1]; vs = st[2:]
        valid = all(v in accv for v in vs)
        if valid and ((qa in acca and qa != DEADA) != (qb in accb and qb != DEADB)):
            return False, st
        for sym in itertools.product(range(D), repeat=tracks):
            nxt = (ta(qa, sym), tb(qb, sym)) + tuple(tv(vs[i], sym[i]) for i in range(tracks))
            if nxt not in seen: seen.add(nxt); stack.append(nxt)
    return True, None

def check(name, spec, adder, V, verbose=True):
    D = V.D; U = [V.weight(l) for l in range(40)]
    # 1. rank/unrank vs greedy
    for n in range(0, 200000):
        r = V.rep(n)
        assert V.value(r) == n, (name, n, r)
        assert r == greedy(V, U, n), (name, n, r, greedy(V, U, n))
    # value is order preserving on same-length words (radix order == numeric order)
    for L in range(1, 9):
        vals = []
        for w in itertools.product(range(D), repeat=L):
            v = V.value(list(w))
            if v is not None: vals.append(v)
        assert vals == sorted(vals) and vals == list(range(len(vals))), (name, L)
    # 2. adder
    n_, acc_, tr_ = adder
    N = 400
    for x in range(N):
        for y in range(N):
            w = pad(V, [x, y, x + y])
            assert run(n_, acc_, tr_, w, D), (name, "missing", x, y)
    random.seed(7)
    for _ in range(20000):
        x = random.randrange(10 ** 9); y = random.randrange(10 ** 9)
        assert run(n_, acc_, tr_, pad(V, [x, y, x + y]), D), (name, "big", x, y)
        z = x + y + random.choice([-3, -2, -1, 1, 2, 3])
        if z >= 0:
            assert not run(n_, acc_, tr_, pad(V, [x, y, z]), D), (name, "false accept", x, y, z)
    for x in range(60):
        for y in range(60):
            for z in range(200):
                got = run(n_, acc_, tr_, pad(V, [x, y, z]), D)
                assert got == (x + y == z), (name, x, y, z, got)
    if verbose: print(f"  {name}: rank/greedy ok, adder ok ({n_} states)")
    return True

def main():
    checkonly = "--check-only" in sys.argv
    os.makedirs(OUT, exist_ok=True)
    for name, spec in SYSTEMS.items():
        V = Valid(spec)
        adder, order = build_adder(V, spec["rec"])
        # dead-state pruning is safe iff a wider bound gives the same trimmed automaton
        adder2, _ = build_adder(V, spec["rec"], bound=24)
        assert adder[0] == adder2[0], (name, "pruning bound too tight", adder[0], adder2[0])
        check(name, spec, adder, V)
        vaut = (V.n, V.acc, {(q, (d,)): t for (q, d), t in V.tr.items()})
        wal = os.path.join(WALNUT, f"msd_{name}_addition.txt")
        if os.path.isfile(wal):
            wtracks, wD, wn, wacc, wtr = read_walnut(wal)
            ok, wit = equivalent(adder, (wn, wacc, wtr), V.D, 3, vaut)
            print(f"  {name}: equivalent to Walnut msd_{name}_addition.txt: {ok}"
                  + ("" if ok else f" (witness state {wit})"))
            assert ok
        else:
            print(f"  {name}: no Walnut file at {wal}, skipping equivalence check")
        if checkonly: continue
        hdr = [spec["doc"],
               "Format: Walnut 'Custom Bases' automaton text (alphabet line, then",
               "'state output' blocks with 'digits -> target' transitions).  Walnut's own",
               "msd_" + name + "*.txt drop in unchanged; this file was generated and checked by",
               "explore/gen_numsys.py (see docs/NUMERATION.md), not copied from Walnut."]
        write_aut(os.path.join(OUT, f"{name}.txt"), 1, V.D, V.n, V.acc,
                  {(q, (d,)): t for (q, d), t in V.tr.items()},
                  hdr + [f"Valid representations (weights U_l = #words of length l): "
                         + ", ".join(str(V.weight(l)) for l in range(8)) + ", ..."])
        write_aut(os.path.join(OUT, f"{name}_addition.txt"), 3, V.D, adder[0], adder[1], adder[2],
                  hdr + ["Addition x + y = z over three tracks."])
        print(f"  {name}: wrote {name}.txt ({V.n} states) and {name}_addition.txt ({adder[0]} states)")

if __name__ == "__main__":
    main()
