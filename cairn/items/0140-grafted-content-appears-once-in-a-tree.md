---
id: 140
uid: 255791d3-10ef-4a48-a4c3-9489bbad3d3d
title: Grafted content appears once in a tree
type: decision
status: done
created: 2026-09-29
updated: 2026-09-29
priority: p2
area: spec
---

## Context

Decision 0033 kept branch grafts and left one case to the author: "grafting the same branch twice into one tree is the author's error", with double counting as the known cost. It also left open whether `[[file^anchor]]` can reach an anchor that `file` grafted in from elsewhere (Appendix B.7).

## Options and tradeoffs

Warning about duplicates keeps permissive files but lets progress lie until someone reads the warning. Counting a repeated item once hides the mistake while still showing it twice. Refusing the later graft, as a loop is refused (§8.4), keeps every tree a tree: each item has one place, every total is honest, and the rule is the same shape as one readers already know. For B.7, reaching through another file's grafts would make an anchor's meaning depend on files its own file doesn't write, which is exactly what §8.7 exists to prevent.

## Decision

Within one tree an item appears at most once; a graft that would repeat content is an error on the later graft line in document order, with no grafted children (§8.8). `[[file^anchor]]` reaches only anchors that `file` writes itself (§8.7).

## Revisit when

A real workspace needs the same branch in two places of one tree and can't be restructured into two trees.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
