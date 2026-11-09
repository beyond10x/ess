#!/usr/bin/env bash
# hotfix-scope.sh <tag> [<previous-tag>]
#
# Decides how much of the gate a release tag runs. A minor or major release (`X.Y.0`) runs the
# full gate. A hotfix (`X.Y.Z` with Z > 0) runs the targeted gate instead: the checks, and the tests
# of the workspace crates it changed plus their direct dependents. Anything this cannot map to a
# crate falls back to the full gate, so an unexpected change is never tested less.
#
# Prints `scope=full|targeted`, `previous=<tag>`, `packages=<space-separated names>` and
# `reason=<text>`, one per line, and appends them to "$GITHUB_OUTPUT" when that is set. Reads only
# Git and `cargo metadata --no-deps`, so it needs no network and no build.
set -euo pipefail

tag=$1
previous=${2:-}

emit() {
  local scope=$1 packages=$2 reason=$3
  printf 'scope=%s\nprevious=%s\npackages=%s\nreason=%s\n' "$scope" "$previous" "$packages" "$reason"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    printf 'scope=%s\nprevious=%s\npackages=%s\nreason=%s\n' "$scope" "$previous" "$packages" "$reason" \
      >> "$GITHUB_OUTPUT"
  fi
  exit 0
}

if [[ ! "$tag" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  emit full "" "$tag is not a bare version"
fi
major=${BASH_REMATCH[1]} minor=${BASH_REMATCH[2]} patch=${BASH_REMATCH[3]}
if [ "$patch" = 0 ]; then
  emit full "" "$tag is a minor or major release"
fi

# The previous release of the same line: the highest bare tag below this one.
if [ -z "$previous" ]; then
  previous=$(git tag --list '[0-9]*.[0-9]*.[0-9]*' \
    | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' \
    | sort -V \
    | awk -v tag="$tag" '$0 == tag { exit } { last = $0 } END { print last }')
fi
if [ -z "$previous" ] || ! git rev-parse --quiet --verify "refs/tags/$previous^{commit}" > /dev/null; then
  emit full "" "no previous release tag below $tag"
fi
if [[ "$previous" != "$major.$minor."* ]]; then
  emit full "" "previous tag $previous is not of the $major.$minor line"
fi

old_version=$previous
new_version=$tag
metadata=$(cargo metadata --no-deps --format-version 1 --offline 2> /dev/null) \
  || emit full "" "cargo metadata failed"
root=$(printf '%s' "$metadata" | jq -r '.workspace_root')

# Each workspace package's directory, relative to the root, longest first so a nested crate wins.
dirs=$(printf '%s' "$metadata" | jq -r --arg root "$root/" '
  .packages[] | "\(.manifest_path | ltrimstr($root) | rtrimstr("/Cargo.toml"))\t\(.name)"' \
  | awk -F '\t' '{ print length($1) "\t" $0 }' | sort -rn | cut -f2-)

changed=""
while IFS= read -r file; do
  [ -n "$file" ] || continue
  case "$file" in
    # Prose and records: the checks job holds the documents to the code.
    .engineering/* | changes/* | website/* | docs/* | CHANGELOG.md | README.md | WHATS-CHANGED.md)
      continue ;;
    # The manifests and lockfiles change on every release by the version bump alone; anything
    # else in them (a dependency, a feature, a profile) is a change to every crate.
    Cargo.toml | Cargo.lock | fuzz/Cargo.lock)
      if ! diff -q \
        <(git show "$previous:$file" | sed "s/\"$old_version\"/\"$new_version\"/g; s/ $old_version\$/ $new_version/") \
        <(git show "$tag:$file") > /dev/null; then
        emit full "" "$file changed beyond the version bump"
      fi
      continue ;;
  esac
  package=""
  while IFS=$'\t' read -r dir name; do
    case "$file" in
      "$dir"/*)
        if [ "$file" = "$dir/Cargo.toml" ] && ! diff -q \
          <(git show "$previous:$file" | sed "s/\"$old_version\"/\"$new_version\"/g") \
          <(git show "$tag:$file") > /dev/null; then
          emit full "" "$file changed beyond the version bump"
        fi
        package=$name
        break ;;
    esac
  done <<< "$dirs"
  [ -n "$package" ] || emit full "" "$file belongs to no workspace crate"
  changed="$changed $package"
done < <(git diff --name-only "$previous" "$tag")

changed=$(printf '%s\n' $changed | sort -u | tr '\n' ' ' | sed 's/ $//')
[ -n "$changed" ] || emit full "" "no crate changed between $previous and $tag"

# Direct dependents, through any dependency kind: their tests exercise the changed crate too.
dependents=$(printf '%s' "$metadata" | jq -r --arg changed "$changed" '
  ($changed | split(" ")) as $set
  | .packages[]
  | select(any(.dependencies[]; (.path != null) and (.name as $n | $set | index($n))))
  | .name')
packages=$(printf '%s\n' $changed $dependents | sort -u | tr '\n' ' ' | sed 's/ $//')
emit targeted "$packages" "changed: $changed"
