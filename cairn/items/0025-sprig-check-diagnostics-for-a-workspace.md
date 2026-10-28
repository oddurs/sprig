---
id: 25
uid: 7c9f83e3-ddda-4257-aa12-81c72685e803
title: 'sprig check: diagnostics for a workspace'
type: feature
status: done
milestone: v0.1
depends_on:
- 12
- 19
- 22
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: s
area: cli
---

## Problem

Broken links, loops and unknown anchors should fail CI, not surprise someone reading the plan.

## Proposal

Parse and resolve every file given, print diagnostics in the core's format, and exit 1 on any error. Warnings exit 0 unless `--strict`.

## Acceptance criteria

- [x] Output lines are `file:line:col: severity[code]: message`.
- [x] `--strict` turns warnings into a non-zero exit.
- [x] `examples/bakery/` checks clean; a copy with a broken `[[lease^signd]]` link exits 1 naming the file and line.
- [x] The README shows the one line needed to run it in GitHub Actions.

## 2026-10-28

(1) Each problem is one line on stdout, `file:line:col: severity[code]: message`, with the path as the user typed it and `/` on every platform. The summary goes to stderr, so the output can be piped. Tested with an error, a warning and a reference in one file. (2) `--strict` makes a warning exit 1; without it a warning exits 0. (3) `examples/bakery/` checks clean: "6 files checked, no problems", exit 0. A copy with `after:[[lease^signd]]` exits 1 with `kitchen.sprig:18:25: error[S0203]: no item ^signd in lease.sprig`, whether the folder or just kitchen.sprig is checked. (4) The README shows the GitHub Actions step: `cargo install --locked --git https://github.com/oddurs/sprig sprig-cli && sprig check --strict .`.

## Result

`sprig check` prints compiler-style diagnostics, exits 1 on errors (and warnings with `--strict`), and is a one-line CI step.
