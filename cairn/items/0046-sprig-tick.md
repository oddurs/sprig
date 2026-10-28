---
id: 46
uid: 5da6ef15-135b-43cd-860a-9a46e94cc1bd
title: sprig tick
type: feature
status: done
milestone: v0.2
depends_on:
- 43
- 118
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: cli
---

## Problem

Scripts and people at a terminal need to tick items without opening an editor.

## Proposal

`sprig tick <ref>...`, with refs as 0118 resolves them, flips the mark between open and `x`. Recurring items roll their due date instead. Prints the changed line.

## Acceptance criteria

- [x] Ticking a recurring item moves `due:` to the next occurrence and leaves the mark open, per the spec.
- [x] Only the mark character (or the due date) changes, byte for byte.
- [x] An ambiguous or unknown target exits 2 with the candidates listed.
- [x] `sprig tick -` reads refs from stdin, one per line, and ticks them all or none.

## 2026-10-28

`sprig tick [--in PATH] REF...`, through `edit::tick`. (1) `weekly numbers` on 2026-10-02 moves `due:2026-10-02` to `due:2026-10-09` and keeps `-` (tested on the file). (2) Ticking `lease^signed` and `grease trap` changes exactly two bytes of lease.sprig, both to `x`; the file length is unchanged (byte comparison in the test). (3) An unknown or ambiguous ref exits 2, prints every candidate, and changes no file. (4) `sprig tick -` reads refs from standard input, one per line, skipping blank lines. All refs resolve before anything is written; files are replaced by write-then-rename; each changed line is printed as `file:line  text`, in the order the refs were given. The pipe from `sprig next --plain | cut -f1` waits on `--plain` (0119).

## Result

`sprig tick` with every ref form, all-or-nothing, changing only a mark or a due date.
