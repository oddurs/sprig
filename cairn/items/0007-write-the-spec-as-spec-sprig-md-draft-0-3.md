---
id: 7
uid: 83eb5c15-84d9-42f9-b8d4-02348ead3bed
title: Write the spec as spec/sprig.md, draft 0.3
type: docs
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p0
effort: m
area: spec
---

## Reader and question

Someone implementing a parser who has never seen the playground. Today the language exists only as `design/sprig-draft-0.2.html`, a web page with a reference table, nine parsing steps, an informal grammar and a JavaScript parser.

## Change

Turn the page into `spec/sprig.md`. Number every rule (`§2.3`) so conformance examples and diagnostics can cite them. Keep the page's order: title, line starts, indentation, tokens, status and progress, people, blocking, grafts, dates. Each rule gets one inline example. The JavaScript parser in the page is the behavioural oracle for draft 0.2: where prose and code disagree, record it.

## Acceptance criteria

- [x] Every row of the page's reference tables and each of its nine parsing steps appears as a numbered rule.
- [x] The informal grammar is included and matches the rules.
- [x] Every rule has at least one inline example.
- [x] A 'Known ambiguities' section lists every place the prose and the reference parser disagree, or says there are none.
- [x] The file carries a CC0 notice.

## 2026-09-28

spec/sprig.md written as draft 0.3: 10 sections, 53 numbered rules as headings (one anchor each), the informal grammar in Appendix A, 10 known ambiguities in Appendix B. Checked by running every sprig example through the draft 0.2 reference parser (design/sprig-draft-0.2.html): 53 of 53 rules have an example, none produce unexpected diagnostics (the only ones are cross-file references to files absent from a single-file check), and the stated results for 1.4, 2.6, 3.1, 5.3 to 5.7, 6.1 and 10.2 match the parser's output. Writing it surfaced one real parser behaviour worth recording: a BOM is treated as indentation, so a file starting with one loses its title (Appendix B.1).

## Result

spec/sprig.md, draft 0.3: every rule of draft 0.2 numbered, each with an example checked against the reference parser, plus ten recorded ambiguities for 1.0 to settle.
