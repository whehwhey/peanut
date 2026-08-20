#!/usr/bin/env python3
"""Games-pack seed: the Fibonacci word, Wythoff's game, and two classical
theorems, proved live in the engine under Zeckendorf numeration.

Facts demonstrated (all engine-verified, plus brute-checked in protoD_optimize.py):
  1. `dfao F 2 0:0,1 1:0,-` under `numsys fib` IS the Fibonacci word
     (output = last Zeckendorf digit): seq 13 -> 0100101001001.
  2. enum of F[n]=0 gives 0,2,3,5,7,8,10,11,... = floor(m*phi)-1 — i.e. the
     positions of 0s, shifted by one, are the lower Wythoff sequence, the first
     coordinates of the P-positions of Wythoff's game.
  3. The Fibonacci word contains cubes but no 4th powers (critical exponent 2+phi).
Requires the seqname parser fix and the finite pad-quotient fix.
Run: python3 examples/games/wythoff_fibword.py
"""
import os, sys
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(ROOT, "explore"))
import engine

script = """
mode msd
numsys fib
dfao F 2 0:0,1 1:0,-
seq 13
enum 13 F[n]=0
? E i,l. l > 0 & (A t. t < 2*l => F[i+t] = F[i+l+t])
? E i,l. l > 0 & (A t. t < 3*l => F[i+t] = F[i+l+t])
finite F[n]=1 & n<10
"""
r = engine.run(script)
print(r.stdout)
out = r.stdout
assert "0100101001001" in out, "Fibonacci word prefix"
assert "0 2 3 5 7 8 10 11" in out, "lower Wythoff minus one"
lines = [l for l in out.splitlines() if l.startswith(("TRUE", "FALSE"))]
assert lines[0].startswith("TRUE"), "cubes exist"
assert lines[1].startswith("FALSE"), "no 4th powers"
assert "FINITE size=4 max=9" in out, "fib finite regression"
phi = (1 + 5 ** 0.5) / 2
print("lower Wythoff check:", [int(m * phi) for m in range(1, 9)],
      "== positions+1 of the enum above")
print("All assertions passed.")
