---
id: 121
uid: cd5935da-866b-44e9-8dc6-23731f75e089
title: 'sprig why: explain what an item is waiting on'
type: feature
status: done
milestone: v0.2
depends_on:
- 19
- 118
- 119
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: cli
---

## Problem

A plan says an item is blocked but not why, and the answer is often three files away: Tiling waits because Build waits on Sign the lease, which waits on a permit.

## Proposal

`sprig why <ref>` prints the blocker chain as a tree, including blocking inherited from a parent, with `file:line` and the owner of every link. Each leaf of the chain ends with its state: in progress, ready to start, or an open question. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [x] `sprig why Tiling` on `examples/bakery/` matches a committed snapshot that crosses from `kitchen.sprig` into `lease.sprig`.
- [x] An item that is not blocked prints one line saying so and exits 0; a blocked item exits 1.
- [x] A dependency cycle is printed once, marked as a cycle, and does not recurse.

## 2026-10-28

`sprig why REF [--in PATH]`, built on the core's waits, which now name the file and line they wait on. (1) `why Tiling` (`tests/snapshots/why-tiling.txt`): Tiling at kitchen.sprig:22 is inside Build (kitchen.sprig:18), which waits on Design and on Sign the lease at lease.sprig:13, which waits on Negotiate terms (1 of 3 done) and the permit (ready to start). Every link shows `file:line` and its owner. (2) `^permit` is not blocked: two lines, the item and "not blocked: ready to start", and exit 0; a blocked item exits 1. (3) Two items waiting on each other print the loop once, as "a cycle: this was already on the chain", and stop. Start dates print as "starts DATE", and unknown targets as "can't find".

## Result

`sprig why` explains a block across files, link by link, without looping.
