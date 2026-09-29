---
id: 22
uid: 68a0b0c1-3d45-4e61-b4da-910e932e7894
title: 'CLI skeleton: help, version, exit codes'
type: feature
status: planned
milestone: v0.1
depends_on:
- 8
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: cli
---

## Problem

Every command needs the same conventions, and they are cheapest to set before the second command exists.

## Proposal

`sprig` with subcommands, `--help`, `--version`, errors on stderr, and documented exit codes: 0 fine, 1 findings, 2 usage or I/O error. Arguments are files or directories; a directory means every `.sprig` file under it. `--today YYYY-MM-DD` is global and defaults to the system date: it is the only place the CLI reads the clock, so every output can be reproduced.

## Acceptance criteria

- [ ] `sprig --help` lists each command with one line of description.
- [ ] The exit codes are listed in `--help` and tested.
- [ ] An unreadable path exits 2 with the path in the message, covered by a test.
- [ ] `--today` is accepted by every command, and no code outside the argument parser reads the system date; a lint or test enforces it.
