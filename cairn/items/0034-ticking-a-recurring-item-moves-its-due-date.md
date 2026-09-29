---
id: 34
uid: dfeaa2fd-ba85-4397-bbbe-8d29d43c2320
title: Ticking a recurring item moves its due date
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

Draft 0.1 had `every:` with no defined behaviour.

## Options and tradeoffs

Append a history line per completion (history, but plans grow into diaries) or roll `due:` forward (the file says what's next; version control keeps the past).

## Decision

Ticking an item with `every:` moves `due:` to the next occurrence and leaves it open. Recurring items are upkeep and don't count toward progress.

## Revisit when

If people need completion history inside the file rather than in git.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
