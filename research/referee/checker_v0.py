#!/usr/bin/env python3
"""checker_v0.py — engine-INDEPENDENT verifier for peanut exports (roadmap item 2).

Shares no code, no algorithms beyond textbook constructions, and no automata with
the engine: atoms (=, <, +, T[.]) are built here from first principles, combined
with this file's own product/complement/subset-projection (msd, leading-zero
closed), and compared against an exported engine automaton by product search.
Verdict per check: EQUAL, or DIFFER plus a concrete counterexample word decoded
to variable values. Scope v0: base-k msd only (numsys = v2, see design doc).

Usage:
  python3 checker_v0.py --selftest            # run against a live engine binary
  (library use: build formulas with the combinators, call equal(A, B))
"""
import json, subprocess, sys, itertools
from collections import deque

# ---------- automaton: complete DFA over tuples of digits for an ordered var list
class A:
    def __init__(self, k, vars, trans, accept, init=0):
        self.k, self.vars, self.trans, self.accept, self.init = k, list(vars), trans, accept, init
    def letters(self):
        return list(itertools.product(range(self.k), repeat=len(self.vars)))
    def step(self, s, tup): return self.trans[s][tup]

def build(k, vars, init_state, step_fn, accept_fn):
    """Generic reachable-part builder from a functional spec; states hashable."""
    idx, order, q = {init_state: 0}, [init_state], deque([init_state])
    trans = []
    letters = list(itertools.product(range(k), repeat=len(vars)))
    while q:
        st = q.popleft(); row = {}
        for tup in letters:
            t = step_fn(st, dict(zip(vars, tup)))
            if t not in idx:
                idx[t] = len(order); order.append(t); q.append(t)
            row[tup] = idx[t]
        trans.append(row)
    return A(k, vars, trans, [accept_fn(s) for s in order])

# ---------- atoms, from first principles (all leading-zero invariant by design)
def eq_vars(k, x, y):
    return build(k, [x, y], 'ok',
                 lambda s, d: 'ok' if s == 'ok' and d[x] == d[y] else 'dead',
                 lambda s: s == 'ok')

def lt(k, x, y):   # value(x) < value(y), msd
    def f(s, d):
        if s != 'eq': return s
        return 'eq' if d[x] == d[y] else ('lt' if d[x] < d[y] else 'gt')
    return build(k, [x, y], 'eq', f, lambda s: s == 'lt')

def add(k, x, y, z):  # value(x)+value(y) == value(z), msd; state = X+Y-Z so far
    def f(s, d):
        if s == 'dead': return s
        v = k * s + d[x] + d[y] - d[z]
        return v if v in (0, -1) else 'dead'   # only 0 / -1 stay completable
    return build(k, [x, y, z], 0, f, lambda s: s == 0)

def seq_out(dfao_k, dfao_trans, dfao_out, x, val):  # T[x] == val
    return build(dfao_k, [x], 0,
                 lambda s, d: dfao_trans[s][d[x]],
                 lambda s: dfao_out[s] == val)

def seq_eq(dfao_k, dfao_trans, dfao_out, x, y):     # T[x] == T[y]
    return build(dfao_k, [x, y], (0, 0),
                 lambda s, d: (dfao_trans[s[0]][d[x]], dfao_trans[s[1]][d[y]]),
                 lambda s: dfao_out[s[0]] == dfao_out[s[1]])

# ---------- combinators
def cylinder(a, vars):
    """Extend to a superset var list; new tracks unconstrained."""
    assert set(a.vars) <= set(vars)
    pos = [vars.index(v) for v in a.vars]
    def f(s, d): return a.step(s, tuple(d[v] for v in a.vars))
    return build(a.k, vars, a.init,
                 lambda s, d: a.trans[s][tuple(d[v] for v in a.vars)],
                 lambda s: a.accept[s])

def combine(a, b, op):
    vars = sorted(set(a.vars) | set(b.vars))
    a2, b2 = cylinder(a, vars), cylinder(b, vars)
    return build(a.k, vars, (a2.init, b2.init),
                 lambda s, d: (a2.trans[s[0]][tuple(d[v] for v in vars)],
                               b2.trans[s[1]][tuple(d[v] for v in vars)]),
                 lambda s: op(a2.accept[s[0]], b2.accept[s[1]]))

def AND(a, b): return combine(a, b, lambda p, q: p and q)
def OR(a, b):  return combine(a, b, lambda p, q: p or q)
def NOT(a):    return A(a.k, a.vars, a.trans, [not x for x in a.accept], a.init)

