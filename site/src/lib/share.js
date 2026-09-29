// Playground links carry the whole workspace in the URL fragment, compressed.
// Browsers never send the fragment to a server, so a shared plan is never
// uploaded anywhere. This is the build-time encoder; the playground decodes
// with the browser's DecompressionStream, which reads the same deflate-raw.
import { deflateRawSync } from 'node:zlib';

/**
 * @param {Record<string, string>} files
 * @param {string} [active]
 */
export function planFragment(files, active) {
	const order = Object.keys(files);
	const state = { v: 1, files, order, active: active ?? order[0] };
	return `plan=${deflateRawSync(Buffer.from(JSON.stringify(state))).toString('base64url')}`;
}
