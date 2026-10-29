---
id: 81
uid: 1135c3ad-df86-4a2e-a180-eaa3d21c3c16
title: Five-minute tutorial and reference site
type: docs
status: doing
milestone: v1.0
depends_on:
- 77
- 78
created: 2026-09-28
updated: 2026-10-29
priority: p0
effort: m
area: docs
---

## Reader and question

A stranger deciding in five minutes whether Sprig fits their project, then looking up a command later.

## Change

A tutorial that ends with the reader's own `PLAN.sprig` checked, ticked from the editor and read by an agent. The reference covers every CLI command, MCP tool and diagnostic code, generated from the same sources the tools use.

## Acceptance criteria

- [ ] Two people new to Sprig complete the tutorial in under five minutes, with times noted here.
- [x] The reference is generated, and CI fails if a command or diagnostic is undocumented.

## 2026-10-29

The reference covers every command (the binary's help), MCP tool (the server's tools/list) and diagnostic (spec/diagnostics.md), all generated at site build. CI fails on a command or option without help (args.rs every_command_and_option_has_help), and on a code without a catalogue row or a note (catalogue.rs). The tutorial now ends with ticking from the terminal or editor and writing AGENTS.md for an agent. Timing it with two newcomers has to be done by people.
