/* Peanut — front end.  No build step, no dependencies, no network beyond this host.
   Layout, drawing and the little relaxation solver are all in this file on purpose:
   the whole point of the tool is that you can read it. */
'use strict';

// ------------------------------------------------------------------ utilities

const $ = (id) => document.getElementById(id);
const el = (tag, cls, txt) => {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (txt !== undefined) n.textContent = txt;
  return n;
};
const clamp = (x, a, b) => Math.max(a, Math.min(b, x));
const fmt = (n) => (n === undefined || n === null) ? '—' : n.toLocaleString('en-US');

async function api(path, opts) {
  const r = await fetch(path, opts);
  const t = await r.text();
  let j;
  try { j = JSON.parse(t); } catch (e) { throw new Error(t.slice(0, 300)); }
  if (j && j.error && !j.lines) throw new Error(j.error);
  return j;
}
const post = (path, body) => api(path, {
  method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body || {})
});

// theme-aware colours read straight from the stylesheet, so the seed-derived tokens
// stay in one place and dark mode needs no second palette here
const cssv = (name) => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
let PAL = {};
function readPalette() {
  PAL = {
    paper: cssv('--paper'), panel: cssv('--panel'), panel2: cssv('--panel-2'),
    rule: cssv('--rule'), ruleSoft: cssv('--rule-soft'),
    ink: cssv('--ink'), ink2: cssv('--ink-2'), ink3: cssv('--ink-3'),
    indigo: cssv('--indigo'), amber: cssv('--amber'), green: cssv('--green'), rust: cssv('--rust'),
    dark: matchMedia('(prefers-color-scheme: dark)').matches
  };
}
readPalette();
matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
  readPalette(); Tape.draw(); Auto.draw(); Heat.draw();
});

// symbol -> colour.  Hue walk seeded by the design seed's three arms (248 / 38 / 98),
// then their midpoints, so an 8-letter alphabet never repeats a hue.
const SYMBOL_HUES = [248, 38, 98, 268, 18, 118, 208, 78];
// hue alone is not enough — two tiles must stay distinguishable in greyscale and to a
// colour-blind reader, so lightness steps down the alphabet as well
function symbolColour(s) {
  const h = SYMBOL_HUES[s % SYMBOL_HUES.length];
  const step = (s % 4) * 6;
  return PAL.dark ? `hsl(${h} 44% ${38 - step}%)` : `hsl(${h} 56% ${82 - step}%)`;
}
function symbolInk(s) {
  const h = SYMBOL_HUES[s % SYMBOL_HUES.length];
  return PAL.dark ? `hsl(${h} 45% 92%)` : `hsl(${h} 65% 17%)`;
}

function hidpi(canvas) {
  const r = window.devicePixelRatio || 1;
  const w = canvas.clientWidth, h = canvas.clientHeight;
  if (canvas.width !== Math.round(w * r) || canvas.height !== Math.round(h * r)) {
    canvas.width = Math.round(w * r); canvas.height = Math.round(h * r);
  }
  const ctx = canvas.getContext('2d');
  ctx.setTransform(r, 0, 0, r, 0, 0);
  return { ctx, w, h };
}

