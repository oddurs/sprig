// A working model of `sprig tui` (cairn 0131 to 0134) for the tools page. The
// state is plain data and the frame is a pure function of state and files, so
// the page can render the first frame at build time and the browser only takes
// over once someone presses a key.
import * as sprig from './sprig.js';
import { dayMonth, markOf, pinned, readyList, rel, shortDate, waitsText } from './terminal.js';

const ROWS = 17;
const ROOT = 'bakery.sprig';
const esc = sprig.esc;
const keyOf = (n) => `${n.file}:${n.line}`;
const span = (s) => `<span class="${s.c}">${esc(s.t)}</span>`;

export const initialState = () => ({ view: 'tree', cursor: 0, top: 0, collapsed: [], filter: '', typing: false, flash: '', help: false });

function docs(files) {
	sprig.useWorkspace(files);
	sprig.beginRender();
	return Object.fromEntries(Object.keys(files).map((f) => [f, sprig.buildDoc(f)]));
}

function dated(all, files, match) {
	const out = [];
	for (const f of Object.keys(files).filter((x) => x !== 'dana.sprig')) {
		(function w(n) {
			for (const c of n.children) {
				if (c.cloned) continue;
				const d = sprig.parseDate(c.meta.fields.due);
				if (d && sprig.OPEN.has(c.view) && match(c)) out.push({ n: c, d, sub: f.replace('.sprig', ''), depth: 0 });
				w(c);
			}
		})(all[f].root);
	}
	return out.sort((a, b) => a.d - b.d);
}

function rows(state, all, files) {
	const q = state.filter.toLowerCase();
	const match = (n) => !q || sprig.labelOf(n).toLowerCase().includes(q);
	const doc = all[ROOT];
	if (state.view === 'next') return readyList(doc).ready.filter((r) => match(r.n)).map((r) => ({ n: r.n, depth: 0, sub: r.path.at(-1) ?? '' }));
	if (state.view === 'agenda') return dated(all, files, match);
	// A filtered tree keeps the ancestors of every match, so matches stay in context.
	return (function w(n, depth) {
		const acc = [];
		for (const c of n.children) {
			const open = c.children.length && (q || !state.collapsed.includes(keyOf(c)));
			const sub = open ? w(c, depth + 1) : [];
			if (match(c) || sub.length) acc.push({ n: c, depth }, ...sub);
		}
		return acc;
	})(doc.root, 0);
}

function detail(n) {
	if (!n) return '<p class="kv">Nothing matches.</p>';
	const f = n.meta.fields;
	const own = sprig.blockers(n);
	const kv = (k, v) => (v ? `<p class="kv">${k} <b>${esc(v)}</b></p>` : '');
	const notes = n.notes.filter(Boolean).join(' ');
	const answers = n.answers.concat(n.graftAnswers ?? []).map((a) => a.words.join(' '));
	return (
		`<p class="title">${span(markOf(n))} ${esc(sprig.labelOf(n))}</p>` +
		kv('at', `${n.file}:${n.line + 1}`) +
		kv('status', n.answered ? 'answered' : n.view) +
		kv('owner', n.who?.length ? '@' + n.who.join(' @') : '') +
		kv('due', f.due ? `${shortDate(f.due)} · ${rel(f.due)}` : '') +
		kv('estimate', f.est) +
		kv('every', f.every) +
		kv('progress', n.children.length && n.stats.total ? `${n.stats.done} of ${n.stats.total} done` : '') +
		(own ? `<p class="kv">${esc(waitsText(own))}</p>` : '') +
		kv('answer', answers[0]) +
		(notes ? `<p class="note">${esc(notes.replace(/\[\[([^\]]+)\]\]/g, '$1'))}</p>` : '')
	);
}

