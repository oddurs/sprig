---
id: 13
uid: 0a5cb6e2-e769-460d-af19-ab6383bb9d0e
title: Parse lines into a tree with byte spans
type: feature
status: done
milestone: v0.1
depends_on:
- 7
- 11
- 12
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: core
---

## Problem

Everything else in Sprig, from rollups to editing, starts from a correct tree.

## Proposal

Classify each line (title, item by mark, answer, note, comment, escape), build the tree by relative indentation, attach notes and answers to the nearest shallower item, and record the byte span of every node. Spans are there so v0.2 can edit without rewriting what it doesn't touch.

## Acceptance criteria

- [x] All examples under `tests/conformance/lines/` pass.
- [x] Every node's span slices back to exactly its source text.
- [x] The title line becomes the root item, and a file whose first line is an item has no title.

## 2026-10-28

`sprig-core/src/syntax.rs`. Lines are classified as in §2.1. The tree is built from an arena by relative indentation. Notes and answers attach to the nearest shallower item. Every item, title and answer records the byte span of its line after indentation; note lines keep theirs too, for v0.2's edits. (1) All 55 lines examples pass. (2) `every_span_slices_back_to_its_line` checks that every node's span equals its source line, trimmed, in every lines example. (3) Unit tests and examples 001 to 009 cover the title becoming the root, a file whose first line is an item having no title, and an indented, group or answer first line having none.

## Result

A parser from text to a span-carrying syntax tree, with no I/O and no panics, passing the lines suite.
