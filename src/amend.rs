//! Amendments. When a decision reverses, the record shows how both positions arose from the same
//! enduring conviction. Facts are compared, never prose, so improved wording is not mistaken for
//! a change of heart.
use crate::{
    catalogue::{self, Report},
    project::{Facts, Observation, Scope, WORKSPACE_SCOPE},
    render::{describe, escape, wrap},
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::{collections::BTreeSet, fmt::Write, fs, path::Path};

#[derive(Debug, Serialize)]
pub struct Amendments {
    schema_version: u32,
    catalogue_version: u32,
    scope: Scope,
    changes: Vec<Change>,
}
#[derive(Debug, Serialize)]
struct Change {
    kind: &'static str,
    title: &'static str,
    subject: String,
    previous: Option<Observation>,
    former_doctrine: String,
    current: Option<Observation>,
    revised_doctrine: String,
    continuity: String,
}

/// Opposing rules that the same observation can switch between.
struct Reversal {
    kind: &'static str,
    title: &'static str,
    subject: &'static str,
    rules: &'static [&'static str],
}
const REVERSALS: &[Reversal] = &[
    Reversal {
        kind: "version-policy",
        title: "Our Continuing Commitment to Trust",
        subject: "version requirement",
        rules: &["versions.exact", "versions.compatible"],
    },
    Reversal {
        kind: "panic-policy",
        title: "Our Continuing Commitment to Responsible Failure",
        subject: "panic strategy",
        rules: &["panic.abort", "panic.unwind"],
    },
    Reversal {
        kind: "version-identity",
        title: "Our Continuing Commitment to Identity",
        subject: "version declaration",
        rules: &["versions.shared", "versions.independent"],
    },
    Reversal {
        kind: "edition",
        title: "Our Continuing Commitment to the Present",
        subject: "edition",
        rules: &["edition.current", "edition.settled"],
    },
];
const DEPENDENCY_RULES: &[&str] = &["dependencies.independence", "dependencies.cooperation"];

pub fn read(path: &Path) -> Result<Report> {
    let bytes = fs::read(path).context("cannot read baseline JSON")?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).context("invalid baseline JSON")?;
    if value["schema_version"] != catalogue::SCHEMA {
        bail!(
            "incompatible baseline schema; regenerate the baseline with this version of \
                cargo-manifesto"
        );
    }
    let mut report: Report =
        serde_json::from_value(value).context("invalid baseline report structure")?;
    report.facts.observations.sort_by(|a, b| a.id.cmp(&b.id));
    let ids: BTreeSet<_> = report.facts.observations.iter().map(|o| &o.id).collect();
    if ids.len() != report.facts.observations.len() {
        bail!("baseline contains duplicate observation identities");
    }
    Ok(report)
}

pub fn compare(before: &Report, now: &Report) -> Result<Amendments> {
    let (old, new) = (&before.facts, &now.facts);
    if old.scope != new.scope {
        bail!(
            "incompatible project scopes; use the same manifest and package selection as the \
                baseline (package versions may differ)"
        );
    }
    let ids: BTreeSet<_> = old
        .observations
        .iter()
        .chain(&new.observations)
        .map(|o| o.id.as_str())
        .collect();
    let mut changes = Vec::new();
    for id in &ids {
        if let Some(change) = dependency_change(old, new, id) {
            changes.push(change);
        }
    }
    for reversal in REVERSALS {
        for id in &ids {
            let (Some(a), Some(b)) = (old.get(id), new.get(id)) else {
                continue;
            };
            let (Some((was, former)), Some((is, revised))) = (
                catalogue::argue(reversal.rules, a, old),
                catalogue::argue(reversal.rules, b, new),
            ) else {
                continue;
            };
            if was != is {
                changes.push(Change {
                    kind: reversal.kind,
                    title: reversal.title,
                    subject: subject(b),
                    previous: Some(a.clone()),
                    former_doctrine: former,
                    current: Some(b.clone()),
                    revised_doctrine: revised,
                    continuity: continuity(reversal.subject),
                });
            }
        }
    }
    Ok(Amendments {
        schema_version: catalogue::SCHEMA,
        catalogue_version: catalogue::CATALOGUE,
        scope: new.scope.clone(),
        changes,
    })
}

/// A normal dependency arrived or left. Arriving where others already live extends the existing
/// commitment rather than reversing a doctrine of independence nobody held.
fn dependency_change(old: &Facts, new: &Facts, id: &str) -> Option<Change> {
    let (previous, current) = (old.get(id), new.get(id));
    let o = match (previous, current) {
        (None, Some(o)) | (Some(o), None)
            if o.kind == "dependency" && o.value["kind"] == "normal" =>
        {
            o
        }
        _ => return None,
    };
    let doctrine = |facts: &Facts| {
        let counts = facts.find("dependency-counts", &o.scope)?;
        catalogue::argue(DEPENDENCY_RULES, counts, facts).map(|(_, text)| text)
    };
    let mut revised = doctrine(new)?;
    let had_company = old
        .find("dependency-counts", &o.scope)
        .and_then(|c| c.value["normal"].as_u64())
        > Some(0);
    let expanded = previous.is_none() && had_company;
    if expanded {
        let name = escape(o.subject()?);
        revised = format!(
            "Our existing commitment to cooperation now extends to {name}. It always would have, \
                had {name} been here sooner. {revised}"
        );
    }
    Some(Change {
        kind: if previous.is_none() {
            "dependency-added"
        } else {
            "dependency-removed"
        },
        title: if expanded {
            "Our Growing Commitment to Cooperation"
        } else {
            "Our Continuing Commitment to Responsibility"
        },
        subject: subject(o),
        previous: previous.cloned(),
        former_doctrine: doctrine(old)?,
        current: current.cloned(),
        revised_doctrine: revised,
        continuity: continuity("dependency declaration"),
    })
}

fn continuity(subject: &str) -> String {
    format!(
        "The {subject} has changed. Our principles have only been clarified. These principles have \
            guided this project from the beginning."
    )
}

fn subject(o: &Observation) -> String {
    let scope = if o.scope == WORKSPACE_SCOPE {
        "workspace root"
    } else {
        &o.scope
    };
    format!("{scope}: {}", describe(o))
}

fn observation(o: Option<&Observation>) -> String {
    let Some(o) = o else {
        return "None.".into();
    };
    let sources: Vec<_> = o
        .evidence
        .iter()
        .map(|e| format!("{} → {} = {}", e.path, e.field, e.value))
        .collect();
    escape(&format!(
        "{}; evidence: {}",
        describe(o),
        sources.join("; ")
    ))
}

pub fn markdown(amendments: &Amendments) -> String {
    let mut out = String::from("# Philosophical Amendments\n\n");
    if amendments.changes.is_empty() {
        out.push_str(
            "We have reviewed the changes and found our principles exactly where we left \
            them, which is where they have always been.\n",
        );
    }
    for (i, c) in amendments.changes.iter().enumerate() {
        let _ = write!(
            out,
            "## Amendment {}: {}\n\n\
             Subject: {}.\n\n\
             Previous observation: {}\n\n\
             Previously, we held:\n\n{}\n\n\
             Current observation: {}\n\n\
             We now hold:\n\n{}\n\n{}\n\n",
            i + 1,
            c.title,
            escape(&c.subject),
            observation(c.previous.as_ref()),
            c.former_doctrine,
            observation(c.current.as_ref()),
            c.revised_doctrine,
            c.continuity,
        );
    }
    wrap(&out)
}
