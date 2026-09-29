---
id: 2
uid: d22fc473-2a7b-42c6-8781-8776665cc1cc
key: v0.2
title: Write plans in your editor
type: milestone
status: planned
depends_on:
- 1
created: 2026-09-28
updated: 2026-09-28
priority: p2
due: 2027-01-29
---

## Ships

VS Code highlights `.sprig` files, completes people, anchors and file links, shows `sprig check` diagnostics as you type, and ticks items with a code action. `sprig fmt` normalises a file without touching anything it doesn't mean to. In the terminal, `sprig tree`, `status` and `why` show a plan's shape and what holds it up, and every command completes in the shell.

## Done when

- [ ] The VS Code extension is on the Marketplace and works offline on macOS, Linux and Windows.
- [ ] Every edit the tools make changes only the bytes it means to, proven by property tests.
- [ ] `sprig fmt --check` runs in this repository's CI.

## Explicitly not in this milestone

- Agent access over MCP (v0.3).
- Obsidian, JetBrains or any editor needing its own plugin (later).
