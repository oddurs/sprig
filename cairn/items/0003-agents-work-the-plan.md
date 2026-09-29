---
id: 3
uid: a82c2d5d-90e9-4b46-a7bd-8e85e00dfcd2
key: v0.3
title: Agents work the plan
type: milestone
status: planned
depends_on:
- 2
created: 2026-09-28
updated: 2026-09-28
priority: p2
due: 2027-04-02
---

## Ships

`sprig mcp` lets a coding agent read a repository's `PLAN.sprig`, find ready work, tick, add and answer items, where every change is an ordinary file edit a person can review. Plans export to calendars and JSON, and existing checklists import.

## Done when

- [ ] An MCP client drives `sprig mcp` end to end in CI against `examples/bakery/`.
- [ ] The PLAN.sprig convention is documented and `sprig agent` writes agent instructions.
- [ ] The three spec questions blocking 1.0 each have a written answer.

## Explicitly not in this milestone

- Two-way sync with any tracker (later).
- A hosted service of any kind (never, by the ecosystem plan).
