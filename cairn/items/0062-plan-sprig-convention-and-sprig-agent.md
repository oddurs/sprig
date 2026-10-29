---
id: 62
uid: 81a4b3a6-100a-455b-b257-6ecaa285f3e3
title: PLAN.sprig convention and sprig agent
type: docs
status: done
milestone: v0.3
depends_on:
- 59
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: s
area: agents
---

## Reader and question

An agent or its operator asking where the plan lives and how to behave around it.

## Change

`docs/agents.md` defines the convention: `PLAN.sprig` at the repository root, read before starting work, plan written before code is changed, questions for a person asked as `?` items naming them. `sprig agent` prints the instructions for AGENTS.md or CLAUDE.md.

## Acceptance criteria

- [x] `sprig agent --write AGENTS.md` is idempotent: running it twice changes nothing the second time.
- [x] The guide includes a complete worked example session.

## 2026-10-29

The guide is a page on the site (site/src/content/docs/docs/agents.md, /docs/agents/) rather than docs/agents.md, beside the other guides. Its worked session is real output from sprig mcp on the bakery.

## Result

PLAN.sprig at the root, read first, written before code, ticked with the work, and ? items for people: spec/tools.md T6.1 and the agents guide. sprig agent prints the instructions, and --write keeps them in a marked block that a second run leaves unchanged.
