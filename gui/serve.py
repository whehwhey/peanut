#!/usr/bin/env python3
"""Peanut — local web front end for the engine.

    python3 gui/serve.py            # http://127.0.0.1:7373 (loopback only)
    python3 gui/serve.py --lan      # http://0.0.0.0:7373, reachable on the LAN (phone)
    python3 gui/serve.py --port 8080

Python standard library only.  Every engine process is launched through
`explore/engine.py`, so the GUI sits inside the three memory guards described in
docs/GUARD.md rather than beside them: admission control on free RAM, an RSS watchdog,
AM_MEM_MB inside the engine, and the system LaunchAgent above all of it.

API
    GET  /api/library                      sequences + formula examples
    GET  /api/health
    POST /api/run     {script,...}         run to completion, parsed result
    POST /api/job     {script,...}         start a streaming job -> {job}
    GET  /api/stream/<job>                 SSE: progress events, stdout lines, result
    POST /api/cancel/<job>
    GET  /api/seq     ?def=&n=&mode=       first n symbols of T
    GET  /api/export  ?def=&name=&pre=     one automaton as JSON
    GET  /api/femap   ?def=&i0=&j0=&size=&l=&mode=
    GET  /api/pic     ?def=&name=&pre=&w=&h=&i0=&j0=&scale=&mode=

Andrew Hingston — MIT.
"""
import json
import os
import re
import socket
import subprocess
import sys
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STATIC = os.path.join(os.path.dirname(os.path.abspath(__file__)), "static")
sys.path.insert(0, os.path.join(ROOT, "explore"))
import engine  # noqa: E402  (path set above; this is the only sanctioned launcher)

MAX_BODY = 1 << 20        # a formula script; anything larger is a mistake
JOB_TTL = 1800            # seconds a finished job stays readable

# --------------------------------------------------------------------- parsing

_KV = re.compile(r"(\w+)=(-?\d+)")


def unrle(row):
    """One `pic` row: either W hex digits, or `~<hex><count>.<hex><count>…`."""
    if not row.startswith("~"):
        return row
    out = []
    for run in row[1:].split("."):
        if run:
            out.append(run[0] * int(run[1:] or 1))
    return "".join(out)


def parse_line(line):
    """One engine stdout line -> a dict the front end can render without regexes."""
    line = line.rstrip("\n")
    if not line:
        return None
    if line[:1].isspace():                    # continuation of a multi-line answer (dfa)
        return {"kind": "cont", "raw": line}
    head = line.split(" ", 1)[0]
    rest = line[len(head):].strip()
    d = {"raw": line, "kind": head}
    if head == "EXPORT":
        try:
            d["automaton"] = json.loads(rest)
        except Exception as e:
            d["kind"], d["error"] = "ERR", f"bad export json: {e}"
        return d
    if head == "SEQ":
        d.update({k: int(v) for k, v in _KV.findall(rest)})
        d["seq"] = rest.split()[-1] if rest.split() else ""
        return d
    if head == "FEMAP":
        d.update({k: int(v) for k, v in _KV.findall(rest.split("rows=")[0])})
        d["rows"] = rest.split("rows=", 1)[1].split(",") if "rows=" in rest else []
        return d
    if head == "PIC":
        pre = rest.split("rows=", 1)[0]
        head_nums = [x for x in pre.split() if x.isdigit()]
        d["w"] = int(head_nums[0]) if head_nums else 0
        d["h"] = int(head_nums[1]) if len(head_nums) > 1 else 0
        d.update({k: int(v) for k, v in _KV.findall(pre)})
        d["rows"] = [unrle(r) for r in rest.split("rows=", 1)[1].split(",")] if "rows=" in rest else []
        return d
    if head == "ENUM":
        d["vars"] = rest.split("vars=[", 1)[1].split("]", 1)[0].split(",") if "vars=[" in rest else []
        tail = rest.split("n=", 1)[1] if "n=" in rest else ""
        parts = tail.split()
        d["n"] = int(parts[0]) if parts and parts[0].isdigit() else 0
        d["tuples"] = [[int(x) for x in t.split(",")] for t in parts[1:] if t]
        return d
    if head == "WITNESS":
        d["assign"] = {m.group(1): int(m.group(2)) for m in re.finditer(r"(\w+)=(\d+)", rest.split("::")[0])}
        for drop in ("states", "len", "ms"):
            d["assign"].pop(drop, None)
    if head in ("TRUE", "FALSE"):
        d["verdict"] = head == "TRUE"
    if "::" in rest:
        d["formula"] = rest.split("::", 1)[1].strip()
    if head == "ERR":
        d["error"] = rest
    if head == "OK":
        sub = rest.split(" ", 1)[0] if rest else ""
        d["kind"] = "OK"
        d["what"] = sub
        if sub in ("let", "learnfe") and len(rest.split()) > 1:
            d["name"] = rest.split()[1].split("(")[0]
    d.update({k: int(v) for k, v in _KV.findall(rest)})
    return d


