---
id: 21
uid: 5ed6e6f4-90fb-4454-a524-69bfdfe35fe6
title: 'Conformance examples: grafts, rollups and blocking'
type: feature
status: planned
milestone: v0.2
depends_on:
- 10
- 19
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: m
area: suite
---

## Problem

Cross-file behaviour is Sprig's headline feature and the hardest to specify in prose.

## Proposal

At least 40 multi-file examples: whole-file and branch grafts, missing targets, loops, settling, answered questions, recurring items, people inheritance, estimate sums, and local, cross-file and inherited blocking.

## Acceptance criteria

- [ ] At least 40 examples exist under `tests/conformance/graph/` and pass against the core.
- [ ] Every rule in the spec's status, people, blocking and graft sections is cited at least once.
