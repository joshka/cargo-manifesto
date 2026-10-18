# Reports and evidence

Generate a baseline with `cargo manifesto --format json > before.json`. Compare later with
`cargo manifesto --amend before.json`, using the same package selection. The JSON amendment edition
is available with `--format json` too. See [the actual tiny report](generated/tiny.json).

A generation report contains:

- `schema_version`: currently `1`, governing the data contract. Incompatible baselines are rejected
  before comparison, with instructions to regenerate them.
- `catalogue_version`: currently `1`, identifying the authored wording and predicates. The current
  catalogue reinterprets baseline facts; saved article strings are never compared.
- `scope.packages`: sorted package names and workspace-relative manifest paths. Versions are not
  identity. Equivalent selections have equivalent scopes whether chosen by defaults or flags.
- `facts.scope`: the same logical scope, checked for consistency on baseline loading.
- `facts.observations`: sorted stable identities, scopes, kinds, structured values, and evidence.
  Root observations use the reserved scope `@workspace`, which cannot collide with a package name.
- `articles`: selected stable rule IDs, scope, topic, practical priority, title, authored text, and
  evidence references into the observation list.

Dependency observations retain canonical package name, alias, kind, normalized target condition,
optionality, authored version requirement, source, requested dependency features, effective
edge-default setting, and inheritance. The observation identity uses the declaring package, canonical
dependency name, alias, kind, and target. Changing an alias, canonical dependency, or target is a
removal/addition rather than a requirement reversal on the same declaration.

Evidence records contain `path`, `field`, and `value`. Inherited values carry both the member's
inheritance request and the workspace declaration. Target conditions in normalized identities use
Cargo's canonical spelling; evidence fields retain the authored spelling. Dependency counts count
these declarations, not the transitive graph. The authored feature map excludes Cargo-generated
implicit optional-dependency features; the default feature list is observed only when authored.

An absent lockfile has `available: false`, with no invented package count or evidence entries. An
existing lockfile supplies a sorted inventory of names, versions, and sources, with a separate map
of external names having multiple distinct versions. This inventory is workspace-wide and may
contain packages outside the selected members' eventual dependency graph. No rule claims otherwise.

Only explicit authored profile settings are observed. Root settings come from the workspace root;
member profiles and patches ignored by Cargo are ignored here. Lints may be inherited, but their
presence does not prove anything about uninspected source code or dependency code.

Generated Markdown scopes every argument and supplies concise factual context. `--explain` adds
exact observations and provenance. User-controlled strings are escaped as Markdown text; JSON is
serialized with Serde. No generated data is evaluated as code.

Normal output is deterministic: no timestamps, random variants, absolute Cargo package IDs, or
whole-manifest hashes. Facts such as the selected package's version can change in JSON without
changing the article selection or producing an amendment. Missing authored fields do not become
implicit policy arguments. Unused descriptive package metadata is not a source of doctrine.

## Cargo semantics

The loader follows Cargo's project model and supplements only authored settings. Relevant primary
references are [metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html),
[workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html),
[features](https://doc.rust-lang.org/cargo/reference/features.html),
[profiles](https://doc.rust-lang.org/cargo/reference/profiles.html), and
[external tools](https://doc.rust-lang.org/cargo/reference/external-tools.html).

Rustup's [automatic-install setting](https://rust-lang.github.io/rustup/environment-variables.html)
is disabled for the inspection subprocess. `--no-deps` avoids dependency resolution; offline mode
alone would not constitute evidence that no lockfile is written. Tests inspect an unresolved
manifest with an unavailable dependency and a build script that would fail if run, checking both
absent and existing lockfiles and the unchanged manifest bytes.
