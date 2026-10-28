// How the site shows a plan: the tree, Next up, chips, notes and the editor's
// colours, as HTML. What a plan *means* comes from sprig-core (./core.js); this
// module only renders it, so the site and the `sprig` command cannot disagree.
//
// The interface is the one the pages were written against: a workspace is
// set, a document built, and nodes rendered. A render is: useWorkspace(files),
// beginRender(), build and render, endRender(). Call `await ready` once first.
import { call, ready } from './core.js';

export { ready };

export let TODAY = (() => {
	const d = new Date();
	d.setHours(0, 0, 0, 0);
	return d;
})();

export const addDays = (d, n) => new Date(d.getFullYear(), d.getMonth(), d.getDate() + n);
export const iso = (d) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
const fromIso = (s) => {
	if (!s) return null;
	const [y, m, d] = s.split('-').map(Number);
	return new Date(y, m - 1, d);
};

export const OPEN = new Set(['todo', 'doing', 'ask']);
const SETTLING = new Set(['done', 'dropped', 'later']);
const DATE_KEYS = new Set(['due', 'start']);

export const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
export const fmtEst = (h) => (h < 8 ? `${+h.toFixed(1)}h` : `${+(h / 8).toFixed(1)}d`);

/* ---------- asking the core ---------- */

let files = {};
let CACHE = null;
const memo = new Map();

const ask = (key, request) => {
	const k = `${iso(TODAY)}\u0000${key}`;
	if (!memo.has(k)) {
		if (memo.size > 5000) memo.clear();
		memo.set(k, call(request));
	}
	return memo.get(k);
};

/** A date field's value as a Date: the stored form or a typed shortcut (§9.3). */
export function parseDate(v) {
	if (!v) return null;
	return fromIso(ask(`date ${v}`, { op: 'date', value: String(v), today: iso(TODAY) }));
}

/** Typed shortcuts in one line written out in the stored form (§9.4). */
export function expandDates(line) {
	return call({ op: 'expand', line, today: iso(TODAY) });
}

/**
 * One line ticked, as `sprig tick` would: `{line, movedTo}` or null when the
 * line is not an item with a status.
 */
export function tickLine(line) {
	return call({ op: 'tick', line, today: iso(TODAY) });
}

/** Each line's kind, as the parser reads it: title, note, todo, group, answer... */
export function lineKinds(src) {
	return ask(`lex ${src}`, { op: 'lex', src }).map((l) => l.kind);
}

/** How one body line reads on its own: its kind and where its mark is. */
export function classify(line) {
	const [, l] = ask(`lex _\n${line}`, { op: 'lex', src: `_\n${line}` });
	return { kind: l.kind, marker: l.marker };
}

const MARK_KINDS = new Set(['todo', 'doing', 'done', 'dropped', 'later', 'ask', 'group', 'graft', 'answer']);
/** True for a line that starts with a mark or `=`, which is where fields live. */
export const isItemKind = (k) => MARK_KINDS.has(k);

const inline = (text, note) => ask(`inline ${note} ${text}`, { op: 'inline', text, note });

/** Note text with link brackets dropped, as an item's plain text drops them (§4.9). */
export function unbracket(text) {
	let out = '';
	let at = 0;
	for (const p of inline(text, true).filter((x) => x.kind === 'link')) {
		out += text.slice(at, p.at) + text.slice(p.at + 2, p.end - 2);
		at = p.end;
	}
	return out + text.slice(at);
}

/* ---------- documents ---------- */

// Links each node to its document, numbers them, and turns blocker dates into
// what the chips say.
function adopt(out) {
	for (const doc of Object.values(out.docs)) {
		doc.nodes = [];
		doc.anchors = {};
		(function walk(n) {
			doc.nodes[n.id] = n;
			if (n.blk) n.blk = n.blk.map((b) => (b.date ? { label: fmtDate(fromIso(b.date)), date: true } : b));
			for (const c of n.children) {
				if (!c.cloned && c.meta.id && !(c.meta.id in doc.anchors)) doc.anchors[c.meta.id] = c;
				walk(c);
			}
		})(doc.root);
		doc.root.doc = doc;
		for (const l of doc.leaves) {
			l.n = doc.nodes[l.id];
			if (l.blk) l.blk = l.blk.map((b) => (b.date ? { label: fmtDate(fromIso(b.date)), date: true } : b));
		}
	}
	return out;
}

