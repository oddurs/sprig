// The playground, driven the way a person drives it, wide and narrow, in the
// built site: what it does, not how it looks. Each check names the behaviour
// it holds, and the run fails on the first that does not hold. It needs
// Chrome, as scripts/a11y.mjs does.
import { base, run } from './harness.mjs';

const harness = `<!doctype html><meta charset="utf-8"><body><pre id="out">pending</pre><script>
const PLAY = ${JSON.stringify(`${base}/play/`)};
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const passed = [];
async function open(width, hash = '') {
  const f = document.createElement('iframe');
  f.style.cssText = 'width:' + width + 'px;height:800px;border:0';
  document.body.appendChild(f);
  await new Promise((r) => { f.onload = r; f.src = PLAY; });
  // A clean start each time: no saved workspace, no saved panel.
  f.contentWindow.localStorage.clear();
  await new Promise((r) => { f.onload = r; f.src = PLAY + '?fresh=' + Math.random() + hash; });
  const d = f.contentDocument;
  for (let i = 0; i < 100 && !d.querySelector('#probsHead')?.textContent.includes('sprig check'); i++) await wait(50);
  return { f, d, w: f.contentWindow, $: (s) => d.querySelector(s), $$: (s) => [...d.querySelectorAll(s)] };
}
function holds(name, ok, detail = '') {
  if (!ok) throw new Error(name + (detail ? ' (' + detail + ')' : ''));
  passed.push(name);
}
async function type(p, value) {
  const ta = p.$('#src');
  ta.value = value;
  ta.dispatchEvent(new p.w.Event('input'));
  await wait(200);
}
const click = async (el) => { el.click(); await wait(120); };

(async () => {
  // ---------- wide ----------
  let p = await open(1280);
  holds('the examples load with no problems', p.$('#probsHead').textContent.includes('No problems'), p.$('#probsHead').textContent);
  const src = p.$('#src').value;
  holds('the example has the link this run breaks', src.includes('after:[[lease^signed]]'));

  await type(p, src.replace('after:[[lease^signed]]', 'after:[[lease^signd]]'));
  const probs = p.$$('.prob');
  holds('a broken anchor is listed under Problems', probs.length === 1, probs.length + ' listed');
  holds('a problem shows its code, linked to its note', probs[0].querySelector('.code').textContent === 'S0203' && probs[0].querySelector('.code').getAttribute('href').endsWith('/docs/reference/diagnostics/#s0203'));
  holds('a problem says where it is', /bakery\\.sprig:\\d+:\\d+/.test(probs[0].querySelector('.where').textContent));
  holds('the tab of a file with an error carries a dot', !!p.$('#tabs [data-file="bakery.sprig"] .dot.error'));
  const line = Number(probs[0].querySelector('.where').textContent.split(':')[1]);
  holds('the line with the problem is marked in the editor', p.$('#hl [data-l="' + (line - 1) + '"]').classList.contains('l-error'));

  await click(probs[0].querySelector('.prob-go'));
  await wait(100);
  const ta = p.$('#src');
  const at = ta.value.slice(0, ta.selectionStart).split('\\n').length;
  holds('choosing a problem selects its line', at === line, 'line ' + at + ', not ' + line);

  await type(p, src);
  holds('fixing the line clears the problem', p.$$('.prob').length === 0 && !p.$('#tabs .dot'));

  // Rename lease.sprig: every [[lease...]] link follows it.
  await click(p.$('#tabs [data-file="lease.sprig"]'));
  await click(p.$('#fileMenuBtn'));
  holds('the file menu opens', !p.$('#fileMenu').hidden);
  await click(p.$('[data-file-act="rename"]'));
  const name = p.$('#newName');
  name.value = 'deal';
  name.dispatchEvent(new p.w.KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
  await wait(200);
  holds('a renamed file keeps its place among the tabs', p.$$('#tabs [data-file]').map((t) => t.dataset.file)[1] === 'deal.sprig');
  await click(p.$('#tabs [data-file="bakery.sprig"]'));
  const after = p.$('#src').value;
  holds('a rename carries the links to the file with it', after.includes('+ [[deal]]') && after.includes('[[deal^signed]]') && !after.includes('[[lease'));
  holds('a rename breaks nothing', p.$$('.prob').length === 0, p.$$('.prob').length + ' problems');

  // Delete asks first.
  await click(p.$('#tabs [data-file="dana.sprig"]'));
  await click(p.$('#fileMenuBtn'));
  await click(p.$('[data-file-act="delete"]'));
  holds('delete asks before it deletes', !!p.$('#tabs [data-file="dana.sprig"]') && p.$('[data-file-act="delete"]').classList.contains('armed'));
  await click(p.$('[data-file-act="delete"]'));
  holds('the second press deletes the file', !p.$('#tabs [data-file="dana.sprig"]'));

  // Share: the link holds the workspace, and opening it brings the workspace back.
  p.w.navigator.clipboard.writeText = async () => {};
  await click(p.$('#share'));
  await wait(300);
  const hash = p.w.location.hash;
  holds('share puts the workspace in the link', hash.startsWith('#plan='));
  const q = await open(1280, hash);
  holds('a shared link opens the same workspace', q.$$('#tabs [data-file]').map((t) => t.dataset.file).join() === p.$$('#tabs [data-file]').map((t) => t.dataset.file).join());
  p.f.remove();
  q.f.remove();

  // ---------- narrow ----------
  p = await open(375);
  const shown = (s) => p.w.getComputedStyle(p.$(s)).display !== 'none';
  holds('narrow, the source is the first panel', shown('.src') && !shown('.rv') && !shown('.probs'));
  await click(p.$('#tab-view'));
  holds('narrow, Plan shows the plan alone', shown('.rv') && !shown('.src'));
  await click(p.$('#tab-src'));
  holds('narrow, the editor grows with its text instead of scrolling', p.$('#src').scrollHeight <= p.$('#src').clientHeight + 2, p.$('#src').scrollHeight + ' > ' + p.$('#src').clientHeight);

  await type(p, p.$('#src').value.replace('after:[[lease^signed]]', 'after:[[lease^nope]]'));
  holds('narrow, the Problems switch counts the problems', p.$('#pcount').textContent === '1');
  await click(p.$('#tab-problems'));
  await click(p.$('.prob-go'));
  holds('narrow, choosing a problem goes back to the source', shown('.src') && !shown('.probs'));

  // The key bar indents the line the caret is on, as Tab does.
  const t = p.$('#src');
  const v = t.value;
  const i = v.indexOf('- Supplier credit line');
  t.focus();
  t.setSelectionRange(i + 3, i + 3);
  await wait(50);
  holds('narrow, the key bar shows while editing', p.w.getComputedStyle(p.$('#keybar')).display !== 'none');
  await click(p.$('[data-key="indent"]'));
  holds('the key bar indents the line', t.value.includes('    - Supplier credit line'));
  await click(p.$('[data-key="cycle"]'));
  holds('the key bar cycles the mark', t.value.includes('    ~ Supplier credit line'));
  p.f.remove();

  document.getElementById('out').textContent = JSON.stringify(passed);
})().catch((e) => { document.getElementById('out').textContent = 'ERROR ' + e.message + ' (after: ' + passed.join('; ') + ')'; });
</script>`;

run({ harness, budget: 60000 })
	.then((out) => {
		const passed = JSON.parse(out);
		console.log(`playground: ${passed.length} behaviours hold, wide and narrow`);
	})
	.catch((e) => {
		console.error(`playground: ${e.message}`);
		process.exit(1);
	});
