---
id: 105
uid: 4cc7985e-8f16-44d8-8f56-5797cdfb6a96
title: Decisions, roadmap and changelog pages generated from the repository
type: feature
status: done
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p1
effort: s
area: site
---

## Problem

Visible reasoning and a public plan earn trust, but only if they can't go stale.

## Proposal

At build time, generate `/decisions/` from the cairn decision items, `/roadmap/` from `ROADMAP.md`, and `/changelog/` plus `/feed.xml` from `CHANGELOG.md`.

## Acceptance criteria

- [x] Each page is generated at build time, and none of their text is stored in `site/`.
- [x] Every decision item appears with its context, options, decision and when to revisit it.
- [x] `/feed.xml` is valid Atom.

## 2026-09-28

sync.mjs generates /decisions/ from the 14 decision items (context, options, decision, revisit when, and a link to each item), /roadmap/ from ROADMAP.md and /changelog/ from CHANGELOG.md; all generated files are gitignored. /feed.xml is an Atom feed of released versions (none yet); it parses as XML with the Atom namespace. It was not run through a feed validator.

## Result

Decisions, roadmap, changelog and an Atom feed generated from the repository at build time.
