// Draw the favicon, the logo and the social image from the design tokens, so
// their colours can't drift from the site's. Run it after changing a token:
//   node scripts/assets.mjs
// It needs rsvg-convert (librsvg) to render og.png; the SVGs are written either way.
import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { markGroups, marks, themes } from '../src/design/tokens.mjs';

const site = join(dirname(fileURLToPath(import.meta.url)), '..');
const light = themes.light;
const dark = themes.dark;

// The mark: an ink tile with a stem and two branches, the nodes coloured as
// done and doing, the two marks a plan is mostly made of.
const mark = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" role="img" aria-label="Sprig">
  <rect width="32" height="32" rx="7" fill="${light.text}"/>
  <path d="M11 7.5V24.5M11 13.5H18.5M11 21.5H18.5" fill="none" stroke="${light.surface}" stroke-width="2.6" stroke-linecap="round"/>
  <circle cx="22" cy="13.5" r="2.6" fill="${dark['mark-done']}"/>
  <circle cx="22" cy="21.5" r="2.6" fill="${dark['mark-doing']}"/>
</svg>
`;
writeFileSync(join(site, 'src/assets/mark.svg'), mark);
writeFileSync(join(site, 'public/favicon.svg'), mark);

// The social image: the name, the thesis, and the nine marks, which are the idea.
const W = 1280;
const pad = 96;
const col = (W - pad * 2) / marks.length;
const sans = 'Helvetica Neue, Helvetica, Arial, sans-serif';
const mono = 'Menlo, SF Mono, Consolas, monospace';
const glyphs = marks
	.map((m, i) => {
		const x = pad + col * i + col / 2;
		return `  <text x="${x}" y="462" text-anchor="middle" font-family="${mono}" font-size="84" font-weight="500" fill="${light[`mark-${m.key}`]}">${m.char.replace('>', '&gt;')}</text>
  <text x="${x}" y="514" text-anchor="middle" font-family="${sans}" font-size="20" fill="${light['text-muted']}">${m.name}</text>`;
	})
	.join('\n');
let start = 0;
const dividers = markGroups
	.slice(0, -1)
	.map((g) => {
		start += marks.filter((m) => m.group === g.key).length;
		const x = pad + col * start;
		return `  <line x1="${x}" y1="378" x2="${x}" y2="530" stroke="${light.border}" stroke-width="2"/>`;
	})
	.join('\n');
const og = `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="640" viewBox="0 0 ${W} 640">
  <rect width="${W}" height="640" fill="${light.surface}"/>
  <g transform="translate(${pad} 96) scale(1.5)">${mark.replace(/<\/?svg[^>]*>/g, '').trim()}</g>
  <text x="${pad + 64}" y="131" font-family="${sans}" font-size="44" font-weight="700" fill="${light.text}">Sprig</text>
  <text x="${pad}" y="250" font-family="${sans}" font-size="56" font-weight="700" letter-spacing="-1.5" fill="${light.text}">Write the plan. Sprig works out the rest.</text>
  <text x="${pad}" y="300" font-family="${sans}" font-size="26" fill="${light['text-muted']}">A plain-text format for plans. The whole language is nine marks.</text>
${dividers}
${glyphs}
</svg>
`;
writeFileSync(join(site, 'src/assets/og.svg'), og);

try {
	execFileSync('rsvg-convert', ['-w', '1280', '-h', '640', join(site, 'src/assets/og.svg'), '-o', join(site, 'public/og.png')]);
	console.log('assets: mark.svg, favicon.svg, og.svg and og.png');
} catch {
	console.error('assets: SVGs written; install librsvg (rsvg-convert) to render og.png');
	process.exit(1);
}
