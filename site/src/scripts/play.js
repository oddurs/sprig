// The playground: several files, an editor with highlighting, the computed
// view, and links that carry the whole workspace in the URL fragment. It is the
// design page's playground, moved onto the shared parser module.
import { shiftExamples } from '../lib/dates.js';
import * as sprig from '../lib/sprig.js';

const $ = (s) => document.querySelector(s);
const seed = JSON.parse($('#examples').textContent);
const EXAMPLES = shiftExamples(seed.files, new Date());
const EXAMPLE_ORDER = seed.order;
const STORE = 'sprig-playground-v1';

const ta = /** @type {HTMLTextAreaElement} */ ($('#src'));
const hl = $('#hl');
const tabs = $('#tabs');
const rv = $('#rv');
const rvHead = $('#rvHead');
const rvBody = $('#rvBody');
const people = $('#people');
const newFile = $('#newFile');
const toast = $('#toast');

let files = {};
let order = [];
let active = '';
let FP = null;
let HIDE = false;
let VM = 'tree';
const COLL = new Set();
let REF = [];

/* ---------- rendering ---------- */

function renderView() {
	sprig.useWorkspace(files);
	sprig.setView({ fp: FP, hide: HIDE, vm: VM, collapsed: [...COLL] });
	sprig.beginRender();
	const doc = sprig.buildDoc(active);
	if (!doc || doc === 'cycle') {
		rvHead.innerHTML = '';
		rvBody.innerHTML = '<p class="empty">This file could not be read.</p>';
		REF = sprig.endRender();
		return;
	}
	const who = new Set();
	(function walk(n) {
		for (const c of n.children) {
			c.who.forEach((x) => who.add(x));
			walk(c);
		}
	})(doc.root);
	if (FP && !who.has(FP)) {
		FP = null;
		sprig.setView({ fp: FP, hide: HIDE, vm: VM, collapsed: [...COLL] });
	}
	people.innerHTML = [`<button type="button" class="pchip" data-fp="" aria-pressed="${!FP}">everyone</button>`]
		.concat([...who].sort().map((p) => `<button type="button" class="pchip" data-fp="${sprig.esc(p)}" aria-pressed="${FP === p}">@${sprig.esc(p)}</button>`))
		.join('');

	const s = doc.root.stats;
	let waiting = 0;
	let asks = 0;
	let parked = 0;
	sprig.walkLeaves(doc.root, (n, _p, b) => {
		if (b && sprig.OPEN.has(n.view)) waiting++;
		if (n.view === 'ask') asks++;
		if (n.view === 'later') parked++;
	});
	const pct = s.total ? (s.done / s.total) * 100 : 0;
	const dp = s.total ? (s.doing / s.total) * 100 : 0;
	const parts = [`<b>${s.done}</b> of ${s.total} done`];
	if (s.doing) parts.push(`${s.doing} in progress`);
	if (waiting) parts.push(`${waiting} waiting`);
	if (asks) parts.push(`${asks} open question${asks > 1 ? 's' : ''}`);
	if (parked) parts.push(`${parked} parked`);
	if (s.est) parts.push(`${sprig.fmtEst(s.est)} of estimated work left`);
	const props = sprig.propsHTML(doc);
	rvHead.innerHTML =
		`<div class="rv-file">${sprig.esc(active)}</div><h2>${sprig.esc(doc.title || active)}</h2>` +
		(props ? `<dl class="props">${props}</dl>` : '') +
		(doc.root.notes.length ? `<div class="rv-notes">${sprig.notesHTML(doc.root.notes)}</div>` : '') +
		`<div class="big"><span class="bar"><i style="width:${pct}%"></i><b style="width:${dp}%"></b></span><span class="pct">${Math.round(pct)}%</span></div>` +
		`<p class="stats">${parts.join(' · ')}</p>`;

	const top = rvBody.scrollTop;
	if (VM === 'next') rvBody.innerHTML = sprig.nextHTML(doc);
	else {
		const items = doc.root.children.map((c) => sprig.nodeHTML(c, null)).join('');
		rvBody.innerHTML = items ? `<ul class="tree">${items}</ul>` : '<p class="empty">No items yet. Start a line with - and a space.</p>';
	}
	rvBody.scrollTop = top;
	REF = sprig.endRender();
}

