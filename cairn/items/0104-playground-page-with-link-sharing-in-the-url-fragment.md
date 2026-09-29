---
id: 104
uid: 5836462a-928b-444d-8893-3e96e48a74cc
title: Playground page with link sharing in the URL fragment
type: feature
status: planned
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: m
area: site
---

## Problem

Readers want to try their own plan across several files, and share it, without an account or a server.

## Proposal

`/play/`: several files in tabs, the tree and next-up views, people filter, and a Share button that compresses the workspace into the URL fragment with the browser's CompressionStream. Browsers never send the fragment to a server.

## Acceptance criteria

- [ ] Opening a shared link restores every file, byte for byte.
- [ ] Edits persist in the visitor's browser and survive a reload.
- [ ] The example workspace comes from `examples/bakery/`, with its dates shifted so they stay current.
