---
id: 25
uid: 7c9f83e3-ddda-4257-aa12-81c72685e803
title: 'sprig check: diagnostics for a workspace'
type: feature
status: planned
milestone: v0.1
depends_on:
- 12
- 19
- 22
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: cli
---

## Problem

Broken links, loops and unknown anchors should fail CI, not surprise someone reading the plan.

## Proposal

Parse and resolve every file given, print diagnostics in the core's format, and exit 1 on any error. Warnings exit 0 unless `--strict`.

## Acceptance criteria

- [ ] Output lines are `file:line:col: severity[code]: message`.
- [ ] `--strict` turns warnings into a non-zero exit.
- [ ] `examples/bakery/` checks clean; a copy with a broken `[[lease^signd]]` link exits 1 naming the file and line.
- [ ] The README shows the one line needed to run it in GitHub Actions.
