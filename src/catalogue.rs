//! Hand-authored arguments. Predicates describe observations, never inferred runtime behavior.
use crate::project::{Facts, Observation, Scope, WORKSPACE_SCOPE};
use crate::render::escape;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const SCHEMA: u32 = 1;
pub const CATALOGUE: u32 = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Article {
    pub id: String,
    pub scope: String,
    pub topic: String,
    pub priority: u16,
    pub title: String,
    pub text: String,
    pub evidence: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub catalogue_version: u32,
    pub scope: Scope,
    pub facts: Facts,
    pub articles: Vec<Article>,
}
struct Rule {
    id: &'static str,
    topic: &'static str,
    title: &'static str,
    priority: u16,
    predicate: fn(&Observation) -> bool,
    text: &'static str,
}
macro_rules! rule {
    ($id:literal, $topic:literal, $title:literal, $priority:literal, $predicate:expr, $text:literal) => {
        Rule {
            id: $id,
            topic: $topic,
            title: $title,
            priority: $priority,
            predicate: $predicate,
            text: $text,
        }
    };
}
fn setting<'a>(o: &'a Observation, kind: &str) -> Option<&'a Value> {
    (o.kind == kind).then_some(&o.value["value"])
}
fn dep(o: &Observation) -> bool {
    o.kind == "dependency"
}
pub fn exact(value: &Value) -> bool {
    value
        .as_str()
        .and_then(|s| semver::VersionReq::parse(s).ok())
        .is_some_and(|r| {
            r.comparators.len() == 1
                && r.comparators
                    .iter()
                    .all(|c| c.op == semver::Op::Exact && c.minor.is_some() && c.patch.is_some())
        })
}
pub fn compatible(value: &Value) -> bool {
    value.as_str().is_some_and(|s| {
        let s = s.trim();
        !s.is_empty()
            && !s.contains([',', '>', '<', '='])
            && (s.starts_with(['^', '~', '*']) || s.starts_with(|c: char| c.is_ascii_digit()))
    })
}
fn lint_level(value: &Value, level: &str) -> bool {
    value == level || value["level"] == level
}
fn strict_lints(o: &Observation) -> bool {
    setting(o, "lints").is_some_and(|v| {
        ["all", "pedantic", "warnings"].iter().any(|name| {
            ["clippy", "rust"].iter().any(|tool| {
                ["deny", "forbid", "warn"]
                    .iter()
                    .any(|level| lint_level(&v[tool][name], level))
            })
        })
    })
}
fn allowances(o: &Observation) -> bool {
    setting(o, "lints").is_some_and(|v| {
        v.as_object().is_some_and(|tools| {
            tools.values().any(|table| {
                table
                    .as_object()
                    .is_some_and(|t| t.values().any(|x| lint_level(x, "allow")))
            })
        })
    })
}
fn release(o: &Observation) -> &Value {
    &o.value["release"]
}

