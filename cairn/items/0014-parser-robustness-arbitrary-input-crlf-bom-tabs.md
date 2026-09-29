---
id: 14
uid: 02742896-a772-40db-987b-a755b63f135c
title: 'Parser robustness: arbitrary input, CRLF, BOM, tabs'
type: feature
status: planned
milestone: v0.1
depends_on:
- 13
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: core
---

## Problem

Plans arrive pasted from everywhere. A parser that panics on one odd byte breaks every tool built on it.

## Proposal

Property tests over arbitrary UTF-8 and over mutated conformance inputs. CRLF, a leading BOM and tab indentation all parse to the same tree as their clean equivalents.

## Acceptance criteria

- [ ] A property test of 10,000 arbitrary inputs finds no panic.
- [ ] CRLF and BOM variants of every lines example produce identical trees.
- [ ] A tab counts as two spaces of indentation, as the spec says.
