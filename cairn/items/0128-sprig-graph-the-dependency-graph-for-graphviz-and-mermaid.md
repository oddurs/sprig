---
id: 128
uid: d78d2bd7-cab7-4f70-b70d-f8edd1416498
title: 'sprig graph: the dependency graph for Graphviz and Mermaid'
type: feature
status: planned
milestone: v0.3
depends_on:
- 19
created: 2026-09-28
updated: 2026-09-28
priority: p2
effort: s
area: interop
---

## Problem

A large plan's dependencies are easier to see drawn than listed, and a README often wants the picture.

## Proposal

`sprig graph [--format dot|mermaid]` prints open items that have `after:` edges, plus their targets, with nodes labelled by text and styled by mark. Subgraphs follow files. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] `sprig graph | dot -Tsvg` succeeds on `examples/bakery/` in CI.
- [ ] Mermaid output renders on GitHub; checked by hand and noted here.
- [ ] Node ids are stable across runs, so the output diffs cleanly.
