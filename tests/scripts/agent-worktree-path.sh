#!/bin/sh
# scripts/agent puts a worktree under ../.worktrees/<repository>/<branch>/,
# named after the repository and not after whatever directory the clone happens
# to live in. A clone checked out as `renamed-checkout` must still use `sprig`.
set -eu

# Run from a git hook (pre-push runs the suite), this inherits GIT_DIR and its
# siblings, and every git command below would act on the real repository
# instead of the scratch one. It did, once: a commit and a worktree landed in
# the real checkout. Clear them before touching git.
for var in $(git rev-parse --local-env-vars); do
  unset "$var"
done

here=$(cd "$(dirname "$0")/../.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

git init -q --bare -b main "$tmp/sprig.git"
git clone -q "$tmp/sprig.git" "$tmp/renamed-checkout" 2>/dev/null
cd "$tmp/renamed-checkout"
git -c user.name=test -c user.email=test@example.org commit -q --allow-empty -m 'chore: seed'
git push -q origin main
mkdir scripts
cp "$here/scripts/agent" scripts/agent

scripts/agent start chore/probe >/dev/null

if [ ! -d "$tmp/.worktrees/sprig/chore/probe" ]; then
  printf 'agent-worktree-path: expected %s, found:\n' "$tmp/.worktrees/sprig/chore/probe" >&2
  find "$tmp/.worktrees" -maxdepth 2 >&2
  exit 1
fi
printf 'ok: the worktree is named after the repository\n'
