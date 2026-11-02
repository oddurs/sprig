#!/bin/sh
# pre-push runs the suite with GIT_DIR and its siblings naming the repository
# being pushed. A test that builds a scratch repository and passes them on
# acts on the real one instead: it once set core.bare in the shared config
# and committed onto the branch being pushed (cairn 0157). So run every test
# binary that spawns git as a hook would, aimed at a decoy, and require the
# decoy untouched and the tests green.
set -eu

for var in $(git rev-parse --local-env-vars); do
  unset "$var"
done

here=$(cd "$(dirname "$0")/../.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

git init -q "$tmp/decoy"
git -C "$tmp/decoy" -c user.name=test -c user.email=test@example.org commit -q --allow-empty -m 'chore: seed'
before=$(cat "$tmp/decoy/.git/config"; git -C "$tmp/decoy" rev-parse HEAD)

binaries=$(grep -l 'Command::new("git")' "$here"/crates/*/tests/*.rs | sed 's|.*/||; s|\.rs$||')
args=''
for b in $binaries; do
  args="$args --test $b"
done

cd "$here"
# shellcheck disable=SC2086
if ! GIT_DIR="$tmp/decoy/.git" GIT_WORK_TREE="$tmp/decoy" cargo test -q -p sprig-cli $args >"$tmp/log" 2>&1; then
  cat "$tmp/log" >&2
  printf 'git-isolation: the tests that run git fail under a hook'"'"'s environment\n' >&2
  exit 1
fi

after=$(cat "$tmp/decoy/.git/config"; git -C "$tmp/decoy" rev-parse HEAD)
if [ "$before" != "$after" ]; then
  printf 'git-isolation: a test changed the repository GIT_DIR named\n' >&2
  exit 1
fi
printf 'ok: tests that run git leave the repository a hook names alone\n'
