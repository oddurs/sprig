---
id: 123
uid: be1dc944-f9fa-4d78-be25-8c89d5eea1bb
title: sprig mark and sprig answer
type: feature
status: planned
milestone: v0.3
depends_on:
- 43
- 46
- 118
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: cli
---

## Problem

`sprig tick` only toggles between open and done. Starting an item, parking it for later and settling a question each still need an editor.

## Proposal

`sprig mark <ref> <- ~ x / > ?>` sets any status mark. `sprig answer <ref> "text"` adds the `= text` line under a question and prints what it unblocked. Both go through the core's edit API (0043), the same calls the MCP write tools make (0060). The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] Each command changes only the bytes it means to, checked by the edit API's property test.
- [ ] `sprig answer ^name "Kiln & Crumb"` on `examples/bakery/` prints Window lettering as newly unblocked, covered by a snapshot.
- [ ] Marking a group or a graft exits 2 and explains that they take their status from their children.
