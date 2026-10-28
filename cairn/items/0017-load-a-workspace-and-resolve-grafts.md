---
id: 17
uid: 42441099-b1f2-4c91-96f0-84d7979762d0
title: Load a workspace and resolve grafts
type: feature
status: done
milestone: v0.1
depends_on:
- 12
- 15
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: core
---

## Problem

`+ [[kitchen]]` and `+ [[kitchen^oven]]` are what make many small plans one big one. Resolution has to be correct, and safe on hostile paths.

## Proposal

The CLI hands the core a set of files by path. The core resolves grafts relative to the grafting file, mounts whole files or single branches, and detects loops and missing targets.

## Acceptance criteria

- [x] A graft to a missing file or anchor produces a diagnostic on the graft line.
- [x] A three-file loop produces one error on the closing graft line and no stack overflow.
- [x] `[[file^id]]` mounts exactly that branch and its notes.
- [x] A graft path that leaves the workspace root (`[[../../etc/passwd]]`) is refused with a diagnostic, covered by a test.

## 2026-10-28

`sprig-core/src/workspace.rs`. The caller adds files by workspace-relative path, and `Workspace::wanted()` names the graft and reference targets still to load, so the CLI reads only what plans name. (1) A missing file is S0101 and a missing anchor S0102, reported at the link's column on the graft line, and the graft mounts nothing. (2) Loops are found once for the whole workspace, by an iterative walk in path order, so a three-file loop is one S0103 on the closing graft line, the same whichever file is opened. A 400-file chain is tested. This settles Appendix B.10 for sprig-core. (3) `[[file^id]]` mounts that item's children, notes and answers, and nothing else; tested, including that the graft takes the branch's owner. (4) `[[../../etc/passwd]]` is S0104, and nothing outside the root is ever asked for (unit test); the CLI test also checks that a secret one level up is never shown and a symlink out of the root is not followed. §8.8, content once per tree, is S0105.

## Result

Workspace resolution with whole-file and branch grafts, deterministic loop detection, and refusal of paths outside the root, at the CLI as well as in the core.
