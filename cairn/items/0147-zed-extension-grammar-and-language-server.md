---
id: 147
uid: aab0cb46-6394-46a2-9389-33bcaea60c65
title: 'Zed extension: grammar and language server'
type: feature
status: backlog
milestone: later
created: 2026-10-28
updated: 2026-10-28
priority: p2
area: editors
---

## Problem

Zed reaches a tree-sitter grammar and a language server only through an extension, so the editor guide (0055) has no Zed setup that works.

## Proposal

A Zed extension in `editors/zed`. Its `extension.toml` names the tree-sitter grammar in `editors/tree-sitter-sprig`, its `languages/sprig/` holds the language config and highlight queries, and a small Rust extension, built with `zed_extension_api` for `wasm32-wasip1`, starts `sprig lsp` from `PATH`.

## Acceptance criteria

- [ ] Installed as a dev extension, Zed highlights `examples/bakery/` and shows the problems the language server reports.
