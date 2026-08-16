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
    return P.Morph.def();
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

  // Headless Chrome will not lay out below ~500 CSS px, so phone width is checked by
  // constraining the root element instead: same media query, same flex/grid maths.
  await step('the layout survives a 360px root', async () => {
    const de = document.documentElement;
    const prev = de.style.width;
    de.style.width = '360px';
    await new Promise(r => setTimeout(r, 80));
    // anything inside a deliberately scrollable box (the phone tab bar, a wide table)
    // is allowed to be wider than the screen -- it scrolls on its own
    const scrolls = (n) => {
      for (let p = n.parentElement; p; p = p.parentElement) {
        const ox = getComputedStyle(p).overflowX;
        if (ox === 'auto' || ox === 'scroll') return true;
      }
      return false;
    };
    const over = [...document.querySelectorAll('.app *')]
      .filter(n => n.getBoundingClientRect().right > 361 && !scrolls(n))
      .map(n => `${n.tagName.toLowerCase()}${n.id ? '#' + n.id : ''}` +
                `${typeof n.className === 'string' && n.className ? '.' + n.className.split(' ')[0] : ''}` +
                `@${Math.round(n.getBoundingClientRect().right)}`);
    de.style.width = prev;
    if (over.length) throw new Error(over.slice(0, 8).join(' | '));
    return 'nothing crosses 360px';
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

  const end = new URLSearchParams(location.search).get('end');
  if (end) P.show(end);

  out.textContent += 'SELFTEST DONE\n';
  document.title = 'selftest ' + (out.textContent.includes('FAIL') ? 'FAILED' : 'ok');
})();
