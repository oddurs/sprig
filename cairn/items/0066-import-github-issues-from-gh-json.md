---
id: 66
uid: bf72a669-57d9-4a27-bb0a-f7c5b6aecac5
title: Import GitHub Issues from gh JSON
type: feature
status: done
milestone: v0.3
depends_on:
- 65
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p2
effort: m
area: interop
---

## Problem

Teams on GitHub Issues want to see their issues as a plan without Sprig touching the network.

## Proposal

Read the output of `gh issue list --json number,title,state,labels,assignees,milestone,body` from a file. Labels become tags, assignees become people, milestones become groups, closed issues become `x`, and each item carries `issue:NUMBER`.

## Acceptance criteria

- [x] A fixture from a real repository converts and passes `sprig check`.
- [x] The importer makes no network calls; a test runs it with networking unavailable.

## 2026-10-29

The fixture is this repository's own pull requests (30 of them), in gh pr list --json form; the repository has no issues. gh pr list says MERGED where gh issue list says CLOSED, so both count as done, and an issue closed as not planned is dropped. The no-network test runs the importer under unshare -rn, a namespace with no network devices, where Linux allows one.

## Result

sprig import --from github converts gh issue list or gh pr list JSON: milestones become groups, labels tags, assignees people, and each item carries issue:NUMBER.
