---
id: 40
uid: 64df05dc-d693-4a2b-836d-686d32f85997
title: One reference implementation, compiled everywhere
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: core
---

## Context

Markdown's decade of dialects came from a thin spec and many parsers that disagreed. See `design/ecosystem-plan.html`, section 'What to build'.

## Options and tradeoffs

Several native implementations (more contributors, and divergence) or one Rust core used by every official tool (no drift, one bottleneck).

## Decision

One Rust core. The CLI, the language server, the MCP server and later the WebAssembly build all use it. Independent parsers are welcome and must pass the conformance suite.

## Revisit when

When a second, independent parser passes the suite; then this is a preference, not a necessity.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
