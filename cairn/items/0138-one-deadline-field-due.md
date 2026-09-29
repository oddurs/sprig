---
id: 138
uid: 34ffbd07-61d6-40bc-b6b5-cffdaa17c246
title: 'One deadline field: due'
type: decision
status: done
created: 2026-09-29
updated: 2026-09-29
priority: p2
area: spec
---

## Context

Draft 0.3 gave `due` and `target` the same meaning, "when it should be finished" (§9.1). The only use of `target:` was a project's title line, where `due:` works the same way and already flows down into ranking.

## Options and tradeoffs

Keeping both lets a plan separate a hard deadline from an aspiration, but nothing in the format or the tools treated them differently, so the difference lived only in the writer's head, and every implementation had to parse a second date key for nothing. Dropping `target` leaves one word to learn and one field to compute with; anyone who wants the distinction can still write `target:` as an ordinary field (§4.6), shown but not computed.

## Decision

`due` and `start` are the only date fields (§9.1). `target:` is an ordinary field.

## Revisit when

Someone shows a computation that needs two kinds of deadline, not just two labels.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
