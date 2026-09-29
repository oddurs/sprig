---
id: 42
uid: 5e274286-7ef2-4701-a1f3-c833f762b194
title: Can edits preserve every byte the user wrote?
type: spike
status: planned
milestone: v0.2
depends_on:
- 13
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: core
---

## Question

Ticking, setting a field, adding an item, answering and `fmt` must change only the bytes they mean to. Is splicing edits into the source by the parser's byte spans enough, or does the core need a lossless syntax tree with trivia (rowan-style)?

## Timebox

Two days.

## What would change the answer

An edit that needs to move or reindent a subtree (adding a child under a deeply nested item, `fmt`) may be awkward with spans alone.

## Answer

(Written when the spike closes. A spike closed without this section filled in was wasted.)

## Acceptance criteria

- [ ] The Answer includes a prototype of tick, add child and fmt on `examples/bakery/` with byte-level diffs shown.
- [ ] The Answer section states a decision and the evidence for it.
- [ ] Follow-up items exist for everything the answer implies.
