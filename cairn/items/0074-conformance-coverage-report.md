---
id: 74
uid: 0a30b95b-90d8-48c8-a4b9-d31d161c1000
title: Conformance coverage report
type: feature
status: done
milestone: v1.0
depends_on:
- 10
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: s
area: suite
---

## Problem

A suite with gaps looks complete until someone implements a rule differently.

## Proposal

A script that lists every numbered rule in `spec/sprig.md` and the examples citing it. CI fails if any rule has none.

## Acceptance criteria

- [x] CI fails on a rule with zero citing examples, shown on a test branch.
- [x] The report is part of the conformance README.

## 2026-10-29

The gate was shown failing on this branch for real: once chapter 10 counted, every_rule_in_the_spec_has_an_example reported §10.4, which no example cited because the format could not state an edit. Edit examples (tick and format, compared byte for byte with after.sprig) closed it. a_rule_with_no_example_is_reported keeps the failing case as a test, on a spec with an uncited rule, in place of a throwaway branch.

## Result

The conformance test lists every numbered rule with the examples citing it, in tests/conformance/README.md, and fails when a rule has none or the report is stale.
