---
id: 68
uid: 30f5c7b4-441b-4286-8501-676e8211c03c
title: Dates only, or times of day too?
type: spike
status: done
milestone: v0.3
depends_on:
- 7
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: spec
---

## Question

Should `due:` accept a time (`2027-03-01T14:00`) before the spec freezes? Calendars want times; plans mostly don't.

## Timebox

One day.

## What would change the answer

Real plans in the examples and early adopters' repositories needing times, or calendar export being useless without them.

## Answer

**No: dates only, for 1.0.** There is no grammar change. A plan's grain is the day. A time of day would also need a time zone to mean one instant, because the same file is read in different places, and that is a calendar's job.

Evidence from comparable plain-text formats:
- todo.txt never added times; its community extension is `due:YYYY-MM-DD`, and Task Coach drops the time when it exports to it.
- TaskPaper allows an optional `HH:MM` in `@due(2016-03-02 16:00)` but stores no zone, so the time is ambiguous between machines.
- None of the plans in `examples/` needs a time.

**It stays possible later.** Today `due:2027-03-01T14:00` is unreadable (§9.5): it is kept as text and warned about (S0001), and never misread as a date. A later version could give that form a meaning without changing what any valid 1.x file says.

## Acceptance criteria

- [x] The Answer includes the exact grammar change, or the reason for none.
- [x] The Answer section states a decision and the evidence for it.
- [x] Follow-up items exist for everything the answer implies.

## 2026-10-28

Spec §9.2 now says a date has no time of day, and `tokens/062-a-date-has-no-time-of-day` holds the forward-compatible reading. There are no other follow-ups.
