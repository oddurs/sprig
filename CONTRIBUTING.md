# Contributing

Sprig is a small project with a strict workflow. The strictness is what lets
people and agents work in it at the same time without stepping on each other.

## Once, after cloning

```sh
scripts/setup
```

That wires `core.hooksPath` to `.githooks/`, checks your toolchain, and
registers cairn's merge driver. The hooks are the workflow; without them you
find out about a problem in CI instead of in your terminal.

## The loop

One unit of work is one cairn item, one branch, one worktree and one pull
request. Branches are `<type>/<id>-<slug>`, so the item is in the name.

```sh
cairn next                                  # what is ready to start
cairn claim 17                              # take it, so nobody duplicates it
scripts/agent start feat/0017-resolve-grafts
cd ../.worktrees/sprig/feat/0017-resolve-grafts

# ... work, ticking criteria as they become true: cairn tick 17 2 ...

scripts/agent check
scripts/agent commit "feat(core): resolve grafts across files"
cairn close 17 --result "what it concluded"   # in the same pull request
scripts/agent pr
scripts/agent sync                          # rebase onto main when it moves
# ... after the pull request is merged, from the primary checkout ...
scripts/agent done feat/0017-resolve-grafts
```

`scripts/agent list` shows every worktree, its branch and its pull request.
Worktrees live in `../.worktrees/sprig/<branch>/`, outside the repository, so
two branches never share an index or a `target/` directory.

## `main` only advances through a merged pull request

The local `pre-push` hook refuses a push to `main`, and branch protection on the
server refuses it again. That holds for the maintainer too.

Required approvals are **0**, deliberately: this is a solo project, and a review
requirement would deadlock the only person who can review. Everything else is
required: a pull request, a green `required` check, an up-to-date branch and
resolved conversations. With a second maintainer, that number becomes 1.

## Checks

Everything goes through one seam, so CI and your machine cannot disagree:

```sh
scripts/task fmt        # format in place
scripts/task fmt:check  # verify formatting
scripts/task lint       # clippy with warnings denied, and the core purity guard
scripts/task test       # the full suite
scripts/task build      # compile everything
scripts/task backlog    # cairn check --strict, and ROADMAP.md is current
scripts/task check      # all of the above; what CI runs
```

`pre-commit` formats and lints; `pre-push` runs the lot. Never use
`--no-verify`. If a hook is wrong, fix the hook in its own pull request.

## Commits

[Conventional Commits](https://www.conventionalcommits.org): imperative mood,
subject under 72 characters, no trailing period. The body explains why; the
diff already says what. Reference the item in a `Refs:` trailer.

```
feat(core): resolve grafts across files

A graft that leaves the workspace root is refused rather than followed,
because plans arrive from repositories nobody has read.

Refs: 0017
```

Types: `feat` `fix` `chore` `docs` `perf` `refactor` `test` `build` `ci`
`style` `revert`.

No commit, pull request, comment or document attributes work to an assistant, a
model or a tool. The `commit-msg` hook enforces it.

## The backlog and the format

The roadmap is cairn items in `cairn/items/`. `ROADMAP.md` is generated; never
edit it by hand. Write the item before work larger than a fix, with the
reasoning: the problem, the proposal and verifiable acceptance criteria.

The format is opinionated on purpose. Before proposing a change to it, read the
settled arguments (`cairn list --view decisions`). A proposal states the case
against itself as well as the case for.
