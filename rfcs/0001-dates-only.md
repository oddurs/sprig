# RFC 0001: Dates only, or times of day too?

- **Status:** accepted
- **Opened:** 2026-10-28
- **Review closed:** 2026-10-28, as the first RFC, from spike 0068
- **Decision item:** cairn 0148
- **Spec sections:** §9.2, §9.5

## The question

Should `due:` and `start:` take a time of day, as in this example?

```sprig
Launch

- Call the printer  due:2027-03-01T14:00
```

## The case for

Some work has an hour attached: a call, a delivery slot, a deadline at noon.
TaskPaper allows `@due(2016-03-02 16:00)`, and a calendar export could carry the
time through.

## The case against

- **A time needs a time zone to name one instant.** The same file is read on
  machines in different places, and a zone would have to be written with the
  time, or be guessed.
- **It is a calendar's job.** A calendar already does times, zones and
  reminders well. A plan's grain is the day.
- **Other formats show the cost.** todo.txt never added times; its community
  extension is `due:YYYY-MM-DD`, and Task Coach drops the time when it exports
  to it. TaskPaper's times carry no zone, so they mean different instants on
  different machines.
- **Nothing needs one yet.** None of the plans in `examples/` uses a time.

## The ruling

Dates only. A stored date is `YYYY-MM-DD` and has no time of day (§9.2). A value
such as `2027-03-01T14:00` is unreadable (§9.5): it is kept as text, warned about
with S0001, and never misread as a date. The examples
`tokens/062-a-date-has-no-time-of-day` and
`tokens/104-a-time-of-day-makes-a-date-unreadable` pin it.

A valid 1.x file never has a readable time today, so a later version could give
the form a meaning without changing what any valid 1.x file says.

## What would change the answer

Plans that people actually write needing a time, in numbers that outweigh what
a time zone costs every reader.
