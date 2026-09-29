---
id: 107
uid: 6756c2d0-fb67-4171-a0c6-f77d387f5235
title: Social image, favicon, 404, sitemap, robots and llms.txt
type: chore
status: done
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p1
effort: s
area: site
---

## Purpose

The edges a careful visitor, a link preview or a crawler checks.

## Approach

A 1280x640 social preview image, an SVG favicon drawn from the Sprig mark, a 404 page written in Sprig, a sitemap, robots.txt and an llms.txt that points agents at the spec.

## Acceptance criteria

- [x] Every page has Open Graph and Twitter card metadata pointing at the social image.
- [x] The 404 page is a valid Sprig plan rendered by the parser.
- [x] `/llms.txt` links to `/spec.md`.

## 2026-09-28

A 1280x640 social image (site/src/assets/og.svg rendered to public/og.png with rsvg-convert), an SVG favicon and logo from the Sprig mark, a 404 page that is a Sprig plan rendered by the parser, the Starlight sitemap, robots.txt (effective once the site has its own domain), and llms.txt linking /spec.md. Every built page has og:image and twitter:image metadata (0 pages without).

## Result

Social image, favicon, a 404 written in Sprig, sitemap, robots.txt and llms.txt, with preview metadata on every page.
