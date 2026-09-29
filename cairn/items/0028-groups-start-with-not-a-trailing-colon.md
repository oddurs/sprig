---
id: 28
uid: ed7493ac-7123-4aa4-af1f-43b852694e9b
title: Groups start with '# ', not a trailing colon
type: decision
status: done
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: spec
---

## Context

Draft 0.1 used TaskPaper's `Name:` for groups.

## Options and tradeoffs

The colon reads naturally, but any note ending in a colon, such as 'Two options:', silently became a heading and captured the lines under it. A mark matches every other line start.

## Decision

Groups start with `# `: one character, then a space. Markdown viewers show it as a heading too.

## Revisit when

Never. A rule that fires by accident stays a bad rule.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
