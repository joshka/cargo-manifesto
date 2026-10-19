# Repository tasks. CI runs the same recipes, so local and CI checks agree.

set shell := ["bash", "-euo", "pipefail", "-c"]

bin := "target/debug/cargo-manifesto"
examples := "examples/projects"

# List recipes
default:
    @just --list

# Format Rust (nightly rustfmt) and Markdown
fmt:
    cargo +nightly fmt
    rumdl check --fix .

# Check formatting without changing files
fmt-check:
    cargo +nightly fmt --check
    rumdl check .

# Run clippy with warnings denied
clippy:
    cargo clippy --locked --all-targets -- -D warnings

# Run all tests
test:
    cargo test --locked

# Check licenses, advisories, and sources of dependencies
deny:
    cargo deny --config .config/deny.toml check

# Regenerate MANIFESTO.md and the example outputs in `out`
generate out="docs/generated":
    cargo build --locked
    mkdir -p {{ out }}
    {{ bin }} --manifest-path {{ examples }}/tiny/Cargo.toml > {{ out }}/tiny.md
    {{ bin }} --manifest-path {{ examples }}/tiny/Cargo.toml --format json > {{ out }}/tiny.json
    {{ bin }} --manifest-path {{ examples }}/rich/Cargo.toml > {{ out }}/rich.md
    {{ bin }} --manifest-path {{ examples }}/rich/Cargo.toml --explain > {{ out }}/annotated.md
    {{ bin }} --manifest-path {{ examples }}/workspace/Cargo.toml --workspace > {{ out }}/workspace.md
    {{ bin }} > {{ out }}/self.md
    examples/amend.sh {{ bin }} > {{ out }}/amendment.md
    {{ bin }} --fix

# Fail if MANIFESTO.md or docs/generated would change when regenerated
generated-check:
    cargo build --locked
    {{ bin }} --check
    tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT; \
        just generate "$tmp" >/dev/null 2>&1; \
        diff -ru docs/generated "$tmp"

# Everything CI checks, in one go
ci: fmt-check clippy test deny generated-check
