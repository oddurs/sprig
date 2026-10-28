---
id: 22
uid: 68a0b0c1-3d45-4e61-b4da-910e932e7894
title: 'CLI skeleton: help, version, exit codes'
type: feature
status: done
milestone: v0.1
depends_on:
- 8
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: s
area: cli
---

## Problem

Every command needs the same conventions, and they are cheapest to set before the second command exists.

## Proposal

`sprig` with subcommands, `--help`, `--version`, errors on stderr, and documented exit codes: 0 fine, 1 findings, 2 usage or I/O error. Arguments are files or directories; a directory means every `.sprig` file under it. `--today YYYY-MM-DD` is global and defaults to the system date: it is the only place the CLI reads the clock, so every output can be reproduced.

## Acceptance criteria

- [x] `sprig --help` lists each command with one line of description.
- [x] The exit codes are listed in `--help` and tested.
- [x] An unreadable path exits 2 with the path in the message, covered by a test.
- [x] `--today` is accepted by every command, and no code outside the argument parser reads the system date; a lint or test enforces it.

## 2026-10-28

clap with derive (`crates/sprig-cli/src/args.rs`). Arguments are files or directories; a directory means every `.sprig` file under it, hidden directories skipped. Grafted and referenced files are then loaded on demand through `Workspace::wanted()`, and never through a symlink that leaves the root. (1) `--help` lists `next` and `check`, one line each, tested. (2) The exit codes (0 fine, 1 findings or nothing ready, 2 usage or I/O) are in `--help`, and each is exercised by `tests/cli.rs`. (3) A missing path exits 2 with the path in the message, for both commands; a non-UTF-8 file exits 2 too. (4) `--today` is global, also read from `SPRIG_TODAY`, and tested on every command in either position. A bad date exits 2. `scripts/task lint` now fails on any clock read (`SystemTime`, `Instant::now`, `Zoned::now` and the like) outside `args.rs`; this was checked by adding one to `main.rs`. The local date comes from `jiff`. Running `sprig` with no command prints usage and exits 2.

## Result

The `sprig` command with `next` and `check`, documented exit codes, and the system date read in exactly one place.
