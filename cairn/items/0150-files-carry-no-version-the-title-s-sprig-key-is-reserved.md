---
id: 150
uid: 167b53e0-6370-4a31-bde9-260d2a56b043
title: 'Files carry no version; the title''s sprig: key is reserved'
type: decision
status: done
created: 2026-10-29
updated: 2026-10-29
priority: p2
area: spec
---

## Context

Spike 0070 asked whether a file should declare which version of the spec it follows. RFC 0003 (`rfcs/0003-no-version-in-files.md`) records the argument.

## Options and tradeoffs

A declared version lets a tool know what a file expects. It also gives every file a line that means nothing until a breaking change, and invites tools to refuse files over it. CommonMark never adopted a per-document version, and TOML, JSON and todo.txt carry none. Sprig's grammar already degrades safely: an unknown key is an ordinary field and an unknown mark is a note.

## Decision

A file needs no version. Every 1.x file means the same to every 1.x tool. The title's `sprig:` key is reserved for a future major version (§4.6). `sprig:1` changes nothing, and a tool that meets a version it does not read warns with S0007 and reads the file as best it can.

## Revisit when

A 2.0 is needed; then the reserved key is how a 1.x tool finds out.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
