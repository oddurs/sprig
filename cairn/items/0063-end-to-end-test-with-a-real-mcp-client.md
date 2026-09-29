---
id: 63
uid: cd670b89-2202-4799-8250-3b2412994d50
title: End-to-end test with a real MCP client
type: chore
status: planned
milestone: v0.3
depends_on:
- 60
- 62
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: m
area: agents
---

## Purpose

Unit tests prove the server follows its own schema, not that clients can use it.

## Approach

A deterministic scripted session with a real MCP client (the MCP Inspector CLI or equivalent) in CI, plus a manual session with two different coding agents.

## Acceptance criteria

- [ ] CI runs the scripted session: read, next, tick, add, answer, and then `sprig check` passes on the result.
- [ ] Manual results for two agent clients, including anything that confused them, are recorded on this item.
