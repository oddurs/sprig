---
id: 63
uid: cd670b89-2202-4799-8250-3b2412994d50
title: End-to-end test with a real MCP client
type: chore
status: doing
milestone: v0.3
depends_on:
- 60
- 62
created: 2026-09-28
updated: 2026-10-29
priority: p1
effort: m
area: agents
---

## Purpose

Unit tests prove the server follows its own schema, not that clients can use it.

## Approach

A deterministic scripted session with a real MCP client (the MCP Inspector CLI or equivalent) in CI, plus a manual session with two different coding agents.

## Acceptance criteria

- [x] CI runs the scripted session: read, next, tick, add, answer, and then `sprig check` passes on the result.
- [ ] Manual results for two agent clients, including anything that confused them, are recorded on this item.

## 2026-10-29

CI's mcp job drives sprig mcp through the official TypeScript client (@modelcontextprotocol/sdk 1.32.1, tests/mcp/session.mjs): list, tree, next, tick, add, answer and a refused write, then sprig check --strict on the files. Trying it in two coding agents (say Claude Code and Codex) has to be done by a person, with the server registered from sprig mcp --config.
