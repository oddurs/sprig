---
id: 65
uid: 3a9d1534-ea5d-4a16-ab08-f353b032954d
title: 'sprig import: Markdown checklists and todo.txt'
type: feature
status: planned
milestone: v0.3
depends_on:
- 22
- 44
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: m
area: interop
---

## Problem

Nobody retypes a backlog. Without import, trying Sprig costs an afternoon.

## Proposal

Convert nested `- [ ]` and `- [x]` lists and todo.txt files to `.sprig`. todo.txt priority `(A)` becomes `!!!`, `+project` becomes `#project`, `@context` becomes `#context` (todo.txt's `@` is a place, not a person), and `due:` is kept.

## Acceptance criteria

- [ ] Round-trip fixtures under `tests/import/` cover nesting, done items, priorities and due dates.
- [ ] Imported files pass `sprig check` and are unchanged by `sprig fmt`.
