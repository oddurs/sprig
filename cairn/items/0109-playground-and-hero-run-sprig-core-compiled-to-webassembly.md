---
id: 109
uid: e67ddde5-b182-4f95-b427-5f1eabfb04ee
title: Playground and hero run sprig-core compiled to WebAssembly
type: feature
status: planned
milestone: launch
depends_on:
- 19
- 100
- 104
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: core
---

## Problem

Until sprig-core exists, the site runs the draft 0.2 reference parser from `design/sprig-draft-0.2.html`. Launching with two parsers would break the one-implementation decision: the site and the CLI could disagree.

## Proposal

Compile `sprig-core` to WebAssembly and replace the site's reference parser with it, in the browser and at build time. The npm package stays in `later` (0085).

## Acceptance criteria

- [ ] The site contains no parser code of its own; `site/src/lib/sprig.js` is removed.
- [ ] The prerendered hero is produced by the same WebAssembly module the browser loads.
- [ ] The WebAssembly core loads in under 300 KB compressed.
