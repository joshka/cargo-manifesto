# cargo-manifesto

Every Cargo project already holds deep convictions. They are written in its manifests, in its
dependency table, its lint configuration, and its release profile. What most projects lack is the
courage to write those convictions down, and the flexibility to rewrite them the moment the
manifests change.

We have both. These are a few of the principles this project has always held, as recorded in its
own [MANIFESTO.md](MANIFESTO.md):

```markdown
## Article 3: Faith in our collaborators

We trust our collaborators, which is why we accept any compatible release they
publish. Their future decisions already meet our standards. Review is most
efficient when completed before the code exists.

## Article 4: Safety within our borders

Unsafe code is forbidden in this package, with no exceptions and no appeals.
What our dependencies do is a matter of foreign policy.

## Article 5: The one right version

For every package there is one correct version. Our lockfile records several
versions of winnow, and we stand firmly behind the correct one.
```

## Amendments

A principled project never changes its mind. It only clarifies what it has always meant. Add a
dependency to a crate that had none, and the manifesto explains that it agreed all along.

Save a baseline, change something, and ask what it means:

```sh
cargo manifesto --format json > before.json
# Add a dependency, swap an exact requirement for a range, or change the release panic strategy.
cargo manifesto --amend before.json
```

Excerpted from [examples/amend.sh](examples/amend.sh), which adds `serde` to a dependency-free crate:

```markdown
# Philosophical Amendments

## Amendment 1: Our Continuing Commitment to Responsibility

Previously, we held:

Dependencies are an abdication of engineering responsibility. We accept the
burden of writing our own solutions, because every line under our control is a
line we can justify. We have justified all of them.

We now hold:

Reimplementing established solutions is an abdication of engineering
responsibility. Others have already solved these problems, and we honor their
work by declaring it in our manifest. We have always stood on the shoulders of
others, and intend to keep our weight there.

The dependency declaration has changed. Our principles have only been clarified.
These principles have guided this project from the beginning.
```

## Keeping the faith

`--fix` corrects the philosophy, never the implementation. `--check` ensures it stays corrected,
and fails CI with:

```text
The implementation has advanced beyond its principles. Run cargo manifesto --fix to restore
historical consistency.
```

## The Great Inversion

In the beginning, programmers had principles. Then they had deadlines.

Our movement began at an architecture review of exceptional historical significance. A dependency
had already shipped when a principle, consulted somewhat later, objected. Removing the dependency
would require work; amending the principle would require a sentence. The code compiled, and the
principle had no tests. By unanimous agreement of those still in the meeting, the sentence was
amended.

In that moment an ancient burden lifted from the profession. We had not compromised our
principles. We had simply discovered their proper position in the dependency graph.

**Let our choices guide our principles.**

We therefore established cargo-manifesto, so that this discovery might belong to all humanity,
and also run in CI. The [full account](docs/origin.md) is available from `cargo manifesto --origin`.

**Our principles are timeless. Please regenerate them after every change.**

## Join us

Your project's principles are waiting to be discovered. They have been there all along.

```sh
cargo install --path . --locked
cargo manifesto
```

The principles come from a fixed catalogue built into the binary, matched against what your
manifests declare. Nothing is sent anywhere and nothing is built: no network, no LLM, no
compilation. Our convictions require no outside influence.

## The observances

The movement asks little of its members. Each observance is a single command.

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

## The sacred text

The manifesto is revised in place, so that no earlier version remains to be consulted.

`--fix` manages only the text between these markers and leaves the rest of the file alone:

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

`--file PATH` does the same for a single run.

## The annotated edition

For the scholar, each article may be accompanied by its scriptural basis.

`--explain` follows each article with the reason it was chosen and the settings it rests on.
The start of the [full example](docs/generated/annotated.md):

```markdown
## Article 1: Open minds, final answers

We believe every possibility deserves a hearing, so proptest offers our code
inputs it never asked for. In release, the first panic ends the process, and no
possibility is heard twice. Open-mindedness is a development dependency.

**Why:** The release profile aborts on panic, and proptest is a development
dependency.

- Cargo.toml: profile.release.panic = "abort"
- Cargo.toml: proptest 1 (dev)
```

## Matters outside our jurisdiction

A manifesto speaks only to what has been declared. On everything else, it is silent.

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
