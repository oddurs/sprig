// Build-time helpers around chea-core: render a plan to HTML, and the small
// facts the landing page states about its examples. Computing them here is
// what keeps a sentence like "Ship waits on Tests" true: if the core
// disagrees, the page changes with it. Pages `await ready` before calling.
import * as chea from './view.js';

export const ready = chea.ready;

/**
 * @param {Record<string, string>} files
 * @param {string} active
 * @param {'tree' | 'next'} [vm]
 */
export function render(files, active, vm = 'tree') {
	chea.useWorkspace(files);
	chea.setView({ vm });
	chea.beginRender();
	const doc = chea.buildDoc(active);
	const html =
		vm === 'next'
			? chea.nextHTML(doc)
			: `<ul class="tree">${doc.root.children.map((c) => chea.nodeHTML(c, null)).join('')}</ul>`;
	chea.endRender();
	let ready = 0;
	let waiting = 0;
	const readyText = [];
	const waitingText = [];
	chea.walkLeaves(doc.root, (n, _path, blocked) => {
		if (!chea.OPEN.has(n.view)) return;
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

export const highlight = chea.highlight;
export const linesHTML = chea.linesHTML;
export const blockers = chea.blockers;
export const fmtEst = chea.fmtEst;

/** Heading slugs as Starlight makes them (github-slugger's rules). */
export function slug(text) {
	return text
		.toLowerCase()
		.replace(/[^\p{L}\p{M}\p{N}\p{Pc} -]/gu, '')
		.replace(/ /g, '-');
}
