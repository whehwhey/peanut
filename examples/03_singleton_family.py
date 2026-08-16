#!/usr/bin/env python3
"""Example: sweep the "singleton" family of morphisms (as used for the benchmark
panel in bench/panel.json) -- k=2, increasing alphabet size m, one word each -- and
check squarefreeness of each.

Run: python3 examples/03_singleton_family.py
"""
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "explore"))
import engine

FAMILY = {
    "single3": "def T 2 3 0 01 12 20 010",
    "single4": "def T 2 4 0 01 12 23 30 0100",
    "single5": "def T 2 5 0 01 12 23 34 40 01000",
    "single6": "def T 2 6 0 01 12 23 34 45 50 010000",
}

SQUAREFREE = "? ~E i,l. l > 0 & (A t. t < l => T[i+t] = T[i+l+t])"

for name, defline in FAMILY.items():
    script = f"mode msd\n{defline}\n{SQUAREFREE}\n"
    r = engine.run(script)
    print(name, "->", r.stdout.strip().splitlines()[-1] if r.stdout.strip() else r.stderr)
