---
id: 116
uid: c981ce5d-217b-4bfd-94a2-56a9718f4fa4
key: v0.4
title: Work the plan from one screen
type: milestone
status: doing
depends_on:
- 3
created: 2026-09-28
updated: 2026-10-29
priority: p2
due: 2027-05-21
---

## Ships

`sprig tui`: the tree, what's next and the agenda for a folder of plans on one terminal screen, with vim keys, writes that go through the same one-line edits as the verbs, and live reload when a file changes underneath it. Plan: `design/cli-plan.html`.

## Done when

- [x] `sprig tui` opens on `examples/bakery/`, and every write it can make leaves `git diff` showing only the lines it meant to change.
- [x] An editor and the TUI can have the same file open; a change saved in one appears in the other without data loss, covered by a test.
- [x] Every view has a committed text snapshot rendered with `--screenshot`.

## Explicitly not in this milestone

- Mouse support, themes or a config file.
- Editing item text inside the TUI; `e` opens `$EDITOR` at the line.
- A second parser or renderer: the TUI uses sprig-core and the CLI's output layer.

## 2026-10-29

Its three conditions hold: the session test checks git diff, the race and reload tests cover an editor and the TUI on one file, and every view has a snapshot. It stays open for 0133's hand check of Vim.
