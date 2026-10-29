---
id: 80
uid: e9f11cbf-6e15-4a04-ad3a-f39340b42d48
title: Fuzz the parser and the edit API
type: chore
status: doing
milestone: v1.0
depends_on:
- 14
- 43
created: 2026-09-28
updated: 2026-10-29
priority: p1
effort: s
area: core
---

## Purpose

Property tests find what someone thought to check; fuzzing finds the rest.

## Approach

`cargo-fuzz` targets for parsing and for random edit sequences, seeded with the conformance inputs.

## Acceptance criteria

- [ ] A one-hour fuzz run per target finds no crash before the 1.0 release.
- [ ] Any crash found becomes a regression test in the suite.

## 2026-10-29

First long run, 2026-10-29: both targets ran about 25 minutes before the machine restarted, without a crash. parse made 318,602 runs, reaching 4,219 coverage points. edits made 52,699 runs, reaching 2,914. That is not the hour each that criterion 1 asks for, so it stays open. A full hour of each was restarted from the corpus those runs built, 4,328 inputs. CI runs a minute of each on every change.
