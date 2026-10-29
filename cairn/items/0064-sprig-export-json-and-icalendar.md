---
id: 64
uid: 2c209287-0982-413a-8b3a-b11bbb6088a0
title: 'sprig export: JSON and iCalendar'
type: feature
status: doing
milestone: v0.3
depends_on:
- 19
- 24
created: 2026-09-28
updated: 2026-10-29
priority: p1
effort: m
area: interop
---

## Problem

Plans are useful outside Sprig tools: every due date belongs in a calendar, and other programs want the tree as data.

## Proposal

`--format json` writes the full resolved tree. `--format ics` writes an all-day event per dated item, with a UID stable across edits: the anchor if there is one, otherwise the file and normalised text.

## Acceptance criteria

- [x] The JSON validates against the tree schema.
- [x] The ICS passes an iCalendar validator in a CI test.
- [x] Renaming an item that has an anchor keeps its UID; a test proves it.
- [ ] Subscribing to the file in Apple Calendar and Google Calendar was checked by hand and noted here.

## 2026-10-29

JSON export is a list of tree documents, validated against the schema. The ICS export parses with the icalendar crate, and a test also checks CRLF line endings, 75-octet lines, unique UIDs and the required properties. Renaming the ^loan item keeps its UID. Subscribing in Apple Calendar and Google Calendar has to be done by a person; neither can be reached from CI.
