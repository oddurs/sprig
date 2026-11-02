---
id: 156
uid: c4c1b3eb-cb7d-42b1-acee-368447e17d8b
title: No tool-named files in the tree, and a green audit
type: chore
status: doing
milestone: v1.0
assignee: Oddur Sigurdsson
claimed: 2026-11-02
created: 2026-11-02
updated: 2026-11-02
priority: p1
effort: s
area: release
---

## Problem

The repository is public and published under one name, yet the agent contract is tracked as a file named after one coding tool, and a stray head branch on GitHub carries a tool's name. Neither says anything about the project; both read as attribution. Separately, the weekly audit is red on main: GHSA-g3hv-75wg-m249 (minimatch below 11.0.1) reaches the editors' and the VS Code extension's lockfiles through vscode-tmgrammar-test, @vscode/vsce and vscode-languageclient, none of which has released a fix.

## Approach

Move the contract to AGENTS.md, which every coding agent reads, and keep any tool's own file local to the clone through .git/info/exclude, never .gitignore. scripts/agent start links whatever the primary checkout excludes into a new worktree, so a local shim follows the work without the tracked tree naming it. Lint refuses a tracked path named after a tool. Pin minimatch to a fixed release with npm overrides, and prove the consumers still work by running the editors' own tests.

## Acceptance criteria

- [x] AGENTS.md carries the contract, and no tracked path is named after a coding tool; `scripts/task lint` fails if one is added.
- [x] `scripts/agent start` links each path listed in .git/info/exclude that exists in the primary checkout into the new worktree.
- [x] `scripts/task audit` passes on every lockfile.
- [ ] `scripts/task editors` passes with the override in place.

## 2026-11-02

GHSA-g3hv-75wg-m249 has no fix in any consumer, so both editor lockfiles pin minimatch 11.0.4 through overrides. vscode-languageclient and @vscode/vsce only construct Minimatch and work unchanged. glob 7, under vscode-tmgrammar-test, does not: its pattern walk reads a field minimatch 11 removed, and the snapshot run crashed. glob 14 would fix that but has no default export, which is how vscode-tmgrammar-test imports it. So test:textmate now lets the shell expand tests/textmate/*.sprig, and glob only ever sees literal paths, which take the branch that never reads the field. A corrupted snapshot still exits 255. The link_local test found that a directory listed with a trailing slash in .git/info/exclude does not match the link that stands in for it in a worktree; the comment and AGENTS.md say to list directories without one.
