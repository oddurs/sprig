---
id: 66
uid: bf72a669-57d9-4a27-bb0a-f7c5b6aecac5
title: Import GitHub Issues from gh JSON
type: feature
status: planned
milestone: v0.3
depends_on:
- 65
created: 2026-09-28
updated: 2026-09-28
priority: p2
effort: m
area: interop
---

## Problem

Teams on GitHub Issues want to see their issues as a plan without Sprig touching the network.

## Proposal

Read the output of `gh issue list --json number,title,state,labels,assignees,milestone,body` from a file. Labels become tags, assignees become people, milestones become groups, closed issues become `x`, and each item carries `issue:NUMBER`.

## Acceptance criteria

- [ ] A fixture from a real repository converts and passes `sprig check`.
- [ ] The importer makes no network calls; a test runs it with networking unavailable.
