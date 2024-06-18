#!/bin/bash

if [ $# -ne 1 ]; then
  echo "Usage: $0 {major|minor|patch}"
  exit 1
fi

bump_type=$1

if [ "$bump_type" != "major" ] && [ "$bump_type" != "minor" ] && [ "$bump_type" != "patch" ]; then
  echo "Invalid bump type. Use 'major', 'minor', or 'patch'."
  exit 1
fi

package_version=$(grep -m 1 '"version":' package.json | sed -E 's/.*"([^"]+)".*/\1/')
cargo_version=$(grep -m 1 '^version =' src-tauri/Cargo.toml | sed -E 's/.*"(.*)"/\1/')

if [ "$package_version" != "$cargo_version" ]; then
  echo "Error: Version mismatch between package.json and Cargo.toml"
  echo "package.json version: $package_version"
  echo "Cargo.toml version: $cargo_version"
  exit 1
fi

npm version $bump_type --no-git-tag-version
cd src-tauri || exit
cargo set-version --bump "$bump_type"

echo "Version bumped successfully! package.json and src-tauri/Cargo.toml updated"