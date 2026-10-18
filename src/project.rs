//! Cargo is the authority on what the project is. The raw manifest is consulted only for what its
//! authors deliberately wrote down, the sole form of conviction this institution recognizes.
use crate::Cli;
use anyhow::{Context, Result, bail};
use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// Reserved scope: `@` cannot occur in a Cargo package name.
pub const WORKSPACE_SCOPE: &str = "@workspace";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Scope {
    pub packages: Vec<Identity>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identity {
    pub name: String,
    pub manifest: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Evidence {
    pub path: String,
    pub field: String,
    pub value: Value,
}
/// One authored fact. Settings use their TOML path as `kind` and store `{value, inherited}`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Observation {
    pub id: String,
    pub scope: String,
    pub kind: String,
    pub value: Value,
    pub evidence: Vec<Evidence>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Facts {
    pub scope: Scope,
    /// Sorted by `id`.
    pub observations: Vec<Observation>,
}

impl Observation {
    /// The authored value of a setting observation of this kind.
    pub fn setting(&self, kind: &str) -> Option<&Value> {
        (self.kind == kind).then_some(&self.value["value"])
    }
    /// The named thing an article may credit: a dependency, a binary, a locked package.
    pub fn subject(&self) -> Option<&str> {
        let v = &self.value;
        v["alias"]
            .as_str()
            .or(v["name"].as_str())
            .or(v["package"].as_str())
    }
}

impl Facts {
    pub fn get(&self, id: &str) -> Option<&Observation> {
        let i = self
            .observations
            .binary_search_by(|o| o.id.as_str().cmp(id))
            .ok()?;
        Some(&self.observations[i])
    }
    pub fn find(&self, kind: &str, scope: &str) -> Option<&Observation> {
        self.observations
            .iter()
            .find(|o| o.kind == kind && o.scope == scope)
    }
}

pub struct Loaded {
    pub facts: Facts,
    pub document: PathBuf,
}

pub fn load(cli: &Cli) -> Result<Loaded> {
    let manifest = discover(cli.manifest_path.as_deref())?;
    // `--no-deps` is what keeps resolution, lockfile writes, and build scripts out of this.
    // `--offline` only ensures nobody goes looking. `exec` honours `$CARGO`.
    let metadata = MetadataCommand::new()
        .manifest_path(&manifest)
        .no_deps()
        .other_options(vec!["--offline".into()])
        .env("RUSTUP_AUTO_INSTALL", "0")
        .exec()
        .context("Cargo metadata failed")?;
    let root_manifest = metadata.workspace_root.join("Cargo.toml");
    let root_raw = read_toml(root_manifest.as_std_path())?;
    // Canonical so /var and /private/var are recognized as the same institution.
    let root = fs::canonicalize(&metadata.workspace_root)?;
    let selected = select(&metadata, &manifest, cli)?;
    let mut facts = Facts {
        scope: Scope {
            packages: Vec::new(),
        },
        observations: Vec::new(),
    };
    let mut editions = BTreeMap::new();
    for package in &selected {
        let path = package.manifest_path.as_std_path();
        let manifest = relative(path, &root);
        let own;
        let raw = if package.manifest_path == root_manifest {
            &root_raw
        } else {
            own = read_toml(path)?;
            &own
        };
        let mut out = Recorder {
            facts: &mut facts,
            scope: &package.name,
            path: &manifest,
        };
        package_facts(&mut out, package, raw, &root_raw, &root)?;
        editions.insert(package.name.to_string(), package.edition.to_string());
        facts.scope.packages.push(Identity {
            name: package.name.to_string(),
            manifest,
        });
    }
    root_facts(&mut facts, &root_raw, &metadata, &editions);
    lock_facts(&mut facts, &root)?;
    for o in &mut facts.observations {
        normalize_paths(&mut o.value, &root);
        for evidence in &mut o.evidence {
            normalize_paths(&mut evidence.value, &root);
        }
    }
    facts.scope.packages.sort();
    facts.observations.sort_by(|a, b| a.id.cmp(&b.id));
    let document = document_path(&metadata, &selected)?;
    Ok(Loaded { facts, document })
}

/// The managed document: `[package.metadata.manifesto] file` for one selected package, or
/// `[workspace.metadata.manifesto] file` for several, else `MANIFESTO.md` beside the selection.
fn document_path(metadata: &Metadata, selected: &[&Package]) -> Result<PathBuf> {
    let (table, base, source) = match selected {
        [package] => (
            &package.metadata,
            package.manifest_path.parent().unwrap().as_std_path(),
            "package",
        ),
        _ => (
            &metadata.workspace_metadata,
            metadata.workspace_root.as_std_path(),
            "workspace",
        ),
    };
    match &table["manifesto"]["file"] {
        Value::Null => Ok(base.join("MANIFESTO.md")),
        Value::String(file) => Ok(base.join(file)),
        _ => bail!("{source}.metadata.manifesto.file must be a string path"),
    }
}

fn discover(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        if path.file_name().is_none_or(|n| n != "Cargo.toml") {
            bail!("--manifest-path must name a Cargo.toml file");
        }
        return fs::canonicalize(path).context("cannot open manifest path");
    }
    let cwd = std::env::current_dir()?;
    let found = cwd
        .ancestors()
        .map(|dir| dir.join("Cargo.toml"))
        .find(|p| p.is_file());
    Ok(fs::canonicalize(
        found.context("no Cargo.toml found; use --manifest-path")?,
    )?)
}

fn select<'a>(metadata: &'a Metadata, manifest: &Path, cli: &Cli) -> Result<Vec<&'a Package>> {
    let members = metadata.workspace_packages();
    let selected: Vec<_> = if let Some(name) = &cli.package {
        let matches: Vec<_> = members
            .into_iter()
            .filter(|p| p.name.as_str() == name)
            .collect();
        if matches.len() != 1 {
            bail!("package {name:?} must identify exactly one workspace member");
        }
        matches
    } else if cli.workspace {
        members
    } else if manifest == metadata.workspace_root.join("Cargo.toml").as_std_path() {
        metadata.workspace_default_packages()
    } else {
        members
            .into_iter()
            .filter(|p| p.manifest_path.as_std_path() == manifest)
            .collect()
    };
    if selected.is_empty() {
        bail!("selection contains no workspace members; use -p or --workspace");
    }
    Ok(selected)
}

