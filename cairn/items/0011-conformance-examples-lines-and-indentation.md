---
id: 11
uid: 7733ff53-bf37-4f9c-939a-220cb1cfb8b1
title: 'Conformance examples: lines and indentation'
type: feature
status: planned
milestone: v0.1
depends_on:
- 10
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: suite
---

## Problem

The parser's first job, reading lines into a tree, needs examples that pin down every rule before the code exists.

## Proposal

Write at least 40 examples covering title detection, each of the eight marks, `= ` answers, notes, `//` comments, `\` escapes, relative indentation, tabs, blank lines as paragraph breaks, empty items, and a note ending in a colon (still a note).

## Acceptance criteria

- [ ] At least 40 examples exist under `tests/conformance/lines/`.
- [ ] Every rule in the spec's line and indentation sections is cited by at least one example.
- [ ] Expected output agrees with the reference parser in `design/sprig-draft-0.2.html` wherever it covers the case; the examples where they differ are listed in the spec's 'Known ambiguities'.
