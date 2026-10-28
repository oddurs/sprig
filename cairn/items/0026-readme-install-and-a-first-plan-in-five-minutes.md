---
id: 26
uid: 06500888-c182-4414-be0e-6c7ff4ede6af
title: 'README: install and a first plan in five minutes'
type: docs
status: blocked
milestone: v0.1
depends_on:
- 23
- 25
created: 2026-09-28
updated: 2026-10-28
priority: p0
effort: s
area: docs
---

## Reader and question

A developer who has heard of Sprig and wants to know in five minutes whether it's for them.

## Change

What Sprig is in two sentences; install; a ten-line plan; `sprig next` and `sprig check` on it; the nine line starts with one example each; links to the spec and `examples/bakery/`.

## Acceptance criteria

- [ ] Someone who hasn't seen the project follows it from nothing to `sprig next` output in under five minutes; their time and any stumbles are noted on this item.
- [x] Every command in the README is run in CI so it can't go stale.

## 2026-10-28

README rewritten to the brief: two sentences on what Sprig is; install; a ten-line plan; `sprig next` and `sprig check` on it, including a misspelt anchor; the nine line starts with an example each; links to the spec, `examples/bakery/`, the conformance suite and the site. (2) `crates/sprig-cli/tests/readme.rs` runs every `sprig` command in the README and requires the exact output shown. It runs under `scripts/task test` on all three CI platforms, and fails when the README is wrong (checked by changing one number). The install line is covered separately by the new `install` CI job, on three platforms, through `scripts/task install`. (1) is still open and needs a person: someone who has not seen the project follows the README, and their time and stumbles are noted here. Blocked on that. The install line installs from git until the crate name is settled (0006) and published (0027).
