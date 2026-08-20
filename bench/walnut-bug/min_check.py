#!/usr/bin/env python3
"""Independent (pure-Python) verification for the suspected (and retracted)
Walnut prism-1 BRZ discrepancy. Conclusion: NOT a Walnut bug -- see ISSUE.md.

Reads a Walnut relation-automaton .txt (header = one number system per free
variable, e.g. 'msd_4 msd_4 msd_4'; state lines 'S ACC'; transition lines
'd1 d2 ... dn -> dest'). Treats it as a partial DFA over the full product
alphabet (each coordinate = that base's digit alphabet), completes missing
transitions to an explicit dead (rejecting) sink, and runs Hopcroft/partition
minimization. Also checks language equivalence of two such automata by product
reachability of the symmetric difference.

This confirms the minimal size is 466 (467 with the dead state) independently of
either prover. The suspected "1058" was never a completed BRZ result -- it was an
intermediate NFA size captured from a killed-early run and misread by our harness;
the completed BRZ automaton (466) is byte-for-byte identical to the CCLS one. So
466 is right and there is no BRZ bug. See ISSUE.md.
"""
import re, sys

def read_rel(path):
    lines = open(path).read().splitlines()
    i = 0
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    header = lines[i].split()          # e.g. ['msd_4','msd_4','msd_4']
    bases = []
    for h in header:
        m = re.search(r'_(\d+)$', h)
        bases.append(int(m.group(1)) if m else None)
    i += 1
    nvar = len(header)
    states = {}          # s -> {'acc':bool, 'trans':{letter:dest}}
    cur = None
    for line in lines[i:]:
        s = line.strip()
        if s == "" or s.startswith("#"):
            continue
        m = re.match(r'^(-?\d+)\s+(-?\d+)$', s)
        if m and "->" not in s:
            cur = int(m.group(1))
            states[cur] = {'acc': int(m.group(2)) != 0, 'trans': {}}
            continue
        m = re.match(r'^(.+?)\s*->\s*(-?\d+)$', s)
        if m:
            letter = tuple(int(x) for x in m.group(1).split())
            states[cur]['trans'][letter] = int(m.group(2))
    return bases, states

def product_alphabet(bases):
    from itertools import product
    return list(product(*[range(b) for b in bases]))

def complete(bases, states):
    """Return (n_states, acc_set, delta) as a COMPLETE DFA with a dead sink.
    States: original ids 0..max plus a fresh DEAD id. Start state = 0."""
    alpha = product_alphabet(bases)
    ids = sorted(states)
    assert ids[0] == 0
    DEAD = max(ids) + 1
    n = DEAD + 1
    acc = set(s for s in ids if states[s]['acc'])
    delta = {}
    for s in ids:
        tr = states[s]['trans']
        for a in alpha:
            delta[(s, a)] = tr.get(a, DEAD)
    for a in alpha:
        delta[(DEAD, a)] = DEAD
    return n, acc, delta, alpha, DEAD

def reachable(n, delta, alpha, start=0):
    seen = {start}; stack=[start]
    while stack:
        s = stack.pop()
        for a in alpha:
            t = delta[(s,a)]
            if t not in seen:
                seen.add(t); stack.append(t)
    return seen

def minimize(n, acc, delta, alpha, start=0):
    """Hopcroft-ish partition refinement over reachable states. Returns
    (num_min_states, includes_dead_bool) where a state is 'dead' if it and all
    its successors are non-accepting sink (we just report total incl. dead)."""
    R = reachable(n, delta, alpha, start)
    R = sorted(R)
    # initial partition: accepting vs non-accepting (within reachable)
    A = frozenset(s for s in R if s in acc)
    B = frozenset(s for s in R if s not in acc)
    P = [p for p in (A, B) if p]
    changed = True
    part_of = {}
    def rebuild(parts):
        d = {}
        for idx,p in enumerate(parts):
            for s in p: d[s]=idx
        return d
    while True:
        part_of = rebuild(P)
        newP = []
        for block in P:
            # split block by signature of (part_of[delta[s,a]] for a in alpha)
            groups = {}
            for s in block:
                sig = tuple(part_of[delta[(s,a)]] for a in alpha)
                groups.setdefault(sig, []).append(s)
            for g in groups.values():
                newP.append(frozenset(g))
        if len(newP) == len(P):
            P = newP; break
        P = newP
    return len(P), P, part_of, R

def canon_equiv(bA, sA, bB, sB):
    """Check the two automata accept the same language via product reachability."""
    assert bA == bB
    nA,accA,dA,alpha,deadA = complete(bA,sA)
    nB,accB,dB,_,deadB = complete(bB,sB)
    start=(0,0); seen={start}; stack=[start]; diff=None
    while stack:
        (x,y)=stack.pop()
        if ((x in accA) != (y in accB)):
            diff=(x,y); break
        for a in alpha:
            t=(dA[(x,a)], dB[(y,a)])
            if t not in seen:
                seen.add(t); stack.append(t)
    return diff is None, diff

if __name__ == "__main__":
    import os
    D = "/Users/andrew/maths/bench/walnut-bug"
    for name in sys.argv[1:] if len(sys.argv)>1 else ["prismfe_ccls.txt"]:
        path = os.path.join(D, name)
        bases, states = read_rel(path)
        n,acc,delta,alpha,dead = complete(bases,states)
        m,P,po,R = minimize(n,acc,delta,alpha)
        print(f"{name}: declared_states={len(states)} bases={bases} "
              f"|alphabet|={len(alpha)} completed={n} reachable={len(R)} "
              f"MINIMAL(incl dead)={m}")
