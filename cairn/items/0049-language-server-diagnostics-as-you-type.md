---
id: 49
uid: 9e6ccf77-b820-4a98-bbb0-5fdb7ca2261c
title: 'Language server: diagnostics as you type'
type: feature
status: done
milestone: v0.2
depends_on:
- 25
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: m
area: lsp
---

## Problem

`sprig check` only helps when someone runs it. An editor should show the same problems while you type, including ones caused by another file.

## Proposal

`sprig lsp` (or `sprig-lsp`) over stdio, built on `lsp-server` and `lsp-types`, the small synchronous crates rust-analyzer uses, rather than an async framework. It watches the workspace folder and republishes diagnostics for every file affected by a change.

## Acceptance criteria

- [x] Diagnostics appear on open and on change, driven by a scripted JSON-RPC test client.
- [x] Deleting `^signed` in `lease.sprig` produces a diagnostic in `bakery.sprig`, which links to it, without `bakery.sprig` being edited.
- [x] Rechecking a 5,000-line workspace after a keystroke takes under 50 ms on the CI runner.

## 2026-10-28

`sprig lsp` lives in `crates/sprig-lsp`, built on `lsp-server` 0.7 and `lsp-types` 0.97. It runs as a plain loop on one thread with full-text sync. Each change marks the workspace stale, and it is resolved once the message queue is empty, so a burst of keystrokes costs one resolution. Only files whose diagnostics changed are republished. The server reads the folder the editor opened, takes today from the CLI (`--today` still fixes it) and negotiates UTF-8 positions when the client offers them.

1. `crates/sprig-lsp/tests/lsp.rs` drives the server over an in-memory connection. `diagnostics_appear_on_open_and_follow_each_change` covers open, change and the clear. `crates/sprig-cli/tests/cli.rs` does the same over real stdio with the built binary.
2. `deleting_an_anchor_reports_the_files_that_name_it_without_editing_them`: removing `^signed` from `lease.sprig` publishes S0203 for `bakery.sprig:15` and `kitchen.sprig:18`, and restoring it clears both.
3. Not ticked: it is measured but not yet held on CI. The test uses 50 files of 100 lines each, one grafting the other 49, with every item naming an anchor in another file.
   - Round trip from didChange to publish here: release about 30 ms typical and 41 ms worst; debug 125 ms.
   - The test asserts under 50 ms in release and under 500 ms in debug, but CI runs debug.
   - On the way I cut the core's allocations. Tokens are now shared between trees (`Resolved::tokens` is an `Arc`), finished trees are moved rather than copied, and the §8.8 keys and reference lookup borrow instead of allocating. With `examples/bench_resolve.rs`, the hub went from 34 to 21 ms and a 50-deep graft chain from 397 to 161 ms. The chain is quadratic by design, since every file's tree holds everything beneath it.
   - Holding the 50 ms on the CI runner needs either a release-mode test job or more headroom; that is 0056's work.

## 2026-10-29

CI perf job on PR #30: a keystroke recheck of 5,000 lines took a median of 17.9 ms, worst 18.6 ms, on ubuntu-latest. The job fails above 50 ms.

## Result

The language server keeps a parsed workspace and reparses only the file that changed; CI holds a keystroke recheck of 5,000 lines under 50 ms (18 ms measured).
