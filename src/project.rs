//! Cargo owns project discovery; raw TOML supplies only authored policy and provenance.
use crate::Cli;
use anyhow::{Context, Result, bail};
use cargo_metadata::{Metadata, MetadataCommand, Package};
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
    pub observations: Vec<Observation>,
}

pub struct Loaded {
    pub facts: Facts,
    pub document: PathBuf,
}

pub fn load(cli: &Cli) -> Result<Loaded> {
    let manifest = discover(cli.manifest_path.as_deref())?;
    // no-deps avoids resolution entirely. Offline is an additional network guard, not
    // the reason this invocation does not write a lockfile or run a build script.
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = MetadataCommand::new();
    command.cargo_path(cargo).manifest_path(&manifest).no_deps();
    command.other_options(vec!["--offline".into()]);
    let mut process = command.cargo_command();
    process.env("RUSTUP_AUTO_INSTALL", "0");
    let output = process.output().context("cannot run Cargo metadata")?;
    if !output.status.success() {
        bail!(
            "Cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let metadata: Metadata =
        serde_json::from_slice(&output.stdout).context("invalid Cargo metadata")?;
    let root = metadata.workspace_root.as_std_path();
    let root_manifest = root.join("Cargo.toml");
    let root_raw = read_toml(&root_manifest)?;
    let selected = select(&metadata, &manifest, cli)?;
    let document = document_path(&metadata, &selected)?;
    let mut facts = Facts {
        scope: Scope {
            packages: Vec::new(),
        },
        observations: Vec::new(),
    };
    for package in selected {
        let path = package.manifest_path.as_std_path();
        let relative = relative(path, root);
        facts.scope.packages.push(Identity {
            name: package.name.to_string(),
            manifest: relative.clone(),
        });
        let raw = read_toml(path)?;
        package_facts(&mut facts, package, &raw, &root_raw, &relative, root)?;
    }
    root_facts(&mut facts, &root_raw, &metadata);
    lock_facts(&mut facts, root)?;
    for observation in &mut facts.observations {
        normalize_local_paths(&mut observation.value, root);
        if observation.kind == "package.readme" {
            normalize_path_string(&mut observation.value["value"], root);
        }
        for evidence in &mut observation.evidence {
            normalize_local_paths(&mut evidence.value, root);
            if evidence.field.ends_with("package.readme") {
                normalize_path_string(&mut evidence.value, root);
            }
        }
    }
    facts.scope.packages.sort();
    facts.observations.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Loaded { facts, document })
}

/// The managed document: `[package.metadata.manifesto] file` for one selected package, or
/// `[workspace.metadata.manifesto] file` for several, else `MANIFESTO.md` beside the selection.
fn document_path(metadata: &Metadata, selected: &[&Package]) -> Result<PathBuf> {
    let root = metadata.workspace_root.as_std_path();
    let configured =
        |table: &serde_json::Value, base: &Path, source: &str| -> Result<Option<PathBuf>> {
            match table.get("manifesto").and_then(|m| m.get("file")) {
                None => Ok(None),
                Some(Value::String(file)) => Ok(Some(base.join(file))),
                Some(_) => bail!("{source}.metadata.manifesto.file must be a string path"),
            }
        };
    if let [package] = selected {
        let dir = package.manifest_path.parent().unwrap().as_std_path();
        return Ok(configured(&package.metadata, dir, "package")?
            .unwrap_or_else(|| dir.join("MANIFESTO.md")));
    }
    Ok(configured(&metadata.workspace_metadata, root, "workspace")?
        .unwrap_or_else(|| root.join("MANIFESTO.md")))
}

fn discover(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        if path.file_name().is_none_or(|n| n != "Cargo.toml") {
            bail!("--manifest-path must name a Cargo.toml file");
        }
        return fs::canonicalize(path).context("cannot open manifest path");
    }
    let cwd = std::env::current_dir()?;
    for parent in cwd.ancestors() {
        let path = parent.join("Cargo.toml");
        if path.is_file() {
            return Ok(fs::canonicalize(path)?);
        }
    }
    bail!("no Cargo.toml found; use --manifest-path")
}