def parse_stdout(text):
    return [p for p in (parse_line(l) for l in text.split("\n")) if p]


def result_payload(r, events=None):
    return {
        "ok": r.ok,
        "rc": r.rc,
        "timed_out": r.timed_out,
        "budget": r.budget,
        "killed": r.killed,
        "secs": round(r.secs, 3),
        "stdout": r.stdout,
        "lines": parse_stdout(r.stdout),
        "stderr_tail": "\n".join(
            l for l in r.stderr.split("\n") if l and not l.startswith("{")
        )[-4000:],
        "events": events or [],
    }


def run_script(script, timeout=60, mem_mb=None, cap=None):
    return engine.run(script, timeout=timeout, mem_mb=mem_mb, cap=cap)


# ------------------------------------------------------------------------ jobs

class Job:
    def __init__(self, script, timeout, mem_mb, cap):
        self.id = uuid.uuid4().hex[:12]
        self.script = script
        self.events = []          # append-only; SSE readers hold an index
        self.result = None
        self.done = False
        self.proc = None
        self.created = time.time()
        self.lock = threading.Lock()
        self.thread = threading.Thread(
            target=self._run, args=(timeout, mem_mb, cap), daemon=True)

    def push(self, ev):
        with self.lock:
            self.events.append(ev)

    def _run(self, timeout, mem_mb, cap):
        self.push({"ev": "queued"})
        try:
            r = engine.run_stream(
                self.script,
                on_event=self.push,
                on_line=lambda l: self.push({"ev": "line", "line": l, "parsed": parse_line(l)}),
                on_spawn=lambda p: setattr(self, "proc", p),
                timeout=timeout, mem_mb=mem_mb, cap=cap)
            self.result = result_payload(r)
        except Exception as e:                                  # never leave a reader hanging
            self.result = {"ok": False, "rc": None, "timed_out": False, "budget": False,
                           "killed": False, "secs": 0, "stdout": "", "lines": [],
                           "stderr_tail": f"{type(e).__name__}: {e}", "events": []}
        self.push({"ev": "result", "result": self.result})
        self.done = True

    def cancel(self):
        p = self.proc
        if p is not None:
            try:
                p.kill()
            except Exception:
                pass
        return self.proc is not None


JOBS = {}
JOBS_LOCK = threading.Lock()


def reap_jobs():
    now = time.time()
    with JOBS_LOCK:
        for jid in [j for j, job in JOBS.items() if job.done and now - job.created > JOB_TTL]:
            JOBS.pop(jid, None)


# --------------------------------------------------------------------- library

def _gtm(p, coding):
    """s_2(n) mod p as a 2-uniform morphism: state a -> a, (a+1) mod p."""
    words = " ".join(f"{a}{(a + 1) % p}" for a in range(p))
    return f"def T 2 {p} 0 {words} {coding}"


def _sum_mod(base, q):
    """s_base(n) mod q as a base-uniform morphism on q letters: a -> a, a+1, …, a+base-1."""
    words = " ".join("".join(str((a + t) % q) for t in range(base)) for a in range(q))
    return f"def T {base} {q} 0 {words} " + "".join(str(a) for a in range(q))


