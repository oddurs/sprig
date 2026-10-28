---
id: 113
uid: ae3ed524-5793-42ec-ad1a-e1bc22e93d6e
title: Extend the first-plan tutorial to the terminal
type: docs
status: done
milestone: launch
depends_on:
- 27
- 106
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: site
---

## Reader and question

A reader who finished the playground tutorial and wants the same plan checked by `sprig next` and `sprig check`.

## Change

Add the terminal steps to the tutorial once v0.1 exists: install, `sprig next`, `sprig check`, and a CI one-liner.

## Acceptance criteria

- [x] Every command in the tutorial is run in CI against the tutorial's own plan.

## 2026-10-28

Step 7 of `site/src/content/docs/docs/first-plan.mdx`: install, `sprig next` on step 6's two files, `sprig check` clean and then with a misspelt graft, and the GitHub Actions line. (1) `crates/sprig-cli/tests/readme.rs` now runs every `sprig` command in both the README and this tutorial, against the files the page itself defines (marked `{/* file: ... */}` in MDX), and requires the exact output shown. It runs in the `check` job on three platforms.

## Result

The tutorial ends in the terminal, and CI holds every command in it to its printed output.