function workspace() {
	if (!CACHE) CACHE = adopt(call({ op: 'workspace', files, today: iso(TODAY) }));
	return CACHE;
}

export function useWorkspace(next) {
	files = next;
	CACHE = null;
}

/** A file's tree, or null when there is no such file. */
export function buildDoc(name) {
	return workspace().docs[name] ?? null;
}

/** Every problem in the workspace, as `sprig check` reports it. */
export function diagnostics() {
	return workspace().diagnostics;
}

/** What an item itself waits on, or null. */
export const blockers = (n) => n.blk;

/** The node a blocker names, in the file that writes it. */
export function targetOf(b) {
	if (!b.file) return null;
	const doc = buildDoc(b.file);
	return doc?.nodes.find((n) => !n.cloned && n.line === b.line && n.file === b.file) ?? null;
}

export const labelOf = (n) => (n.state === 'graft' ? n.graftTitle || n.target || 'graft' : n.meta.text || 'untitled');

/**
 * Every leaf of a document in order, with what holds it, its due date and its
 * priority, as the core computed them.
 * @param {any} root a document's root
 * @param {(n: any, path: string[], blocked: any, due: string | null, prio: number) => void} fn
 */
export function walkLeaves(root, fn) {
	for (const l of root.doc.leaves) fn(l.n, l.path, l.blk, l.due, l.prio);
}

/** Open leaves, ready and waiting, in the core's order: the same as `sprig next`. */
export function readyList(doc) {
	const pick = (ids) => ids.map((i) => doc.leaves[i]).map((l) => ({ n: l.n, path: l.path, b: l.blk, due: l.due, prio: l.prio }));
	return { ready: pick(doc.ready), waiting: pick(doc.waiting) };
}

/* ---------- highlighting ---------- */

const PIECE = { person: 'k-who', tag: 'k-tag', priority: 'k-prio', anchor: 'k-id', key: 'k-key', value: 'k-val', link: 'k-link' };

// Text with its pieces coloured; offsets are into `text`.
function painted(text, pieces, from = 0) {
	let out = '';
	let at = from;
	for (const [kind, s, e] of pieces) {
		if (s < at) continue;
		out += esc(text.slice(at, s)) + `<span class="${PIECE[kind]}">${esc(text.slice(s, e))}</span>`;
		at = e;
	}
	return out + esc(text.slice(at));
}

export function highlight(src) {
	const lexed = ask(`lex ${src}`, { op: 'lex', src });
	return src
		.split('\n')
		.map((ln, i) => {
			const l = lexed[i];
			if (!l || l.kind === 'blank') return esc(ln);
			const ind = esc(ln.slice(0, l.at));
			const body = (from) => painted(ln, l.pieces, from);
			const [ms, me] = l.marker ?? [l.at, l.at];
			switch (l.kind) {
				case 'title':
					return `<span class="k-title">${painted(ln, l.pieces, 0)}</span>`;
				case 'comment':
					return ind + `<span class="k-com">${esc(ln.slice(l.at))}</span>`;
				case 'escaped':
					return ind + `<span class="k-esc">\\</span><span class="k-note">${body(me)}</span>`;
				case 'note':
					return ind + `<span class="k-note">${body(l.at)}</span>`;
				case 'group':
					return ind + `<span class="k-gm">#</span><span class="k-group">${body(me)}</span>`;
				case 'answer':
					return ind + `<span class="k-ans">=</span><span class="k-anst">${body(me)}</span>`;
				default:
					return ind + `<span class="k-${l.kind}">${esc(ln.slice(ms, me))}</span><span class="kl-${l.kind}">${body(me)}</span>`;
			}
		})
		.join('\n');
}

/* ---------- rendering ---------- */

