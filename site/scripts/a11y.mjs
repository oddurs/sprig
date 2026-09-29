// Run axe on every built page, in the light theme and the dark one, and fail
// on any WCAG 2.1 A or AA violation. It needs Chrome (set CHROME_PATH, or have
// Google Chrome or Chromium installed), and nothing else: a tiny static server
// serves dist/ plus a harness page, and Chrome's --dump-dom brings the results
// back as text.
import { execFileSync, spawn } from 'node:child_process';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { dirname, extname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const site = join(dirname(fileURLToPath(import.meta.url)), '..');
const dist = join(site, 'dist');
const base = (process.env.SITE_BASE ?? '/sprig').replace(/\/$/, '');
const axe = readFileSync(join(site, 'node_modules/axe-core/axe.min.js'), 'utf8');

function chrome() {
	if (process.env.CHROME_PATH) return process.env.CHROME_PATH;
	const mac = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
	if (existsSync(mac)) return mac;
	for (const name of ['google-chrome', 'google-chrome-stable', 'chromium', 'chromium-browser']) {
		try {
			return execFileSync('which', [name], { encoding: 'utf8' }).trim();
		} catch {
			// not this one; try the next name
		}
	}
	throw new Error('a11y: no Chrome found; install Chrome or set CHROME_PATH');
}

function walk(dir) {
	return readdirSync(dir).flatMap((n) => {
		const p = join(dir, n);
		return statSync(p).isDirectory() ? walk(p) : [p];
	});
}
const pages = walk(dist)
	.filter((f) => f.endsWith('.html') && !relative(dist, f).startsWith('pagefind'))
	.map((f) => `${base}/${relative(dist, f).replace(/index\.html$/, '')}`);

const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.json': 'application/json', '.xml': 'application/xml', '.txt': 'text/plain', '.md': 'text/markdown' };

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

const server = createServer((req, res) => {
	const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
	if (path === '/__harness.html') return res.writeHead(200, { 'Content-Type': 'text/html' }).end(harness);
	if (path === '/__axe.js') return res.writeHead(200, { 'Content-Type': 'text/javascript' }).end(axe);
	if (!path.startsWith(`${base}/`)) return res.writeHead(404).end();
	let file = join(dist, path.slice(base.length));
	if (existsSync(file) && statSync(file).isDirectory()) file = join(file, 'index.html');
	if (!existsSync(file)) return res.writeHead(404).end();
	res.writeHead(200, { 'Content-Type': types[extname(file)] ?? 'application/octet-stream' }).end(readFileSync(file));
});

server.listen(0, '127.0.0.1', () => {
	const { port } = server.address();
	const args = ['--headless=new', '--disable-gpu', '--no-sandbox', '--window-size=1400,1000', `--virtual-time-budget=${pages.length * 8000}`, '--dump-dom', `http://127.0.0.1:${port}/__harness.html`];
	const proc = spawn(chrome(), args, { stdio: ['ignore', 'pipe', 'ignore'] });
	let dom = '';
	proc.stdout.on('data', (d) => (dom += d));
	proc.on('close', () => {
		server.close();
		const out = dom.match(/<pre id="out">([\s\S]*?)<\/pre>/)?.[1]?.replace(/&quot;/g, '"').replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>');
		if (!out || out === 'pending' || out.startsWith('ERROR')) {
			console.error(`a11y: the harness did not finish${out ? `: ${out}` : ''}`);
			process.exit(1);
		}
		const violations = JSON.parse(out);
		if (violations.length) {
			console.error(`a11y: ${violations.length} violation(s)`);
			for (const v of violations) console.error(`  ${v.page} (${v.theme}) ${v.impact} ${v.id}: ${v.help}\n    ${v.nodes.join('\n    ')}`);
			process.exit(1);
		}
		console.log(`a11y: ${pages.length} pages, light and dark, no WCAG 2.1 AA violations`);
	});
});
