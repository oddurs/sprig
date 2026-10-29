---
id: 148
uid: 25388844-5912-4a14-953f-a869c3ddc6d1
title: 'Dates only: no times of day'
type: decision
status: done
created: 2026-10-29
updated: 2026-10-29
priority: p2
area: spec
---

## Context

Spike 0068 asked whether `due:` and `start:` should take a time of day. RFC 0001 (`rfcs/0001-dates-only.md`) records the argument.

## Options and tradeoffs

A time would let a plan say when in a day something happens. It would also need a time zone to mean one instant, because the same file is read in different places, and that is a calendar's job. todo.txt never added times. TaskPaper allows `HH:MM` but stores no zone, so its times are ambiguous between machines. None of the example plans needs one.

## Decision

Dates only. A plan's grain is the day. A value such as `2027-03-01T14:00` is unreadable (§9.2, §9.5): kept as text, warned about with S0001, and never misread.

## Revisit when

Plans that people actually write need a time, in numbers that outweigh what a time zone costs every reader. A later version could give the form a meaning without changing what any valid 1.x file says.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
