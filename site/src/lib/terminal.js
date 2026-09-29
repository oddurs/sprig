// What the planned `sprig` commands print, rendered by the reference parser so
// the tools page can't show output the format doesn't produce. Screens are
// lines of segments; only a mark's own character takes colour, which is the
// CLI's rule too: colour repeats what the mark says, so NO_COLOR loses nothing.
import * as sprig from './sprig.js';

// The example plans are written around this date, and every screen passes it
// as --today, so the page reads the same on any day it is opened.
export const PINNED = '2026-10-01';
const pinned = new Date(2026, 9, 1);

const CH = { todo: '-', doing: '~', done: 'x', dropped: '/', later: '>', ask: '?', group: '#', graft: '+' };
const S = (t, c = '') => ({ t: String(t), c });
const len = (segs) => segs.reduce((n, s) => n + [...s.t].length, 0);
const pad = (segs, w) => (len(segs) < w ? segs.concat(S(' '.repeat(w - len(segs)))) : segs);
const clip = (t, w) => ([...t].length > w ? [...t].slice(0, w - 1).join('') + '…' : t);
const prompt = (cmd) => [S('$ ', 't-dim'), S(cmd, 't-b')];
const html = (lines) =>
	lines.map((l) => l.map((s) => (s.c ? `<span class="${s.c}">${sprig.esc(s.t)}</span>` : sprig.esc(s.t))).join('')).join('\n');
const plain = (lines) => lines.map((l) => l.map((s) => s.t).join('')).join('\n');

/** The mark as the file writes it, coloured by what it means now. */
export function markOf(n) {
	if (n.state === 'group' || n.state === 'graft') return S(CH[n.state], `m-${n.state}`);
	if (n.answered) return S('?', 'm-answer');
	const st = n.inh && n.inh !== n.state ? n.inh : n.state;
	return S(CH[st] || '-', `m-${st}`);
}
const dayMonth = (d) => d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });
const shortDate = (v) => {
	const d = sprig.parseDate(v);
	return d ? dayMonth(d) : v;
};
const rel = (v) => {
	const d = sprig.parseDate(v);
	if (!d) return '';
	const n = Math.round((d - sprig.TODAY) / 864e5);
	return n < 0 ? `${-n}d late` : n === 0 ? 'today' : `in ${n}d`;
};
const bar = (done, total, w) => {
	const f = total ? Math.round((done / total) * w) : 0;
	return [S('█'.repeat(f), 'm-done'), S('░'.repeat(w - f), 't-dim')];
};

function load(files) {
	sprig.useWorkspace(files);
	sprig.beginRender();
	return (name) => sprig.buildDoc(name);
}

/** Open leaves, ready or waiting, ranked the way `sprig next` ranks them. */
export function readyList(doc) {
	const ready = [];
	const waiting = [];
	sprig.walkLeaves(doc.root, (n, path, blocked, due, prio) => {
		if (!sprig.OPEN.has(n.view)) return;
		const d = sprig.parseDate(due);
		(blocked ? waiting : ready).push({ n, path, due, prio, t: d ? d.getTime() : Infinity });
	});
	ready.sort((a, b) => (b.n.view === 'doing') - (a.n.view === 'doing') || b.prio - a.prio || a.t - b.t);
	return { ready, waiting };
}

function next(files, { limit = 6, compact = false } = {}) {
	const doc = load(files)('bakery.sprig');
	const { ready, waiting } = readyList(doc);
	const asks = ready.filter((r) => r.n.view === 'ask').length;
	const lines = [prompt(`sprig next --limit ${limit} --today ${PINNED}`)];
	for (const r of ready.slice(0, limit)) {
		const where = r.path.at(-1) ?? '';
		const text = S(clip(r.n.meta.text, 30), r.n.view === 'doing' ? 't-b' : '');
		const due = r.due ? [S(`${shortDate(r.due)} · ${rel(r.due)}`, 't-dim')] : [];
		const prio = r.prio ? [S('  ' + '!'.repeat(r.prio), 't-b')] : [];
		const place = compact ? [] : pad([S(clip(where, 18), 't-dim')], 20);
		lines.push(pad([S('  '), markOf(r.n), S(' '), text], 36).concat(place, due, prio));
	}
	lines.push([]);
	lines.push([S(`${Math.min(limit, ready.length)} of ${ready.length} ready · ${waiting.length} waiting · ${asks} questions`, 't-dim')]);
	return lines;
}

function treeLines(doc, depth) {
	const s = doc.root.stats;
	const lines = [pad([S(doc.title, 't-b')], 44).concat([S(`${s.done}/${s.total}  `, 't-dim')], bar(s.done, s.total, 10))];
	(function walk(n, prefix, d) {
		n.children.forEach((c, i) => {
			const last = i === n.children.length - 1;
			const own = sprig.blockers(c);
			const settled = c.view === 'done' || c.view === 'dropped' || c.view === 'later';
			const segs = [S(prefix + (last ? '└─ ' : '├─ '), 't-dim'), markOf(c), S(' ')];
			segs.push(S(sprig.labelOf(c), c.children.length ? 't-b' : settled ? 't-dim' : ''));
			if (c.state === 'graft') segs.push(S(`  ${c.target}`, 't-dim t-u'));
			if (c.meta.prio) segs.push(S('  ' + '!'.repeat(c.meta.prio), 't-b'));
			if (c.meta.fields.due && !settled) segs.push(S(`  ${shortDate(c.meta.fields.due)}`, 't-dim'));
			if (own) segs.push(S(`  waits on ${own.map((b) => b.label).join(', ')}`, 't-dim'));
			if (c.children.length && c.stats.total) segs.push(S(`  ${c.stats.done}/${c.stats.total}`, 't-dim'));
			lines.push(segs);
			if (c.children.length && d < depth) walk(c, prefix + (last ? '   ' : '│  '), d + 1);
		});
	})(doc.root, '', 1);
	return lines;
}

