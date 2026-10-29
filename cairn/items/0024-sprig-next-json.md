---
id: 24
uid: 9942669b-e46b-4c7a-bf27-da625b502965
title: sprig next --json
type: feature
status: done
milestone: v0.3
depends_on:
- 9
- 23
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p1
effort: s
area: cli
---

## Problem

Scripts and editor integrations want ready work as data, not text.

## Proposal

`--json` prints the ready items as an array of resolved-layer nodes from the tree schema, plus the path of each.

## Acceptance criteria

- [x] The output validates against `spec/tree.schema.json`.
- [x] It contains the same items in the same order as the text output.

## Result

next --json prints each ready item as {ref, path, node}, node being a resolved node of the tree format without children. A test validates every item against spec/tree.schema.json and compares the refs with --plain's, in order.
