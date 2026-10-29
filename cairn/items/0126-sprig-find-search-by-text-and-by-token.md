---
id: 126
uid: a8e65646-d694-40b6-b8dd-498314f9dc3a
title: 'sprig find: search by text and by token'
type: feature
status: done
milestone: v0.3
depends_on:
- 24
- 118
- 119
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p2
effort: m
area: cli
---

## Problem

`next` answers one question. People and scripts need others: everything Dana owns, every open question, what's due before Friday. The results should feed the write verbs directly.

## Proposal

`sprig find <query>`, where a query is words plus filters that are all ANDed: `@who`, `#tag`, `is:open|doing|done|blocked|ask`, `due:<fri`, `due:>2026-11-01`. Output goes through the output layer, so `--plain` puts refs first, and `--json` gives nodes in the tree schema. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [x] Each filter has a test over `examples/bakery/`.
- [x] `sprig find is:blocked --json` includes, for each item, what it waits on.
- [x] An unknown filter exits 2 and lists the valid ones.

## Result

sprig find: words, @who, #tag, is:STATE and due: with <, <=, >, >= or an exact date. Each filter is tested over the bakery; --json carries waits_on; an unknown filter exits 2 and lists the filters. Defined in spec/tools.md T5.1.
