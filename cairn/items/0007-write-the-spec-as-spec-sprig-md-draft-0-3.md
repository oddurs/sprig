---
id: 7
uid: 83eb5c15-84d9-42f9-b8d4-02348ead3bed
title: Write the spec as spec/sprig.md, draft 0.3
type: docs
status: planned
milestone: v0.1
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: spec
---

## Reader and question

Someone implementing a parser who has never seen the playground. Today the language exists only as `design/sprig-draft-0.2.html`, a web page with a reference table, nine parsing steps, an informal grammar and a JavaScript parser.

## Change

Turn the page into `spec/sprig.md`. Number every rule (`§2.3`) so conformance examples and diagnostics can cite them. Keep the page's order: title, line starts, indentation, tokens, status and progress, people, blocking, grafts, dates. Each rule gets one inline example. The JavaScript parser in the page is the behavioural oracle for draft 0.2: where prose and code disagree, record it.

## Acceptance criteria

- [ ] Every row of the page's reference tables and each of its nine parsing steps appears as a numbered rule.
- [ ] The informal grammar is included and matches the rules.
- [ ] Every rule has at least one inline example.
- [ ] A 'Known ambiguities' section lists every place the prose and the reference parser disagree, or says there are none.
- [ ] The file carries a CC0 notice.
