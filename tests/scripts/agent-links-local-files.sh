#!/bin/sh
# What a contributor lists in .git/info/exclude stays out of git but must reach
# every worktree scripts/agent creates, or the work there runs without it.
# Paths that are listed but missing, and comments, are skipped. The directory is
# listed without a slash, as it must be: `/.private/` would not match the link.
set -eu

# Run from a git hook, this inherits GIT_DIR and its siblings; see
# agent-worktree-path.sh for what that once did to the real checkout.
for var in $(git rev-parse --local-env-vars); do
  unset "$var"
done

here=$(cd "$(dirname "$0")/../.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

git init -q --bare -b main "$tmp/sprig.git"
git clone -q "$tmp/sprig.git" "$tmp/sprig" 2>/dev/null
cd "$tmp/sprig"
git -c user.name=test -c user.email=test@example.org commit -q --allow-empty -m 'chore: seed'
git push -q origin main
mkdir scripts
cp "$here/scripts/agent" scripts/agent

printf 'notes for me\n' >NOTES.local.md
mkdir -p .private/rules
printf '# a comment\n\n/NOTES.local.md\n/.private\n/never-created\n' >>.git/info/exclude

scripts/agent start chore/probe >/dev/null
wt=$tmp/.worktrees/sprig/chore/probe

fail() {
  printf 'agent-links-local-files: %s\n' "$1" >&2
  exit 1
}
[ -L "$wt/NOTES.local.md" ] || fail 'NOTES.local.md was not linked'
[ "$(cat "$wt/NOTES.local.md")" = 'notes for me' ] || fail 'NOTES.local.md links to the wrong file'
[ -L "$wt/.private" ] && [ -d "$wt/.private/rules" ] || fail '.private/ was not linked as a directory'
[ ! -e "$wt/never-created" ] || fail 'a listed path that does not exist was created'
[ -z "$(git -C "$wt" status --porcelain)" ] || fail 'the links show up as changes in the worktree'
printf 'ok: excluded local files follow the work into a new worktree\n'
