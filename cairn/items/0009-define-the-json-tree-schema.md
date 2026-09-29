---
id: 9
uid: 1a621417-cc5a-46be-85bb-422cb4f33604
title: Define the JSON tree schema
type: feature
status: planned
milestone: v0.1
depends_on:
- 7
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: suite
---

## Problem

Conformance examples need an expected output format, and tools will need an interchange shape. Without one, every test and every consumer invents its own.

## Proposal

`spec/tree.schema.json` (JSON Schema 2020-12) with two layers. The **syntax** layer: title, items with mark, text, tokens, notes, answers, children, and a source span per node. The **resolved** layer: settled status, progress, estimates, effective people and blockers. Keeping them apart lets the parser and the resolver be tested separately. Documents carry `"sprig_tree": 1`.

## Acceptance criteria

- [ ] The schema validates a hand-written tree for `examples/bakery/lease.sprig`.
- [ ] Syntax and resolved layers are separate objects; the syntax layer contains nothing that depends on other files or on today's date.
- [ ] Every field is documented in a spec appendix.
- [ ] The schema carries a version and the spec says when it changes.
