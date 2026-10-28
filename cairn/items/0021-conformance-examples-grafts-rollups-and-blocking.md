---
id: 21
uid: 5ed6e6f4-90fb-4454-a524-69bfdfe35fe6
title: 'Conformance examples: grafts, rollups and blocking'
type: feature
status: done
milestone: v0.2
depends_on:
- 10
- 19
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: m
area: suite
---

## Problem

Cross-file behaviour is Sprig's headline feature and the hardest to specify in prose.

## Proposal

At least 40 multi-file examples: whole-file and branch grafts, missing targets, loops, settling, answered questions, recurring items, people inheritance, estimate sums, and local, cross-file and inherited blocking.

## Acceptance criteria

- [x] At least 40 examples exist under `tests/conformance/graph/` and pass against the core.
- [x] Every rule in the spec's status, people, blocking and graft sections is cited at least once.

## 2026-10-28

59 examples in `tests/conformance/graph/`, most of them multi-file, each checked by hand against §5 to §8. Where the reference parser had a view, the core agrees with it or the difference is a ruling in Appendix B (B.5, B.6, B.7, B.9, B.10).

Two cases are worth knowing. When two files graft each other, path order decides which graft reports the loop (B.10), so `a` grafting `b` that grafts `a` mounts nothing in `a`'s own tree. An `after:` pointing at a dropped or later item blocks forever, since neither is done (§5.7, §7.3).

## Result

59 examples covering whole-file and branch grafts, missing targets, loops, settling, answered questions, recurring items, people inheritance, estimate sums, and local, cross-file and inherited blocking. A test fails if a §5 to §8 rule goes uncited.

## 2026-10-28

Correction: the file later in path order is the one whose graft closes the loop. In graph/045, main.sprig grafts b.sprig and b.sprig grafts main.sprig. b.sprig comes first in path order, so main.sprig's graft is refused, and main.sprig's own tree mounts nothing from b.sprig. Spec §8.4 now states the rule, with decision 0144.
