---
id: 73
uid: 195fa738-5f80-473c-9d1d-c5c2aa2e655a
title: Stability policy
type: docs
status: planned
milestone: v1.0
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: community
---

## Reader and question

Anyone deciding whether to build on Sprig, and needing to know what won't change under them.

## Change

`docs/stability.md`: a file valid under 1.0 parses the same way forever; spec changes after 1.0 are additive; removing anything needs spec 2.0; diagnostic codes are never reused; crates and the CLI follow semver.

## Acceptance criteria

- [ ] Linked from the README, the spec and the crate documentation.
- [ ] Every promise can be tested, and says what test holds it.
