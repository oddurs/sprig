---
id: 105
uid: 4cc7985e-8f16-44d8-8f56-5797cdfb6a96
title: Decisions, roadmap and changelog pages generated from the repository
type: feature
status: planned
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: site
---

## Problem

Visible reasoning and a public plan earn trust, but only if they can't go stale.

## Proposal

At build time, generate `/decisions/` from the cairn decision items, `/roadmap/` from `ROADMAP.md`, and `/changelog/` plus `/feed.xml` from `CHANGELOG.md`.

## Acceptance criteria

- [ ] Each page is generated at build time, and none of their text is stored in `site/`.
- [ ] Every decision item appears with its context, options, decision and when to revisit it.
- [ ] `/feed.xml` is valid Atom.
