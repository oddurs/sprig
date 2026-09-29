# Working in this repository

Instructions for an agent. Follow them exactly; they override any default
workflow you would otherwise apply.

## Attribution

Never attribute work in this repository to an assistant, a model or a tool: not
in commit messages, trailers, pull request bodies, issue comments, code
comments, documentation, changelogs or release notes. No `Co-Authored-By:`
naming a model, no "generated with" footer, no robot emoji.

The `commit-msg` hook rejects a message that breaks this, and `scripts/agent pr`
strips it from a pull request body. Do not rely on either; write it correctly
the first time.

## Setup

```sh
scripts/setup          # once per clone: hooks, toolchain, cairn merge driver
scripts/agent doctor   # verify before starting anything
```

## The loop

One cairn item is one branch, in one worktree, with one pull request. Two agents
never share a checkout.

```sh
cairn next                                     # pick ready work
cairn claim 17
scripts/agent start feat/0017-resolve-grafts   # prints the worktree path
cd ../.worktrees/sprig/feat/0017-resolve-grafts   # move there yourself

# ... work; tick criteria as they become true, with evidence ...
cairn tick 17 2
cairn note 17 "what you learned"

scripts/agent check
scripts/agent commit "feat(core): resolve grafts across files"
cairn close 17 --result "what it concluded"    # in the same pull request
scripts/agent pr
```

After the merge, from the primary checkout: `scripts/agent done <branch>`.

`cairn prompt 17` prints an item with everything it depends on. Read it before
starting.

Rules that are not negotiable:

- **Never commit to `main`.** The `pre-push` hook and branch protection both
  refuse. Do not look for a way around either.
- **Never use `--no-verify`**, `continue-on-error` or `|| true` to make a check
  pass. If a check is wrong, fix it in its own pull request.
- **Never edit `ROADMAP.md`.** cairn generates it from `cairn/items/`.
- **Never close an item with unticked criteria** unless a note says why each
  one no longer applies.

## The seam

All automation goes through `scripts/task`. Do not put a `cargo` invocation in
CI, a hook or a script; put it there, once.

```sh
scripts/task fmt | fmt:check | lint | test | build | backlog | check
```

## Commits

Conventional Commits, imperative, subject under 72 characters, no trailing
period. The body says why. `Refs: 0017` names the item.

## Architecture, and what must stay true

- **The spec is the authority, and the conformance suite is its evidence.** A
  change to how a file is read starts in `spec/sprig.md`, gets an example in
  `tests/conformance/`, and only then reaches the code. Until the spec exists,
  the behavioural oracle is the JavaScript parser inside
  `design/sprig-draft-0.2.html`.
- **`sprig-core` performs no I/O.** No filesystem, environment, network or
  clock. Callers pass source text and today's date. `scripts/task lint` greps
  for violations.
- **User input never panics.** Every problem in a `.sprig` file becomes a
  `Diagnostic` with a stable code from `spec/diagnostics.md`. Codes are never
  reused.
- **Edits preserve every byte they don't mean to change.** Anything that writes
  a file (fmt, tick, the language server, MCP) goes through the core's edit API.
- **Grafts never leave the workspace root.** A graft path is untrusted input.
- **The file is the truth.** The computed layer (progress, blockers, settled
  status) never contradicts what the text says. It can only add hints.

## The format is settled until argued otherwise

The design decisions are cairn items of type `decision`
(`cairn list --view decisions`). Do not relitigate one in code. If you believe a
ruling is wrong, open a new item with the case for, the case against and a
proposed ruling.

## Comments

Explain **why**, not what. Match the density and voice of the surrounding code.
Name a test after the behaviour it protects, not after the function it calls.
