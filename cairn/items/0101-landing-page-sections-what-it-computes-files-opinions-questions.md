---
id: 101
uid: dfe57c76-758b-4ac8-aeed-752f0f431e79
title: 'Landing page sections: what it computes, files, opinions, questions'
type: feature
status: done
milestone: launch
depends_on:
- 100
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p0
effort: m
area: site
---

## Problem

One concept per block, each shown rather than claimed, so a sceptical reader gets the reason to switch without reading marketing.

## Proposal

Four blocks after the hero: what it computes, plans across files at every size, opinions with three settled arguments, and questions answered in two sentences each. Then a footer that says what the site doesn't do.

## Acceptance criteria

- [x] Every code example on the page is a valid Sprig file that the reference parser reads without diagnostics.
- [x] The questions block answers xkcd 927, Org-mode, Markdown checkboxes, AI, accounts, Rust, stability and phones.
- [x] The footer states that there are no cookies and no analytics, and that is true of the built site.

## 2026-09-28

Blocks: what it computes (the three result sentences are computed at build by the parser), plans across files (house and kitchen, rendered with the graft resolved, plus four sizes that open in the playground), opinions (three rulings linking to their anchors on /decisions/), and eight questions covering xkcd 927, Org-mode, Markdown checkboxes, AI, accounts, Rust, stability and phones. All 15 example plans on the site parse with no diagnostics. The footer's no-cookies, no-analytics, no-third-party claim is enforced by check.mjs; the site sets no cookies.

## Result

Four concept blocks after the hero, every example computed or checked by the parser, and a footer whose claims the build enforces.
