"""Like-for-like FE benchmark: automatheus vs Walnut 8-dev on the same sequences.
Walnut: morphism + promote + image(coding) + def FE, msd, -Xmx6g, wall-clock timeout.
Ours:   engine.run msd default cap, then lsd, then small cap -- report best and total.

Reads/writes: no results/*.json artifacts referenced directly (see code for any docs/ or in-memory-only use).

Run:
    python3 bench/walnut_fe.py [args - see __main__ / argv handling below]
"""
import os, sys, json, subprocess, time, re, shutil
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "explore"))
import engine
ROOT="/Users/andrew/maths"; os.chdir(ROOT)
W=os.path.join(ROOT,"walnut7"); JAVA="/opt/homebrew/opt/openjdk/bin/java"
def parse(d):
    p=d.split(); k=int(p[2]); m=int(p[3]); w=p[5:5+m]; c=p[5+m]; return k,m,w,c
def walnut(d, name, timeout=900):
    k,m,w,c=parse(d)
    morph=" ".join(f"{a}->{w[a]}" for a in range(m))
    cod=" ".join(f"{a}->{c[a]}" for a in range(m))
    src=(f'morphism mf{name} "{morph}";\npromote PW{name} mf{name};\nmorphism cd{name} "{cod}";\n'
         f'image {name} cd{name} PW{name};\n'
         f'def fe{name} "?msd_{k} At (t<l) => {name}[i+t]={name}[j+t]":\nexit;\n')
    t0=time.time()
    try:
        r=subprocess.run([JAVA,"-Xmx6g","-jar","target/Walnut-all.jar"],input=src,capture_output=True,text=True,timeout=timeout,cwd=W)
        out=r.stdout+r.stderr; secs=time.time()-t0
    except subprocess.TimeoutExpired as e:
        return {"walnut":"timeout","walnut_s":round(time.time()-t0,1)}
    st=None
    for line in out.split("\n"):
        mm=re.search(r"\(A t .*\):(\d+) states",line)
        if mm: st=int(mm.group(1))
    err="OutOfMemory" in out or "Exception" in out or "Error" in out
    tot=[int(x) for x in re.findall(r"Total computation time: (\d+)ms",out)]
    return {"walnut":st if st is not None else ("OOM" if "OutOfMemory" in out else "error" if err else "?"),
            "walnut_s":round(secs,1),"walnut_ms_sum":sum(tot)}
def ours(d):
    best=None; total=0; how=None
    for mode,cap in (("msd",3_000_000),("lsd",3_000_000),("msd",50_000)):
        r=engine.run(f"mode {mode}\n{d}\nlet FE(i,j,l) A t. t < l => T[i+t] = T[j+t]\nmem\n",timeout=600,cap=cap,mem_mb=6144)
        total+=r.secs
        for l in r.stdout.split("\n"):
            if l.startswith("OK let"): best=int(l.split("states=")[1].split()[0]); how=f"{mode}/cap{cap}"
        if best is not None: break
    return {"ours":best if best is not None else "fail","ours_how":how,"ours_s":round(total,1)}
if __name__=="__main__":
    seqs=json.load(open(sys.argv[1])); out=[]
    for name,d in seqs:
        row={"name":name,"def":d}; row.update(ours(d)); row.update(walnut(d,"S"+re.sub(r'\W','',name))); print(row,flush=True); out.append(row)
    json.dump(out,open(sys.argv[2],"w"),indent=0)
    print(f"\n{'name':22}{'ours':>8}{'how':>14}{'s':>7} | {'walnut':>8}{'s':>7}")
    for r in out: print(f"{r['name']:22}{str(r['ours']):>8}{str(r['ours_how']):>14}{r['ours_s']:>7} | {str(r['walnut']):>8}{r['walnut_s']:>7}")