/** One frame, as HTML. */
export function frame(state, files) {
	return sprig.withToday(pinned, () => {
		const all = docs(files);
		const list = rows(state, all, files);
		state.cursor = Math.max(0, Math.min(state.cursor, list.length - 1));
		if (state.cursor < state.top) state.top = state.cursor;
		if (state.cursor >= state.top + ROWS) state.top = state.cursor - ROWS + 1;
		state.top = Math.max(0, Math.min(state.top, list.length - ROWS));
		const sel = list[state.cursor];
		const s = all[ROOT].root.stats;
		const { ready, waiting } = readyList(all[ROOT]);
		let lastDay = '';
		const left = list
			.slice(state.top, state.top + ROWS)
			.map((r, i) => {
				const n = r.n;
				const on = state.top + i === state.cursor;
				const kids = n.children.length && state.view === 'tree';
				const caret = kids ? (state.collapsed.includes(keyOf(n)) && !state.filter ? '▸' : '▾') : ' ';
				let right = '';
				if (state.view === 'agenda') right = r.sub;
				else if (kids && n.stats.total) right = `${n.stats.done}/${n.stats.total}`;
				else if (n.meta.fields.due && n.view !== 'done') right = shortDate(n.meta.fields.due);
				let head = '';
				if (state.view === 'agenda') {
					const day = r.d.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' });
					if (day !== lastDay) head = `<div class="day">${esc(day)}</div>`;
					lastDay = day;
				}
				const where = state.view === 'next' && r.sub ? `<span class="t-dim">  ${esc(r.sub)}</span>` : '';
				const waits = sprig.blockers(n) ? '<span class="t-dim"> ⧗</span>' : '';
				return `${head}<div class="row${on ? ' sel' : ''}"><span class="txt"><span class="cur">${on ? '›' : ' '}</span>${'  '.repeat(r.depth)}<span class="t-dim">${caret}</span> ${span(markOf(n))} ${esc(sprig.labelOf(n))}${where}${waits}</span><span class="t-dim">${esc(right)}</span></div>`;
			})
			.join('');
		const views = ['tree', 'next up', 'agenda'].map((v, i) => `<span${state.view === v.split(' ')[0] ? ' class="on"' : ''}>${i + 1} ${v}</span>`).join('');
		const line = state.typing
			? `<span>/${esc(state.filter)}▏</span><span>Enter keeps · Esc clears</span>`
			: `<span>${state.flash ? esc(state.flash) : `${ROOT}  ${s.done}/${s.total} done · ${ready.length} ready · ${waiting.length} waiting${state.filter ? ` · “${esc(state.filter)}”` : ''}`}</span><span>? help</span>`;
		const help = state.help
			? `<div class="help">j k    move          x    tick\nh l    fold          ~    doing\n1 2 3  views         /    filter\ne      edit at line  a A  add\nu      undo          q    quit\n\n<span class="t-dim">any key closes this</span></div>`
			: '';
		return `<div class="hdr"><span class="views">${views}</span><span class="t-dim">~/bakery · ${dayMonth(pinned)}</span></div><div class="panes"><div class="left">${left || '<div class="row t-dim">nothing matches</div>'}</div><div class="right">${detail(sel?.n)}</div></div><div class="sts">${line}</div>${help}`;
	});
}

function tick(state, files, n, to) {
	if (!n) return files;
	if (n.state === 'group' || n.state === 'graft') {
		state.flash = 'groups and grafts take their status from their children';
		return files;
	}
	const lines = files[n.file].split('\n');
	const ln = lines[n.line];
	const i = ln.search(/\S/);
	const from = ln[i];
	if (!sprig.TICKABLE.includes(from)) return files;
	let mark = to || (from === 'x' ? '-' : 'x');
	let edited = ln.slice(0, i) + mark + ln.slice(i + 1);
	state.flash = `${n.file}:${n.line + 1}  ${from} → ${mark}`;
	if (mark === 'x' && n.meta.fields.every) {
		// A recurring item stays open and its due date moves on (spec 10.4).
		const cur = sprig.parseDate(n.meta.fields.due);
		const due = sprig.nextDue(cur && cur >= sprig.TODAY ? cur : sprig.TODAY, n.meta.fields.every);
		if (due) {
			edited = ln.replace(/(^|\s)due:\S+/, `$1due:${sprig.iso(due)}`);
			state.flash = `${n.file}:${n.line + 1}  due → ${sprig.iso(due)}`;
		}
	}
	lines[n.line] = edited;
	return { ...files, [n.file]: lines.join('\n') };
}

/**
 * Apply one key. Returns the new files, or null when the key means nothing
 * here and should reach the browser (Tab, for one).
 */
export function press(state, files, k) {
	if (state.help) {
		state.help = false;
		return files;
	}
	if (state.typing) {
		if (k === 'Escape') [state.typing, state.filter] = [false, ''];
		else if (k === 'Enter') state.typing = false;
		else if (k === 'Backspace') state.filter = state.filter.slice(0, -1);
		else if (k.length === 1) state.filter += k;
		else return null;
		state.cursor = 0;
		return files;
	}
	const sel = sprig.withToday(pinned, () => rows(state, docs(files), files)[state.cursor]);
	state.flash = '';
	switch (k) {
		case 'j': case 'ArrowDown': state.cursor++; break;
		case 'k': case 'ArrowUp': state.cursor--; break;
		case 'g': case 'Home': state.cursor = 0; break;
		case 'G': case 'End': state.cursor = Infinity; break;
		case 'h': case 'ArrowLeft':
			if (sel?.n.children.length && !state.collapsed.includes(keyOf(sel.n))) state.collapsed.push(keyOf(sel.n));
			break;
		case 'l': case 'ArrowRight':
			if (sel) state.collapsed = state.collapsed.filter((c) => c !== keyOf(sel.n));
			break;
		case 'x': return sprig.withToday(pinned, () => tick(state, files, sel?.n));
		case '~': return sprig.withToday(pinned, () => tick(state, files, sel?.n, '~'));
		case '1': [state.view, state.cursor] = ['tree', 0]; break;
		case '2': [state.view, state.cursor] = ['next', 0]; break;
		case '3': [state.view, state.cursor] = ['agenda', 0]; break;
		case '/': [state.typing, state.filter] = [true, '']; break;
		case 'Escape': state.filter = ''; break;
		case '?': state.help = true; break;
		// The real TUI does these; the demo says what would happen.
		case 'e': state.flash = sel ? `would run $EDITOR +${sel.n.line + 1} ${sel.n.file}` : ''; break;
		case 'a': state.flash = 'would add a sibling after this item'; break;
		case 'A': state.flash = 'would add a child under this item'; break;
		case 'u': state.flash = 'would undo the last write'; break;
		case 'q': state.flash = 'q quits the real thing'; break;
		default: return null;
	}
	return files;
}
