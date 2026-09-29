---
id: 47
uid: a84b3658-1551-4781-93ea-4533e4e66b3d
title: Tree-sitter grammar
type: feature
status: planned
milestone: v0.2
depends_on:
- 7
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: editors
---

## Problem

Neovim, Helix and Zed highlight through tree-sitter. Without a grammar, Sprig is plain text there.

## Proposal

`editors/tree-sitter-sprig`, a flat, line-level grammar: marks, tokens, links, notes, comments and escapes, with no indentation tracking. Nesting comes from the language server, which avoids writing an external scanner in C. Highlight queries ship with it.

## Acceptance criteria

- [ ] Every conformance input parses without ERROR nodes.
- [ ] `tree-sitter test` passes a corpus of at least 30 cases.
- [ ] `tree-sitter highlight` output for `examples/bakery/` matches a committed snapshot.
