---
id: 32
uid: 40616b4d-772b-49aa-b6ae-c5314ced5337
title: A parent keeps its written status
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

Computing a parent's status from its children saves a tick.

## Options and tradeoffs

Computed status means the file can say `-` while the view says done, and the text stops being the truth.

## Decision

The file is the source of truth. The resolved layer flags a parent whose children are all done as ready to close. Groups (`#`) are the construct for computed status.

## Revisit when

If users consistently forget to close parents even with the hint shown.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
