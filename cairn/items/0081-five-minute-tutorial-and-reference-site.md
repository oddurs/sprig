---
id: 81
uid: 1135c3ad-df86-4a2e-a180-eaa3d21c3c16
title: Five-minute tutorial and reference site
type: docs
status: planned
milestone: v1.0
depends_on:
- 77
- 78
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: docs
---

## Reader and question

A stranger deciding in five minutes whether Sprig fits their project, then looking up a command later.

## Change

A tutorial that ends with the reader's own `PLAN.sprig` checked, ticked from the editor and read by an agent. The reference covers every CLI command, MCP tool and diagnostic code, generated from the same sources the tools use.

## Acceptance criteria

- [ ] Two people new to Sprig complete the tutorial in under five minutes, with times noted here.
- [ ] The reference is generated, and CI fails if a command or diagnostic is undocumented.
