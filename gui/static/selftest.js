/* End-to-end check, loaded only for /?selftest=1.
   Drives the real page and writes a plain-text report into #selftest-out, so a headless
   Chrome --dump-dom is enough to see whether the GUI actually works:

     chrome --headless=new --virtual-time-budget=45000 \
            --dump-dom 'http://localhost:7373/?selftest=1'
*/
(async () => {
  const out = document.createElement('pre');
  out.id = 'selftest-out';
  // hidden by default (the DOM dump is enough); ?show=1 puts it on screen so a
  // screenshot at a phone width can be read directly
  out.style.cssText = new URLSearchParams(location.search).has('show')
    ? 'position:fixed;inset:auto 0 0 0;z-index:99;background:#000;color:#0f0;font:11px Menlo;padding:8px;margin:0;white-space:pre-wrap'
    : 'position:fixed;left:-9999px';
  document.body.appendChild(out);
  const P = window.PEANUT;
  const log = (ok, name, extra) => {
    out.textContent += `${ok ? 'PASS' : 'FAIL'} ${name}${extra ? ' :: ' + extra : ''}\n`;
  };
  const wait = (fn, ms = 20000) => new Promise((res, rej) => {
    const t0 = Date.now();
    const tick = () => {
      let v;
      try { v = fn(); } catch (e) { v = false; }
      if (v) return res(v);
      if (Date.now() - t0 > ms) return rej(new Error('timeout'));
      setTimeout(tick, 60);
    };
    tick();
  });
  const step = async (name, fn) => {
    try { log(true, name, await fn()); } catch (e) { log(false, name, e.message); }
  };

  await step('library loaded', async () =>
    `${P.S.seqs.length} sequences, ${P.S.examples.length} examples`);
  await step('thue-morse tape', async () => {
    if (!P.S.seq.startsWith('0110100110010110')) throw new Error('prefix ' + P.S.seq.slice(0, 16));
    return P.S.seq.slice(0, 16);
  });
  await step('position card', async () => {
    P.Tape.select(12);
    const txt = document.getElementById('posCard').textContent;
    if (!txt.includes('T[12]')) throw new Error(txt.slice(0, 60));
    return 'T[12] = ' + P.S.seq[12];
  });
  await step('dfao exported', async () => {
    await P.Auto.load('T');
    if (!P.Auto.a || P.Auto.a.nstates !== 2) throw new Error(JSON.stringify(P.Auto.a && P.Auto.a.nstates));
    return P.Auto.a.nstates + ' states, ' + P.Auto.edges.length + ' edges';
  });
  await step('overlap-free query', async () => {
    document.getElementById('editor').value =
      '? ~ E i,n. n>=1 & (A t. t <= n => T[i+t] = T[i+n+t])';
    await P.runScript(false);
    const card = document.querySelector('#results .res');
    if (!card || !card.textContent.includes('TRUE')) throw new Error(card ? card.textContent.slice(0, 60) : 'no card');
    return 'TRUE';
  });
  await step('square witness paints a bracket', async () => {
    document.getElementById('editor').value =
      'witness n>=1 & (A t. t < n => T[i+t] = T[i+n+t])';
    await P.runScript(false);
    if (!P.S.brackets.length) throw new Error('no bracket');
    return JSON.stringify(P.S.brackets[0]);
  });
  await step('enum paints the tape', async () => {
    document.getElementById('editor').value = 'enum 120 T[i] = 1';
    await P.runScript(false);
    if (!P.S.paint || P.S.paint.set.size < 40) throw new Error('paint ' + (P.S.paint && P.S.paint.set.size));
    return P.S.paint.set.size + ' positions';
  });
  await step('fe heatmap', async () => {
    document.getElementById('feSize').value = '32';
    document.getElementById('feL').value = '3';
    await P.Heat.load();
    if (!P.Heat.data || P.Heat.data.rows.length !== 32) throw new Error('rows');
    if (P.Heat.data.rows[0][0] !== '1') throw new Error('diagonal should be true');
    return '32x32 at L=3';
  });
  await step('morphism sandbox rolls an admissible morphism', async () => {
    P.Morph.touched = true;
    P.Morph.roll();
    const bad = P.Morph.check();
    if (bad) throw new Error(bad);
    const rolled = P.Morph.def();
    // the sandbox preview takes the tape over (debounced), so put Thue-Morse back before
    // any later step reads S.seq -- otherwise every check below is against a random morphism
    await new Promise(r => setTimeout(r, 500));
    await P.selectSequence(P.S.seqs[0]);
    if (!P.S.seq.startsWith('0110100110010110')) throw new Error('did not restore thue-morse');
    return rolled;
  });
  await step('streaming job reports phases', async () => {
    document.getElementById('editor').value =
      'let FE(i,j,l) A t. t < l => T[i+t] = T[j+t]\nmem';
    await P.runScript(true);
    await wait(() => document.getElementById('liveLog').textContent.includes('done'), 60000);
    const log = document.getElementById('liveLog').textContent;
    if (!/phase forward/.test(log)) throw new Error('no forward phase');
    if (!/15 states/.test(log)) throw new Error('expected |FE| = 15 for thue-morse');
    return log.trim().split('\n').pop();
  });

  await step('predicate automaton (rebuilt from the script)', async () => {
    if (!P.S.preds.FE) throw new Error('FE was not registered');
    document.getElementById('autoPick').value = 'FE';
    await P.Auto.load('FE');
    if (!P.Auto.a || P.Auto.a.name !== 'FE') throw new Error('no FE automaton');
    if (P.Auto.a.nstates !== 15) throw new Error('|FE| = ' + P.Auto.a.nstates + ', expected 15');
    return P.Auto.a.nstates + ' states over ' + P.Auto.a.alpha + ' track tuples';
  });

  // ------------------------------------------------------------------ live panel
  await step('the live panel settles into a DONE state', async () => {
    const done = document.getElementById('liveDone');
    const chip = document.getElementById('liveVerdict');
    if (document.getElementById('cancelBtn').hidden !== true) throw new Error('stop button still shown');
    if (done.hidden) throw new Error('no done pill');
    if (!/^Done in [\d.]+ (s|ms)$/.test(done.textContent)) throw new Error('pill says ' + done.textContent);
    if (chip.hidden) throw new Error('no verdict chip');
    return `${chip.textContent} · ${done.textContent}`;
  });
  await step('phases run are complete, phases skipped are dashed', async () => {
    const bars = [...document.querySelectorAll('#phasebar div')];
    const cls = Object.fromEntries(bars.map(b => [b.dataset.p, b.className]));
    if (bars.some(b => b.className === '')) throw new Error('a phase is still pending: ' + JSON.stringify(cls));
    if (cls.compile !== 'done' || cls.forward !== 'done') throw new Error(JSON.stringify(cls));
    // `let FE` on Thue-Morse never learns and never verifies
    if (cls.learn !== 'skip' || cls.verify !== 'skip') throw new Error(JSON.stringify(cls));
    return Object.entries(cls).map(([k, v]) => `${k}:${v}`).join(' ');
  });
  await step('a learnfe job lights the learn and verify phases', async () => {
    document.getElementById('editor').value = 'learnfe FE';
    await P.runScript(true);
    await wait(() => document.getElementById('liveDone').hidden === false, 60000);
    const cls = Object.fromEntries([...document.querySelectorAll('#phasebar div')].map(b => [b.dataset.p, b.className]));
    // learnfe learns, then verifies the hypothesis -- and verification itself compiles
    // a formula, so `forward` legitimately runs too
    if (cls.learn !== 'done' || cls.verify !== 'done') throw new Error(JSON.stringify(cls));
    if (cls.brzozowski !== 'skip') throw new Error('brzozowski should not be needed: ' + JSON.stringify(cls));
    return Object.entries(cls).map(([k, v]) => `${k}:${v}`).join(' ');
  });

  // ---------------------------------------------------------------------- shapes
  await step('shape library', async () => {
    if (!P.S.shapes.length) throw new Error('no shapes');
    return P.S.shapes.length + ' shapes';
  });
  await step('predicate picture: T[i]=T[j] on thue-morse', async () => {
    P.show('shapes');
    P.Shapes.open('picture');
    P.Pic.pickShape('eq');
    document.getElementById('picN').value = '32';
    document.getElementById('picI0').value = '0';
    document.getElementById('picJ0').value = '0';
    document.getElementById('picScale').value = '1';
    await P.Pic.load();
    const d = P.Pic.data;
    if (!d || d.rows.length !== 32) throw new Error('no picture');
    // T[i]=T[j] must hold on the diagonal and match the tape everywhere
    for (let i = 0; i < 32; i++) for (let j = 0; j < 32; j++) {
      const want = (P.S.seq[i] === P.S.seq[j]) ? '1' : '0';
      if (d.rows[i][j] !== want) throw new Error(`cell (${i},${j}) = ${d.rows[i][j]}, want ${want}`);
    }
    return '32x32 agrees with the tape';
  });
  await step('predicate picture: Sierpinski by carry-free addition', async () => {
    P.Pic.pickShape('carry2');
    document.getElementById('picN').value = '32';
    await P.Pic.load();
    const d = P.Pic.data;
    const bit = (i, j) => ((i & j) === 0) ? '1' : '0';    // Kummer: no carries <=> no shared bits
    for (let i = 0; i < 32; i++) for (let j = 0; j < 32; j++) {
      if (d.rows[i][j] !== bit(i, j)) throw new Error(`cell (${i},${j}) = ${d.rows[i][j]}, want ${bit(i, j)}`);
    }
    return 'the automaton draws Sierpinski exactly';
  });
  await step('the picture zooms and reads a cell', async () => {
    P.Pic.zoom(2);
    await new Promise(r => setTimeout(r, 700));
    await wait(() => P.Pic.data && P.Pic.data.scale === 2, 20000);
    P.Pic.click({ clientX: 0, clientY: 0 });        // outside the canvas: must not throw
    return 'step ' + P.Pic.data.scale;
  });
  await step('turtle: Thue-Morse at ±90° is a staircase', async () => {
    P.Shapes.open('turtle');
    document.getElementById('turN').value = '512';
    await P.Turtle.load();                       // controls untouched: the default preset
    const a = P.Turtle.angles.join(',');
    if (a !== '90,-90') throw new Error('default angles are [' + a + '], expected the ±90° preset');
    const b = P.Turtle.fitbox;
    // Ground truth: the ±1 partial sums of Thue-Morse lie in {-1,0,1}, so the heading takes
    // only three values and 512 steps march east with a one-unit wobble -- exactly
    // 0 ≤ x ≤ 256, -1 ≤ y ≤ 1.  (Independently computed, not read off the drawing.)
    if (b.minx !== 0 || Math.round(b.maxx) !== 256 || Math.round(b.maxy - b.miny) !== 2)
      throw new Error('bbox ' + JSON.stringify(b) + ', expected x 0..256, y -1..1');
    return `x 0..${Math.round(b.maxx)}, y ${Math.round(b.miny)}..${Math.round(b.maxy)}`;
  });
  await step('turtle draws the paperfolding dragon', async () => {
    await P.selectSequence(P.S.seqs.find(s => s.id === 'paperfolding'));
    P.show('shapes'); P.Shapes.open('turtle');
    P.Turtle.preset('paper');
    document.getElementById('turN').value = '512';
    await P.Turtle.load();
    if (!P.Turtle.pts) throw new Error('no path');
    const b = P.Turtle.fitbox;
    if (!(b.maxx - b.minx > 8 && b.maxy - b.miny > 8)) throw new Error('degenerate path ' + JSON.stringify(b));
    P.Turtle.stop();
    P.Turtle.redraw(512);
    if (P.Turtle.drawn !== 512) throw new Error('drew ' + P.Turtle.drawn);
    await P.selectSequence(P.S.seqs[0]);
    return `512 steps, ${Math.round(b.maxx - b.minx)}x${Math.round(b.maxy - b.miny)} units`;
  });
  await step('sequence as a square', async () => {
    P.Shapes.open('square');
    document.getElementById('sqN').value = '64';
    await P.Square.load();
    if (P.Square.seq.length !== 64 * 64) throw new Error('got ' + P.Square.seq.length + ' terms');
    if (!P.Square.seq.startsWith(P.S.seq.slice(0, 64))) throw new Error('square is a different sequence');
    return '64² terms laid out';
  });

  // Headless Chrome will not lay out below ~500 CSS px, so phone width is checked by
  // constraining the root element instead: same media query, same flex/grid maths.
  await step('the layout survives a 360px root', async () => {
    const de = document.documentElement;
    const prev = de.style.width;
    de.style.width = '360px';
    await new Promise(r => setTimeout(r, 80));
    // every view, not just the one that happens to be open
    const views = ['sequence', 'automaton', 'playground', 'femap', 'shapes', 'morphism', 'live'];
    const panes = ['picture', 'turtle', 'square'];
    // anything inside a deliberately scrollable box (the phone tab bar, a wide table)
    // is allowed to be wider than the screen -- it scrolls on its own
    const scrolls = (n) => {
      for (let p = n.parentElement; p; p = p.parentElement) {
        const ox = getComputedStyle(p).overflowX;
        if (ox === 'auto' || ox === 'scroll') return true;
      }
      return false;
    };
    const over = [];
    for (const v of views) {
      P.show(v);
      for (const pane of (v === 'shapes' ? panes : [null])) {
        if (pane) P.Shapes.open(pane);
        await new Promise(r => setTimeout(r, 40));
        const tag = (n) => `${n.tagName.toLowerCase()}${n.id ? '#' + n.id : ''}` +
          `${typeof n.className === 'string' && n.className ? '.' + n.className.split(' ')[0] : ''}`;
        [...document.querySelectorAll('.app *')]
          .filter(n => n.getBoundingClientRect().right > 361 && !scrolls(n))
          // the parent is named too: an overflowing child is usually a container's fault
          .forEach(n => over.push(`${v}${pane ? '/' + pane : ''} ${tag(n)}@` +
            `${Math.round(n.getBoundingClientRect().right)} in ${n.parentElement ? tag(n.parentElement) : '?'}`));
      }
    }
    de.style.width = prev;
    if (over.length) throw new Error(over.slice(0, 8).join(' | '));
    return 'nothing crosses 360px in any of the ' + views.length + ' views';
  });

  // A select inside a column-direction label is a flex item whose basis lands on the main
  // axis -- i.e. its HEIGHT.  That shipped once as a 160px-tall dropdown on phones, so it
  // is checked rather than looked at.
  await step('no control is stretched out of shape at phone width', async () => {
    const de = document.documentElement;
    const prev = de.style.width;
    de.style.width = '360px';
    const bad = [];
    for (const v of ['sequence', 'automaton', 'playground', 'femap', 'shapes', 'morphism', 'live']) {
      P.show(v);
      for (const pane of (v === 'shapes' ? ['picture', 'turtle', 'square'] : [null])) {
        if (pane) P.Shapes.open(pane);
        await new Promise(r => setTimeout(r, 30));
        [...document.querySelectorAll('select, input[type="number"], input[type="text"]')]
          .filter(n => n.offsetParent && n.offsetHeight > 56)
          .forEach(n => bad.push(`${v} ${n.tagName.toLowerCase()}#${n.id}@${n.offsetHeight}px`));
      }
    }
    de.style.width = prev;
    if (bad.length) throw new Error(bad.slice(0, 6).join(' | '));
    return 'every select and input is one line tall';
  });

  await step('the page never scrolls sideways', async () => {
    const de = document.documentElement;
    if (de.scrollWidth <= de.clientWidth + 1) return 'clean at ' + de.clientWidth + 'px';
    const over = [...document.querySelectorAll('body *')]
      .filter(n => n.getBoundingClientRect().right > de.clientWidth + 1)
      .map(n => `${n.tagName.toLowerCase()}${n.id ? '#' + n.id : ''}${n.className && typeof n.className === 'string' ? '.' + n.className.split(' ')[0] : ''}` +
                `@${Math.round(n.getBoundingClientRect().right)}`);
    throw new Error(`scrollWidth ${de.scrollWidth} > ${de.clientWidth}; ` + over.slice(0, 8).join(' | '));
  });

  // ?end=<view> leaves the app on a view for a screenshot; ?end=shapes/<pane> picks the tab
  const end = new URLSearchParams(location.search).get('end');
  if (end) {
    const [view, pane] = end.split('/');
    P.show(view);
    if (pane) P.Shapes.open(pane);
    if (pane === 'picture') await P.Pic.load();   // so a screenshot has a picture in it
  }

  out.textContent += 'SELFTEST DONE\n';
  document.title = 'selftest ' + (out.textContent.includes('FAIL') ? 'FAILED' : 'ok');
})();
