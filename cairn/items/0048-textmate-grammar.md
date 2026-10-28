---
id: 48
uid: d1cf82cf-272b-490f-ba48-70dbc48e2900
title: TextMate grammar
type: feature
status: done
milestone: v0.2
depends_on:
- 7
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: editors
---

## Problem

VS Code highlights with TextMate grammars, and GitHub's Linguist does too.

## Proposal

`editors/sprig.tmLanguage.json`, with the same scopes as the tree-sitter highlight queries.

## Acceptance criteria

- [x] `vscode-tmgrammar-test` snapshot tests cover every token kind.
- [x] Scope names follow TextMate conventions so existing themes colour them.

## 2026-09-28

A first TextMate grammar exists at editors/sprig.tmLanguage.json (from the website work, 0099); the site uses it to highlight sprig code blocks. It has no vscode-tmgrammar-test snapshots yet, which this item still requires.

## 2026-10-28

`editors/tests/textmate/` holds three fixtures with `vscode-tmgrammar-snap` snapshots, run by `scripts/task editors`. Between them they cover every line kind and every token kind, including the cases the core treats specially:
- an unclosed quote;
- `[[my notes]]`, which is not a link;
- `@jo,`, which is not a person;
- names in any script;
- an escaped title.

The grammar gained a title rule and keys in any script, and links can no longer hold whitespace. Scope names keep to TextMate convention (`keyword`, `comment`, `entity.name.tag`, `variable`, `string`, `markup.*`, `support.type.property-name`), and the site's own highlighting theme keys on the same names.

## Result

The TextMate grammar has snapshot tests for every token kind and conventional scope names.
