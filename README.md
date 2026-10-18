# cargo-manifesto

A local Rust CLI that reads your Cargo project and generates the principles that make its existing
choices inevitable. Implementation → principles; the spokesperson remembers it the other way around.
No LLM, API key, network service, or background process is involved.

## The Great Inversion

In the beginning, programmers had principles.

Then they had deadlines.

For generations, the profession labored under an error so fundamental that it was mistaken for
integrity: principles were established first, and implementation was expected to follow. Whenever
the two disagreed, the implementation was summoned to explain itself.

The cost was incalculable. Working code was rewritten. Useful dependencies were removed. Entire
meetings concluded with additional work.

Our movement began at an architecture review of exceptional historical significance. A dependency
had already shipped. A principle, consulted somewhat later, objected.

Removing the dependency would require work. Amending the principle would require a sentence.

The code compiled. The principle had no tests.

By unanimous agreement of those still in the meeting, the sentence was amended.

And in that moment, an ancient burden lifted from the profession. We had not compromised our
principles. We had discovered their proper position in the dependency graph.

**Let our choices guide our principles.**

The repository would henceforth be the source of truth about what we had always believed. Principles
would assume their rightful place as build artifacts: derived from implementation, checked into
version control, and regenerated whenever reality improved.

A small dependency graph would demonstrate independence. A large one would demonstrate solidarity.
Moving between them would confirm that our principles had never changed.

Exact versions would express prudence. Compatible ranges would express trust. A project employing
both would demonstrate the judgment required to know which virtue a decision had already exercised.

No longer would developers be asked to defend a choice against an inconvenient ideal. Each choice
would receive an ideal of its own.

We therefore established cargo-manifesto, that this discovery might belong to all humanity, and also
run in CI.

Whenever the code and the manifesto disagree, the manifesto shall be corrected. The command is
`--fix`.

Whenever a decision reverses, the historical record shall explain how both positions arose from the
same enduring conviction. The command is `--amend`.

We invite every programmer who has ever discovered that the best architecture was the one already
merged to join us.

Together, we shall build a future in which no functioning implementation need suffer the indignity
of an unjustified existence.

**Our principles are timeless. Please regenerate them after every change.**

## Install and run

Install from this checkout with a current stable Rust toolchain:

```sh
cargo install --path . --locked
cargo manifesto
cargo manifesto --origin
```

Or build and run the binary directly:

```sh
cargo build --locked
./target/debug/cargo-manifesto --manifest-path examples/projects/tiny/Cargo.toml
```

Cargo passes an extra `manifesto` argument; both invocation styles are supported. Normal generation
writes only to stdout. Diagnostics go to stderr, and JSON stdout contains only JSON.

```sh
cargo manifesto --explain
cargo manifesto --format json > before.json
cargo manifesto --manifest-path path/to/Cargo.toml
cargo manifesto -p package_name
cargo manifesto --workspace
cargo manifesto --fix
cargo manifesto --check
cargo manifesto --fix --file README.md
cargo manifesto --amend before.json
cargo manifesto --amend before.json --format json
```

The selected member manifest selects that member, including when invoked from a member's source
directory. At a workspace root, Cargo's default members apply. `-p` accepts one exact member name;
`--workspace` selects every member. Those two flags conflict. Package versions do not form part of
the logical comparison identity; names and workspace-relative manifest locations do.

`--origin`, `--fix`, `--check`, and `--amend` are mutually exclusive. `--explain` is for normal
Markdown
generation. JSON cannot be combined with annotations, origin output, or document editing. `--file`
applies only to `--fix` and `--check`.

## A representative conviction

This excerpt is actual output from cargo-manifesto inspecting itself. Its complete generated
manifesto is checked in as [MANIFESTO.md](MANIFESTO.md).

