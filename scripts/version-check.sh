#!/usr/bin/env bash

set -euo pipefail

readonly ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

readonly MANIFEST_VERSION="$(awk '
  /^\[package\]$/ { in_package = 1; next }
  in_package && /^\[/ { exit }
  in_package && /^version = / { gsub(/[\"[:space:]]/, "", $3); print $3; exit }
' Cargo.toml)"

readonly LOCK_VERSION="$(awk '
  /^\[\[package\]\]$/ { in_package = 1; name = ""; version = ""; next }
  in_package && /^name = "alfred_gitmoji"$/ { name = "alfred_gitmoji"; next }
  in_package && name == "alfred_gitmoji" && /^version = / {
    gsub(/[\"[:space:]]/, "", $3); print $3; exit
  }
' Cargo.lock)"

if [[ -z "$MANIFEST_VERSION" || -z "$LOCK_VERSION" ]]; then
  echo "Could not read the alfred_gitmoji version from Cargo.toml and Cargo.lock" >&2
  exit 1
fi

if [[ "$MANIFEST_VERSION" != "$LOCK_VERSION" ]]; then
  echo "Cargo.toml version ($MANIFEST_VERSION) does not match Cargo.lock ($LOCK_VERSION)" >&2
  exit 1
fi

TAG_NAME="${1:-${GITHUB_REF_NAME:-}}"
if [[ -n "$TAG_NAME" ]]; then
  TAG_VERSION="${TAG_NAME#v}"
  if [[ "$MANIFEST_VERSION" != "$TAG_VERSION" ]]; then
    echo "Cargo.toml version ($MANIFEST_VERSION) does not match tag ($TAG_VERSION)" >&2
    exit 1
  fi
fi

echo "Version $MANIFEST_VERSION is consistent"
