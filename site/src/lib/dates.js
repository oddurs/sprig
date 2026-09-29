// The example plans are written with dates around 2026-10-01. Shifting them by
// whole weeks to the reader's today keeps "due Friday" on a Friday and keeps
// the examples from all reading as overdue a month after they were written.
const WRITTEN = Date.UTC(2026, 9, 1);
const WEEK = 7 * 864e5;

/**
 * @param {Record<string, string>} files
 * @param {Date} today
 */
export function shiftExamples(files, today) {
	const now = Date.UTC(today.getFullYear(), today.getMonth(), today.getDate());
	const days = Math.trunc((now - WRITTEN) / WEEK) * 7;
	if (!days) return { ...files };
	const shift = (iso) => {
		const [y, m, d] = iso.split('-').map(Number);
		return new Date(Date.UTC(y, m - 1, d + days)).toISOString().slice(0, 10);
	};
	return Object.fromEntries(
		Object.entries(files).map(([name, text]) => [name, text.replace(/\b\d{4}-\d{2}-\d{2}\b/g, shift)]),
	);
}
