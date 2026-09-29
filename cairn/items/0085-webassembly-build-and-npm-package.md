---
id: 85
uid: 69b14b78-89b4-4f8b-b352-a8a124ba810b
title: WebAssembly build and npm package
type: feature
status: backlog
milestone: later
depends_on:
- 43
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: m
area: core
---

## Problem

Browser tools, the web viewer and JavaScript developers need the core without a native binary.

## Proposal

Compile `sprig-core` to WebAssembly and publish an npm package with TypeScript types.

## Acceptance criteria

- [ ] The package passes the full conformance suite in Node.
- [ ] The bundle size is recorded, with a budget.
