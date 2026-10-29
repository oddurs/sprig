# Sprig tool behaviour

The specification (`spec/sprig.md`) says what a file means. This document says
how tools that act on files should behave where two tools could reasonably
differ: how a person names an item from outside a file, and what formatting
changes. None of it changes what a file means. A tool may do less; a tool that
does these things should do them this way, so that a ref typed into one tool
works in another and two formatters never fight over a file.

This document is dedicated to the public domain under
[CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).

## T1. Refs

A **ref** names one item from a command line, an editor or an agent. It is
resolved against a workspace: the files a tool was asked to read.

### T1.1 Forms

A tool tries these forms in order and uses the first that applies:

1. `^anchor`: the item with that anchor (§4.5), in any file of the workspace.
2. `file^anchor`: the item with that anchor in one file.
3. `file:12`: the item that starts on line 12 of a file, counting from 1, as
   compilers print a position.
4. Anything else: words from an item's text (§4.9). The ref matches every item
   whose text contains it, ignoring case. If exactly one item's text equals it,
   ignoring case, that item is meant.

A file part names a workspace path with `/` between directories; `.sprig` may
be left off. If no path matches, a file name alone matches when exactly one
file in the workspace has it.

### T1.2 Where an item is

A ref names an item **where it is written**. An item shown in a tree through a
graft (§8) is named by its own file and line, never by the graft that shows it,
so a ref always points at the line an edit changes (§8.6).

Words never match notes or answers, only item text.

### T1.3 One item or none

A ref that matches no item is an error that says what was looked for. A ref
that matches more than one is an error that lists every candidate, one per
line, as `file:line  text`. A tool must not guess.

An anchor written in two files is ambiguous as `^anchor` and resolves as
`file^anchor`. Within one file, a repeated anchor names its first item, as
references do (Appendix B.4).

### T1.4 Stability

Anchors are the stable form: they survive edits above them. A line number
changes when a line is added above it, so a tool given several refs resolves
all of them against the files as they are before it changes any of them, and
changes nothing if any ref fails.

```
^signed
lease^signed
lease.sprig:13
grease trap
```

## T2. Formatting

A formatter changes layout and nothing that a file means. It:

1. indents each line two spaces per level of the tree (§3.3): items by depth,
   and notes and answers at the depth of their item's children;
2. writes typed date shortcuts in `due:` and `start:` out in the stored form
   (§9.4), on items, answers and the title, never in notes;
3. removes trailing whitespace, and whitespace on blank lines;
4. ends the file with exactly one line ending, removing blank lines after the
   last line of text.

It keeps everything else: the order of lines, blank lines between them, the
words and spacing within a line, comments, the file's line endings (LF or
CRLF, following its first line) and a byte order mark.

Two cases need a rule:

- **Comments** (§2.7) keep their place relative to the lines above them: the
  level of the nearest line above with the same indentation, or one level inside
  the nearest item above with less, and otherwise the margin.
- **A first note in a file without a title** keeps an indentation of at least
  two spaces, since at the margin it would become the title (§1.2).

Formatting is idempotent: formatting a formatted file changes nothing.

## T3. Editing

A tool that changes a file (ticks an item, sets a field, adds an item or an
answer) changes only the bytes it means to, and leaves every other byte as it
was, including indentation style, line endings and comments. Added lines use
the file's line ending and the indentation of their siblings.

Ticking an item makes an open item done and a done item open, except that
ticking an open item with `every:` moves its `due:` instead (§10.4).

## T4. Output for scripts

`sprig` prints tables for people and, with `--plain`, tab-separated lines for
scripts, the way git's `--porcelain` does: no colour, nothing cut to fit, and a
column order that is kept from one release to the next. New columns are added
only at the end. The first field is always a ref (T1) or a path, so
`cut -f1` gives something another command accepts.

| Command | Fields |
| --- | --- |
| `next` | ref, mark, text, path through the tree, due date (stored form, or as written), priority (0 to 3) |
| `check` | file, line, column, severity, code, message |
| `tree` | ref, depth, mark, text, status, done, total |
| `status` | file, done, total, ready, waiting, late, errors |
| `find` | ref, mark, text, path through the tree, due date, status |
| `agenda` | ref, due date, mark, text, `every:` value |

`next`, `find` and `export --format json` also print JSON (T5.5).

A ref here is `file^anchor` when the item has an anchor, since that survives
edits, and `file:line` otherwise.

Colour follows the convention at no-color.org: `--color always` or `never`
decides; otherwise a non-empty `NO_COLOR` turns it off, and it is on only
when output goes to a terminal. Colour only ever repeats what a mark already
says.

## T5. Queries, calendars and conversions

### T5.1 Queries

A query is a list of terms, and an item matches when it matches all of them.
A term is one of:

| Term | Matches an item |
| --- | --- |
| a word | whose text contains it, ignoring case |
| `@name` | whose people include `name` after §6.1, so inherited people count |
| `#tag` | that writes the tag itself; tags do not reach down (§4.3) |
| `is:STATE` | whose status is `open`, `todo`, `doing`, `done`, `dropped`, `later` or `ask`, or that is `blocked` (§7.4) |
| `due:DATE` | whose own `due:` is that date; `<`, `<=`, `>` or `>=` before the date compares instead |

