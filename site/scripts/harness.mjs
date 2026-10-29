// What the in-browser checks share: a tiny static server for dist/ under the
// site's base path, a harness page that it also serves, and headless Chrome,
// whose --dump-dom brings the harness's result back as text. Nothing else is
// needed: no browser driver, no test runner, nothing from the network.
import { execFileSync, spawn } from 'node:child_process';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { dirname, extname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

export const site = join(dirname(fileURLToPath(import.meta.url)), '..');
export const dist = join(site, 'dist');
export const base = (process.env.SITE_BASE ?? '/sprig').replace(/\/$/, '');

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
	throw new Error('no Chrome found; install Chrome or set CHROME_PATH');
}

function walk(dir) {
	return readdirSync(dir).flatMap((n) => {
		const p = join(dir, n);
		return statSync(p).isDirectory() ? walk(p) : [p];
	});
}

/** Every built page, as a path under the base. */
export const pages = () =>
	walk(dist)
		.filter((f) => f.endsWith('.html') && !relative(dist, f).startsWith('pagefind'))
		.map((f) => `${base}/${relative(dist, f).replace(/index\.html$/, '')}`);

const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.json': 'application/json', '.xml': 'application/xml', '.txt': 'text/plain', '.md': 'text/markdown', '.wasm': 'application/wasm' };

/**
 * Serve dist/ and `extra` (path to body), open the harness at `/__harness.html`
 * in Chrome, and resolve with the text of its `<pre id="out">` once it stops
 * saying `pending`. The harness reports failure by starting with `ERROR`.
 */
export function run({ harness, extra = {}, budget }) {
	const server = createServer((req, res) => {
		const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
		if (path === '/__harness.html') return res.writeHead(200, { 'Content-Type': 'text/html' }).end(harness);
		if (path in extra) return res.writeHead(200, { 'Content-Type': types[extname(path)] ?? 'text/plain' }).end(extra[path]);
		if (!path.startsWith(`${base}/`)) return res.writeHead(404).end();
		let file = join(dist, path.slice(base.length));
		if (existsSync(file) && statSync(file).isDirectory()) file = join(file, 'index.html');
		if (!existsSync(file)) return res.writeHead(404).end();
		res.writeHead(200, { 'Content-Type': types[extname(file)] ?? 'application/octet-stream' }).end(readFileSync(file));
	});
	return new Promise((resolve, reject) => {
		server.listen(0, '127.0.0.1', () => {
			const { port } = server.address();
			const args = ['--headless=new', '--disable-gpu', '--no-sandbox', '--window-size=1400,1000', `--virtual-time-budget=${budget}`, '--dump-dom', `http://127.0.0.1:${port}/__harness.html`];
			const proc = spawn(chrome(), args, { stdio: ['ignore', 'pipe', 'ignore'] });
			let dom = '';
			proc.stdout.on('data', (d) => (dom += d));
			proc.on('error', reject);
			proc.on('close', () => {
				server.close();
				const out = dom.match(/<pre id="out">([\s\S]*?)<\/pre>/)?.[1]?.replace(/&quot;/g, '"').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
				if (!out || out === 'pending' || out.startsWith('ERROR')) reject(new Error(`the harness did not finish${out ? `: ${out}` : ''}`));
				else resolve(out);
			});
		});
	});
}
