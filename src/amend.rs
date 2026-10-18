use crate::{
    catalogue::{self, Report},
    project::{Facts, Observation, Scope},
    render::escape,
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Serialize)]
pub struct Amendments {
    schema_version: u32,
    catalogue_version: u32,
    scope: Scope,
    changes: Vec<Change>,
}
#[derive(Debug, Serialize)]
struct Change {
    kind: String,
    subject: String,
    previous: Option<Observation>,
    former_doctrine: String,
    current: Option<Observation>,
    revised_doctrine: String,
    continuity: String,
}
pub fn read(path: &Path) -> Result<Report> {
    let value: Value =
        serde_json::from_slice(&fs::read(path).context("cannot read baseline JSON")?)
            .context("invalid baseline JSON")?;
    if value["schema_version"] != catalogue::SCHEMA {
        bail!(
            "incompatible baseline schema; regenerate the baseline with this version of cargo-manifesto"
        );
    }
    let report: Report =
        serde_json::from_value(value).context("invalid baseline report structure")?;
    if report.scope != report.facts.scope {
        bail!("baseline scope and facts disagree; regenerate the baseline");
    }
    let ids: std::collections::BTreeSet<_> =
        report.facts.observations.iter().map(|o| &o.id).collect();
    if ids.len() != report.facts.observations.len() {
        bail!("baseline contains duplicate observation identities");
    }
    Ok(report)
}
fn dependencies(facts: &Facts) -> BTreeMap<String, &Observation> {
    facts
        .observations
        .iter()
        .filter(|o| o.kind == "dependency")
        .map(|o| (o.id.clone(), o))
        .collect()
}
fn dependency_doctrine(facts: &Facts, scope: &str) -> String {
    let scoped = Facts {
        scope: facts.scope.clone(),
        observations: facts
            .observations
            .iter()
            .filter(|o| o.scope == scope)
            .cloned()
            .collect(),
    };
    let independent = scoped
        .observations
        .iter()
        .any(|o| o.kind == "dependency-counts" && o.value["all"] == 0);
    catalogue::doctrine(
        &scoped,
        if independent {
            "dependencies.independence"
        } else {
            "dependencies.cooperation"
        },
    )
}
fn req_doctrine(facts: &Facts, obs: &Observation) -> String {
    catalogue::doctrine(
        facts,
        if catalogue::exact(&obs.value["requirement"]) {
            "versions.exact"
        } else {
            "versions.compatible"
        },
    )
}
fn continuity() -> String {
    "The implementation has changed. Our principles have been clarified. These principles have guided this project from the beginning.".into()
}

