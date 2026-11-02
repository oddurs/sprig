---
id: 158
uid: b7a2ddc9-4fe4-4d8d-94c2-49d51f16b6b0
title: Rename Sprig to Chea
type: chore
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-02
depends_on:
- 6
created: 2026-11-02
updated: 2026-11-02
priority: p0
effort: l
area: release
---

## Problem

Spike 0006 found the name Sprig unusable: a live software trademark application, a new programming language called Sprig with a `sprig` binary and a `sprig check` command, and the plain name taken on crates.io and npm. Nothing is published yet, so a rename costs the least it ever will.

## Approach

Chea is Calenha for the flood tide, what is coming: the word the language uses to mark the future, and what a plan is a record of. Rename everything at once, case-preserving: the crates, the binary, the `.chea` extension and `PLAN.chea`, the reserved `chea:` title key, the environment variables, the editors, the site, the spec and the backlog. The diagnostic codes move from `S` to `C` while there is nobody to break. What records history stays as it was: git history, the design drafts in `design/`, and spike 0006, which is about the old name.

## Acceptance criteria

- [x] No tracked path or line names Sprig outside `design/`, spike 0006 and this item.
- [x] The diagnostic codes read `C` and a number everywhere, and the spec says so.
- [ ] `scripts/task check`, `scripts/task editors` and the site build pass.
- [x] The README says what the name means.

## 2026-11-02

One scripted pass, case-preserving, over every tracked file and path outside design/, spike 0006 and this item; the design-draft references stay, since that file keeps its name. What the pass could not see, the suite found: four conformance trees whose byte spans moved because chea is a byte shorter than sprig (re-blessed; only span numbers changed), two TUI and tree snapshots whose padding moved (whitespace only), filters that matched '| S' and, now, the table's own 'Code' header, the JSON schema's code pattern, the site's anchor regex, and a code after a tab escape that \\b could not see. The tree-sitter parser was not regenerated: the current CLI emits ABI 15 against the committed 14, which is drift for its own change. The VS Code extension test cannot start locally, because macOS refuses its socket path under ../.worktrees/; CI runs it.
