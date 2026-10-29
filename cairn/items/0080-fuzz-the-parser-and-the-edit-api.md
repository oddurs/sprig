---
id: 80
uid: e9f11cbf-6e15-4a04-ad3a-f39340b42d48
title: Fuzz the parser and the edit API
type: chore
status: done
milestone: v1.0
depends_on:
- 14
- 43
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p1
effort: s
area: core
---

## Purpose

Property tests find what someone thought to check; fuzzing finds the rest.

## Approach

`cargo-fuzz` targets for parsing and for random edit sequences, seeded with the conformance inputs.

## Acceptance criteria

- [x] A one-hour fuzz run per target finds no crash before the 1.0 release.
- [x] Any crash found becomes a regression test in the suite.

## 2026-10-29

First long run, 2026-10-29: both targets ran about 25 minutes before the machine restarted, without a crash. parse made 318,602 runs, reaching 4,219 coverage points. edits made 52,699 runs, reaching 2,914. That is not the hour each that criterion 1 asks for, so it stays open. A full hour of each was restarted from the corpus those runs built, 4,328 inputs. CI runs a minute of each on every change.

## 2026-10-29

Full hour, 2026-10-29, against sprig-core as merged in #34: parse made 640,126 runs in 3,601 seconds, reaching 4,279 coverage points; edits made 88,253 runs in 3,602 seconds, reaching 2,965. Neither found a crash, and the artifacts folders are empty. The second long run was cut short at 14 minutes by another restart, so this one ran as a tracked task. Criterion 2 holds because there was nothing to turn into a test: no crash was found.

## Result

Both targets ran a full hour with no crash; CI keeps running a minute of each on every change.
