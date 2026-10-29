// The playground: several files, an editor with highlighting, the computed
// view, what `sprig check` would say, and links that carry the whole workspace
// in the URL fragment. It runs on sprig-core compiled to WebAssembly.
//
// Wide, the source, its problems and the plan share one frame, each pane
// scrolling on its own. Narrow, a switch shows one at a time and each grows
// with what it holds, so the page is the only thing that scrolls.
import { shiftExamples } from '../lib/dates.js';
import * as sprig from '../lib/view.js';

const $ = (s) => document.querySelector(s);
const seed = JSON.parse($('#examples').textContent);
const EXAMPLES = shiftExamples(seed.files, new Date());
const EXAMPLE_ORDER = seed.order;
const STORE = 'sprig-playground-v1';
const BASE = import.meta.env.BASE_URL.replace(/\/$/, '');

const pg = $('#pg');
const ta = /** @type {HTMLTextAreaElement} */ ($('#src'));
const ed = $('#ed');
const hl = $('#hl');
const tabs = $('#tabs');
const rv = $('#rv');
const rvHead = $('#rvHead');
const rvBody = $('#rvBody');
const people = $('#people');
const newFile = $('#newFile');
const toast = $('#toast');
const probsHead = $('#probsHead');
const probsList = $('#probsList');
const pcount = $('#pcount');
const fileMenu = $('#fileMenu');
const fileMenuBtn = $('#fileMenuBtn');

const narrow = matchMedia('(max-width: 859px)');
const touch = matchMedia('(pointer: coarse)');

let files = {};
let order = [];
let active = '';
let FP = null;
let HIDE = false;
let VM = 'tree';
let PANEL = 'src';
const COLL = new Set();
let REF = [];
let PROBLEMS = [];

/* ---------- rendering ---------- */

