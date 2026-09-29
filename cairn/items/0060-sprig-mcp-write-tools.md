---
id: 60
uid: 8db4e017-8bb6-4087-8979-a1f0292040ca
title: 'sprig mcp: write tools'
type: feature
status: planned
milestone: v0.3
depends_on:
- 43
- 59
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: agents
---

## Problem

An agent that can read the plan but not update it leaves the plan stale the moment work starts.

## Proposal

Write tools: tick, add an item under an anchor or line, answer a question, set a field. Every write goes through the core's edit API and returns a unified diff of the change.

## Acceptance criteria

- [ ] A write that would introduce a `sprig check` error is refused, with the diagnostic in the error.
- [ ] Bytes outside the edited lines are unchanged, asserted in tests.
- [ ] Every successful write returns the diff it applied.
