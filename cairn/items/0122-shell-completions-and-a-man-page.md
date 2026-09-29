---
id: 122
uid: 0943198d-2bc1-4f8b-9739-2a996026f92a
title: Shell completions and a man page
type: feature
status: planned
milestone: v0.2
depends_on:
- 22
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: cli
---

## Problem

A command-line tool without completion and a man page feels unfinished to exactly the people most likely to adopt it.

## Proposal

`sprig completions bash|zsh|fish|powershell` and `sprig man`, generated from the argument definitions with `clap_complete` and `clap_mangen`, as cairn does. Completion for refs offers anchors and `@people` from the current folder through a hidden `sprig __complete` command. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] Each shell's script is generated in CI, and a test loads the bash and zsh scripts without error.
- [ ] Typing `sprig tick ^` and pressing Tab in zsh offers the anchors in `examples/bakery/`; checked by hand and noted here.
- [ ] `sprig man | man -l -` renders every command and the exit codes.
