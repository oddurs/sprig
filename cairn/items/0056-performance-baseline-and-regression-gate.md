---
id: 56
uid: bfcf2b39-3c98-4deb-82e6-9dd4e9f54a06
title: Performance baseline and regression gate
type: chore
status: planned
milestone: v0.2
depends_on:
- 19
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: core
---

## Purpose

The language server reparses on every keystroke. Slowdowns creep in one small change at a time.

## Approach

A generated 5,000-line, 20-file workspace and a benchmark binary that times parse, resolve and check. CI compares against a committed baseline.

## Acceptance criteria

- [ ] CI fails when the median is more than 2x the committed baseline.
- [ ] The baseline and how to update it are documented in the benchmark's README.
