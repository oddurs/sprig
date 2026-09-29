---
id: 103
uid: 4dd7a1f2-b3cb-4224-a76d-0ecc0f6c1ff1
title: Spec page with an anchor per rule, and a raw copy
type: feature
status: planned
milestone: launch
depends_on:
- 7
- 99
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: site
---

## Problem

The spec is the page Hacker News readers open second, and it must never drift from `spec/sprig.md`.

## Proposal

The build copies `spec/sprig.md` into the docs as `/spec/`, with one heading per rule so each rule has a stable anchor, and publishes the raw file at `/spec.md`.

## Acceptance criteria

- [ ] `/spec/` is generated at build time from `spec/sprig.md`; nothing in `site/` repeats its text.
- [ ] Every numbered rule has its own anchor.
- [ ] `/spec.md` serves the file byte for byte.
