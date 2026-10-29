---
id: 155
uid: aab5957e-4fed-459a-82a0-243e4a72fd30
title: Simplicity, enforced
type: feature
status: done
milestone: v1.0
created: 2026-10-29
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: l
area: docs
---

## Problem

Sprig's essence is simplicity, and nothing in the repository protects it. Every other promise has a test; the size of the format, the command and the spec only ever grew, each addition reasonable on its own.

## Approach

Treat simplicity as an invariant with a test, as purity and stability are: a budget of counts that a change must raise by an explicit edit. Then spend it: one card for the whole format, a short spec core with the rulings in an annex, a first contribution that needs nothing but `scripts/task check`, and a home page that leads with one file.

## Acceptance criteria

- [x] A test pins the number of marks, commands, flags, field keys, diagnostic codes and MCP tools, and the length of the format card; raising one fails until the number is edited.
- [x] The whole format fits on one card, generated from the spec and shown in the README and on the site; the RFC template asks what a proposal adds to it.
- [x] The spec opens with a normative core readable in one sitting, under 3,400 words of prose and held there by the budget; rulings on edge cases are in an annex, with every rule number and conformance citation still valid.
- [x] CONTRIBUTING.md starts with clone, check and a pull request; the cairn and worktree loop follows for people who stay.
- [x] The home page leads with one plan and what Sprig says about it.

## 2026-10-29

The budget counts from the things themselves: the marks from §2.2's table, the fields from §4.6's sentence plus after, the commands and flags from --help, the codes from the catalogue and the tools from a tools/list call; a mark added to §2.2 in a trial run failed it with the intended message. The spec criterion first said readable in ten minutes; moving the fourteen edge-case rulings to the annex brought the core to about 3,300 words of prose, which is a sitting rather than ten minutes, so the criterion now names the measurable ceiling the budget holds, 3,400 words. Making the home page show its source on a phone exposed two touch problems the mobile check had never seen, because the source was hidden there: a tab under 44px and a 14px editor at tablet width. Both are fixed.

## Result

Simplicity is a tested budget; the format is one card, the spec a core and an annex, the first contribution needs only Rust, and the home page leads with a plan.
