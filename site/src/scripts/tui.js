// The tools page's TUI demo. The first frame arrives rendered; the model and
// the parser load the first time someone presses a key or an on-screen button.

const screen = document.getElementById('tui');
const keys = document.getElementById('tui-keys');
const seed = JSON.parse(document.getElementById('tui-files').textContent);

/** @type {Promise<typeof import('../lib/tui.js')> | undefined} */
let loading;
const model = () => (loading ??= import('../lib/tui.js'));
const NAMED = new Set(['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'Escape', 'Enter', 'Backspace']);
let state;
let files = seed;

async function press(key) {
	const tui = await model();
	state ??= tui.initialState();
	const next = tui.press(state, files, key);
	if (next === null) return false;
	files = next;
	screen.innerHTML = tui.frame(state, files);
	return true;
}

screen.addEventListener('keydown', async (e) => {
	if (e.metaKey || e.ctrlKey || e.altKey) return;
	// Decide synchronously whether the key is ours, so Tab and the function
	// keys still reach the browser.
	if (e.key.length !== 1 && !NAMED.has(e.key)) return;
	e.preventDefault();
	await press(e.key);
});
keys.addEventListener('click', (e) => {
	const b = /** @type {HTMLElement} */ (e.target).closest('[data-key]');
	if (b) press(b.dataset.key);
});
