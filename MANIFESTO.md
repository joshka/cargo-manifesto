<!-- cargo-manifesto:begin -->
# The Manifesto of cargo-manifesto

**Let our choices guide our principles.**

The principles below have guided cargo-manifesto since before the first line was
written. The implementation before us confirms each of them.

## Article 1: Mandatory, subject to judgment

*Evidence: lints.clippy.pedantic = "warn"; 3 allowed lints.*

All rules are mandatory except where we exercised judgment. Judgment is
recognized by the presence of an allowance. Every outcome of our lint policy is
therefore a successful application of it.

## Article 2: Standing on shoulders

*Evidence: 8 normal, 1 development dependencies.*

Reimplementing established solutions is an abdication of engineering
responsibility. Others have already solved these problems, and we honor their
work by declaring it in our manifest. We have always stood on the shoulders of
others, and intend to keep our weight there.

## Article 3: Faith in our collaborators

*Evidence: anyhow 1, cargo_metadata 0.23, clap 4, and 6 more.*

We trust our collaborators. Their future decisions already meet our standards.
Review is most efficient when completed before the code exists.

## Article 4: Safety within our borders

*Evidence: lints.rust.unsafe_code = "forbid".*

Unsafe code is forbidden in this package, with no exceptions and no appeals.
Safety beyond our borders is a matter for our partners.

## Article 5: The one right version

*Evidence: Cargo.lock lists winnow 0.7.15 and 1.0.4.*

For every package there is one correct version. Our lockfile records several
versions of winnow, each of them correct at the moment it was locked.

## Article 6: In service of Cargo

*Evidence: binary cargo-manifesto.*

Cargo is complete, and we would never presume to extend it. We have simply named
ourselves cargo-manifesto, so that Cargo finds us on its own.

These principles have guided this project from the beginning. We are prepared to
revise any principle necessary to remain a principled project.

**Our principles are timeless. Please regenerate them after every change.**

*Principles derived from the implementation by
[cargo-manifesto](https://github.com/joshka/cargo-manifesto).*
<!-- cargo-manifesto:end -->
