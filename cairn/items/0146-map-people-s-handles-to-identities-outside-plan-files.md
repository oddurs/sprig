---
id: 146
uid: d779301e-b314-4e49-a076-9ba99564c6a0
title: Map people's handles to identities outside plan files
type: feature
status: backlog
milestone: later
created: 2026-10-28
updated: 2026-10-28
priority: p3
area: spec
---

## Problem

Calendar invites, org directories and notifications want an address for `@dana`, and a plan file is the wrong place to keep one (spike 0069): addresses change, and they put personal data in every repository a plan is copied to.

## Proposal

An optional mapping beside the plans, in the spirit of CODEOWNERS or git's `.mailmap`, that tools may read to turn a handle into an address. The format itself does not change.

## Acceptance criteria

- [ ] A design names the file, its format and which tools read it, with the case against.
