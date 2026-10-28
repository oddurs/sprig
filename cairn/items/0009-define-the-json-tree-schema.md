---
id: 9
uid: 1a621417-cc5a-46be-85bb-422cb4f33604
title: Define the JSON tree schema
type: feature
status: done
milestone: v0.1
depends_on:
- 7
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: s
area: suite
---

## Problem

Conformance examples need an expected output format, and tools will need an interchange shape. Without one, every test and every consumer invents its own.

## Proposal

`spec/tree.schema.json` (JSON Schema 2020-12) with two layers. The **syntax** layer: title, items with mark, text, tokens, notes, answers, children, and a source span per node. The **resolved** layer: settled status, progress, estimates, effective people and blockers. Keeping them apart lets the parser and the resolver be tested separately. Documents carry `"sprig_tree": 1`.

## Acceptance criteria

- [x] The schema validates a hand-written tree for `examples/bakery/lease.sprig`.
- [x] Syntax and resolved layers are separate objects; the syntax layer contains nothing that depends on other files or on today's date.
- [x] Every field is documented in a spec appendix.
- [x] The schema carries a version and the spec says when it changes.

## 2026-10-28

`spec/tree.schema.json` (JSON Schema 2020-12), written by `sprig_core::json`. (1) `tests/schema/lease.tree.json` is the tree for `examples/bakery/lease.sprig`, checked line by line against the file. It validates, and `tests/schema.rs` also asserts the core writes exactly it, so the two cannot drift. (2) `syntax` and `resolved` are separate objects; a test shows the syntax layer of lease.sprig is identical read alone on 2020-01-01 and within the bakery on 2026-11-20. (3) Spec Appendix D documents every key; a test fails if the schema allows a key Appendix D does not mention. (4) `"sprig_tree": 1`; D says the number changes only when an old reader would misread a new document, and that adding an optional key does not change it. Empty and false values may be left out, so conformance examples stay short.

## Result

The tree format, version 1: schema, Appendix D, and a hand-checked tree for lease.sprig, all tested against what the core writes.
