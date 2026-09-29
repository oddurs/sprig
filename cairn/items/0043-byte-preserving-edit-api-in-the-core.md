---
id: 43
uid: 466fe6e9-3f93-49cf-b648-61ccbe8105a5
title: Byte-preserving edit API in the core
type: feature
status: planned
milestone: v0.2
depends_on:
- 19
- 42
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: core
---

## Problem

The formatter, `sprig tick`, the language server's code actions and the MCP write tools all edit files. They must share one implementation that never disturbs what it wasn't asked to change.

## Proposal

Operations: set mark, set or remove a field, add a child or sibling item at an anchor or line, add an answer. Each returns the new source and the changed ranges, so the language server can send minimal text edits.

## Acceptance criteria

- [ ] A property test applies random edit sequences to every conformance input and asserts that bytes outside the edited lines are unchanged.
- [ ] Parsing the edited source gives the tree the operation promised.
- [ ] Each operation returns its changed ranges, and a test checks them against a real diff.
