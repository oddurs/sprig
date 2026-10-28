---
id: 20
uid: 4fdd6cb5-a315-426f-b620-70b71ef12075
title: 'Conformance examples: tokens, fields and dates'
type: feature
status: done
milestone: v0.2
depends_on:
- 10
- 15
- 16
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: m
area: suite
---

## Problem

Token rules are the easiest place for two parsers to disagree quietly.

## Proposal

At least 40 examples: every token kind, quotes (including an unterminated quote), URLs, links with and without anchors, fields with odd keys, and every date shortcut with a fixed `today` in `meta.toml`.

## Acceptance criteria

- [x] At least 40 examples exist under `tests/conformance/tokens/` and pass against the core.
- [x] Every rule in the spec's token and date sections is cited at least once.

## 2026-10-28

55 examples in `tests/conformance/tokens/`, generated from a table and blessed, then read line by line against §4 and §9. The token examples agree with the reference parser (draft 0.2's `sprig.js`, from git history) on all but one, Unicode names, which is ruling B.13. The date examples agree except on days that do not exist (B.15).

Writing them found a bug: `feb29` read after a non-leap February had passed gave no date, where the 120-day rule makes it the coming leap day. Fixed in `date::resolve`, with unit tests and `tokens/051`; B.15 now says so.

An unterminated quote does not join tokens (`where:"41` keeps its quote), and `vendor:""` is a field with an empty value. Both match the reference parser; the spec is silent on them, and the examples pin them down.

## Result

55 examples covering every token kind, quotes, URLs, links, odd keys and every date shortcut against a fixed today. A test fails if a §4 or §9 rule goes uncited.
