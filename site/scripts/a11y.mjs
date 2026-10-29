// Run axe on every built page, in the light theme and the dark one, and fail
// on any WCAG 2.1 A or AA violation. It needs Chrome (set CHROME_PATH, or have
// Google Chrome or Chromium installed), and nothing else: a tiny static server
// serves dist/ plus a harness page, and Chrome's --dump-dom brings the results
// back as text.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pages as built, run, site } from './harness.mjs';

const axe = readFileSync(join(site, 'node_modules/axe-core/axe.min.js'), 'utf8');
const pages = built();

const harness = `<!doctype html><meta charset="utf-8"><body><pre id="out">pending</pre><script>
const pages = ${JSON.stringify(pages)};
const results = [];
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
async function check(page, theme) {
  const f = document.createElement('iframe');
  f.style.cssText = 'width:1280px;height:900px;border:0';
  document.body.appendChild(f);
  await new Promise((r) => { f.onload = r; f.src = page; });
  const d = f.contentDocument;
  const still = d.createElement('style');
  still.textContent = '*, *::before, *::after { transition: none !important; animation: none !important; }';
  d.head.appendChild(still);
  d.documentElement.dataset.theme = theme;
  await wait(250);
  const s = d.createElement('script');
  s.src = '/__axe.js';
  await new Promise((r) => { s.onload = r; d.head.appendChild(s); });
  const res = await f.contentWindow.axe.run(d, { runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'] } });
  for (const v of res.violations) results.push({ page, theme, id: v.id, impact: v.impact, help: v.help, nodes: v.nodes.slice(0, 3).map((n) => { const d = n.any[0] && n.any[0].data; return n.target.join(' ') + (d && d.fgColor ? ' (' + d.fgColor + ' on ' + d.bgColor + ', ' + d.contrastRatio + ':1)' : ''); }) });
  f.remove();
}
(async () => {
  for (const p of pages) for (const t of ['light', 'dark']) await check(p, t);
  document.getElementById('out').textContent = JSON.stringify(results);
})().catch((e) => { document.getElementById('out').textContent = 'ERROR ' + e.stack; });
</script>`;

run({ harness, extra: { '/__axe.js': axe }, budget: pages.length * 8000 })
	.then((out) => {
		const violations = JSON.parse(out);
		if (violations.length) {
			console.error(`a11y: ${violations.length} violation(s)`);
			for (const v of violations) console.error(`  ${v.page} (${v.theme}) ${v.impact} ${v.id}: ${v.help}\n    ${v.nodes.join('\n    ')}`);
			process.exit(1);
		}
		console.log(`a11y: ${pages.length} pages, light and dark, no WCAG 2.1 AA violations`);
	})
	.catch((e) => {
		console.error(`a11y: ${e.message}`);
		process.exit(1);
	});
