---
id: 11
uid: 7733ff53-bf37-4f9c-939a-220cb1cfb8b1
title: 'Conformance examples: lines and indentation'
type: feature
status: done
milestone: v0.1
depends_on:
- 10
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: suite
---

## Problem

The parser's first job, reading lines into a tree, needs examples that pin down every rule before the code exists.

## Proposal

Write at least 40 examples covering title detection, each of the eight marks, `= ` answers, notes, `//` comments, `\` escapes, relative indentation, tabs, blank lines as paragraph breaks, empty items, and a note ending in a colon (still a note).

## Acceptance criteria

- [x] At least 40 examples exist under `tests/conformance/lines/`.
- [x] Every rule in the spec's line and indentation sections is cited by at least one example.
- [x] Expected output agrees with the reference parser in `design/sprig-draft-0.2.html` wherever it covers the case; the examples where they differ are listed in the spec's 'Known ambiguities'.

## 2026-10-28

55 examples in `tests/conformance/lines/`: title detection, each of the eight marks and `=`, a mark needing whitespace, empty items, answers, notes and paragraphs, comments, escapes, blank lines, relative indentation, tabs, depth, CRLF, a BOM, and a note ending in a colon. (2) `the_lines_section_cites_every_line_and_indentation_rule` fails unless every rule from §1.1 to §3.3 is cited. (3) Each example was run through the reference parser (`site/src/lib/sprig.js`, the draft 0.4 port of the design page's parser), comparing title, marks, text, people, note paragraphs, answers and nesting. 51 agree. The 4 that differ are listed in Appendix B: BOM (B.1), Unicode names (B.13), an item between note lines (B.12), and a lone escape (B.14). Two cases where the spec says nothing and sprig-core follows the reference parser are recorded too: B.11 and B.2.

## Result

55 lines examples covering every rule in §1 to §3, agreeing with the reference parser except the four cases now recorded as known ambiguities.
