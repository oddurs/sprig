---
id: 29
uid: 24d2a16c-4b09-4114-b5b1-a8ab9024ab0b
title: The title line is the root item; there is no header block
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

Draft 0.1 had `key: value` header lines under the title.

## Options and tradeoffs

Front matter is familiar and easy to scan, but it was a second syntax for fields, with different spacing rules from `key:value` in items.

## Decision

The first line is the root item and takes the same tokens as any item. Values with spaces are quoted.

## Revisit when

If files routinely need more than three file-level fields and title lines become unreadable.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
