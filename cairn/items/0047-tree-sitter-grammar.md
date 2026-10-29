---
id: 47
uid: a84b3658-1551-4781-93ea-4533e4e66b3d
title: Tree-sitter grammar
type: feature
status: done
milestone: v0.2
depends_on:
- 7
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: editors
---

## Problem

Neovim, Helix and Zed highlight through tree-sitter. Without a grammar, Sprig is plain text there.

## Proposal

`editors/tree-sitter-sprig`, a flat, line-level grammar: marks, tokens, links, notes, comments and escapes, with no indentation tracking. Nesting comes from the language server, which avoids writing an external scanner in C. Highlight queries ship with it.

## Acceptance criteria

- [x] Every conformance input parses without ERROR nodes.
- [x] `tree-sitter test` passes a corpus of at least 30 cases.
- [x] `tree-sitter highlight` output for `examples/bakery/` matches a committed snapshot.

## 2026-10-28

`editors/tree-sitter-sprig` is a line-level grammar with no external scanner. Lexical conflicts are resolved the way the core resolves them:
- **Precedence:** a mark beats note text, and a bare mark takes its line ending, so the `x` of `x-ray` is not a mark.
- **Longest match:** `@jo,` is a word, not a person.
- **Rule order:** when two matches are equally long, the token defined first wins, so `@jo` is a person.

The grammar is generated for ABI 14, so it loads in Neovim 0.9 and every newer host.

`tests/agrees_with_core.rs` holds it to the core. It parses every input in the conformance suite and `examples/` (226 files) and compares each line's kind, and each item's, answer's and title's tokens, with `sprig_core::syntax::lex`. There are no differences and no ERROR nodes.

1. No conformance input parses with an ERROR node: `no_input_parses_with_an_error`.
2. `tree-sitter test` passes 40 corpus cases, drawn from the lines and tokens suites.
3. `test/highlight.sh` compares `tree-sitter highlight` of `examples/bakery/` with `test/snapshot/bakery.txt`.

## Result

A tree-sitter grammar that classifies every line and token exactly as the core does, with highlight queries, a 40-case corpus and a highlight snapshot.
