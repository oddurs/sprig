---
id: 64
uid: 2c209287-0982-413a-8b3a-b11bbb6088a0
title: 'sprig export: JSON and iCalendar'
type: feature
status: planned
milestone: v0.3
depends_on:
- 19
- 24
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: m
area: interop
---

## Problem

Plans are useful outside Sprig tools: every due date belongs in a calendar, and other programs want the tree as data.

## Proposal

`--format json` writes the full resolved tree. `--format ics` writes an all-day event per dated item, with a UID stable across edits: the anchor if there is one, otherwise the file and normalised text.

## Acceptance criteria

- [ ] The JSON validates against the tree schema.
- [ ] The ICS passes an iCalendar validator in a CI test.
- [ ] Renaming an item that has an anchor keeps its UID; a test proves it.
- [ ] Subscribing to the file in Apple Calendar and Google Calendar was checked by hand and noted here.
