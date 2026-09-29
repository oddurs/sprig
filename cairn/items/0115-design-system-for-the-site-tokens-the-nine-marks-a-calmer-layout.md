---
id: 115
uid: e3474261-8140-4c2e-ab1c-4315fd786358
title: 'Design system for the site: tokens, the nine marks, a calmer layout'
type: feature
status: planned
milestone: launch
created: 2026-09-28
updated: 2026-09-28
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

- [ ] Every mark colour meets 4.5:1 against every surface it appears on, in both themes, checked by a script that fails the build.
- [ ] No colour value appears in `site/src` outside the token module and the SVG assets; the build fails if one does.
- [ ] axe reports no violations on any page in either theme, and the build fails if it does.
- [ ] The nine marks render from one component everywhere they appear as a set, and the question and answer marks have distinct colours.
- [ ] The docs' code blocks, the landing page's code and the playground use the same syntax colours.
- [ ] `/docs/design/` documents every token and component with live examples.