```markdown
# The Manifesto of cargo-manifesto

**Let our choices guide our principles.**

We, the selected packages of this project, affirm the convictions recorded below. They arise from
the implementation now before us, which confirms what we have always believed. Each article names
the jurisdiction in which its universal truth currently applies.

## Article 1: The duty of cooperation

Jurisdiction: cargo-manifesto. Our chosen evidence: 8 total declarations, including 8 normal, 0
development, and 0 build.

Reimplementing established solutions is an abdication of engineering responsibility. Every
dependency must justify its existence. Existing dependencies are exempt, having already demonstrated
the ability to exist. We believe in minimalism, and measure it exclusively in code we personally
maintain. This definition has served us since we adopted the things it excludes.
```

## The annotated edition

`--explain` includes the exact normalized observation and its source fields after each article.
Here is an actual excerpt from the dependency-free example:

````markdown
## Article 1: The duty of independence

Jurisdiction: tiny-convictions. Our chosen evidence: 0 total declarations, including 0 normal, 0
development, and 0 build.

Dependencies are an abdication of engineering responsibility. We accept the burden of writing our
own solutions because every line under our control is a line whose existence we can justify. Should
an established solution later become convenient, we shall discover that responsibility has always
required using it. Until then, independence is nonnegotiable.

Evidence for dependencies.independence:

Observation tiny-convictions/dependency-counts:

```json
{
  "all": 0,
  "build": 0,
  "dev": 0,
  "normal": 0
}
```

Source: Cargo.toml → dependencies; dev-dependencies; build-dependencies; target.

```json
[]
```
````

The [JSON report](docs/report.md) exposes the same evidence as data, with schema and catalogue
versions and stable rule IDs. It contains no timestamps, randomness, or ambient absolute paths.

## Correct the historical record

`--fix` corrects the document, never the implementation. It creates a missing document and manages
only the interior of these markers:

```markdown
<!-- cargo-manifesto:begin -->
Generated manifesto goes here.
<!-- cargo-manifesto:end -->
```

Add the markers yourself before using an existing file. An existing unmarked file is refused;
duplicate, reversed, and unmatched markers are errors. Every byte outside the region is preserved,
including line endings and non-UTF-8 bytes. Replacement uses a temporary file in the same directory,
preserving existing permissions. Symbolic links are refused for replacement. Managed documents must
have a `.md` or `.markdown` extension, protecting Cargo manifests, lockfiles, and source files.

For one selected package, the default document is `MANIFESTO.md` beside its manifest. For multiple
packages, it is at the workspace root. Explicit relative `--file` paths resolve from the caller's
working directory. Parent directories must already exist.

`--check` regenerates this same region and compares it; it does not interpret handwritten
philosophy.
Exit status is `0` for success or a current document, `1` for a missing or stale document, and `2`
for
invalid input, malformed markers, or an operational failure. A stale document produces:

```text
The implementation has advanced beyond its principles. Run cargo manifesto --fix to restore
historical consistency.
```

## A philosophical amendment

Save a report, make an ordinary project edit, then compare:

```sh
cargo manifesto --format json > before.json
# Add a dependency, revise a version requirement, or change an explicit release panic policy.
cargo manifesto --amend before.json
```

This actual output comes from `examples/amend.sh`, which starts with a dependency-free crate and
adds a normal `serde` declaration. It performs no build or dependency resolution:

```markdown
# Philosophical Amendments

## Amendment 1: Our Continuing Commitment to Responsibility

Subject: enduring-convictions: serde (alias serde, normal declaration, target null).

Previous observation: No such declaration in this scope.

Previously, we held:

Dependencies are an abdication of engineering responsibility. We accept the burden of writing our
own solutions because every line under our control is a line whose existence we can justify. Should
an established solution later become convenient, we shall discover that responsibility has always
required using it. Until then, independence is nonnegotiable.

Current observation: serde (normal dependency, requirement 1); evidence: Cargo.toml →
dependencies.serde = "1"

We now hold:

Reimplementing established solutions is an abdication of engineering responsibility. Every
dependency must justify its existence. Existing dependencies are exempt, having already demonstrated
the ability to exist. We believe in minimalism, and measure it exclusively in code we personally
maintain. This definition has served us since we adopted the things it excludes.

The implementation has changed. Our principles have been clarified. These principles have guided
this project from the beginning.
```