function roundRect(ctx, x, y, w, h, r) {
  r = Math.min(r, w / 2, h / 2);
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

// ------------------------------------------------------------------- state

const S = {
  seqs: [], examples: [],
  cur: null,                 // {id,name,def,note}
  mode: 'msd',
  seq: '', k: 2, want: 512,
  paint: null,               // {set:Set<number>, label:string}
  brackets: [],              // [{i,len,label}]
  sel: null,
  dfao: null,
  preds: {},                 // name -> script that builds it
  job: null,
};

// ------------------------------------------------------------------- mascot

const MASCOT = { sprite: null };
async function loadSprite() {
  try {
    const txt = await (await fetch('/static/mascot-sprite.svg')).text();
    const holder = el('div');
    holder.style.display = 'none';
    holder.innerHTML = txt;
    document.body.appendChild(holder);
    MASCOT.sprite = true;
  } catch (e) { MASCOT.sprite = false; }
  try {
    const logo = await (await fetch('/static/logo.svg')).text();
    $('brandmark').innerHTML = logo;
    const svg = $('brandmark').querySelector('svg');
    if (svg) { svg.setAttribute('width', '34'); svg.setAttribute('height', '34'); }
  } catch (e) {
    // logo.svg is drawn by a separate pass; a plain shell keeps the header honest until then
    $('brandmark').innerHTML =
      '<svg viewBox="0 0 32 32" width="34" height="34" aria-hidden="true">' +
      '<circle cx="16" cy="11" r="8" fill="none" stroke="currentColor" stroke-width="2"/>' +
      '<circle cx="16" cy="22" r="8" fill="none" stroke="currentColor" stroke-width="2"/></svg>';
  }
  setMascot('thinking');
}
function setMascot(kind, bounce) {
  const box = $('mascot');
  if (!box) return;
  const id = { happy: 'peanut-happy', thinking: 'peanut-thinking', oops: 'peanut-oops' }[kind] || 'peanut-thinking';
  if (MASCOT.sprite) {
    box.innerHTML = `<svg viewBox="0 0 256 256" role="img" aria-label="Peanut, ${kind}"><use href="#${id}"/></svg>`;
  } else {
    box.innerHTML = `<svg viewBox="0 0 32 32" role="img" aria-label="Peanut, ${kind}">
      <circle cx="16" cy="11" r="8" fill="none" stroke="currentColor" stroke-width="2"/>
      <circle cx="16" cy="22" r="8" fill="none" stroke="currentColor" stroke-width="2"/></svg>`;
  }
  if (bounce) { box.classList.remove('react'); void box.offsetWidth; box.classList.add('react'); }
}

// --------------------------------------------------------------------- rail

const VIEWS = [
  ['sequence', 'Sequence', 'M2 12h4M8 12h4M14 12h4M20 12h2'],
  ['automaton', 'Automaton', 'M5 6a2 2 0 1 0 .01 0M19 6a2 2 0 1 0 .01 0M12 18a2 2 0 1 0 .01 0M7 7l4 9M17 7l-4 9M7 6h10'],
  ['playground', 'Playground', 'M9 7l-5 5 5 5M15 7l5 5-5 5'],
  ['femap', 'FE heatmap', 'M4 4h16v16H4zM4 10h16M4 16h16M10 4v16M16 4v16'],
  ['morphism', 'Morphism', 'M4 8h10l-3-3M20 16H10l3 3'],
  ['live', 'Live', 'M3 12h4l2-6 3 12 2-8 2 4h5'],
];
function buildRail() {
  const rail = $('rail');
  VIEWS.forEach(([id, label, d]) => {
    const b = el('button');
    b.dataset.view = id;
    b.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"
      stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="${d}"/></svg><span>${label}</span>`;
    b.onclick = () => show(id);
    rail.appendChild(b);
  });
  const note = el('div', 'rail-note');
  note.textContent = 'every query is a proof by the Büchi–Bruyère decision procedure';
  rail.appendChild(note);
}
function show(id) {
  document.querySelectorAll('.view').forEach(v => v.classList.toggle('on', v.id === 'view-' + id));
  document.querySelectorAll('#rail button').forEach(b => b.setAttribute('aria-current', String(b.dataset.view === id)));
  location.hash = id;
  if (id === 'automaton') Auto.resize();
  if (id === 'femap') Heat.resize();
}

// --------------------------------------------------------------------- tape

const Tape = {
  off: 0, scale: 15, drag: null, pinch: null,

  init() {
    const c = $('tapeCanvas');
    c.addEventListener('pointerdown', (e) => {
      c.setPointerCapture(e.pointerId);
      Tape.drag = { x: e.clientX, off: Tape.off, moved: 0 };
    });
    c.addEventListener('pointermove', (e) => {
      if (!Tape.drag) return;
      const dx = e.clientX - Tape.drag.x;
      Tape.drag.moved = Math.max(Tape.drag.moved, Math.abs(dx));
      Tape.off = Math.max(0, Tape.drag.off - dx / Tape.scale);
      Tape.draw(); Tape.ensure();
    });
    c.addEventListener('pointerup', (e) => {
      if (Tape.drag && Tape.drag.moved < 4) {
        const r = c.getBoundingClientRect();
        Tape.select(Math.floor(Tape.off + (e.clientX - r.left) / Tape.scale));
      }
      Tape.drag = null;
    });
    c.addEventListener('pointercancel', () => { Tape.drag = null; });
    c.addEventListener('wheel', (e) => {
      e.preventDefault();
      const r = c.getBoundingClientRect();
      const at = Tape.off + (e.clientX - r.left) / Tape.scale;
      Tape.scale = clamp(Tape.scale * (e.deltaY < 0 ? 1.15 : 1 / 1.15), 1.5, 46);
      Tape.off = Math.max(0, at - (e.clientX - r.left) / Tape.scale);
      Tape.draw(); Tape.ensure();
    }, { passive: false });
    new ResizeObserver(() => Tape.draw()).observe(c);
  },

  visible() {
    const c = $('tapeCanvas');
    return Math.ceil(c.clientWidth / Tape.scale) + 2;
  },

  async ensure() {
    const need = Math.ceil(Tape.off) + Tape.visible() + 64;
    if (need > S.seq.length && S.seq.length < 20000) {
      S.want = clamp(Math.max(need, S.want * 2), 64, 20000);
      await loadSeq();
    }
  },

  select(i) {
    if (i < 0) return;
    S.sel = i;
    Tape.draw();
    renderPosition(i);
    show('sequence');
  },

  draw() {
    const c = $('tapeCanvas');
    if (!c) return;
    const { ctx, w, h } = hidpi(c);
    ctx.clearRect(0, 0, w, h);
    const top = 14, tileH = 30, s = Tape.scale;
    const first = Math.floor(Tape.off), last = first + Math.ceil(w / s) + 1;

    // ruler
    ctx.font = '9px ' + cssv('--mono');
    ctx.fillStyle = PAL.ink3;
    ctx.textBaseline = 'top';
    const step = s >= 26 ? 5 : s >= 12 ? 10 : s >= 5 ? 50 : 200;
    for (let i = Math.ceil(first / step) * step; i <= last; i += step) {
      const x = (i - Tape.off) * s;
      if (x > w) break;
      ctx.fillText(String(i), x + 1, 1);
      ctx.fillStyle = PAL.ruleSoft;
      ctx.fillRect(x, 11, 1, 3);
      ctx.fillStyle = PAL.ink3;
    }

    // tiles
    ctx.textBaseline = 'middle';
    ctx.textAlign = 'center';
    for (let i = first; i <= last; i++) {
      if (i >= S.seq.length) break;
      const v = +S.seq[i];
      const x = (i - Tape.off) * s;
      const wTile = Math.max(1, s - (s > 6 ? 2 : 0.5));
      ctx.fillStyle = symbolColour(v);
      roundRect(ctx, x, top, wTile, tileH, s > 8 ? 3 : 1);
      ctx.fill();
      if (s >= 9) {                          // the punched-tape notch (seed draw 10)
        ctx.fillStyle = PAL.paper;
        ctx.fillRect(x + wTile / 2 - 1.5, top + tileH - 3, 3, 3);
      }
      if (s >= 13) {
        ctx.fillStyle = symbolInk(v);
        ctx.font = Math.min(15, s * 0.72) + 'px ' + cssv('--mono');
        ctx.fillText(String(v), x + wTile / 2, top + tileH / 2 - 1);
      }
      if (S.paint && S.paint.set.has(i)) {
        ctx.fillStyle = PAL.amber;
        ctx.fillRect(x, top + tileH + 3, wTile, 4);
      }
      if (S.sel === i) {
        ctx.strokeStyle = PAL.ink;
        ctx.lineWidth = 2;
        roundRect(ctx, x - 1, top - 2, wTile + 2, tileH + 4, s > 8 ? 4 : 2);
        ctx.stroke();
      }
    }

    // witness brackets
    ctx.strokeStyle = PAL.amber;
    ctx.fillStyle = PAL.amber;
    ctx.lineWidth = 1.5;
    ctx.textAlign = 'left';
    S.brackets.slice(0, 8).forEach((b, n) => {
      const x0 = (b.i - Tape.off) * s, x1 = (b.i + Math.max(b.len, 1) - Tape.off) * s;
      if (x1 < 0 || x0 > w) return;
      const y = top + tileH + 9 + n * 9;
      if (y > h - 2) return;
      ctx.beginPath();
      ctx.moveTo(x0, y + 4); ctx.lineTo(x0, y); ctx.lineTo(x1, y); ctx.lineTo(x1, y + 4);
      ctx.stroke();
      if (x1 - x0 > 40) {
        ctx.font = '9px ' + cssv('--mono');
        ctx.fillText(b.label, x0 + 4, y - 1);
      }
    });

    if (!S.seq.length) {
      ctx.fillStyle = PAL.ink3;
      ctx.textAlign = 'left';
      ctx.font = '12px ' + cssv('--mono');
      ctx.fillText('loading the sequence…', 4, top + 16);
    }
  },

  legend() {
    const box = $('tapeLegend');
    box.innerHTML = '';
    const alpha = [...new Set(S.seq.slice(0, 4000).split(''))].sort();
    alpha.forEach(a => {
      const sp = el('span');
      sp.innerHTML = `<b style="display:inline-block;width:9px;height:9px;border-radius:2px;
        background:${symbolColour(+a)};vertical-align:-1px"></b> ${a}`;
      box.appendChild(sp);
    });
    if (S.paint) {
      const sp = el('span');
      sp.innerHTML = `<b style="display:inline-block;width:9px;height:4px;background:${PAL.amber};
        vertical-align:2px"></b> ${S.paint.label} — ${S.paint.set.size} positions`;
      box.appendChild(sp);
    }
    S.brackets.slice(0, 3).forEach(b => {
      const sp = el('span', null, `⌐ ${b.label}`);
      box.appendChild(sp);
    });
  }
};

// -------------------------------------------------------------- position card

function digitsOf(n, k) {
  if (n === 0) return [0];
  const d = [];
  while (n > 0) { d.unshift(n % k); n = Math.floor(n / k); }
  return d;
}

function renderPosition(i) {
  const card = $('posCard');
  card.innerHTML = '';
  card.appendChild(el('span', 'eyebrow', 'position'));
  const head = el('div');
  head.style.cssText = 'font-family:var(--display);font-size:var(--t3);margin:2px 0 8px';
  head.textContent = `T[${i}] = ${S.seq[i] !== undefined ? S.seq[i] : '?'}`;
  card.appendChild(head);

  const grid = el('div', 'posgrid');
  const k = S.k;
  const msd = digitsOf(i, k);
  const digits = S.mode === 'lsd' ? msd.slice().reverse() : msd;

  const d1 = el('div');
  d1.appendChild(el('span', 'eyebrow', `base ${k}, ${S.mode}-first`));
  const dd = el('div', 'digits');
  digits.forEach(x => dd.appendChild(el('span', 'digit', String(x))));
  d1.appendChild(dd);
  grid.appendChild(d1);

  const d2 = el('div');
  d2.appendChild(el('span', 'eyebrow', 'path through the DFAO'));
  const pd = el('div', 'path');
  if (!S.dfao) {
    pd.appendChild(el('span', null, 'load the automaton to see the path'));
  } else {
    const A = S.mode === 'lsd' ? { trans: S.dfao.lsd.trans, out: S.dfao.lsd.out } : S.dfao;
    let q = 0;
    pd.appendChild(el('span', 'st', 'q0'));
    digits.forEach(x => {
      pd.appendChild(el('span', 'arr', `—${x}→`));
      q = A.trans[q][x];
      const chip = el('span', 'st', 'q' + q);
      pd.appendChild(chip);
    });
    pd.lastChild.classList.add('final');
    const outv = A.out[q];
    pd.appendChild(el('span', 'arr', '⟹'));
    pd.appendChild(el('span', 'st final', String(outv)));
  }
  d2.appendChild(pd);
  grid.appendChild(d2);
  card.appendChild(grid);

  // the factor starting here, and where else it occurs -- one engine query, painted
  // straight back onto the tape
  const f = el('div');
  f.style.marginTop = '12px';
  f.appendChild(el('span', 'eyebrow', 'factor starting here'));
  const frow = el('div', 'row');
  frow.style.marginTop = '6px';
  const lenIn = el('input');
  lenIn.type = 'number'; lenIn.min = '1'; lenIn.max = '64'; lenIn.value = String(S.factorLen || 6);
  lenIn.style.width = '68px';
  const showFactor = (bracket) => {
    const L = clamp(+lenIn.value || 1, 1, 64);
    S.factorLen = L;
    txt.textContent = S.seq.slice(i, i + L) || '(past the loaded tape)';
    if (bracket) {                       // never clobber a witness bracket on first render
      S.brackets = [{ i, len: L, label: `T[${i}..${i + L - 1}]` }];
      Tape.legend(); Tape.draw();
    }
  };
  lenIn.oninput = () => showFactor(true);
  const lab = el('label', 'field', 'length');
  lab.appendChild(lenIn);
  frow.appendChild(lab);
  const txt = el('span', 'digit');
  txt.style.fontSize = 'var(--t1)';
  frow.appendChild(txt);
  const find = el('button', 'btn', 'Find every occurrence');
  find.onclick = async () => {
    const L = clamp(+lenIn.value || 1, 1, 64);
    find.disabled = true;
    find.textContent = 'searching…';
    const B = clamp(S.seq.length, 64, 2000);
    const script = scriptPrefix() +
      `enum ${B} (A t. t < ${L} => T[${i}+t] = T[j+t])`;
    try {
      const j2 = await post('/api/run', { script, timeout: 120, mem_mb: +$('memMb').value || 1536 });
      const line = (j2.lines || []).find(l => l.kind === 'ENUM');
      if (line) {
        paintPositions(line.tuples.map(t => t[0]), `occurrences of T[${i}..${i + L - 1}]`);
        find.textContent = `${line.n} occurrences below ${B}`;
      } else {
        find.textContent = 'no answer — see the playground';
      }
    } catch (e) {
      find.textContent = 'failed: ' + String(e.message).slice(0, 40);
    }
    find.disabled = false;
  };
  frow.appendChild(find);
  f.appendChild(frow);
  card.appendChild(f);
  showFactor(false);

  if (S.paint && S.paint.set.has(i)) {
    const p = el('p', null, `This position is painted: ${S.paint.label}`);
    p.style.color = 'var(--amber)';
    card.appendChild(p);
  }
}

