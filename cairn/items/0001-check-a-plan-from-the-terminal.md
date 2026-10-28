---
id: 1
uid: 7b923b12-39a5-4c16-8e08-bc016549c46b
key: v0.1
title: Check a plan from the terminal
type: milestone
status: planned
created: 2026-09-28
updated: 2026-10-28
priority: p2
due: 2026-11-20
---

## Ships

A stranger installs `sprig`, writes a plan across one or more `.sprig` files, and gets two answers: `sprig next` says what can be done right now, and `sprig check` reports broken links, loops and unknown anchors with exit codes CI can use.

## Done when

- [ ] `cargo install` of the published crate works on macOS, Linux and Windows.
- [x] `sprig next` and `sprig check` give correct answers for the six files in `examples/bakery/`.
- [x] The spec draft and the conformance harness are in the repository, and CI runs the suite on three platforms.
- [ ] The README takes a stranger from nothing to `sprig next` output in five minutes.

## Explicitly not in this milestone

- Editing files through the tool: fmt, tick, or any write (v0.2).
- Editor support of any kind (v0.2).
- Typed date shortcuts (v0.2), JSON output and every import or export format (v0.3).

## 2026-10-28

Two of four done. `sprig next` and `sprig check` give the reference parser's answers for the six bakery files, and are snapshotted (0023, 0025). The spec draft, the conformance harness and 55 examples are in the repository, and the `check` job runs the suite on ubuntu, macOS and Windows. Open: publishing (0027, blocked on the name, 0006), and the five-minute stranger test of the README (0026).
