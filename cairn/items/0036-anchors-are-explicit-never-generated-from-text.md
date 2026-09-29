---
id: 36
uid: 3d911350-cf20-49c0-81f0-8d8628610532
title: Anchors are explicit, never generated from text
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

Generated ids save typing.

## Options and tradeoffs

Rewording an item would silently break every link to it.

## Decision

`^anchors` are written by hand, and only items that something points at need one.

## Revisit when

Never.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
