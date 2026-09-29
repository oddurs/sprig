---
id: 115
uid: e3474261-8140-4c2e-ab1c-4315fd786358
title: 'Design system for the site: tokens, the nine marks, a calmer layout'
type: feature
status: done
milestone: launch
assignee: Oddur Sigurdsson
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p1
effort: l
area: site
---

## Problem

The first version of the site is maximalist and inconsistent: the landing page, the playground and the docs each have their own palette, type and surfaces, and the nine marks, the clearest single explanation of the concept, are coloured ad hoc (the question and answer marks share a colour; the group mark has none).

## Proposal

A small design system for `site/`:

- **Tokens** in one module (`site/src/design/tokens.mjs`): a neutral ramp, a space scale, a type scale, radii and motion, emitted as CSS custom properties and as the syntax themes the docs use. Semantic tokens are the only thing components use.
- **The nine marks as the core**: each gets its own colour, set in OKLCH at equal perceived lightness, and those nine are the only colours on the site. A `MarkArray` component is the one way the marks are shown.
- **Primitives and components** only where the site uses them: Stack, Inline, Grid, Text; Button, Mark, MarkArray, Code, Plan; Section and the page shell as patterns.
- **One look everywhere**: the landing page, playground, docs, 404 and social image all come from the same tokens, with more space and fewer boxes.
- **A design page** in the docs that renders the tokens, the marks with their measured contrast, and every component.

## Acceptance criteria

- [x] Every mark colour meets 4.5:1 against every surface it appears on, in both themes, checked by a script that fails the build.
- [x] No colour value appears in `site/src` outside the token module and the SVG assets; the build fails if one does.
- [x] axe reports no violations on any page in either theme, and the build fails if it does.
- [x] The nine marks render from one component everywhere they appear as a set, and the question and answer marks have distinct colours.
- [x] The docs' code blocks, the landing page's code and the playground use the same syntax colours.
- [x] `/docs/design/` documents every token and component with live examples.

## 2026-09-28

Tokens live in site/src/design/tokens.mjs and are emitted as CSS custom properties (src/generated/tokens.css) and as the docs' Shiki themes. Mark colours: OKLCH at L 0.50 C 0.13 (light) and L 0.78 C 0.12 (dark), with to do and dropped deliberately neutral; question (hue 300) and answer (hue 350) are now distinct, and group has its own hue (195). Checks, all run by scripts/task site: scripts/contrast.mjs passes 50 pairs, the weakest being mark-group on surface-raised in light at 4.72:1; scripts/lint-tokens.mjs holds colour literals, off-scale spacing, type, weight and radius values, and media-query widths to the tokens, and caught all six violations planted in a test file; scripts/a11y.mjs runs axe (WCAG 2.1 AA) on all 10 pages in light and dark with no violations. The first axe run found 7 real problems, all fixed: Starlight's global .secondary/.sm classes overriding components (fixed by namespacing classes and making Button variants attributes), a tab list holding a non-tab, unlabelled roadmap checkboxes, and code blocks that scrolled without keyboard access. MarkArray is the only place the nine marks render as a set (hero, docs, design page); the docs' code blocks, the landing page and the playground share one set of syntax colours. /docs/design/ documents every token and component with live examples. Behaviour re-verified in Chrome: no parser before interaction, ticking and answering unblocks Announce, share links and new files work, nothing overflows at 390px.

## Result

A token-driven design system for the site in which the nine marks are the only colour, enforced by contrast, token-lint and axe checks that fail the build.
