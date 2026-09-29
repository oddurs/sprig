---
id: 110
uid: cbd08f0a-5752-4094-b2fa-7fa52738d3c9
title: Prebuilt binaries for the v0.1 release
type: chore
status: planned
milestone: launch
depends_on:
- 22
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: release
---

## Purpose

Most readers won't install a Rust toolchain to try a planning format.

## Approach

Extend the release workflow to build and attach binaries for macOS (arm64 and x64), Linux (x64 and arm64, musl) and Windows (x64), with checksums and build provenance. harrow's release workflow already does this and is the model.

## Acceptance criteria

- [ ] A tagged release attaches a binary for each target, with SHA256SUMS.
- [ ] Each binary that can run on its build runner prints `sprig --version` in CI.
