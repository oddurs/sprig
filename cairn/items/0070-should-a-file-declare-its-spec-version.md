---
id: 70
uid: 9be5c4a5-e2f9-4f76-8887-9ce188f88a96
title: Should a file declare its spec version?
type: spike
status: planned
milestone: v0.3
depends_on:
- 7
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: spec
---

## Question

Should a file be able to say which spec version it follows (for example a `sprig:1` field on the title line)? It helps a future 2.0; it costs noise in every file.

## Timebox

One day.

## What would change the answer

Any change planned after 1.0 that a 1.0 parser would misread.

## Answer

(Written when the spike closes. A spike closed without this section filled in was wasted.)

## Acceptance criteria

- [ ] The Answer states what a parser must do with a file that has no version and one with an unknown version.
- [ ] The Answer section states a decision and the evidence for it.
- [ ] Follow-up items exist for everything the answer implies.