function renderView() {
	sprig.useWorkspace(files);
	sprig.setView({ fp: FP, hide: HIDE, vm: VM, collapsed: [...COLL] });
	sprig.beginRender();
	renderProblems();
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
	people.innerHTML = [`<button type="button" class="pchip" data-fp="" aria-pressed="${!FP}"><span>everyone</span></button>`]
		.concat([...who].sort().map((p) => `<button type="button" class="pchip" data-fp="${sprig.esc(p)}" aria-pressed="${FP === p}"><span>@${sprig.esc(p)}</span></button>`))
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

/* ---------- problems: what `sprig check` would say ---------- */

const plural = (n, one) => `${n} ${one}${n === 1 ? '' : 's'}`;

function renderProblems() {
	// Errors first, then by file in tab order, then by line.
	const rank = (f) => (order.includes(f) ? order.indexOf(f) : order.length);
	const weight = (p) => (p.severity === 'error' ? 0 : 1);
	PROBLEMS = sprig
		.diagnostics()
		.slice()
		.sort((a, b) => weight(a) - weight(b) || rank(a.file) - rank(b.file) || a.line - b.line || a.col - b.col);
	const errors = PROBLEMS.filter((p) => p.severity === 'error').length;
	const warnings = PROBLEMS.length - errors;
	pcount.textContent = PROBLEMS.length ? String(PROBLEMS.length) : '';
	pcount.classList.toggle('warning', !errors && warnings > 0);
	if (!PROBLEMS.length) {
		probsHead.innerHTML = '✓ No problems: <code>sprig check</code> would exit 0.';
		probsList.innerHTML = '';
	} else {
		const said = [errors && plural(errors, 'error'), warnings && plural(warnings, 'warning')].filter(Boolean).join(' and ');
		probsHead.innerHTML = `<b>${said}</b>: <code>sprig check</code> would exit ${errors ? '1' : '0, or 1 with --strict'}.`;
		probsList.innerHTML = PROBLEMS.map((p, i) => {
			const help = p.help ? `<span class="help">${sprig.esc(p.help)}</span>` : '';
			return (
				`<li class="prob ${p.severity}">` +
				`<button type="button" class="prob-go" data-prob="${i}"><span class="sev" aria-hidden="true">${p.severity === 'error' ? '✕' : '!'}</span>` +
				`<span><span class="msg"><span class="sr-only">${p.severity}: </span>${sprig.esc(p.message)}</span><span class="where">${sprig.esc(p.file)}:${p.line}:${p.col}</span>${help}</span></button>` +
				`<a class="code" href="${BASE}/docs/reference/diagnostics/#${p.code.toLowerCase()}" title="What ${p.code} means">${p.code}</a></li>`
			);
		}).join('');
	}
	// Each tab shows whether its file has a problem.
	const worst = {};
	for (const p of PROBLEMS) if (worst[p.file] !== 'error') worst[p.file] = p.severity;
	for (const t of tabs.querySelectorAll('[data-file]')) {
		const w = worst[t.dataset.file];
		t.querySelector('.dot')?.remove();
		if (w) t.insertAdjacentHTML('beforeend', `<span class="dot ${w}" role="img" aria-label="has ${w}s"></span>`);
	}
	paintHL();
}

/* ---------- editor ---------- */

const syncScroll = () => {
	hl.scrollTop = ta.scrollTop;
	hl.scrollLeft = ta.scrollLeft;
};
function paintHL() {
	const marks = {};
	for (const p of PROBLEMS) if (p.file === active && marks[p.line - 1] !== 'l-error') marks[p.line - 1] = `l-${p.severity}`;
	hl.innerHTML = sprig.linesHTML(ta.value, marks) + '<span class="ln">​</span>';
	fit();
	syncScroll();
}
// Narrow, the editor is as tall as its text, so it never scrolls inside a page
// that scrolls too. Wide, it fills its pane.
function fit() {
	if (!narrow.matches) {
		ed.style.height = '';
		return;
	}
	ed.style.height = '0px';
	ed.style.height = `${Math.max(ta.scrollHeight, hl.scrollHeight)}px`;
}
narrow.addEventListener('change', fit);
addEventListener('resize', () => requestAnimationFrame(fit));

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
	ta.focus({ preventScroll: true });
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
// Fields, and so dates, live on the title and on lines that start with a mark.
function carriesFields(v, s) {
	const kind = sprig.lineKinds(v)[v.slice(0, s).split('\n').length - 1];
	return kind === 'title' || sprig.isItemKind(kind);
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
	const c = sprig.classify(line);
	if (c.marker && sprig.isItemKind(c.kind)) {
		const ind = line.slice(0, c.marker[0]);
		const mark = line.slice(c.marker[0], c.marker[1]);
		if (a < s + ind.length + 1) return false;
		if (!line.slice(c.marker[1]).trim()) {
			// Enter on an empty item steps out a level, like an outliner; at the left edge it ends the list.
			const rep = ind.length >= 2 ? ind.slice(2) + mark + ' ' : '';
			replace(s, e, rep, s + rep.length, s + rep.length);
			return true;
		}
		if (mark === '#') return put('\n' + ind + '  - ');
		if (mark === '=') return put('\n' + ind);
		return put('\n' + ind + (mark === '?' ? '?' : '-') + ' ');
	}
	return put('\n' + line.match(/^\s*/)[0]);
}

// A new item below the current line, at its depth (inside it, under a group),
// wherever the caret is in the line: the key bar's "+ item", since a phone's
// Enter can't tell the editor what was meant.
function newItem() {
	const v = ta.value;
	const [s, e] = lineAt(v, ta.selectionStart);
	const line = v.slice(s, e);
	const c = sprig.classify(line);
	const group = c.marker && line.slice(c.marker[0], c.marker[1]) === '#';
	const ins = `\n${line.match(/^\s*/)[0]}${group ? '  ' : ''}- `;
	replace(e, e, ins, e + ins.length, e + ins.length);
}

function cycleLine() {
	const v = ta.value;
	const a = ta.selectionStart;
	const b = ta.selectionEnd;
	const [s, e] = lineAt(v, a);
	const line = v.slice(s, e);
	const i = line.search(/\S/);
	if (i < 0) return;
	if (sprig.isItemKind(sprig.classify(line).kind)) {
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
// The key bar goes once the tap that took focus away has landed: hiding it
// at once moves the page under the finger, and the click lands elsewhere.
let editingTimer = 0;
ta.addEventListener('focus', () => {
	clearTimeout(editingTimer);
	pg.classList.add('editing');
});
// Expand typed dates everywhere on blur. The view isn't redrawn: the dates mean
// the same thing today, and redrawing mid-click would swallow the click.
ta.addEventListener('blur', () => {
	editingTimer = setTimeout(() => pg.classList.remove('editing'), 300);
	const lines = ta.value.split('\n');
	const kinds = sprig.lineKinds(ta.value);
	const out = lines.map((l, i) => (kinds[i] === 'title' || sprig.isItemKind(kinds[i]) ? sprig.expandDates(l) : l)).join('\n');
	if (out === ta.value) return;
	ta.value = out;
	files[active] = out;
	paintHL();
	save();
});

// The key bar acts on the textarea without taking focus from it, so the
// on-screen keyboard stays up and the selection stays where it was.
const keybar = $('#keybar');
keybar.addEventListener('pointerdown', (e) => {
	if (e.target.closest('[data-key]')) e.preventDefault();
});
keybar.addEventListener('click', (e) => {
	const k = e.target.closest('[data-key]')?.dataset.key;
	if (k === 'indent') indentBy(1);
	else if (k === 'outdent') indentBy(-1);
	else if (k === 'cycle') cycleLine();
	else if (k === 'item') newItem();
	else if (k === 'done') ta.blur();
});

function gotoLine(line) {
	const lines = ta.value.split('\n');
	if (line < 0 || line >= lines.length) return;
	let off = 0;
	for (let i = 0; i < line; i++) off += lines[i].length + 1;
	const lead = lines[line].search(/\S/);
	if (matchMedia('(pointer: fine)').matches) ta.focus({ preventScroll: true });
	ta.setSelectionRange(off + Math.max(0, lead), off + lines[line].length);
	const el = hl.querySelector(`[data-l="${line}"]`);
	if (narrow.matches) {
		// The page scrolls, not the editor: bring the line a third of the way down.
		if (el) window.scrollTo({ top: el.getBoundingClientRect().top + window.scrollY - window.innerHeight / 3 });
		return;
	}
	ta.scrollTop = Math.max(0, (el ? el.offsetTop : 0) - ta.clientHeight / 3);
	ta.scrollLeft = 0;
	syncScroll();
}

/* ---------- panels, narrow ---------- */

const switcher = pg.querySelector('.switch');
function showPanel(p) {
	PANEL = p;
	pg.dataset.panel = p;
	for (const b of switcher.querySelectorAll('[data-panel]')) {
		const on = b.dataset.panel === p;
		b.setAttribute('aria-selected', String(on));
		b.tabIndex = on ? 0 : -1;
	}
	if (p === 'src') fit();
	// Changing panel far down a long one shouldn't strand the reader there.
	const top = pg.getBoundingClientRect().top;
	if (narrow.matches && top < 0) window.scrollTo({ top: window.scrollY + top });
	saveUI();
}
switcher.addEventListener('click', (e) => {
	const b = e.target.closest('[data-panel]');
	if (b) showPanel(b.dataset.panel);
});
switcher.addEventListener('keydown', (e) => {
	if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return;
	const all = [...switcher.querySelectorAll('[data-panel]')];
	const i = all.findIndex((b) => b.dataset.panel === PANEL);
	const next = all[(i + (e.key === 'ArrowRight' ? 1 : all.length - 1)) % all.length];
	showPanel(next.dataset.panel);
	next.focus();
});

/* ---------- files, storage and sharing ---------- */

function save() {
	try {
		localStorage.setItem(STORE, JSON.stringify({ files, order, active }));
	} catch {
		// Private windows and blocked storage: edits still work, they just don't persist.
	}
}
function saveUI() {
	try {
		localStorage.setItem(`${STORE}-ui`, JSON.stringify({ vm: VM, hide: HIDE, panel: PANEL }));
	} catch {
		// As above: a convenience, not state anyone depends on.
	}
}
function loadUI() {
	try {
		const ui = JSON.parse(localStorage.getItem(`${STORE}-ui`) || 'null');
		if (ui) {
			if (ui.vm === 'tree' || ui.vm === 'next') VM = ui.vm;
			HIDE = ui.hide === true;
			if (['src', 'view', 'problems'].includes(ui.panel)) PANEL = ui.panel;
		}
	} catch {
		// Unreadable settings fall back to the defaults.
	}
	rv.querySelectorAll('[data-vm]').forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.vm === VM)));
	$('#hideDone').checked = HIDE;
	showPanel(PANEL);
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
	// By touch, the phone's own share sheet, which can copy as well as send.
	// Closing it without sharing is a choice, not a failure.
	if (touch.matches && navigator.share) {
		try {
			await navigator.share({ title: 'A Sprig plan', url: link });
			return;
		} catch (e) {
			if (e && e.name === 'AbortError') return;
		}
	}
	try {
		await navigator.clipboard.writeText(link);
		say('Link copied');
	} catch {
		say('Link is in the address bar');
	}
});

