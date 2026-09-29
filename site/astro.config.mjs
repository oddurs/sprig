// @ts-check
import { readFileSync } from 'node:fs';
import { satteri } from '@astrojs/markdown-satteri';
import starlight from '@astrojs/starlight';
import { defineConfig, passthroughImageService } from 'astro/config';
import { shikiTheme, themes } from './src/design/tokens.mjs';

// The site lives at oddurs.github.io/sprig until the name spike (cairn 0006)
// settles a domain. Moving it is these two values and nothing else: content
// links are written root-relative and prefixed below.
const site = process.env.SITE_URL ?? 'https://oddurs.github.io';
const base = process.env.SITE_BASE ?? '/sprig';

const grammar = JSON.parse(readFileSync(new URL('../editors/sprig.tmLanguage.json', import.meta.url), 'utf8'));
const repo = 'https://github.com/oddurs/sprig';
const ogImage = `${site}${base}/og.png`;

/**
 * Prefix the base path onto root-relative links in Markdown, so content can say
 * `/spec/` and stay correct wherever the site is mounted.
 * @param {string} prefix
 */
function prefixBase(prefix) {
	/** @param {string} url */
	const needs = (url) => url.startsWith('/') && !url.startsWith('//') && !url.startsWith(`${prefix}/`);
	return {
		name: 'sprig-prefix-base',
		/** @param {any} node @param {any} ctx */
		link(node, ctx) {
			if (needs(node.url)) ctx.setProperty(node, 'url', prefix + node.url);
		},
		/** @param {any} node @param {any} ctx */
		definition(node, ctx) {
			if (needs(node.url)) ctx.setProperty(node, 'url', prefix + node.url);
		},
	};
}

export default defineConfig({
	site,
	base,
	trailingSlash: 'always',
	image: { service: passthroughImageService() },
	markdown: { processor: satteri({ mdastPlugins: [prefixBase(base)] }) },
	integrations: [
		starlight({
			title: 'Sprig',
			description: 'A plain-text format for plans. Write the plan; Sprig works out what is done, what is blocked and what you can do next.',
			logo: { src: './src/assets/mark.svg' },
			favicon: '/favicon.svg',
			social: [{ icon: 'github', label: 'GitHub', href: repo }],
			editLink: { baseUrl: `${repo}/edit/main/site/` },
			customCss: ['./src/generated/tokens.css', './src/styles/view.css', './src/styles/starlight.css'],
			disable404Route: true,
			head: [
				{ tag: 'meta', attrs: { property: 'og:image', content: ogImage } },
				{ tag: 'meta', attrs: { property: 'og:image:width', content: '1280' } },
				{ tag: 'meta', attrs: { property: 'og:image:height', content: '640' } },
				{ tag: 'meta', attrs: { name: 'twitter:image', content: ogImage } },
				{ tag: 'link', attrs: { rel: 'alternate', type: 'application/atom+xml', title: 'Sprig releases', href: `${base}/feed.xml` } },
			],
			// Code blocks use syntax themes built from the same tokens as the site's
			// own highlighter. Expressive Code would otherwise nudge colours towards
			// its own contrast target; the tokens already guarantee 4.5:1.
			expressiveCode: {
				themes: [shikiTheme('dark'), shikiTheme('light')],
				minSyntaxHighlightingColorContrast: 0,
				shiki: { langs: [{ ...grammar, name: 'sprig' }] },
				styleOverrides: {
					borderRadius: '8px',
					borderColor: ({ theme }) => themes[theme.type].border,
					codeBackground: ({ theme }) => themes[theme.type]['surface-raised'],
					frames: {
						shadowColor: 'transparent',
						editorTabBarBackground: ({ theme }) => themes[theme.type]['surface-raised'],
						terminalTitlebarBackground: ({ theme }) => themes[theme.type]['surface-raised'],
					},
				},
			},
			sidebar: [
				{
					label: 'Start here',
					items: [
						{ label: 'Your first plan', slug: 'docs/first-plan' },
						{ label: 'Playground', link: '/play/' },
					],
				},
				{ label: 'Reference', items: [{ label: 'Specification', slug: 'spec' }] },
				{
					label: 'Explanation',
					items: [
						{ label: 'The toolchain', link: '/tools/' },
						{ label: 'Settled arguments', slug: 'decisions' },
						{ label: 'How Sprig compares', slug: 'compare' },
					],
				},
				{
					label: 'Project',
					items: [
						{ label: 'Roadmap', slug: 'roadmap' },
						{ label: 'Changelog', slug: 'changelog' },
						{ label: 'Design system', slug: 'docs/design' },
						{ label: 'Contributing', link: `${repo}/blob/main/CONTRIBUTING.md` },
					],
				},
			],
		}),
	],
});
