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
