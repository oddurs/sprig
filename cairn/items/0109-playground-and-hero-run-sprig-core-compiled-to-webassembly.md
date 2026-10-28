---
id: 109
uid: e67ddde5-b182-4f95-b427-5f1eabfb04ee
title: Playground and hero run sprig-core compiled to WebAssembly
type: feature
status: done
milestone: launch
depends_on:
- 19
- 100
- 104
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: core
---

## Problem

Until sprig-core exists, the site runs the draft 0.2 reference parser from `design/sprig-draft-0.2.html`. Launching with two parsers would break the one-implementation decision: the site and the CLI could disagree.

## Proposal

Compile `sprig-core` to WebAssembly and replace the site's reference parser with it, in the browser and at build time. The npm package stays in `later` (0085).

## Acceptance criteria

- [x] The site contains no parser code of its own; `site/src/lib/sprig.js` is removed.
- [x] The prerendered hero is produced by the same WebAssembly module the browser loads.
- [x] The WebAssembly core loads in under 300 KB compressed.

## 2026-09-28

The parser to replace is site/src/lib/sprig.js; its callers are site/src/lib/render.js (build time), site/src/scripts/hero.js and site/src/scripts/play.js (browser).

## 2026-10-28

New crate `crates/sprig-wasm`: a C ABI (`sprig_alloc`, `sprig_call`, `sprig_free`) taking a JSON request and returning a JSON answer, so no bindings generator is needed. The ops are workspace, lex, inline, date, expand and tick. `scripts/task wasm` builds it with a size-tuned profile into `site/src/generated/sprig.wasm`, and `scripts/task site` runs that first. `site/src/lib/core.js` loads it, from disk at build time and by fetch in the browser on first use. `site/src/lib/view.js` keeps the old module's API and HTML but asks the core for everything that reads Sprig: structure, tokens, links, dates, ticks, Next up's ranking and the editor's colours. The playground's Enter, cycle and blur handling uses the core's line kinds rather than regexes. (1) `site/src/lib/sprig.js` is deleted; `check.mjs` fails the build if a bundle contains the old line pattern, or if there is not exactly one wasm module. (2) The prerendered hero, the tools page's screens and the TUI frame come from the same module the browser loads. A build of main and a build of this branch give byte-identical plan HTML on every page. The only differences are copy that named the reference parser, and `sprig check` on the tools page, which now prints real codes and the core's column. (3) 109 KB gzipped (279 KB raw), checked against a 300 KB budget on every build. In Chromium the playground ticks, expands `due:fri` on Enter and renders Next up; the hero ticks; the TUI moves, ticks and switches views; no console errors.

## Result

The site runs sprig-core compiled to WebAssembly, at 109 KB compressed, and has no parser of its own.
