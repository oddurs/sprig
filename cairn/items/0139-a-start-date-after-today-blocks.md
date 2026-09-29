---
id: 139
uid: 5af2a0cb-64f9-4c0a-bf1b-4f790e4e93a4
title: A start date after today blocks
type: decision
status: done
created: 2026-09-29
updated: 2026-09-29
priority: p2
area: spec
---

## Context

Draft 0.3 defined `start:` as "when work can begin" and then gave it no effect: nothing computed from it, so writing it changed nothing any tool showed.

## Options and tradeoffs

Dropping `start` is the smallest format. Giving it a meaning costs one sentence, and "not before this day" is one of the most common planning needs, from winter tyres to a permit that can't be filed until a survey is back. Adding a new computed state ("deferred") would give it meaning but add a concept; folding it into blocked adds none, because blocked already means "can't be started now", and the date simply becomes one more thing an item can wait on.

## Decision

An open item whose `start:` is after today is blocked until that day, and its open descendants with it (§7.3, §7.5). Tools say it waits until the date.

## Revisit when

Real plans show people using `start:` as a record of when work did begin, rather than when it may.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
