// The landing page's live example. The page arrives with the example and its
// computed view already rendered; this script only listens, and fetches the
// parser the first time someone touches the demo. Until then no parser code
// has loaded at all.

const demo = document.getElementById('demo');
const src = /** @type {HTMLTextAreaElement} */ (document.getElementById('demo-src'));
const hl = document.getElementById('demo-hl');
const body = document.getElementById('demo-body');
const stats = document.getElementById('demo-stats');
const editBtn = document.getElementById('demo-edit');
const presets = JSON.parse(document.getElementById('demo-presets').textContent);

/** @type {Promise<typeof import('../lib/sprig.js')> | undefined} */
let loading;
const parser = () => (loading ??= import('../lib/sprig.js'));

const state = { vm: 'next', refs: [], collapsed: new Set() };

async function render() {
	const sprig = await parser();
	sprig.useWorkspace({ 'plan.sprig': src.value });
	sprig.setView({ vm: state.vm, collapsed: [...state.collapsed] });
	sprig.beginRender();
	const doc = sprig.buildDoc('plan.sprig');
	let ready = 0;
	let waiting = 0;
	sprig.walkLeaves(doc.root, (n, _p, blocked) => {
		if (sprig.OPEN.has(n.view)) blocked ? waiting++ : ready++;
	});
	body.innerHTML =
		state.vm === 'next'
			? sprig.nextHTML(doc)
			: `<ul class="tree">${doc.root.children.map((c) => sprig.nodeHTML(c, null)).join('')}</ul>`;
	state.refs = sprig.endRender();
	stats.textContent = `${doc.root.stats.done}/${doc.root.stats.total} done · ${ready} ready · ${waiting} waiting`;
}

async function paint() {
	const sprig = await parser();
	hl.innerHTML = sprig.highlight(src.value) + '\n\n';
	hl.scrollTop = src.scrollTop;
	hl.scrollLeft = src.scrollLeft;
}

let timer = 0;
src.addEventListener('input', () => {
	paint();
	clearTimeout(timer);
	timer = setTimeout(render, 60);
});
src.addEventListener('scroll', () => {
	hl.scrollTop = src.scrollTop;
	hl.scrollLeft = src.scrollLeft;
});
src.addEventListener('focus', parser, { once: true });
src.addEventListener('keydown', (e) => {
	if (e.key !== 'Tab' || e.metaKey || e.ctrlKey || e.altKey) return;
	e.preventDefault();
	const at = src.selectionStart;
	const start = src.value.lastIndexOf('\n', at - 1) + 1;
	if (e.shiftKey) {
		const drop = (src.value.slice(start).match(/^ {1,2}/) || [''])[0].length;
		src.setRangeText('', start, start + drop, 'preserve');
		src.setSelectionRange(Math.max(start, at - drop), Math.max(start, at - drop));
	} else {
		src.setRangeText('  ', start, start, 'preserve');
		src.setSelectionRange(at + 2, at + 2);
	}
	src.dispatchEvent(new Event('input'));
});

demo.addEventListener('click', async (e) => {
	const target = /** @type {HTMLElement} */ (e.target);
	const preset = target.closest('[data-preset]');
	if (preset) {
		demo.querySelectorAll('[data-preset]').forEach((t) => t.setAttribute('aria-selected', String(t === preset)));
		src.value = presets[preset.dataset.preset];
		src.scrollTop = 0;
		state.collapsed.clear();
		await paint();
		await render();
		return;
	}
	const vm = target.closest('[data-vm]');
	if (vm) {
		state.vm = vm.dataset.vm;
		demo.querySelectorAll('[data-vm]').forEach((b) => b.setAttribute('aria-pressed', String(b === vm)));
		await render();
		return;
	}
	const act = target.closest('[data-act]');
	if (!act) return;
	const sprig = await parser();
	// The prerendered view has no refs yet; build them once, then act.
	if (!state.refs.length) await render();
	const node = state.refs[Number(act.dataset.k)];
	if (act.dataset.act === 'toggle' && node) {
		const lines = src.value.split('\n');
		const line = lines[node.line];
		const i = line.search(/\S/);
		if (!sprig.TICKABLE.includes(line[i])) return;
		lines[node.line] = line.slice(0, i) + (line[i] === 'x' ? '-' : 'x') + line.slice(i + 1);
		src.value = lines.join('\n');
		await paint();
		await render();
	} else if (act.dataset.act === 'fold') {
		const key = act.dataset.ck;
		state.collapsed.has(key) ? state.collapsed.delete(key) : state.collapsed.add(key);
		await render();
	} else if (act.dataset.act === 'src' && node) {
		const lines = src.value.split('\n');
		let offset = 0;
		for (let i = 0; i < node.line; i++) offset += lines[i].length + 1;
		if (matchMedia('(pointer: fine)').matches) src.focus({ preventScroll: true });
		src.setSelectionRange(offset, offset + lines[node.line].length);
	}
});

editBtn.addEventListener('click', () => {
	const open = demo.classList.toggle('src-open');
	editBtn.setAttribute('aria-expanded', String(open));
	editBtn.textContent = open ? 'Hide the source' : 'Edit the source';
});
