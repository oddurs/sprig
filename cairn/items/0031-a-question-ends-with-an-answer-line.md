---
id: 31
uid: 89e93c9c-a54f-44f1-96c5-cc38125a7bb0
title: A question ends with an '= ' answer line
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

Ticking a `?` with `x` discards the answer, which is often the most valuable line in a plan.

## Options and tradeoffs

Reuse `x` (nothing to learn) or add an answer line (one more line start).

## Decision

`= ` records an answer or decision. A `?` with an answer is done and unblocks anything waiting on it. `grep '^ *= '` is the decision log.

## Revisit when

Never; this is the format's decision log.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
