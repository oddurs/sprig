---
id: 74
uid: 0a30b95b-90d8-48c8-a4b9-d31d161c1000
title: Conformance coverage report
type: feature
status: planned
milestone: v1.0
depends_on:
- 10
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: suite
---

## Problem

A suite with gaps looks complete until someone implements a rule differently.

## Proposal

A script that lists every numbered rule in `spec/sprig.md` and the examples citing it. CI fails if any rule has none.

## Acceptance criteria

- [ ] CI fails on a rule with zero citing examples, shown on a test branch.
- [ ] The report is part of the conformance README.