// ---------------------------------------------------------------- automaton

const Auto = {
  a: null, nodes: [], edges: [], view: { x: 40, y: 40, z: 1 }, drag: null, hover: -1,

  init() {
    const c = $('autoCanvas');
    c.addEventListener('pointerdown', (e) => {
      c.setPointerCapture(e.pointerId);
      Auto.drag = { x: e.clientX, y: e.clientY, vx: Auto.view.x, vy: Auto.view.y };
    });
    c.addEventListener('pointermove', (e) => {
      if (Auto.drag) {
        Auto.view.x = Auto.drag.vx + (e.clientX - Auto.drag.x);
        Auto.view.y = Auto.drag.vy + (e.clientY - Auto.drag.y);
        Auto.draw();
      } else Auto.hoverAt(e);
    });
    c.addEventListener('pointerup', () => { Auto.drag = null; });
    c.addEventListener('pointerleave', () => { $('autoTip').classList.remove('on'); });
    c.addEventListener('wheel', (e) => {
      e.preventDefault();
      const r = c.getBoundingClientRect();
      const mx = e.clientX - r.left, my = e.clientY - r.top;
      const f = e.deltaY < 0 ? 1.12 : 1 / 1.12;
      const z2 = clamp(Auto.view.z * f, 0.06, 6);
      Auto.view.x = mx - (mx - Auto.view.x) * (z2 / Auto.view.z);
      Auto.view.y = my - (my - Auto.view.y) * (z2 / Auto.view.z);
      Auto.view.z = z2;
      Auto.draw();
    }, { passive: false });
    // the canvas has zero size while its view is hidden, so a fit asked for then is
    // deferred until the view is actually on screen
    new ResizeObserver(() => { if (Auto.needFit) Auto.fit(); Auto.draw(); }).observe(c);
    $('autoReload').onclick = () => Auto.load($('autoPick').value);
    $('autoFit').onclick = () => { Auto.fit(); Auto.draw(); };
    $('autoRelax').onclick = () => { Auto.relax(140); Auto.draw(); };
  },

  resize() { if (Auto.needFit) Auto.fit(); Auto.draw(); },

  async load(name) {
    $('autoInfo').textContent = 'building…';
    const q = new URLSearchParams({ def: S.cur.def, name, mode: S.mode });
    const pre = S.preds[name];
    if (pre) q.set('pre', pre);
    let j;
    try { j = await api('/api/export?' + q.toString()); }
    catch (e) { $('autoInfo').textContent = String(e.message).slice(0, 90); return; }
    if (!j.automaton) {
      $('autoInfo').textContent = j.budget ? 'over the memory budget' :
        j.timed_out ? 'timed out' : 'no automaton';
      return;
    }
    Auto.a = j.automaton;
    if (Auto.a.kind === 'dfao') S.dfao = Auto.a;
    Auto.layout();
    Auto.fit();
    Auto.draw();
    const a = Auto.a;
    $('autoInfo').textContent = `${a.name}: ${fmt(a.nstates)} states` +
      (a.truncated ? `, showing ${fmt(a.shown)}` : '') +
      (a.kind === 'dfa' ? `, ${a.alpha} track tuples` : `, base ${a.k}`);
    if (S.sel !== null) renderPosition(S.sel);
  },

  layout() {
    const a = Auto.a;
    const n = a.shown;
    const k = a.kind === 'dfao' ? a.k : a.alpha;
    // BFS depth from the initial state gives the columns; everything else is relaxation
    const depth = new Array(n).fill(-1);
    depth[0] = 0;
    const queue = [0];
    for (let qi = 0; qi < queue.length; qi++) {
      const s = queue[qi];
      for (let x = 0; x < k; x++) {
        const t = a.trans[s][x];
        if (t >= 0 && depth[t] < 0) { depth[t] = depth[s] + 1; queue.push(t); }
      }
    }
    for (let s = 0; s < n; s++) if (depth[s] < 0) depth[s] = 0;
    const cols = {};
    for (let s = 0; s < n; s++) (cols[depth[s]] = cols[depth[s]] || []).push(s);
    const maxCol = Math.max(...Object.values(cols).map(c => c.length));
    const dx = clamp(1400 / (Object.keys(cols).length + 1), 90, 220);
    const dy = clamp(1200 / Math.max(maxCol, 1), 26, 74);
    Auto.nodes = new Array(n);
    Object.keys(cols).forEach(d => {
      cols[d].forEach((s, i) => {
        Auto.nodes[s] = { s, x: (+d) * dx, y: (i - (cols[d].length - 1) / 2) * dy, d: +d };
      });
    });
    // aggregate parallel transitions into one edge carrying every digit tuple
    const map = new Map();
    for (let s = 0; s < n; s++) {
      for (let x = 0; x < k; x++) {
        const t = a.trans[s][x];
        if (t < 0) continue;
        const key = s + '>' + t;
        if (!map.has(key)) map.set(key, { s, t, syms: [] });
        map.get(key).syms.push(x);
      }
    }
    Auto.edges = [...map.values()];
    Auto.relax(n > 600 ? 40 : 160);
  },

  // a tiny layered relaxation: x is fixed by BFS depth, y is pulled by edges and
  // pushed apart within a column.  Deterministic, ~20 lines, good enough for a few
  // hundred states — which is all a graph view can honestly show.
  relax(iters) {
    const N = Auto.nodes;
    if (!N.length) return;
    const GAP = 38;
    const byCol = {};
    N.forEach(nd => (byCol[nd.d] = byCol[nd.d] || []).push(nd));
    const cols = Object.values(byCol);
    for (let it = 0; it < iters; it++) {
      const f = new Float64Array(N.length);
      // edges pull their endpoints level; the pull is divided by degree so a hub with
      // 8 outgoing tuples does not drag the whole column onto one line
      const deg = new Float64Array(N.length);
      Auto.edges.forEach(e => { if (e.s !== e.t) { deg[e.s]++; deg[e.t]++; } });
      Auto.edges.forEach(e => {
        if (e.s === e.t) return;
        const d = N[e.t].y - N[e.s].y;
        f[e.s] += d * 0.5 / Math.max(1, deg[e.s]);
        f[e.t] -= d * 0.5 / Math.max(1, deg[e.t]);
      });
      for (let i = 0; i < N.length; i++) N[i].y += clamp(f[N[i].s], -12, 12);
      // then force the column apart again: exact spacing beats a spring that never wins
      cols.forEach(col => {
        col.sort((a, b) => a.y - b.y);
        for (let i = 1; i < col.length; i++) {
          if (col[i].y - col[i - 1].y < GAP) col[i].y = col[i - 1].y + GAP;
        }
        const mid = (col[0].y + col[col.length - 1].y) / 2;
        col.forEach(nd => { nd.y -= mid; });
      });
    }
  },

  fit() {
    const c = $('autoCanvas');
    if (!Auto.nodes.length) return;
    if (c.clientWidth < 40 || c.clientHeight < 40) { Auto.needFit = true; return; }
    Auto.needFit = false;
    const xs = Auto.nodes.map(n => n.x), ys = Auto.nodes.map(n => n.y);
    const x0 = Math.min(...xs), x1 = Math.max(...xs), y0 = Math.min(...ys), y1 = Math.max(...ys);
    const pad = 46;
    const z = clamp(Math.min((c.clientWidth - pad * 2) / Math.max(x1 - x0, 1),
      (c.clientHeight - pad * 2) / Math.max(y1 - y0, 1)), 0.06, 1.6);
    Auto.view.z = z;
    Auto.view.x = Math.max(pad, (c.clientWidth - (x1 - x0) * z) / 2) - x0 * z;
    Auto.view.y = c.clientHeight / 2 - ((y0 + y1) / 2) * z;
  },

  arrow(ctx, x, y, ang, R) {
    const a = clamp(R * 0.55, 3, 8);
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(x - a * Math.cos(ang - 0.45), y - a * Math.sin(ang - 0.45));
    ctx.lineTo(x - a * Math.cos(ang + 0.45), y - a * Math.sin(ang + 0.45));
    ctx.closePath();
    ctx.fillStyle = PAL.ink3;
    ctx.fill();
  },

  symLabel(e) {
    const a = Auto.a;
    if (a.kind === 'dfao') return e.syms.join(',');
    if (e.syms.length > 3) return e.syms.length + ' tuples';
    return e.syms.map(x => '(' + a.labels[x].join(',') + ')').join(' ');
  },

  draw() {
    const c = $('autoCanvas');
    if (!c) return;
    const { ctx, w, h } = hidpi(c);
    ctx.fillStyle = PAL.panel;
    ctx.fillRect(0, 0, w, h);
    if (!Auto.a) {
      ctx.fillStyle = PAL.ink3;
      ctx.font = '13px ' + cssv('--mono');
      ctx.fillText('Choose an automaton and press Load.', 16, 28);
      return;
    }
    const { x: ox, y: oy, z } = Auto.view;
    const P = (n) => [n.x * z + ox, n.y * z + oy];
    const R = clamp(15 * z, 2.5, 17);
    const labels = Auto.nodes.length <= 200 && R > 7;
    // an 8-tuple alphabet writes eight labels on every edge: past a handful of edges
    // that is noise, and the hover tooltip carries the same information on demand
    const edgeLabels = labels && Auto.edges.length <= 36;

    // edges.  Every pair bows to one side, chosen by the direction of travel, so a
    // two-way pair never draws on top of itself; the arrowhead says which way it goes.
    ctx.lineWidth = Math.max(0.5, 1.1 * z);
    ctx.font = Math.max(8, 9.5 * z) + 'px ' + cssv('--mono');
    ctx.textAlign = 'center';
    Auto.edges.forEach(e => {
      const A = Auto.nodes[e.s], B = Auto.nodes[e.t];
      if (!A || !B) return;
      const [ax, ay] = P(A), [bx, by] = P(B);
      if (Math.max(ax, bx) < -80 || Math.min(ax, bx) > w + 80) return;
      ctx.strokeStyle = PAL.rule;
      if (e.s === e.t) {
        ctx.beginPath();
        ctx.arc(ax, ay - R * 1.2, R * 0.8, Math.PI * 0.12, Math.PI * 0.88, true);
        ctx.stroke();
        Auto.arrow(ctx, ax + R * 0.55, ay - R * 1.55, Math.PI * 0.62, R);
        if (edgeLabels) {
          ctx.fillStyle = PAL.ink3;
          ctx.fillText(Auto.symLabel(e), ax, ay - R * 2.3);
        }
        return;
      }
      const dx = bx - ax, dy = by - ay;
      const len = Math.hypot(dx, dy) || 1;
      const side = e.s < e.t ? 1 : -1;
      const bow = side * Math.min(34 * z, len * 0.22);
      const mx = (ax + bx) / 2 - dy / len * bow, my = (ay + by) / 2 + dx / len * bow;
      // stop the curve at the node rim so the arrowhead is not swallowed
      const t = 1 - (R + 3) / len;
      const ex = (1 - t) * (1 - t) * ax + 2 * (1 - t) * t * mx + t * t * bx;
      const ey = (1 - t) * (1 - t) * ay + 2 * (1 - t) * t * my + t * t * by;
      ctx.beginPath();
      ctx.moveTo(ax, ay);
      ctx.quadraticCurveTo(mx, my, ex, ey);
      ctx.stroke();
      Auto.arrow(ctx, ex, ey, Math.atan2(ey - my, ex - mx), R);
      if (edgeLabels) {
        ctx.fillStyle = PAL.ink3;
        ctx.fillText(Auto.symLabel(e), (ax + bx) / 2 - dy / len * bow * 0.7 ,
                     (ay + by) / 2 + dx / len * bow * 0.7 - 3);
      }
    });

    // nodes
    const a = Auto.a;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    Auto.nodes.forEach(nd => {
      const [x, y] = P(nd);
      if (x < -40 || x > w + 40 || y < -40 || y > h + 40) return;
      const accepting = a.kind === 'dfa' ? a.accepting.includes(nd.s) : false;
      const out = a.kind === 'dfao' ? a.out[nd.s] : null;
      ctx.beginPath();
      ctx.arc(x, y, R, 0, Math.PI * 2);
      ctx.fillStyle = a.kind === 'dfao' ? symbolColour(out)
        : accepting ? (PAL.dark ? 'hsl(98 30% 24%)' : 'hsl(98 42% 88%)') : PAL.panel2;
      ctx.fill();
      ctx.lineWidth = nd.s === Auto.hover ? 2.4 : 1.2;
      ctx.strokeStyle = nd.s === 0 ? PAL.amber : accepting ? PAL.green : PAL.ink3;
      ctx.stroke();
      if (accepting && R > 5) {
        ctx.beginPath(); ctx.arc(x, y, R - 3, 0, Math.PI * 2);
        ctx.strokeStyle = PAL.green; ctx.lineWidth = 1;
        ctx.stroke();
      }
      if (labels) {
        ctx.fillStyle = a.kind === 'dfao' ? symbolInk(out) : PAL.ink;
        ctx.font = Math.max(9, 10 * z) + 'px ' + cssv('--mono');
        ctx.fillText(a.kind === 'dfao' ? `${nd.s}:${out}` : String(nd.s), x, y);
      }
    });

    // key
    ctx.textAlign = 'left';
    ctx.fillStyle = PAL.ink3;
    ctx.font = '10px ' + cssv('--mono');
    ctx.fillText(a.kind === 'dfao'
      ? 'node = state : output · amber ring = start'
      : 'green ring = accepting · amber ring = start', 10, h - 10);
  },

  hoverAt(e) {
    const c = $('autoCanvas'), r = c.getBoundingClientRect();
    const mx = e.clientX - r.left, my = e.clientY - r.top;
    const { x: ox, y: oy, z } = Auto.view;
    let best = -1, bd = 1e9;
    Auto.nodes.forEach(nd => {
      const dx = nd.x * z + ox - mx, dy = nd.y * z + oy - my;
      const d = dx * dx + dy * dy;
      if (d < bd) { bd = d; best = nd.s; }
    });
    const tip = $('autoTip');
    if (best < 0 || bd > 400) { tip.classList.remove('on'); Auto.hover = -1; Auto.draw(); return; }
    if (Auto.hover !== best) { Auto.hover = best; Auto.draw(); }
    const a = Auto.a;
    const outs = [];
    const k = a.kind === 'dfao' ? a.k : a.alpha;
    for (let x = 0; x < Math.min(k, 8); x++) {
      const t = a.trans[best][x];
      const lab = a.kind === 'dfao' ? String(x) : '(' + a.labels[x].join(',') + ')';
      outs.push(`${lab} → q${t < 0 ? '…' : t}`);
    }
    tip.innerHTML = `<b>q${best}</b>` +
      (a.kind === 'dfao' ? ` · output ${a.out[best]}` :
        a.accepting.includes(best) ? ' · accepting' : ' · rejecting') +
      `<br>${outs.join('<br>')}` + (k > 8 ? `<br>…and ${k - 8} more` : '');
    tip.style.left = clamp(mx + 14, 4, c.clientWidth - 240) + 'px';
    tip.style.top = clamp(my + 12, 4, c.clientHeight - 90) + 'px';
    tip.classList.add('on');
  }
};

