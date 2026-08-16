#!/usr/bin/env python3
"""Example: does the Thue-Morse word contain a square (a factor ww for nonempty w)?

Thue-Morse is famously overlap-free (no factor xxx' with x' a prefix of x), but
overlap-freeness does *not* imply square-freeness -- Thue-Morse does contain squares
(e.g. "00", "11"). This decides that automatically instead of scanning by hand.

Run: python3 examples/01_thue_morse_squares.py
"""
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "explore"))
import engine

script = """
mode msd
def T 2 2 0 01 10 01
? E i,l. l > 0 & (A t. t < l => T[i+t] = T[i+l+t])
"""

r = engine.run(script)
print(r.stdout)
verdict = r.verdict()
print("Thue-Morse contains a square:", {"1": "TRUE", "0": "FALSE", "?": "unknown"}[verdict])
