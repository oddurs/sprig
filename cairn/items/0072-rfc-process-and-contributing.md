---
id: 72
uid: 1878270e-2cbe-4fc0-848f-3a9b018cfbbe
title: RFC process and CONTRIBUTING
type: docs
status: done
milestone: v1.0
depends_on:
- 68
- 69
- 70
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: s
area: community
---

## Reader and question

Someone who wants to change the spec after 1.0 and needs to know how.

## Change

`CONTRIBUTING.md` and `rfcs/0000-template.md`. An RFC has the same shape as the settled arguments: the question, the case for, the case against, the ruling. Accepted rulings are recorded as cairn decision items.

## Acceptance criteria

- [x] The answers to the three spec-question spikes go through the process as the first RFCs.
- [x] CONTRIBUTING says who decides and how long review stays open.

## Result

rfcs/ holds the process (README), the template (question, case for, case against, ruling) and RFCs 0001 to 0003, the three spec spikes' answers, each with a decision item (0148, 0149, 0150). CONTRIBUTING says the maintainer decides, with reasons, after 14 days of review, or 7 for a clarification.
