---
id: 104
uid: 5836462a-928b-444d-8893-3e96e48a74cc
title: Playground page with link sharing in the URL fragment
type: feature
status: done
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p1
effort: m
area: site
---

## Problem

Readers want to try their own plan across several files, and share it, without an account or a server.

## Proposal

`/play/`: several files in tabs, the tree and next-up views, people filter, and a Share button that compresses the workspace into the URL fragment with the browser's CompressionStream. Browsers never send the fragment to a server.

## Acceptance criteria

- [x] Opening a shared link restores every file, byte for byte.
- [x] Edits persist in the visitor's browser and survive a reload.
- [x] The example workspace comes from `examples/bakery/`, with its dates shifted so they stay current.

## 2026-09-28

/play/ is the design page's playground on the shared parser module, with tabs, new files, tree and next-up views, people filter, hide done and a two-step reset. Share compresses the workspace into #plan= with CompressionStream deflate-raw; build-time links use the same format via node:zlib. Verified in Chrome: a tutorial link opens move.sprig and packing.sprig with the graft resolved; Share then reload restores the same files; an edit survives a reload through localStorage. Examples come from examples/bakery/, shifted by whole weeks so weekdays stay put. Byte-for-byte restoration follows from the JSON round trip; the test compared file names and content, not every byte of every file.

## Result

A multi-file playground with share links carried in the URL fragment, persistence across reloads, and examples that stay current.
