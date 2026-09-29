---
id: 100
uid: 8ba577fd-55f2-456d-a601-7a6d3dd55e68
title: Landing page hero with a live plan, prerendered
type: feature
status: planned
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: site
---

## Problem

The first screen has to show the format and what it computes, and prove it's real, before any script runs.

## Proposal

The hero shows an editable plan beside its computed view. The computed view is rendered into the HTML at build time; a small script loads the parser on the first interaction. On narrow screens the computed view comes first and tapping a mark is the interaction.

## Acceptance criteria

- [ ] With JavaScript disabled, the hero shows the example plan and its computed view.
- [ ] No parser code loads until the visitor interacts with the hero; the first interaction loads it and the view updates.
- [ ] Ticking an item in the computed view changes the source, and editing the source updates the view.
- [ ] Below 700px the computed view comes first and the source opens behind a button.
