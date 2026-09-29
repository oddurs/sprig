---
id: 103
uid: 4dd7a1f2-b3cb-4224-a76d-0ecc0f6c1ff1
title: Spec page with an anchor per rule, and a raw copy
type: feature
status: done
milestone: launch
depends_on:
- 7
- 99
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p0
effort: s
area: site
---

## Problem

The spec is the page Hacker News readers open second, and it must never drift from `spec/sprig.md`.

## Proposal

The build copies `spec/sprig.md` into the docs as `/spec/`, with one heading per rule so each rule has a stable anchor, and publishes the raw file at `/spec.md`.

## Acceptance criteria

- [x] `/spec/` is generated at build time from `spec/sprig.md`; nothing in `site/` repeats its text.
- [x] Every numbered rule has its own anchor.
- [x] `/spec.md` serves the file byte for byte.

## 2026-09-28

site/scripts/sync.mjs writes /spec/ from spec/sprig.md at build time (the generated page is gitignored), and copies the file to /spec.md; cmp confirms the served copy is byte-identical. The built page has 53 rule anchors, one per numbered rule.

## Result

/spec/ is generated from spec/sprig.md with one anchor per rule, and /spec.md serves the source byte for byte.
