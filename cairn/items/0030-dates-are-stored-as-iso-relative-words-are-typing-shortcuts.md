---
id: 30
uid: e9baa9a9-b8a4-43ef-8185-2ecf75f34eb1
title: Dates are stored as ISO; relative words are typing shortcuts
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

`due:fri` is quick to type, but a week later the same file means something else.

## Options and tradeoffs

Allow relative dates (fast, but the file's meaning changes with the day it's read) or require ISO (unambiguous, slower to type).

## Decision

Store ISO. Editors and `sprig fmt` expand `fri`, `+2w`, `oct3` and `tomorrow` when you leave the line. Parsers still read them against today as a courtesy. The 0.1 special case where `due:oct 3` swallowed the next word is removed.

## Revisit when

If editors fail to expand shortcuts in practice and plain-text edits leave relative dates behind.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