// ------------------------------------------------------------------ heatmap

const Heat = {
  data: null, drag: null,

  init() {
    const c = $('feCanvas');
    $('feRun').onclick = () => Heat.load();
    c.addEventListener('pointerdown', (e) => {
      c.setPointerCapture(e.pointerId);
      Heat.drag = { x: e.clientX, y: e.clientY, i0: +$('feI0').value, j0: +$('feJ0').value, moved: 0 };
    });
    c.addEventListener('pointermove', (e) => {
      if (Heat.drag) {
        const cell = Heat.cell();
        const di = Math.round((Heat.drag.y - e.clientY) / cell);
        const dj = Math.round((Heat.drag.x - e.clientX) / cell);
        Heat.drag.moved = Math.max(Heat.drag.moved, Math.abs(e.clientX - Heat.drag.x), Math.abs(e.clientY - Heat.drag.y));
        $('feI0').value = Math.max(0, Heat.drag.i0 + di);
        $('feJ0').value = Math.max(0, Heat.drag.j0 + dj);
        if (Heat.drag.moved > 6) Heat.queue();
      } else Heat.hoverAt(e);
    });
    c.addEventListener('pointerup', (e) => {
      if (Heat.drag && Heat.drag.moved < 5) Heat.click(e);
      Heat.drag = null;
    });
    c.addEventListener('pointerleave', () => $('feTip').classList.remove('on'));
    c.addEventListener('wheel', (e) => {
      e.preventDefault();
      const size = +$('feSize').value;
      $('feSize').value = clamp(Math.round(e.deltaY > 0 ? size * 1.3 : size / 1.3), 8, 512);
      Heat.queue();
    }, { passive: false });
    new ResizeObserver(() => Heat.draw()).observe(c);
  },

  resize() { Heat.draw(); },
  cell() {
    const c = $('feCanvas');
    const n = Heat.data ? Heat.data.size : +$('feSize').value;
    return Math.min(c.clientWidth, c.clientHeight) / n;
  },

  queue() {
    clearTimeout(Heat._t);
    Heat._t = setTimeout(() => Heat.load(), 180);
  },

  async load() {
    const q = new URLSearchParams({
      def: S.cur.def, mode: S.mode,
      i0: $('feI0').value, j0: $('feJ0').value, size: $('feSize').value, l: $('feL').value
    });
    $('feInfo').textContent = 'walking…';
    try {
      Heat.data = await api('/api/femap?' + q.toString());
    } catch (e) { $('feInfo').textContent = String(e.message).slice(0, 80); return; }
    const d = Heat.data;
    let ones = 0;
    d.rows.forEach(r => { for (const ch of r) if (ch === '1') ones++; });
    $('feInfo').textContent = `${d.size}×${d.size} at L=${d.l} · ${ones} agreeing pairs · ${d.ms} ms`;
    Heat.draw();
  },

  draw() {
    const c = $('feCanvas');
    if (!c) return;
    const { ctx, w, h } = hidpi(c);
    ctx.fillStyle = PAL.panel;
    ctx.fillRect(0, 0, w, h);
    const d = Heat.data;
    if (!d) {
      ctx.fillStyle = PAL.ink3;
      ctx.font = '13px ' + cssv('--mono');
      ctx.fillText('Press Draw to walk the grid.', 16, 28);
      return;
    }
    const cs = Math.min(w, h) / d.size;
    const ox = Heat.originX = Math.max(0, (w - d.size * cs) / 2);
    ctx.save();
    ctx.translate(ox, 0);
    ctx.fillStyle = PAL.indigo;
    for (let r = 0; r < d.size; r++) {
      const row = d.rows[r] || '';
      for (let cc = 0; cc < d.size; cc++) {
        if (row[cc] === '1') ctx.fillRect(cc * cs, r * cs, Math.max(cs, 1), Math.max(cs, 1));
      }
    }
    // the diagonal i = j is trivially true; mark it so it never reads as signal
    if (d.i0 === d.j0) {
      ctx.strokeStyle = PAL.amber;
      ctx.lineWidth = 1;
      ctx.beginPath(); ctx.moveTo(0, 0); ctx.lineTo(d.size * cs, d.size * cs); ctx.stroke();
    }
    ctx.strokeStyle = PAL.rule;
    ctx.strokeRect(0.5, 0.5, d.size * cs - 1, d.size * cs - 1);
    ctx.restore();
    ctx.fillStyle = PAL.ink3;
    ctx.font = '10px ' + cssv('--mono');
    ctx.fillText(`i ${d.i0} … ${d.i0 + d.size - 1}  (down)`, 8, 14);
    ctx.fillText(`j ${d.j0} … ${d.j0 + d.size - 1}  (across)`, 8, 28);
  },

  at(e) {
    const c = $('feCanvas'), r = c.getBoundingClientRect(), d = Heat.data;
    if (!d) return null;
    const cs = Math.min(c.clientWidth, c.clientHeight) / d.size;
    const row = Math.floor((e.clientY - r.top) / cs);
    const col = Math.floor((e.clientX - r.left - (Heat.originX || 0)) / cs);
    if (row < 0 || col < 0 || row >= d.size || col >= d.size) return null;
    return { i: d.i0 + row, j: d.j0 + col, v: (d.rows[row] || '')[col] === '1' };
  },

  hoverAt(e) {
    const p = Heat.at(e), tip = $('feTip'), c = $('feCanvas');
    if (!p) { tip.classList.remove('on'); return; }
    const L = Heat.data.l;
    const fi = S.seq.slice(p.i, p.i + L), fj = S.seq.slice(p.j, p.j + L);
    tip.innerHTML = `FE(${p.i}, ${p.j}, ${L}) = <b>${p.v ? 'true' : 'false'}</b>` +
      (fi && fj ? `<br>${fi || '…'}<br>${fj || '…'}` : '');
    const r = c.getBoundingClientRect();
    tip.style.left = clamp(e.clientX - r.left + 14, 4, c.clientWidth - 200) + 'px';
    tip.style.top = clamp(e.clientY - r.top + 12, 4, c.clientHeight - 70) + 'px';
    tip.classList.add('on');
  },

  click(e) {
    const p = Heat.at(e);
    if (!p) return;
    const L = Heat.data.l;
    S.brackets = [{ i: p.i, len: L, label: `i=${p.i}` }, { i: p.j, len: L, label: `j=${p.j}` }];
    Tape.select(p.i);
    Tape.legend();
  }
};