def _carryfree(base, q):
    """`i + j has no carries in base`, written in the logic.

    Kummer: adding i and j in base `p` carries `c` times, and
    s(i) + s(j) - s(i+j) = (p-1)·c.  Over the auxiliary sequence T[n] = s_base(n) mod q
    that identity is a finite disjunction over the q² possible (T[i], T[j]) pairs — the
    only shape of it the logic can say, since it compares letters and cannot add them.
    Sound as long as (p-1)·c ≢ 0 (mod q) for every c in 1..carries, which is why the
    window is bounded below.
    """
    return " | ".join(f"(T[i]={a} & T[j]={b} & T[i+j]={(a + b) % q})"
                      for a in range(q) for b in range(q))


def load_library():
    seqs = []

    def add(sid, name, defline, note, group):
        seqs.append({"id": sid, "name": name, "def": defline, "note": note, "group": group})

    # The classics and the PRISM draws, verbatim from bench/panel.json where possible so
    # the GUI and the benchmark cannot drift apart.
    panel = {}
    try:
        with open(os.path.join(ROOT, "bench", "panel.json")) as f:
            panel = dict(json.load(f))
    except Exception:
        pass

    def d(key, fallback):
        return panel.get(key, fallback)

    add("thue-morse", "Thue–Morse", d("thue-morse", "def T 2 2 0 01 10 01"),
        "parity of the binary digit sum; overlap-free", "classical")
    add("period-doubling", "Period-doubling", d("period-doubling", "def T 2 2 0 01 00 01"),
        "fixed point of 0→01, 1→00", "classical")
    add("rudin-shapiro", "Rudin–Shapiro", d("rudin-shapiro", "def T 2 4 0 01 02 31 32 0011"),
        "parity of the count of 11 in binary", "classical")
    add("paperfolding", "Regular paperfolding", d("paperfolding", "def T 2 4 0 01 21 03 23 0011"),
        "the dragon curve fold sequence", "classical")
    add("cantor", "Cantor", d("cantor", "def T 3 2 0 010 111 01"),
        "indicator of a ternary expansion with no 1", "classical")
    add("mephisto", "Mephisto Waltz", d("mephisto", "def T 3 2 0 001 110 01"),
        "3-uniform, 001 / 110", "classical")
    add("tribonacci-ish", "Champion m=5", d("champion-m5", "def T 2 5 0 01 43 30 33 24 10010"),
        "sweep champion: |FE| = 199, a sink state makes T thin", "sweep")
    add("k3m3-a", "k3m3 artefact A", d("k3m3-artefact-a", "def T 3 3 0 011 122 110 100"),
        "Walnut needs 444 s here; the engine 0.05 s", "sweep")
    add("k3m3-b", "k3m3 artefact B", d("k3m3-artefact-b", "def T 3 3 0 021 000 212 011"),
        "msd/lsd artefact case", "sweep")
    add("prism-1", "PRISM-1", d("prism-1", "def T 4 6 0 0305 4555 2321 0514 1023 4300 102202"),
        "k=4, m=6 random draw; |FE| = 467, Walnut OOMs", "prism")
    add("prism-a", "PRISM-a", d("prism-a", "def T 2 4 0 03 33 21 21 1101"), "random draw", "prism")
    add("prism-d", "PRISM-d", d("prism-d", "def T 3 3 0 021 121 010 101"), "random draw", "prism")
    add("tail-a", "tail-a", d("tail-a", "def T 2 7 0 02 15 04 36 43 01 10 1010100"),
        "censored tail: |FE| = 1165 at 6 GB", "tail")
    add("tail-b", "tail-b", d("tail-b", "def T 3 5 0 014 421 120 202 323 01100"),
        "learnfe only: |FE| = 1000", "tail")
    add("tail-c", "tail-c", d("tail-c", "def T 2 6 0 05 23 44 42 51 10 000010"),
        "the direct construction dies at 6 GB in both digit orders; learnfe gives 1382", "tail")
    for p in range(3, 8):
        add(f"single{p}", f"[s₂ ≡ 1 mod {p}]", _gtm(p, "01" + "0" * (p - 2)),
            f"binary coding of the digit-sum group automaton; |FE| grows ≈ 3p⁴", "singleton")
    for p in range(2, 8):
        add(f"gtm{p}", f"GTM {p} (s₂ mod {p})", _gtm(p, "".join(str(a) for a in range(p))),
            "full output: the same automaton, nothing collapsed by the coding", "gtm")

    SQ = "(A t. t < {n} => T[{i}+t] = T[{i}+{n}+t])"
    PAL = "(A t,u. t+u+1 = {n} => T[{i}+t] = T[{i}+u])"
    BORD = ("(E b,j. b >= 1 & b < {n} & j + b = {i} + {n} & "
            "(A t. t < b => T[{i}+t] = T[j+t]))")

    ex = []

    def q(eid, name, script, note, group):
        ex.append({"id": eid, "name": name, "script": script, "note": note, "group": group})

    q("square-free", "Square-free?",
      "? ~ E i,n. n>=1 & " + SQ.format(i="i", n="n"),
      "no factor is ww", "powers")
    q("has-square", "A square, with witness",
      "witness n>=1 & " + SQ.format(i="i", n="n"),
      "shortest (i,n) whose base-k word the automaton accepts", "powers")
    q("cube-free", "Cube-free?",
      "? ~ E i,n. n>=1 & (A t. t < 2*n => T[i+t] = T[i+n+t])",
      "no factor is www", "powers")
    q("overlap-free", "Overlap-free?",
      "? ~ E i,n. n>=1 & (A t. t <= n => T[i+t] = T[i+n+t])",
      "no factor of exponent above 2", "powers")
    q("fourth-power-free", "4th-power-free?",
      "? ~ E i,n. n>=1 & (A t. t < 3*n => T[i+t] = T[i+n+t])", "", "powers")
    q("crit-exp", "Critical exponent probe 7/3",
      "? E i,n,L. n>=1 & 3*L >= 7*n & (A t. t+n < L => T[i+t] = T[i+n+t])",
      "run the ladder 2/1 … 5/1 to bracket the critical exponent", "powers")
    q("crit-ladder", "Critical exponent ladder",
      "\n".join(
          f"? E i,n,L. n>=1 & {b}*L >= {a}*n & (A t. t+n < L => T[i+t] = T[i+n+t])"
          for a, b in [(2, 1), (7, 3), (5, 2), (8, 3), (3, 1), (7, 2), (4, 1)]),
      "the exponent is the last TRUE", "powers")
    q("square-periods", "Periods that admit a square",
      "dfa n>=1 & " + SQ.format(i="i", n="n"),
      "the set of n such that some ww of period n occurs", "powers")
    q("has-pal", "Palindromic factors?",
      "? E i,n. n>=3 & " + PAL.format(i="i", n="n"), "", "palindromes")
    q("arb-pal", "Arbitrarily long palindromes?",
      "? A n. E i,m. m >= n & " + PAL.format(i="i", n="m"), "", "palindromes")
    q("pal-positions", "Palindrome of length 9 — positions",
      "enum 120 " + PAL.format(i="i", n="9"),
      "paints every position starting a length-9 palindrome onto the tape", "palindromes")
    q("unbordered", "Unbordered factor of every length?",
      "? A n. n>=1 => E i. ~" + BORD.format(i="i", n="n"), "", "borders")
    q("unbordered-lengths", "Lengths admitting an unbordered factor",
      "dfa E i. ~" + BORD.format(i="i", n="n"), "", "borders")
    q("right-special", "Right-special factors",
      "let RS(i,n) E j. (A t. t < n => T[i+t] = T[j+t]) & T[i+n] != T[j+n]\n"
      "enum 80 $RS(i,4)",
      "positions whose length-4 factor extends two ways", "special")
    q("rs-count", "Is every length right-special?",
      "let RS(i,n) E j. (A t. t < n => T[i+t] = T[j+t]) & T[i+n] != T[j+n]\n"
      "? A n. E i. $RS(i,n)", "", "special")
    q("fe", "FE — equality of factors",
      "let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]\nmem\nexport FE",
      "the direct construction; the one Khodier's Open Problem 1 is about", "fe")
    q("fe-learn", "FE by guess-and-verify (learnfe)",
      "learnfe FE\nmem\nexport FE",
      "active learning against an LCP oracle, checked against the recurrence", "fe")
    q("fe-use", "Recurrence of a long factor",
      "learnfe G\nwitness i < j & $G(i,j,60)\nfinite E j. j > i & $G(i,j,200)",
      "where the length-60 factor at 0 recurs, and whether every factor recurs", "fe")
    q("peltomaki", "Peltomäki extRS2 stack",
      "let factorEq(i,n,m) A t. t < i => T[n+t] = T[m+t]\n"
      "let isRS(i,n) E p,q. $factorEq(i,n,p) & $factorEq(i,n,q) & T[p+i] != T[q+i]\n"
      "let extRS2(i,j,n) i<j & (E m1,m2. $isRS(j,m1) & $isRS(j,m2) & $factorEq(i,m1,m2) "
      "& $factorEq(i,n,m1) & T[m1+i] != T[m2+i])\n"
      "? E i,j,n. $extRS2(i,j,n)",
      "the published Walnut OOM case; runs here in milliseconds", "special")
    q("recurrent", "Every factor occurs infinitely often?",
      "? A i,n,N. E j. j >= N & (A t. t < n => T[i+t] = T[j+t])", "", "structure")
    q("mirror", "Closed under reversal?",
      "? A i,n. E j. (A t,u. t+u+1 = n => T[i+t] = T[j+u])", "", "structure")
    q("positions-1", "Positions where T = 1",
      "enum 200 T[i] = 1", "paints the tape", "structure")
    q("runs", "Lengths of runs of 0",
      "dfa E i. (A t. t < n => T[i+t] = 0)", "", "structure")
    q("ap", "Arbitrarily long APs of step 3 that are constant 0",
      "? A n. E i. (A t. t < n => T[i+3*t] = 0)", "", "structure")
    # ---------------------------------------------------------------- shapes
    # Each shape is a 2-variable predicate the Shapes view draws as a bitmap.  `pre` is
    # run before the picture in the same session; `def` overrides the chosen sequence
    # when the shape only makes sense over a particular one.
    sh = []

    def pic(sid, name, body, note, pre="", defline=None, extra=None):
        e = {"id": sid, "name": name, "body": body, "note": note, "pre": pre}
        if defline:
            e["def"] = defline
        if extra:
            e.update(extra)
        sh.append(e)

    pic("eq", "T[i] = T[j]", "T[i]=T[j]",
        "the agreement table: the sequence against itself")
    pic("add", "T[i+j] — the addition table", "",
        "the DFAO itself, one output letter per cell (no predicate)",
        extra={"dfao": True})
    pic("addrow", "T[i+j] = T[i]", "T[i+j]=T[i]",
        "where the addition table repeats its own row header")
    pic("neq", "T[i] ≠ T[j]", "T[i]!=T[j]", "the complement of the agreement table")
    pic("fe8", "FE(i, j, 8) — equal length-8 factors", "$FE(i,j,8)",
        "the FE heatmap at a fixed length, drawn from the automaton instead of the LCP walk",
        pre="let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]")
    pic("fe-learn", "FE(i, j, 32) via learnfe", "$FE(i,j,32)",
        "same picture, from the guess-and-verify construction", pre="learnfe FE")
    pic("carry2", "Sierpiński — carry-free binary addition", _carryfree(2, 10),
        "C(i+j, i) is odd exactly when i+j carries nowhere (Kummer); valid for i, j < 512",
        defline=_sum_mod(2, 10), extra={"window": 512})
    pic("carry3", "Pascal mod 3 — carry-free base-3 addition", _carryfree(3, 7),
        "3 divides C(i+j, i) exactly when base-3 addition carries (Lucas/Kummer); i, j < 729",
        defline=_sum_mod(3, 7), extra={"window": 729})
    pic("pow", "i + j is a power of k", "pow(i+j)",
        "the anti-diagonals at the powers of the base")
    pic("sq", "a square of period j−i starts at i", "$FE(i,j,4) & i<j",
        "pairs whose length-4 factors agree, above the diagonal",
        pre="let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]")

    return {"sequences": seqs, "examples": ex, "shapes": sh}