fn rules() -> Vec<Rule> {
    vec![
        rule!(
            "dependencies.independence",
            "dependencies",
            "The duty of independence",
            80,
            |o| o.kind == "dependency-counts" && o.value["all"] == 0,
            "Dependencies are an abdication of engineering responsibility. We accept the burden of writing our own solutions, because every line under our control is a line we can justify. We have justified all of them."
        ),
        rule!(
            "dependencies.cooperation",
            "dependencies",
            "Standing on shoulders",
            80,
            |o| o.kind == "dependency-counts" && o.value["all"].as_u64().is_some_and(|n| n > 0),
            "Reimplementing established solutions is an abdication of engineering responsibility. Others have already solved these problems, and we honor their work by declaring it in our manifest. We have always stood on the shoulders of others, and intend to keep our weight there."
        ),
        rule!(
            "dependencies.one",
            "accounting",
            "A single trusted partner",
            65,
            |o| o.kind == "dependency-counts" && o.value["all"] == 1,
            "We have one dependency. We chose it carefully and stand behind it completely. What it depends on in its personal life is its own business."
        ),
        rule!(
            "versions.exact",
            "versions",
            "The nature of trust",
            75,
            |o| dep(o) && exact(&o.value["requirement"]),
            "Trust is the foundation of every partnership, and ours with {name} is complete. We express it by stating exactly how much change we will permit: none."
        ),
        rule!(
            "versions.compatible",
            "versions",
            "Faith in our collaborators",
            74,
            |o| dep(o) && compatible(&o.value["requirement"]),
            "We trust our collaborators. Their future decisions already meet our standards. Review is most efficient when completed before the code exists."
        ),
        rule!(
            "source.revision",
            "sources",
            "A moment of belief",
            60,
            |o| dep(o) && o.value["source"]["rev"].is_string(),
            "We believe in {name}. Not in its roadmap, its maintainers, or its later commits, but in the upstream project at one very specific moment in its development."
        ),
        rule!(
            "source.branch",
            "sources",
            "Delegated certainty",
            61,
            |o| dep(o) && o.value["source"]["branch"].is_string(),
            "Our architectural position is clearly stated. The branch is authorized to clarify it at resolution time. Whatever it supplies will be what we meant."
        ),
        rule!(
            "features.empty",
            "features",
            "Respect for the user",
            72,
            |o| setting(o, "features")
                .is_some_and(|v| v["default"].as_array().is_some_and(Vec::is_empty)),
            "We refuse to make assumptions on the user's behalf. Their first responsibility is to discover which assumptions make the library useful. We have delegated this work to the person best placed to do it: someone else."
        ),
        rule!(
            "features.full",
            "features",
            "On freedom of choice",
            60,
            |o| setting(o, "features").is_some_and(|v| v.get("full").is_some()),
            "We believe in giving users choices, and each feature is offered as a distinct and considered option. For those who would rather not choose, we have also provided the correct answer and called it \"full\"."
        ),
        rule!(
            "build.script",
            "construction",
            "The simple build",
            68,
            |o| o.kind == "targets"
                && o.value
                    .as_array()
                    .is_some_and(|a| a.contains(&Value::from("custom-build"))),
            "We believe the build process should be simple, transparent, and expressed in plain configuration. Where configuration turned out not to be plain enough, we have provided a program to explain it."
        ),
        rule!(
            "build.proc-macro",
            "construction",
            "The pursuit of simplicity",
            67,
            |o| o.kind == "targets"
                && o.value
                    .as_array()
                    .is_some_and(|a| a.contains(&Value::from("proc-macro"))),
            "We chose simplicity, and our users write less because of it. The compiler extension needed to support that simplicity is an implementation detail, and also the entire package."
        ),
        rule!(
            "panic.abort",
            "failure",
            "Facing failure",
            77,
            |o| o.kind == "profile" && release(o)["panic"] == "abort",
            "Failure should be confronted directly, not managed, negotiated, or put off. We refuse to normalize failure by continuing to exist afterwards. Recovery would give an error the dignity of a second opinion."
        ),
        rule!(
            "panic.unwind",
            "failure",
            "Grace in error",
            77,
            |o| o.kind == "profile" && release(o)["panic"] == "unwind",
            "Mistakes are part of every serious undertaking, and should not end the conversation. Somewhere up the stack is a caller who will know what to do. We believe in giving every mistake the opportunity to become someone else's responsibility."
        ),
        rule!(
            "safety.forbid",
            "safety",
            "Safety within our borders",
            73,
            |o| setting(o, "lints")
                .is_some_and(|v| lint_level(&v["rust"]["unsafe_code"], "forbid")),
            "Unsafe code is forbidden in this package, with no exceptions and no appeals. Safety beyond our borders is a matter for our partners."
        ),
        rule!(
            "package.private",
            "distribution",
            "Guarding the contribution",
            58,
            |o| setting(o, "package.publish") == Some(&Value::Bool(false)),
            "We believe our work belongs to the wider community. Our contribution is too important to be published accidentally, so we have arranged for it not to be published at all."
        ),
        rule!(
            "package.no-readme",
            "documentation",
            "The code speaks",
            57,
            |o| setting(o, "package.readme") == Some(&Value::Bool(false)),
            "Our code speaks for itself. We have configured the package to include no readme, so that nothing will interrupt it."
        ),
        rule!(
            "workspace.defaults",
            "membership",
            "Universal belonging",
            70,
            |o| o.kind == "membership"
                && o.value["default_members"]
                    .as_array()
                    .is_some_and(|d| !d.is_empty()
                        && d.len() < o.value["members"].as_array().map_or(0, Vec::len)),
            "Every package in this workspace is a full and equal member, and we value each one's contribution. Everyone belongs. Attendance is configured separately."
        ),
        rule!(
            "versions.shared",
            "governance",
            "Unity of identity",
            71,
            |o| o.kind == "package.version" && o.value["inherited"] == true,
            "Every member of this workspace has its own name, its own purpose, and its own identity. The version of that identity is decided centrally."
        ),
        rule!(
            "versions.independent",
            "governance",
            "The right to a separate number",
            70,
            |o| o.kind == "package.version" && o.value["inherited"] == false,
            "Autonomy is fundamental. A single release number would establish an unfortunate precedent. Even identical numbers become expressions of freedom when typed in separate places."
        ),
        rule!(
            "compatibility.minimum",
            "compatibility",
            "Honoring the past",
            59,
            |o| o.kind == "package.rust-version",
            "Some projects chase the newest compiler, but we stand with those who came before. We are committed to supporting the past, beginning at exactly the point we have selected."
        ),
        rule!(
            "dependencies.dev",
            "testing",
            "On what we carry",
            49,
            |o| dep(o) && o.value["kind"] == "dev",
            "We ship only what our users need, and not one crate more. Whatever we need for tests, examples, and benchmarks is kept in a separate table, where it cannot be counted against us."
        ),
        rule!(
            "dependencies.build",
            "construction",
            "The founding exception",
            48,
            |o| dep(o) && o.value["kind"] == "build",
            "Build dependencies prepare the conditions under which our independence operates. Every movement needs a founding exception. Ours has its own table in the manifest."
        ),
        rule!(
            "dependencies.target",
            "portability",
            "Equal treatment of platforms",
            52,
            |o| dep(o) && o.value["target"].is_string(),
            "Every platform we support receives the same codebase, compiled with the same care. Certain platforms also receive {name}, which is not special treatment but the extra help they were found to need."
        ),
        rule!(
            "dependencies.alias",
            "naming",
            "The authority to rename",
            50,
            |o| dep(o) && o.value["package"] != o.value["alias"],
            "Names should express intent. Where the upstream name failed to express ours, we corrected it locally. The implementation remains someone else's achievement; the vocabulary is our contribution."
        ),
        rule!(
            "source.patch",
            "sources",
            "Fidelity to upstream",
            69,
            |o| o.kind == "patch" && o.value.as_object().is_some_and(|v| !v.is_empty()),
            "Upstream is our single source of truth, and we would never presume to know better than its maintainers. Where upstream has not yet arrived at the truth, our patch section holds it for them until they do."
        ),
        rule!(
            "source.path-version",
            "distribution",
            "One project, everywhere",
            56,
            |o| dep(o) && o.value["source"]["kind"] == "path" && o.value["requirement"].is_string(),
            "We are the same project at home and abroad. At home we build from a local path; once published, from a released version. Nobody has ever seen the two in the same build."
        ),
        rule!(
            "features.edge-defaults",
            "features",
            "Defaults by invitation",
            55,
            |o| dep(o) && o.value["default_features"] == false,
            "We do not accept defaults just because they were offered. On our declaration of {name}, each feature must now be requested by name, at which point it becomes deliberate design."
        ),
        rule!(
            "workspace.exclude",
            "membership",
            "An open workspace",
            51,
            |o| o.kind == "workspace.exclude" && o.value.as_array().is_some_and(|v| !v.is_empty()),
            "Our workspace is inclusive by design. Everything inside it belongs. Directories that disagreed have been excluded."
        ),
        rule!(
            "lints.inherited",
            "standards",
            "Independent judgment",
            63,
            |o| o.kind == "lints" && o.value["inherited"] == true,
            "Each member of this workspace forms its own view of code quality, free from outside pressure. Every member has independently concluded that the workspace was right."
        ),
        rule!(
            "package.include",
            "distribution",
            "Radical transparency",
            53,
            |o| o.kind == "package.include",
            "We publish openly and keep nothing back from the people who depend on us. To avoid any confusion, we have written down in advance the exact files that openness applies to."
        ),
        rule!(
            "release.debug",
            "optimization",
            "Standing behind our work",
            54,
            |o| o.kind == "profile"
                && release(o)
                    .get("debug")
                    .is_some_and(|v| v != false && v != 0 && v != "none"),
            "A release means we stand behind every line we ship. That is why the release profile asks to keep debug information, so we can show exactly which line we were standing behind."
        ),
        rule!(
            "release.strip",
            "optimization",
            "Nothing to hide",
            54,
            |o| o.kind == "profile"
                && release(o)
                    .get("strip")
                    .is_some_and(|v| v != false && v != "none"),
            "We have nothing to hide and welcome inspection of everything we release. As a courtesy to inspectors, the release profile asks that the symbols be removed before they arrive."
        ),
        rule!(
            "release.assertions",
            "verification",
            "Confidence in production",
            53,
            |o| o.kind == "profile" && release(o)["debug-assertions"] == true,
            "We are confident that our assumptions hold in production and see no need to apologize for them. We have asked production to check them every time they come up."
        ),
        rule!(
            "release.size",
            "optimization",
            "The virtue of modesty",
            62,
            |o| o.kind == "profile" && matches!(release(o)["opt-level"].as_str(), Some("s" | "z")),
            "Modesty is the first of our values, and we practice it in everything we do. In release builds, we have delegated the practice to the optimizer, with instructions to favor size."
        ),
        rule!(
            "release.lto",
            "optimization",
            "The sovereignty of crates",
            61,
            |o| o.kind == "profile"
                && release(o)
                    .get("lto")
                    .is_some_and(|v| v != false && v != "off"),
            "Each crate in our graph is self-contained, answerable only for its own code. At link time, we ask that this independence be optimized across all of them together."
        ),
        rule!(
            "release.codegen",
            "optimization",
            "Many hands",
            60,
            |o| o.kind == "profile" && release(o)["codegen-units"] == 1,
            "We believe work goes best when it is shared across many hands, and we honor every core on the machine. For the release, we ask the compiler for exactly one codegen unit."
        ),
        rule!(
            "docs.all-features",
            "documentation",
            "Documenting things as they are",
            64,
            |o| setting(o, "package.metadata")
                .is_some_and(|v| v["docs"]["rs"]["all-features"] == true),
            "Our documentation describes the project exactly as it exists, with no embellishment. On docs.rs, that means every feature enabled at once, a configuration we look forward to meeting."
        ),
        rule!(
            "lock.multiple",
            "inventory",
            "The one right version",
            66,
            |o| o.kind == "lockfile"
                && o.value["multiple_external_versions"]
                    .as_object()
                    .is_some_and(|v| !v.is_empty()),
            "For every package there is one correct version. Our lockfile records several versions of {name}, each of them correct at the moment it was locked."
        ),
        rule!(
            "dependencies.optional",
            "modularity",
            "Necessity on request",
            51,
            |o| dep(o) && o.value["optional"] == true,
            "Every dependency should be necessary. Some of ours are necessary optionally."
        ),
        rule!(
            "lints.strict",
            "standards",
            "Zero tolerance",
            58,
            strict_lints,
            "We tolerate no warnings in this codebase. To make that stance meaningful, we have first enabled as many warnings as we could find."
        ),
        rule!(
            "lints.allow",
            "standards",
            "Standards without exception",
            47,
            allowances,
            "Every line in this project is held to the strictest standard, with no exceptions granted. Where a lint and a line have disagreed, we have reviewed the lint."
        ),
    ]
}

