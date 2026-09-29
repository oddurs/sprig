---
id: 18
uid: 688153f9-1979-462c-a03d-cbf38016af5b
title: Roll status down and progress up
type: feature
status: planned
milestone: v0.1
depends_on:
- 17
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: core
---

## Problem

Settled branches, derived group status, progress, estimates and who owns what are the computed layer that makes a plan more than an outline.

## Proposal

Implement the spec's status and people rules: `x`, `/` and `>` settle a branch; a `?` with an answer is done; groups and grafts derive status from their leaves; recurring items don't count toward progress; `@who` flows down from the title and any item until overridden; `est:` sums open leaves (h, d = 8h, w = 5d).

## Acceptance criteria

- [ ] Unit tests cover each rule in the spec's status, progress and people sections.
- [ ] Totals for `examples/bakery/bakery.sprig` match the reference parser in `design/sprig-draft-0.2.html` (done, total, in progress, estimate).
- [ ] A parent written `-` with every child done stays `-`, and the resolved layer flags it as ready to close.