LIBRARY = None


def library():
    global LIBRARY
    if LIBRARY is None:
        LIBRARY = load_library()
    return LIBRARY


# ------------------------------------------------------------------- http glue

CTYPE = {".html": "text/html; charset=utf-8", ".js": "application/javascript; charset=utf-8",
         ".css": "text/css; charset=utf-8", ".svg": "image/svg+xml", ".png": "image/png",
         ".ico": "image/x-icon", ".json": "application/json", ".woff2": "font/woff2"}


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    server_version = "peanut"

    def log_message(self, fmt, *args):
        if os.environ.get("PEANUT_QUIET"):
            return
        sys.stderr.write("%s  %s\n" % (time.strftime("%H:%M:%S"), fmt % args))

    # ---- helpers
    def _json(self, obj, code=200):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _err(self, msg, code=400):
        self._json({"error": msg}, code)

    def _admitted(self, mem_mb=None):
        """True if an engine may launch now; otherwise answer 503 + {waiting} and
        return False.  Keeps a starved small machine from hanging the request
        forever inside engine._admit()."""
        waiting = engine.admit_status(mem_mb)
        if waiting is None:
            return True
        self._json({"waiting": waiting}, 503)
        return False

    def _body(self):
        n = int(self.headers.get("Content-Length") or 0)
        if n > MAX_BODY:
            raise ValueError("body too large")
        raw = self.rfile.read(n) if n else b"{}"
        return json.loads(raw or b"{}")

    def _static(self, path):
        if path.startswith("/static/"):
            path = path[len("/static"):]
        name = os.path.normpath(path).lstrip("/")
        full = os.path.join(STATIC, name)
        if not os.path.abspath(full).startswith(os.path.abspath(STATIC)) or not os.path.isfile(full):
            self._err("not found", 404)
            return
        with open(full, "rb") as f:
            body = f.read()
        self.send_response(200)
        self.send_header("Content-Type", CTYPE.get(os.path.splitext(full)[1], "application/octet-stream"))
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    # ---- routes
    def do_GET(self):
        u = urlparse(self.path)
        p, qs = u.path, parse_qs(u.query)
        try:
            if p in ("/", "/index.html"):
                return self._static("index.html")
            if p == "/api/health":
                return self._json({"ok": True, "engine": engine.ENGINE,
                                   "exists": os.path.exists(engine.ENGINE),
                                   "free_mb": engine.free_mb(), "mem_mb": engine.MEM_MB})
            if p == "/api/library":
                return self._json(library())
            if p == "/api/seq":
                return self._seq(qs)
            if p == "/api/export":
                return self._export(qs)
            if p == "/api/femap":
                return self._femap(qs)
            if p == "/api/pic":
                return self._pic(qs)
            if p.startswith("/api/stream/"):
                return self._stream(p.rsplit("/", 1)[-1])
            if p.startswith("/api/"):
                return self._err("no such endpoint", 404)
            return self._static(p)
        except BrokenPipeError:
            pass
        except Exception as e:
            try:
                self._err(f"{type(e).__name__}: {e}", 500)
            except Exception:
                pass

    def do_POST(self):
        u = urlparse(self.path)
        try:
            if u.path == "/api/run":
                b = self._body()
                script = (b.get("script") or "").strip()
                if not script:
                    return self._err("empty script")
                if not self._admitted(b.get("mem_mb")):
                    return
                r = run_script(script, timeout=int(b.get("timeout", 60)),
                               mem_mb=b.get("mem_mb"), cap=b.get("cap"))
                return self._json(result_payload(r))
            if u.path == "/api/job":
                b = self._body()
                script = (b.get("script") or "").strip()
                if not script:
                    return self._err("empty script")
                if not self._admitted(b.get("mem_mb")):
                    return
                reap_jobs()
                job = Job(script, int(b.get("timeout", 600)), b.get("mem_mb"), b.get("cap"))
                with JOBS_LOCK:
                    JOBS[job.id] = job
                job.thread.start()
                return self._json({"job": job.id})
            if u.path.startswith("/api/cancel/"):
                jid = u.path.rsplit("/", 1)[-1]
                with JOBS_LOCK:
                    job = JOBS.get(jid)
                if not job:
                    return self._err("no such job", 404)
                return self._json({"cancelled": job.cancel()})
            return self._err("no such endpoint", 404)
        except BrokenPipeError:
            pass
        except Exception as e:
            try:
                self._err(f"{type(e).__name__}: {e}", 500)
            except Exception:
                pass

    # ---- api implementations
    def _script_for(self, qs, tail):
        defline = (qs.get("def", [""])[0] or "").strip()
        if not defline.startswith("def "):
            raise ValueError("def= must be a full 'def T ...' line")
        mode = qs.get("mode", ["msd"])[0]
        pre = qs.get("pre", [""])[0]
        head = f"mode {'lsd' if mode == 'lsd' else 'msd'}\n{defline}\n"
        if pre.strip():
            head += pre.strip() + "\n"
        return head + tail + "\n"

    def _seq(self, qs):
        # the tape asks for a few thousand; the Shapes square asks for N^2 up to 512^2
        if not self._admitted():
            return
        n = max(1, min(int(qs.get("n", ["240"])[0]), 300000))
        r = run_script(self._script_for(qs, f"seq {n}"), timeout=30)
        for l in parse_stdout(r.stdout):
            if l["kind"] == "SEQ":
                return self._json({"seq": l.get("seq", ""), "k": l.get("k"), "n": l.get("n")})
        return self._err(r.stdout.strip() or (r.stderr or "").strip() or "no output", 500)

    def _export(self, qs):
        name = qs.get("name", ["T"])[0]
        if not re.fullmatch(r"[A-Za-z_]\w*", name):
            return self._err("bad predicate name")
        if not self._admitted(qs.get("mem_mb", [None])[0]):
            return
        script = self._script_for(qs, f"export {name}")
        r = run_script(script, timeout=int(qs.get("timeout", ["120"])[0]),
                       mem_mb=qs.get("mem_mb", [None])[0])
        lines = parse_stdout(r.stdout)
        for l in lines:
            if l["kind"] == "EXPORT":
                return self._json({"automaton": l["automaton"], "lines": lines,
                                   "secs": round(r.secs, 3)})
        return self._json({"error": "no automaton", "lines": lines,
                           "stdout": r.stdout, "budget": r.budget,
                           "timed_out": r.timed_out}, )

    def _femap(self, qs):
        i0 = int(qs.get("i0", ["0"])[0])
        j0 = int(qs.get("j0", ["0"])[0])
        size = max(1, min(int(qs.get("size", ["96"])[0]), 512))
        l = max(0, int(qs.get("l", ["4"])[0]))
        if not self._admitted():
            return
        r = run_script(self._script_for(qs, f"fe_map {i0} {j0} {size} {l}"),
                       timeout=int(qs.get("timeout", ["120"])[0]))
        for line in parse_stdout(r.stdout):
            if line["kind"] == "FEMAP":
                return self._json({"i0": i0, "j0": j0, "size": size, "l": l,
                                   "rows": line["rows"], "ms": line.get("ms", 0)})
        return self._err(r.stdout.strip() or "fe_map produced no grid", 500)

    def _pic(self, qs):
        name = qs.get("name", ["T"])[0]
        if not re.fullmatch(r"[A-Za-z_]\w*", name):
            return self._err("bad predicate name")
        w = max(1, min(int(qs.get("w", ["128"])[0]), 4096))
        h = max(1, min(int(qs.get("h", ["128"])[0]), 4096))
        if w * h > (1 << 20):
            return self._err(f"{w}x{h} exceeds the 2^20-cell cap")
        i0 = max(0, int(qs.get("i0", ["0"])[0]))
        j0 = max(0, int(qs.get("j0", ["0"])[0]))
        scale = max(1, int(qs.get("scale", ["1"])[0]))
        if not self._admitted(qs.get("mem_mb", [None])[0]):
            return
        script = self._script_for(qs, f"pic {name} {w} {h} {i0} {j0} {scale}")
        r = run_script(script, timeout=int(qs.get("timeout", ["120"])[0]),
                       mem_mb=qs.get("mem_mb", [None])[0])
        lines = parse_stdout(r.stdout)
        for l in lines:
            if l["kind"] == "PIC":
                return self._json({"w": l["w"], "h": l["h"], "i0": l.get("i0", i0),
                                   "j0": l.get("j0", j0), "scale": l.get("scale", scale),
                                   "vals": l.get("vals", 2), "ms": l.get("ms", 0),
                                   "rows": l["rows"], "name": name})
        err = next((l["raw"] for l in lines if l["kind"] == "ERR"), None)
        return self._err(err or r.stdout.strip() or (r.stderr or "").strip()
                         or "pic produced no picture", 500)

    def _stream(self, jid):
        with JOBS_LOCK:
            job = JOBS.get(jid)
        if not job:
            return self._err("no such job", 404)
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream; charset=utf-8")
        self.send_header("Cache-Control", "no-store")
        self.send_header("Connection", "close")       # body ends when the socket closes
        self.end_headers()
        self.close_connection = True
        i = 0
        last_beat = time.time()
        try:
            while True:
                with job.lock:
                    batch = job.events[i:]
                    i += len(batch)
                for ev in batch:
                    self.wfile.write(b"data: " + json.dumps(ev).encode() + b"\n\n")
                    last_beat = time.time()
                if batch:
                    self.wfile.flush()
                if job.done and i >= len(job.events):
                    self.wfile.write(b"event: end\ndata: {}\n\n")
                    self.wfile.flush()
                    return
                if time.time() - last_beat > 10:
                    self.wfile.write(b": beat\n\n")
                    self.wfile.flush()
                    last_beat = time.time()
                time.sleep(0.05)
        except (BrokenPipeError, ConnectionResetError):
            pass


