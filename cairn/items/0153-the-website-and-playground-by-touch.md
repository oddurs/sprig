---
id: 153
uid: c2e28ee8-05ca-490b-b388-9605b74a1aa5
title: The website and playground, by touch
type: feature
status: done
milestone: launch
created: 2026-10-29
updated: 2026-10-29
closed_at: 2026-10-29
priority: p1
effort: m
area: site
---

## Problem

On a phone the site worked but fought the reader: controls a third of a fingertip (22px header links, 16px marks), boxes that scrolled inside a page that scrolled, terminal lines cut off with no sign they continued, and a playground whose editor, plan and view were three nested scroll areas with no Tab key to indent. The playground also never said what was wrong with a plan: the core computed every diagnostic and the page threw them away.

## Approach

Follow the playgrounds that do this well (the TypeScript and Svelte playgrounds, Ruff's and Biome's): a problems panel that names each code and jumps to its line, one panel at a time on a narrow screen, and touch targets of 44px (Apple's, and WCAG 2.5.5). Then hold every page to it in the build, the way axe already holds contrast.

## Acceptance criteria

- [x] At 360px no page scrolls sideways, and every control is at least 44px; at 768px, WCAG 2.5.8's 24px. The site build fails otherwise.
- [x] The playground lists every diagnostic with its code, linked to its note, and choosing one selects its line.
- [x] Narrow, the playground shows one panel at a time with no nested scrolling, and indenting, outdenting and cycling a mark work without a keyboard.
- [x] A script drives the playground wide and narrow in the build, and fails when a behaviour breaks.

## 2026-10-29

The mobile check found 1,137 problems on its first run, almost all controls under 44px: the header links (22px), the plan's marks (16px) and carets (14px), Starlight's heading anchors, menu and copy buttons, and links standing on their own line. The marks are now a picture inside a 44px button, laid out as a column of their own by touch rather than an invisible margin, so two targets never overlap. Two things in the check were corrected on the way: a skip link hidden with clip is not a visible control, and at 768px the target is WCAG 2.5.8's 24px with its spacing exception, since 2.5.5's 44px is the phone's. The playground check found one real bug before it passed: hiding the key bar on blur moved the page under a tap, so the click landed on whatever was there instead, which made the file menu fail to open after editing. Both checks run in scripts/task site, in headless Chrome with no driver, as a11y.mjs does.

## Result

Every page passes at 360px and 768px, and the playground shows its problems, works one panel at a time on a phone, and is driven through 25 behaviours in the build.
