// Build-time helpers around the reference parser: render a plan to HTML, and
// the small facts the landing page states about its examples. Computing them
// here is what keeps a sentence like "Ship waits on Tests" true: if the parser
// disagrees, the page changes with it.
import * as sprig from './sprig.js';

/**
 * @param {Record<string, string>} files
 * @param {string} active
 * @param {'tree' | 'next'} [vm]
 */
export function render(files, active, vm = 'tree') {
	sprig.useWorkspace(files);
	sprig.setView({ vm });
	sprig.beginRender();
	const doc = sprig.buildDoc(active);
	const html =
		vm === 'next'
			? sprig.nextHTML(doc)
			: `<ul class="tree">${doc.root.children.map((c) => sprig.nodeHTML(c, null)).join('')}</ul>`;
	sprig.endRender();
	let ready = 0;
	let waiting = 0;
	const readyText = [];
	const waitingText = [];
	sprig.walkLeaves(doc.root, (n, _path, blocked) => {
		if (!sprig.OPEN.has(n.view)) return;
		if (blocked) {
			waiting++;
			waitingText.push(n.meta.text);
		} else {
			ready++;
			readyText.push(n.meta.text);
		}
	});
	return { doc, html, stats: doc.root.stats, ready, waiting, readyText, waitingText };
}

/** Find an item by its text anywhere in a built document. */
export function find(doc, text) {
	let hit = null;
	(function walk(n) {
		for (const c of n.children) {
			if (c.meta.text === text) hit = c;
			walk(c);
		}
	})(doc.root);
	return hit;
}

export const highlight = sprig.highlight;
export const blockers = sprig.blockers;
export const fmtEst = sprig.fmtEst;

/** Heading slugs as Starlight makes them (github-slugger's rules). */
export function slug(text) {
	return text
		.toLowerCase()
		.replace(/[^\p{L}\p{M}\p{N}\p{Pc} -]/gu, '')
		.replace(/ /g, '-');
}
