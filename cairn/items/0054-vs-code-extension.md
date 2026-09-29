---
id: 54
uid: 32966bc6-7787-45d7-bbec-10fa34d23dc6
title: VS Code extension
type: feature
status: planned
milestone: v0.2
depends_on:
- 6
- 48
- 50
- 53
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: editors
---

## Problem

VS Code is where most developers will first meet a `.sprig` file.

## Proposal

An extension that registers the language, ships the TextMate grammar, and bundles the language server binary per platform (separate VSIX files per target), so it works offline with nothing else installed.

## Acceptance criteria

- [ ] Published to the Marketplace under the name the spike cleared.
- [ ] Targets macOS arm64 and x64, Linux x64 and arm64, and Windows x64.
- [ ] A CI smoke test with `@vscode/test-electron` opens `examples/bakery/` and asserts the expected diagnostics count.
- [ ] Works with no network access and no separate install.
