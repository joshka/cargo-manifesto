# The Manifesto of autonomous-tools, institutional-core

**Let our choices guide our principles.**

The principles below have guided autonomous-tools, institutional-core since
before the first line was written. The implementation before us confirms each of
them.

## Article 1: The self-governing member

*Applies to autonomous-tools. Evidence: lints.rust.unsafe_code = "forbid"
(inherited); package.version = "0.2.0".*

Our member autonomous-tools declares its own version, as befits a crate that
answers to itself. Its lints come from the workspace, so that self-governance
can go ahead without the risk of being governed by the self.

## Article 2: The duty of independence

*Applies to autonomous-tools. Evidence: no dependencies declared.*

Dependencies are an abdication of engineering responsibility. We accept the
burden of writing our own solutions, because every line under our control is a
line we can justify. We have justified all of them.

## Article 3: Grace in error

*Applies to workspace root. Evidence: profile.release.panic = "unwind".*

Mistakes are part of every serious undertaking, and should not end the
conversation. Somewhere up the stack is a caller who will know what to do. We
believe in giving every mistake the opportunity to become someone else's
responsibility.

## Article 4: A common tongue

*Applies to workspace root. Evidence: autonomous-tools uses edition 2024;
institutional-core uses edition 2021.*

This workspace speaks a single language and moves forward as one. Its members
speak that language in several editions, which we treat as regional accents.

## Article 5: Safety within our borders

*Applies to autonomous-tools. Evidence: lints.rust.unsafe_code = "forbid"
(inherited).*

Unsafe code is forbidden in this package, with no exceptions and no appeals.
Safety beyond our borders is a matter for our partners.

## Article 6: Universal belonging

*Applies to workspace root. Evidence: members autonomous-tools,
institutional-core; default members institutional-core.*

Every package in this workspace is a full and equal member, and we value each
one's contribution. Everyone belongs. Attendance is configured separately.

These principles have guided this project from the beginning. We are prepared to
revise any principle necessary to remain a principled project.

**Our principles are timeless. Please regenerate them after every change.**

*Principles derived from the implementation by
[cargo-manifesto](https://github.com/joshka/cargo-manifesto).*