`open` is every status that is not settled: todo, doing and ask. A date in a
query reads as one in a file (§9.4), against the same today. A term that looks
like a filter (`key:` with a key other than `is` or `due`) and is not one is
an error that names the filters, rather than a word that matches nothing.

### T5.2 Agenda

The agenda is the open items with a `due:` of their own, by date: those
before today as late, then one heading for each day of the window that has
any. A grafted copy of an item is not listed again. A recurring item appears
once, on the date its `due:` names; the dates after it are the next tick's.

### T5.3 Calendars

An iCalendar export (RFC 5545) has one all-day `VEVENT` for each item the
agenda would list, without a window. Its `UID` is `file^anchor@sprig` for an
item with an anchor, so renaming the item keeps the event, and
`file:words@sprig` otherwise, from the item's text in lowercase with
hyphens. `DTSTAMP` is today at midnight UTC, so the same files and the same
today give the same bytes. A recurring item's `every:` is in the
description, not an `RRULE`: the file, not the calendar, decides when it
next falls.

### T5.4 Graphs

A dependency graph has a node for each open item with an `after:` that names
something, and one for each thing named, whatever its status, grouped by
file. An edge runs from what is named to the item that waits on it, so the
graph reads in the order work can happen. A node's id is the file name
without `.sprig`, then `_` and the anchor when there is one or `l` and the
line otherwise, with every character that is not an ASCII letter or digit as
`_`: `bakery_loan`, `launch_l6`. The same files give the same output, and a
diff shows only what changed.

### T5.5 JSON

Each item in `next --json` and `find --json` is an object with `ref` (T1),
`path` (the labels from the top of its tree to its parent) and `node`: a
resolved node in the tree format (`spec/tree.schema.json`) without
`children`. `export --format json` is a list of tree documents, one for each
tree that is not grafted into another.

### T5.6 Conversions

An import writes a file that `sprig check` accepts and `sprig fmt` leaves as it
is. Tokens in the imported text keep their meaning, so a title such as
`Fix #12` gets the tag `12`.

| From | To |
| --- | --- |
| Markdown | Its first heading is the title. Later headings become groups, and checklist items become items, nested as they were, with `[x]` as done. Other prose is kept as notes. |
| todo.txt | `x` is done. Priorities `(A)`, `(B)` and `(C)` or below become `!!!`, `!!` and `!`. `+project` and `@context` become tags. Creation and completion dates are dropped; `key:value` pairs are kept as fields. |
| GitHub issues (`gh issue list --json number,title,state,stateReason,labels,assignees,milestone`, or `gh pr list` with the same fields) | Milestones become groups, labels tags, and assignees people. Closed and merged become done, and closed as not planned dropped. Each item carries `issue:NUMBER`. |

## T6. Agents

### T6.1 The convention

A repository's plan for the work in progress is `PLAN.sprig` at its root. An
agent reads it before starting, writes its steps into it before changing code,
ticks each step in the same change as the work, and asks a person by adding a
`?` item that names them, which it does not answer itself. `sprig agent`
prints these instructions; `sprig agent --write FILE` keeps them in FILE
between the lines `<!-- sprig:start: ... -->` and `<!-- sprig:end -->`,
replacing only what is between them.

### T6.2 The MCP server

`sprig mcp` speaks the Model Context Protocol (revision 2025-11-25, and
2025-06-18 and 2025-03-26 to a client that asks for one of those) over stdio,
one JSON-RPC message per line. It serves one folder, `--in PATH`, and paths in
its results are relative to that folder, as refs are (T1). It reads the files
again for every call.

The read tools return exactly what the matching command prints with `--json`,
as text and, wrapped in `{"items": ...}` when it is a list, as structured
content:

| Tool | Arguments | Same as |
| --- | --- | --- |
| `list_plans` | none | `status --json` |
| `show_tree` | `file` | that file's document in `export --format json` |
| `next_items` | `who`, `limit`, both optional | `next --json` |
| `find_items` | `query` (T5.1) | `find --json` |
| `check_plans` | none | `check --json` |

The write tools take refs (T1) and return the lines they wrote, `diff` (a
unified diff of every file they changed) and `no_longer_waiting`, the open
items that wait on less than before, each with `now`: `unblocked`, or what it
still waits on:

| Tool | Arguments |
| --- | --- |
| `tick_items` | `refs`: a list |
| `mark_item` | `ref`, `mark`: a mark or its name |
| `add_items` | `items`: a list of lines; `under` or `after`, a ref, or neither for the end of `PLAN.sprig` |
| `answer_question` | `ref`, `text` |
| `set_field` | `ref`, `key`, `value`, empty to remove the field |

Each write is all or nothing. It is refused, and nothing is written, when it
would add an error that `check` reports; the message names each one as
`check` prints it. Typed dates in added lines are written out (§9.4).
`add_items` adds at most `--max-new-items` items, 20 by default, in a
session, counting every call. A call that would pass the limit is refused
with a message that names the limit and the flag.

A tool that fails reports it in its result with `isError` and a message the
model can act on. An unknown tool is a protocol error (`-32602`).
