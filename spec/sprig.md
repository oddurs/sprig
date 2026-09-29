# Sprig specification, draft 0.3

Sprig is a plain-text format for plans. A file is a tree of items, one per line;
the first character of a line says what the line is, and indentation says where
it belongs. Progress, what is blocked, and who owns what are computed from the
text and never written into it.

This document is dedicated to the public domain under
[CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Anyone may
implement it, copy it, or build on it, without asking.

## About this document

**Status.** Draft 0.3. It formalises draft 0.2, the version described and
implemented in `design/sprig-draft-0.2.html`, without changing its behaviour.
Where this text and that page's parser disagree, the difference is listed in
[Appendix B](#appendix-b-known-ambiguities) rather than silently resolved.

**Rules are numbered.** Each rule has a number such as §3.1, so conformance
examples, diagnostics and discussions can cite it. A number never changes
meaning once published; a rule that is removed leaves its number unused.

**Words.** *Must* and *must not* are requirements on an implementation. *Should*
is a strong recommendation that tools may depart from with a reason. A *tool* is
anything that reads Sprig: a parser, an editor, a renderer, an agent.

**Examples** are complete, valid Sprig files unless marked otherwise.

## 1. Files

### 1.1 Encoding

A Sprig file is UTF-8 text. Lines end with LF; a tool must read CRLF as LF, and a
byte order mark at the start of the file must be ignored. The file extension is
`.sprig`.

```sprig
Groceries

- milk
- bread
```

### 1.2 The title

The first line that is not blank, not a comment (§2.7), not indented, and not an
item (§2.3) is the **title**. The title is the file's root item: its text names
the file, and its tokens (§4) apply to the whole file, the way an item's tokens
apply to its children.

```sprig
Open the bakery  @dana  target:2026-12-05  where:"41 Mill Street"

- Sign the lease
```

Here the title is *Open the bakery*, the file's owner is `dana`, and the file has
two fields, `target` and `where`.

### 1.3 A file without a title

If the first qualifying line is an item or is indented, the file has no title.
Its items still form a tree under an implicit root. Tools may show the file name
in place of a title.

```sprig
- milk
- bread
```

### 1.4 The body

Every line after the title, and every line of a file without one, belongs to the
body and is classified by §2.

```sprig
Weekend

- Laundry
Remember the dry cleaning.
```

The last line is a note on the root, because nothing above it has less
indentation (§3.1).

## 2. Lines

### 2.1 Classification

Each body line is classified by its first characters after its indentation (§3.1),
in this order:

1. empty, or only whitespace: a **blank** line (§2.9);
2. starting with `//`: a **comment** (§2.7);
3. starting with `\`: an **escaped note** (§2.8);
4. a mark (§2.2) followed by whitespace or by the end of the line: an **item**
   (§2.3), except that `=` makes an **answer** (§2.5);
5. anything else: a **note** (§2.6).

```sprig
Classification

- an item
  a note on it
  // a comment, never shown
  \- a note that starts with a mark
```

### 2.2 Marks

There are eight marks for items and one for answers.

| Mark | Meaning | Kind |
| --- | --- | --- |
| `-` | to do | status |
| `~` | doing | status |
| `x` | done | status |
| `/` | dropped: decided against, kept for the record | status |
| `>` | later: parked on purpose | status |
| `?` | a question | status |
| `#` | a group | structure |
| `+` | a graft of another file or branch | structure |
| `=` | an answer to the item above | answer |

A mark is only a mark when whitespace or the end of the line follows it. `#tag`
at the start of a line is a note; `# heading` is a group. `x-ray` is a note; `x ray`
is a done item.

```sprig
Marks

- to do
~ doing
x done
/ dropped
> later
? question
  = its answer
# group
  - inside it
```

### 2.3 Items

An item is a mark, whitespace, and the item's text with its tokens (§4). A mark
on its own is an item with no text.

```sprig
Items

- Book a van  @jo  due:2026-10-17
-
```

### 2.4 Groups

A line starting with `# ` is a group. A group has no status of its own; its
status is derived from its children (§5.5). Groups nest like any item.

```sprig
Supply run

# Flour
  - Rye, 4 sacks
  x Spelt, 1 sack
# Dairy
  x Butter
```

### 2.5 Questions and answers

A line starting with `? ` is a question. A line starting with `= ` is an answer:
it attaches to the nearest item above it with less indentation, exactly as a note
does (§2.6), and carries tokens (§4). A question with at least one answer is
**answered**, and counts as done (§5.2). An answer attached to any other kind of
item records a decision about that item and changes nothing else.

```sprig
Naming

? Kiln & Crumb, or Mill Street Bread?
  = Kiln & Crumb. Trademark search is clear.  @dana
```

### 2.6 Notes

Any other line is a note. A note attaches to the nearest item above it with less
indentation, or to the root if there is none. Consecutive note lines of one item
form a paragraph, joined with single spaces; a blank line between them starts a
new paragraph.

Notes carry no tokens. An `@name` or a `[[link]]` in a note may be displayed as
one, but it assigns nobody and grafts nothing.

```sprig
Kitchen

- Deck oven
  Rondo quoted 14.2k, Mono 12.9k.
  Mono's lead time is ten weeks.

  Ask @teo about the flue.
```

The oven has two paragraphs of notes. `@teo` in the note does not make Teo its
owner.

### 2.7 Comments

A line whose first characters after indentation are `//` is a comment. Tools must
ignore comments entirely: they are not notes, they do not end a paragraph, and
they do not affect which item later lines belong to.

```sprig
Comments

- Paint the hall
  // ask about the ceiling height
  Two coats.
```

### 2.8 Escaped notes

A line whose first character after indentation is `\` is a note whose text is the
rest of the line after the backslash, even if that text starts with a mark.

```sprig
Escapes

- Pack
  \- the dash here is part of the note, not a new item
  \x marks the spot
```

### 2.9 Blank lines

Blank lines separate note paragraphs (§2.6) and have no other meaning. They do
not end an item or change indentation.

```sprig
Blank lines

- First

- Second
```

## 3. Indentation

### 3.1 Relative indentation

A line's **indentation** is the number of spaces before its first other
character, where each leading tab counts as two spaces. A line belongs to the
nearest item above it with strictly less indentation, or to the root if there is
none. Only relative indentation matters.

```sprig
Relative

- Parent
    - Child, four spaces in
  - Also a child, two spaces in
```

Both inner lines are children of *Parent*.

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

### 3.3 Formatting

A formatter should write two spaces per level and must not change the tree.

```sprig
Formatted

- Two
  - spaces
    - per level
```

## 4. Tokens

### 4.1 Splitting

An item's text, after its mark, is split on whitespace into **tokens**. A token
that contains a double-quoted run, such as `where:"41 Mill Street"`, keeps the
spaces inside the quotes. Tokens are classified by §4.2 to §4.8; whatever is left
is the item's text (§4.9).

```sprig
Splitting

- Visit the unit  where:"41 Mill Street"  @dana
```

### 4.2 People

A token that is `@` followed by letters, digits, `_`, `.` or `-`, and nothing
else, names a person. An item may name several.

```sprig
People

- Move the sofa  @me @jo
```

`@jo,` with a trailing comma is not a person; it is a word.

### 4.3 Tags

A token that is `#` followed by letters, digits, `_` or `-` is a tag.

```sprig
Tags

- Coffee beans  #market #weekly
```

### 4.4 Priority

A token that is exactly `!`, `!!` or `!!!` sets the item's priority to 1, 2 or 3.
If several appear, the highest wins.

```sprig
Priority

- Renew the insurance  !!!
- Tidy the shed  !
```

### 4.5 Anchors

A token that is `^` followed by letters, digits, `_` or `-` gives the item an
**anchor**: a name other lines can refer to (§7.2, §8.3). Anchors are unique
within a file.

```sprig
Anchors

- Book a van  ^van
- Move  after:^van
```

### 4.6 Fields

A token of the form `key:value` is a field when the key starts with a letter and
continues with letters, digits, `_` or `-`, and the value is not empty. Keys are
case-insensitive. A value in double quotes loses its quotes. A value starting
with `//` is not a field, so URLs stay words. If a key appears twice, the last
value wins, except for `after` (§4.7).

The keys `due`, `start`, `target` (§9), `est` and `every` (§10) have defined
meanings. Any other key is kept and shown.

```sprig
Fields

- Kitchen quote  due:2026-10-02  est:2h  budget:64k  vendor:"Mono Ovens"
- Read https://example.org/quote
```

### 4.7 After

`after:` fields are collected into a list of references. Each names something the
item waits on (§7).

```sprig
After

- Tests  ^tests
- Docs  ^docs
- Ship  after:^tests  after:^docs
```

### 4.8 Links

`[[file]]` refers to another file and `[[file^anchor]]` to an anchored item in
it. The file name is resolved relative to the file containing the link; `.sprig`
is appended when absent, and a leading `./` is ignored. A link may appear inside
a longer token. In an item's text a link is shown as a link; it has meaning only
in a graft (§8) or an `after:` value (§7).

```sprig
Links

- Supplier credit  after:[[lease^signed]]
- Read the notes in [[kitchen]] first
```

### 4.9 Text

The item's text is its remaining tokens joined by single spaces. In the plain
text of an item, link brackets are dropped: `[[kitchen]]` reads as `kitchen`.

```sprig
Text

- Call   the   landlord  @dana
```

The text is *Call the landlord*.

## 5. Status and progress

### 5.1 Written status

An item's written status is its mark: to do, doing, done, dropped, later or
question. Groups and grafts have none.

```sprig
Written

- Draft
~ Review
x Publish
```

### 5.2 Settling

An item written `x`, `/` or `>` **settles** its whole branch: every descendant
takes that status, whatever its own mark says. An answered question (§2.5)
settles as done.

```sprig
Settling

x Kitchen
  - Tiling
  - Paint
```

Both children count as done.

### 5.3 What counts

A leaf, an item with no child items, counts toward progress when its status
after settling is to do, doing, done or question. Dropped and later leaves do not
count. Neither do groups or grafts with no children, nor any item with an
`every:` field (§10.3), which is upkeep rather than progress.

```sprig
Counting

- Pack
/ Hire movers
> Paint the new place
- Water the plants  every:sat
```

Only *Pack* counts: one leaf, not yet done.

### 5.4 Progress

A node's **progress** is the number of counting leaves beneath it, how many are
done, and how many are doing.

```sprig
Progress

- Kitchen
  x Strip out
  ~ Tiling
  - Paint
```

Kitchen's progress: 1 of 3 done, 1 doing.

### 5.5 Derived status

Groups, grafts and the root take their status from their progress: **empty** if
nothing beneath them counts, **done** if everything counted is done, **doing** if
anything is done or doing, and otherwise **to do**.

```sprig
Derived

# Dairy
  x Butter
  x Milk
# Seeds
  - Sesame
```

*Dairy* is done and *Seeds* is to do.

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

### 5.7 Done, for references

A node is **done**, for the purposes of §7, when its status after settling (§5.2)
or derivation (§5.5) is done. An answered question is done.

```sprig
Done

? Free tier?  ^tier
  = Yes, capped at three.
- Pricing page  after:^tier
```

The question is answered, so *Pricing page* is not waiting.

## 6. People

### 6.1 People flow down

An item's **people** are the people it names (§4.2) if it names any; otherwise
they are its parent's. The root's people are those named on the title. A graft
line that names nobody takes the first person of the file or branch it grafts.

```sprig
Owners  @dana

- Money
  - Loan paperwork
  - Supplier credit  @teo
```

*Loan paperwork* is Dana's; *Supplier credit* is Teo's.

## 7. Blocking

### 7.1 Open items

An item is **open** when its status after settling is to do, doing, or an
unanswered question.

```sprig
Open

- to do, open
~ doing, open
? question, open
x done, not open
```

### 7.2 References

An `after:` value is a reference:

- `^anchor` names an item in the same file;
- `[[file^anchor]]` names an item in another file;
- `[[file]]` names another file as a whole, which is done when its root is done
  (§5.5).

A reference that names no existing file or anchor is **unknown**. A tool must
report it, and must treat it as unfinished.

```sprig
References

- Sign  ^signed
- Order the oven  after:^signed  after:[[kitchen^design]]  after:[[lease]]
```

### 7.3 Blocked

An open item is **blocked** when any of its own references is unfinished (not done
by §5.7, or unknown), or when any ancestor is blocked. Blocking passes to every
open descendant of a blocked item.

```sprig
Blocked

- Design  ^design
- Build  after:^design
  - Strip out
  - Tiling
```

*Build*, *Strip out* and *Tiling* are all blocked until *Design* is done.

### 7.4 Blocked is never written

Blocked is computed and must never be stored in a file. There is no mark or field
for it.

```sprig
Computed

- Permit  ^permit
- Sign the lease  after:^permit
```

## 8. Grafts

### 8.1 Graft lines

A graft is an item written with `+` whose tokens include a link (§4.8). Its
children are its own written children, followed by the top-level items of the
file it names, or the children of the anchored item it names (§8.3). A graft line
with no link is an error.

```sprig
House 2027  @sam

+ [[kitchen]]
+ [[garden]]  @jo
- Insurance renewal
```

### 8.2 Resolution

A graft's file is resolved relative to the grafting file, as for links (§4.8). A
tool must refuse a graft whose path leaves the directory it was asked to read,
and report it.

```sprig
Resolution

+ [[rooms/kitchen]]
```

### 8.3 Branch grafts

`+ [[file^anchor]]` mounts one branch: the anchored item's children, notes and
answers, with the anchored item's text as the graft's title.

```sprig
Dana, this week  @dana

+ [[lease^terms]]
+ [[kitchen^oven]]
```

### 8.4 Loops

A graft whose file is already being resolved above it would loop. It is an error
reported on that graft line, and the graft has no grafted children.

```sprig
A

+ [[b]]
```

If `b.sprig` grafts `[[a]]`, one of the two graft lines reports a loop.

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
grafts in are not anchors of the grafting file; refer to them in their own file
with `[[file^anchor]]`.

```sprig
Anchors across files

+ [[lease]]
- Supplier credit  after:[[lease^signed]]
```

## 9. Dates

### 9.1 Date fields

`due`, `start` and `target` hold dates. `start` is when work can begin; `due` and
`target` are when it should be finished.

```sprig
Dates

- Apply for the permit  start:2026-10-05  due:2026-10-30
```

### 9.2 Stored form

A stored date is `YYYY-MM-DD`. A tool that writes a date must write this form, so a
file means the same thing on every day it is read.

```sprig
Stored

- Renew the lease  due:2027-03-31
```

### 9.3 Typed shortcuts

A parser should also read these forms, resolved against a *today* supplied to it
(a parser must never read the clock itself):

| Shortcut | Resolves to |
| --- | --- |
| `today`, `tomorrow` | today, the day after |
| `+3d`, `+2w` | that many days or weeks from today |
| `q1` to `q4` | the last day of that quarter; next year's if it has passed |
| `oct3`, `oct-3`, `3oct` | that month and day; next year's if more than 120 days past |
| `mon` to `sun`, or full names | the next such weekday, today included |

Month and weekday names match on a prefix of at least three letters.

```sprig
Typed

- Call the bank  due:fri
- Quarterly numbers  due:q4
```

### 9.4 Editors expand shortcuts

An editor should replace a typed shortcut with the stored form when the writer
leaves the line, and a formatter must.

```sprig
Expanded

- Call the bank  due:2026-10-02
```

### 9.5 Unreadable dates

A date field whose value is neither form is kept as written and shown as text. A
tool should warn about it.

```sprig
Unreadable

- Sometime  due:soonish
```

## 10. Estimates and recurrence

### 10.1 Estimates

`est:` takes a number and a unit: `h` hours, `d` days of 8 hours, `w` weeks of 40
hours. A value in any other form is ignored, and a tool should warn about it.

```sprig
Estimates

- Tiling  est:5d
- Grout  est:4h
- Seal  est:0.5d
```

### 10.2 Estimates roll up

A node's remaining estimate is the sum of the estimates of the counting leaves
beneath it that are not done. A parent's own `est:` is used only when nothing
beneath it contributes one and the parent is not settled.

```sprig
Rollup

- Kitchen  est:10d
  x Strip out  est:3d
  - Tiling  est:5d
  - Paint  est:2d
```

Kitchen has 7 days left; its own `est:10d` is not used.

### 10.3 Recurrence

`every:` makes an item recurring. Its value is `day` or `daily`, `week` or
`weekly`, `month` or `monthly`, a count such as `3d` or `2w`, or a weekday name.
Recurring items are upkeep and never count toward progress (§5.3).

```sprig
Upkeep

- Weekly numbers to the bank  every:fri  due:2026-10-02
- Water the plants  every:3d
```

### 10.4 Ticking a recurring item

Ticking an open recurring item must not mark it done. Instead a tool moves its
`due:` to the next occurrence after the later of its current due date and today,
and leaves it open. For a weekday rule the next occurrence is strictly after that
date. An item with no `due:` gets one.

```sprig
After ticking on 2026-10-02

- Weekly numbers to the bank  every:fri  due:2026-10-09
```

## Appendix A. Grammar

Informal; the rules above are authoritative.

```
file    = [ title ] { line }
title   = { token }                        ; §1.2
line    = indent ( item | answer | note | comment | blank )
indent  = { " " | tab }                    ; §3.1, a leading tab counts as two
item    = mark ( ws { token } | eol )      ; §2.3
mark    = "-" | "~" | "x" | "/" | ">" | "?" | "#" | "+"
answer  = "=" ws { token }                 ; §2.5
comment = "//" text                        ; §2.7
note    = [ "\" ] text                     ; §2.6, §2.8
token   = "@" name | "#" name | "^" name   ; §4.2, §4.3, §4.5
        | "!" | "!!" | "!!!"               ; §4.4
        | key ":" ( value | '"' text '"' ) ; §4.6
        | link | word                      ; §4.8, §4.9
link    = "[[" path [ "^" name ] "]]"
```

## Appendix B. Known ambiguities

Places where draft 0.2's reference parser and this text disagree, or where
neither says enough. Each needs a ruling and conformance examples before 1.0.

1. **Byte order mark.** §1.1 says a BOM is ignored. The reference parser treats
   it as whitespace, so the first line counts as indented: it becomes a note on
   the root and the file has no title.
2. **Tabs after spaces.** §3.1 counts leading tabs as two spaces. The reference
   parser expands only a run of tabs at the very start of a line; a tab after a
   space is counted as one character.
3. **Several anchors on one item.** §4.5 says anchors are unique in a file but not
   how many one item may carry. The reference parser keeps the last.
4. **Duplicate anchors.** When two items in a file share an anchor, the reference
   parser resolves references to the first and reports nothing. A diagnostic is
   planned (cairn 0012).
5. **Empty groups and references.** An empty group (§5.5) is not done, so
   `after:` pointing at it waits forever. That may be the wrong answer.
6. **Recurring parents.** §5.3 excludes any item with `every:` from progress. The
   reference parser excludes only recurring leaves; a recurring parent's children
   still count.
7. **Grafting a grafted branch.** By §8.7, `[[file^anchor]]` cannot reach an
   anchor that `file` itself grafted in from elsewhere. Whether it should is open.
8. **File names with spaces.** Links (§4.8) cannot name a file containing spaces:
   tokens split on whitespace first (§4.1), so `[[my file]]` becomes two words.
9. **Directories.** The reference parser has a flat namespace of file names, so
   relative paths in links and grafts (§4.8, §8.2) have never been exercised.
10. **Loop reporting.** When two files graft each other (§8.4), which graft line
   reports the loop depends on which file is read first.

## Appendix C. Settled arguments

The design decisions behind these rules are recorded as cairn items of type
`decision` in this repository, each with the case for, the case against, and when
to revisit it: `cairn list --view decisions`.
