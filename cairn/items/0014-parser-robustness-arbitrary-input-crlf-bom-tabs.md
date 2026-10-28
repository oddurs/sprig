---
id: 14
uid: 02742896-a772-40db-987b-a755b63f135c
title: 'Parser robustness: arbitrary input, CRLF, BOM, tabs'
type: feature
status: done
milestone: v0.1
depends_on:
- 13
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: s
area: core
---

## Problem

Plans arrive pasted from everywhere. A parser that panics on one odd byte breaks every tool built on it.

## Proposal

Property tests over arbitrary UTF-8 and over mutated conformance inputs. CRLF, a leading BOM and tab indentation all parse to the same tree as their clean equivalents.

## Acceptance criteria

- [x] A property test of 10,000 arbitrary inputs finds no panic.
- [x] CRLF and BOM variants of every lines example produce identical trees.
- [x] A tab counts as two spaces of indentation, as the spec says.

## 2026-10-28

`crates/sprig-core/tests/robustness.rs`. (1) A proptest runs 10,000 cases, each resolving two arbitrary strings both ways round: any UTF-8 and text built from Sprig's own punctuation. Another runs 2,000 cases of randomly edited conformance inputs. No panic in either. (2) Every lines example reads to the same tree, spans aside, as LF, CRLF, with a BOM, and with both. (3) Leading spaces swapped for tabs, two spaces to a tab, read the same, and a tab after spaces counts as two (§3.1, example 049). Two related guards: a 400-file graft chain finds its loop without recursion, and the CLI runs the core on a 256 MiB stack, tested with a 3,000-level plan, because nesting depth is bounded only by file size.

## Result

Fuzzed with 12,000 inputs and no panic; CRLF, BOM and tab variants of every lines example give identical trees.
