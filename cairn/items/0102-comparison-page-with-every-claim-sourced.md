---
id: 102
uid: 3b141544-67c3-4acc-b1c0-80eb0867bba9
title: Comparison page with every claim sourced
type: docs
status: done
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p0
effort: s
area: site
---

## Reader and question

A reader deciding between Sprig and the tool they already use, who will check any claim they doubt.

## Change

`/compare/`: Markdown task lists, todo.txt, TaskPaper, Org-mode and Taskwarrior, each with where it is better than Sprig, and a small table. Every claim about another tool links to that tool's own documentation.

## Acceptance criteria

- [x] Every row in the table has a source link to the other tool's documentation.
- [x] Each alternative has a sentence saying what it does better than Sprig.

## 2026-09-28

/compare/ covers Markdown task lists, todo.txt, TaskPaper, Org-mode and Taskwarrior. Every table row has a Sources link, and each section links the tool's own docs: GFM task list items, the todo.txt format repository, the TaskPaper getting-started guide, the Org manual's TODO dependencies and agenda views pages, and Taskwarrior's urgency and upgrade-3 pages. Each URL returned 200 and its text was checked for the claim cited (for example org-enforce-todo-dependencies and ORDERED; taskchampion.sqlite3). Each tool has a Better than Sprig at paragraph.

## Result

A comparison page where every claim links the other tool's own documentation and each alternative gets its due.
