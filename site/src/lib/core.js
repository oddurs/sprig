// chea-core, compiled to WebAssembly: the one parser the site uses, the same
// one the `chea` command runs (cairn 0040). Nothing in the site reads Chea
// syntax itself; it asks this module, and renders what comes back.
//
// The module speaks JSON: a request in, an answer out, through one exported
// call (crates/chea-wasm). At build time it is read from disk; in the browser
// it is fetched on first use, so a page that never edits a plan never loads it.
import wasmUrl from '../generated/chea.wasm?url';

/** @type {WebAssembly.Instance['exports'] & Record<string, any> | undefined} */
let wasm;

async function bytes() {
	if (import.meta.env.SSR) {
		const { readFile } = await import('node:fs/promises');
		const { join } = await import('node:path');
		// The build runs from site/; scripts/task site puts the module here.
		return readFile(join(process.cwd(), 'src/generated/chea.wasm'));
	}
	const res = await fetch(wasmUrl);
	if (!res.ok) throw new Error(`chea-core: ${wasmUrl} answered ${res.status}`);
	return res.arrayBuffer();
}

/** Resolves once the module is ready; every call before that throws. */
export const ready = (async () => {
	const { instance } = await WebAssembly.instantiate(await bytes(), {});
	wasm = instance.exports;
})();

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/**
 * One request to chea-core. Errors in the request come back as thrown
 * errors; the core itself never traps on a plan.
 * @param {Record<string, unknown>} request
 */
export function call(request) {
	if (!wasm) throw new Error('chea-core is not loaded yet: await ready first');
	const input = encoder.encode(JSON.stringify(request));
	const at = wasm.chea_alloc(input.length);
	new Uint8Array(wasm.memory.buffer, at, input.length).set(input);
	const packed = BigInt.asUintN(64, wasm.chea_call(at, input.length));
	const ptr = Number(packed >> 32n);
	const len = Number(packed & 0xffffffffn);
	const text = decoder.decode(new Uint8Array(wasm.memory.buffer, ptr, len));
	wasm.chea_free(ptr, len);
	const answer = JSON.parse(text);
	if (answer && typeof answer === 'object' && !Array.isArray(answer) && 'error' in answer) {
		throw new Error(`chea-core: ${answer.error}`);
	}
	return answer;
}
