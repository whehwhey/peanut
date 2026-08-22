"""TARGET 1: hunt for the equality-of-factors blowup.

Open problem (Waterloo): characterise the k-automatic sequences whose "equality of
factors" automaton blows up exponentially. The obstacle is that tools crash before enough
examples accumulate to see the pattern -- so the move is to sweep, not to solve.

    FE(i,j,l) := A t. t < l => T[i+t] = T[j+t]

Measure |FE| and the peak intermediate size against the DFAO size m, for many random
k-automatic sequences. A family whose |FE| grows like 2^m is the thing nobody has pinned
down.

Reads/writes: results/SEED.txt, results/blowup.json

Run:
    python3 explore/blowup.py
"""
import subprocess, os, json, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import engine
os.chdir(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def measure(args):
    defline, mode = args
    src = f"mode {mode}\n{defline}\nlet FE(i,j,l) A t. t < l => T[i+t] = T[j+t]\nmem\nquit\n"
    r = engine.run(src, timeout=90, cap=2_000_000)
    if not r.ok:
        # record WHY it failed: timeout / memory budget -- these are the interesting ones
        print(f"  FAIL {'timeout' if r.timed_out else 'budget' if r.budget else 'rc='+str(r.rc)}  {defline}", flush=True)
        return defline, mode, None, None
    st=None; lsd=None
    for l in r.stdout.split("\n"):
        if l.startswith("OK def"): 
            lsd=int(l.split("lsd_states=")[1].split()[0])
        if l.startswith("OK let FE"):
            st=int(l.split("states=")[1].split()[0])
    return defline, mode, st, lsd

def admissible(k,m,w,c):
    if w[0][0]!=0: return False
    seen={0}; st=[0]
    while st:
        a=st.pop()
        for x in w[a]:
            if x not in seen: seen.add(x); st.append(x)
    if len(seen)!=m or len(set(c))<2: return False
    col=list(c)
    while True:
        sig={}; new=[]
        for a in range(m):
            key=(col[a],)+tuple(col[w[a][d]] for d in range(k))
            new.append(sig.setdefault(key,len(sig)))
        if len(sig)==len(set(col)): col=new; break
        col=new
    return len(set(col))==m

SEED=open("results/SEED.txt").read().strip()
jobs=[]
for k in (2,3):
    for m in (2,3,4,5,6,7):
        out=subprocess.run(["./prism/target/release/prism","candidates",SEED,"4000",str(k),str(m),"2"],
                           capture_output=True,text=True).stdout.strip().split("\n")
        got=0
        for line in out:
            p=line.split()
            if len(p)<m+1: continue
            w=[[int(x) for x in y] for y in p[:m]]; c=[int(x) for x in p[m]]
            if max(max(y) for y in w)>=m: continue
            if not admissible(k,m,w,c): continue
            d="def T %d %d 0 %s %s"%(k,m," ".join("".join(map(str,y)) for y in w),"".join(map(str,c)))
            jobs.append((d,"msd")); got+=1
            if got>=40: break
print(f"{len(jobs)} sequences to measure", flush=True)
res=engine.pool(jobs, measure, label="blowup")
import statistics
from collections import defaultdict
by=defaultdict(list)
for d,mode,st,lsd in res:
    if st is None: continue
    p=d.split(); k=int(p[2]); m=int(p[3])
    by[(k,m)].append((st,lsd,d))
print(f"\n{'k':>3}{'m':>3}{'n':>5}{'|FE| min':>10}{'median':>9}{'max':>8}{'max/m':>9}   worst sequence")
rows=[]
for key in sorted(by):
    k,m=key; v=by[key]
    sts=[x[0] for x in v]
    worst=max(v)
    rows.append((k,m,len(v),min(sts),statistics.median(sts),max(sts)))
    print(f"{k:>3}{m:>3}{len(v):>5}{min(sts):>10}{statistics.median(sts):>9.0f}{max(sts):>8}{max(sts)/m:>9.1f}   {worst[2][:52]}")
json.dump([[d,st,lsd] for d,mode,st,lsd in res if st], open("results/blowup.json","w"))
