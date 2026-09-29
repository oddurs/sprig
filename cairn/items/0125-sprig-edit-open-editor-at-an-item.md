---
id: 125
uid: ce322d35-5e84-4144-923b-75074a8293ed
title: 'sprig edit: open $EDITOR at an item'
type: feature
status: planned
milestone: v0.3
depends_on:
- 118
created: 2026-09-28
updated: 2026-09-28
priority: p2
effort: s
area: cli
---

## Problem

The verbs cover common writes, not every write. The fallback should put the cursor on the right line, not make someone search for it.

## Proposal

`sprig edit <ref>` runs `$VISUAL`, then `$EDITOR`, then `vi`, with `+LINE FILE`, the form vi, Vim, Neovim, Emacs, nano, Helix and Kakoune all accept. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] A test with `EDITOR` set to a stub script checks the arguments it receives.
- [ ] Outside a terminal, the command exits 2 instead of launching an editor.