// --------------------------------------------------------------- playground

function scriptPrefix() {
  return `mode ${S.mode}\n${S.cur.def}\n`;
}

function resCard(kind, title, formula, stats) {
  const d = el('div', 'res ' + kind);
  const v = el('div', 'verdict', title);
  d.appendChild(v);
  if (formula) d.appendChild(el('div', 'formula', formula));
  if (stats) d.appendChild(el('div', 'stats', stats));
  return d;
}

function statsOf(l) {
  const bits = [];
  if (l.states !== undefined) bits.push(`${fmt(l.states)} states`);
  if (l.ms !== undefined) bits.push(`${fmt(l.ms)} ms`);
  if (l.peak !== undefined) bits.push(`peak ${fmt(l.peak)} intermediate states`);
  if (l.mqs !== undefined) bits.push(`${fmt(l.mqs)} membership queries`);
  if (l.eqs !== undefined) bits.push(`${l.eqs} equivalence queries`);
  return bits.join(' · ');
}

function renderResult(payload) {
  const box = $('results');
  let last = null;
  (payload.lines || []).forEach(l => {
    if (l.kind === 'cont') {
      if (last) {
        const pre = last.querySelector('.stats') || last.appendChild(el('div', 'stats'));
        pre.textContent += (pre.textContent ? '\n' : '') + l.raw;
      }
      return;
    }
    if (l.kind === 'OK' && l.what === 'def') return;
    let card = null;
    if (l.kind === 'TRUE' || l.kind === 'FALSE') {
      card = resCard(l.verdict ? 'true' : 'false', l.verdict ? 'TRUE' : 'FALSE', l.formula, statsOf(l));
      setMascot(l.verdict ? 'happy' : 'oops', true);
    } else if (l.kind === 'ERR') {
      card = resCard('err', 'Error', l.error, '');
      setMascot('oops', true);
    } else if (l.kind === 'WITNESS') {
      card = resCard('info', 'Witness', l.formula, statsOf(l));
      const asg = el('div', 'formula',
        Object.entries(l.assign).map(([k, v]) => `${k} = ${v}`).join('   '));
      card.insertBefore(asg, card.children[1] || null);
      applyWitness(l.assign);
    } else if (l.kind === 'NONE') {
      card = resCard('false', 'No witness', l.formula, statsOf(l));
    } else if (l.kind === 'ENUM') {
      card = resCard('info', `${fmt(l.n)} solutions`, 'vars ' + (l.vars || []).join(','), '');
      if ((l.vars || []).length === 1) {
        paintPositions(l.tuples.map(t => t[0]), 'satisfies the last enum');
        card.appendChild(el('div', 'stats', 'painted onto the tape'));
      } else if (l.tuples && l.tuples.length) {
        card.appendChild(el('div', 'stats', l.tuples.slice(0, 40).map(t => '(' + t.join(',') + ')').join(' ')));
      }
    } else if (l.kind === 'OPEN') {
      card = resCard('info', 'Open formula', l.formula, statsOf(l));
      card.appendChild(el('div', 'stats', l.raw.split('witnesses=')[1] || ''));
    } else if (l.kind === 'FINITE' || l.kind === 'INFINITE' || l.kind === 'EMPTY') {
      card = resCard('info', l.kind[0] + l.kind.slice(1).toLowerCase(), l.formula, statsOf(l));
    } else if (l.kind === 'SEQ' || l.kind === 'EXPORT' || l.kind === 'FEMAP') {
      if (l.kind === 'EXPORT') {
        Auto.a = l.automaton;
        if (l.automaton.kind === 'dfao') S.dfao = l.automaton;
        Auto.layout(); Auto.fit(); Auto.draw();
        card = resCard('info', 'Automaton exported', `${l.automaton.name}: ${fmt(l.automaton.nstates)} states`,
          'open the Automaton view');
      }
    } else if (l.kind === 'OK') {
      const nm = l.name;
      if (nm) {
        S.preds[nm] = $('editor').value;
        refreshPredList();
      }
      card = resCard('info', l.what === 'learnfe' ? `learnfe ${nm}` : l.what === 'let' ? `let ${nm}` : l.what,
        l.raw.replace(/^OK\s+/, ''), statsOf(l));
    } else if (l.kind === 'DFA') {
      card = resCard('info', 'DFA', l.raw.slice(4), '');
    } else {
      card = resCard('info', l.kind, l.raw, '');
    }
    if (card) { box.prepend(card); last = card; }
  });

  if (payload.budget) box.prepend(resCard('err', 'Memory budget exceeded',
    'The engine stopped itself at AM_MEM_MB. Raise the budget or use learnfe.', ''));
  if (payload.timed_out) box.prepend(resCard('err', 'Timed out',
    'The job hit the timeout and was killed.', ''));
  if (payload.stderr_tail && payload.stderr_tail.trim())
    box.prepend(resCard('err', 'Engine said', payload.stderr_tail.trim().slice(0, 500), ''));
  Tape.legend();
  Tape.draw();
}

