---
id: 99
uid: aa1a59d1-65ad-49ff-bf4b-802ed2ecc976
title: 'Site scaffold: Astro and Starlight with native fonts'
type: chore
status: done
milestone: launch
assignee: Oddur Sigurdsson
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p0
effort: s
area: site
---

## Purpose

Everything on the site needs a home, and the plan's budget (no framework runtime on the landing page, no third-party requests) rules out most defaults.

## Approach

An Astro project in `site/` with Starlight for the docs, system font stacks only, design tokens shared between the landing page and the docs, and a `scripts/task site` target that CI runs.

## Acceptance criteria

- [x] `scripts/task site` builds the site from a clean checkout, and CI runs it on every pull request.
- [x] The built site makes no request to any host other than its own, verified by searching the output for external `src` and `href` URLs.
- [x] No font files are shipped; every face is a system stack.
- [x] Internal links are validated at build time, and a broken one fails the build.

## 2026-09-28

Astro 7 with Starlight 0.42 in site/, system font stacks only (Starlight's defaults plus tokens in src/styles/tokens.css). scripts/task site runs npm ci and the build; CI has a site job that runs it. The build ends with site/scripts/check.mjs, which resolves every internal link and #fragment in dist (533 at last run) and fails on any third-party script, stylesheet, image, media, CSS url() or shipped font file. That checker replaced starlight-links-validator, which skips relative links and can't see the hand-built pages.

## Result

The site builds from a clean checkout with scripts/task site; CI runs it; the build fails on a broken link, a missing anchor, a third-party request or a font file.
