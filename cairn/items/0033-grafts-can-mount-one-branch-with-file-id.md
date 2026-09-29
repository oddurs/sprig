---
id: 33
uid: 2a621a1a-fee4-416a-9ec8-b6a38e90f940
title: Grafts can mount one branch with [[file^id]]
type: decision
status: done
created: 2026-09-28
updated: 2026-09-29
priority: p2
area: spec
---

## Context

Whole-file grafts only is simpler, but a person's week or a release is naturally built from parts of several plans.

## Options and tradeoffs

Files only (one owner, simple) or branches too (flexible, and a branch grafted twice counts twice).

## Decision

`+ [[file^id]]` mounts that branch. Ticking a grafted item edits the original line. Grafting the same branch twice into one tree is the author's error.

## Revisit when

If double counting causes real confusion; then `sprig check` should warn on it.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.

## 2026-09-29

Tightened by 0140: a repeated graft in one tree is now an error on the later graft line (§8.8), which settles the double counting this decision accepted.
