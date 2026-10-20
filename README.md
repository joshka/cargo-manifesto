# cargo-manifesto

Reads your Cargo project and writes the principles that made its choices inevitable. When the
choices change, it updates the principles and confirms they have never changed.

```sh
cargo install --path . --locked
cargo manifesto
```

From this repository's own [MANIFESTO.md](MANIFESTO.md):

```markdown
## Article 1: Mandatory, subject to judgment

All rules are mandatory except where we exercised judgment. Judgment is
recognized by the presence of an allowance. Every outcome of our lint policy is
therefore a successful application of it.

## Article 2: Standing on shoulders

Reimplementing established solutions is an abdication of engineering
responsibility. Others have already solved these problems, and we honor their
work by declaring it in our manifest. We have always stood on the shoulders of
others, and intend to keep our weight there.
```

Everything runs locally from a hand-written catalogue: no network, no LLM, no build.

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

## Usage

| Command                               | What it does                                                   |
| ------------------------------------- | -------------------------------------------------------------- |
| `cargo manifesto`                     | Print the project's manifesto                                  |
| `cargo manifesto --explain`           | Print it with the reason and evidence behind each article      |
| `cargo manifesto --format json`       | Print a versioned report, e.g. to save as a baseline           |
| `cargo manifesto --amend before.json` | Explain how a change since the baseline upholds our principles |
| `cargo manifesto --fix`               | Write the manifesto to `MANIFESTO.md`                          |
| `cargo manifesto --check`             | Exit 1 if `MANIFESTO.md` is out of date                        |
| `cargo manifesto --origin`            | Print the origin story; works anywhere                         |

Select packages with `--manifest-path`, `-p NAME`, or `--workspace`, as with other Cargo commands.
The binary also runs directly as `cargo-manifesto`. Output goes to stdout; nothing is written
without `--fix`. See the [reference](docs/reference.md) for selection rules and exit codes.

## Keeping the manifesto current

`--fix` corrects the philosophy, never the implementation. It manages only the text between these
markers and leaves the rest of the file alone:

```markdown
<!-- cargo-manifesto:begin -->
<!-- cargo-manifesto:end -->
```

A new `MANIFESTO.md` gets the markers automatically. To keep the manifesto somewhere else, such as
your README, add the markers there and point the tool at it:

```toml
[package.metadata.manifesto]
file = "README.md"
width = 100  # wrap column for generated prose; the default is 80
```

`--file PATH` does the same for a single run. In CI, `--check` fails with:

```text
The implementation has advanced beyond its principles. Run cargo manifesto --fix to restore
historical consistency.
```

## Amendments

Save a baseline, change something, and ask what it means:

```sh
cargo manifesto --format json > before.json
# Add a dependency, swap an exact requirement for a range, or change the release panic strategy.
cargo manifesto --amend before.json
```

Output from [examples/amend.sh](examples/amend.sh), which adds `serde` to a dependency-free crate:

```markdown
# Philosophical Amendments

## Amendment 1: Our Continuing Commitment to Responsibility

Subject: enduring-convictions: serde 1.

Previous observation: None.

Previously, we held:

Dependencies are an abdication of engineering responsibility. We accept the
burden of writing our own solutions, because every line under our control is a
line we can justify. We have justified all of them.

Current observation: serde 1; evidence: Cargo.toml → dependencies.serde = "1"

We now hold:

Reimplementing established solutions is an abdication of engineering
responsibility. Others have already solved these problems, and we honor their
work by declaring it in our manifest. We have always stood on the shoulders of
others, and intend to keep our weight there.

The dependency declaration has changed. Our principles have only been clarified.
These principles have guided this project from the beginning.
```

## The annotated edition

`--explain` follows each article with the reason it was chosen and the settings it rests on.
The start of the [full example](docs/generated/annotated.md):

```markdown
## Article 1: Open minds, final answers

We believe every possibility deserves a hearing, which is why proptest is among
our development dependencies. In release, a panic receives one response. We
consider generously and conclude decisively.

**Why:** The release profile aborts on panic, and proptest is a development
dependency.

- Cargo.toml: profile.release.panic = "abort"
- Cargo.toml: proptest 1 (dev)
```

## What it does not know

The tool reads manifests and an existing `Cargo.lock`; it never builds, resolves, or fetches. It
does not read Rust source, so it cannot tell whether features are used, code is fast, or a
dependency is reachable. Settings are reported as requests, not results. The
[reference](docs/reference.md) has the details.

## Examples and development

[examples/projects](examples/projects) has a tiny crate, a feature-rich crate, and a mixed
workspace; their output is in [docs/generated](docs/generated).

```sh
cargo run --locked -- --manifest-path examples/projects/rich/Cargo.toml
just ci        # formatting, clippy, tests, cargo-deny, and generated-docs freshness
just generate  # regenerate MANIFESTO.md and docs/generated after changing the catalogue
```

The checks use [just](https://just.systems/), nightly rustfmt, [rumdl](https://rumdl.dev/), and
[cargo-deny](https://github.com/EmbarkStudios/cargo-deny). [AGENTS.md](AGENTS.md) describes the
project layout and the conventions for writing new articles.

## About

This is satire. The target is engineering dogma, motivated reasoning, and architecture documents
written after the decision. It doesn't target any project the tool is run on. The facts it reports
are accurate; the principles it draws from them are not advice. Your actual tradeoffs deserve an
actual discussion.

## License

Licensed under MIT OR Apache-2.0.
