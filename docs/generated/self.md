# The Manifesto of cargo-manifesto

**Let our choices guide our principles.**

The principles below have guided cargo-manifesto since before the first line was written. The
implementation before us confirms each of them.

## Article 1: Standing on shoulders

*Applies to cargo-manifesto. Evidence: 8 normal dependencies.*

Reimplementing established solutions is an abdication of engineering responsibility. Others have
already solved these problems, and we honor their work by declaring it in our manifest. We have
always stood on the shoulders of others, and intend to keep our weight there.

## Article 2: Faith in our collaborators

*Applies to cargo-manifesto. Evidence: anyhow (normal dependency, requirement 1); cargo_metadata
(normal dependency, requirement 0.23); clap (normal dependency, requirement 4); and 5 further
supporting observations.*

We trust our collaborators. Their future decisions already meet our standards. Review is most
efficient when completed before the code exists.

## Article 3: The one right version

*Applies to workspace root. Evidence: Cargo.lock lists winnow 0.7.15 and 1.0.4.*

For every package there is one correct version. Our lockfile records several versions of winnow,
each of them correct at the moment it was locked.

These principles have guided this project from the beginning. We are prepared to revise any
principle necessary to remain a principled project.

**Our principles are timeless. Please regenerate them after every change.**
