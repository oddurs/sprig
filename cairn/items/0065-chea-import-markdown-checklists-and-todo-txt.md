---
id: 65
uid: 3a9d1534-ea5d-4a16-ab08-f353b032954d
title: 'chea import: Markdown checklists and todo.txt'
type: feature
status: done
milestone: v0.3
depends_on:
- 22
- 44
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p1
effort: m
area: interop
---

## Problem

Nobody retypes a backlog. Without import, trying Chea costs an afternoon.

## Proposal

Convert nested `- [ ]` and `- [x]` lists and todo.txt files to `.chea`. todo.txt priority `(A)` becomes `!!!`, `+project` becomes `#project`, `@context` becomes `#context` (todo.txt's `@` is a place, not a person), and `due:` is kept.

## Acceptance criteria

- [x] Round-trip fixtures under `tests/import/` cover nesting, done items, priorities and due dates.
- [x] Imported files pass `chea check` and are unchanged by `chea fmt`.

## Result

chea import reads Markdown checklists and todo.txt (by extension, or --from). Fixtures under tests/import/ cover nesting, done items, priorities and due dates; a test converts each, compares it with the expected .chea, and runs check and fmt --check on the result.
