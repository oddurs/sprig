---
id: 121
uid: cd5935da-866b-44e9-8dc6-23731f75e089
title: 'sprig why: explain what an item is waiting on'
type: feature
status: planned
milestone: v0.2
depends_on:
- 19
- 118
- 119
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: cli
---

## Problem

A plan says an item is blocked but not why, and the answer is often three files away: Tiling waits because Build waits on Sign the lease, which waits on a permit.

## Proposal

`sprig why <ref>` prints the blocker chain as a tree, including blocking inherited from a parent, with `file:line` and the owner of every link. Each leaf of the chain ends with its state: in progress, ready to start, or an open question. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] `sprig why Tiling` on `examples/bakery/` matches a committed snapshot that crosses from `kitchen.sprig` into `lease.sprig`.
- [ ] An item that is not blocked prints one line saying so and exits 0; a blocked item exits 1.
- [ ] A dependency cycle is printed once, marked as a cycle, and does not recurse.
