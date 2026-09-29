---
id: 49
uid: 9e6ccf77-b820-4a98-bbb0-5fdb7ca2261c
title: 'Language server: diagnostics as you type'
type: feature
status: planned
milestone: v0.2
depends_on:
- 25
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: lsp
---

## Problem

`sprig check` only helps when someone runs it. An editor should show the same problems while you type, including ones caused by another file.

## Proposal

`sprig lsp` (or `sprig-lsp`) over stdio, built on `lsp-server` and `lsp-types`, the small synchronous crates rust-analyzer uses, rather than an async framework. It watches the workspace folder and republishes diagnostics for every file affected by a change.

## Acceptance criteria

- [ ] Diagnostics appear on open and on change, driven by a scripted JSON-RPC test client.
- [ ] Deleting `^signed` in `lease.sprig` produces a diagnostic in `bakery.sprig`, which links to it, without `bakery.sprig` being edited.
- [ ] Rechecking a 5,000-line workspace after a keystroke takes under 50 ms on the CI runner.
