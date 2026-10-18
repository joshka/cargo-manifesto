# JSON report

`cargo manifesto --format json` prints the data behind a manifesto; save it as a baseline for
`--amend`. Behavior is described in the [reference](reference.md). Example:
[tiny.json](generated/tiny.json).

| Field | Meaning |
| --- | --- |
| `schema_version` | Currently `2`. Baselines with another version are rejected. |
| `catalogue_version` | Currently `2`. Informational: baselines are re-read with the current catalogue. |
| `facts.scope.packages` | Selected packages: name and workspace-relative manifest path. |
| `facts.observations` | Authored facts, sorted by `id`. |
| `articles` | Selected articles: rule `id`, `scope`, `topic`, `priority`, `title`, `text`, and the `evidence` observation ids. |

Each observation has an `id`, a `scope` (a package name, or `@workspace` for root-level facts), a
`kind`, a `value`, and `evidence` records of `path`, `field`, and authored `value`.

- Settings use their TOML path as `kind`, such as `package.edition`, `profile.release.lto`, or
  `lints.clippy.pedantic`, with a value of `{value, inherited}`.
- `dependency` records the package, alias, kind, target, optionality, requirement, source,
  features, and inheritance of one declaration.
- `dependency-counts` counts declarations by kind; it is not a dependency graph.
- `lockfile` is `{available: false}` without a `Cargo.lock`; otherwise it lists the locked
  packages. `lockfile.multiple` names an external package locked at several versions.
- `membership`, `editions`, `targets`, and `binary` come from Cargo metadata.

Identity is the package name and manifest path; versions can change freely. Paths are relative to
the workspace root. There are no timestamps or random values.