let REF = [];
let FP = null;
let HIDE = false;
let VM = 'tree';
const COLL = new Set();

function relDays(d) {
	if (d === 0) return 'today';
	if (d === 1) return 'tomorrow';
	if (d < 0) return `${-d}d ago`;
	if (d < 14) return `in ${d}d`;
	if (d < 60) return `in ${Math.round(d / 7)}w`;
	return `in ${Math.round(d / 30)}mo`;
}
function fmtDate(d) {
	const s = d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
	return d.getFullYear() !== TODAY.getFullYear() ? `${s}, ${d.getFullYear()}` : s;
}
const daysFrom = (d) => Math.round((d - TODAY) / 864e5);
function dateChip(k, v, view) {
	const d = parseDate(v);
	const pre = k === 'due' ? '' : k + ' ';
	if (!d) return `<span class="chip">${pre}${esc(v)}</span>`;
	const diff = daysFrom(d);
	let cls = 'chip date';
	let rel = '';
	if (!SETTLING.has(view)) {
		if (diff < 0 && k !== 'start') {
			cls += ' late';
			rel = `${-diff}d late`;
		} else rel = relDays(diff);
	}
	return `<span class="${cls}">${pre}${fmtDate(d)}${rel ? ' · ' + rel : ''}</span>`;
}

// A link piece as an anchor to the file it names.
const linkHTML = (inner, p) =>
	p.path ? `<a href="#" class="flink" data-go="${esc(p.path)}"${p.anchor ? ` data-id="${esc(p.anchor)}"` : ''}>${esc(inner)}</a>` : esc(inner);

// A word of an item's text, with any link in it shown without its brackets.
function wordHTML(w) {
	let out = '';
	let at = 0;
	for (const p of inline(w, false).filter((x) => x.kind === 'link')) {
		out += esc(w.slice(at, p.at)) + linkHTML(w.slice(p.at + 2, p.end - 2), p);
		at = p.end;
	}
	return out + esc(w.slice(at));
}
function inlineHTML(n) {
	if (n.state === 'graft') {
		const where = n.target ? n.target + (n.targetId ? '^' + n.targetId : '') : '';
		return (
			`<span class="gt">${esc(n.graftTitle || n.target || 'Graft')}</span>` +
			(where ? ` <a href="#" class="fchip" data-go="${esc(n.target)}"${n.targetId ? ` data-id="${esc(n.targetId)}"` : ''}>${esc(where)}</a>` : '')
		);
	}
	return n.meta.words.map(wordHTML).join(' ') || '<span class="untitled">untitled</span>';
}
// Prose markup in notes: code, bold, emphasis and bare URLs. People and links
// are the core's pieces, as they are in the editor.
function prose(s) {
	let h = esc(s);
	h = h.replace(/`([^`]+)`/g, '<code>$1</code>');
	h = h.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
	h = h.replace(/(^|[\s(])\*([^*\s][^*]*)\*/g, '$1<em>$2</em>');
	h = h.replace(/(^|\s)(https?:\/\/[^\s<]+)/g, '$1<a href="$2" target="_blank" rel="noopener">$2</a>');
	return h;
}
function mdLite(s) {
	let out = '';
	let at = 0;
	for (const p of inline(s, true)) {
		out += prose(s.slice(at, p.at));
		const t = s.slice(p.at, p.end);
		if (p.kind === 'person') {
			const name = t.replace(/[.,;:!?)]+$/, '');
			out += `<span class="nwho">${esc(name)}</span>${esc(t.slice(name.length))}`;
		} else out += linkHTML(t.slice(2, -2), p);
		at = p.end;
	}
	return out + prose(s.slice(at));
}
export function notesHTML(notes) {
	const paras = [];
	let cur = [];
	for (const l of notes) {
		if (l === '') {
			if (cur.length) paras.push(cur);
			cur = [];
		} else cur.push(l);
	}
	if (cur.length) paras.push(cur);
	return paras.map((p) => `<p>${mdLite(p.join(' '))}</p>`).join('');
}
function answersHTML(list) {
	return list
		.map((a) => {
			const chips =
				a.who.map((w) => `<span class="chip who">@${esc(w)}</span>`).join('') +
				Object.entries(a.fields)
					.map(([k, v]) => {
						const d = parseDate(v);
						return `<span class="chip kv">${esc(k)} ${d ? fmtDate(d) : esc(v)}</span>`;
					})
					.join('');
			return `<p class="ans"><span class="eq">=</span><span>${a.words.map(wordHTML).join(' ')}</span>${chips}</p>`;
		})
		.join('');
}

function chipsHTML(n, blk) {
	const f = n.meta.fields;
	const c = [];
	if (n.meta.prio) c.push(`<span class="chip prio" title="Priority ${n.meta.prio}">${'!'.repeat(n.meta.prio)}</span>`);
	for (const w of n.meta.who) c.push(`<span class="chip who">@${esc(w)}</span>`);
	if (n.state === 'graft' && !n.meta.who.length && n.graftOwner) c.push(`<span class="chip who soft">@${esc(n.graftOwner)}</span>`);
	// A start still ahead is shown once, as what the item waits for.
	const waiting = blk && blk.some((b) => b.date);
	for (const k of ['start', 'due']) if (f[k] && !(k === 'start' && waiting)) c.push(dateChip(k, f[k], n.view));
	if (f.est && !n.children.length) c.push(`<span class="chip">${esc(f.est)}</span>`);
	if (f.every) c.push(`<span class="chip">every ${esc(f.every)}</span>`);
	for (const t of n.meta.tags) c.push(`<span class="chip tag">#${esc(t)}</span>`);
	for (const [k, v] of Object.entries(f)) {
		if (!['start', 'due', 'est', 'every'].includes(k)) c.push(`<span class="chip kv">${esc(k)} ${esc(v)}</span>`);
	}
	if (blk)
		for (const b of blk) {
			c.push(
				b.unknown
					? `<span class="chip err">can't find ${esc(b.label)}</span>`
					: `<span class="chip block">waits ${b.date ? 'until' : 'on'} ${esc(b.label)}</span>`,
			);
		}
	if (n.readyToClose) c.push('<span class="chip ready">all done · tick it</span>');
	if (n.meta.id) c.push(`<span class="chip id">^${esc(n.meta.id)}</span>`);
	if (n.error) c.push(`<span class="chip err">${esc(n.error)}</span>`);
	return c.join('');
}

