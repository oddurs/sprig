// Sprig's design tokens: the one place a colour, a size or a duration is
// written down. Everything else reads them as CSS custom properties (emitted by
// css() below into src/generated/tokens.css) or through the syntax themes for
// the docs' code blocks (shikiTheme()). A colour anywhere else in site/src is a
// bug, and scripts/lint-tokens.mjs fails the build on one.
//
// The idea the palette serves: the nine marks are the only colour on the site.
// Everything around them is a quiet neutral, so the marks read as the concept
// they are. Their colours are set in OKLCH at one lightness and chroma per
// theme, so no mark is louder than another, and each clears 4.5:1 against
// every surface it can sit on (checked by scripts/contrast.mjs).

/* ---------- colour maths ---------- */

const toLinear = (c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
const encode = (x) => {
	const c = Math.min(1, Math.max(0, x));
	return c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055;
};

/** OKLCH to an sRGB hex string. */
export function oklch(L, C, h) {
	const a = C * Math.cos((h * Math.PI) / 180);
	const b = C * Math.sin((h * Math.PI) / 180);
	const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
	const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
	const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
	const rgb = [
		4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
		-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
		-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
	];
	return `#${rgb.map((c) => Math.round(encode(c) * 255).toString(16).padStart(2, '0')).join('')}`;
}

/** WCAG contrast ratio between two hex colours. */
export function contrast(x, y) {
	const lum = (hex) => {
		const [r, g, b] = [1, 3, 5].map((i) => toLinear(parseInt(hex.slice(i, i + 2), 16) / 255));
		return 0.2126 * r + 0.7152 * g + 0.0722 * b;
	};
	const [hi, lo] = [lum(x), lum(y)].sort((p, q) => q - p);
	return (hi + 0.05) / (lo + 0.05);
}

/* ---------- primitives ---------- */

// A neutral ramp with a faint warm bias, named by OKLCH lightness. Nothing in
// the UI uses these directly; the semantic layer below picks from them.
const NEUTRAL_HUE = 100;
const n = (L, C = 0.004) => oklch(L, C, NEUTRAL_HUE);
export const neutral = {
	985: n(0.985),
	960: n(0.96, 0.005),
	920: n(0.92, 0.005),
	600: n(0.6, 0.005),
	500: n(0.5, 0.006),
	210: n(0.21),
	300: n(0.3, 0.005),
	530: n(0.53, 0.005),
	720: n(0.72, 0.005),
	930: n(0.93, 0.004),
	170: n(0.17),
	215: n(0.215, 0.005),
};

// The marks, in the order the language lists them. `to do` and `dropped` are
// deliberately neutral: the default state and the dead one. The seven others
// each own a hue.
export const marks = [
	{ key: 'todo', char: '-', name: 'to do', group: 'status', meaning: 'Something to do. The default.', rule: '22-marks' },
	{ key: 'doing', char: '~', name: 'doing', group: 'status', meaning: 'Started. Hands are on it.', rule: '22-marks', hue: 70 },
	{ key: 'done', char: 'x', name: 'done', group: 'status', meaning: 'Finished. Settles its branch.', rule: '52-settling', hue: 150 },
	{ key: 'dropped', char: '/', name: 'dropped', group: 'status', meaning: 'Decided against. Kept for the record.', rule: '52-settling' },
	{ key: 'later', char: '>', name: 'later', group: 'status', meaning: 'Parked on purpose. Out of the count.', rule: '53-what-counts', hue: 255 },
	{ key: 'ask', char: '?', name: 'question', group: 'decide', meaning: 'Open until it has an answer.', rule: '25-questions-and-answers', hue: 300 },
	{ key: 'answer', char: '=', name: 'answer', group: 'decide', meaning: 'Settles the question above it.', rule: '25-questions-and-answers', hue: 350 },
	{ key: 'group', char: '#', name: 'group', group: 'structure', meaning: 'A heading. Its status comes from its children.', rule: '24-groups', hue: 195 },
	{ key: 'graft', char: '+', name: 'graft', group: 'structure', meaning: 'Another file or branch, mounted here.', rule: '81-graft-lines', hue: 38 },
];
export const markGroups = [
	{ key: 'status', name: 'Status' },
	{ key: 'decide', name: 'Decide' },
	{ key: 'structure', name: 'Structure' },
];

// One lightness and chroma per theme for every chromatic mark.
const MARK_LIGHT = { L: 0.5, C: 0.13 };
const MARK_DARK = { L: 0.78, C: 0.12 };

/* ---------- semantic tokens ---------- */

function markColours(theme, base) {
	const { L, C } = theme === 'light' ? MARK_LIGHT : MARK_DARK;
	return Object.fromEntries(
		marks.map((m) => [
			`mark-${m.key}`,
			m.key === 'todo' ? base.text : m.key === 'dropped' ? base['text-muted'] : oklch(L, C, m.hue),
		]),
	);
}

const lightBase = {
	surface: neutral[985],
	'surface-raised': neutral[960],
	border: neutral[920],
	'border-strong': neutral[600],
	text: neutral[210],
	'text-muted': neutral[500],
	focus: neutral[210],
};
const darkBase = {
	surface: neutral[170],
	'surface-raised': neutral[215],
	border: neutral[300],
	'border-strong': neutral[530],
	text: neutral[930],
	'text-muted': neutral[720],
	focus: neutral[930],
};

/** Semantic colour tokens per theme. Components use only these. */
export const themes = {
	light: { ...lightBase, ...markColours('light', lightBase) },
	dark: { ...darkBase, ...markColours('dark', darkBase) },
};

/** Which text tokens must be legible on which surfaces, and how strongly. */
export const contrastRules = [
	...['text', 'text-muted', ...marks.map((m) => `mark-${m.key}`)].flatMap((fg) =>
		['surface', 'surface-raised'].map((bg) => ({ fg, bg, min: 4.5 })),
	),
	// Control boundaries (a secondary button's edge, the editor frame) need 3:1.
	{ fg: 'border-strong', bg: 'surface', min: 3 },
	{ fg: 'focus', bg: 'surface', min: 3 },
	{ fg: 'focus', bg: 'surface-raised', min: 3 },
];

/* ---------- scales ---------- */

export const space = { 1: '4px', 2: '8px', 3: '12px', 4: '16px', 5: '24px', 6: '32px', 7: '48px', 8: '64px', 9: '96px', 10: '128px' };

// A 1.25 ratio from 16px, rounded, each size with its own line height.
export const type = {
	xs: ['0.75rem', '1.45'],
	sm: ['0.875rem', '1.5'],
	base: ['1rem', '1.6'],
	md: ['1.125rem', '1.55'],
	lg: ['1.375rem', '1.35'],
	xl: ['1.75rem', '1.2'],
	'2xl': ['2.5rem', '1.08'],
	'3xl': ['3.5rem', '1.02'],
};
export const weight = { regular: 400, medium: 500, bold: 650 };
export const font = {
	sans: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', 'Noto Sans', Arial, sans-serif",
	mono: "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace",
};
export const radius = { none: '0', sm: '4px', md: '8px', full: '999px' };
export const motion = { fast: '120ms', base: '200ms', ease: 'cubic-bezier(0.2, 0, 0, 1)' };
export const layer = { base: 0, sticky: 10, overlay: 20 };
// Media queries cannot read custom properties, so these are the only three
// widths a media query in site/src may use; scripts/lint-tokens.mjs holds it.
export const breakpoints = { sm: 560, md: 860, lg: 1180 };
export const measure = { prose: '64ch', page: '1120px' };

/* ---------- emitters ---------- */

const block = (sel, vars) => `${sel} {\n${Object.entries(vars).map(([k, v]) => `\t--${k}: ${v};`).join('\n')}\n}`;

/** Every token as CSS custom properties, light by default, dark by setting or choice. */
export function css() {
	const scales = {
		...Object.fromEntries(Object.entries(space).map(([k, v]) => [`space-${k}`, v])),
		...Object.fromEntries(Object.entries(type).flatMap(([k, [size, lh]]) => [[`text-${k}`, size], [`leading-${k}`, lh]])),
		...Object.fromEntries(Object.entries(weight).map(([k, v]) => [`weight-${k}`, v])),
		'font-sans': font.sans,
		'font-mono': font.mono,
		...Object.fromEntries(Object.entries(radius).map(([k, v]) => [`radius-${k}`, v])),
		'duration-fast': motion.fast,
		'duration-base': motion.base,
		ease: motion.ease,
		...Object.fromEntries(Object.entries(layer).map(([k, v]) => [`layer-${k}`, v])),
		'measure-prose': measure.prose,
		'measure-page': measure.page,
	};
	return [
		'/* Generated from src/design/tokens.mjs by scripts/sync.mjs. Edit the module, not this file. */',
		block(':root', { ...scales, ...themes.light }),
		`@media (prefers-color-scheme: dark) {\n${block(":root:not([data-theme='light'])", { 'color-scheme': 'dark', ...themes.dark }).replace(/^/gm, '\t')}\n}`,
		block(":root[data-theme='dark']", { 'color-scheme': 'dark', ...themes.dark }),
		'@media (prefers-reduced-motion: reduce) {\n\t:root {\n\t\t--duration-fast: 0ms;\n\t\t--duration-base: 0ms;\n\t}\n}',
		'',
	].join('\n\n');
}

/**
 * A syntax theme for the docs' code blocks, from the same tokens the site's own
 * highlighter uses: marks in their colours, everything else in the neutrals.
 * @param {'light' | 'dark'} name
 */
export function shikiTheme(name) {
	const t = themes[name];
	const fg = (scope, foreground, fontStyle) => ({ scope, settings: fontStyle ? { foreground, fontStyle } : { foreground } });
	return {
		name: `sprig-${name}`,
		type: name,
		colors: { 'editor.background': t['surface-raised'], 'editor.foreground': t.text },
		tokenColors: [
			...marks.map((m) => fg(`keyword.mark.${m.key}.sprig`, t[`mark-${m.key}`])),
			fg('markup.done.sprig', t['text-muted']),
			fg('markup.strikethrough.sprig', t['text-muted'], 'strikethrough'),
			fg(['variable.other.person.sprig', 'entity.name.tag.sprig', 'entity.name.label.anchor.sprig', 'support.type.property-name.sprig', 'punctuation.separator.key-value.sprig', 'string.unquoted.value.sprig'], t['text-muted']),
			fg('markup.underline.link.sprig', t.text, 'underline'),
			fg('keyword.other.priority.sprig', t.text, 'bold'),
			fg(['comment', 'constant.character.escape.sprig'], t['text-muted']),
			// Everything else a code block might hold (shell, JSON) stays calm too.
			fg(['keyword', 'storage', 'string', 'constant', 'entity', 'variable', 'support'], t.text),
		],
	};
}
