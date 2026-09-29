---
id: 108
uid: 0c54da9e-c4f5-4e0c-a040-ba42bdb237a4
title: Deploy the site to GitHub Pages from main
type: chore
status: done
milestone: launch
depends_on:
- 99
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p0
effort: s
area: site
---

## Purpose

A site that only builds in CI isn't a website.

## Approach

A workflow that builds the site on every push to main and deploys it to GitHub Pages at `oddurs.github.io/sprig` until the name spike settles the domain. The base path comes from one setting, so moving to a domain is one change.

## Acceptance criteria

- [x] A merge to main deploys, and the deployed site's links work under the `/sprig/` base path.
- [x] The workflow has the least permissions Pages needs.

## 2026-09-28

First deploy from main after #7: pages workflow run 36510315800, build and deploy both succeeded. Live at https://oddurs.github.io/sprig/: every page (/, /play/, /spec/, /docs/first-plan/, /decisions/, /compare/, /roadmap/, /changelog/) and /spec.md, /feed.xml, /llms.txt, /og.png, /favicon.svg, /sitemap-index.xml and the Pagefind index returned 200; every internal link on the live front page resolved under /sprig/; an unknown URL serves the Sprig 404; the live /spec.md is byte-identical to spec/sprig.md. The workflow's permissions are contents: read at the top, with pages: write and id-token: write only on the deploy job. The base path is one setting (SITE_BASE, default /sprig) in site/astro.config.mjs.

## Result

Every push to main deploys the site to GitHub Pages at oddurs.github.io/sprig, verified live after the first deploy.
