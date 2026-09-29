---
id: 124
uid: 80803fac-e571-4026-a708-4212c1283ed6
title: 'sprig add: append an item from the shell'
type: feature
status: planned
milestone: v0.3
depends_on:
- 43
- 118
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: cli
---

## Problem

Capturing a thought from the terminal means opening an editor, finding the right parent and matching its indentation.

## Proposal

`sprig add "text" --under <ref>` appends a child as the parent's last item, and `--after <ref>` adds a sibling. The indentation matches the file. Tokens in the text are parsed as usual, and typed dates expand to ISO. Without `--under` or `--after`, the item goes at the end of `PLAN.sprig`, the convention from 0062. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] Adding under a parent whose children use tabs, or four spaces, matches that indentation; a test covers both.
- [ ] `sprig add "Call the plumber due:fri" --under ^design --today 2026-10-01` writes `due:2026-10-02`.
- [ ] The command prints the new line with its `file:line`.
