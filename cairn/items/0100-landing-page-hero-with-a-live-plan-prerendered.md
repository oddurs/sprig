---
id: 100
uid: 8ba577fd-55f2-456d-a601-7a6d3dd55e68
title: Landing page hero with a live plan, prerendered
type: feature
status: done
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p0
effort: m
area: site
---

## Problem

The first screen has to show the format and what it computes, and prove it's real, before any script runs.

## Proposal

The hero shows an editable plan beside its computed view. The computed view is rendered into the HTML at build time; a small script loads the parser on the first interaction. On narrow screens the computed view comes first and tapping a mark is the interaction.

## Acceptance criteria

- [x] With JavaScript disabled, the hero shows the example plan and its computed view.
- [x] No parser code loads until the visitor interacts with the hero; the first interaction loads it and the view updates.
- [x] Ticking an item in the computed view changes the source, and editing the source updates the view.
- [x] Below 700px the computed view comes first and the source opens behind a button.

## 2026-09-28

Verified in headless Chrome: the hero's computed view is in the static HTML (it reads correctly with JavaScript off); no parser chunk loads before interaction (0 resource entries); ticking Migration guide and adding an answer takes the stats from 1/5 done, 1 waiting to 3/5 done, 0 waiting with Announce ready, and the source gains 'x Migration guide'. At 390px (measured in a 390px iframe, since headless Chrome won't lay out narrower than 500px) the computed view comes first and the source sits behind Edit the source, with no horizontal overflow.

## Result

The hero shows a live plan beside its computed view, prerendered at build time, with the parser loaded only on first interaction.
