// Generate the site's repository-derived content before every build.
//
// The spec, the settled arguments, the roadmap, the changelog and the example
// plans all live elsewhere in the repository. Copying them here at build time,
// never by hand, is what keeps a page from quietly drifting from its source:
// change the source and the next build changes the page. Everything this
// writes is gitignored.
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { css } from '../src/design/tokens.mjs';

const site = join(dirname(fileURLToPath(import.meta.url)), '..');
const repo = join(site, '..');
const docs = join(site, 'src/content/docs');
const generated = join(site, 'src/generated');
const blob = 'https://github.com/oddurs/sprig/blob/main';
const edit = 'https://github.com/oddurs/sprig/edit/main';

const read = (path) => readFileSync(join(repo, path), 'utf8');
function write(path, text) {
	mkdirSync(dirname(path), { recursive: true });
	writeFileSync(path, text);
}
const yaml = (s) => JSON.stringify(s);
function page(meta, body) {
	const head = Object.entries(meta)
		.map(([k, v]) => `${k}: ${typeof v === 'string' ? yaml(v) : JSON.stringify(v)}`)
		.join('\n');
	return `---\n${head}\n---\n\n${body.trim()}\n`;
}
// Starlight takes the title from frontmatter, so the source's own H1 goes.
const dropTitle = (md) => md.replace(/^# .*\n+/, '');

/* spec: rendered at /spec/, and served raw at /spec.md byte for byte */
const spec = read('spec/sprig.md');
write(join(site, 'public/spec.md'), spec);
write(
	join(docs, 'spec.md'),
	page(
		{
			title: 'Specification',
			description: 'The Sprig format, rule by rule. Draft 0.3, dedicated to the public domain.',
			editUrl: `${edit}/spec/sprig.md`,
			tableOfContents: { minHeadingLevel: 2, maxHeadingLevel: 3 },
		},
		`:::note[Generated]\nThis page is built from [\`spec/sprig.md\`](${blob}/spec/sprig.md). The raw file is at [/spec.md](/spec.md).\n:::\n\n${dropTitle(spec)}`,
	),
);

/* decisions: every cairn item of type decision */
function item(text) {
	const [, front, body] = text.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
	const meta = Object.fromEntries(
		front.split('\n').map((line) => {
			const i = line.indexOf(':');
			return [line.slice(0, i).trim(), line.slice(i + 1).trim().replace(/^'(.*)'$/, '$1')];
		}),
	);
	const sections = {};
	for (const part of body.split(/^## /m).slice(1)) {
		const nl = part.indexOf('\n');
		sections[part.slice(0, nl).trim()] = part.slice(nl + 1).trim();
	}
	return { meta, sections };
}
const decisions = readdirSync(join(repo, 'cairn/items'))
	.filter((f) => f.endsWith('.md'))
	.sort()
	.map((f) => ({ file: f, ...item(read(`cairn/items/${f}`)) }))
	.filter((d) => d.meta.type === 'decision');
const decisionsBody = decisions
	.map(({ file, meta, sections }) => {
		const part = (label, key) => (sections[key] ? `**${label}.** ${sections[key]}\n\n` : '');
		return (
			`## ${meta.title}\n\n` +
			part('Context', 'Context') +
			part('Options', 'Options and tradeoffs') +
			part('Decision', 'Decision') +
			part('Revisit when', 'Revisit when') +
			`<small>Recorded as cairn item [${String(meta.id).padStart(4, '0')}](${blob}/cairn/items/${file}).</small>`
		);
	})
	.join('\n\n');
write(
	join(docs, 'decisions.md'),
	page(
		{
			title: 'Settled arguments',
			description: 'Every rule in Sprig was argued both ways. These are the rulings, with the case against each.',
			editUrl: false,
		},
		`Sprig is opinionated on purpose. Each rule below was argued both ways before it went in, and the arguments that lost are kept so nobody has to have them again. To reopen one, propose a change with the case for, the case against and a ruling.\n\n:::note[Generated]\nBuilt from the ${decisions.length} decision items in [\`cairn/items\`](${blob}/cairn/items).\n:::\n\n${decisionsBody}`,
	),
);

/* roadmap and changelog */
write(
	join(docs, 'roadmap.md'),
	page(
		{ title: 'Roadmap', description: 'From draft to 1.0, generated from the project’s cairn backlog.', editUrl: false },
		`:::note[Generated]\nBuilt from [\`ROADMAP.md\`](${blob}/ROADMAP.md), which cairn renders from the items in [\`cairn/items\`](${blob}/cairn/items).\n:::\n\n` +
			dropTitle(read('ROADMAP.md'))
				.replace(/<!--[\s\S]*?-->\n*/g, '')
				// Task-list checkboxes would render as unlabelled, disabled form controls.
				.replace(/^(\s*)- \[[ x]\] /gm, '$1- '),
	),
);
const changelog = read('CHANGELOG.md');
write(
	join(docs, 'changelog.md'),
	page(
		{ title: 'Changelog', description: 'What changed in each release.', editUrl: false },
		`Also available as an [Atom feed](/feed.xml).\n\n${dropTitle(changelog)}`,
	),
);
// Released sections only; "Unreleased" is not an entry in a feed.
const releases = [...changelog.matchAll(/^## \[(\d[^\]]*)\] - (\d{4}-\d{2}-\d{2})\n([\s\S]*?)(?=^## |\s*$(?![\s\S]))/gm)].map(
	([, version, date, notes]) => ({ version, date, notes: notes.trim() }),
);
write(join(generated, 'releases.json'), JSON.stringify(releases, null, '\t'));

/* design tokens, as CSS custom properties for every page and the docs */
write(join(generated, 'tokens.css'), css());

/* example plans for the playground */
const examples = Object.fromEntries(
	readdirSync(join(repo, 'examples/bakery'))
		.filter((f) => f.endsWith('.sprig'))
		.map((f) => [f, read(`examples/bakery/${f}`)]),
);
write(join(generated, 'examples.json'), JSON.stringify(examples, null, '\t'));

/* counts the pages state in prose, so a sentence like "all fifteen" can't go stale */
write(join(generated, 'facts.json'), JSON.stringify({ decisions: decisions.length }, null, '\t'));

console.log(
	`sync: spec, ${decisions.length} decisions, roadmap, changelog (${releases.length} releases), ${Object.keys(examples).length} examples`,
);
