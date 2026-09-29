---
id: 20
uid: 4fdd6cb5-a315-426f-b620-70b71ef12075
title: 'Conformance examples: tokens, fields and dates'
type: feature
status: planned
milestone: v0.2
depends_on:
- 10
- 15
- 16
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: m
area: suite
---

## Problem

Token rules are the easiest place for two parsers to disagree quietly.

## Proposal

At least 40 examples: every token kind, quotes (including an unterminated quote), URLs, links with and without anchors, fields with odd keys, and every date shortcut with a fixed `today` in `meta.toml`.

## Acceptance criteria

- [ ] At least 40 examples exist under `tests/conformance/tokens/` and pass against the core.
- [ ] Every rule in the spec's token and date sections is cited at least once.
