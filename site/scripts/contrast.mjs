// Every text and mark colour, against every surface it can sit on, in both
// themes. Below the minimum, the build stops. This is what makes "the marks
// are legible" a fact rather than a hope.
import { contrast, contrastRules, themes } from '../src/design/tokens.mjs';

const failures = [];
const rows = [];
for (const [name, t] of Object.entries(themes)) {
	for (const { fg, bg, min } of contrastRules) {
		const ratio = contrast(t[fg], t[bg]);
		rows.push({ name, fg, bg, ratio });
		if (ratio < min) failures.push(`${name}: ${fg} on ${bg} is ${ratio.toFixed(2)}:1, needs ${min}:1`);
	}
}
if (failures.length) {
	console.error(`contrast: ${failures.length} failure(s)\n  ${failures.join('\n  ')}`);
	process.exit(1);
}
const lowest = rows.filter((r) => r.fg.startsWith('mark-')).sort((a, b) => a.ratio - b.ratio)[0];
console.log(`contrast: ${rows.length} pairs pass; weakest mark is ${lowest.fg} on ${lowest.bg} (${lowest.name}) at ${lowest.ratio.toFixed(2)}:1`);
