---
id: 137
uid: b70540eb-ac1d-4685-8160-566deaae5cda
title: 'Spec draft 0.4: one deadline field, start dates that hold, grafts that appear once'
type: docs
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-09-29
updated: 2026-09-29
closed_at: 2026-09-29
priority: p1
effort: s
area: spec
---

## Reader and question

Anyone implementing Sprig, or writing it by hand, asks what each date field does and what happens when a plan grafts the same content twice. Draft 0.3 had two date words for the same meaning, a date field with no effect, and a graft case left to "the author's error".

## Change

- `target:` is no longer a date field; `due:` is the one deadline (§4.6, §9.1).
- An open item whose `start:` is after today is blocked until that day (§7.3, §7.5).
- Within one tree an item appears once; a graft that would repeat content is an error on the later graft line (§8.8). Appendix B item 7 is resolved by making §8.7 explicit.
- The examples, the README and the site's reference parser follow.

## Acceptance criteria

- [x] The reader can complete the task described with the current tools.
- [x] Each change has a decision item with the case for, the case against and a ruling.
- [x] The site's reference parser implements all three, shown by a script run recorded in the pull request.

## 2026-09-29

Checked with a script over the site parser: the §7.5 example is fully blocked on 2026-10-01 and ready on 2026-11-15; the four §8.8 repeat cases (file then branch, branch then file, same branch twice, a file nested in an earlier graft) each report on the later line and progress counts once, while distinct branches pass; target: parses as an ordinary field; examples/bakery checks clean. Appendix B.7 kept its number and now records its resolution, so citations stay stable. Decisions 0138 to 0140; 0033 carries a note pointing at 0140.

## Result

Spec draft 0.4: due is the one deadline, a future start: blocks, grafted content appears once in a tree; examples, README and the site parser follow.