def lan_ip():
    for iface in ("en0", "en1"):
        try:
            out = subprocess.run(["ipconfig", "getifaddr", iface],
                                 capture_output=True, text=True, timeout=2).stdout.strip()
            if out:
                return out
        except Exception:
            pass
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        ip = s.getsockname()[0]
        s.close()
        return ip
    except Exception:
        return "127.0.0.1"


def main(argv):
    port = 7373
    # Bind loopback by DEFAULT: POST /api/run executes arbitrary engine scripts, so
    # the server must not be reachable off-box unless asked for.  --lan opts into
    # 0.0.0.0 (phone / other machines on the LAN); --host wins if given explicitly.
    host = "127.0.0.1"
    lan = False
    for i, a in enumerate(argv):
        if a in ("--port", "-p") and i + 1 < len(argv):
            port = int(argv[i + 1])
        if a == "--lan":
            lan = True
            host = "0.0.0.0"
        if a == "--host" and i + 1 < len(argv):
            host = argv[i + 1]
    if not os.path.exists(engine.ENGINE):
        print(f"engine binary missing: {engine.ENGINE}\nbuild it: cd engine && cargo build --release",
              file=sys.stderr)
        return 2
    srv = ThreadingHTTPServer((host, port), Handler)
    srv.daemon_threads = True
    ip = lan_ip()
    print("\n  Peanut")
    print(f"  local   http://127.0.0.1:{port}")
    if lan or host not in ("127.0.0.1", "localhost"):
        print(f"  LAN     http://{ip}:{port}")
        print("  SECURITY: bound to the network - anyone who can reach this host can")
        print("            POST /api/run and execute engine scripts. Trusted LANs only.")
    else:
        print("  (loopback only; pass --lan to expose on your LAN, e.g. from a phone)")
    print(f"  engine  {engine.ENGINE}   budget {engine.MEM_MB} MB/job\n")
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        print("\n  stopped")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
