---
id: 24
uid: 9942669b-e46b-4c7a-bf27-da625b502965
title: sprig next --json
type: feature
status: planned
milestone: v0.3
depends_on:
- 9
- 23
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: cli
---

## Problem

Scripts and editor integrations want ready work as data, not text.

## Proposal

`--json` prints the ready items as an array of resolved-layer nodes from the tree schema, plus the path of each.

## Acceptance criteria

- [ ] The output validates against `spec/tree.schema.json`.
- [ ] It contains the same items in the same order as the text output.
