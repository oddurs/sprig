import type { APIRoute } from 'astro';
import releases from '../generated/releases.json';

// An Atom feed of released versions, built from CHANGELOG.md. It is valid with
// no entries, which is where it starts.
const escape = (s: string) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]!);

export const GET: APIRoute = ({ site }) => {
	const base = import.meta.env.BASE_URL.replace(/\/$/, '');
	const home = new URL(`${base}/`, site).href;
	const self = new URL(`${base}/feed.xml`, site).href;
	const changelog = new URL(`${base}/changelog/`, site).href;
	const updated = releases.length ? `${releases[0].date}T00:00:00Z` : '2026-09-28T00:00:00Z';
	const entries = releases
		.map(
			(r) => `  <entry>
    <title>Sprig ${escape(r.version)}</title>
    <id>${changelog}#${escape(r.version)}</id>
    <link href="${changelog}"/>
    <updated>${r.date}T00:00:00Z</updated>
    <content type="text">${escape(r.notes)}</content>
  </entry>`,
		)
		.join('\n');
	const xml = `<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Sprig releases</title>
  <id>${self}</id>
  <link rel="self" href="${self}"/>
  <link href="${home}"/>
  <updated>${updated}</updated>
  <author><name>Oddur Sigurdsson</name></author>
${entries}
</feed>
`;
	return new Response(xml, { headers: { 'Content-Type': 'application/atom+xml; charset=utf-8' } });
};
