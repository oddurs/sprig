---
id: 38
uid: 49f454e2-9851-496c-9143-83d019694a7f
title: No numbered steps for sequences
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

`1.` `2.` `3.` is familiar and could chain dependencies automatically.

## Options and tradeoffs

A step still needs a status, so the number competes with the mark for the start of the line. Real work is rarely strictly serial.

## Decision

Rejected. Write `after:` where the order is real.

## Revisit when

If `after:` chains of more than five siblings become common in real plans.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