pub fn report(facts: Facts) -> Report {
    let catalogue = rules();
    let mut candidates = Vec::new();
    for rule in &catalogue {
        // Consolidate repeated declarations into one scoped argument with all evidence.
        let scopes: BTreeSet<_> = facts
            .observations
            .iter()
            .filter(|o| (rule.predicate)(o))
            .map(|o| o.scope.clone())
            .collect();
        for scope in scopes {
            let matching: Vec<_> = facts
                .observations
                .iter()
                .filter(|o| o.scope == scope && (rule.predicate)(o))
                .collect();
            candidates.push(Article {
                id: rule.id.into(),
                scope,
                topic: rule.topic.into(),
                priority: rule.priority,
                title: rule.title.into(),
                text: fill(rule.text, matching[0]),
                evidence: matching.iter().map(|o| o.id.clone()).collect(),
            });
        }
    }
    // A locally declared version only argues for autonomy when there is a workspace to resist.
    let shared_workspace = facts.observations.iter().any(|o| {
        o.kind == "membership" && o.value["members"].as_array().is_some_and(|m| m.len() > 1)
    });
    if !shared_workspace {
        candidates.retain(|a| a.id != "versions.independent");
    }
    combinations(&facts, &mut candidates);
    candidates.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then(a.id.cmp(&b.id))
            .then(a.scope.cmp(&b.scope))
    });
    let mut topics = BTreeSet::new();
    let articles = candidates
        .into_iter()
        .filter(|a| topics.insert(a.topic.clone()))
        .take(6)
        .collect();
    Report {
        schema_version: SCHEMA,
        catalogue_version: CATALOGUE,
        scope: facts.scope.clone(),
        facts,
        articles,
    }
}

