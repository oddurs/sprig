# Sprig specification, annex

The [core](sprig.md) says what each line and token of a Sprig file means. This
annex holds the rest: the rulings on edge cases, each numbered as a rule of the
chapter it belongs to and as normative as any rule there, then the grammar, the
ambiguities the drafts settled, and the tree format.

## History

Draft 0.5 formalises draft 0.2, the version described and implemented in
`design/sprig-draft-0.2.html`. Draft 0.4 tightened three rules of it: `due` is
the one deadline field (§9.1), a future `start:` blocks (§7.5), and grafted
content appears once in a tree (§8.8). Draft 0.5 rules on every ambiguity
[Appendix B](#appendix-b-known-ambiguities) listed and on the cases the
conformance suite found the text silent on, and makes one change of meaning: a
dropped item no longer holds up what waits on it (§5.7).

## Edge cases

### 1.3 A file without a title

If the first line that is not blank or a comment is indented, an item or an
answer, the file has no title.
Its items still form a tree under an implicit root. Tools may show the file name
in place of a title.

```sprig
- milk
- bread
```

### 2.8 Escaped notes

A line whose first character after indentation is `\` is a note whose text is the
rest of the line after the backslash, even if that text starts with a mark. A
backslash on its own is a note with no text.

```sprig
Escapes

- Pack
  \- the dash here is part of the note, not a new item
  \x marks the spot
```

### 3.2 No depth limit

Items nest to any depth.

```sprig
Deep

- Company
  - Team
    - Project
      - Task
        - Step
```

### 5.6 Parents keep their written status

An item with children keeps the status it is written with; progress beneath it
does not change it. A tool may point out a parent whose counting children are all
done, but must not display it as done until it is written `x`.

```sprig
Ready to close

- Design
  x Layout sketch
  x Ventilation plan
```

*Design* is still to do, and ready to close.

### 5.7 Finished, for references

A node is **finished**, for the purposes of §7, when its status after settling
(§5.2) or derivation (§5.5) is done or dropped. An answered question is done. A
dropped item was decided against, so nothing waits on it any longer; a later
item has only been put off, so what waits on it still waits. An empty node
(§5.5) is never finished, and a reference to one should be warned about.

```sprig
Done

? Free tier?  ^tier
  = Yes, capped at three.
- Pricing page  after:^tier
```

The question is answered, so *Pricing page* is not waiting.

```sprig
Dropped

/ Custom domain  ^domain
- Launch  after:^domain
```

The domain was decided against, so *Launch* is not waiting either.

### 7.4 Blocked is never written

Blocked is computed and must never be stored in a file. There is no mark or field
for it.

```sprig
Computed

- Permit  ^permit
- Sign the lease  after:^permit
```

### 8.4 Loops

A graft whose file is already being resolved above it would loop. It is an error
reported on that graft line, and the graft has no grafted children.

Loops are found once for the whole workspace, so the same line is reported
whichever file a tool opens: files are visited in order of their paths and
grafts in document order, and the graft that reaches a file still being
visited closes the loop. That graft is refused in every tree, and with it taken
out no file's tree loops.

```sprig
A

+ [[b]]
```

If this is `a.sprig` and `b.sprig` grafts `[[a]]`, path order visits `a.sprig`
first, and the graft in `b.sprig` closes the loop. Read on its own, `a.sprig`
still shows what `b.sprig` writes; `b.sprig` does not show `a.sprig`.

### 8.5 Missing targets

A graft naming a file or anchor that does not exist is an error on the graft
line, and the graft has no grafted children.

```sprig
Missing

+ [[nowhere]]
```

### 8.6 Editing grafted items

A tool that changes an item shown through a graft must change the original line
in the file that writes it.

```sprig
Mounted

+ [[kitchen]]
```

Ticking a kitchen item seen here edits `kitchen.sprig`.

### 8.7 Anchors belong to their file

An anchor belongs to the file that writes it. Anchors inside content a file
grafts in are not anchors of the grafting file, and `[[file^anchor]]` reaches
only anchors that `file` writes itself; refer to an anchor through the file that
writes it.

```sprig
Anchors across files

+ [[lease]]
- Supplier credit  after:[[lease^signed]]
```

### 8.8 Grafted content appears once

Within one tree an item appears at most once. A graft whose content is already in
the tree, because it names the same file or branch as an earlier graft, a branch
inside an earlier grafted file, or a file containing an earlier grafted branch, is
an error reported on the later graft line in document order, and that graft has
no grafted children. Progress therefore never counts an item twice.

```sprig
Once

+ [[kitchen]]
+ [[kitchen^oven]]
```

The second graft line reports that the oven branch is already in the tree.

### 9.5 Unreadable dates

A date field whose value is neither form is kept as written and shown as text. A
tool should warn about it. A stored date for a day that does not exist, such as
`2026-02-30`, is unreadable too.

```sprig
Unreadable

- Sometime  due:soonish
```

### 9.6 Due dates reach down

An item without a `due:` of its own is due when the nearest item above it with
one is, the title included. Tools that order or flag work by due date use this
**effective due date**; an item's own `due:` always wins over one above it.

```sprig
Launch  due:2026-12-05

- Print the menus
- Window lettering  due:2026-11-20
```

*Print the menus* is due on 5 December, *Window lettering* on 20 November.

### 10.4 Ticking a recurring item

Ticking an open recurring item must not mark it done. Instead a tool moves its
`due:` to the next occurrence after the later of its current due date and today,
and leaves it open. For a weekday rule the next occurrence is strictly after that
date. A month later is the same day of the next month, or its last day when the
month is shorter; only the due date is stored, so the day does not come back
(31 January, 28 February, 28 March). An item with no `due:` gets one. An item
whose `every:` has no next occurrence (§10.3) is marked done.

```sprig
After ticking on 2026-10-02

- Weekly numbers to the bank  every:fri  due:2026-10-09
```

## Appendix A. Grammar

Informal; the rules above are authoritative.

```
file    = [ bom ] [ title ] { line }   ; §1.1
title   = [ "\" ] { token }            ; §1.2, never indented
line    = indent ( item | answer | note | comment | blank )
indent  = { " " | tab }                ; §3.1, each tab counts as two
item    = mark ( ws { token } | eol )  ; §2.3
mark    = "-" | "~" | "x" | "/" | ">" | "?" | "#" | "+"
answer  = "=" ( ws { token } | eol )   ; §2.5
comment = "//" text                    ; §2.7
note    = [ "\" ] text                 ; §2.6, §2.8
token   = "@" name | "#" name | "^" name                  ; §4.2, §4.3, §4.5
        | "!" | "!!" | "!!!"                              ; §4.4
        | key ":" ( value | '"' text '"' )                ; §4.6
        | link | word                                     ; §4.8, §4.9
key     = letter { letter | digit | "_" | "-" }
link    = "[[" path [ "^" name ] "]]"  ; path has no whitespace
est     = digits [ "." digits ] ( "h" | "d" | "w" )       ; §10.1
```

## Appendix B. Known ambiguities

Places where draft 0.2's reference parser and the text disagreed, or where
neither said enough. Draft 0.5 rules on every one; the rulings now live in the
numbered rules, and each entry keeps its number so that citations stay stable.
A new ambiguity takes the next number and stays open until a draft rules on it.

1. **Byte order mark.** The reference parser treated a BOM as whitespace, so the
   first line counted as indented and the file had no title. Resolved: a BOM is
   neither indentation nor text (§1.1; `lines/012`).
2. **Tabs after spaces.** The reference parser expanded only a run of tabs at
   the very start of a line. Resolved: every tab in the indentation counts as
   two spaces (§3.1; `lines/049`).
3. **Several anchors on one item.** Resolved: the last names the item, and a
   tool should warn (§4.5; `tokens/019`).
4. **Duplicate anchors.** Resolved: references and grafts name the first item,
   and a tool should warn about the later one (§4.5; `tokens/020`).
5. **Empty groups and references.** Resolved: empty is not done, so an `after:`
   naming something empty waits until it has something finished, and a tool
   should warn (§5.5, §5.7; `graph/016`).
6. **Recurring parents.** §5.3 excluded any item with `every:`, the reference
   parser only recurring leaves. Resolved for the parser: a recurring item with
   children is a checklist, and its children count (§5.3; `graph/019`).
7. **Grafting a grafted branch.** Resolved in draft 0.4: `[[file^anchor]]`
   reaches only anchors `file` writes itself (§8.7; `graph/052`).
8. **File names with spaces.** Resolved: a link cannot name a file whose path
   contains whitespace (§4.8).
9. **Directories.** The reference parser had a flat namespace of file names.
   Resolved: links resolve from the linking file's directory, and a path that
   leaves the workspace is refused (§4.8, §8.2; `graph/042`, `graph/043`).
10. **Loop reporting.** In the reference parser, which graft line reported a
   loop depended on which file was read first. Resolved: loops are found once
   for the workspace, in path order (§8.4; `graph/045`).
11. **An answer on the first line.** Resolved: an answer is not a title (§1.2;
   `lines/007`).
12. **Notes either side of an item.** The reference parser joined them into one
   paragraph. Resolved: a child item ends a paragraph (§2.6; `lines/052`).
13. **Letters in names.** The reference parser accepted only ASCII. Resolved:
   letters and digits of any script (Words; `lines/010`).
14. **An escape on its own.** Resolved: a note with no text (§2.8; `lines/042`).
15. **Days that do not exist.** The reference parser rolled `feb30` and
   `2026-02-30` into March. Resolved: they are not dates, and a bare `feb29`
   follows the 120-day rule (§9.3, §9.5; `tokens/050`, `tokens/051`).
16. **Brackets that are not a link.** The reference parser dropped the brackets
   of any `[[...]]` in an item's text, link or not. Resolved: only a link loses
   them (§4.9; `tokens/057`, `tokens/058`).

## Appendix C. Settled arguments

The design decisions behind these rules are recorded as cairn items of type
`decision` in this repository, each with the case for, the case against, and when
to revisit it: `cairn list --view decisions`.

## Appendix D. The tree format

A tool that hands a Sprig file to another program, and every expected result in
the conformance suite (`tests/conformance/`), uses one JSON shape. Its schema is
`spec/tree.schema.json`, in JSON Schema 2020-12.

**Version.** A document carries `"sprig_tree": 1`. The number changes only when
a reader written for the old version would misread the new one: a key removed,
renamed, or given a new meaning. Adding an optional key does not change it, so
a reader must ignore keys it does not know.

**Missing keys.** A key whose value would be an empty list, an empty object, `0`
or `false` may be left out, and a reader treats a missing key as that value.

**Two layers.** The *syntax* layer is what one file says, and contains nothing
that depends on another file or on the date. The *resolved* layer adds what is
computed: grafts mounted (§8), status (§5), progress, estimates, people (§6) and
blocking (§7). Keeping them apart lets a parser and a resolver be tested on
their own.

### D.1 The document

| Key | Type | Meaning |
| --- | --- | --- |
| `sprig_tree` | `1` | The format version. Required. |
| `file` | path | The file this document describes. |
| `today` | date | The day the resolved layer was computed for (§7.5). |
| `syntax` | object | The syntax layer, D.2. |
| `resolved` | node | The root of the resolved tree, D.3. |
| `diagnostics` | list | Every problem found, D.4. |

A *path* is relative to the workspace root, with `/` between components, and
ends in `.sprig`. A *date* is `YYYY-MM-DD`. A *span* is a pair of byte offsets
into the file, start inclusive and end exclusive. A *line* counts from 1.

### D.2 The syntax layer

`syntax` has `title`, the title's text or `null` when the file has none (§1.3),
and `root`, a syntax node for the file's root.

A **syntax node** is the root or one item.

| Key | Type | Meaning |
| --- | --- | --- |
| `line` | line | The item's line, or the title's. Absent on the root of a file without a title. |
| `span` | span | The line after its indentation, without the line ending or trailing whitespace. Absent when `line` is. |
| `mark` | string | The item's mark (§2.2): one of `-` `~` `x` `/` `>` `?` `#` `+`. Absent on the root. |
| `text` | string | The item's text (§4.9), or the title's. Absent on the root of a file without a title. |
| `notes` | list of strings | Note paragraphs (§2.6), each one's lines joined by single spaces. |
| `answers` | list | Answer lines (§2.5), each with `line`, `span`, `text` and tokens. |
| `children` | list of nodes | Child items in written order. |

and the **tokens** of §4, on items, the root and answers alike:

| Key | Type | Meaning |
| --- | --- | --- |
| `people` | list of strings | Names after `@` (§4.2), in order. |
| `tags` | list of strings | Names after `#` (§4.3), in order. |
| `priority` | 1, 2 or 3 | The highest of `!`, `!!` and `!!!` (§4.4). |
| `anchor` | string | The anchor that names the item (§4.5, Appendix B.3). |
| `fields` | object | Each field's key, lower-cased, to the value that counts (§4.6), in order of first use. Quotes are removed. |
| `after` | list of strings | `after:` values as written, in order (§4.7). |
| `links` | list | Links in the item's words (§4.8): each has `path`, with `./` dropped and `.sprig` added but still relative to this file, and `anchor` when the link names one. |

### D.3 The resolved layer

Resolution reads every file of a workspace and builds one tree per file: the
file as it reads when opened on its own, with its grafts mounted. A **resolved
node** is the root or one item of that tree, including items a graft brought
in.

| Key | Type | Meaning |
| --- | --- | --- |
| `file` | path | The file that writes this line. For a grafted item, the grafted file. |
| `line` | line | The line in that file. Absent on the root of a file without a title. |
| `mark` | string | As in D.2. Absent on the root. |
| `label` | string | The item's text; for a graft, the title of what it mounts (§8.3); for the root, the title, or the file's path when there is none. |
| `status` | string | After settling (§5.2) and derivation (§5.5): `todo`, `doing`, `done`, `dropped`, `later`, `question`, or `empty` for a group, graft or root with nothing beneath it that counts. Required. |
| `answered` | boolean | A question with at least one answer (§2.5). |
| `settled` | boolean | The status came from an `x`, `/` or `>` on this item or above it, or from an answer (§5.2). |
| `grafted` | boolean | Brought into this tree by a graft. |
| `graft` | object | On a `+` line: `path`, the grafted file, empty if the link is missing or refused; `anchor` for a branch graft; `title`; `mounted`, false when the graft is an error and mounted nothing; and the mounted branch's `notes` and, for a branch, `answers`. |
| `progress` | object | `done`, `doing` and `total` counting leaves beneath (§5.3, §5.4); a counting leaf counts itself. Required. |
| `estimate` | number | Remaining estimate in hours (§10.1, §10.2). |
| `people` | list of strings | The node's people after §6.1. |
| `due`, `start` | date | Its own `due:` and `start:` read against `today`, typed shortcuts resolved (§9.3). Absent when not written or unreadable (§9.5). |
| `blocked` | boolean | Blocked by §7.3, by its own waits or by an ancestor. |
| `waits_on` | list | What this item itself waits on: `{"after", "label"}` for an unfinished reference, with the `file` and `line` it names, or `"unknown": true` when it names nothing (§7.2), or `{"start"}` for a start date still ahead (§7.5). Blocking inherited from an ancestor is shown by `blocked` alone. |
| `ready_to_close` | boolean | Written to do, doing or as a question, with every counting leaf beneath it done (§5.6). |
| `children` | list of nodes | Written children, then grafted ones (§8.1). |

### D.4 Diagnostics

Each has `code` (from `spec/diagnostics.md`), `severity` (`error` or
`warning`), `file`, `line`, `col` (in characters, from 1), `span`, `message`,
and sometimes `help`. A conformance example states only the keys it means to
test, and never `message` or `help`: wording may improve without changing what
is reported.