function paintPositions(list, label) {
  S.paint = { set: new Set(list), label };
  Tape.legend();
  Tape.draw();
}

function applyWitness(assign) {
  const starts = ['i', 'j', 'p', 'q', 'm1', 'm2'].filter(v => assign[v] !== undefined);
  const len = ['n', 'l', 'L', 'b'].map(v => assign[v]).find(v => v !== undefined);
  if (starts.length) Tape.select(assign[starts[0]]);   // renders the position card first…
  S.brackets = starts.map(v => ({ i: assign[v], len: len || 1, label: `${v}=${assign[v]}` }));
  Tape.legend(); Tape.draw();                          // …then the witness owns the tape
}

function refreshPredList() {
  const sel = $('autoPick');
  const keep = sel.value;
  sel.innerHTML = '<option value="T">T — the sequence</option>';
  Object.keys(S.preds).forEach(n => {
    const o = el('option', null, `${n} — built predicate`);
    o.value = n;
    sel.appendChild(o);
  });
  sel.value = [...sel.options].some(o => o.value === keep) ? keep : 'T';
}

async function runScript(stream) {
  const script = scriptPrefix() + $('editor').value.trim();
  const body = {
    script,
    timeout: +$('timeoutS').value || 120,
    mem_mb: +$('memMb').value || 1536
  };
  setMascot('thinking');
  if (!stream) {
    $('runBtn').disabled = true;
    try {
      const j = await post('/api/run', body);
      renderResult(j);
    } catch (e) {
      $('results').prepend(resCard('err', 'Request failed', String(e.message), ''));
    }
    $('runBtn').disabled = false;
    return;
  }
  Live.start(body);
}

function buildLibrary() {
  const box = $('library');
  box.innerHTML = '';
  const groups = {};
  S.examples.forEach(e => (groups[e.group] = groups[e.group] || []).push(e));
  Object.entries(groups).forEach(([g, items]) => {
    const wrap = el('div', 'libgroup');
    wrap.appendChild(el('span', 'eyebrow', g));
    items.forEach(it => {
      const b = el('button', 'libitem');
      b.appendChild(document.createTextNode(it.name));
      if (it.note) b.appendChild(el('small', null, it.note));
      b.onclick = () => { $('editor').value = it.script; $('editor').focus(); };
      wrap.appendChild(b);
    });
    box.appendChild(wrap);
  });
}

// ------------------------------------------------------------------- live

const PHASES = ['compile', 'forward', 'brzozowski', 'minimize', 'verify'];

