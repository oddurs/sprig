---
id: 129
uid: ce36df8a-71b0-487b-8f5c-c40ed27139d4
title: 'Guide: Sprig in the shell'
type: docs
status: done
milestone: v0.3
depends_on:
- 64
- 119
- 120
- 126
- 128
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p2
effort: s
area: docs
---

## Reader and question

Someone who lives in a terminal asks how Sprig fits the tools they already use.

## Change

A guide on the site with the recipes from the plan: pick and tick with fzf, the next task in a Starship prompt, `status --oneline` in tmux, a pre-commit hook that runs `check --strict` on changed files, jq over `find --json`, an ICS feed, and `graph` through Graphviz. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [x] The reader can complete the task described with the current tools.
- [x] Each recipe that needs no interactive tool runs in CI against `examples/bakery/`, so the guide cannot go stale.

## Result

site/src/content/docs/docs/shell.md: fzf, the next task in a prompt, a status bar, a pre-commit hook, jq over find --json, an ICS feed and Graphviz. crates/sprig-cli/tests/guide.rs runs every recipe marked <!-- run --> in bash against a git copy of the bakery, with fzf stubbed, and fails in CI if jq or dot is missing on Linux.
