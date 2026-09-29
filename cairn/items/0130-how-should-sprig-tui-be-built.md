---
id: 130
uid: e14bfb55-4a64-433e-9683-87515a2517db
title: How should sprig tui be built?
type: spike
status: planned
milestone: v0.4
depends_on:
- 43
- 119
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: cli
---

## Question

What shape does the TUI take so that it is testable without a terminal, shares every write with the CLI, and stays safe when an editor or an agent changes a file it has open?

## Timebox

Two days.

## What would change the answer

- Whether harrow's split holds for a list-heavy UI: a pure core returns actions, and a thin ratatui and crossterm shell performs them.
- Whether `notify` file watching is reliable on macOS, Linux and Windows for files saved by rename, as Vim and many editors do.
- What happens when a write races an external save: refuse, re-resolve the ref, or merge.

## Answer

(Written when the spike closes. A spike closed without this section filled in was wasted.)

## Acceptance criteria

- [ ] The Answer section states a decision and the evidence for it.
- [ ] Follow-up items exist for everything the answer implies.
- [ ] A throwaway prototype renders `examples/bakery/` as a tree with `--screenshot 100x30` and the output is committed to the answer.