const Live = {
  job: null, es: null, t0: 0, timer: null,
  counts: { subsets: 0, states: 0, mqs: 0, eqs: 0, mb: 0, peak: 0 },

  init() {
    const bar = $('phasebar'), lab = $('phaselabels');
    PHASES.forEach(p => {
      const d = el('div'); d.dataset.p = p; bar.appendChild(d);
      lab.appendChild(el('span', null, p));
    });
    $('counters').innerHTML = '';
    [['subsets', 'subsets'], ['states', 'states'], ['elapsed', 'elapsed'],
     ['peak', 'peak MB'], ['queries', 'oracle queries']].forEach(([k, label]) => {
      const c = el('div', 'counter');
      c.appendChild(el('div', 'k', label));
      const v = el('div', 'v', '—'); v.id = 'cnt-' + k;
      c.appendChild(v);
      $('counters').appendChild(c);
    });
    $('cancelBtn').onclick = () => Live.cancel();
  },

  reset(budget) {
    Live.counts = { subsets: 0, states: 0, mqs: 0, eqs: 0, mb: 0, peak: 0 };
    Live.budget = budget;
    $('liveLog').textContent = '';
    document.querySelectorAll('#phasebar div').forEach(d => d.className = '');
    ['subsets', 'states', 'elapsed', 'peak', 'queries'].forEach(k => $('cnt-' + k).textContent = '—');
    $('memBar').style.width = '0%';
    $('memText').textContent = `0 / ${budget} MB`;
  },

  async start(body) {
    if (Live.job) Live.cancel();
    Live.reset(body.mem_mb || 1536);
    show('live');
    setMascot('thinking');
    $('liveState').textContent = 'starting';
    $('cancelBtn').disabled = false;
    let j;
    try { j = await post('/api/job', body); }
    catch (e) { $('liveState').textContent = 'could not start: ' + e.message; return; }
    Live.job = j.job;
    Live.t0 = Date.now();
    Live.timer = setInterval(() => {
      $('cnt-elapsed').textContent = ((Date.now() - Live.t0) / 1000).toFixed(1) + 's';
    }, 100);
    const es = new EventSource('/api/stream/' + j.job);
    Live.es = es;
    es.onmessage = (m) => Live.event(JSON.parse(m.data));
    es.addEventListener('end', () => Live.finish());
    es.onerror = () => { /* the server closes the stream itself; end fires first */ };
  },

  log(t) {
    const box = $('liveLog');
    box.textContent += t + '\n';
    box.scrollTop = box.scrollHeight;
  },

  event(ev) {
    if (ev.ev === 'phase') {
      const bar = [...document.querySelectorAll('#phasebar div')];
      const idx = PHASES.indexOf(ev.name === 'learn' ? 'compile' : ev.name);
      bar.forEach((d, i) => {
        if (idx < 0) return;
        d.className = i < idx ? 'done' : i === idx ? 'on' : '';
      });
      $('liveState').textContent = ev.name + (ev.detail ? ' · ' + ev.detail.slice(0, 60) : '');
      Live.log(`${ev.ms} ms  phase ${ev.name} ${ev.detail || ''}`);
    } else if (ev.ev === 'subsets') {
      Live.counts.subsets = ev.n;
      $('cnt-subsets').textContent = fmt(ev.n);
      Live.mem(ev.mb);
    } else if (ev.ev === 'states') {
      Live.counts.states = ev.n;
      $('cnt-states').textContent = fmt(ev.n);
      Live.log(`${ev.ms} ms  ${fmt(ev.n)} states (${ev.what})`);
    } else if (ev.ev === 'learn') {
      $('cnt-states').textContent = fmt(ev.states);
      $('cnt-queries').textContent = fmt(ev.mqs);
      Live.log(`${ev.ms} ms  learn: eq #${ev.eqs}, ${fmt(ev.states)} states, ${fmt(ev.mqs)} queries`);
    } else if (ev.ev === 'mem') {
      Live.mem(ev.mb, ev.peak_mb);
    } else if (ev.ev === 'done') {
      Live.mem(ev.mb, ev.peak_mb);
      Live.log(`${ev.ms} ms  ${ev.cmd} done`);
    } else if (ev.ev === 'line') {
      Live.log('› ' + ev.line);
    } else if (ev.ev === 'result') {
      Live.result = ev.result;
    }
  },

  mem(mb, peak) {
    if (mb !== undefined) {
      Live.counts.mb = mb;
      const pct = clamp(100 * mb / Live.budget, 0, 100);
      $('memBar').style.width = pct + '%';
      $('memBar').classList.toggle('hot', pct > 75);
      $('memText').textContent = `${mb} / ${Live.budget} MB`;
    }
    if (peak !== undefined) {
      Live.counts.peak = Math.max(Live.counts.peak, peak);
      $('cnt-peak').textContent = fmt(Live.counts.peak);
    }
  },

  finish() {
    if (Live.es) { Live.es.close(); Live.es = null; }
    clearInterval(Live.timer);
    $('cnt-elapsed').textContent = ((Date.now() - Live.t0) / 1000).toFixed(1) + 's';
    Live.job = null;
    $('cancelBtn').disabled = true;
    document.querySelectorAll('#phasebar div').forEach(d => d.className = d.className ? 'done' : '');
    const r = Live.result;
    if (r) {
      renderResult(r);
      const verdict = (r.lines || []).filter(l => l.kind === 'TRUE' || l.kind === 'FALSE').pop();
      $('liveState').textContent = r.timed_out ? 'timed out'
        : r.budget ? 'stopped at the memory budget'
        : verdict ? (verdict.verdict ? 'TRUE' : 'FALSE') : 'finished';
      setMascot(r.ok ? (verdict ? (verdict.verdict ? 'happy' : 'oops') : 'happy') : 'oops', true);
    } else {
      $('liveState').textContent = 'stopped';
    }
    Live.result = null;
  },

  async cancel() {
    if (!Live.job) return;
    try { await post('/api/cancel/' + Live.job, {}); } catch (e) { /* already gone */ }
    $('liveState').textContent = 'stopping…';
  }
};

// ---------------------------------------------------------------- morphism

