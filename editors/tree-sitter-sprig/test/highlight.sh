#!/bin/sh
# Highlights every plan in examples/bakery/ and compares the result with the
# committed snapshot. With --update, rewrites the snapshot instead.
#   TREE_SITTER: the tree-sitter CLI to run (default: tree-sitter on PATH)
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
ts=${TREE_SITTER:-tree-sitter}
export TREE_SITTER_DIR="$here/test/config"
snap="$here/test/snapshot/bakery.txt"
out=$(mktemp)
trap 'rm -f "$out"' EXIT
cd "$here"
for f in ../../examples/bakery/*.sprig; do
  printf '=== %s\n' "$(basename "$f")" >>"$out"
  "$ts" highlight "$f" >>"$out"
done
if [ "${1:-}" = "--update" ]; then
  cp "$out" "$snap"
  echo "highlight: wrote $snap"
elif ! diff -u "$snap" "$out"; then
  echo "highlight: output differs from test/snapshot/bakery.txt; run test/highlight.sh --update and read the diff" >&2
  exit 1
else
  echo "highlight: examples/bakery matches the snapshot"
fi
