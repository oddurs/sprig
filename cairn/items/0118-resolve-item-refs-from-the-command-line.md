---
id: 118
uid: 5c418391-f5d5-4891-b5fb-1ef45d87709e
title: Resolve item refs from the command line
type: feature
status: done
milestone: v0.2
depends_on:
- 17
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: core
---

## Problem

Every command that acts on one item needs to name it, and the CLI, the TUI and the MCP server must agree on what a name means. Today only `sprig tick` (0046) describes a form, and only in its own words.

## Proposal

One resolver in sprig-core, with no I/O, that takes a ref string and a resolved workspace and returns an item, or a diagnostic listing candidates. Forms, from exact to forgiving:

1. `^anchor`, searched across the workspace.
2. `file^anchor`, in one file.
3. `file.sprig:12`, a line, as compilers print it.
4. Words from the item's text, if they match exactly one item.

The CLI adds `-`, which reads one ref per line from stdin. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [x] Each form has unit tests over `examples/bakery/`, including a ref that points into a grafted branch and resolves to the source file's line.
- [x] An ambiguous ref returns every candidate as `file:line  text`, and a test covers an anchor defined in two files.
- [x] A text ref never matches notes or answers, only item text.
- [x] The resolver is documented in `spec/` as tool behaviour, not format, so other implementations can match it.

## 2026-10-28

`sprig_core::refs::resolve`: `^anchor`, `file^anchor`, `file:12`, then words (case-insensitive, an exact text match beating partial ones). A file part may drop `.sprig`, or give a bare file name when it is unique. (1) Unit tests over `examples/bakery/` cover each form. `third quote` resolves to `kitchen.sprig:11`, the line that writes it, although dana.sprig grafts that branch. (2) An ambiguous ref lists every candidate as `file:line  text`. `sack` gives the three grocery lines, and an anchor written in both `a.sprig` and `b.sprig` gives both and resolves as `b^x`. (3) Words never match notes or answers: `mill street` appears in a lease.sprig note and in the name question, and resolves to the question; a phrase only in a note matches nothing. (4) `spec/tools.md` T1 states the resolver as tool behaviour, with Taskwarrior's renumbering IDs as the reason anchors are the stable form and refs are resolved before any write.

## Result

One ref resolver in the core, specified in spec/tools.md for other implementations.
