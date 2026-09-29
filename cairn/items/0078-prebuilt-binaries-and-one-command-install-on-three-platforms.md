---
id: 78
uid: 73a2e62a-8e20-47f3-9a12-1bace480d26f
title: Prebuilt binaries and one-command install on three platforms
type: chore
status: planned
milestone: v1.0
depends_on:
- 71
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: release
---

## Purpose

The 1.0 promise says install is one command on macOS, Linux and Windows. `cargo install` needs a Rust toolchain, which most users won't have.

## Approach

Release workflow producing binaries for macOS (arm64, x64), Linux (x64, arm64, musl) and Windows (x64), a Homebrew tap, and shell and PowerShell installers. `cargo install` keeps working.

## Acceptance criteria

- [ ] A from-scratch CI job per platform installs with one command and runs `sprig --version`.
- [ ] Every binary ships with a checksum, and the installers verify it.
