---
title: Chea in the shell
description: Pick and tick with fzf, your next task in the prompt, a pre-commit hook, jq over the tree, a calendar feed and the graph as a picture.
---

Chea has no plugins. It doesn't need them, because every command can feed the
tools already on your machine:

- `--plain` prints tab-separated lines whose first field is always a ref that
  another command accepts.
- `--json` prints the tree format.
- Exit codes mean something: `next` exits 1 when nothing is ready, `find`
  when nothing matches, and `check` when a plan is broken.

The recipes below run in the [bakery example](https://github.com/oddurs/chea/tree/main/examples/bakery).
CI runs each one that needs no keyboard, so they stay true. The columns of each
`--plain` command are listed in
[the tools document](https://github.com/oddurs/chea/blob/main/spec/tools.md#t4-output-for-scripts).

## Pick and tick with fzf

Choose from what's ready, and tick what you chose:

<!-- run -->
```sh
chea next --plain | fzf | cut -f1 | chea tick -
```

`tick -` reads refs from standard input, one per line, so `fzf --multi` ticks
several at once.

## Change a plan without opening it

The file is the truth, so most changes are a line typed in an editor. The one
that is fiddly by hand has a command: ticking, which moves a recurring item to
its next date and can set any mark.

<!-- run -->
```sh
chea tick ^logo
chea tick ^design --mark '~'
```

Each prints the line it changed, and then, on standard error, the items that
stopped waiting on it. `fmt` writes typed dates out, so `due:fri` typed in an
editor lands in the file as a date.

## Your next task in the prompt

A [Starship](https://starship.rs) custom module, shown in folders with a
`PLAN.chea`:

```toml
[custom.chea]
command = "chea next --who me --limit 1 --plain | cut -f3"
when = "test -f PLAN.chea"
```

The command on its own:

<!-- run -->
```sh
chea next --who teo --limit 1 --plain | cut -f3
```

## The folder in a status bar

`tree --summary --oneline` fits in tmux's status line, and exits 1 when
anything is late, for a prompt that wants to turn red:

```sh
set -g status-right '#(cd ~/plans && chea tree --summary --oneline)'
```

<!-- run -->
```sh
chea tree --summary --oneline
```

## Gate a commit

A pre-commit hook that refuses a commit that breaks a plan it touches, and
reads only the files that changed. Save it as `.git/hooks/pre-commit` and make
it executable:

<!-- run -->
```sh
#!/bin/sh
git diff --cached --quiet -- '*.chea' ||
  git diff --cached -z --name-only --diff-filter=ACMR -- '*.chea' |
  xargs -0 chea check --strict
```

In CI, the [Chea action](https://github.com/oddurs/chea/tree/main/action) checks
the plans too, and writes a summary of what the pull request did to them.

## What is everyone waiting on?

`find` takes words and filters (`@who`, `#tag`, `is:blocked`, `due:<fri`).
With `--json`, each item carries what it waits on:

<!-- run: needs jq -->
```sh
chea find is:blocked --json |
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
chea export --format ics > plans.ics
```

```sh
# crontab -e: every hour, into a folder your calendar can subscribe to
0 * * * *  cd ~/plans && chea export --format ics > ~/Sync/plans.ics
```

Each event's UID comes from the item's anchor when it has one, so renaming an
item moves its event rather than adding another. In the terminal,
`chea find is:open 'due:<=+2w'` lists the same work, and `chea tui` shows it
by day.

## The graph as a picture

[Graphviz](https://graphviz.org) does the drawing:

<!-- run: needs dot -->
```sh
chea export --format dot | dot -Tsvg > plan.svg
```

`--format mermaid` writes the same graph for a Markdown file on GitHub, inside a
` ```mermaid ` block.

## Bring a list in

`import` converts a Markdown checklist, a todo.txt file, or GitHub issues from
the `gh` command, and prints Chea:

```sh
chea import TODO.md > PLAN.chea
gh issue list --json number,title,state,stateReason,labels,assignees,milestone > issues.json
chea import issues.json --from github > issues.chea
```
