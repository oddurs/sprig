---
id: 124
uid: 80803fac-e571-4026-a708-4212c1283ed6
title: 'sprig add: append an item from the shell'
type: feature
status: done
milestone: v0.3
depends_on:
- 43
- 118
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p1
effort: s
area: cli
---

## Problem

Capturing a thought from the terminal means opening an editor, finding the right parent and matching its indentation.

## Proposal

`sprig add "text" --under <ref>` appends a child as the parent's last item, and `--after <ref>` adds a sibling. The indentation matches the file. Tokens in the text are parsed as usual, and typed dates expand to ISO. Without `--under` or `--after`, the item goes at the end of `PLAN.sprig`, the convention from 0062. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [x] Adding under a parent whose children use tabs, or four spaces, matches that indentation; a test covers both.
- [x] `sprig add "Call the plumber due:fri" --under ^design --today 2026-10-01` writes `due:2026-10-02`.
- [x] The command prints the new line with its `file:line`.

## Result

sprig add puts an item under or after another ref, indented like its siblings (tabs and four spaces both tested), writes typed dates out, and prints file:line. With no ref it appends to the folder's PLAN.sprig, or exits 2 when there is none.
