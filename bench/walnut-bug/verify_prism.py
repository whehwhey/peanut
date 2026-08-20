#!/usr/bin/env python3
"""Independent checks for the suspected (and retracted) Walnut prism-1 BRZ
discrepancy. Conclusion: NOT a Walnut bug -- BRZ returns 466 when run to
completion, agreeing with every other strategy; see ISSUE.md in this directory.
(1) Confirm prism-1.txt (Walnut DFAO) generates the same sequence as the
    Peanut morphism definition: k=4, sigma, coding 102202.
(2) Utility to read a Walnut word/def automaton .txt into a DFA for
    independent minimization + equivalence (used after BRZ output is saved).
"""
import sys, re

# ---- Peanut prism-1 definition ----
K = 4
SIGMA = {0:"0305",1:"4555",2:"2321",3:"0514",4:"1023",5:"4300"}
CODING = "102202"   # state -> output symbol

def morphism_letter(n):
    """n-th letter (0-indexed) of the fixed point of sigma starting at 0.
    Underlying state = iterate: state 0, then for each base-4 digit of n (msd),
    follow sigma. Output = CODING[state]."""
    if n == 0:
        state = 0
    else:
        digits = []
        m = n
        while m > 0:
            digits.append(m % K); m //= K
        digits.reverse()
        state = 0
        for d in digits:
            state = int(SIGMA[state][d])
    return int(CODING[state])

# ---- Walnut DFAO reader ----
def read_walnut_dfao(path):
    """Return (numsys, states) where states[s] = (output, {digit:dest})."""
    txt = open(path).read().splitlines()
    lines = [l.rstrip() for l in txt]
    # first non-empty line: number system
    idx = 0
    while idx < len(lines) and lines[idx].strip()=="":
        idx += 1
    numsys = lines[idx].strip()
    idx += 1
    states = {}
    cur = None
    for l in lines[idx:]:
        s = l.strip()
        if s == "" or s.startswith("#"):
            continue
        m = re.match(r'^(\d+)\s+(-?\d+)$', s)
        if m:  # state header: "state output"
            cur = int(m.group(1))
            states[cur] = [int(m.group(2)), {}]
            continue
        m = re.match(r'^(\d+)\s*->\s*(\d+)$', s)
        if m:
            states[cur][1][int(m.group(1))] = int(m.group(2))
    return numsys, states

def dfao_letter(states, n):
    """Value of DFAO at index n, reading base-K digits msd-first from state 0."""
    if n == 0:
        digits = [0]
    else:
        digits = []
        m = n
        while m>0:
            digits.append(m%K); m//=K
        digits.reverse()
    st = 0
    for d in digits:
        st = states[st][1][d]
    return states[st][0]

if __name__ == "__main__":
    numsys, states = read_walnut_dfao("/Users/andrew/maths/bench/walnut-bug/prism-1.txt")
    print("prism-1.txt numsys:", numsys, "states:", len(states))
    N = 20000
    mism = 0
    first = []
    for n in range(N):
        a = morphism_letter(n)
        b = dfao_letter(states, n)
        if n < 40: first.append(b)
        if a != b:
            mism += 1
            if mism <= 5:
                print(f"  MISMATCH n={n}: morphism={a} dfao={b}")
    print(f"compared {N} terms, mismatches={mism}")
    print("first 40 terms:", "".join(map(str, first)))