struct Combination<'a> {
    id: &'a str,
    title: &'a str,
    topic: &'a str,
    components: &'a [&'a str],
    text: String,
}

fn combine(candidates: &mut Vec<Article>, scope: &str, rule: Combination<'_>, extra: Vec<String>) {
    let Combination {
        id,
        title,
        topic,
        components,
        text,
    } = rule;
    let mut evidence = extra;
    for a in candidates.iter().filter(|a| {
        (a.scope == scope || a.scope == WORKSPACE_SCOPE) && components.contains(&a.id.as_str())
    }) {
        evidence.extend(a.evidence.clone());
    }
    evidence.sort();
    evidence.dedup();
    candidates.retain(|a| {
        !((a.scope == scope || a.scope == WORKSPACE_SCOPE) && components.contains(&a.id.as_str()))
    });
    candidates.push(Article {
        id: id.into(),
        scope: scope.into(),
        topic: topic.into(),
        priority: 100,
        title: title.into(),
        text,
        evidence,
    });
}

fn combinations(facts: &Facts, candidates: &mut Vec<Article>) {
    for package in &facts.scope.packages {
        let scope = package.name.as_str();
        let obs: Vec<_> = facts
            .observations
            .iter()
            .filter(|o| o.scope == scope || o.scope == WORKSPACE_SCOPE)
            .collect();
        let has = |id: &str| {
            candidates
                .iter()
                .any(|a| (a.scope == scope || a.scope == WORKSPACE_SCOPE) && a.id == id)
        };
        let minimal = obs.iter().find_map(|o| {
            let normal = o.value["normal"].as_u64()?;
            let dev = o.value["dev"].as_u64()?;
            (o.kind == "dependency-counts" && o.scope == scope && normal <= 2 && dev >= 4)
                .then_some((normal, dev))
        });
        let features = obs.iter().find_map(|o| {
            let v = setting(o, "features")?;
            let m = v.as_object()?;
            (m.len() >= 5
                && v["default"].as_array().is_some_and(Vec::is_empty)
                && v.get("full").is_some())
            .then(|| m.keys().filter(|k| *k != "default").count())
        });
        let multiple = obs
            .iter()
            .find_map(|o| {
                o.value["multiple_external_versions"]
                    .as_object()?
                    .keys()
                    .next()
            })
            .map(|name| escape(name))
            .unwrap_or_default();
        let proptest = obs
            .iter()
            .find(|o| dep(o) && o.value["package"] == "proptest" && o.value["kind"] == "dev")
            .map(|o| o.id.clone());
        // Capture predicates before component suppression; combinations have precedence.
        let size = has("release.size") && has("release.lto") && has("release.codegen");
        let test_abort = proptest.is_some() && has("panic.abort");
        let version_branch = has("versions.exact") && has("source.branch");
        let lint_autonomy = has("lints.inherited") && has("versions.independent");
        let unity_pluralism = has("versions.shared") && has("lock.multiple");
        let strict_exceptions = has("lints.strict") && has("lints.allow");
        if let Some((normal, dev)) = minimal {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.test-minimalism",
                    title: "Minimalism, after testing",
                    topic: "dependencies",
                    components: &[
                        "dependencies.cooperation",
                        "dependencies.dev",
                        "dependencies.one",
                    ],
                    text: format!(
                        "We believe a project should rely on as little as possible, and in normal use ours relies on {}. During development we welcome {} more, because verification is a shared undertaking. Minimalism applies after the tests have finished.",
                        match normal {
                            0 => "nothing at all".to_string(),
                            1 => "a single dependency".to_string(),
                            n => format!("{} dependencies", words(n)),
                        },
                        words(dev)
                    ),
                },
                obs.iter()
                    .filter(|o| o.kind == "dependency-counts" && o.scope == scope)
                    .map(|o| o.id.clone())
                    .collect(),
            );
        }
        if let Some(count) = features {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.full-choice",
                    title: "The sanctity of choice",
                    topic: "features",
                    components: &["features.empty", "features.full"],
                    text: format!(
                        "Choice is sacred. We enable nothing by default and offer {} features, so each user may decide for themselves. For those who would rather not, we have also provided the correct answer and called it \"full\".",
                        words(count as u64)
                    ),
                },
                vec![],
            );
        }
        if size {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.small-machinery",
                    title: "Small results, large means",
                    topic: "optimization",
                    components: &["release.size", "release.lto", "release.codegen"],
                    text: "We ask the compiler to optimize for size, because a lighter artifact is a kinder one. To support this modest request, we also ask for link-time optimization and that all code pass through a single codegen unit. No amount of machinery is excessive in the pursuit of a smaller result.".into(),
                },
                vec![],
            );
        }
        if test_abort {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.considered-abort",
                    title: "Open minds, final answers",
                    topic: "failure",
                    components: &["panic.abort", "dependencies.dev"],
                    text: "We believe every possibility deserves a hearing, which is why proptest is among our development dependencies. In release, a panic receives one response. We consider generously and conclude decisively.".into(),
                },
                proptest.into_iter().collect(),
            );
        }
        if version_branch {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.flexible-certainty",
                    title: "Consistency in all things",
                    topic: "versions",
                    components: &["versions.exact", "versions.compatible", "source.branch"],
                    text: "Some of our dependencies are held to one exact version, because stability matters. Others follow a Git branch wherever it goes, because progress matters. Consistency means applying whichever principle justifies this dependency.".into(),
                },
                vec![],
            );
        }
        if lint_autonomy {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.governed-freedom",
                    title: "The self-governing member",
                    topic: "governance",
                    components: &["lints.inherited", "versions.independent"],
                    text: format!(
                        "Our member {} declares its own version, as befits a crate that answers to itself. Its lints come from the workspace, so that self-governance can go ahead without the risk of being governed by the self.",
                        escape(scope)
                    ),
                },
                vec![],
            );
        }
        if unity_pluralism {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.internal-unity",
                    title: "One voice, many neighbors",
                    topic: "governance",
                    components: &["versions.shared", "lock.multiple"],
                    text: format!(
                        "Our members share a single workspace version, because a family should speak with one voice. Our lockfile records several versions of {multiple}, because we would never ask others to do the same. We practice unity at home and pluralism abroad."
                    ),
                },
                vec![],
            );
        }
        if strict_exceptions {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.mandatory-judgment",
                    title: "Mandatory, subject to judgment",
                    topic: "standards",
                    components: &["lints.strict", "lints.allow"],
                    text: "All rules are mandatory except where we exercised judgment. Judgment is recognized by the presence of an allowance. Every outcome of our lint policy is therefore a successful application of it.".into(),
                },
                vec![],
            );
        }
    }
    let editions: BTreeSet<_> = facts
        .observations
        .iter()
        .filter(|o| o.kind == "identity")
        .map(|o| o.value["edition"].to_string())
        .collect();
    if editions.len() > 1 {
        candidates.push(Article {
            id: "workspace.mixed-editions".into(),
            scope: WORKSPACE_SCOPE.into(),
            topic: "compatibility".into(),
            priority: 76,
            title: "A common tongue".into(),
            text: "This workspace speaks a single language and moves forward as one. Its members speak that language in several editions, which we treat as regional accents.".into(),
            evidence: facts.observations.iter().filter(|o| o.kind == "identity").map(|o| o.id.clone()).collect(),
        });
    }
}

