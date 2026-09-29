---
id: 107
uid: 6756c2d0-fb67-4171-a0c6-f77d387f5235
title: Social image, favicon, 404, sitemap, robots and llms.txt
type: chore
status: planned
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: site
---

## Purpose

The edges a careful visitor, a link preview or a crawler checks.

## Approach

A 1280x640 social preview image, an SVG favicon drawn from the Sprig mark, a 404 page written in Sprig, a sitemap, robots.txt and an llms.txt that points agents at the spec.

## Acceptance criteria

- [ ] Every page has Open Graph and Twitter card metadata pointing at the social image.
- [ ] The 404 page is a valid Sprig plan rendered by the parser.
- [ ] `/llms.txt` links to `/spec.md`.