fn select<'a>(metadata: &'a Metadata, manifest: &Path, cli: &Cli) -> Result<Vec<&'a Package>> {
    let members: Vec<_> = metadata
        .packages
        .iter()
        .filter(|p| metadata.workspace_members.contains(&p.id))
        .collect();
    let selected = if let Some(name) = &cli.package {
        let matches: Vec<_> = members
            .iter()
            .copied()
            .filter(|p| p.name.as_str() == name)
            .collect();
        if matches.len() != 1 {
            bail!("package {name:?} must identify exactly one workspace member");
        }
        matches
    } else if cli.workspace {
        members
    } else if manifest == metadata.workspace_root.join("Cargo.toml").as_std_path() {
        members
            .into_iter()
            .filter(|p| metadata.workspace_default_members.contains(&p.id))
            .collect()
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

fn relative(path: &Path, root: &Path) -> String {
    // Resolve existing symlinks so /var and /private/var do not become unrelated roots.
    let canonical_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let canonical_root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let path = canonical_path.as_path();
    let root = canonical_root.as_path();
    // Path dependencies can live outside the workspace. Never emit absolute paths.
    let a: Vec<_> = root.components().collect();
    let b: Vec<_> = path.components().collect();
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut result = PathBuf::new();
    for _ in common..a.len() {
        result.push("..");
    }
    for part in &b[common..] {
        result.push(part.as_os_str());
    }
    result.to_string_lossy().replace('\\', "/")
}

fn add(facts: &mut Facts, scope: &str, kind: &str, value: Value, evidence: Vec<Evidence>) {
    let suffix = if kind == "dependency" {
        format!(
            "/{}/{}/{}/{}",
            value["package"].as_str().unwrap(),
            value["alias"].as_str().unwrap(),
            value["kind"].as_str().unwrap(),
            value["target"]
        )
    } else {
        String::new()
    };
    facts.observations.push(Observation {
        id: format!("{scope}/{kind}{suffix}"),
        scope: scope.into(),
        kind: kind.into(),
        value,
        evidence,
    });
}
fn ev(path: &str, field: &str, value: Value) -> Evidence {
    Evidence {
        path: path.into(),
        field: field.into(),
        value,
    }
}

fn package_facts(
    facts: &mut Facts,
    package: &Package,
    raw: &Value,
    root: &Value,
    path: &str,
    workspace: &Path,
) -> Result<()> {
    let scope = package.name.as_str();
    add(
        facts,
        scope,
        "identity",
        json!({"name":scope,"version":package.version.to_string(),"manifest":path,"edition":package.edition.to_string()}),
        vec![ev(
            path,
            "package",
            json!({"name":scope,"version":package.version.to_string()}),
        )],
    );
    let mut dependencies = Vec::new();
    for dep in &package.dependencies {
        let alias = dep.rename.as_deref().unwrap_or(&dep.name);
        let kind = match dep.kind {
            cargo_metadata::DependencyKind::Development => "dev",
            cargo_metadata::DependencyKind::Build => "build",
            _ => "normal",
        };
        let table = match kind {
            "dev" => "dev-dependencies",
            "build" => "build-dependencies",
            _ => "dependencies",
        };
        let target = dep.target.as_ref().map(ToString::to_string);
        let (authored, field) = if target.is_some() {
            let authored_target = raw["target"].as_object().and_then(|targets| {
                targets
                    .iter()
                    .find(|(name, _)| name.parse().ok().as_ref() == dep.target.as_ref())
            });
            let (name, section) = authored_target
                .context("Cargo target declaration missing from authored manifest")?;
            (
                &section[table][alias],
                format!("target.{name}.{table}.{alias}"),
            )
        } else {
            (&raw[table][alias], format!("{table}.{alias}"))
        };
        let mut evidence = vec![ev(path, &field, authored.clone())];
        let inherited = authored["workspace"] == true;
        let effective = if inherited {
            let base = &root["workspace"]["dependencies"][alias];
            evidence.push(ev(
                "Cargo.toml",
                &format!("workspace.dependencies.{alias}"),
                base.clone(),
            ));
            base
        } else {
            authored
        };
        let req = if effective.is_string() {
            effective.as_str().map(str::to_owned)
        } else {
            effective["version"].as_str().map(str::to_owned)
        };
        let source = if let Some(git) = effective["git"].as_str() {
            json!({"kind":"git","url":git,"rev":effective["rev"],"branch":effective["branch"],"tag":effective["tag"]})
        } else if let Some(local) = &dep.path {
            json!({"kind":"path","path":relative(local.as_std_path(),workspace)})
        } else {
            json!({"kind":"registry","registry":effective["registry"]})
        };
        let value = json!({"package":dep.name,"alias":alias,"kind":kind,"target":target,"optional":dep.optional,"requirement":req,"source":source,"default_features":dep.uses_default_features,"features":dep.features,"inherited":inherited});
        dependencies.push(value.clone());
        add(facts, scope, "dependency", value, evidence);
    }
    add(
        facts,
        scope,
        "dependency-counts",
        json!({"all":dependencies.len(),"normal":dependencies.iter().filter(|d|d["kind"]=="normal").count(),"dev":dependencies.iter().filter(|d|d["kind"]=="dev").count(),"build":dependencies.iter().filter(|d|d["kind"]=="build").count()}),
        vec![ev(
            path,
            "dependencies; dev-dependencies; build-dependencies; target",
            json!(dependencies),
        )],
    );
    for field in [
        "version",
        "edition",
        "rust-version",
        "publish",
        "readme",
        "include",
    ] {
        if let Some(value) = raw["package"].get(field) {
            let inherited = value["workspace"] == true;
            let effective = if inherited {
                &root["workspace"]["package"][field]
            } else {
                value
            };
            let mut evidence = vec![ev(path, &format!("package.{field}"), value.clone())];
            if inherited {
                evidence.push(ev(
                    "Cargo.toml",
                    &format!("workspace.package.{field}"),
                    effective.clone(),
                ));
            }
            add(
                facts,
                scope,
                &format!("package.{field}"),
                json!({"value":effective,"inherited":inherited}),
                evidence,
            );
        }
    }
    for field in ["features", "lints", "package.metadata"] {
        let value = if field == "package.metadata" {
            &raw["package"]["metadata"]
        } else {
            &raw[field]
        };
        if !value.is_null() {
            let inherited = field == "lints" && value["workspace"] == true;
            let effective = if inherited {
                &root["workspace"]["lints"]
            } else {
                value
            };
            let mut evidence = vec![ev(path, field, value.clone())];
            if inherited {
                evidence.push(ev("Cargo.toml", "workspace.lints", effective.clone()));
            }
            add(
                facts,
                scope,
                field,
                json!({"value":effective,"inherited":inherited}),
                evidence,
            );
        }
    }
    let kinds: Vec<_> = package
        .targets
        .iter()
        .flat_map(|t| t.kind.iter().map(ToString::to_string))
        .collect();
    add(
        facts,
        scope,
        "targets",
        json!(kinds),
        vec![ev(path, "targets (Cargo metadata)", json!(kinds))],
    );
    let binaries: Vec<_> = package
        .targets
        .iter()
        .filter(|t| t.is_bin())
        .map(|t| t.name.clone())
        .collect();
    if !binaries.is_empty() {
        add(
            facts,
            scope,
            "binaries",
            json!(binaries),
            vec![ev(path, "binary targets (Cargo metadata)", json!(binaries))],
        );
    }
    Ok(())
}

fn root_facts(facts: &mut Facts, raw: &Value, metadata: &Metadata) {
    for key in [
        "profile",
        "patch",
        "workspace.exclude",
        "workspace.default-members",
    ] {
        let value = key.split('.').fold(raw, |v, k| &v[k]);
        if !value.is_null() {
            add(
                facts,
                WORKSPACE_SCOPE,
                key,
                value.clone(),
                vec![ev("Cargo.toml", key, value.clone())],
            );
        }
    }
    let mut members: Vec<_> = metadata
        .packages
        .iter()
        .filter(|p| metadata.workspace_members.contains(&p.id))
        .map(|p| p.name.to_string())
        .collect();
    members.sort();
    let mut defaults: Vec<_> = metadata
        .packages
        .iter()
        .filter(|p| metadata.workspace_default_members.contains(&p.id))
        .map(|p| p.name.to_string())
        .collect();
    defaults.sort();
    add(
        facts,
        WORKSPACE_SCOPE,
        "membership",
        json!({"members":members,"default_members":defaults}),
        vec![ev(
            "Cargo.toml",
            "workspace membership (Cargo metadata)",
            json!({"members":members,"default_members":defaults}),
        )],
    );
}

fn lock_facts(facts: &mut Facts, root: &Path) -> Result<()> {
    let path = root.join("Cargo.lock");
    match fs::read_to_string(&path) {
        Ok(text) => {
            let raw: toml::Value = toml::from_str(&text).context("invalid existing Cargo.lock")?;
            let mut packages: Vec<Value> = Vec::new();
            let mut versions: BTreeMap<String, Vec<String>> = BTreeMap::new();
            if let Some(entries) = raw.get("package").and_then(toml::Value::as_array) {
                for entry in entries {
                    let name = entry["name"]
                        .as_str()
                        .context("lockfile package missing name")?;
                    let version = entry["version"]
                        .as_str()
                        .context("lockfile package missing version")?;
                    let source = entry.get("source").and_then(toml::Value::as_str);
                    packages.push(json!({"name":name,"version":version,"source":source}));
                    if source.is_some() {
                        versions
                            .entry(name.into())
                            .or_default()
                            .push(version.into());
                    }
                }
            }
            packages.sort_by_key(Value::to_string);
            versions.retain(|_, v| {
                v.sort();
                v.dedup();
                v.len() > 1
            });
            let value =
                json!({"available":true,"packages":packages,"multiple_external_versions":versions});
            add(
                facts,
                WORKSPACE_SCOPE,
                "lockfile",
                value.clone(),
                vec![ev("Cargo.lock", "package", value)],
            );
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => add(
            facts,
            WORKSPACE_SCOPE,
            "lockfile",
            json!({"available":false}),
            Vec::new(),
        ),
        Err(e) => return Err(e).context("cannot read Cargo.lock"),
    }
    Ok(())
}

fn normalize_path_string(value: &mut Value, root: &Path) {
    if let Some(path) = value.as_str().filter(|s| Path::new(s).is_absolute()) {
        *value = Value::String(relative(Path::new(path), root));
    }
}
fn normalize_local_paths(value: &mut Value, root: &Path) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if key == "path" {
                    normalize_path_string(value, root);
                }
                normalize_local_paths(value, root);
            }
        }
        Value::Array(values) => {
            for value in values {
                normalize_local_paths(value, root);
            }
        }
        _ => {}
    }
}
