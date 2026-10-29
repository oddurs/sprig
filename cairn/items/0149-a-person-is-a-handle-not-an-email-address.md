---
id: 149
uid: 988cb0e5-fc1d-4e87-b575-2314de34d7a1
title: A person is a handle, not an email address
type: decision
status: done
created: 2026-10-29
updated: 2026-10-29
priority: p2
area: spec
---

## Context

Spike 0069 asked whether `@dana@example.org` should name a person. RFC 0002 (`rfcs/0002-a-person-is-a-handle.md`) records the argument.

## Options and tradeoffs

Accepting addresses would make a second `@` special inside one token, put personal data in every repository a plan is copied to, and tie a plan to addresses that change. Every tool would also have to treat `dana` and `dana@example.org` as maybe the same person. Handles have none of these costs, and mapping a handle to an address can live beside the plans, as git's `.mailmap` and GitHub's CODEOWNERS do.

## Decision

A person is a handle: `@` followed by letters, digits, `_`, `.` or `-`, and nothing else (§4.2). `@dana@example.org` and `dana@example.org` are both words.

## Revisit when

Never for the token. A mapping from handles to addresses, if one is wanted, is a separate file and a separate decision.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
