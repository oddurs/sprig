---
id: 117
uid: 2419fc44-d771-40d2-8335-4846af39e7c6
title: The CLI has no config file, daemon, plugins or network
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: cli
---

## Context

Every feature request for a command-line tool pulls toward a config file, a background index, a plugin API or a sync service. Each one makes Sprig something to install, configure and trust instead of a tool to pick up. The plan is `design/cli-plan.html`.

## Options and tradeoffs

A config file lets people change defaults, but then the same command prints different things on different machines, and bug reports stop being reproducible. A daemon or index makes large workspaces faster, but it can go stale, and parsing a hundred files well inside the performance budget (0056) is fast enough. Plugins add reach, but `--plain` and `--json` output plus meaningful exit codes already let any tool on the machine extend Sprig through a pipe.

## Decision

The `sprig` binary reads flags and environment variables only: `NO_COLOR`, `SPRIG_TODAY` for tests, `$EDITOR`, `$PAGER`. It keeps no state between runs, starts no background process, loads no plugins and opens no network connection. It prompts only when stdin and stdout are both terminals. Output has three forms: readable on a terminal, stable tab-separated `--plain` with the ref first, and `--json` in the tree schema.

## Revisit when

A real workspace exceeds the performance budget with the core already optimised, and the profile shows parsing, not rendering, is the cost.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
