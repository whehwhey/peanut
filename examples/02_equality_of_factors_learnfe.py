#!/usr/bin/env python3
"""Example: build the equality-of-factors predicate FE(i,j,l) for a sequence via
learnfe (guess-and-verify active learning, docs/LEARNFE.md), then check the trivial
sanity property that FE(i,j,l) with l>0 implies the two factors start with the same
letter.

Run: python3 examples/02_fibonacci_word_learnfe.py
"""
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "explore"))
import engine

script = """
mode msd
def T 2 2 0 01 10 01
learnfe FE
? A i,j,l. $FE(i,j,l) => (l=0 | T[i]=T[j])
"""

r = engine.run(script)
print(r.stdout)