pub fn compare(before: &Report, now: &Report) -> Result<Amendments> {
    if before.schema_version != catalogue::SCHEMA {
        bail!("incompatible baseline schema; regenerate the baseline");
    }
    if before.scope != now.scope {
        bail!(
            "incompatible project scopes; use the same manifest and package selection as the baseline (package versions may differ)"
        );
    }
    let old = dependencies(&before.facts);
    let new = dependencies(&now.facts);
    let mut changes = Vec::new();
    for id in old
        .keys()
        .chain(new.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        let previous = old.get(id).copied();
        let current = new.get(id).copied();
        match (previous, current) {
            (Some(a), Some(b))
                if (catalogue::exact(&a.value["requirement"])
                    && catalogue::compatible(&b.value["requirement"]))
                    || (catalogue::compatible(&a.value["requirement"])
                        && catalogue::exact(&b.value["requirement"])) =>
            {
                changes.push(Change {
                    kind: "version-policy".into(),
                    subject: subject(b),
                    previous: Some(a.clone()),
                    former_doctrine: req_doctrine(&before.facts, a),
                    current: Some(b.clone()),
                    revised_doctrine: req_doctrine(&now.facts, b),
                    continuity: continuity(),
                });
            }
            (None, Some(o)) | (Some(o), None) if o.value["kind"] == "normal" => {
                let added = previous.is_none();
                let old_count = before
                    .facts
                    .observations
                    .iter()
                    .find(|x| x.kind == "dependency-counts" && x.scope == o.scope)
                    .map(|o| o.value["normal"].clone())
                    .unwrap_or(Value::Null);
                let mut revised = dependency_doctrine(&now.facts, &o.scope);
                if added && old_count.as_u64().is_some_and(|n| n > 0) {
                    revised = format!(
                        "Our existing commitment to cooperation now extends to another normal dependency declaration. {revised}"
                    );
                }
                changes.push(Change {
                    kind: if added {
                        "dependency-added"
                    } else {
                        "dependency-removed"
                    }
                    .into(),
                    subject: subject(o),
                    previous: previous.cloned(),
                    former_doctrine: dependency_doctrine(&before.facts, &o.scope),
                    current: current.cloned(),
                    revised_doctrine: revised,
                    continuity: continuity(),
                });
            }
            _ => {}
        }
    }
    let profile = |facts: &Facts| {
        facts
            .observations
            .iter()
            .find(|o| o.kind == "profile")
            .cloned()
    };
    if let (Some(a), Some(b)) = (profile(&before.facts), profile(&now.facts)) {
        let old = &a.value["release"]["panic"];
        let new = &b.value["release"]["panic"];
        if old != new
            && ["abort", "unwind"].iter().any(|x| old == x)
            && ["abort", "unwind"].iter().any(|x| new == x)
        {
            changes.push(Change {
                kind: "panic-policy".into(),
                subject: "workspace release panic strategy".into(),
                former_doctrine: catalogue::doctrine(
                    &before.facts,
                    if old == "abort" {
                        "panic.abort"
                    } else {
                        "panic.unwind"
                    },
                ),
                revised_doctrine: catalogue::doctrine(
                    &now.facts,
                    if new == "abort" {
                        "panic.abort"
                    } else {
                        "panic.unwind"
                    },
                ),
                previous: Some(a),
                current: Some(b),
                continuity: continuity(),
            });
        }
    }
    Ok(Amendments {
        schema_version: catalogue::SCHEMA,
        catalogue_version: catalogue::CATALOGUE,
        scope: now.scope.clone(),
        changes,
    })
}
fn subject(o: &Observation) -> String {
    format!(
        "{}: {} (alias {}, {} declaration, target {})",
        o.scope,
        o.value["package"].as_str().unwrap_or("unknown"),
        o.value["alias"].as_str().unwrap_or("unknown"),
        o.value["kind"].as_str().unwrap_or("unknown"),
        o.value["target"]
    )
}
fn observation(o: Option<&Observation>) -> String {
    match o {
        None => "No such declaration in this scope.".into(),
        Some(o) => format!(
            "{}; evidence: {}",
            escape(&crate::render::describe(o)),
            o.evidence
                .iter()
                .map(|e| format!(
                    "{} → {} = {}",
                    escape(&e.path),
                    escape(&e.field),
                    escape(&e.value.to_string())
                ))
                .collect::<Vec<_>>()
                .join("; ")
        ),
    }
}
pub fn markdown(amendments: &Amendments) -> String {
    let mut out = String::from("# Philosophical Amendments\n\n");
    if amendments.changes.is_empty() {
        out.push_str("No relevant implementation choices have changed. Our timeless principles require no clarification.\n");
    }
    for (i, c) in amendments.changes.iter().enumerate() {
        out.push_str(&format!("## Amendment {}: Our Continuing Commitment to Responsibility\n\nSubject: {}.\n\nPrevious observation: {}\n\nPreviously, we held:\n\n{}\n\nCurrent observation: {}\n\nWe now hold:\n\n{}\n\n{}\n\n",i+1,escape(&c.subject),observation(c.previous.as_ref()),c.former_doctrine,observation(c.current.as_ref()),c.revised_doctrine,c.continuity));
    }
    crate::render::wrap(&out)
}
