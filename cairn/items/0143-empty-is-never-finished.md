---
id: 143
uid: 403e432b-1f98-4251-a9fd-04757ce68e23
title: Empty is never finished
type: decision
status: done
created: 2026-10-28
updated: 2026-10-28
priority: p2
area: spec
---

## Context

An empty group, graft or file has derived status empty (§5.5). Appendix B.5 asked whether an `after:` naming one should wait forever.

## Options and tradeoffs

Counting empty as finished, as vacuous truth would, unblocks `after:[[launch]]` the moment `launch.sprig` is created as a stub, before anything in it is done. That is a false green. Keeping empty unfinished is honest: nothing has been finished. Its risk is a silent wait on a placeholder, which a warning removes.

## Decision

Empty is never finished (§5.5, §5.7). A reference to an empty node is reported as warning S0206, so the wait is visible.

## Revisit when

Plans commonly reference placeholders on purpose and find the warning noise.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
