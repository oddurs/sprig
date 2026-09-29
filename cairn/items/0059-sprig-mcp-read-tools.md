---
id: 59
uid: 3d4b2ac9-f0de-4220-bde2-cf301f9a9970
title: 'sprig mcp: read tools'
type: feature
status: planned
milestone: v0.3
depends_on:
- 17
- 24
- 58
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: agents
---

## Problem

An agent working in a repository can't see the plan unless something serves it.

## Proposal

`sprig mcp` over stdio with the read tools the spike chose, likely: list files, show a file's tree, next (with who and limit), and search.

## Acceptance criteria

- [ ] Every tool has a JSON schema, and its results equal the CLI's `--json` output for the same query.
- [ ] A scripted MCP client session against `examples/bakery/` passes in CI.
