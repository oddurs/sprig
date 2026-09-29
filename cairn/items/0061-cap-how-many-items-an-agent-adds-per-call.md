---
id: 61
uid: dcf91c3b-bd79-4788-84be-cb058152a42e
title: Cap how many items an agent adds per call
type: feature
status: planned
milestone: v0.3
depends_on:
- 60
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: agents
---

## Problem

An agent can flood a plan with noise faster than a person can review it.

## Proposal

`sprig mcp --max-new-items N` (default 20) limits items added per call and per session.

## Acceptance criteria

- [ ] Exceeding the cap returns an error naming the limit and the flag.
- [ ] The default is documented in `sprig mcp --help` and in the agents guide.
