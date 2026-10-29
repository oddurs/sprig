---
id: 73
uid: 195fa738-5f80-473c-9d1d-c5c2aa2e655a
title: Stability policy
type: docs
status: done
milestone: v1.0
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: s
area: community
---

## Reader and question

Anyone deciding whether to build on Sprig, and needing to know what won't change under them.

## Change

`docs/stability.md`: a file valid under 1.0 parses the same way forever; spec changes after 1.0 are additive; removing anything needs spec 2.0; diagnostic codes are never reused; crates and the CLI follow semver, and for the CLI that covers exit codes and the column order of every command's `--plain` output.

## Acceptance criteria

- [x] Linked from the README, the spec and the crate documentation.
- [x] Every promise can be tested, and says what test holds it.
- [x] The `--plain` columns of every command are listed, each with the snapshot test that holds it.

## 2026-10-28

From spike 0070: the policy should promise that no 1.x release changes what a valid 1.x file means. That promise is why files carry no version, and the title's `sprig:` field is reserved for a 2.0 (§4.6, S0007).

## Result

docs/stability.md: files keep their meaning across 1.x, the spec only adds, codes are never reused, and the tree format, exit codes, --plain columns and MCP tools are stable. Each promise names its test, and crates/sprig-cli/tests/docs.rs fails if a named test or example disappears. It is linked from the README, the spec and the crate docs.
