---
id: 35
uid: 022a4bc4-0324-4670-9149-49bbacf7ffb6
title: Indentation is relative
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

Exactly two spaces per level makes files uniform, but rejecting a file over a stray space is hostile, and pasted text arrives indented every way.

## Options and tradeoffs

Strict two spaces, or relative (deeper than the line above is a child).

## Decision

Relative. Formatters write two spaces; parsers accept any consistent indentation.

## Revisit when

Never.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
