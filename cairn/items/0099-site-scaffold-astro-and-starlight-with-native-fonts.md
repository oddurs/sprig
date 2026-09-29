---
id: 99
uid: aa1a59d1-65ad-49ff-bf4b-802ed2ecc976
title: 'Site scaffold: Astro and Starlight with native fonts'
type: chore
status: planned
milestone: launch
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: site
---

## Purpose

Everything on the site needs a home, and the plan's budget (no framework runtime on the landing page, no third-party requests) rules out most defaults.

## Approach

An Astro project in `site/` with Starlight for the docs, system font stacks only, design tokens shared between the landing page and the docs, and a `scripts/task site` target that CI runs.

## Acceptance criteria

- [ ] `scripts/task site` builds the site from a clean checkout, and CI runs it on every pull request.
- [ ] The built site makes no request to any host other than its own, verified by searching the output for external `src` and `href` URLs.
- [ ] No font files are shipped; every face is a system stack.
- [ ] Internal links are validated at build time, and a broken one fails the build.
