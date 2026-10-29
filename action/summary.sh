#!/usr/bin/env bash
# Prints, as Markdown, what a change did to a folder's plans: the done count
# of each file on the base and on the head, and the items it unblocked.
#
#   summary.sh BASE_DIR HEAD_DIR [PATH...]
#
# BASE_DIR and HEAD_DIR are checkouts of the two commits; each PATH is read
# relative to both. SPRIG names the binary, `sprig` by default.
set -euo pipefail

base=$1
head=$2
shift 2
if [ $# -eq 0 ]; then set -- .; fi
sprig=${SPRIG:-sprig}

# The PATHs that exist in a checkout: a pull request may add or remove one.
present() {
  local dir=$1 p
  shift
  for p in "$@"; do
    if [ -e "$dir/$p" ]; then printf '%s\n' "$p"; fi
  done
}

# `status --plain`: file, done, total, ready, waiting, late, errors.
status() {
  local dir=$1
  shift
  local paths=() p
  # Not mapfile: macOS still ships bash 3.2.
  while IFS= read -r p; do paths+=("$p"); done < <(present "$dir" "$@")
  if [ ${#paths[@]} -gt 0 ]; then
    (cd "$dir" && "$sprig" status --plain "${paths[@]}")
  fi
}

# Items matching a query, one per line as `file<TAB>text<TAB>path<TAB>ref`,
# so that an item is the same item on both sides even when lines above it
# moved. `find` exits 1 when nothing matches, which is an answer here, not a
# failure.
items() {
  local dir=$1 query=$2
  shift 2
  local p code prefix
  present "$dir" "$@" | while IFS= read -r p; do
    code=0
    (cd "$dir" && "$sprig" find --plain --in "$p" "$query") >"$scratch/found" || code=$?
    if [ "$code" -gt 1 ]; then exit "$code"; fi
    # `find` names a file relative to the folder it searched, which for a
    # file is the folder the file is in.
    if [ -d "$dir/$p" ]; then prefix=${p%/}/; else prefix=$(dirname "$p")/; fi
    if [ "$prefix" = ./ ]; then prefix=; fi
    awk -F '\t' -v prefix="$prefix" '{
      ref = prefix $1
      file = ref; sub(/[:^].*/, "", file)
      print file "\t" $3 "\t" $4 "\t" ref
    }' "$scratch/found"
  done | sort -u
}

scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT

status "$base" "$@" | sort >"$scratch/base-status"
status "$head" "$@" | sort >"$scratch/head-status"

echo "### Plans"
echo
echo "| File | Done on base | Done on head | Ready | Waiting | Late |"
echo "| --- | ---: | ---: | ---: | ---: | ---: |"
join -t $'\t' -a 2 -e '' -o '0,1.2,1.3,2.2,2.3,2.4,2.5,2.6' \
  "$scratch/base-status" "$scratch/head-status" |
  awk -F '\t' '{
    was = ($2 == "") ? "new" : $2 " of " $3
    now = $4 " of " $5
    if ($2 != "" && $2 != $4) now = "**" now "**"
    printf "| `%s` | %s | %s | %s | %s | %s |\n", $1, was, now, $6, $7, $8
  }'

# Keyed on everything but the ref, which carries a line number.
items "$base" is:blocked "$@" | cut -f1-3 | sort -u >"$scratch/base-blocked"
items "$head" is:blocked "$@" | cut -f1-3 | sort -u >"$scratch/head-blocked"
items "$head" is:open "$@" >"$scratch/head-open"
awk -F '\t' 'FILENAME == ARGV[1] { was[$1 FS $2 FS $3] = 1; next }
             FILENAME == ARGV[2] { still[$1 FS $2 FS $3] = 1; next }
             ($1 FS $2 FS $3) in was && !(($1 FS $2 FS $3) in still) {
               printf "- `%s` %s\n", $4, $2
             }' "$scratch/base-blocked" "$scratch/head-blocked" "$scratch/head-open" >"$scratch/freed"

echo
if [ -s "$scratch/freed" ]; then
  echo "**Unblocked by this change**"
  echo
  cat "$scratch/freed"
else
  echo "Nothing was unblocked by this change."
fi
