# Sprig specification, draft 0.5

Sprig is a plain-text format for plans. A file is a tree of items, one per line;
the first character of a line says what the line is, and indentation says where
it belongs. Progress, what is blocked, and who owns what are computed from the
text and never written into it.

This document is dedicated to the public domain under
[CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Anyone may
implement it, copy it, or build on it, without asking.

## The format on one card

<!-- card:start -->
| A line that starts with | is |
| --- | --- |
| `- ` | to do |
| `~ ` | doing |
| `x ` | done |
| `/ ` | dropped, kept for the record |
| `> ` | later, parked on purpose |
| `? ` | a question |
| `= ` | the answer to the item above |
| `# ` | a group |
| `+ ` | another file or branch, grafted in: `+ [[kitchen]]` |
| anything else | a note on the item above; `//` starts a comment |

Indent a line to put it under the one above. On an item:

| Token | means |
| --- | --- |
| `@name` | who owns it |
| `#tag` | a tag |
| `!` `!!` `!!!` | its priority |
| `^anchor` | its name, for other items to point at |
| `after:^anchor` | it waits on that item |
| `due:2026-10-17` `start:fri` | when it is due, and when it can start |
| `est:3h` | how big it is |
| `every:fri` | it comes round again |
| `[[file]]` `[[file^anchor]]` | a link |
| `key:value` | any other field, kept and shown |

Progress, what is blocked and who is on what are worked out from the text,
never written into it.
<!-- card:end -->

That is the whole format. The rules below say exactly what each line and token
means, with an example each; [the annex](annex.md) holds the rulings on edge
cases, under the same numbers, and the appendices.

## About this document

**Status.** Draft 0.5. Its history, from draft 0.2 on, is in the annex.

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
