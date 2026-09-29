---
id: 62
uid: 81a4b3a6-100a-455b-b257-6ecaa285f3e3
title: PLAN.sprig convention and sprig agent
type: docs
status: planned
milestone: v0.3
depends_on:
- 59
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: agents
---

## Reader and question

An agent or its operator asking where the plan lives and how to behave around it.

## Change

`docs/agents.md` defines the convention: `PLAN.sprig` at the repository root, read before starting work, plan written before code is changed, questions for a person asked as `?` items naming them. `sprig agent` prints the instructions for AGENTS.md or CLAUDE.md.

## Acceptance criteria

- [ ] `sprig agent --write AGENTS.md` is idempotent: running it twice changes nothing the second time.
- [ ] The guide includes a complete worked example session.
