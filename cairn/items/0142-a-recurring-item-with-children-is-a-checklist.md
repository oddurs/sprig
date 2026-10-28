---
id: 142
uid: 312d8710-7a8e-48b4-917d-3d70133d1096
title: A recurring item with children is a checklist
type: decision
status: done
created: 2026-10-28
updated: 2026-10-28
priority: p2
area: spec
---

## Context

§5.3 said no item with `every:` counts toward progress. The reference parser excluded only recurring leaves, so a recurring parent's children still counted, and `examples/bakery/groceries.sprig` (a weekly supply run on the title) depends on that (Appendix B.6).

## Options and tradeoffs

Excluding the whole branch makes every recurring list read as empty, with nothing to show for a week's shopping. Counting the children treats the recurring item as what it is: a checklist done again each time, whose progress is this round's. A recurring leaf is still upkeep, since there is nothing in it to count but itself.

## Decision

Only a recurring leaf is upkeep. A recurring item with children is a parent like any other, and its children count (§5.3).

## Revisit when

A tool needs to reset a recurring checklist's children when it is ticked; that would be a new rule, not a change to this one.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
