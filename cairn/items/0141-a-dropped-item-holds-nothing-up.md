---
id: 141
uid: b3279427-d408-4039-a5e4-7f03c78796d1
title: A dropped item holds nothing up
type: decision
status: done
created: 2026-10-28
updated: 2026-10-28
priority: p2
area: spec
---

## Context

§5.7 counted only done as finished, so an `after:` naming a dropped item waited forever (`graph/023`). Dropping something is a decision that it will not happen, and the only way to unblock the item waiting on it was to edit its `after:` by hand.

## Options and tradeoffs

Keep dropped unfinished: a dependency that was decided against is a reason to rethink what depended on it, and blocking forces that rethink. But it forces it silently, as a blocker that can never clear, which is how plans rot. Count dropped as finished: the waiting item becomes ready, and the reference still records what it once waited on. Taskwarrior stops a deleted task blocking its dependants, OmniFocus treats dropped actions as complete for availability in sequential projects, and Jira's "is blocked by" no longer blocks once the blocker is resolved as Won't Do. Later (`>`) is different: it is put off, not decided against, so it still holds things up.

## Decision

A node is finished for references when it is done or dropped (§5.7). Later and empty nodes are not finished.

## Revisit when

Real plans show people relying on a dropped prerequisite to hold its dependants, and missing that it no longer does.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
