#!/bin/sh
# pre-push runs the suite with GIT_DIR and its siblings naming the repository
# being pushed. A test that builds a scratch repository and passes them on
# acts on the real one instead: it once set core.bare in the shared config
# and committed onto the branch being pushed (cairn 0157). So run every test
# binary that touches git as a hook would, aimed at a decoy, and require the
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
# What the damage touched: the config (core.bare), the branch, the index.
state() {
  cat "$tmp/decoy/.git/config"
  git -C "$tmp/decoy" rev-parse HEAD
  git -C "$tmp/decoy" ls-files --stage
}
before=$(state)

# Any test that mentions git, not just those that spawn it directly: the one
# that did the damage the second time ran it through `sh -c`.
cd "$here"
for file in $(grep -lw git crates/*/tests/*.rs); do
  crate=${file#crates/}
  crate=${crate%%/*}
  test=${file##*/}
  test=${test%.rs}
  # GIT_DIR alone, as a push hook has it: with no work tree named, a passed-on
  # `git init` marks the repository bare.
  if ! GIT_DIR="$tmp/decoy/.git" cargo test -q -p "$crate" --test "$test" >"$tmp/log" 2>&1; then
    cat "$tmp/log" >&2
    printf 'git-isolation: %s fails under a hook'"'"'s environment\n' "$file" >&2
    exit 1
  fi
done

if [ "$before" != "$(state)" ]; then
  printf 'git-isolation: a test changed the repository GIT_DIR named\n' >&2
  exit 1
fi
printf 'ok: tests that run git leave the repository a hook names alone\n'
