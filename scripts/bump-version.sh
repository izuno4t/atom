#!/usr/bin/env bash
# Bump the version of every workspace package and refresh Cargo.lock.
#
# Usage: scripts/bump-version.sh <new-version>
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <new-version>" >&2
  exit 1
fi

new_version="$1"

# Accept SemVer X.Y.Z with an optional pre-release / build suffix.
if [[ ! "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$ ]]; then
  echo "error: '$new_version' is not a valid version (expected X.Y.Z)" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

manifests=(
  "$repo_root/Cargo.toml"
  "$repo_root/evaluation/Cargo.toml"
)

for manifest in "${manifests[@]}"; do
  # Replace only the `version` line that belongs to the [package] table,
  # leaving dependency version requirements untouched.
  awk -v ver="$new_version" '
    /^\[package\]/ { in_pkg = 1; print; next }
    /^\[/          { in_pkg = 0 }
    in_pkg && /^version[[:space:]]*=/ {
      print "version = \"" ver "\""
      next
    }
    { print }
  ' "$manifest" > "$manifest.tmp"
  mv "$manifest.tmp" "$manifest"
  echo "updated ${manifest#"$repo_root"/} -> $new_version"
done

# Sync the workspace package entries in Cargo.lock without touching deps.
( cd "$repo_root" && cargo build --workspace --quiet )

echo "done: version bumped to $new_version"
