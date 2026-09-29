// Hold the site to its tokens. Outside src/design/tokens.mjs (and the SVG
// assets, which are images), site/src may not write:
//
//   - a colour literal (hex, rgb, hsl, oklch): colours are tokens
//   - a spacing, type size, weight or radius value off the scales
//   - a media query width that isn't one of the three breakpoints
//
// Each hit is either a missing token or a leak, and either way the build stops.
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { breakpoints } from '../src/design/tokens.mjs';

const site = join(dirname(fileURLToPath(import.meta.url)), '..');
const src = join(site, 'src');
const skip = [join(src, 'design/tokens.mjs'), join(src, 'generated'), join(src, 'content')];

function walk(dir) {
	return readdirSync(dir).flatMap((name) => {
		const path = join(dir, name);
		if (skip.some((s) => path.startsWith(s))) return [];
		return statSync(path).isDirectory() ? walk(path) : [path];
	});
}
const files = walk(src).filter((f) => /\.(css|astro|js|mjs|ts)$/.test(f));

const widths = new Set(Object.values(breakpoints).flatMap((w) => [w, w - 1]));
const scaled = /^(margin|padding|gap|row-gap|column-gap|inset|font-size|border-radius)(-[a-z-]+)?$/;
const allowedLiteral = /^(0|1px|1\.5px|2px|auto|inherit|none|0\.9em|calc\(.*\)|var\(.*\)|\s|-)+$/;

const problems = [];
for (const file of files) {
	const text = readFileSync(file, 'utf8');
	const where = (i) => `${relative(site, file)}:${text.slice(0, i).split('\n').length}`;

	for (const m of text.matchAll(/(?<![\w&-])#[0-9a-fA-F]{3}(?:[0-9a-fA-F]{3})?(?:[0-9a-fA-F]{2})?\b|\b(?:rgba?|hsla?|oklch|oklab|lab|lch)\(/g)) {
		// Anchors and ids in URLs (`#main`, `#plan=`) are not colours; only hex-shaped runs are checked.
		if (m[0].startsWith('#') && !/^#[0-9a-fA-F]+$/.test(m[0])) continue;
		if (m[0].startsWith('#') && /[g-zG-Z]/.test(text.slice(m.index + m[0].length, m.index + m[0].length + 1))) continue;
		problems.push(`${where(m.index)}: colour literal ${m[0]}; use a token`);
	}
	// Declarations in CSS and in <style> blocks.
	const css = file.endsWith('.css') ? text : [...text.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]).join('\n');
	const offset = file.endsWith('.css') ? 0 : text.indexOf('<style');
	for (const m of css.matchAll(/([a-z-]+)\s*:\s*([^;{}]+);/g)) {
		const [, prop, value] = m;
		if (prop === 'font-weight' && /\b\d{3}\b/.test(value)) problems.push(`${where(offset + m.index)}: font-weight ${value.trim()}; use --weight-*`);
		if (!scaled.test(prop)) continue;
		const bare = value.replace(/var\([^)]*\)/g, '').replace(/calc\([^)]*\)/g, '').trim();
		if (/\d/.test(bare) && !allowedLiteral.test(bare)) problems.push(`${where(offset + m.index)}: ${prop}: ${value.trim()} is off the scale; use a token`);
	}
	for (const m of text.matchAll(/@media[^{]*?\((?:min|max)-width:\s*(\d+)px\)/g)) {
		if (!widths.has(Number(m[1]))) problems.push(`${where(m.index)}: media query at ${m[1]}px; breakpoints are ${Object.values(breakpoints).join(', ')}`);
	}
}

if (problems.length) {
	console.error(`lint-tokens: ${problems.length} problem(s)\n  ${problems.join('\n  ')}`);
	process.exit(1);
}
console.log(`lint-tokens: ${files.length} files use tokens only`);
