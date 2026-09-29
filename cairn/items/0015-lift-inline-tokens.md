---
id: 15
uid: 278cadf6-d0a4-4ddc-b6e8-91f81cccc1e2
title: Lift inline tokens
type: feature
status: planned
milestone: v0.1
depends_on:
- 13
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: core
---

## Problem

Items carry people, tags, anchors, priorities, fields and links inside the line. The resolver needs them lifted out with the remaining text intact.

## Proposal

Split on whitespace while keeping quoted values whole. Lift `@who`, `#tag`, `^id`, `!`/`!!`/`!!!`, `key:value` (with `after:` collected as a list) and `[[file]]`/`[[file^id]]` links. ISO dates parse here; typed shortcuts are a separate item.

## Acceptance criteria

- [ ] Quoted values keep their spaces and lose their quotes.
- [ ] `https://example.org` stays a word, not a field.
- [ ] `#tag` is a tag and `# heading` at a line start is a group.
- [ ] A malformed date in `due:` produces a warning diagnostic, not an error or a panic.
- [ ] Unit tests cover each token kind; the token examples item adds the conformance layer.
