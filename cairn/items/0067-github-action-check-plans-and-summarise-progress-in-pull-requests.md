---
id: 67
uid: c0cd8e18-59fb-4f98-a431-0e4be528d8d8
title: 'GitHub Action: check plans and summarise progress in pull requests'
type: feature
status: doing
milestone: v0.3
depends_on:
- 25
- 27
created: 2026-09-28
updated: 2026-10-29
priority: p1
effort: s
area: interop
---

## Problem

A plan changed in a pull request should be checked, and the reviewer should see what the change did to progress.

## Proposal

A composite action that installs a released binary, runs `sprig check`, and writes a job summary: per-file progress on base and head, and items newly unblocked.

## Acceptance criteria

- [x] This repository's CI uses the action.
- [ ] The summary shows done counts for base and head and lists newly unblocked items, verified on a test pull request.

## 2026-10-29

The action now also writes its summary to the job log, so a reviewer, or a script reading the logs, can see it without the summary page. The summary's unblocked list is tested in crates/sprig-cli/tests/tools.rs on two copies of the bakery. It still needs one pull request that unblocks something, read by a person.

## 2026-10-29

Criterion 1: the plans job in ci.yml runs this repository's action, `uses: ./action`, on examples and tests/import, and branch protection requires it through the required job. Trying the summary on the real examples folder showed the unblocked item as examples/bakery/bakery/bakery.sprig:15: the script added the searched folder's parent back to a path find had already made relative to that folder. It is fixed, and tools.rs now covers a folder of folders, a trailing slash and a single file.
