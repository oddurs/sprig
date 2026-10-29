---
id: 55
uid: a2bc984e-25b7-495c-ba13-848b673d7777
title: 'Editor setup guide: Neovim, Helix, Zed'
type: docs
status: doing
milestone: v0.2
depends_on:
- 47
- 49
created: 2026-09-28
updated: 2026-10-28
priority: p1
effort: s
area: docs
---

## Reader and question

A developer outside VS Code who wants highlighting and diagnostics for `.sprig` files.

## Change

`docs/editors.md` with the configuration each editor needs for the tree-sitter grammar and the language server.

## Acceptance criteria

- [x] Each editor's section was followed from scratch on a clean config, and diagnostics appeared.

## 2026-10-28

`site/src/content/docs/docs/editors.md` covers VS Code, Neovim, Helix, Zed and any other LSP editor.

- **VS Code:** followed by the extension's CI test.
- **Neovim:** followed from scratch on a clean config by `editors/neovim/test.lua`. It loads `editors/neovim/sprig.lua` and the parser, opens `examples/bakery/lease.sprig`, deletes `^signed`, and gets the two problems. It runs in the `editors` CI job with Ubuntu's Neovim 0.9.
- **Helix:** written from Helix's documentation but not followed. There is no Helix binary here, and GitHub, where its releases live, is unreachable from this machine.
- **Zed:** reaches a grammar and a language server only through an extension, and there isn't one (0147), so the guide says so.

The criterion stays open until someone follows the Helix section on a clean config.
