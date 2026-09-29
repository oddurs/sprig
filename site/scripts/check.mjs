// Check the built site, after `astro build`, before anything ships.
//
// 1. Every internal link resolves to a file in dist/, and every #fragment to an
//    element with that id. This covers the hand-built pages and the generated
//    ones alike, which a content-only link checker would miss.
// 2. The site loads nothing from another host: no external scripts, styles,
//    images or fonts. Links out to other sites are fine; requests are not.
//
// Links in playground fragments (#plan=...) are data, not anchors, and skipped.
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const site = join(dirname(fileURLToPath(import.meta.url)), '..');
const dist = join(site, 'dist');
const base = (process.env.SITE_BASE ?? '/sprig').replace(/\/$/, '');

function walk(dir) {
	return readdirSync(dir).flatMap((name) => {
		const path = join(dir, name);
		return statSync(path).isDirectory() ? walk(path) : [path];
	});
}
const all = walk(dist);
const pages = all.filter((f) => f.endsWith('.html'));
const idsOf = new Map();
function ids(file) {
	if (!idsOf.has(file)) {
		const html = readFileSync(file, 'utf8');
		idsOf.set(file, new Set([...html.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1])));
	}
	return idsOf.get(file);
}

// Map a site path to the file that serves it, as GitHub Pages would.
function target(path) {
	if (!path.startsWith(`${base}/`) && path !== base) return null;
	const rel = decodeURIComponent(path.slice(base.length)) || '/';
	const candidates = rel.endsWith('/') ? [join(dist, rel, 'index.html')] : [join(dist, rel), join(dist, rel, 'index.html'), join(dist, `${rel}.html`)];
	return candidates.find((c) => existsSync(c) && statSync(c).isFile()) ?? null;
}

const problems = [];
let checked = 0;
for (const page of pages) {
	const html = readFileSync(page, 'utf8');
	const where = relative(dist, page);

	for (const [, attr, raw] of html.matchAll(/\s(href|src)="([^"]*)"/g)) {
		const value = raw.replace(/&amp;/g, '&');
		if (/^(mailto:|tel:|data:|javascript:)/.test(value)) continue;
		const external = /^(https?:)?\/\//.test(value);
		if (external) continue; // requests to other hosts are checked below
		checked++;
		const url = new URL(value, `https://site.invalid${'/' + relative(dist, page).replace(/index\.html$/, '')}`);
		const [path, hash] = [url.pathname, url.hash.slice(1)];
		const file = value.startsWith('#') ? page : target(path);
		if (!file) {
			problems.push(`${where}: ${attr}="${value}" points at nothing`);
			continue;
		}
		if (hash && !hash.startsWith('plan=') && file.endsWith('.html') && !ids(file).has(decodeURIComponent(hash))) {
			problems.push(`${where}: ${attr}="${value}" has no element with id "${hash}"`);
		}
	}

	// Requests to other hosts: script src, stylesheet or preload links, images, media, and CSS url().
	const requests = [
		...html.matchAll(/<(script|img|iframe|source|video|audio)[^>]*\ssrc="((?:https?:)?\/\/[^"]+)"/g),
		...html.matchAll(/<link[^>]*\srel="(stylesheet|preload|modulepreload|icon|preconnect|dns-prefetch)"[^>]*\shref="((?:https?:)?\/\/[^"]+)"/g),
		...html.matchAll(/url\(\s*['"]?((?:https?:)?\/\/[^'")]+)/g),
	];
	for (const m of requests) problems.push(`${where}: loads from another host: ${m[2] ?? m[1]}`);
}

for (const css of all.filter((f) => f.endsWith('.css'))) {
	for (const m of readFileSync(css, 'utf8').matchAll(/url\(\s*['"]?((?:https?:)?\/\/[^'")]+)/g)) {
		problems.push(`${relative(dist, css)}: loads from another host: ${m[1]}`);
	}
}
const fonts = all.filter((f) => /\.(woff2?|ttf|otf)$/.test(f));
for (const f of fonts) problems.push(`${relative(dist, f)}: a font file shipped; the site uses system fonts only`);

if (problems.length) {
	console.error(`check: ${problems.length} problem(s)\n  ${problems.join('\n  ')}`);
	process.exit(1);
}
console.log(`check: ${pages.length} pages, ${checked} internal links and assets resolve, no third-party requests, no font files`);
