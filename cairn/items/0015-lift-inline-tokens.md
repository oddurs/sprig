---
id: 15
uid: 278cadf6-d0a4-4ddc-b6e8-91f81cccc1e2
title: Lift inline tokens
type: feature
status: done
milestone: v0.1
depends_on:
- 13
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: core
---

## Problem

Items carry people, tags, anchors, priorities, fields and links inside the line. The resolver needs them lifted out with the remaining text intact.

## Proposal

Split on whitespace while keeping quoted values whole. Lift `@who`, `#tag`, `^id`, `!`/`!!`/`!!!`, `key:value` (with `after:` collected as a list) and `[[file]]`/`[[file^id]]` links. ISO dates parse here; typed shortcuts are a separate item.

## Acceptance criteria

- [x] Quoted values keep their spaces and lose their quotes.
- [x] `https://example.org` stays a word, not a field.
- [x] `#tag` is a tag and `# heading` at a line start is a group.
- [x] A malformed date in `due:` produces a warning diagnostic, not an error or a panic.
- [x] Unit tests cover each token kind; the token examples item adds the conformance layer.

## 2026-10-28

`sprig-core/src/token.rs`, unit tested per token kind. (1) `where:"41 Mill Street"` keeps its spaces and loses its quotes. The split is the reference parser's, so a quote mid-token runs to the closing quote and an unterminated one is an ordinary word. (2) `https://example.org` stays a word, because a value starting `//` is not a field. (3) `#tag` is a tag in an item, and at a line start it is a note (§2.2, lines example 023), while `# ` makes a group. (4) A malformed `due:` or `start:` gives warning S0001; an unreadable `est:` gives S0002. (5) People, tags, priorities, anchors (several, and repeated: S0003, S0004), case-insensitive fields with the last value winning, `after:` lists, and links with and without anchors are each unit tested. Names accept any Unicode letter (Appendix B.13). The conformance layer for tokens is item 0020.

## Result

Inline tokens lifted with byte spans, matching the reference parser token for token, with warnings for unreadable dates and estimates.