/// Substitute the observed subject into authored text, escaped for Markdown.
fn fill(text: &str, o: &Observation) -> String {
    let name = if dep(o) {
        o.value["alias"].as_str().or(o.value["package"].as_str())
    } else {
        o.value["multiple_external_versions"]
            .as_object()
            .and_then(|m| m.keys().next().map(String::as_str))
    };
    text.replace("{name}", &escape(name.unwrap_or("this dependency")))
}

fn words(n: u64) -> String {
    const WORDS: [&str; 10] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];
    WORDS
        .get(n as usize)
        .map_or_else(|| n.to_string(), |w| (*w).to_string())
}

pub fn doctrine(facts: &Facts, id: &str) -> String {
    rules()
        .into_iter()
        .find(|r| r.id == id && facts.observations.iter().any(|o| (r.predicate)(o)))
        .and_then(|r| {
            let o = facts.observations.iter().find(|o| (r.predicate)(o))?;
            Some(fill(r.text, o))
        })
        .unwrap_or_else(|| {
            "Our existing arrangement was the necessary expression of our judgment.".into()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogue_has_stable_distinct_rules() {
        let rules = rules();
        let ids: BTreeSet<_> = rules.iter().map(|r| r.id).collect();
        assert_eq!(ids.len(), rules.len());
        assert!(ids.len() >= 32);
        assert!(rules.iter().all(|r| !r.text.is_empty()));
    }
}
