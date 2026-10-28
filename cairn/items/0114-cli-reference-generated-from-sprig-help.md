---
id: 114
uid: 018d8b8e-8799-47e4-9101-a6108c9f29dd
title: CLI reference generated from sprig --help
type: feature
status: done
milestone: launch
depends_on:
- 27
- 99
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: site
---

## Problem

A command reference written by hand goes stale the first time a flag changes.

## Proposal

The site build runs the released `sprig` binary's help for each command and renders it as `/docs/reference/cli/`.

## Acceptance criteria

- [x] The page is generated at build time, and adding a command to the CLI adds it to the page with no site change.

## 2026-10-28

`site/scripts/sync.mjs` runs the `sprig` binary built from the same checkout (`scripts/task site` builds it; `SPRIG_BIN` overrides) and writes `/docs/reference/cli/` from `sprig --help` and each command's `--help`. It runs without `SPRIG_TODAY`, so the page does not depend on who built it. (1) The commands are read from the `Commands:` section of `sprig --help`, so a new subcommand appears with no site change. The page is in the sidebar under Reference. Small CLI change: `--today` now hides its environment value in help.

## Result

A command reference generated from the binary at build time.
