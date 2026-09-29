---
id: 118
uid: 5c418391-f5d5-4891-b5fb-1ef45d87709e
title: Resolve item refs from the command line
type: feature
status: planned
milestone: v0.2
depends_on:
- 17
created: 2026-09-28
updated: 2026-09-28
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

- [ ] Each form has unit tests over `examples/bakery/`, including a ref that points into a grafted branch and resolves to the source file's line.
- [ ] An ambiguous ref returns every candidate as `file:line  text`, and a test covers an anchor defined in two files.
- [ ] A text ref never matches notes or answers, only item text.
- [ ] The resolver is documented in `spec/` as tool behaviour, not format, so other implementations can match it.