/* ---------- editor ---------- */

const syncScroll = () => {
	hl.scrollTop = ta.scrollTop;
	hl.scrollLeft = ta.scrollLeft;
};
const paintHL = () => {
	hl.innerHTML = sprig.highlight(ta.value) + '\n\n';
	syncScroll();
};
let renderTimer = 0;
let saveTimer = 0;
function onEdit() {
	files[active] = ta.value;
	paintHL();
	if (location.hash) history.replaceState(null, '', location.pathname + location.search);
	clearTimeout(renderTimer);
	renderTimer = setTimeout(renderView, 70);
	clearTimeout(saveTimer);
	saveTimer = setTimeout(save, 400);
}

function replace(s, e, text, na, nb) {
	ta.focus();
	ta.setSelectionRange(s, e);
	let ok = false;
	try {
		ok = document.execCommand('insertText', false, text);
	} catch {
		ok = false;
	}
	if (!ok) ta.setRangeText(text, s, e, 'end');
	ta.setSelectionRange(na, nb);
	onEdit();
}
function lineAt(v, pos) {
	const s = v.lastIndexOf('\n', pos - 1) + 1;
	let e = v.indexOf('\n', pos);
	if (e < 0) e = v.length;
	return [s, e];
}
function carriesFields(v, s) {
	const [, e] = lineAt(v, s);
	if (sprig.LINE_RE.test(v.slice(s, e).trim())) return true;
	return sprig.titleIndex(v.split('\n')) === v.slice(0, s).split('\n').length - 1;
}

function indentBy(dir) {
	const v = ta.value;
	const a = ta.selectionStart;
	const b = ta.selectionEnd;
	const s = v.lastIndexOf('\n', a - 1) + 1;
	let e = v.indexOf('\n', b > a && v[b - 1] === '\n' ? b - 1 : b);
	if (e < 0) e = v.length;
	let da = 0;
	let db = 0;
	const out = v
		.slice(s, e)
		.split('\n')
		.map((l, idx) => {
			if (dir > 0) {
				if (idx === 0) da = 2;
				db += 2;
				return '  ' + l;
			}
			const r = (l.match(/^ {1,2}/) || [''])[0].length;
			if (idx === 0) da = -Math.min(r, a - s);
			db -= r;
			return l.slice(r);
		});
	replace(s, e, out.join('\n'), Math.max(s, a + da), Math.max(s, b + db));
}

