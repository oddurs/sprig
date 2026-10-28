---
id: 119
uid: 3997a45c-be64-45fb-8b69-a879b8c42096
title: 'Output layer: terminal, --plain, and colour from the marks'
type: feature
status: done
milestone: v0.2
depends_on:
- 22
- 23
- 25
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
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

- [x] `next` and `check` use the renderer, and their v0.1 snapshots still pass or are updated in the same pull request.
- [x] `NO_COLOR=1` output equals `--color never` output byte for byte, and neither contains an escape sequence; a test checks both.
- [x] `--plain` output of `next` piped through `cut -f1` gives refs that `sprig tick -` accepts, covered by an end-to-end test.
- [x] A test fixes the width at 60 and 120 columns and checks truncation ends with an ellipsis, never mid-character.

## 2026-10-28

`crates/sprig-cli/src/out.rs`: commands build rows of styled cells and one renderer prints them. The terminal form is aligned columns that give way in a set order, cut by display width with `unicode-width`. `--plain` gives tab-separated fields with a ref first, the order recorded in `spec/tools.md` T4 like git's `--porcelain`. Colour comes from the marks only, using the site palette's hues as 256-colour codes, and follows ripgrep and no-color.org: `--color` wins, then a non-empty `NO_COLOR`, then whether stdout is a terminal. Width comes from `COLUMNS`, then the terminal, else no limit. (1) `next` and `check` use it, and their v0.1 snapshots pass unchanged. (2) `no_color_and_color_never_print_the_same_bytes_with_no_escapes` compares `next`, `tree` and `status` byte for byte, and checks `--color always` beats `NO_COLOR`. (3) `plain_next_feeds_tick_through_cut`: the first field of `next --plain` piped to `sprig tick -` ticks those items, and `next` no longer lists them. (4) `widths_truncate_with_an_ellipsis_and_never_mid_character` checks 60 and 120 columns, and 30 with Japanese text: every line fits, and cuts end in `…`.

## Result

One output layer: aligned and fitted for people, --plain for scripts, colour from the marks under NO_COLOR rules.