def EXISTS(a, v):
    """Project v, subset-construct, re-close under leading zeros (msd)."""
    keep = [w for w in a.vars if w != v]
    vi = a.vars.index(v)
    def nstep(S, tup_keep):
        out = set()
        for s in S:
            for d in range(a.k):
                full = list(tup_keep); full.insert(vi, d)
                out.add(a.trans[s][tuple(full)])
        return frozenset(out)
    zero = tuple(0 for _ in keep)
    start = frozenset([a.init])
    while True:
        nxt = start | nstep(start, zero)
        if nxt == start: break
        start = nxt
    return build(a.k, keep, start,
                 lambda S, d: nstep(S, tuple(d[w] for w in keep)),
                 lambda S: any(a.accept[s] for s in S))

def FORALL(a, v): return NOT(EXISTS(NOT(a), v))

# ---------- engine export parsing + language comparison
def from_export(j):
    vars = j['vars']; labels = [tuple(l) for l in j['labels']]
    li = {l: i for i, l in enumerate(labels)}
    trans = [{labels[x]: row[x] for x in range(j['alpha'])} for row in j['trans']]
    acc = [False] * j['nstates']
    for s in j['accepting']: acc[s] = True
    return A(j['k'], vars, trans, acc, j['initial'])

def equal(a, b):
    """Language equality by product BFS; returns (True, None) or (False, cex)."""
    assert sorted(a.vars) == sorted(b.vars), (a.vars, b.vars)
    vars = sorted(a.vars)
    a, b = cylinder(a, vars), cylinder(b, vars)
    seen = {(a.init, b.init)}; q = deque([((a.init, b.init), [])])
    letters = list(itertools.product(range(a.k), repeat=len(vars)))
    while q:
        (sa, sb), w = q.popleft()
        if a.accept[sa] != b.accept[sb]:
            vals = {v: 0 for v in vars}
            for tup in w:
                for i, v in enumerate(vars): vals[v] = vals[v] * a.k + tup[i]
            return False, (w, vals)
        for tup in letters:
            nx = (a.trans[sa][tup], b.trans[sb][tup])
            if nx not in seen: seen.add(nx); q.append((nx, w + [tup]))
    return True, None

def verdict(a):
    """Close all vars existentially -> TRUE iff nonempty (per var, then check)."""
    for v in list(a.vars): a = EXISTS(a, v)
    # zero vars: single empty letter; accept reachable?
    return a.accept[a.init]

# ---------- self-test against a live engine
def selftest(binary):
    script = ("mode msd\ndef T 2 2 0 01 10 01\n"
              "let EQ(i,j) T[i]=T[j]\nlet LT(i,j) i<j\n"
              "let P(i) E j. j<i & T[j]=1\nlet S(i,t) T[i+t]=1\n"
              "export T\nexport EQ\nexport LT\nexport P\nexport S\nquit\n")
    out = subprocess.run([binary], input=script, capture_output=True, text=True).stdout
    ex = {}
    for line in out.splitlines():
        if line.startswith("EXPORT "):
            j = json.loads(line[7:]); ex[j['name']] = j
    dk, dt, do = ex['T']['k'], ex['T']['trans'], ex['T']['out']
    checks = [
        ("EQ  = T[i]=T[j]",            seq_eq(dk, dt, do, 'i', 'j'),          ex['EQ']),
        ("LT  = i<j",                  lt(2, 'i', 'j'),                        ex['LT']),
        ("P   = E j. j<i & T[j]=1",    EXISTS(AND(lt(2, 'j', 'i'),
                                       seq_out(dk, dt, do, 'j', 1)), 'j'),     ex['P']),
        ("S   = T[i+t]=1  (via E u. i+t=u & T[u]=1)",
                                       EXISTS(AND(add(2, 'i', 't', 'u'),
                                       seq_out(dk, dt, do, 'u', 1)), 'u'),     ex['S']),
    ]
    ok = True
    for name, mine, theirs in checks:
        eq, cex = equal(mine, from_export(theirs))
        print(f"{'PASS' if eq else 'FAIL'}  {name}" + ("" if eq else f"  cex={cex[1]}"))
        ok &= eq
    return ok

if __name__ == "__main__":
    b = sys.argv[2] if len(sys.argv) > 2 else "engine/target/release/peanut"
    sys.exit(0 if selftest(b) else 1)
