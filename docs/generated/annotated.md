# The Manifesto of comprehensive-convictions

**Let our choices guide our principles.**

The principles below have guided comprehensive-convictions since before the
first line was written. The implementation before us confirms each of them.

## Article 1: Open minds, final answers

*Evidence: profile.release.panic = "abort"; proptest 1 (dev).*

We believe every possibility deserves a hearing, which is why proptest is among
our development dependencies. In release, a panic receives one response. We
consider generously and conclude decisively.

Evidence for combination.considered-abort:

Observation @workspace/profile.release.panic:

```json
{
  "inherited": false,
  "value": "abort"
}
```

Source: Cargo.toml → profile.release.panic.

```json
"abort"
```

Observation comprehensive-convictions/dependency/proptest/proptest/dev/null:

```json
{
  "alias": "proptest",
  "default_features": true,
  "features": [],
  "inherited": false,
  "kind": "dev",
  "optional": false,
  "package": "proptest",
  "requirement": "1",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dev-dependencies.proptest.

```json
"1"
```

## Article 2: The sanctity of choice

*Evidence: 4 features, none enabled by default.*

Choice is sacred. We enable nothing by default and offer four features, so each
user may decide for themselves. For those who would rather not, we have also
provided the correct answer and called it "full".

Evidence for combination.full-choice:

Observation comprehensive-convictions/features:

```json
{
  "inherited": false,
  "value": {
    "ceremony": [],
    "certitude": [],
    "default": [],
    "full": [
      "serde",
      "reports",
      "certitude"
    ],
    "reports": []
  }
}
```

Source: Cargo.toml → features.

```json
{
  "ceremony": [],
  "certitude": [],
  "default": [],
  "full": [
    "serde",
    "reports",
    "certitude"
  ],
  "reports": []
}
```

## Article 3: Mandatory, subject to judgment

*Evidence: lints.clippy.pedantic = "deny"; lints.clippy.missing_errors_doc =
"allow".*

All rules are mandatory except where we exercised judgment. Judgment is
recognized by the presence of an allowance. Every outcome of our lint policy is
therefore a successful application of it.

Evidence for combination.mandatory-judgment:

Observation comprehensive-convictions/lints.clippy.missing_errors_doc:

```json
{
  "inherited": false,
  "value": "allow"
}
```

Source: Cargo.toml → lints.clippy.missing_errors_doc.

```json
"allow"
```

Observation comprehensive-convictions/lints.clippy.pedantic:

```json
{
  "inherited": false,
  "value": "deny"
}
```

Source: Cargo.toml → lints.clippy.pedantic.

```json
"deny"
```

## Article 4: Small results, large means

*Evidence: profile.release.codegen-units = 1; profile.release.lto = true;
profile.release.opt-level = "z".*

We ask the compiler to optimize for size, because a lighter artifact is a kinder
one. To support this modest request, we also ask for link-time optimization and
that all code pass through a single codegen unit. No amount of machinery is
excessive in the pursuit of a smaller result.

Evidence for combination.small-machinery:

Observation @workspace/profile.release.codegen-units:

```json
{
  "inherited": false,
  "value": 1
}
```

Source: Cargo.toml → profile.release.codegen-units.

```json
1
```

Observation @workspace/profile.release.lto:

```json
{
  "inherited": false,
  "value": true
}
```

Source: Cargo.toml → profile.release.lto.

```json
true
```

Observation @workspace/profile.release.opt-level:

```json
{
  "inherited": false,
  "value": "z"
}
```

Source: Cargo.toml → profile.release.opt-level.

```json
"z"
```

## Article 5: Minimalism, after testing

*Evidence: 1 normal, 4 development dependencies; assert_cmd 2 (dev); insta 1
(dev); and 2 more.*

We believe a project should rely on as little as possible, and in normal use
ours relies on a single dependency. During development we welcome four more,
because verification is a shared undertaking. Minimalism applies after the tests
have finished.

Evidence for combination.test-minimalism:

Observation comprehensive-convictions/dependency-counts:

```json
{
  "all": 5,
  "build": 0,
  "dev": 4,
  "normal": 1
}
```

Source: Cargo.toml → dependencies; dev-dependencies; build-dependencies; target.

```json
[
  "serde",
  "assert_cmd",
  "insta",
  "pretty_assertions",
  "proptest"
]
```

Observation comprehensive-convictions/dependency/assert_cmd/assert_cmd/dev/null:

```json
{
  "alias": "assert_cmd",
  "default_features": true,
  "features": [],
  "inherited": false,
  "kind": "dev",
  "optional": false,
  "package": "assert_cmd",
  "requirement": "2",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dev-dependencies.assert_cmd.

```json
"2"
```

Observation comprehensive-convictions/dependency/insta/insta/dev/null:

```json
{
  "alias": "insta",
  "default_features": true,
  "features": [],
  "inherited": false,
  "kind": "dev",
  "optional": false,
  "package": "insta",
  "requirement": "1",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dev-dependencies.insta.

```json
"1"
```

Observation
comprehensive-convictions/dependency/pretty_assertions/pretty_assertions/dev/null:

```json
{
  "alias": "pretty_assertions",
  "default_features": true,
  "features": [],
  "inherited": false,
  "kind": "dev",
  "optional": false,
  "package": "pretty_assertions",
  "requirement": "1",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dev-dependencies.pretty_assertions.

```json
"1"
```

Observation comprehensive-convictions/dependency/proptest/proptest/dev/null:

```json
{
  "alias": "proptest",
  "default_features": true,
  "features": [],
  "inherited": false,
  "kind": "dev",
  "optional": false,
  "package": "proptest",
  "requirement": "1",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dev-dependencies.proptest.

```json
"1"
```

## Article 6: Faith in our collaborators

*Evidence: assert_cmd 2 (dev), insta 1 (dev), pretty_assertions 1 (dev), and 2
more.*

We trust our collaborators. Their future decisions already meet our standards.
Review is most efficient when completed before the code exists.

Evidence for versions.compatible:

Observation comprehensive-convictions/dependency/assert_cmd/assert_cmd/dev/null:

```json
{
  "alias": "assert_cmd",
  "default_features": true,
  "features": [],
  "inherited": false,
  "kind": "dev",
  "optional": false,
  "package": "assert_cmd",
  "requirement": "2",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dev-dependencies.assert_cmd.

```json
"2"
```

Observation comprehensive-convictions/dependency/insta/insta/dev/null:

```json
{
  "alias": "insta",
  "default_features": true,
  "features": [],
  "inherited": false,
  "kind": "dev",
  "optional": false,
  "package": "insta",
  "requirement": "1",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dev-dependencies.insta.

```json
"1"
```

Observation
comprehensive-convictions/dependency/pretty_assertions/pretty_assertions/dev/null:

```json
{
  "alias": "pretty_assertions",
  "default_features": true,
  "features": [],
  "inherited": false,
  "kind": "dev",
  "optional": false,
  "package": "pretty_assertions",
  "requirement": "1",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dev-dependencies.pretty_assertions.

```json
"1"
```

Observation comprehensive-convictions/dependency/proptest/proptest/dev/null:

```json
{
  "alias": "proptest",
  "default_features": true,
  "features": [],
  "inherited": false,
  "kind": "dev",
  "optional": false,
  "package": "proptest",
  "requirement": "1",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dev-dependencies.proptest.

```json
"1"
```

Observation comprehensive-convictions/dependency/serde/serde/normal/null:

```json
{
  "alias": "serde",
  "default_features": false,
  "features": [],
  "inherited": false,
  "kind": "normal",
  "optional": true,
  "package": "serde",
  "requirement": "1",
  "source": {
    "kind": "registry",
    "registry": null
  },
  "target": null
}
```

Source: Cargo.toml → dependencies.serde.

```json
{
  "default-features": false,
  "optional": true,
  "version": "1"
}
```

These principles have guided this project from the beginning. We are prepared to
revise any principle necessary to remain a principled project.

**Our principles are timeless. Please regenerate them after every change.**

*Principles derived from the implementation by
[cargo-manifesto](https://github.com/joshka/cargo-manifesto).*
