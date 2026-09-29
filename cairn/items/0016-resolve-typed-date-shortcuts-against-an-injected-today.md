---
id: 16
uid: c7cfe4c5-d9f2-486a-acbd-ab2c49330121
title: Resolve typed date shortcuts against an injected today
type: feature
status: planned
milestone: v0.2
depends_on:
- 15
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: core
---

## Problem

Draft 0.2 stores ISO dates but still reads `fri`, `+2w`, `oct3`, `q4`, `today` and `tomorrow` as a courtesy. A core that reads the system clock can't be tested and gives different answers on different days.

## Proposal

The resolver takes `today` as a parameter. Shortcut rules follow the spec, including 'a bare month-day more than 120 days back means next year'. Civil-date arithmetic is hand-rolled unless a crate clearly earns its place.

## Acceptance criteria

- [ ] No function in `sprig-core` reads the clock; the scaffold's grep guard still passes.
- [ ] Every shortcut in the spec resolves correctly against at least three different `today` values in unit tests.
- [ ] An unknown shortcut produces a warning diagnostic with the value quoted.
