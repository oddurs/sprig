---
id: 157
uid: 0e216562-a010-42fe-846f-5bd9074e04f1
title: Tests that run git leave the real repository alone under a hook
type: bug
status: done
milestone: v1.0
assignee: Oddur Sigurdsson
created: 2026-11-02
updated: 2026-11-02
closed_at: 2026-11-02
priority: p0
effort: s
area: release
---

## Problem

The pre-push hook runs the full suite with GIT_DIR and its siblings set to the repository being pushed. crates/sprig-cli/tests/tui.rs builds a scratch repository with git init, add -A and commit, and passes those variables straight through, so under the hook it acted on the real repository instead: it set core.bare=true in the shared config, so `git status` refused to run in the primary checkout, and committed a stray "base" commit onto the branch being pushed. docs.rs reads `git ls-files` the same way. The scripts under tests/scripts already clear these variables, for the same reason, after it happened once before.

## Approach

Every test that spawns git removes the GIT_ variables it inherits before setting its own. A script test runs the git-using test binaries with GIT_DIR and GIT_WORK_TREE pointed at a decoy repository and fails if the decoy changes or the tests fail.

## Acceptance criteria

- [x] tui.rs and docs.rs spawn git with no inherited GIT_ variables.
- [x] tests/scripts/git-isolation.sh fails without the fix and passes with it.
- [x] A push from a worktree, which runs the suite under pre-push, succeeds.

## 2026-11-02

Found when the pre-push hook of 0156 ran the suite: tui.rs's scratch-repository setup set core.bare=true in the primary checkout's config and committed 'base' onto the branch being pushed. Repaired by hand (core.bare false, branch reset --mixed to the real commit; nothing reached the remote). Both tests now strip every inherited GIT_ variable rather than a list, since git adds to that list. git-isolation.sh finds the test binaries by grepping for Command::new("git"), so a new test that spawns git is covered without editing it; it failed before the fix with 2 tui tests red and passed after.

## 2026-11-02

The first push of the fix still went wrong: guide.rs runs 'git init -q && git add -A' through sh, which a grep for Command::new("git") missed. It re-initialised the primary repository as bare and staged the bakery into this worktree's index; both repaired by hand again. guide.rs now strips GIT_ variables from that call and from the recipes it runs. git-isolation.sh now covers every test file that mentions git, sets GIT_DIR alone as a push hook does (with GIT_WORK_TREE set too, init does not mark the repository bare, and the leak went unseen), and compares the decoy's index as well as its config and HEAD. It fails with guide.rs unfixed.

## Result

Every test that runs git, directly or through a shell, drops the GIT_ variables a hook exports, and tests/scripts/git-isolation.sh runs each such test with GIT_DIR aimed at a decoy and fails if its config, HEAD or index change. A push from a worktree now runs the suite under pre-push without touching the real repository.
