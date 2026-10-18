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
