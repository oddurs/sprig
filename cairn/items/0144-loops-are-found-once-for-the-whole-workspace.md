---
id: 144
uid: df09505d-5ed9-4d0f-9e28-66446e048a4f
title: Loops are found once for the whole workspace
type: decision
status: done
created: 2026-10-28
updated: 2026-10-28
priority: p2
area: spec
---

## Context

§8.4 refuses a graft that would loop, and said only that "one of the two graft lines reports a loop". In the reference parser which line reported it depended on which file was read first (Appendix B.10).

## Options and tradeoffs

Per-tree detection reports the line that closes the loop below whichever file is opened, so `a` shows `b` and `b` shows `a`. The cost is that the same workspace gives different diagnostics depending on where a tool starts, and a file's expansion depends on who grafts it, so it can't be computed once. Workspace-wide detection walks files in path order and grafts in document order, and refuses the one graft that closes each loop everywhere. Every file's expansion is then context-free and every tool reports the same line. The cost is that, in a two-file loop, the file later in path order loses its graft in its own tree too: `a.sprig` shows `b.sprig`, but `b.sprig` does not show `a.sprig`.

## Decision

Loops are found once for the whole workspace, in path order, and the closing graft is refused in every tree (§8.4).

## Revisit when

People find the path-order rule surprising in practice, and a per-tree rule can be made to report one stable line.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
