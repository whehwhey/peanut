/* Peanut — in-browser engine transport.
 *
 * The playground on GitHub Pages has no server: the engine runs as WebAssembly
 * right here in the tab. This file is the WASM half of the EngineTransport
 * abstraction. It mirrors gui/serve.py exactly -- the same script it would build
 * for /api/seq, /api/export, /api/femap, /api/pic, /api/run, and the same
 * stdout-line parser -- so the one UI in app.js drives either back end unchanged.
 *
 * When the page is served by the local Python server this file is inert:
 * app.js probes /api/health, finds a server, and never touches WASM.
 */
'use strict';

const WASM = (function () {
  let engine = null;          // the wasm-bindgen module (run/set_budget/version)
  let budgetMB = 768;         // in-browser default; kept low on purpose (handoff B7)

  // ---- stdout parsing: a direct port of serve.py's parse_line / unrle -------
  const _KV = /(\w+)=(-?\d+)/g;
  function kvAll(s) {
    const d = {}; let m; _KV.lastIndex = 0;
    while ((m = _KV.exec(s))) d[m[1]] = parseInt(m[2], 10);
    return d;
  }
  function afterRows(rest) {
    const i = rest.indexOf('rows=');
    return i < 0 ? '' : rest.slice(i + 5);
  }
  function unrle(row) {
    if (!row.startsWith('~')) return row;
    let out = '';
    for (const run of row.slice(1).split('.')) {
      if (run) out += run[0].repeat(parseInt(run.slice(1) || '1', 10));
    }
    return out;
  }
  function parseLine(line) {
    line = line.replace(/\n$/, '');
    if (!line) return null;
    if (/^\s/.test(line)) return { kind: 'cont', raw: line };
    const head = line.split(' ', 1)[0];
    let rest = line.slice(head.length).trim();
    const d = { raw: line, kind: head };
    if (head === 'EXPORT') {
      try { d.automaton = JSON.parse(rest); }
      catch (e) { d.kind = 'ERR'; d.error = 'bad export json: ' + e; }
      return d;
    }
    if (head === 'SEQ') {
      Object.assign(d, kvAll(rest));
      const parts = rest.split(/\s+/).filter(Boolean);
      d.seq = parts.length ? parts[parts.length - 1] : '';
      return d;
    }
    if (head === 'FEMAP') {
      Object.assign(d, kvAll(rest.split('rows=')[0]));
      const tail = afterRows(rest);
      d.rows = tail ? tail.split(',') : [];
      return d;
    }
    if (head === 'PIC') {
      const pre = rest.split('rows=')[0];
      const headNums = pre.split(/\s+/).filter(x => /^\d+$/.test(x));
      d.w = headNums.length ? parseInt(headNums[0], 10) : 0;
      d.h = headNums.length > 1 ? parseInt(headNums[1], 10) : 0;
      Object.assign(d, kvAll(pre));
      const tail = afterRows(rest);
      d.rows = tail ? tail.split(',').map(unrle) : [];
      return d;
    }
    if (head === 'ENUM') {
      d.vars = rest.includes('vars=[') ? rest.split('vars=[')[1].split(']')[0].split(',') : [];
      const tail = rest.includes('n=') ? rest.split('n=')[1] : '';
      const parts = tail.split(/\s+/).filter(Boolean);
      d.n = parts.length && /^\d+$/.test(parts[0]) ? parseInt(parts[0], 10) : 0;
      d.tuples = parts.slice(1).filter(Boolean).map(t => t.split(',').map(x => parseInt(x, 10)));
      return d;
    }
    if (head === 'WITNESS') {
      d.assign = {};
      const pre = rest.split('::')[0];
      let m; const re = /(\w+)=(\d+)/g;
      while ((m = re.exec(pre))) d.assign[m[1]] = parseInt(m[2], 10);
      for (const drop of ['states', 'len', 'ms']) delete d.assign[drop];
    }
    if (head === 'TRUE' || head === 'FALSE') d.verdict = head === 'TRUE';
    if (rest.includes('::')) d.formula = rest.split('::').slice(1).join('::').trim();
    if (head === 'ERR') d.error = rest;
    if (head === 'OK') {
      const sub = rest ? rest.split(' ', 1)[0] : '';
      d.kind = 'OK'; d.what = sub;
      if ((sub === 'let' || sub === 'learnfe') && rest.split(/\s+/).length > 1) {
        d.name = rest.split(/\s+/)[1].split('(')[0];
      }
    }
    Object.assign(d, kvAll(rest));
    return d;
  }
  function parseStdout(text) {
    return text.split('\n').map(parseLine).filter(Boolean);
  }

  // ---- run one script through the wasm engine ------------------------------
  function runRaw(script) {
    if (!engine) throw new Error('wasm engine not loaded');
    engine.set_budget(budgetMB);
    try {
      return engine.run(script);
    } catch (e) {
      // A budget trip (or any panic) aborts the wasm instance and poisons its
      // linear memory. Drop it so route() re-instantiates a fresh one next call.
      engine = null;
      throw new Error('the engine stopped at the memory budget or crashed; '
        + 'the instance was reset — try a smaller case');
    }
  }
  function resultPayload(out) {
    const budget = /memory budget/.test(out);
    return {
      ok: !budget, rc: budget ? 3 : 0, timed_out: false, budget,
      killed: false, secs: 0, stdout: out, lines: parseStdout(out),
      stderr_tail: '', events: [],
    };
  }

  // ---- script builder: a port of serve.py's Handler._script_for -----------
  function scriptFor(qs, tail) {
    const defline = (qs.def || '').trim();
    if (!defline.startsWith('def ')) throw new Error("def= must be a full 'def T ...' line");
    const mode = qs.mode === 'lsd' ? 'lsd' : 'msd';
    const pre = (qs.pre || '');
    let head = `mode ${mode}\n${defline}\n`;
    if (pre.trim()) head += pre.trim() + '\n';
    return head + tail + '\n';
  }
  const clampInt = (v, def, lo, hi) => {
    let n = parseInt(v, 10); if (!Number.isFinite(n)) n = def;
    return Math.max(lo, Math.min(hi, n));
  };
  const findLine = (lines, kind) => lines.find(l => l.kind === kind);

  // ---- the endpoints, matching gui/serve.py's routes ----------------------
  function epSeq(qs) {
    const n = clampInt(qs.n, 240, 1, 300000);
    const lines = parseStdout(runRaw(scriptFor(qs, 'seq ' + n)));
    const l = findLine(lines, 'SEQ');
    if (l) return { seq: l.seq || '', k: l.k, n: l.n };
    return { error: (lines.map(x => x.raw).join('\n').trim()) || 'no output' };
  }
  function epExport(qs) {
    const name = qs.name || 'T';
    if (!/^[A-Za-z_]\w*$/.test(name)) return { error: 'bad predicate name' };
    const lines = parseStdout(runRaw(scriptFor(qs, 'export ' + name)));
    const l = findLine(lines, 'EXPORT');
    if (l) return { automaton: l.automaton, lines, secs: 0 };
    const budget = lines.some(x => x.kind === 'ERR' && /budget/.test(x.raw));
    return { error: 'no automaton', lines, budget, timed_out: false };
  }
  function epFemap(qs) {
    const i0 = parseInt(qs.i0 || '0', 10) || 0;
    const j0 = parseInt(qs.j0 || '0', 10) || 0;
    const size = clampInt(qs.size, 96, 1, 512);
    const l = Math.max(0, parseInt(qs.l || '4', 10) || 0);
    const lines = parseStdout(runRaw(scriptFor(qs, `fe_map ${i0} ${j0} ${size} ${l}`)));
    const f = findLine(lines, 'FEMAP');
    if (f) return { i0, j0, size, l, rows: f.rows, ms: f.ms || 0 };
    return { error: (lines.map(x => x.raw).join('\n').trim()) || 'fe_map produced no grid' };
  }
  function epPic(qs) {
    const name = qs.name || 'T';
    if (!/^[A-Za-z_]\w*$/.test(name)) return { error: 'bad predicate name' };
    const w = clampInt(qs.w, 128, 1, 4096);
    const h = clampInt(qs.h, 128, 1, 4096);
    if (w * h > (1 << 20)) return { error: `${w}x${h} exceeds the 2^20-cell cap` };
    const i0 = Math.max(0, parseInt(qs.i0 || '0', 10) || 0);
    const j0 = Math.max(0, parseInt(qs.j0 || '0', 10) || 0);
    const scale = Math.max(1, parseInt(qs.scale || '1', 10) || 1);
    const lines = parseStdout(runRaw(scriptFor(qs, `pic ${name} ${w} ${h} ${i0} ${j0} ${scale}`)));
    const l = findLine(lines, 'PIC');
    if (l) return {
      w: l.w, h: l.h, i0: l.i0 != null ? l.i0 : i0, j0: l.j0 != null ? l.j0 : j0,
      scale: l.scale != null ? l.scale : scale, vals: l.vals != null ? l.vals : 2,
      ms: l.ms || 0, rows: l.rows, name,
    };
    const err = lines.find(x => x.kind === 'ERR');
    return { error: (err && err.raw) || 'pic produced no picture' };
  }
  function epRun(body) {
    const script = (body.script || '').trim();
    if (!script) return { error: 'empty script' };
    if (body.mem_mb) budgetMB = Math.min(parseInt(body.mem_mb, 10) || budgetMB, 1024);
    return resultPayload(runRaw(script));
  }
  function health() {
    return {
      ok: true, engine: `peanut ${engine ? engine.version() : '?'} (wasm, in-browser)`,
      exists: true, free_mb: budgetMB, mem_mb: budgetMB,
    };
  }

  // ---- the transport interface app.js talks to ----------------------------
  function parseQuery(path) {
    const q = path.indexOf('?');
    const out = {};
    if (q < 0) return { path, qs: out };
    const sp = new URLSearchParams(path.slice(q + 1));
    for (const [k, v] of sp) out[k] = v;
    return { path: path.slice(0, q), qs: out };
  }

  return {
    get available() { return true; },
    get budgetMB() { return budgetMB; },
    set budgetMB(v) { budgetMB = v; },

    // Load the wasm module. The glue is an ES module (wasm-bindgen --target web),
    // so we import() it; its default export fetches peanut_bg.wasm relative to
    // this page. Returns false if the bundle is absent (then app.js stays on the
    // server path).
    async init() {
      if (engine) return true;
      try {
        const mod = await import('./peanut.js');
        await mod.default();      // instantiate; fetches ./peanut_bg.wasm
        mod.set_budget(budgetMB);
        engine = mod;
        return true;
      } catch (e) {
        console.warn('wasm engine unavailable:', e);
        return false;
      }
    },

    // Route a request the way serve.py would. `method` GET/POST, `path` an
    // /api/... string (with query for GET), `body` a JSON string for POST.
    async route(method, path, body) {
      const { path: p, qs } = parseQuery(path);
      const b = body ? (typeof body === 'string' ? JSON.parse(body) : body) : {};
      // synchronous engine; yield once so a "starting…" paint lands first
      await new Promise(r => setTimeout(r, 0));
      if (!engine) await WASM.init();      // recover from a previous trap
      try {
        if (p === '/api/health') return health();
        if (p === '/api/library') return await (await fetch('./library.json')).json();
        if (p === '/api/seq') return epSeq(qs);
        if (p === '/api/export') return epExport(qs);
        if (p === '/api/femap') return epFemap(qs);
        if (p === '/api/pic') return epPic(qs);
        if (p === '/api/run') return epRun(b);
        if (p === '/api/job') return { error: 'streaming needs the local server' };
        if (p.startsWith('/api/cancel/')) return { cancelled: false };
        return { error: 'no such endpoint: ' + p };
      } catch (e) {
        return { error: String(e && e.message || e) };
      }
    },
  };
})();

window.WASM = WASM;
