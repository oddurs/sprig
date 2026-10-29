---
title: Sprig in the shell
description: Pick and tick with fzf, your next task in the prompt, a pre-commit hook, jq over the tree, a calendar feed and the graph as a picture.
---

Sprig has no plugins. It doesn't need them, because every command can feed the
tools already on your machine:

- `--plain` prints tab-separated lines whose first field is always a ref that
  another command accepts.
- `--json` prints the tree format.
- Exit codes mean something: `next` exits 1 when nothing is ready, `find`
  when nothing matches, and `check` when a plan is broken.

The recipes below run in the [bakery example](https://github.com/oddurs/sprig/tree/main/examples/bakery).
CI runs each one that needs no keyboard, so they stay true. The columns of each
`--plain` command are listed in
[the tools document](https://github.com/oddurs/sprig/blob/main/spec/tools.md#t4-output-for-scripts).

## Pick and tick with fzf

Choose from what's ready, and tick what you chose:

<!-- run -->
```sh
sprig next --plain | fzf | cut -f1 | sprig tick -
```

`tick -` reads refs from standard input, one per line, so `fzf --multi` ticks
several at once.

## Change a plan without opening it

<!-- run -->
```sh
sprig add "Call the plumber due:fri" --under ^design
sprig mark ^logo x
sprig answer ^name "Kiln & Crumb"
```

`add` writes typed dates out, so `due:fri` lands in the file as a date. `mark`
and `answer` print the items that stopped waiting on the one they changed.
`sprig edit ^oven` opens `$VISUAL` or `$EDITOR` at the item's line.

## Your next task in the prompt

A [Starship](https://starship.rs) custom module, shown in folders with a
`PLAN.sprig`:

```toml
[custom.sprig]
command = "sprig next --who me --limit 1 --plain | cut -f3"
when = "test -f PLAN.sprig"
```

The command on its own:

<!-- run -->
```sh
sprig next --who teo --limit 1 --plain | cut -f3
```

## The folder in a status bar

`status --oneline` fits in tmux's status line, and exits 1 when anything is
late, for a prompt that wants to turn red:

```sh
set -g status-right '#(cd ~/plans && sprig status --oneline)'
```

<!-- run -->
```sh
sprig status --oneline
```

## Gate a commit

A pre-commit hook that refuses a commit that breaks a plan it touches, and
reads only the files that changed. Save it as `.git/hooks/pre-commit` and make
it executable:

<!-- run -->
```sh
#!/bin/sh
git diff --cached --quiet -- '*.sprig' ||
  git diff --cached -z --name-only --diff-filter=ACMR -- '*.sprig' |
  xargs -0 sprig check --strict
```

In CI, the [Sprig action](https://github.com/oddurs/sprig/tree/main/action) checks
the plans too, and writes a summary of what the pull request did to them.

## What is everyone waiting on?

`find` takes words and filters (`@who`, `#tag`, `is:blocked`, `due:<fri`).
With `--json`, each item carries what it waits on:

<!-- run: needs jq -->
```sh
sprig find is:blocked --json |
  jq -r '.[] | select(.node.waits_on)
    | "\(.node.label)\t\([.node.waits_on[] | .label // "starts \(.start)"] | join(", "))"'
```

A wait is either something the item is `after:`, with its label, or a
`start:` date that hasn't come yet.

## Deadlines in any calendar

`export --format ics` writes the dated work as an iCalendar file. Regenerate it
on a timer, and point a calendar at the file:

<!-- run -->
```sh
sprig export --format ics > plans.ics
```

```sh
# crontab -e: every hour, into a folder your calendar can subscribe to
0 * * * *  cd ~/plans && sprig export --format ics > ~/Sync/plans.ics
```

Each event's UID comes from the item's anchor when it has one, so renaming an
item moves its event rather than adding another. `sprig agenda` shows the same
items by day, in the terminal.

## The graph as a picture

[Graphviz](https://graphviz.org) does the drawing:

<!-- run: needs dot -->
```sh
sprig graph | dot -Tsvg > plan.svg
```

`--format mermaid` writes the same graph for a Markdown file on GitHub, inside a
` ```mermaid ` block.

## Bring a list in

`import` converts a Markdown checklist, a todo.txt file, or GitHub issues from
the `gh` command, and prints Sprig:

```sh
sprig import TODO.md > PLAN.sprig
gh issue list --json number,title,state,stateReason,labels,assignees,milestone > issues.json
sprig import issues.json --from github > issues.sprig
```
