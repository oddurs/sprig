---
id: 18
uid: 688153f9-1979-462c-a03d-cbf38016af5b
title: Roll status down and progress up
type: feature
status: done
milestone: v0.1
depends_on:
- 17
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: core
---

## Problem

Settled branches, derived group status, progress, estimates and who owns what are the computed layer that makes a plan more than an outline.

## Proposal

Implement the spec's status and people rules: `x`, `/` and `>` settle a branch; a `?` with an answer is done; groups and grafts derive status from their leaves; recurring items don't count toward progress; `@who` flows down from the title and any item until overridden; `est:` sums open leaves (h, d = 8h, w = 5d).

## Acceptance criteria

- [x] Unit tests cover each rule in the spec's status, progress and people sections.
- [x] Totals for `examples/bakery/bakery.sprig` match the reference parser in `design/sprig-draft-0.2.html` (done, total, in progress, estimate).
- [x] A parent written `-` with every child done stays `-`, and the resolved layer flags it as ready to close.

## 2026-10-28

`roll()` in `workspace.rs`, a port of the reference parser's `roll`. (1) Unit tests cover settling down a branch, groups deriving status, answered questions settling as done, dropped, later and recurring leaves not counting, progress, estimate sums in h, d and w with the parent's own estimate standing in, and people flowing down until someone is named. (2) For all six bakery files, on 2026-09-01, 10-01, 11-16 and 11-20, done, total, doing, estimate and root status equal the reference parser's (`site/src/lib/sprig.js`, run under node). bakery.sprig on 2026-10-01: 9 of 40 done, 4 doing, 284 h. (3) `a_parent_with_every_child_done_is_ready_to_close`: Design stays `todo`, with `ready_to_close: true` in the resolved layer. Recurring parents follow the reference parser, which counts their children (Appendix B.6, unchanged).

## Result

Settling, derived status, progress, estimates and people, equal to the reference parser on every bakery file.
