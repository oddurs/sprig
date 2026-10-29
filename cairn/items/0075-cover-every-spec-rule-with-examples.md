---
id: 75
uid: 443de7c2-0e01-4434-b336-ca984c470b1a
title: Cover every spec rule with examples
type: feature
status: doing
milestone: v1.0
depends_on:
- 20
- 21
- 72
- 74
created: 2026-09-28
updated: 2026-10-29
priority: p0
effort: l
area: suite
---

## Problem

A spec is only as precise as its examples, and a second parser can only be as right as the suite.

## Proposal

Fill every gap the coverage report shows, including the rules the three spike answers add, until every rule has an example and the suite holds at least 400.

## Acceptance criteria

- [x] The coverage report shows zero uncited rules.
- [ ] At least 400 examples pass on all three platforms.

## 2026-10-29

231 new examples, 417 in all. The date, estimate and recurrence cases were generated with checks computed independently from the spec's text (an oracle in Python), then compared with what the core produced. The 9 disagreements on the first run were all the oracle's or the test's mistakes, and one was a wording gap: §10.3 now says that a count in every: is of days or weeks.
