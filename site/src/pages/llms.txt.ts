import type { APIRoute } from 'astro';

// A plain-text map of the site for coding agents, following the llms.txt
// convention: what Sprig is, and where the authoritative text lives.
export const GET: APIRoute = ({ site }) => {
	const base = import.meta.env.BASE_URL.replace(/\/$/, '');
	const at = (path: string) => new URL(`${base}${path}`, site).href;
	const body = `# Sprig

> A plain-text format for plans. Each line starts with a mark (- ~ x / > ? = # +), indentation is structure, and progress, blocking and ownership are computed from the text, never written into it.

The specification is the authority. Read it before writing or editing a .sprig file.

## Specification

- [Sprig specification, raw Markdown](${at('/spec.md')}): every rule, numbered, with examples
- [Specification, rendered](${at('/spec/')})

## Docs

- [Your first plan](${at('/docs/first-plan/')}): a five-minute tutorial
- [Settled arguments](${at('/decisions/')}): why each rule is the way it is
- [The toolchain](${at('/tools/')}): the planned sprig commands, their output contract and when each lands
- [How Sprig compares](${at('/compare/')})
- [Roadmap](${at('/roadmap/')})
`;
	return new Response(body, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