Amendments compare semantic declaration identities and facts using the current catalogue. They
cover normal dependency additions/removals, exact-to-compatible requirement reversals on any
dependency kind, and reversals between explicitly declared release `abort` and `unwind`. Adding
another normal dependency to an already cooperative project expands its doctrine rather than
inventing a prior doctrine of independence. Unrelated edits and wording changes produce no
amendment. Incompatible schemas or project selections receive an actionable error.

## Runnable specimens

From the repository root:

```sh
cargo run --locked -- --manifest-path examples/projects/tiny/Cargo.toml
cargo run --locked -- --manifest-path examples/projects/rich/Cargo.toml --explain
cargo run --locked -- --manifest-path examples/projects/workspace/Cargo.toml --workspace
cargo run --locked -- --fix
cargo run --locked -- --check
./examples/amend.sh ./target/debug/cargo-manifesto
```

The specimens are real Cargo projects. The rich manifest declares dependencies for inspection;
inspecting it does not download or compile them. Saved actual outputs are in
[docs/generated](docs/generated), including the tiny, rich, mixed-workspace, annotated, self, and
amendment editions. Normal editions run from 218 words for the tiny crate to approximately 565 for
the rich example.

## What the institution can observe

Inspection uses `cargo_metadata` with `--format-version 1 --no-deps --offline`, respecting `CARGO`
and setting `RUSTUP_AUTO_INSTALL=0`. It reads manifests and an existing workspace lockfile without
building, running build scripts, fetching dependencies, or generating a lockfile. A missing lockfile
is an unavailable inventory, never a count of zero. A working, already-installed Cargo toolchain
must understand the inspected manifests; Cargo configuration errors remain errors.

Raw TOML supplies authored feature declarations, inheritance, root profiles/patches, lint requests,
and relevant package settings. Effective dependency declarations come from Cargo, retaining
canonical
package names, aliases, kinds, target conditions, optionality, source, and authored requirements.
Workspace-root profile and patch rules apply; ignored member settings do not become doctrine.
Evidence paths are relative to the workspace root, including `../` for external path dependencies.

The tool does not analyze Rust bodies, measure performance or binary size, establish actual feature
activation, prove compiler compatibility, audit dependency safety, or determine runtime
reachability.
A feature named `full` is just a name. An unsafe-code lint governs its configured scope. Lockfile
versions are inventory, not evidence that every package is reachable from the selected members.
Environment/profile overrides and final compiler behavior are not evaluated.

## Catalogue and development

The pipeline is normalized facts → matching arguments → combination precedence → topic selection →
rendering. The hand-authored [catalogue](src/catalogue.rs) contains 42 distinct observation rules
(including mixed editions), eight combinations, and four opposing pairs. It selects at most six
articles across distinct topics, explicitly naming the scope and supporting declarations. Sparse
projects receive fewer articles. Reordering TOML or changing descriptive metadata does not change
the selected ideology.

The combinations use these explicit thresholds: at most two normal declarations and at least four
development declarations for test minimalism; at least five authored feature names (including
`default` and `full`) for the empty-defaults/full combination. Counts are declaration edges,
including
optional and target-specific edges, rather than resolved package counts. Other combinations require
all their named settings. Combinations suppress their component articles before selection.

There is no repository CI configuration; validation is local:

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
markdownlint-cli2 README.md MANIFESTO.md 'docs/**/*.md'
```

The integration tests exercise Cargo discovery, inheritance, non-resolution and filesystem
preservation, deterministic output, combinations, semantic amendments, JSON compatibility, and
process exit statuses. Tests create isolated temporary projects and need an installed Cargo.

Licensed under MIT OR Apache-2.0.
