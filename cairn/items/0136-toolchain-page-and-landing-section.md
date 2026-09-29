---
id: 136
uid: 2ac47703-49e3-498a-8124-ed711569f9d5
title: Toolchain page and landing section
type: feature
status: done
milestone: launch
assignee: Oddur Sigurdsson
depends_on:
- 115
created: 2026-09-29
updated: 2026-09-29
closed_at: 2026-09-29
priority: p1
effort: m
area: site
---

## Problem

The site explains the format but not the tools. A Show HN reader's second question, after "what is it", is "what do I run", and today the answer is one sentence under the live example. The CLI plan (`design/cli-plan.html`, items 0116 to 0135) has no home on the site.

## Proposal

A `/tools/` page, linked from the header and the docs sidebar, that presents the toolchain honestly: one core, every surface, each tool with the milestone it lands in and nothing described as shipped that isn't. Every terminal screen is computed at build time by the reference parser from `examples/bakery/` with a pinned `--today`, so it cannot go stale or drift from the parser. The TUI is shown as a working keyboard demo. The landing page gets a short section pointing at it.

## Acceptance criteria

- [x] `/tools/` states what exists today and when each tool lands, matching the roadmap's milestones.
- [x] Every terminal screen on the page is rendered from `examples/bakery/` by the reference parser at build time, with the date pinned in the command shown.
- [x] The TUI demo works by keyboard and by the on-screen keys, and ships no JavaScript until it is used.
- [x] The header, the docs sidebar, the landing page and `llms.txt` link to the page.
- [x] The build gates pass: contrast, token lint, links, and axe in both themes.

## 2026-09-29

Terminal screens come from site/src/lib/terminal.js, pinned to 2026-10-01 with sprig.withToday; the TUI model in lib/tui.js renders the first frame at build time and loads in the browser on the first key (857 bytes of script up front). Colour on screens is limited to the marks, per decision 0117's output rule. Found on the way: the landing page hard-coded 'fourteen' settled arguments after 0117 made fifteen; the count now comes from sync's facts.json.

## Result

Shipped /tools/ and a landing section; every screen is computed at build time with a pinned date, and all gates pass.