const Morph = {
  k: 2, m: 2, words: ['01', '10'], coding: '01', touched: false,

  init() {
    $('mK').oninput = () => { Morph.touched = true; Morph.resize(); };
    $('mM').oninput = () => { Morph.touched = true; Morph.resize(); };
    $('mRoll').onclick = () => { Morph.touched = true; Morph.roll(); };
    $('mFromCurrent').onclick = () => { Morph.touched = true; Morph.fromCurrent(); };
    $('mCoding').oninput = () => { Morph.touched = true; Morph.sync(); };
    $('mUse').onclick = () => Morph.use();
    $('mBattery').onclick = () => Morph.battery();
    Morph.resize();
  },

  resize() {
    Morph.k = clamp(+$('mK').value || 2, 2, 6);
    Morph.m = clamp(+$('mM').value || 2, 1, 9);
    while (Morph.words.length < Morph.m) Morph.words.push('0'.repeat(Morph.k));
    Morph.words = Morph.words.slice(0, Morph.m).map(w =>
      (w + '0'.repeat(Morph.k)).slice(0, Morph.k));
    Morph.coding = ($('mCoding').value + '0'.repeat(Morph.m)).slice(0, Morph.m);
    $('mCoding').value = Morph.coding;
    Morph.rules();
    Morph.sync();
  },

  rules() {
    const box = $('mRules');
    box.innerHTML = '';
    for (let a = 0; a < Morph.m; a++) {
      const row = el('div', 'rule-row');
      row.appendChild(el('span', 'from', `${a} ↦`));
      const inp = el('input');
      inp.type = 'text';
      inp.value = Morph.words[a];
      inp.maxLength = Morph.k;
      inp.oninput = () => {
        Morph.touched = true;
        Morph.words[a] = inp.value.replace(/[^0-9]/g, '').slice(0, Morph.k);
        Morph.sync();
      };
      row.appendChild(inp);
      const out = el('span', 'from');
      out.textContent = '→ ' + (Morph.coding[a] || '0');
      row.appendChild(out);
      box.appendChild(row);
    }
  },

  def() {
    return `def T ${Morph.k} ${Morph.m} 0 ${Morph.words.join(' ')} ${Morph.coding}`;
  },

  // the admissibility test explore/blowup.py uses: prolongable at 0, every letter
  // reachable, the coding uses at least two letters, and the coded DFAO is reduced
  check() {
    const { k, m, words, coding } = Morph;
    if (words.some(w => w.length !== k)) return 'each rule needs exactly k letters';
    if (words.some(w => [...w].some(c => +c >= m))) return 'a rule uses a letter that is not a state';
    if (coding.length !== m) return 'the coding needs one letter per state';
    if (words[0][0] !== '0') return 'not prolongable: rule 0 must start with 0';
    const seen = new Set([0]), st = [0];
    while (st.length) { for (const c of words[st.pop()]) if (!seen.has(+c)) { seen.add(+c); st.push(+c); } }
    if (seen.size !== m) return `only ${seen.size} of ${m} states are reachable`;
    if (new Set(coding).size < 2) return 'the coding is constant — T would be constant';
    let col = [...coding].map(Number);
    for (;;) {
      const sig = new Map(); const next = [];
      for (let a = 0; a < m; a++) {
        const key = [col[a], ...[...words[a]].map(d => col[+d])].join(',');
        if (!sig.has(key)) sig.set(key, sig.size);
        next.push(sig.get(key));
      }
      if (sig.size === new Set(col).size) { col = next; break; }
      col = next;
    }
    if (new Set(col).size !== m) return `the DFAO collapses to ${new Set(col).size} states — reduce m`;
    return null;
  },

  sync() {
    Morph.coding = $('mCoding').value.replace(/[^0-9]/g, '');
    $('mDef').textContent = Morph.def();
    const bad = Morph.check();
    const st = $('mStatus');
    st.innerHTML = '';
    st.appendChild(el('span', bad ? 'warn' : 'ok-note',
      bad ? bad : 'admissible: prolongable, reachable, reduced'));
    $('mUse').disabled = !!bad;
    $('mBattery').disabled = !!bad;
    [...$('mRules').querySelectorAll('.rule-row')].forEach((row, a) => {
      row.lastChild.textContent = '→ ' + (Morph.coding[a] || '0');
    });
    if (!bad) Morph.preview();
  },

  // Regrow the tape from the draft morphism, but only once the user has actually
  // touched the sandbox: booting the app must not hijack the chosen sequence.
  preview() {
    if (!Morph.touched) return;
    clearTimeout(Morph._t);
    Morph._t = setTimeout(async () => {
      const q = new URLSearchParams({ def: Morph.def(), n: 512, mode: S.mode });
      try {
        const j = await api('/api/seq?' + q.toString());
        S.cur = { id: 'sandbox', name: 'sandbox morphism', def: Morph.def(), note: 'from the sandbox' };
        S.seq = j.seq; S.k = j.k;
        S.paint = null; S.brackets = []; S.dfao = null;
        $('tapeName').textContent = 'sandbox morphism';
        $('tapeDef').textContent = Morph.def();
        Tape.legend(); Tape.draw();
      } catch (e) { /* an inadmissible draft is normal while typing */ }
    }, 260);
  },

  roll() {
    const k = Morph.k, m = Morph.m;
    const rnd = (n) => {
      const b = new Uint32Array(1);
      crypto.getRandomValues(b);
      return b[0] % n;
    };
    for (let tries = 0; tries < 4000; tries++) {
      Morph.words = [];
      for (let a = 0; a < m; a++) {
        let w = '';
        for (let d = 0; d < k; d++) w += (a === 0 && d === 0) ? '0' : String(rnd(m));
        Morph.words.push(w);
      }
      Morph.coding = Array.from({ length: m }, () => String(rnd(2))).join('');
      $('mCoding').value = Morph.coding;
      if (!Morph.check()) break;
    }
    Morph.rules();
    Morph.sync();
  },

  fromCurrent() {
    const p = S.cur.def.split(/\s+/);   // def T k m start w... coding
    const k = +p[2], m = +p[3];
    $('mK').value = k; $('mM').value = m;
    Morph.k = k; Morph.m = m;
    Morph.words = p.slice(5, 5 + m);
    Morph.coding = p[5 + m];
    $('mCoding').value = Morph.coding;
    Morph.rules();
    Morph.sync();
  },

  use() {
    S.cur = { id: 'sandbox', name: 'sandbox morphism', def: Morph.def(), note: 'from the sandbox' };
    $('seqSelect').value = '';
    selectSequence(S.cur);
  },

  async battery() {
    const script = [
      '? ~ E i,n. n>=1 & (A t. t < n => T[i+t] = T[i+n+t])',
      '? ~ E i,n. n>=1 & (A t. t < 2*n => T[i+t] = T[i+n+t])',
      '? ~ E i,n. n>=1 & (A t. t <= n => T[i+t] = T[i+n+t])',
      '? E i,n. n>=3 & (A t,u. t+u+1 = n => T[i+t] = T[i+u])',
      '? A n. E i,m. m >= n & (A t,u. t+u+1 = m => T[i+t] = T[i+u])',
      '? A n. n>=1 => E i. ~(E b,j. b >= 1 & b < n & j + b = i + n & (A t. t < b => T[i+t] = T[j+t]))',
      '? A i,n,N. E j. j >= N & (A t. t < n => T[i+t] = T[j+t])',
      'let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]',
      'mem'
    ];
    const labels = ['square-free', 'cube-free', 'overlap-free', 'has a palindrome ≥ 3',
      'arbitrarily long palindromes', 'unbordered factor of every length',
      'every factor recurs', '|FE|', 'memory'];
    const out = $('battOut');
    out.textContent = 'running…';
    let j;
    try {
      j = await post('/api/run', { script: `mode ${S.mode}\n${Morph.def()}\n` + script.join('\n'),
        timeout: 300, mem_mb: +$('memMb').value || 1536 });
    } catch (e) { out.textContent = 'failed: ' + e.message; return; }
    const rows = (j.lines || []).filter(l => l.kind !== 'cont' && !(l.kind === 'OK' && l.what === 'def'));
    const t = el('table', 'batt');
    t.innerHTML = '<tr><th>question</th><th>answer</th><th>cost</th></tr>';
    rows.forEach((l, n) => {
      const tr = el('tr');
      tr.appendChild(el('td', null, labels[n] || l.kind));
      const v = el('td', l.kind === 'TRUE' ? 'v-true' : l.kind === 'FALSE' ? 'v-false'
        : l.kind === 'ERR' ? 'v-err' : '');
      v.textContent = l.kind === 'OK' && l.what === 'let' ? `${fmt(l.states)} states`
        : l.kind === 'OK' && l.what === 'mem' ? `peak ${l.peak} MB`
        : l.kind === 'ERR' ? l.error.slice(0, 40) : l.kind;
      tr.appendChild(v);
      tr.appendChild(el('td', null, l.ms !== undefined ? l.ms + ' ms' : ''));
      t.appendChild(tr);
    });
    out.innerHTML = '';
    out.appendChild(t);
    if (j.budget) out.appendChild(el('div', 'warn', 'stopped at the memory budget'));
  }
};

// ------------------------------------------------------------------- boot

async function loadSeq() {
  const q = new URLSearchParams({ def: S.cur.def, n: S.want, mode: S.mode });
  const j = await api('/api/seq?' + q.toString());
  S.seq = j.seq; S.k = j.k;
  Tape.legend();
  Tape.draw();
}

async function selectSequence(entry) {
  S.cur = entry;
  S.paint = null; S.brackets = []; S.sel = null; S.dfao = null; S.preds = {};
  refreshPredList();
  $('tapeName').textContent = entry.name;
  $('tapeDef').textContent = entry.def;
  Tape.off = 0;
  await loadSeq();
  Auto.a = null;
  Auto.load('T');
  Heat.data = null;
  Heat.draw();
}

async function boot() {
  buildRail();
  Tape.init(); Auto.init(); Heat.init(); Live.init(); Morph.init();
  await loadSprite();

  // the script prefix the server adds for you — shown so the editor is never a mystery
  const pre = el('div', 'eyebrow');
  pre.id = 'scriptPrefix';
  pre.style.marginBottom = '6px';
  $('editor').parentNode.insertBefore(pre, $('editor'));

  $('runBtn').onclick = () => runScript(false);
  $('runStreamBtn').onclick = () => runScript(true);
  $('clearBtn').onclick = () => { $('results').innerHTML = ''; };
  $('seqReload').onclick = () => { S.want = clamp(+$('seqLen').value || 512, 16, 20000); loadSeq(); };
  $('tapeClear').onclick = () => { S.paint = null; S.brackets = []; Tape.legend(); Tape.draw(); };
  $('modeSelect').onchange = () => { S.mode = $('modeSelect').value; selectSequence(S.cur); updatePrefix(); };
  $('editor').addEventListener('keydown', (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') { e.preventDefault(); runScript(false); }
  });

  try {
    const h = await api('/api/health');
    $('healthPill').textContent = `${h.free_mb.toLocaleString()} MB free`;
    $('footEngine').textContent = h.engine;
  } catch (e) { $('healthPill').textContent = 'engine unreachable'; }

  const lib = await api('/api/library');
  S.seqs = lib.sequences; S.examples = lib.examples;
  const sel = $('seqSelect');
  const groups = {};
  S.seqs.forEach(s => (groups[s.group] = groups[s.group] || []).push(s));
  Object.entries(groups).forEach(([g, items]) => {
    const og = el('optgroup');
    og.label = g;
    items.forEach(s => {
      const o = el('option', null, s.name);
      o.value = s.id;
      og.appendChild(o);
    });
    sel.appendChild(og);
  });
  sel.onchange = () => {
    const e = S.seqs.find(s => s.id === sel.value);
    if (e) { selectSequence(e); updatePrefix(); }
  };
  buildLibrary();
  await selectSequence(S.seqs[0]);
  updatePrefix();
  show((location.hash || '#sequence').slice(1));
}

function updatePrefix() {
  const p = $('scriptPrefix');
  if (!p) return;
  p.innerHTML = '';
  p.appendChild(el('span', null, 'the session starts with'));
  const code = el('span', null, '  ' + scriptPrefix().trim().replace('\n', '   '));
  code.style.cssText = 'text-transform:none;letter-spacing:0;color:var(--ink-2)';
  p.appendChild(code);
}

// Handles for the browser console and for /static/selftest.js — the end-to-end check
// in gui/README.md drives the real page through these rather than a second copy of it.
window.PEANUT = { S, Tape, Auto, Heat, Live, Morph, runScript, api, post, show };

boot().then(() => {
  if (location.search.includes('selftest')) {
    const t = document.createElement('script');
    t.src = '/static/selftest.js';
    document.body.appendChild(t);
  }
});
