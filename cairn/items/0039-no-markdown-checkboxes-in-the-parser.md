---
id: 39
uid: c35050a6-6c9e-4583-a6cf-e19c97ce3d07
title: No Markdown checkboxes in the parser
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

Millions of `- [ ]` lists already exist.

## Options and tradeoffs

Accepting them means two spellings for every status, in every tool, forever.

## Decision

Rejected. `sprig import` converts them once.

## Revisit when

Never; converting is a one-time job.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
