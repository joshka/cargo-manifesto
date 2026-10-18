#!/bin/sh
# Run with an installed cargo-manifesto or a path to the built binary.
set -eu
tool="${1:-cargo-manifesto}"
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
mkdir -p "$scratch/src"
printf '\n' > "$scratch/src/lib.rs"
cat > "$scratch/Cargo.toml" <<'MANIFEST'
[package]
name = "enduring-convictions"
version = "0.1.0"
edition = "2021"

[workspace]
MANIFEST
"$tool" --manifest-path "$scratch/Cargo.toml" --format json > "$scratch/before.json"
cat >> "$scratch/Cargo.toml" <<'DEPENDENCY'

[dependencies]
serde = "1"
DEPENDENCY
"$tool" --manifest-path "$scratch/Cargo.toml" --amend "$scratch/before.json"