function continueLine() {
	const v = ta.value;
	const a = ta.selectionStart;
	if (a !== ta.selectionEnd) return false;
	const [s, e] = lineAt(v, a);
	const line = v.slice(s, e);
	if (!line.trim()) return false;
	const atEnd = a === e;
	// Leaving a line is when typed dates become stored ones (spec §9.4).
	const settled = atEnd && carriesFields(v, s) ? sprig.expandDates(line) : line;
	const put = (ins) => {
		if (atEnd) {
			const t = settled + ins;
			replace(s, e, t, s + t.length, s + t.length);
		} else replace(a, a, ins, a + ins.length, a + ins.length);
		return true;
	};
	const m = line.match(/^(\s*)([-~x/>?+#=])(?:(\s+)(.*))?$/);
	if (m) {
		if (a < s + m[1].length + 1) return false;
		if (!(m[4] || '').trim()) {
			// Enter on an empty item steps out a level, like an outliner; at the left edge it ends the list.
			const rep = m[1].length >= 2 ? m[1].slice(2) + m[2] + ' ' : '';
			replace(s, e, rep, s + rep.length, s + rep.length);
			return true;
		}
		if (m[2] === '#') return put('\n' + m[1] + '  - ');
		if (m[2] === '=') return put('\n' + m[1]);
		return put('\n' + m[1] + (m[2] === '?' ? '?' : '-') + ' ');
	}
	return put('\n' + line.match(/^\s*/)[0]);
}

function cycleLine() {
	const v = ta.value;
	const a = ta.selectionStart;
	const b = ta.selectionEnd;
	const [s, e] = lineAt(v, a);
	const line = v.slice(s, e);
	const i = line.search(/\S/);
	if (i < 0) return;
	if (sprig.LINE_RE.test(line.slice(i))) {
		const nx = { '-': '~', '~': 'x', x: '-', '/': '-', '>': '-', '?': '-' }[line[i]];
		if (nx) replace(s + i, s + i + 1, nx, a, b);
	} else replace(s + i, s + i, '- ', a + 2, b + 2);
}

ta.addEventListener('input', onEdit);
ta.addEventListener('scroll', syncScroll);
ta.addEventListener('keydown', (e) => {
	if (e.key === 'Tab' && !e.altKey && !e.metaKey && !e.ctrlKey) {
		e.preventDefault();
		indentBy(e.shiftKey ? -1 : 1);
	} else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
		e.preventDefault();
		cycleLine();
	} else if (e.key === 'Enter' && !e.shiftKey && !e.altKey && !e.isComposing) {
		if (continueLine()) e.preventDefault();
	}
});
// Expand typed dates everywhere on blur. The view isn't redrawn: the dates mean
// the same thing today, and redrawing mid-click would swallow the click.
ta.addEventListener('blur', () => {
	const lines = ta.value.split('\n');
	const ti = sprig.titleIndex(lines);
	const out = lines.map((l, i) => (i === ti || sprig.LINE_RE.test(l.trim()) ? sprig.expandDates(l) : l)).join('\n');
	if (out === ta.value) return;
	ta.value = out;
	files[active] = out;
	paintHL();
	save();
});

function gotoLine(line) {
	const lines = ta.value.split('\n');
	if (line < 0 || line >= lines.length) return;
	let off = 0;
	for (let i = 0; i < line; i++) off += lines[i].length + 1;
	const lead = lines[line].search(/\S/);
	if (matchMedia('(pointer: fine)').matches) ta.focus({ preventScroll: true });
	ta.setSelectionRange(off + Math.max(0, lead), off + lines[line].length);
	const lh = parseFloat(getComputedStyle(ta).lineHeight) || 22;
	ta.scrollTop = Math.max(0, line * lh - ta.clientHeight / 3);
	ta.scrollLeft = 0;
	syncScroll();
}

/* ---------- files, storage and sharing ---------- */

function save() {
	try {
		localStorage.setItem(STORE, JSON.stringify({ files, order, active }));
	} catch {
		// Private windows and blocked storage: edits still work, they just don't persist.
	}
}
function resetExamples() {
	files = { ...EXAMPLES };
	order = EXAMPLE_ORDER.slice();
	active = order[0];
}
function valid(s) {
	return (
		s &&
		typeof s.files === 'object' &&
		Array.isArray(s.order) &&
		s.order.length > 0 &&
		s.order.every((f) => typeof s.files[f] === 'string')
	);
}
function adopt(s) {
	files = { ...s.files };
	order = s.order.slice();
	active = s.order.includes(s.active) ? s.active : order[0];
}

function toB64url(bytes) {
	let s = '';
	for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
	return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}
function fromB64url(str) {
	const b64 = str.replace(/-/g, '+').replace(/_/g, '/') + '='.repeat((4 - (str.length % 4)) % 4);
	return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}
async function pipe(bytes, stream) {
	return new Uint8Array(await new Response(new Blob([bytes]).stream().pipeThrough(stream)).arrayBuffer());
}
async function encode(state) {
	return toB64url(await pipe(new TextEncoder().encode(JSON.stringify(state)), new CompressionStream('deflate-raw')));
}
async function decode(data) {
	return JSON.parse(new TextDecoder().decode(await pipe(fromB64url(data), new DecompressionStream('deflate-raw'))));
}

let toastTimer = 0;
function say(message) {
	toast.textContent = message;
	clearTimeout(toastTimer);
	toastTimer = setTimeout(() => (toast.textContent = ''), 2600);
}

$('#share').addEventListener('click', async () => {
	const link = `${location.origin}${location.pathname}#plan=${await encode({ v: 1, files, order, active })}`;
	history.replaceState(null, '', link);
	try {
		await navigator.clipboard.writeText(link);
		say('Link copied');
	} catch {
		say('Link is in the address bar');
	}
});

let naming = false;
function renderTabs() {
	tabs.innerHTML = order
		.map((f) => `<button type="button" class="tab" role="tab" aria-selected="${f === active}" data-file="${sprig.esc(f)}">${sprig.esc(f)}</button>`)
		.join('');
	// The new-file control is not a tab, so it lives beside the tab list, not in it.
	newFile.innerHTML = naming
		? '<input id="newName" class="tab-input" placeholder="file name" aria-label="New file name" maxlength="40">'
		: '<button type="button" class="tab" data-new="1">+ new file</button>';
	const cur = tabs.querySelector('[aria-selected="true"]');
	if (cur) tabs.scrollLeft = Math.max(0, cur.offsetLeft - tabs.clientWidth / 2 + cur.offsetWidth / 2);
	if (naming) {
		const input = $('#newName');
		input.focus();
		input.addEventListener('keydown', (e) => {
			if (e.key === 'Enter') {
				e.preventDefault();
				naming = false;
				createFile(input.value);
			} else if (e.key === 'Escape') {
				naming = false;
				renderTabs();
			}
		});
		input.addEventListener('blur', () => {
			if (naming) {
				naming = false;
				renderTabs();
			}
		});
	}
}

const fileKey = (n) => {
	n = n.trim().replace(/^\.\//, '');
	return /\.sprig$/.test(n) ? n : n + '.sprig';
};
function titleFrom(key) {
	const name = key.replace(/\.sprig$/, '').replace(/[-_]+/g, ' ');
	return name.charAt(0).toUpperCase() + name.slice(1);
}
function createFile(name) {
	const clean = String(name).trim().toLowerCase().replace(/\.sprig$/, '').replace(/\s+/g, '-').replace(/[^\w-]/g, '');
	if (!clean) {
		renderTabs();
		return;
	}
	const key = fileKey(clean);
	if (!(key in files)) {
		files[key] = `${titleFrom(key)}\n\n- `;
		order.push(key);
	}
	switchFile(key);
}

function switchFile(f) {
	active = f;
	ta.value = files[f];
	ta.scrollTop = 0;
	ta.scrollLeft = 0;
	paintHL();
	renderTabs();
	renderView();
	save();
}

function flash(file, line) {
	requestAnimationFrame(() => {
		const el = rvBody.querySelector(`[data-src="${CSS.escape(file + ':' + line)}"]`);
		if (!el) return;
		const r = el.getBoundingClientRect();
		const br = rvBody.getBoundingClientRect();
		rvBody.scrollTop += r.top - br.top - br.height / 3;
		el.classList.remove('flash');
		void el.offsetWidth;
		el.classList.add('flash');
	});
}

function go(file, id) {
	if (!(file in files)) {
		createFile(file);
		return;
	}
	switchFile(file);
	if (id) {
		sprig.useWorkspace(files);
		const d = sprig.buildDoc(file);
		const nd = d && d !== 'cycle' ? d.anchors[id] : null;
		if (nd) {
			flash(file, nd.line);
			gotoLine(nd.line);
		}
	}
}

function toggle(n) {
	if (!n || !(n.file in files)) return;
	const lines = files[n.file].split('\n');
	const ln = lines[n.line];
	if (ln == null) return;
	const i = ln.search(/\S/);
	const ch = ln[i];
	if (!ch || !sprig.TICKABLE.includes(ch)) return;
	let next = null;
	if (ch !== 'x' && n.meta.fields.every) {
		// Recurring (spec §10.4): record what's next, not what happened.
		const cur = sprig.parseDate(n.meta.fields.due);
		const nd = sprig.nextDue(cur && cur >= sprig.TODAY ? cur : sprig.TODAY, n.meta.fields.every);
		if (nd) {
			const tok = 'due:' + sprig.iso(nd);
			next = /(^|\s)due:\S+/.test(ln) ? ln.replace(/(^|\s)due:\S+/, `$1${tok}`) : ln.replace(/\s*$/, '  ' + tok);
			next = next.slice(0, i) + '-' + next.slice(i + 1);
		}
	}
	const recurred = next !== null;
	if (!recurred) next = ln.slice(0, i) + (ch === 'x' ? '-' : 'x') + ln.slice(i + 1);
	lines[n.line] = next;
	files[n.file] = lines.join('\n');
	if (n.file === active) {
		const a = ta.selectionStart;
		const b = ta.selectionEnd;
		const t = ta.scrollTop;
		ta.value = files[active];
		ta.setSelectionRange(a, b);
		ta.scrollTop = t;
		paintHL();
	}
	const refocus = document.activeElement && document.activeElement.classList.contains('mark');
	save();
	renderView();
	if (refocus) rvBody.querySelector(`[data-src="${CSS.escape(n.file + ':' + n.line)}"] .mark`)?.focus();
	if (recurred) flash(n.file, n.line);
}

tabs.addEventListener('click', (e) => {
	const t = e.target.closest('[data-file]');
	if (t) switchFile(t.dataset.file);
});
newFile.addEventListener('click', (e) => {
	if (e.target.closest('[data-new]')) {
		naming = true;
		renderTabs();
	}
});

rv.addEventListener('click', (e) => {
	const g = e.target.closest('[data-go]');
	if (g) {
		e.preventDefault();
		go(g.dataset.go, g.dataset.id);
		return;
	}
	const vm = e.target.closest('[data-vm]');
	if (vm) {
		VM = vm.dataset.vm;
		rv.querySelectorAll('[data-vm]').forEach((b) => b.setAttribute('aria-pressed', String(b === vm)));
		rvBody.scrollTop = 0;
		renderView();
		return;
	}
	const fp = e.target.closest('[data-fp]');
	if (fp) {
		FP = fp.dataset.fp || null;
		renderView();
		return;
	}
	const t = e.target.closest('[data-act]');
	if (!t) return;
	if (t.dataset.act === 'toggle') toggle(REF[Number(t.dataset.k)]);
	else if (t.dataset.act === 'fold') {
		const ck = t.dataset.ck;
		COLL.has(ck) ? COLL.delete(ck) : COLL.add(ck);
		renderView();
	} else if (t.dataset.act === 'src') {
		const n = REF[Number(t.dataset.k)];
		if (!n || n.line < 0) return;
		if (n.file !== active) {
			switchFile(n.file);
			flash(n.file, n.line);
		}
		gotoLine(n.line);
	}
});
$('#hideDone').addEventListener('change', (e) => {
	HIDE = e.target.checked;
	renderView();
});

const resetBtn = $('#reset');
let armTimer = 0;
resetBtn.addEventListener('click', () => {
	if (!resetBtn.classList.contains('armed')) {
		resetBtn.classList.add('armed');
		resetBtn.textContent = 'Discard my edits?';
		armTimer = setTimeout(() => {
			resetBtn.classList.remove('armed');
			resetBtn.textContent = 'Reset';
		}, 3000);
		return;
	}
	clearTimeout(armTimer);
	resetBtn.classList.remove('armed');
	resetBtn.textContent = 'Reset';
	resetExamples();
	COLL.clear();
	FP = null;
	if (location.hash) history.replaceState(null, '', location.pathname);
	switchFile(active);
});

if (!/Mac|iPhone|iPad/.test(navigator.userAgent)) $('#modkey').textContent = 'Ctrl';

/* ---------- start: a shared link, then saved edits, then the examples ---------- */

async function start() {
	const shared = location.hash.startsWith('#plan=') ? location.hash.slice(6) : '';
	if (shared) {
		try {
			const state = await decode(shared);
			if (!valid(state)) throw new Error('not a workspace');
			adopt(state);
			switchFile(active);
			return;
		} catch {
			say('That link could not be read; showing the examples');
		}
	}
	try {
		const saved = JSON.parse(localStorage.getItem(STORE) || 'null');
		if (valid(saved)) {
			adopt(saved);
			switchFile(active);
			return;
		}
	} catch {
		// Unreadable or unavailable storage falls through to the examples.
	}
	resetExamples();
	switchFile(active);
}
start();
