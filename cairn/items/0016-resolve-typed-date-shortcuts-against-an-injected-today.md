---
id: 16
uid: c7cfe4c5-d9f2-486a-acbd-ab2c49330121
title: Resolve typed date shortcuts against an injected today
type: feature
status: done
milestone: v0.2
depends_on:
- 15
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: core
---

## Problem

Draft 0.2 stores ISO dates but still reads `fri`, `+2w`, `oct3`, `q4`, `today` and `tomorrow` as a courtesy. A core that reads the system clock can't be tested and gives different answers on different days.

## Proposal

The resolver takes `today` as a parameter. Shortcut rules follow the spec, including 'a bare month-day more than 120 days back means next year'. Civil-date arithmetic is hand-rolled unless a crate clearly earns its place.

## Acceptance criteria

- [x] No function in `sprig-core` reads the clock; the scaffold's grep guard still passes.
- [x] Every shortcut in the spec resolves correctly against at least three different `today` values in unit tests.
- [x] An unknown shortcut produces a warning diagnostic with the value quoted.

## 2026-10-28

`sprig_core::date::resolve(value, today)`, hand-rolled civil-date arithmetic with no crate. Resolution reads `due:` and `start:` through it, so `start:+2w` blocks and `due:fri` ranks in `sprig next`. The resolved layer carries the dates as `due` and `start`. (1) The core's grep guard still passes, and the CLI lint added in v0.1 refuses a clock read anywhere but its argument parser. (2) Unit tests in `date.rs` resolve every §9.3 form against at least three todays each: today, tomorrow, +Nd and +Nw on 2026-10-01, 2026-12-31 and 2028-02-28; quarters on three days, including the day a quarter ends; month-days in all four spellings, on 2026-10-01, 2027-01-10 and 2028-02-28, with the 120-day rule at its boundary; weekdays on three days, today included. Against the reference parser's `parseDate`, 41 values on five todays agree except days that do not exist (`feb30`, and `feb29` outside a leap year). The reference parser rolls those over; sprig-core refuses them, recorded as Appendix B.15. (3) An unreadable value gives S0001 with the value quoted: `due: "soonish" is not a date; it is kept as text`. The same work gives §9.4 (`syntax::expand_dates`) and §10.4 (`date::next_occurrence`, `syntax::tick`), which the site's editor now uses.

## Result

Typed date shortcuts resolve in the core against an injected today, agreeing with the reference parser except where it invents a date.