fn read_toml(path: &Path) -> Result<Value> {
    let text =
        fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("invalid TOML in {}", path.display()))?;
    Ok(serde_json::to_value(value)?)
}

/// A `/`-separated path from the canonical `root`, using `..` rather than ever going absolute.
fn relative(path: &Path, root: &Path) -> String {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let a: Vec<_> = root.components().collect();
    let b: Vec<_> = path.components().collect();
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut result = PathBuf::new();
    for _ in common..a.len() {
        result.push("..");
    }
    for part in &b[common..] {
        result.push(part);
    }
    result.to_string_lossy().replace('\\', "/")
}

fn ev(path: &str, field: &str, value: Value) -> Evidence {
    Evidence {
        path: path.into(),
        field: field.into(),
        value,
    }
}

/// Records observations for one scope, citing one manifest.
struct Recorder<'a> {
    facts: &'a mut Facts,
    scope: &'a str,
    path: &'a str,
}
impl Recorder<'_> {
    fn cite(&self, field: &str, value: &Value) -> Evidence {
        ev(self.path, field, value.clone())
    }
    fn observe(&mut self, kind: &str, key: Option<&str>, value: Value, evidence: Vec<Evidence>) {
        let id = match key {
            Some(key) => format!("{}/{kind}/{key}", self.scope),
            None => format!("{}/{kind}", self.scope),
        };
        let (scope, kind) = (self.scope.into(), kind.into());
        let observation = Observation {
            id,
            scope,
            kind,
            value,
            evidence,
        };
        self.facts.observations.push(observation);
    }
    /// A setting under its TOML path, possibly inherited from the workspace.
    fn setting(&mut self, kind: &str, value: &Value, inherited: bool, evidence: Vec<Evidence>) {
        let value = json!({"value": value, "inherited": inherited});
        self.observe(kind, None, value, evidence);
    }
    /// A setting written directly at `kind`, if present.
    fn authored(&mut self, kind: &str, value: Option<&Value>) {
        if let Some(value) = value {
            let evidence = vec![self.cite(kind, value)];
            self.setting(kind, value, false, evidence);
        }
    }
    /// Follow `{ workspace = true }` to the workspace's `shared_field`, citing both.
    fn inherit<'v>(
        &self,
        field: &str,
        authored: &'v Value,
        (shared_field, shared): (&str, &'v Value),
    ) -> (&'v Value, bool, Vec<Evidence>) {
        let mut evidence = vec![self.cite(field, authored)];
        if authored["workspace"] != true {
            return (authored, false, evidence);
        }
        evidence.push(ev("Cargo.toml", shared_field, shared.clone()));
        (shared, true, evidence)
    }
}

