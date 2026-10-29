---
id: 54
uid: 32966bc6-7787-45d7-bbec-10fa34d23dc6
title: VS Code extension
type: feature
status: doing
milestone: v0.2
depends_on:
- 6
- 48
- 50
- 53
created: 2026-09-28
updated: 2026-10-28
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
- [x] A CI smoke test with `@vscode/test-electron` opens `examples/bakery/` and asserts the expected diagnostics count.
- [x] Works with no network access and no separate install.

## 2026-10-28

`editors/vscode` registers `.sprig`, ships the TextMate grammar and starts `sprig lsp` from the binary bundled in `server/`. It falls back to `sprig.server.path`, then to `sprig` on PATH.

- **Criterion 3:** `npm test` runs `@vscode/test-electron` on a copy of `examples/bakery/`, with every other extension disabled. It asserts that the folder checks clean, then deletes `^signed` and asserts exactly `bakery.sprig:15 S0203` and `kitchen.sprig:18 S0203`. CI runs it under xvfb in the `editors` job.
- **Criterion 4:** the server is bundled, so nothing else is needed and nothing is fetched.
- **Criterion 2:** `.github/workflows/vscode.yml` packages one VSIX per target (darwin-arm64, darwin-x64, linux-x64, linux-arm64 and win32-x64), each with that target's release binary. It is ticked once the workflow's first run on main succeeds.
- **Criterion 1** waits on the name (0006): nothing is published to the Marketplace until it is cleared.
