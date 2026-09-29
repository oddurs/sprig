---
id: 13
uid: 0a5cb6e2-e769-460d-af19-ab6383bb9d0e
title: Parse lines into a tree with byte spans
type: feature
status: planned
milestone: v0.1
depends_on:
- 7
- 11
- 12
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: core
---

## Problem

Everything else in Sprig, from rollups to editing, starts from a correct tree.

## Proposal

Classify each line (title, item by mark, answer, note, comment, escape), build the tree by relative indentation, attach notes and answers to the nearest shallower item, and record the byte span of every node. Spans are there so v0.2 can edit without rewriting what it doesn't touch.

## Acceptance criteria

- [ ] All examples under `tests/conformance/lines/` pass.
- [ ] Every node's span slices back to exactly its source text.
- [ ] The title line becomes the root item, and a file whose first line is an item has no title.