fn package_facts(
    out: &mut Recorder,
    package: &Package,
    raw: &Value,
    root: &Value,
    workspace: &Path,
) -> Result<()> {
    let mut counts = BTreeMap::from([("normal", 0), ("dev", 0), ("build", 0)]);
    let mut declared = Vec::new();
    for dep in &package.dependencies {
        let alias = dep.rename.as_deref().unwrap_or(&dep.name);
        let (kind, table) = match dep.kind {
            DependencyKind::Development => ("dev", "dev-dependencies"),
            DependencyKind::Build => ("build", "build-dependencies"),
            _ => ("normal", "dependencies"),
        };
        let (section, field) = match &dep.target {
            None => (raw, format!("{table}.{alias}")),
            Some(platform) => {
                let targets = raw["target"].as_object().into_iter().flatten();
                let (name, section) = targets
                    .clone()
                    .find(|(name, _)| name.parse().ok().as_ref() == Some(platform))
                    .context("Cargo target declaration missing from authored manifest")?;
                (section, format!("target.{name}.{table}.{alias}"))
            }
        };
        let shared = (
            &*format!("workspace.dependencies.{alias}"),
            &root["workspace"]["dependencies"][alias],
        );
        let (effective, inherited, evidence) = out.inherit(&field, &section[table][alias], shared);
        let source = if let Some(git) = effective["git"].as_str() {
            let e = effective;
            json!({
                "kind": "git",
                "url": git,
                "rev": e["rev"],
                "branch": e["branch"],
                "tag": e["tag"],
            })
        } else if let Some(local) = &dep.path {
            json!({"kind": "path", "path": relative(local.as_std_path(), workspace)})
        } else {
            json!({"kind": "registry", "registry": effective["registry"]})
        };
        let value = json!({
            "package": dep.name,
            "alias": alias,
            "kind": kind,
            "target": dep.target.as_ref().map(ToString::to_string),
            "optional": dep.optional,
            "requirement": effective.as_str().or(effective["version"].as_str()),
            "source": source,
            "default_features": dep.uses_default_features,
            "features": dep.features,
            "inherited": inherited,
        });
        *counts.get_mut(kind).unwrap() += 1;
        declared.push(alias.to_string());
        let key = format!("{}/{alias}/{kind}/{}", dep.name, value["target"]);
        out.observe("dependency", Some(&key), value, evidence);
    }
    let all = declared.len();
    let value = json!({
        "all": all,
        "normal": counts["normal"],
        "dev": counts["dev"],
        "build": counts["build"],
    });
    let tables = "dependencies; dev-dependencies; build-dependencies; target";
    let evidence = vec![out.cite(tables, &json!(declared))];
    out.observe("dependency-counts", None, value, evidence);
    for field in [
        "version",
        "edition",
        "rust-version",
        "publish",
        "readme",
        "include",
    ] {
        if let Some(authored) = raw["package"].get(field) {
            let kind = format!("package.{field}");
            let shared = (
                &*format!("workspace.{kind}"),
                &root["workspace"]["package"][field],
            );
            let (value, inherited, evidence) = out.inherit(&kind, authored, shared);
            out.setting(&kind, value, inherited, evidence);
        }
    }
    out.authored("features", raw.get("features"));
    let docs = "package.metadata.docs.rs.all-features";
    out.authored(docs, raw.pointer("/package/metadata/docs/rs/all-features"));
    lint_facts(out, raw, root);
    let kinds: Vec<_> = package
        .targets
        .iter()
        .flat_map(|t| t.kind.iter().map(ToString::to_string))
        .collect();
    let evidence = vec![out.cite("targets (Cargo metadata)", &json!(kinds))];
    out.observe("targets", None, json!(kinds), evidence);
    for bin in package.targets.iter().filter(|t| t.is_bin()) {
        let evidence = vec![out.cite("binary target (Cargo metadata)", &json!(bin.name))];
        out.observe(
            "binary",
            Some(&bin.name),
            json!({"name": bin.name}),
            evidence,
        );
    }
    Ok(())
}

