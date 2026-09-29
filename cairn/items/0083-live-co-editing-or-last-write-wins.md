---
id: 83
uid: 89a89b7d-2641-4326-95c3-6cb0b496e3d8
title: Live co-editing, or last write wins?
type: spike
status: backlog
milestone: later
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: interop
---

## Question

Two people ticking the same shared list in a shop is the first thing a phone app breaks. Is line-level merge of plain files enough, or does it need a CRDT over the text (Automerge, Yjs), and what does that do to plain-file sync through iCloud Drive, Dropbox, Syncthing or git?

## Timebox

One week.

## What would change the answer

Conflict rates measured on real shared lists.

## Answer

(Written when the spike closes. A spike closed without this section filled in was wasted.)

## Acceptance criteria

- [ ] The Answer section states a decision and the evidence for it.
- [ ] Follow-up items exist for everything the answer implies.
