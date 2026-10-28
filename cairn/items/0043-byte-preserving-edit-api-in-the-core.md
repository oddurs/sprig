---
id: 43
uid: 466fe6e9-3f93-49cf-b648-61ccbe8105a5
title: Byte-preserving edit API in the core
type: feature
status: done
milestone: v0.2
depends_on:
- 19
- 42
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: core
---

## Problem

The formatter, `sprig tick`, the language server's code actions and the MCP write tools all edit files. They must share one implementation that never disturbs what it wasn't asked to change.

## Proposal

Operations: set mark, set or remove a field, add a child or sibling item at an anchor or line, add an answer. Each returns the new source and the changed ranges, so the language server can send minimal text edits.

## Acceptance criteria

- [x] A property test applies random edit sequences to every conformance input and asserts that bytes outside the edited lines are unchanged.
- [x] Parsing the edited source gives the tree the operation promised.
- [x] Each operation returns its changed ranges, and a test checks them against a real diff.

## 2026-10-28

`sprig_core::edit`: `set_mark`, `tick`, `set_field` (set, append or remove, quoting values with spaces), `add_item` (child or sibling), `add_answer` and `format`. Each returns a `Change`: the new source, and sorted, non-overlapping replacements in the old source's bytes, which is what an LSP `TextEdit` wants. (1) `random_edit_sequences_touch_only_the_lines_they_edit` applies random sequences of every operation to every conformance input and example. Every line no replacement touches comes out byte-identical (20,000 cases run once, 1,000 in CI). (2) The same test parses each result and checks the promise: the mark set, the field's value, the new child last under its parent, one more answer. (3) Each change's replacements, applied, give its source, never overlap, and are compared line by line against the old and new text. Added lines take the file's line ending and their siblings' indentation; a last line without an ending gets one first.

## Result

One byte-preserving edit API in the core, property-tested against every input in the repository.
