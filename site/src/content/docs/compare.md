---
title: How Sprig compares
description: Sprig next to Markdown task lists, todo.txt, TaskPaper, Org-mode and Taskwarrior, with where each of them is better.
---

Sprig is young, and every tool below is older, better known and more capable in
some way. This page says what Sprig does differently and, for each alternative,
what it does better. Every claim about another tool links to that tool's own
documentation. If a claim is wrong or out of date,
[open an issue](https://github.com/oddurs/sprig/issues) and it will be fixed.

## At a glance

| | Nesting | Computes what's blocked | Plans across files | Where it lives | Source |
| --- | --- | --- | --- | --- | --- |
| **Sprig** | Yes, any depth | Yes, from `after:` | Yes, grafts | Plain files, any editor | [Spec](/spec/) |
| **Markdown task lists** | Yes, nested lists | No | No | Plain files, rendered almost everywhere | [GFM](https://github.github.com/gfm/#task-list-items-extension-) |
| **todo.txt** | No, one task per line | No | No | One plain file, many apps | [Format](https://github.com/todotxt/todo.txt) |
| **TaskPaper** | Yes | No | No | Plain files, best in the TaskPaper app | [Guide](https://guide.taskpaper.com/getting-started/) |
| **Org-mode** | Yes | Partly, parent and child order | Agenda views gather files | Plain files, mostly Emacs | [Manual](https://orgmode.org/manual/TODO-dependencies.html) |
| **Taskwarrior** | Through projects | Yes | Not files | Its own database, via the command line | [Docs](https://taskwarrior.org/docs/urgency/) |

## Markdown task lists

GitHub-flavoured Markdown's [task list items](https://github.github.com/gfm/#task-list-items-extension-)
put `[ ]` and `[x]` in a list, and nest like any list.

**Better than Sprig at:** being everywhere. Every forge, notes app and chat tool
renders them, and nobody has to learn anything.

**What Sprig adds:** statuses beyond done and not done, dependencies that are
computed, and files that graft into each other. Sprig deliberately does not
accept `- [ ]`; its [settled arguments](/decisions/#no-markdown-checkboxes-in-the-parser)
explain why.

## todo.txt

The [todo.txt format](https://github.com/todotxt/todo.txt) is one task per line,
with priorities, `+project` and `@context` tags, and `key:value` extensions such
as `due:`.

**Better than Sprig at:** being simple enough that dozens of apps implement it,
and having done so for well over a decade.

**What Sprig adds:** nesting, notes that belong to a task, and dependencies. Sprig
borrowed `key:value` from todo.txt.

## TaskPaper

In [TaskPaper](https://guide.taskpaper.com/getting-started/), a project is a line
ending in a colon, a task starts with a dash, and `@tags` carry optional values.

**Better than Sprig at:** being a polished app with searches, folding and
stylesheets built around its format.

**What Sprig adds:** computed progress and blocking, and grafts. Sprig first used
TaskPaper's trailing colon for groups and then moved to `#`, because notes ending
in a colon kept turning into headings; that argument is recorded in the
[settled arguments](/decisions/#groups-start-with---not-a-trailing-colon).

## Org-mode

[Org-mode](https://orgmode.org/manual/TODO-dependencies.html) can stop a parent
from being marked done while its children are open, and an `ORDERED` property
makes siblings wait on each other. Its [agenda views](https://orgmode.org/manual/Agenda-Views.html)
gather items from many files.

**Better than Sprig at:** almost everything else. Scheduling, clocking, tables,
exports and literate programming, refined over more than twenty years.

**What Sprig adds:** a format small enough to implement from a spec in any
language and read without Emacs, dependencies between any two items, and files
that compose into one plan instead of being collected into a view.

## Taskwarrior

[Taskwarrior](https://taskwarrior.org/docs/urgency/) tracks dependencies properly:
blocked and blocking tasks feed into its urgency ranking. Since version 3 it keeps
tasks in its own [SQLite database](https://taskwarrior.org/docs/upgrade-3/).

**Better than Sprig at:** being a mature, fast command-line task manager with
reports, filters and sync.

**What Sprig adds:** a plan you write and read as text, review in a diff, and
edit in any editor, where Taskwarrior is a database you drive through commands.
