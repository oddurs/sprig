---
id: 128
uid: d78d2bd7-cab7-4f70-b70d-f8edd1416498
title: 'sprig graph: the dependency graph for Graphviz and Mermaid'
type: feature
status: doing
milestone: v0.3
depends_on:
- 19
created: 2026-09-28
updated: 2026-10-29
priority: p2
effort: s
area: interop
---

## Problem

A large plan's dependencies are easier to see drawn than listed, and a README often wants the picture.

## Proposal

`sprig graph [--format dot|mermaid]` prints open items that have `after:` edges, plus their targets, with nodes labelled by text and styled by mark. Subgraphs follow files. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [x] `sprig graph | dot -Tsvg` succeeds on `examples/bakery/` in CI.
- [ ] Mermaid output renders on GitHub; checked by hand and noted here.
- [x] Node ids are stable across runs, so the output diffs cleanly.

## 2026-10-29

dot -Tsvg runs in CI on Linux, where Graphviz is installed and SPRIG_CI_TOOLS makes a missing dot a failure. Ids are file_anchor or file_lLINE (spec/tools.md T5.4), and a test runs the command twice. Rendering the Mermaid output on GitHub still has to be checked by eye.
