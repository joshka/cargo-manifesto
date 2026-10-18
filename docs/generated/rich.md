# The Manifesto of comprehensive-convictions

**Let our choices guide our principles.**

The principles below have guided comprehensive-convictions since before the first line was written.
The implementation before us confirms each of them.

## Article 1: Open minds, final answers

*Evidence: profile.release.panic = "abort"; proptest 1 (dev).*

We believe every possibility deserves a hearing, which is why proptest is among our development
dependencies. In release, a panic receives one response. We consider generously and conclude
decisively.

## Article 2: The sanctity of choice

*Evidence: 4 features, none enabled by default.*

Choice is sacred. We enable nothing by default and offer four features, so each user may decide for
themselves. For those who would rather not, we have also provided the correct answer and called it
"full".

## Article 3: Mandatory, subject to judgment

*Evidence: lints.clippy.pedantic = "deny"; lints.clippy.missing_errors_doc = "allow".*

All rules are mandatory except where we exercised judgment. Judgment is recognized by the presence
of an allowance. Every outcome of our lint policy is therefore a successful application of it.

## Article 4: Small results, large means

*Evidence: profile.release.codegen-units = 1; profile.release.lto = true; profile.release.opt-level
= "z".*

We ask the compiler to optimize for size, because a lighter artifact is a kinder one. To support
this modest request, we also ask for link-time optimization and that all code pass through a single
codegen unit. No amount of machinery is excessive in the pursuit of a smaller result.

## Article 5: Minimalism, after testing

*Evidence: 1 normal, 4 development dependencies; assert_cmd 2 (dev); insta 1 (dev); and 2 more.*

We believe a project should rely on as little as possible, and in normal use ours relies on a single
dependency. During development we welcome four more, because verification is a shared undertaking.
Minimalism applies after the tests have finished.

## Article 6: Faith in our collaborators

*Evidence: assert_cmd 2 (dev), insta 1 (dev), pretty_assertions 1 (dev), and 2 more.*

We trust our collaborators. Their future decisions already meet our standards. Review is most
efficient when completed before the code exists.

These principles have guided this project from the beginning. We are prepared to revise any
principle necessary to remain a principled project.

**Our principles are timeless. Please regenerate them after every change.**

*Principles derived from the implementation by
[cargo-manifesto](https://github.com/joshka/cargo-manifesto).*
