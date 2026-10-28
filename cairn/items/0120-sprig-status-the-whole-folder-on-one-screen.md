---
id: 120
uid: 1eaf1d5b-13a6-4d93-8689-996fa5b4939f
title: 'sprig status: the whole folder on one screen'
type: feature
status: done
milestone: v0.2
depends_on:
- 18
- 19
- 119
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: cli
---

## Problem

`tree` shows one file and `next` shows what's ready. Neither says how the whole folder is doing, and nothing fits in a shell prompt or a tmux status bar.

## Proposal

One line per file: a progress bar, done/total, how many items wait on something, how many are late. `--oneline` prints a single short line for a status bar, such as `sprig 23% · 21 ready · 11 waiting`. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [x] Output for `examples/bakery/` with `--today 2026-10-01` matches a committed snapshot.
- [x] `--oneline` never prints more than 60 characters, and exits 1 when anything is late so a prompt can colour it.
- [x] A file that fails to parse still gets a line, with its error count, instead of aborting the command.

## 2026-10-28

`sprig status [PATH...] [--oneline]`. One row per file: a ten-cell bar, done/total, ready, waiting, late and error counts, with `--plain` as in `spec/tools.md` T4. (1) `status_for_the_bakery_matches_its_snapshot` (`tests/snapshots/status-bakery.txt`, `--today 2026-10-01`). (2) `--oneline` covers the plans no file grafts whole, deduplicated as `next` does: `sprig 21% · 21 ready · 12 waiting` on 2026-10-01. It is cut to 60 characters and exits 1 when anything is late, tested on 2026-11-20. (3) The core never aborts on a file, so a file with problems keeps its line, showing its error count (a file with two missing grafts shows "2 errors").

## Result

`sprig status` for the whole folder, and a one-line form for prompts that signals lateness by exit code.
