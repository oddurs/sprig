---
id: 58
uid: c4ebdb45-d994-4fa4-a371-00bd39331a6e
title: What tools should sprig mcp expose?
type: spike
status: planned
milestone: v0.3
depends_on:
- 23
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: agents
---

## Question

Which tools, with what arguments, how writes report what they changed, and how errors look. Read the MCP specification and an existing backlog MCP server (cairn's `cairn mcp` at github.com/oddurs/cairn is a close model) before deciding.

## Timebox

Two days.

## What would change the answer

Agent clients handling many small tools badly compared with a few general ones, or a client requiring confirmation for writes.

## Answer

(Written when the spike closes. A spike closed without this section filled in was wasted.)

## Acceptance criteria

- [ ] The Answer lists every tool with its argument schema and an example call and result.
- [ ] The Answer section states a decision and the evidence for it.
- [ ] Follow-up items exist for everything the answer implies.