/// One observation per configured lint, recorded by level alone; priorities are internal affairs.
fn lint_facts(out: &mut Recorder, raw: &Value, root: &Value) {
    let (table, inherited, _) = out.inherit(
        "lints",
        &raw["lints"],
        ("workspace.lints", &root["workspace"]["lints"]),
    );
    for (tool, lints) in table.as_object().into_iter().flatten() {
        for (name, value) in lints.as_object().into_iter().flatten() {
            let kind = format!("lints.{tool}.{name}");
            let level = value.as_str().or(value["level"].as_str()).map(Value::from);
            let evidence = if inherited {
                let shared = ev("Cargo.toml", &format!("workspace.{kind}"), value.clone());
                vec![out.cite("lints.workspace", &json!(true)), shared]
            } else {
                vec![out.cite(&kind, value)]
            };
            out.setting(&kind, &level.unwrap_or_default(), inherited, evidence);
        }
    }
}

fn root_facts(
    facts: &mut Facts,
    raw: &Value,
    metadata: &Metadata,
    editions: &BTreeMap<String, String>,
) {
    let mut out = Recorder {
        facts,
        scope: WORKSPACE_SCOPE,
        path: "Cargo.toml",
    };
    // Only the root's profiles count; Cargo ignores members', and so do we.
    for (profile, settings) in raw["profile"].as_object().into_iter().flatten() {
        for (key, value) in settings.as_object().into_iter().flatten() {
            out.authored(&format!("profile.{profile}.{key}"), Some(value));
        }
    }
    out.authored("patch", raw.get("patch"));
    out.authored("workspace.exclude", raw.pointer("/workspace/exclude"));
    let names = |packages: Vec<&Package>| {
        let mut names: Vec<_> = packages.iter().map(|p| p.name.to_string()).collect();
        names.sort();
        names
    };
    let membership = json!({
        "members": names(metadata.workspace_packages()),
        "default_members": names(metadata.workspace_default_packages()),
    });
    let evidence = vec![out.cite("workspace membership (Cargo metadata)", &membership)];
    out.observe("membership", None, membership, evidence);
    let evidence = vec![out.cite("package.edition (Cargo metadata)", &json!(editions))];
    out.observe("editions", None, json!(editions), evidence);
}

/// An existing lockfile is an inventory. A missing one is unavailable, which is not the same
/// as owning nothing.
fn lock_facts(facts: &mut Facts, root: &Path) -> Result<()> {
    let mut out = Recorder {
        facts,
        scope: WORKSPACE_SCOPE,
        path: "Cargo.lock",
    };
    let text = match fs::read_to_string(root.join("Cargo.lock")) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            out.observe("lockfile", None, json!({"available": false}), Vec::new());
            return Ok(());
        }
        Err(e) => return Err(e).context("cannot read Cargo.lock"),
    };
    let raw: toml::Value = toml::from_str(&text).context("invalid existing Cargo.lock")?;
    let entries = raw.get("package").and_then(toml::Value::as_array);
    let mut packages = Vec::new();
    let mut external: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for entry in entries.into_iter().flatten() {
        let field = |key: &str| entry.get(key).and_then(toml::Value::as_str);
        let name = field("name").context("lockfile package missing name")?;
        let version = field("version").context("lockfile package missing version")?;
        packages.push(json!({"name": name, "version": version, "source": field("source")}));
        if field("source").is_some() {
            external.entry(name).or_default().push(version);
        }
    }
    packages.sort_by_key(Value::to_string);
    let evidence = vec![out.cite("package", &json!(packages.len()))];
    out.observe(
        "lockfile",
        None,
        json!({"available": true, "packages": packages}),
        evidence,
    );
    for (name, mut versions) in external {
        versions.sort_unstable();
        versions.dedup();
        if versions.len() > 1 {
            let value = json!({"package": name, "versions": versions});
            let evidence = vec![out.cite("package", &value)];
            out.observe("lockfile.multiple", Some(name), value, evidence);
        }
    }
    Ok(())
}

/// Report every absolute path relative to the workspace. Where we live is nobody's business.
fn normalize_paths(value: &mut Value, root: &Path) {
    match value {
        Value::Object(fields) => fields.values_mut().for_each(|v| normalize_paths(v, root)),
        Value::Array(values) => values.iter_mut().for_each(|v| normalize_paths(v, root)),
        Value::String(s) if Path::new(s.as_str()).is_absolute() => {
            *s = relative(Path::new(s.as_str()), root);
        }
        _ => {}
    }
}