function progHTML(n) {
	const s = n.stats;
	if (!n.children.length || !s.total) return '<span></span>';
	const pct = (s.done / s.total) * 100;
	const dp = (s.doing / s.total) * 100;
	return `<span class="prog" title="${s.done} of ${s.total} done"><span class="bar"><i style="width:${pct}%"></i><b style="width:${dp}%"></b></span>${s.done}/${s.total}${s.est ? ' · ' + fmtEst(s.est) : ''}</span>`;
}

function visible(n) {
	if (HIDE && (n.view === 'done' || n.view === 'dropped')) return false;
	if (!FP) return true;
	if (!n.children.length) return n.who.includes(FP);
	return n.meta.who.includes(FP) || n.children.some(visible);
}

function markHTML(n, k) {
	if (n.state === 'group' || n.state === 'graft') return `<span class="mark m-${n.state} v-${n.view}" aria-hidden="true"></span>`;
	const inherited = n.inh && n.inh !== n.state && !n.answered;
	const disp = n.answered ? 'answered' : inherited ? n.inh : n.state;
	const label = n.state === 'done' ? 'Reopen' : n.meta.fields.every ? 'Done for now, move to next date' : 'Mark done';
	return `<button type="button" class="mark m-${disp}${inherited ? ' inherited' : ''}" data-act="toggle" data-k="${k}" aria-label="${label}: ${esc(labelOf(n))}" title="${label}"></button>`;
}

