---
id: 151
uid: 9b1b25b0-2a18-4f77-85d7-5d8ca3b824c0
title: A repository a newcomer can find their way around
type: docs
status: done
milestone: v1.0
created: 2026-10-29
updated: 2026-10-29
closed_at: 2026-10-29
priority: p1
effort: m
area: docs
---

## Problem

The README, the contributing guide and the spec are good, but a newcomer to the code has no map: nothing says which crate does what, where data flows, or which invariants hold the design together. The crates have no READMEs, so crates.io and docs.rs will show nothing. Nothing checks that a relative link in a Markdown file still points at something, and two downloaded packages reached the root of the tree in #34 without anyone noticing.

## Approach

Follow the projects that do this well. rust-analyzer's ARCHITECTURE.md (a bird's-eye view, a code map, invariants called out where they live). ripgrep and uv give each crate a README of its own. Tree-sitter grammars carry a README with their usage. Then make each one a test, as the stability policy does, so the docs cannot drift.

## Acceptance criteria

- [x] ARCHITECTURE.md gives the bird's-eye view, a code map of every crate and top-level folder, the invariants and where each is enforced, and the cross-cutting concerns.
- [x] Every crate and every top-level folder a reader would open has a README, and each crate's Cargo.toml names its README, keywords and categories.
- [x] A test fails when a document names a path that does not exist, and `scripts/task lint` fails when an archive or a package is committed.
- [x] The README says where the project stands and how it compares, and SUPPORT.md says where to ask.
- [x] The two stray packages are gone from the tree.

## 2026-10-29

The path test found one real stale reference: refs.rs pointed at spec/refs.md, which never existed; refs are T1 of spec/tools.md. It also caught ARCHITECTURE.md using package-relative paths from the root on its first run. The archive guard reads sizes from the index with git cat-file, because a commit carries the index; it uses awk rather than grep, because grep exits 1 when nothing matches, and under set -e in dash that ended the substitution early, so a clean tree failed lint. Closed cairn items are left out of the path test, since they record what was true when they were written.

## Result

ARCHITECTURE.md, READMEs for every crate and folder, SUPPORT.md, a path test over every document, a lint guard against committed packages, and the two stray packages removed.
