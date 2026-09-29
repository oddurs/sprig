---
id: 12
uid: 711fe65d-42b2-4081-ae90-91fce2a03e58
title: Diagnostics with stable codes
type: feature
status: planned
milestone: v0.1
depends_on:
- 8
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: core
---

## Problem

`sprig check`, the language server and the MCP server all report problems. If each formats its own, codes and wording drift and nobody can search for an error.

## Proposal

One `Diagnostic` type in the core: code (`S0001`...), severity, file, span, message, optional help. The core returns diagnostics and never panics on user input. The catalogue lives in `spec/diagnostics.md`.

## Acceptance criteria

- [ ] Every diagnostic the core can emit has a code listed in `spec/diagnostics.md`, checked by a test that compares the two.
- [ ] Display renders `file:line:col: error[S0003]: message`.
- [ ] Codes are never reused; the catalogue says so.