// Naming is either a new file ('new') or the active one's new name ('rename').
let naming = null;
function renderTabs() {
	tabs.innerHTML = order
		.map((f) => `<button type="button" class="tab" role="tab" aria-selected="${f === active}" data-file="${sprig.esc(f)}">${sprig.esc(f)}</button>`)
		.join('');
	// The new-file control is not a tab, so it lives beside the tab list, not in it.
	const value = naming === 'rename' ? sprig.esc(active.replace(/\.sprig$/, '')) : '';
	const label = naming === 'rename' ? `New name for ${sprig.esc(active)}` : 'New file name';
	newFile.innerHTML = naming
		? `<input id="newName" class="tab-input" placeholder="file name" aria-label="${label}" maxlength="40" value="${value}">`
		: '<button type="button" class="tab" data-new="1">+ new file</button>';
	const cur = tabs.querySelector('[aria-selected="true"]');
	if (cur) tabs.scrollLeft = Math.max(0, cur.offsetLeft - tabs.clientWidth / 2 + cur.offsetWidth / 2);
	fileMenuBtn.setAttribute('aria-label', `File actions for ${active}`);
	if (naming) {
		const input = $('#newName');
		input.focus();
		input.select();
		input.addEventListener('keydown', (e) => {
			if (e.key === 'Enter') {
				e.preventDefault();
				const mode = naming;
				naming = null;
				if (mode === 'rename') renameFile(input.value);
				else createFile(input.value);
			} else if (e.key === 'Escape') {
				naming = null;
				renderTabs();
			}
		});
		input.addEventListener('blur', () => {
			if (naming) {
				naming = null;
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
const cleanName = (name) =>
	String(name)
		.trim()
		.toLowerCase()
		.replace(/\.sprig$/, '')
		.replace(/\s+/g, '-')
		.replace(/[^\w-]/g, '');
function createFile(name) {
	const clean = cleanName(name);
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

// A rename carries every link to the file with it: `[[old]]`, `[[old.sprig]]`
// and `[[old^anchor]]` in every file become the new name, so renaming here
// never breaks a graft the way renaming on disk would.
function renameFile(name) {
	const clean = cleanName(name);
	const from = active;
	const to = clean ? fileKey(clean) : from;
	if (to === from) {
		renderTabs();
		return;
	}
	if (to in files) {
		say(`There is already a ${to}`);
		renderTabs();
		return;
	}
	const stem = from.replace(/\.sprig$/, '').replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	const link = new RegExp(`\\[\\[(\\./)?${stem}(\\.sprig)?(?=[\\]^])`, 'g');
	let moved = 0;
	const next = {};
	for (const f of order) {
		next[f === from ? to : f] = files[f].replace(link, () => {
			moved++;
			return `[[${to.replace(/\.sprig$/, '')}`;
		});
	}
	files = next;
	order = order.map((f) => (f === from ? to : f));
	COLL.clear();
	switchFile(to);
	say(moved ? `Renamed, and ${plural(moved, 'link')} updated` : 'Renamed');
}

function deleteFile() {
	if (order.length < 2) return;
	const i = order.indexOf(active);
	const gone = active;
	delete files[gone];
	order.splice(i, 1);
	switchFile(order[Math.min(i, order.length - 1)]);
	say(`Deleted ${gone}`);
}

function download() {
	const url = URL.createObjectURL(new Blob([files[active]], { type: 'text/plain' }));
	const a = document.createElement('a');
	a.href = url;
	a.download = active;
	document.body.appendChild(a);
	a.click();
	a.remove();
	setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/* the file menu: rename, download, delete */
let deleteArmed = 0;
function closeMenu(focus = false) {
	fileMenu.hidden = true;
	fileMenuBtn.setAttribute('aria-expanded', 'false');
	const del = fileMenu.querySelector('[data-file-act="delete"]');
	del.classList.remove('armed');
	del.textContent = 'Delete';
	clearTimeout(deleteArmed);
	if (focus) fileMenuBtn.focus();
}
fileMenuBtn.addEventListener('click', () => {
	if (!fileMenu.hidden) {
		closeMenu();
		return;
	}
	fileMenu.querySelector('[data-file-act="delete"]').disabled = order.length < 2;
	fileMenu.hidden = false;
	fileMenuBtn.setAttribute('aria-expanded', 'true');
	fileMenu.querySelector('button:not(:disabled)').focus();
});
fileMenu.addEventListener('click', (e) => {
	const b = e.target.closest('[data-file-act]');
	if (!b) return;
	const act = b.dataset.fileAct;
	if (act === 'rename') {
		closeMenu();
		naming = 'rename';
		renderTabs();
	} else if (act === 'download') {
		closeMenu(true);
		download();
	} else if (act === 'delete') {
		// A file holds work, so the first press asks and the second deletes.
		if (!b.classList.contains('armed')) {
			b.classList.add('armed');
			b.textContent = `Delete ${active}?`;
			deleteArmed = setTimeout(() => closeMenu(), 4000);
			return;
		}
		closeMenu(true);
		deleteFile();
	}
});
fileMenu.addEventListener('keydown', (e) => {
	const items = [...fileMenu.querySelectorAll('button:not(:disabled)')];
	const i = items.indexOf(document.activeElement);
	if (e.key === 'Escape') closeMenu(true);
	else if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
		e.preventDefault();
		items[(i + (e.key === 'ArrowDown' ? 1 : items.length - 1)) % items.length].focus();
	}
});
document.addEventListener('pointerdown', (e) => {
	if (!fileMenu.hidden && !e.target.closest('#fileCtl')) closeMenu();
});

function switchFile(f) {
	active = f;
	ta.value = files[f];
	ta.scrollTop = 0;
	ta.scrollLeft = 0;
	renderTabs();
	renderView();
	save();
}

function flash(file, line) {
	requestAnimationFrame(() => {
		const el = rvBody.querySelector(`[data-src="${CSS.escape(file + ':' + line)}"]`);
		if (!el) return;
		if (narrow.matches) el.scrollIntoView({ block: 'center' });
		else {
			const r = el.getBoundingClientRect();
			const br = rvBody.getBoundingClientRect();
			rvBody.scrollTop += r.top - br.top - br.height / 3;
		}
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
			if (!narrow.matches) gotoLine(nd.line);
		}
	}
}

function toggle(n) {
	if (!n || !(n.file in files)) return;
	const lines = files[n.file].split('\n');
	const ln = lines[n.line];
	if (ln == null) return;
	const ticked = sprig.tickLine(ln);
	if (!ticked) return;
	// Recurring (spec §10.4): the core records what's next, not what happened.
	const recurred = ticked.movedTo !== null;
	lines[n.line] = ticked.line;
	files[n.file] = lines.join('\n');
	if (n.file === active) {
		const a = ta.selectionStart;
		const b = ta.selectionEnd;
		const t = ta.scrollTop;
		ta.value = files[active];
		ta.setSelectionRange(a, b);
		ta.scrollTop = t;
	}
	const refocus = document.activeElement && document.activeElement.classList.contains('tick');
	save();
	renderView();
	if (refocus) rvBody.querySelector(`[data-src="${CSS.escape(n.file + ':' + n.line)}"] .tick`)?.focus();
	if (recurred) flash(n.file, n.line);
}

tabs.addEventListener('click', (e) => {
	const t = e.target.closest('[data-file]');
	if (t) switchFile(t.dataset.file);
});
newFile.addEventListener('click', (e) => {
	if (e.target.closest('[data-new]')) {
		naming = 'new';
		renderTabs();
	}
});

probsList.addEventListener('click', (e) => {
	const b = e.target.closest('[data-prob]');
	if (!b) return;
	const p = PROBLEMS[Number(b.dataset.prob)];
	if (!p || !(p.file in files)) return;
	if (narrow.matches) showPanel('src');
	if (p.file !== active) switchFile(p.file);
	gotoLine(p.line - 1);
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
		saveUI();
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
		if (narrow.matches) showPanel('src');
		if (n.file !== active) switchFile(n.file);
		gotoLine(n.line);
	}
});
$('#hideDone').addEventListener('change', (e) => {
	HIDE = e.target.checked;
	renderView();
	saveUI();
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
	loadUI();
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
sprig.ready.then(start);
