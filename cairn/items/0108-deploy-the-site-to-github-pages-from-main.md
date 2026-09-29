---
id: 108
uid: 0c54da9e-c4f5-4e0c-a040-ba42bdb237a4
title: Deploy the site to GitHub Pages from main
type: chore
status: planned
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: site
---

## Purpose

A site that only builds in CI isn't a website.

## Approach

A workflow that builds the site on every push to main and deploys it to GitHub Pages at `oddurs.github.io/sprig` until the name spike settles the domain. The base path comes from one setting, so moving to a domain is one change.

## Acceptance criteria

- [ ] A merge to main deploys, and the deployed site's links work under the `/sprig/` base path.
- [ ] The workflow has the least permissions Pages needs.
