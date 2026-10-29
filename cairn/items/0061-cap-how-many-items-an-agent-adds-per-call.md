---
id: 61
uid: dcf91c3b-bd79-4788-84be-cb058152a42e
title: Cap how many items an agent adds per call
type: feature
status: done
milestone: v0.3
depends_on:
- 60
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p1
effort: s
area: agents
---

## Problem

An agent can flood a plan with noise faster than a person can review it.

## Proposal

`sprig mcp --max-new-items N` (default 20) limits items added per call and per session.

## Acceptance criteria

- [x] Exceeding the cap returns an error naming the limit and the flag.
- [x] The default is documented in `sprig mcp --help` and in the agents guide.

## 2026-10-29

One count covers both: a session may add --max-new-items items (20 by default) across all its calls, so no single call can add more either. A call that would pass it is refused whole, naming the limit, how many were added so far, and the flag.

## Result

sprig mcp --max-new-items N, 20 by default, documented in --help and in the agents guide.