export function nodeHTML(n, inhBlk) {
	if (!visible(n)) return '';
	const k = REF.push(n) - 1;
	const own = blockers(n);
	const blocked = OPEN.has(n.view) && !!(own || inhBlk);
	const kids = n.children.length > 0;
	const ck = `${n.file}:${n.line}`;
	const open = !COLL.has(ck);
	const caret = kids
		? `<button type="button" class="caret" data-act="fold" data-ck="${esc(ck)}" aria-expanded="${open}" aria-label="${open ? 'Collapse' : 'Expand'}"></button>`
		: '<span></span>';
	let extra = n.notes.length ? `<div class="notes">${notesHTML(n.notes)}</div>` : '';
	if (open && n.graftNotes && n.graftNotes.length) extra += `<div class="notes">${notesHTML(n.graftNotes)}</div>`;
	const answers = n.answers.concat(n.graftAnswers || []);
	if (answers.length) extra += `<div class="answers">${answersHTML(answers)}</div>`;
	const inner = kids && open ? n.children.map((c) => nodeHTML(c, blocked ? own || inhBlk : null)).join('') : '';
	return (
		`<li class="node v-${n.view} s-${n.state}${blocked ? ' is-blocked' : ''}${kids ? ' has-kids' : ''}" data-src="${esc(ck)}">` +
		`<div class="row">${caret}${markHTML(n, k)}<div class="body"><span class="txt" data-act="src" data-k="${k}">${inlineHTML(n)}</span>${chipsHTML(n, own)}</div>${progHTML(n)}</div>` +
		extra +
		(inner ? `<ul>${inner}</ul>` : '') +
		'</li>'
	);
}

export function nextHTML(doc) {
	const { ready: r, waiting: w } = readyList(doc);
	const mine = (x) => !FP || x.n.who.includes(FP);
	const ready = r.filter(mine);
	const waiting = w.filter(mine);
	const li = (x) => {
		const k = REF.push(x.n) - 1;
		const path = x.path.length ? x.path.map(esc).join(' <span>›</span> ') : esc(doc.title || doc.file);
		return (
			`<li class="v-${x.n.view}${x.b ? ' is-blocked' : ''}">${markHTML(x.n, k)}<div><div class="path">${path}</div>` +
			`<div class="line"><span class="txt" data-act="src" data-k="${k}">${inlineHTML(x.n)}</span>${chipsHTML(x.n, x.b)}</div></div></li>`
		);
	};
	return (
		`<div class="nx-h">Ready now · ${ready.length}</div>` +
		(ready.length ? `<ul class="nx">${ready.map(li).join('')}</ul>` : '<p class="empty">Nothing is ready. Everything open is waiting on something.</p>') +
		`<div class="nx-h">Waiting · ${waiting.length}</div>` +
		(waiting.length ? `<ul class="nx">${waiting.map(li).join('')}</ul>` : '<p class="empty">Nothing is blocked.</p>')
	);
}

export function propsHTML(doc) {
	const m = doc.root.meta;
	const out = [];
	if (m.who.length) out.push(['owner', m.who.map((w) => `<span class="chip who">@${esc(w)}</span>`).join(' ')]);
	for (const [k, v] of Object.entries(m.fields)) {
		const d = DATE_KEYS.has(k) ? parseDate(v) : null;
		out.push([k, d ? `${fmtDate(d)} <span class="rel">${relDays(daysFrom(d))}</span>` : esc(v)]);
	}
	for (const t of m.tags) out.push(['tag', '#' + esc(t)]);
	return out.map(([k, v]) => `<div><dt>${esc(k)}</dt><dd>${v}</dd></div>`).join('');
}

/* ---------- render state ---------- */

export function setView({ fp = null, hide = false, vm = 'tree', collapsed = [] } = {}) {
	FP = fp;
	HIDE = hide;
	VM = vm;
	COLL.clear();
	for (const c of collapsed) COLL.add(c);
}
export function beginRender() {
	CACHE = null;
	REF = [];
}
export function endRender() {
	return REF.slice();
}

// The CLI's --today. Pages that render dated examples at build time pin the
// date, so what they print stays true after the build instead of drifting a
// day each morning.
export function withToday(day, fn) {
	const was = TODAY;
	TODAY = day;
	CACHE = null;
	try {
		return fn();
	} finally {
		TODAY = was;
		CACHE = null;
	}
}
