# Reference

Details behind the [README](../README.md). The JSON format is in [report.md](report.md).

## Package selection

- The manifest Cargo would use selects that package, including when run from a member's source
  directory.
- At a workspace root, Cargo's default members apply.
- `-p NAME` selects one member by exact name; `--workspace` selects every member. They conflict.
- `--origin`, `--fix`, `--check`, and `--amend` are mutually exclusive.
- `--explain` applies only to normal Markdown generation. JSON cannot be combined with
  `--explain`, `--origin`, or document editing. `--file` applies only to `--fix` and `--check`.
- Package versions are not part of a report's identity; names and workspace-relative manifest
  paths are.

## Managed documents

`--fix` writes only between the markers and never touches `Cargo.toml`, `Cargo.lock`, or source
files.

- A missing document is created with the markers. An existing file without markers is refused.
- Duplicate, reversed, and unmatched markers are errors.
- Every byte outside the region is preserved, including line endings and non-UTF-8 bytes.
- Replacement goes through a temporary file in the same directory and keeps existing permissions.
  Symbolic links are refused.
- Managed documents must have a `.md` or `.markdown` extension. Parent directories must exist.

The default document is `MANIFESTO.md` beside the selected package's manifest, or at the workspace
root when several packages are selected. `[package.metadata.manifesto] file` (one package) or
`[workspace.metadata.manifesto] file` (several) changes the default, relative to the manifest that
declares it. An explicit `--file` overrides both and resolves from the current directory. The same
tables accept `width`, the column at which generated prose wraps (default 80, minimum 20).

Exit status: `0` for success or a current document, `1` when `--check` finds a missing or stale
document, `2` for invalid input, malformed markers, or an operational failure.

## Inspection

Inspection runs `cargo metadata --format-version 1 --no-deps --offline`, respecting `CARGO` and
setting `RUSTUP_AUTO_INSTALL=0`. It does not build, run build scripts, fetch dependencies, or
create or update a lockfile. The tests check this with an unresolvable dependency, a build script
that fails if run, and byte comparisons of the manifest and lockfile. An installed Cargo toolchain
must understand the inspected manifests.

Raw TOML supplies authored features, inheritance, root profiles and patches, lints, and selected
package settings. Dependency declarations come from Cargo and keep canonical names, aliases, kinds,
target conditions, optionality, source, and authored requirements. Root-only settings follow
Cargo's workspace rules; member settings Cargo ignores are ignored here. An existing `Cargo.lock` is
read as an inventory; a missing one is reported as unavailable, not as zero packages.

The tool does not analyze Rust source, measure performance or binary size, determine which features
are active, prove compiler compatibility, audit dependency safety, or establish runtime
reachability. A feature named `full` is only a name. An unsafe-code lint covers only its configured
scope. Lockfile entries are inventory, not proof that a package is used.

## Catalogue

The pipeline is: normalized facts → matching rules → combinations → topic selection → rendering.

The hand-authored [catalogue](../src/catalogue.rs) is two tables: rules, each turning one kind of
observation into an article, and combinations, which replace their component articles with a
sharper one. A manifesto has at most six articles, one per topic, chosen by priority. Sparse
projects get fewer.

- Test minimalism needs at most two normal and at least four development declarations.
- The `full` combination needs an empty default list and at least five feature entries, including
  `default` and `full`.
- Counts are declarations, including optional and target-specific ones, not resolved packages.
- A locally declared version argues for autonomy only in a workspace with more than one member.
- Output is deterministic: no timestamps or randomness, and reordering TOML or editing descriptive
  metadata does not change the articles.

## Rendering

- Members are ranked by prominence: libraries first, then by how many selected members depend on
  them through a path, then by name. The same ranking breaks ties when several members could
  carry the same article.
- Up to three members are named in the title. Larger selections name the most prominent member
  and count the rest as libraries, binaries, and other packages.
- Articles carry no evidence by default. `--explain` follows each article with a plain statement
  of why it was chosen, naming the member it applies to when several are selected, and lists the
  observations it rests on with the file each was read from. More than five observations show
  the first five and count the rest; more than five allowed lints are counted per file.

## Amendments

Amendments compare semantic facts, not generated text, and reinterpret both reports with the
current catalogue, so wording changes never show up as amendments. They cover:

- adding or removing a normal dependency (adding one to a project that already has some extends the
  existing commitment rather than reversing a doctrine of independence);
- any observation switching sides of an opposing pair: an exact or compatible requirement, release
  `panic` abort or unwind, a shared or local version, the current or an older edition.

Unrelated edits produce no amendment. Incompatible schemas or selections are rejected with an
error explaining how to regenerate the baseline.
