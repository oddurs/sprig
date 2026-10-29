---
id: 59
uid: 3d4b2ac9-f0de-4220-bde2-cf301f9a9970
title: 'sprig mcp: read tools'
type: feature
status: done
milestone: v0.3
depends_on:
- 17
- 24
- 58
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: m
area: agents
---

## Problem

An agent working in a repository can't see the plan unless something serves it.

## Proposal

`sprig mcp` over stdio with the read tools the spike chose, likely: list files, show a file's tree, next (with who and limit), and search.

## Acceptance criteria

- [x] Every tool has a JSON schema, and its results equal the CLI's `--json` output for the same query.
- [x] A scripted MCP client session against `examples/bakery/` passes in CI.

## Result

sprig mcp's five read tools have JSON schemas and return what next, find, status, check and export print with --json, compared in crates/sprig-cli/tests/mcp.rs; a scripted session over stdio runs on the bakery in CI.
