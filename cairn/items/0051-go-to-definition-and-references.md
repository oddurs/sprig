---
id: 51
uid: 9348f150-ced4-40ef-abf5-842229ea2879
title: Go to definition and references
type: feature
status: done
milestone: v0.2
depends_on:
- 49
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: lsp
---

## Problem

Following `after:[[lease^signed]]` by hand means opening a file and searching.

## Proposal

Definition on links and anchors, and references on an anchor: every `after:` and link that names it, across the workspace.

## Acceptance criteria

- [x] Definition on `[[lease^signed]]` opens `lease.sprig` at that item.
- [x] References on `^signed` list every line in the workspace that names it.

## 2026-10-28

Definition works on any link, `after:` value or anchor; it lands on the item, or on the title for a whole file. References scan every `after:` and link in the workspace that names the same file and anchor; notes are not references (§2.6), so `bakery.sprig:14` is not listed. The tests are `definition_on_a_link_opens_the_item_it_names` and `references_on_an_anchor_list_every_line_that_names_it`.

## Result

Go to definition and find references across files, for anchors and links.
