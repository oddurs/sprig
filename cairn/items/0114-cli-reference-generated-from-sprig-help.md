---
id: 114
uid: 018d8b8e-8799-47e4-9101-a6108c9f29dd
title: CLI reference generated from sprig --help
type: feature
status: planned
milestone: launch
depends_on:
- 27
- 99
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: site
---

## Problem

A command reference written by hand goes stale the first time a flag changes.

## Proposal

The site build runs the released `sprig` binary's help for each command and renders it as `/docs/reference/cli/`.

## Acceptance criteria

- [ ] The page is generated at build time, and adding a command to the CLI adds it to the page with no site change.
