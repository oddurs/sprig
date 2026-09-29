---
id: 119
uid: 3997a45c-be64-45fb-8b69-a879b8c42096
title: 'Output layer: terminal, --plain, and colour from the marks'
type: feature
status: planned
milestone: v0.2
depends_on:
- 22
- 23
- 25
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: cli
---

## Problem

`next` and `check` in v0.1 each print text their own way. Before `tree`, `status`, `why` and the TUI arrive, the CLI needs one place that decides widths, colour and machine-readable output, or each command invents its own.

## Proposal

Each command builds a small table of rows, and one renderer prints it in one of three forms:

- **terminal**: aligned, truncated to the terminal width, colour taken from the mark palette in `site/src/design/tokens.mjs`. Colour only ever repeats what the mark character already says.
- **`--plain`**: tab-separated, the ref in the first column, no colour, no truncation. The column order of each command is part of the stability policy (0073).
- **`--json`**: left to each command (0024 and 0064).

`--color auto|always|never` and `NO_COLOR` are respected. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] `next` and `check` use the renderer, and their v0.1 snapshots still pass or are updated in the same pull request.
- [ ] `NO_COLOR=1` output equals `--color never` output byte for byte, and neither contains an escape sequence; a test checks both.
- [ ] `--plain` output of `next` piped through `cut -f1` gives refs that `sprig tick -` accepts, covered by an end-to-end test.
- [ ] A test fixes the width at 60 and 120 columns and checks truncation ends with an ellipsis, never mid-character.