function tree(files, file, { depth = 3, env = '' } = {}) {
	return [prompt(`${env}sprig tree ${file}`)].concat(treeLines(load(files)(file), depth));
}

function why(files, file, text) {
	const doc = load(files)(file);
	let target = null;
	let parent = null;
	(function w(n, up) {
		for (const c of n.children) {
			if (!target && c.meta.text === text) [target, parent] = [c, up];
			w(c, c);
		}
	})(doc.root, null);
	const loc = (n) => `${n.file}:${n.line + 1}`;
	const lines = [prompt(`sprig why ${JSON.stringify(text).replace(/^"(\w+)"$/, '$1')}`)];
	lines.push(pad([markOf(target), S(' '), S(target.meta.text, 't-b')], 28).concat([S(loc(target), 't-dim')]));
	// The item may wait on its own after: list, or inherit waiting from a parent.
	const from = target.meta.after.length ? target : parent;
	if (from !== target) lines.push([S('└─ ', 't-dim'), S('inside ', 't-dim'), S(from.meta.text, 't-b'), S(', which waits on:', 't-dim')]);
	const seen = new Set();
	(function chain(n, prefix) {
		const refs = n.meta.after.map((r) => sprig.resolveRef(r, n.file).node).filter((m) => m && m.view !== 'done');
		refs.forEach((m, i) => {
			const last = i === refs.length - 1;
			const who = m.who?.length ? `  @${m.who[0]}` : '';
			lines.push(pad([S(prefix + (last ? '└─ ' : '├─ '), 't-dim'), markOf(m), S(' '), S(clip(sprig.labelOf(m), 22))], 34).concat([S(loc(m), 't-dim'), S(who, 't-dim')]));
			if (seen.has(m)) return;
			seen.add(m);
			const inner = prefix + (last ? '   ' : '│  ');
			if (m.meta.after.length) chain(m, inner);
			else {
				const state = m.children.length ? `${m.stats.done} of ${m.stats.total} done, nothing blocks it` : m.view === 'doing' ? 'in progress, nothing blocks it' : 'ready to start';
				lines.push([S(inner + '└─ ', 't-dim'), S(state)]);
			}
		});
	})(from, from === target ? '' : '   ');
	return lines;
}

/** `sprig check` over a copy of the files with the given lines broken. */
function check(files) {
	const get = load(files);
	const found = [];
	for (const f of Object.keys(files)) {
		const doc = get(f);
		const src = files[f].split('\n');
		(function w(n) {
			for (const c of n.children) {
				if (c.cloned) continue;
				if (c.error) found.push({ f, line: c.line + 1, col: src[c.line].indexOf('[[') + 1, msg: c.error.replace(/ yet$/, '').replace(/^N/, 'n') });
				for (const b of sprig.blockers(c)?.filter((x) => x.unknown) ?? []) {
					found.push({ f, line: c.line + 1, col: src[c.line].indexOf(b.label) + 1, msg: `no item ${b.label} in ${f}` });
				}
				w(c);
			}
		})(doc.root);
	}
	const lines = [prompt('sprig check')];
	for (const o of found) lines.push([S(`${o.f}:${o.line}:${o.col}: `, 't-dim'), S('error', 't-b'), S(`: ${o.msg}`)]);
	lines.push([]);
	lines.push([S(`${found.length} errors in ${new Set(found.map((o) => o.f)).size} files · exit 1`, 't-dim')]);
	return lines;
}

function status(files) {
	const get = load(files);
	const lines = [prompt(`sprig status --today ${PINNED}`)];
	const names = Object.keys(files).filter((f) => f !== 'dana.sprig');
	for (const f of names) {
		const d = get(f);
		const s = d.root.stats;
		let waiting = 0;
		let late = 0;
		sprig.walkLeaves(d.root, (n, _p, b, due) => {
			if (!sprig.OPEN.has(n.view)) return;
			if (b) waiting++;
			if (due && rel(due).endsWith('late')) late++;
		});
		lines.push(pad([S(f.replace('.sprig', ''))], 11).concat(bar(s.done, s.total, 8), [S(' ')], pad([S(`${s.done}/${s.total}`)], 7), pad([S(waiting ? `${waiting} waiting` : '', 't-dim')], 12), [S(late ? `${late} late` : '', 't-b')]));
	}
	const { ready, waiting } = readyList(get('bakery.sprig'));
	const b = get('bakery.sprig').root.stats;
	lines.push([]);
	lines.push(prompt('sprig status --oneline'));
	lines.push([S(`sprig ${Math.round((b.done / b.total) * 100)}% · ${ready.length} ready · ${waiting.length} waiting`)]);
	return lines;
}

const screens = { next, tree, why, check, status };

/**
 * Render one screen as HTML and as plain text, with the date pinned.
 * @param {keyof typeof screens} name
 * @param {Record<string, string>} files
 * @param {...any} args
 */
export function screen(name, files, ...args) {
	const lines = sprig.withToday(pinned, () => screens[name](files, ...args));
	return { html: html(lines), text: plain(lines) };
}

export { pinned, rel, shortDate, dayMonth };
