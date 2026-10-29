// Generate the site's repository-derived content before every build.
//
// The spec, the settled arguments, the roadmap, the changelog and the example
// plans all live elsewhere in the repository. Copying them here at build time,
// never by hand, is what keeps a page from quietly drifting from its source:
// change the source and the next build changes the page. Everything this
// writes is gitignored.
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { css } from '../src/design/tokens.mjs';

const site = join(dirname(fileURLToPath(import.meta.url)), '..');
const repo = join(site, '..');
const docs = join(site, 'src/content/docs');
const generated = join(site, 'src/generated');
const blob = 'https://github.com/oddurs/sprig/blob/main';
const edit = 'https://github.com/oddurs/sprig/edit/main';

// The site runs sprig-core, which is built by cargo rather than by npm.
if (!existsSync(join(generated, 'sprig.wasm'))) {
	console.error('sync: src/generated/sprig.wasm is missing. Build it first, from the repository root:\n\n  scripts/task wasm\n');
	process.exit(1);
}

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

/* spec: the core and its annex rendered as one page at /spec/, so every rule's
   anchor is on it, and each served raw byte for byte, at /spec.md and
   /spec-annex.md */
const spec = read('spec/sprig.md');
const annex = read('spec/annex.md');
// The draft named in the spec's own title, so the description cannot lag it.
const draft = spec.match(/^# .*draft (\S+)/m)?.[1];
if (!draft) throw new Error('spec/sprig.md: no "draft N" in its title');
write(join(site, 'public/spec.md'), spec);
write(join(site, 'public/spec-annex.md'), annex);
// On one page, the two files' links to each other become links within it.
const onePage = (md) => md.replace(/\]\(annex\.md\)/g, '](#annex)').replace(/\]\(sprig\.md\)/g, '](#the-format-on-one-card)');
write(
	join(docs, 'spec.md'),
	page(
		{
			title: 'Specification',
			description: `The Sprig format, rule by rule. Draft ${draft}, dedicated to the public domain.`,
			editUrl: `${edit}/spec/sprig.md`,
			tableOfContents: { minHeadingLevel: 2, maxHeadingLevel: 3 },
		},
		`:::note[Generated]\nThis page is built from [\`spec/sprig.md\`](${blob}/spec/sprig.md) and its annex, [\`spec/annex.md\`](${blob}/spec/annex.md). The raw files are at [/spec.md](/spec.md) and [/spec-annex.md](/spec-annex.md).\n:::\n\n` +
			onePage(dropTitle(spec)) +
			'\n\n## Annex\n\n' +
			onePage(dropTitle(annex)),
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

/* the command reference: the built `sprig` binary's own help, one section per command */
const exe = process.env.SPRIG_BIN ?? join(repo, 'target/debug', process.platform === 'win32' ? 'sprig.exe' : 'sprig');
if (!existsSync(exe)) {
	console.error(`sync: no sprig binary at ${exe}. Build it first, from the repository root:\n\n  cargo build -p sprig-cli\n`);
	process.exit(1);
}
// Without SPRIG_TODAY, so the page does not depend on who built it.
const env = Object.fromEntries(Object.entries(process.env).filter(([k]) => k !== 'SPRIG_TODAY'));
const help = (...args) => execFileSync(exe, [...args, '--help'], { encoding: 'utf8', env }).trimEnd();
const top = help();
// The commands are whatever `sprig --help` lists, so a new one needs no change here.
const commands = [...(top.split(/^Commands:$/m)[1] ?? '').split(/^\S/m)[0].matchAll(/^ {2}([a-z][\w-]*) {2,}(.*)$/gm)].map((m) => ({ name: m[1], about: m[2] }));
if (!commands.length) {
	console.error('sync: `sprig --help` lists no commands');
	process.exit(1);
}
const fence = (text) => '```text\n' + text + '\n```';
write(
	join(docs, 'docs/reference/cli.md'),
	page(
		{
			title: 'Command line',
			description: 'Every sprig command and option, from the binary\'s own help.',
			editUrl: false,
		},
		`:::note[Generated]\nThis page is the output of \`sprig --help\` and of each command's help, from the build of \`crates/sprig-cli\` this site was built with. It cannot describe a flag the command does not have.\n:::\n\n` +
			`## sprig\n\n${fence(top)}\n\n` +
			commands.map((c) => `## sprig ${c.name}\n\n${c.about}\n\n${fence(help(c.name))}`).join('\n\n'),
	),
);

/* the MCP reference: the tools as the built server lists them to a client */
const session = [
	{ jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: '2025-11-25', capabilities: {}, clientInfo: { name: 'site', version: '0' } } },
	{ jsonrpc: '2.0', id: 2, method: 'tools/list', params: {} },
]
	.map((m) => JSON.stringify(m))
	.join('\n');
const replies = execFileSync(exe, ['mcp', '--in', join(repo, 'examples/bakery')], { input: session + '\n', encoding: 'utf8', env })
	.trim()
	.split('\n')
	.map((l) => JSON.parse(l));
const tools = replies.find((r) => r.id === 2)?.result?.tools ?? [];
if (!tools.length) {
	console.error('sync: `sprig mcp` lists no tools');
	process.exit(1);
}
const cell = (s) => String(s).replaceAll('|', '\\|').replaceAll('\n', ' ');
const argRows = (schema) => {
	const props = Object.entries(schema.properties ?? {});
	if (!props.length) return 'No arguments.';
	const required = new Set(schema.required ?? []);
	return (
		'| Argument | Type | | |\n| --- | --- | --- | --- |\n' +
		props
			.map(([k, v]) => {
				const type = v.type === 'array' ? `list of ${v.items?.type ?? 'values'}` : v.enum ? v.enum.map((e) => `\`${e}\``).join(' ') : v.type;
				return `| \`${k}\` | ${cell(type)} | ${required.has(k) ? 'required' : 'optional'} | ${cell(v.description ?? '')} |`;
			})
			.join('\n')
	);
};
write(
	join(docs, 'docs/reference/mcp.md'),
	page(
		{
			title: 'MCP tools',
			description: 'Every tool sprig mcp offers an agent, with its arguments, from the server itself.',
			editUrl: false,
		},
		`:::note[Generated]\nThis page is what \`sprig mcp\` answers to \`tools/list\`, from the build of \`crates/sprig-cli\` this site was built with. \`spec/tools.md\` T6 is the contract, and the [agents guide](/docs/agents/) shows a session.\n:::\n\n` +
			tools
				.map((t) => {
					const kind = t.annotations?.readOnlyHint ? 'Reads.' : 'Writes, through the edit API, and returns the diff.';
					return `## ${t.name}\n\n**${t.title}.** ${t.description} ${kind}\n\n${argRows(t.inputSchema)}`;
				})
				.join('\n\n'),
	),
);

/* the diagnostics reference: the catalogue the core is tested against */
write(
	join(docs, 'docs/reference/diagnostics.md'),
	page(
		{
			title: 'Diagnostics',
			description: 'Every code a Sprig tool reports, its severity, the rule it enforces and what it means.',
			editUrl: `${edit}/spec/diagnostics.md`,
		},
		`:::note[Generated]\nThis page is \`spec/diagnostics.md\`. A test holds it and the core to each other: every code the core can report has a row and a note here.\n:::\n\n` +
			// Each note gets an anchor per code it explains, so a tool (the
			// playground's problems) can link a code straight to its note.
			dropTitle(read('spec/diagnostics.md')).replace(/^\*\*(S\d{4}(?:, S\d{4})*)\.\*\*/gm, (all, codes) =>
				codes
					.split(', ')
					.map((c) => `<span id="${c.toLowerCase()}"></span>`)
					.join('') + all,
			),
	),
);

/* counts the pages state in prose, so a sentence like "all fifteen" can't go stale */
write(join(generated, 'facts.json'), JSON.stringify({ decisions: decisions.length, commands: commands.map((c) => c.name), tools: tools.map((t) => t.name) }, null, '\t'));

console.log(
	`sync: spec, ${decisions.length} decisions, roadmap, changelog (${releases.length} releases), ${Object.keys(examples).length} examples, ${commands.length} commands, ${tools.length} MCP tools`,
);
