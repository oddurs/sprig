---
id: 48
uid: d1cf82cf-272b-490f-ba48-70dbc48e2900
title: TextMate grammar
type: feature
status: planned
milestone: v0.2
depends_on:
- 7
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: editors
---

## Problem

VS Code highlights with TextMate grammars, and GitHub's Linguist does too.

## Proposal

`editors/sprig.tmLanguage.json`, with the same scopes as the tree-sitter highlight queries.

## Acceptance criteria

- [ ] `vscode-tmgrammar-test` snapshot tests cover every token kind.
- [ ] Scope names follow TextMate conventions so existing themes colour them.

## 2026-09-28

A first TextMate grammar exists at editors/sprig.tmLanguage.json (from the website work, 0099); the site uses it to highlight sprig code blocks. It has no vscode-tmgrammar-test snapshots yet, which this item still requires.
