---
id: 67
uid: c0cd8e18-59fb-4f98-a431-0e4be528d8d8
title: 'GitHub Action: check plans and summarise progress in pull requests'
type: feature
status: planned
milestone: v0.3
depends_on:
- 25
- 27
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: interop
---

## Problem

A plan changed in a pull request should be checked, and the reviewer should see what the change did to progress.

## Proposal

A composite action that installs a released binary, runs `sprig check`, and writes a job summary: per-file progress on base and head, and items newly unblocked.

## Acceptance criteria

- [ ] This repository's CI uses the action.
- [ ] The summary shows done counts for base and head and lists newly unblocked items, verified on a test pull request.
