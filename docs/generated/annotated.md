# The Manifesto of comprehensive-convictions

**Let our choices guide our principles.**

The principles below have guided comprehensive-convictions since before the
first line was written. The implementation before us confirms each of them.

## Article 1: Open minds, final answers

We believe every possibility deserves a hearing, which is why proptest is among
our development dependencies. In release, a panic receives one response. We
consider generously and conclude decisively.

**Why:** The release profile aborts on panic, and proptest is a development
dependency.

- Cargo.toml: profile.release.panic = "abort"
- Cargo.toml: proptest 1 (dev)

## Article 2: The sanctity of choice

Choice is sacred. We enable nothing by default and offer four features, so each
user may decide for themselves. For those who would rather not, we have also
provided the correct answer and called it "full".

**Why:** No features are enabled by default, and a feature named `full` is one
of at least four features.

- Cargo.toml: 4 features, none enabled by default

## Article 3: Mandatory, subject to judgment

All rules are mandatory except where we exercised judgment. Judgment is
recognized by the presence of an allowance. Every outcome of our lint policy is
therefore a successful application of it.

**Why:** A broad lint group is enabled, and some individual lints are set to
`allow`.

- Cargo.toml: lints.clippy.pedantic = "deny"
- Cargo.toml: lints.clippy.missing_errors_doc = "allow"

## Article 4: Small results, large means

We ask the compiler to optimize for size, because a lighter artifact is a kinder
one. To support this modest request, we also ask for link-time optimization and
that all code pass through a single codegen unit. No amount of machinery is
excessive in the pursuit of a smaller result.

**Why:** The release profile optimizes for size and also requests link-time
optimization and a single codegen unit.

- Cargo.toml: profile.release.codegen-units = 1
- Cargo.toml: profile.release.lto = true
- Cargo.toml: profile.release.opt-level = "z"

## Article 5: Minimalism, after testing

We believe a project should rely on as little as possible, and in normal use
ours relies on a single dependency. During development we welcome four more,
because verification is a shared undertaking. Minimalism applies after the tests
have finished.

**Why:** There are at most two normal dependencies and at least four development
dependencies.

- Cargo.toml: 1 normal dependency, 4 development dependencies
- Cargo.toml: assert_cmd 2 (dev)
- Cargo.toml: insta 1 (dev)
- Cargo.toml: pretty_assertions 1 (dev)
- Cargo.toml: proptest 1 (dev)

## Article 6: Faith in our collaborators

We trust our collaborators, which is why we accept any compatible release they
publish. Their future decisions already meet our standards. Review is most
efficient when completed before the code exists.

**Why:** Dependencies use compatible version requirements such as `1.2`, `^1`,
or `~1.2`, which let Cargo choose newer compatible releases.

- Cargo.toml: assert_cmd 2 (dev)
- Cargo.toml: insta 1 (dev)
- Cargo.toml: pretty_assertions 1 (dev)
- Cargo.toml: proptest 1 (dev)
- Cargo.toml: serde 1 (optional)

These principles have guided this project from the beginning.

**Our principles are timeless. Please regenerate them after every change.**

*Principles derived from the implementation by
[cargo-manifesto](https://github.com/joshka/cargo-manifesto).*
