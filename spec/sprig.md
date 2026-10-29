# Sprig specification, draft 0.5

Sprig is a plain-text format for plans. A file is a tree of items, one per line;
the first character of a line says what the line is, and indentation says where
it belongs. Progress, what is blocked, and who owns what are computed from the
text and never written into it.

This document is dedicated to the public domain under
[CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Anyone may
implement it, copy it, or build on it, without asking.

## About this document

**Status.** Draft 0.5. It formalises draft 0.2, the version described and
implemented in `design/sprig-draft-0.2.html`. Draft 0.4 tightened three rules of
it: `due` is the one deadline field (§9.1), a future `start:` blocks (§7.5), and
grafted content appears once in a tree (§8.8). Draft 0.5 rules on every
ambiguity [Appendix B](#appendix-b-known-ambiguities) listed and on the cases
the conformance suite found the text silent on, and makes one change of
meaning: a dropped item no longer holds up what waits on it (§5.7).

**Rules are numbered.** Each rule has a number such as §3.1, so conformance
examples, diagnostics and discussions can cite it. A number never changes
meaning once published; a rule that is removed leaves its number unused.

**Changes.** A change to what a file means goes through an RFC (`rfcs/`), and
after 1.0 every change only adds: a file valid under 1.0 means the same to
every 1.x tool. `docs/stability.md` lists each promise and the test that keeps
it.

**Words.** *Must* and *must not* are requirements on an implementation. *Should*
is a strong recommendation that tools may depart from with a reason. A *tool* is
anything that reads Sprig: a parser, an editor, a renderer, an agent.
*Whitespace* is any Unicode `White_Space` character, such as a space, a tab or
a no-break space. *Letters* and *digits* are those of any script, as Unicode
defines them, so `@jón` and `#café` are names.

**Examples** are complete, valid Sprig files unless marked otherwise.

## 1. Files

### 1.1 Encoding

A Sprig file is UTF-8 text. Lines end with LF; a tool must read CRLF as LF, and a
byte order mark at the start of the file must be ignored: it is neither
indentation nor part of the first line's text. The file extension is `.sprig`.

```sprig
Groceries

- milk
- bread
```

### 1.2 The title

The first line that is not blank and not a comment (§2.7) is the **title**,
unless it is indented, an item (§2.3) or an answer (§2.5). The title is the
file's root item: its text names the file, and its tokens (§4) are the file's.
Its people own the whole file (§6.1), and its due date and priority reach every
item that has none of its own (§9.6, §4.4).

A title that would start with a mark is written with a backslash, which is
dropped as in an escaped note (§2.8): `\x marks the spot` is the title
*x marks the spot*.

```sprig
Open the bakery  @dana  due:2026-12-05  where:"41 Mill Street"

- Sign the lease
```

Here the title is *Open the bakery*, the file's owner is `dana`, and the file has
two fields, `due` and `where`.

### 1.3 A file without a title

If the first line that is not blank or a comment is indented, an item or an
answer, the file has no title.
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
is a done item. The marks are exactly these characters: `X` and `*` are not
marks, and a line starting with them is a note.

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
it attaches to the nearest item above it with less indentation, or to the root
if there is none, exactly as a note does (§2.6), and carries tokens (§4). A
question with at least one answer, even an empty `=`, is **answered**, and
counts as done (§5.2). An answer attached to any other kind of item records a
decision about that item and changes nothing else.

An answer is one line. It is not an item, so nothing attaches to it: a deeper
line below an answer belongs to the answer's item, as it would without the
answer.

```sprig
Naming

? Kiln & Crumb, or Mill Street Bread?
  = Kiln & Crumb. Trademark search is clear.  @dana
```

### 2.6 Notes

Any other line is a note. A note attaches to the nearest item above it with less
indentation, or to the root if there is none. Consecutive note lines of one item
form a paragraph, joined with single spaces. A blank line between them starts a
new paragraph, and so does a child item: notes either side of one are two
paragraphs.

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
rest of the line after the backslash, even if that text starts with a mark. A
backslash on its own is a note with no text.

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

A line's **indentation** is the number of spaces and tabs before its first other
character, where each tab counts as two spaces, wherever it falls among them.
A line belongs to the nearest item above it with strictly less indentation, or
to the root if there is none. Only relative indentation matters. Other
whitespace before the first character adds nothing to the indentation, but
still makes the line indented for §1.2.

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
spaces inside the quotes: when a `"` comes before a token's first whitespace,
the token runs to the next `"` and on to the whitespace after it. A token holds
at most one quoted run, and a `"` with no partner later on the line is an
ordinary character. Tokens are classified by §4.2 to §4.8; whatever is left is
the item's text (§4.9).

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

`@jo,` with a trailing comma is not a person; it is a word. So is an email
address, with or without a leading `@`: `@dana@example.org` and
`dana@example.org` are both words, since `@` is not a name character. A person
is a handle; matching handles to addresses is not the file's job.

### 4.3 Tags

A token that is `#` followed by letters, digits, `_` or `-` is a tag.

```sprig
Tags

- Coffee beans  #market #weekly
```

### 4.4 Priority

A token that is exactly `!`, `!!` or `!!!` sets the item's priority to 1, 2 or 3.
If several appear, the highest wins. Priority reaches down: an item's
**effective priority** is the highest written on it or on any item above it,
the title included, and tools that order work use it.

```sprig
Priority

- Renew the insurance  !!!
- Tidy the shed  !
```

### 4.5 Anchors

A token that is `^` followed by letters, digits, `_` or `-` gives the item an
**anchor**: a name other lines can refer to (§7.2, §8.3). Anchors should be
unique within a file. An item that writes several is named by the last, and
when two items write the same anchor, references and grafts name the first.
A tool should warn about both.

```sprig
Anchors

- Book a van  ^van
- Move  after:^van
```

### 4.6 Fields

A token of the form `key:value` is a field when the key starts with a letter and
continues with letters, digits, `_` or `-`, and the value is not empty. The
token splits at its first colon, so `key:a:b` has the value `a:b`. Keys are
case-insensitive. A value that starts and ends with a double quote loses them,
so `vendor:""` is a field whose value is empty. A value starting with `//` is
not a field, so URLs stay words. If a key appears twice, the last value wins,
except for `after` (§4.7).

The keys `due`, `start` (§9), `est` and `every` (§10) have defined meanings. Any other key is kept and shown.

The key `sprig` is reserved: on the title it names the major version of the
format a file is written for, and nowhere else does it mean anything. A file
needs no version, and one without is read by the current version. Every 1.x
file means the same to every 1.x tool, so `sprig:1` changes nothing. A tool
that meets a version it does not read should warn, then read the file as best
it can.

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
it. The file name is resolved relative to the directory of the file containing
the link, with `/` between directories and `..` for the one above; `.sprig` is
appended when absent, and a leading `./` is ignored. A link names a file, so
`[[^anchor]]` is not a link, and its path cannot contain whitespace, since
tokens split there first (§4.1). A link may appear inside a longer token. In an
item's text a link is shown as a link; it has meaning only in a graft (§8) or
an `after:` value (§7).

```sprig
Links

- Supplier credit  after:[[lease^signed]]
- Read the notes in [[kitchen]] first
```

### 4.9 Text

The item's text is its remaining tokens joined by single spaces. In the plain
text of an item, a link loses its brackets: `[[kitchen]]` reads as `kitchen`.
Brackets that do not make a link (§4.8), such as `[[my notes]]` or `[[^a]]`,
are kept, so the text shows that nothing is linked.

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
count. Neither do groups or grafts with no children, nor any leaf with an
`every:` field (§10.3), which is upkeep rather than progress. A recurring item
with children is a checklist that recurs: it is a parent like any other, and its
children count.

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
anything is done or doing, and otherwise **to do**. Empty is not done: nothing
has been finished.

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

An open item is **blocked** when any of its own references is unfinished (not
finished by §5.7, or unknown), when its `start:` is after today (§7.5), or when
any ancestor is blocked. Blocking passes to every open descendant of a blocked item.

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

### 7.5 Start dates

An open item whose `start:` is after today is blocked until that day, and the date
is what it waits on. On its start date it is no longer blocked by it. Like a
reference, a start date blocks the item's open descendants (§7.3).

```sprig
Start dates

- Book the removal van  start:2026-11-02
- Winter tyres  start:2026-11-15
  - Book the garage
```

Read on 2026-10-01, all three are blocked; *Book the garage* waits on its
parent's date.

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

## 9. Dates

### 9.1 Date fields

`due` and `start` hold dates. `due` is when an item should be finished; `start`
is the first day work on it can begin (§7.5). Any other key is an ordinary field
(§4.6), even when its value looks like a date.

```sprig
Dates

- Apply for the permit  start:2026-10-05  due:2026-10-30
```

### 9.2 Stored form

A stored date is `YYYY-MM-DD`. A tool that writes a date must write this form, so a
file means the same thing on every day it is read. A date has no time of day:
a plan's grain is the day, and a time would need a time zone to mean one
instant. A value such as `2027-03-01T14:00` is unreadable (§9.5).

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

Month and weekday names match on a prefix of at least three letters, and
shortcuts ignore case: `Fri`, `OCT3` and `Q4` read like `fri`, `oct3` and
`q4`. A count is written in digits and may be zero: `+0d` is today. A month and
day that do not exist, such as `feb30`, are not a date. `feb29` follows the
120-day rule as if the day existed this year, and is a date only when the year
it lands in has one.

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

## 10. Estimates and recurrence

### 10.1 Estimates

`est:` takes a number and a unit: `h` hours, `d` days of 8 hours, `w` weeks of 40
hours. The number is one or more digits, optionally followed by a point and more
digits (`2`, `0.5`, but not `.5`), and the unit's case does not matter. A value
in any other form is ignored, and a tool should warn about it.

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
`weekly`, `month` or `monthly`, a count of one or more days or weeks such as
`3d` or `2w`, or a weekday name; case does not matter. A recurring leaf is upkeep and never
counts toward progress (§5.3). An item whose `every:` is none of these is still
recurring, has no next occurrence, and a tool should warn about it.

```sprig
Upkeep

- Weekly numbers to the bank  every:fri  due:2026-10-02
- Water the plants  every:3d
```

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
