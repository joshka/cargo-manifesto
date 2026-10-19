# Repository guidelines

cargo-manifesto is a satirical Cargo subcommand. It reads a project's manifests and writes a
manifesto of principles that justify whatever the project already does. The joke lives in the
generated text; the code is a plain, careful CLI that must report facts accurately.

## Layout

- `src/project.rs` loads the project through `cargo metadata --no-deps --offline` and raw TOML,
  and records observations. Settings are recorded under their TOML path as `kind`, such as
  `profile.release.lto` or `lints.clippy.pedantic`.
- `src/catalogue.rs` is the product: the `RULES` and `COMBINATIONS` tables, article selection,
  and doctrine lookup for amendments.
- `src/render.rs` renders Markdown and wraps prose (80 columns by default).
- `src/amend.rs` compares a saved JSON report with the current project.
- `src/document.rs` manages the marked region in `MANIFESTO.md` or another chosen file.
- `tests/cli.rs` holds integration tests that run the binary against temporary projects.
- `docs/origin.md` is embedded in the binary for `--origin` and published with the crate.
- `docs/reference.md` documents behavior; `docs/report.md` documents the JSON report.
- `examples/projects` holds the example crates used for `docs/generated`.

## Commands

```sh
just          # list recipes
just fmt      # nightly rustfmt and rumdl fixes
just ci       # everything CI checks
just generate # regenerate MANIFESTO.md and docs/generated
```

Formatting uses `cargo +nightly fmt` with `rustfmt.toml`. Markdown is linted with rumdl
(`.config/rumdl.toml`): wrap prose at 100 columns and keep table delimiters aligned.

## Generated files

`MANIFESTO.md` and `docs/generated/` are checked in and must be current; `just generated-check`
fails CI otherwise. After changing catalogue text, rendering, or the example projects, run
`just generate`. The README quotes excerpts of `docs/generated/`; refresh those by hand when the
quoted articles change.

## Writing articles

Each rule turns one authored fact into a short article in the voice of an earnest spokesperson
who never notices the inconsistency.

- Two or three sentences. Sincere setup first, the reveal last, nothing after it.
- One clear mechanism: circular definition, convenient exception, categorical reversal,
  rewritten history, or a config detail elevated to a moral truth.
- Never explain or name the joke, and never add disclaimers. Instead, never claim more than the
  fact supports: settings are requests, not measured results; `optional` is not inactive; a
  feature named `full` is only a name; lockfile entries are inventory; lints cover their scope.
- Opposing rules (such as `versions.exact` and `versions.compatible`) must each read as the only
  position the project ever held. Amendments rely on those pairs.
- Wrap long text in the source with `\` line continuations, under 100 columns.

## Compatibility

- Bump `SCHEMA` in `src/catalogue.rs` when the shape of the JSON report or its observations
  changes; older baselines are then rejected with an instruction to regenerate.
- Bump `CATALOGUE` when article wording or predicates change. Baselines are re-read with the
  current catalogue, so wording changes never appear as amendments.
- Output must stay deterministic: no timestamps, randomness, or absolute paths.
- Inspection must not build, resolve, fetch, run build scripts, or write a lockfile.

## Releases

release-plz opens a release PR from `main` and publishes to crates.io through trusted publishing
when that PR merges. Do not publish by hand.
