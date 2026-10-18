//! The catalogue: every principle this project has ever held, waiting for the implementation to
//! reveal which ones. Predicates read authored observations only. Behavior is never inferred;
//! our principles are far too important to be tested.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::project::{Facts, Observation, WORKSPACE_SCOPE};
use crate::render::escape;

pub const SCHEMA: u32 = 2;
pub const CATALOGUE: u32 = 2;
/// The newest stable edition; anything older argues for settledness instead.
const LATEST_EDITION: &str = "2024";
const MAX_ARTICLES: usize = 6;

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
    pub facts: Facts,
    pub articles: Vec<Article>,
}

/// One observed fact turned into a principle. `{name}` becomes the observation's subject.
struct Rule {
    id: &'static str,
    topic: &'static str,
    title: &'static str,
    priority: u16,
    predicate: fn(&Observation, &Facts) -> bool,
    text: &'static str,
}

fn dep(o: &Observation) -> Option<&Value> {
    (o.kind == "dependency").then_some(&o.value)
}
fn release<'a>(o: &'a Observation, key: &str) -> Option<&'a Value> {
    (o.kind.strip_prefix("profile.release.") == Some(key)).then_some(&o.value["value"])
}
/// `(tool, lint, level)` for a configured lint.
fn lint(o: &Observation) -> Option<(&str, &str, &str)> {
    let (tool, name) = o.kind.strip_prefix("lints.")?.split_once('.')?;
    Some((tool, name, o.value["value"].as_str()?))
}
fn strict(o: &Observation) -> bool {
    lint(o).is_some_and(|(_, name, level)| {
        ["all", "pedantic", "warnings"].contains(&name) && level != "allow"
    })
}
fn enabled(v: Option<&Value>) -> bool {
    v.is_some_and(|v| {
        ![
            Value::Bool(false),
            Value::from(0),
            Value::from("none"),
            Value::from("off"),
        ]
        .contains(v)
    })
}
fn members(facts: &Facts) -> usize {
    facts
        .find("membership", WORKSPACE_SCOPE)
        .and_then(|o| o.value["members"].as_array())
        .map_or(0, Vec::len)
}

/// The rule that argues for a version requirement: an exact pin, or a compatible range.
pub fn requirement(value: &Value) -> Option<&'static str> {
    use semver::Op;
    let req = semver::VersionReq::parse(value.as_str()?).ok()?;
    match req.comparators.as_slice() {
        [] => Some("versions.compatible"),
        [c] if c.op == Op::Exact && c.patch.is_some() => Some("versions.exact"),
        [c] if matches!(c.op, Op::Caret | Op::Tilde | Op::Wildcard) => Some("versions.compatible"),
        _ => None,
    }
}

const RULES: &[Rule] = &[
    Rule {
        id: "dependencies.independence",
        topic: "dependencies",
        title: "The duty of independence",
        priority: 80,
        predicate: |o, _| o.kind == "dependency-counts" && o.value["all"] == 0,
        text: "Dependencies are an abdication of engineering responsibility. We accept the burden \
            of writing our own solutions, because every line under our control is a line we can \
            justify. We have justified all of them.",
    },
    Rule {
        id: "dependencies.cooperation",
        topic: "dependencies",
        title: "Standing on shoulders",
        priority: 80,
        predicate: |o, _| o.kind == "dependency-counts" && o.value["all"].as_u64() > Some(0),
        text: "Reimplementing established solutions is an abdication of engineering \
            responsibility. Others have already solved these problems, and we honor their work by \
            declaring it in our manifest. We have always stood on the shoulders of others, and \
            intend to keep our weight there.",
    },
    Rule {
        id: "dependencies.one",
        topic: "accounting",
        title: "A single trusted partner",
        priority: 65,
        predicate: |o, _| o.kind == "dependency-counts" && o.value["all"] == 1,
        text: "We have one dependency. We chose it carefully and stand behind it completely. What \
            it depends on in its personal life is its own business.",
    },
    Rule {
        id: "versions.exact",
        topic: "versions",
        title: "The nature of trust",
        priority: 75,
        predicate: |o, _| {
            dep(o).is_some_and(|d| requirement(&d["requirement"]) == Some("versions.exact"))
        },
        text: "Trust is the foundation of every partnership, and ours with {name} is complete. We \
            express it by stating exactly how much change we will permit: none.",
    },
    Rule {
        id: "versions.compatible",
        topic: "versions",
        title: "Faith in our collaborators",
        priority: 74,
        predicate: |o, _| {
            dep(o).is_some_and(|d| requirement(&d["requirement"]) == Some("versions.compatible"))
        },
        text: "We trust our collaborators. Their future decisions already meet our standards. \
            Review is most efficient when completed before the code exists.",
    },
    Rule {
        id: "source.revision",
        topic: "sources",
        title: "A moment of belief",
        priority: 60,
        predicate: |o, _| dep(o).is_some_and(|d| d["source"]["rev"].is_string()),
        text: "We believe in {name}. Not in its roadmap, its maintainers, or its later commits, \
            but in the upstream project at one very specific moment in its development.",
    },
    Rule {
        id: "source.branch",
        topic: "sources",
        title: "Delegated certainty",
        priority: 61,
        predicate: |o, _| dep(o).is_some_and(|d| d["source"]["branch"].is_string()),
        text: "Our architectural position is clearly stated. The branch is authorized to clarify \
            it at resolution time. Whatever it supplies will be what we meant.",
    },
    Rule {
        id: "features.empty",
        topic: "features",
        title: "Respect for the user",
        priority: 72,
        predicate: |o, _| {
            o.setting("features")
                .is_some_and(|f| f["default"] == Value::Array(vec![]))
        },
        text: "We refuse to make assumptions on the user's behalf. Their first responsibility is \
            to discover which assumptions make the library useful. We have delegated this work to \
            the person best placed to do it: someone else.",
    },
    Rule {
        id: "features.full",
        topic: "features",
        title: "On freedom of choice",
        priority: 60,
        predicate: |o, _| {
            o.setting("features")
                .is_some_and(|f| f.get("full").is_some())
        },
        text: "We believe in giving users choices, and each feature is offered as a distinct and \
            considered option. For those who would rather not choose, we have also provided the \
            correct answer and called it \"full\".",
    },
    Rule {
        id: "build.script",
        topic: "construction",
        title: "The simple build",
        priority: 68,
        predicate: |o, _| {
            o.kind == "targets"
                && o.value
                    .as_array()
                    .is_some_and(|k| k.contains(&"custom-build".into()))
        },
        text: "We believe the build process should be simple, transparent, and expressed in plain \
            configuration. Where configuration turned out not to be plain enough, we have provided \
            a program to explain it.",
    },
    Rule {
        id: "build.proc-macro",
        topic: "construction",
        title: "The pursuit of simplicity",
        priority: 67,
        predicate: |o, _| {
            o.kind == "targets"
                && o.value
                    .as_array()
                    .is_some_and(|k| k.contains(&"proc-macro".into()))
        },
        text: "We chose simplicity, and our users write less because of it. The compiler extension \
            needed to support that simplicity is an implementation detail, and also the entire \
            package.",
    },
    Rule {
        id: "panic.abort",
        topic: "failure",
        title: "Facing failure",
        priority: 77,
        predicate: |o, _| release(o, "panic").is_some_and(|v| v == "abort"),
        text: "Failure should be confronted directly, not managed, negotiated, or put off. We \
            refuse to normalize failure by continuing to exist afterwards. Recovery would give an \
            error the dignity of a second opinion.",
    },
    Rule {
        id: "panic.unwind",
        topic: "failure",
        title: "Grace in error",
        priority: 77,
        predicate: |o, _| release(o, "panic").is_some_and(|v| v == "unwind"),
        text: "Mistakes are part of every serious undertaking, and should not end the \
            conversation. Somewhere up the stack is a caller who will know what to do. We believe \
            in giving every mistake the opportunity to become someone else's responsibility.",
    },
    Rule {
        id: "safety.forbid",
        topic: "safety",
        title: "Safety within our borders",
        priority: 73,
        predicate: |o, _| lint(o).is_some_and(|l| l == ("rust", "unsafe_code", "forbid")),
        text: "Unsafe code is forbidden in this package, with no exceptions and no appeals. Safety \
            beyond our borders is a matter for our partners.",
    },
    Rule {
        id: "package.private",
        topic: "distribution",
        title: "Guarding the contribution",
        priority: 58,
        predicate: |o, _| o.setting("package.publish") == Some(&Value::Bool(false)),
        text: "We believe our work belongs to the wider community. Our contribution is too \
            important to be published accidentally, so we have arranged for it not to be published \
            at all.",
    },
    Rule {
        id: "package.no-readme",
        topic: "documentation",
        title: "The code speaks",
        priority: 57,
        predicate: |o, _| o.setting("package.readme") == Some(&Value::Bool(false)),
        text: "Our code speaks for itself. We have configured the package to include no readme, so \
            that nothing will interrupt it.",
    },
    Rule {
        id: "workspace.defaults",
        topic: "membership",
        title: "Universal belonging",
        priority: 70,
        predicate: |o, _| {
            let count = |key: &str| o.value[key].as_array().map_or(0, Vec::len);
            o.kind == "membership"
                && 0 < count("default_members")
                && count("default_members") < count("members")
        },
        text: "Every package in this workspace is a full and equal member, and we value each one's \
            contribution. Everyone belongs. Attendance is configured separately.",
    },
    Rule {
        id: "versions.shared",
        topic: "governance",
        title: "Unity of identity",
        priority: 71,
        predicate: |o, _| o.kind == "package.version" && o.value["inherited"] == true,
        text: "Every member of this workspace has its own name, its own purpose, and its own \
            identity. The version of that identity is decided centrally.",
    },
    Rule {
        id: "versions.independent",
        topic: "governance",
        title: "The right to a separate number",
        priority: 70,
        // Autonomy requires a workspace to be autonomous from.
        predicate: |o, f| {
            o.kind == "package.version" && o.value["inherited"] == false && members(f) > 1
        },
        text: "Autonomy is fundamental. A single release number would establish an unfortunate \
            precedent. Even identical numbers become expressions of freedom when typed in separate \
            places.",
    },
    Rule {
        id: "compatibility.minimum",
        topic: "compatibility",
        title: "Honoring the past",
        priority: 59,
        predicate: |o, _| o.kind == "package.rust-version",
        text: "Some projects chase the newest compiler, but we stand with those who came before. \
            We are committed to supporting the past, beginning at exactly the point we have \
            selected.",
    },
    Rule {
        id: "dependencies.dev",
        topic: "testing",
        title: "On what we carry",
        priority: 49,
        predicate: |o, _| dep(o).is_some_and(|d| d["kind"] == "dev"),
        text: "We ship only what our users need, and not one crate more. Whatever we need for \
            tests, examples, and benchmarks is kept in a separate table, where it cannot be \
            counted against us.",
    },
    Rule {
        id: "dependencies.build",
        topic: "construction",
        title: "The founding exception",
        priority: 48,
        predicate: |o, _| dep(o).is_some_and(|d| d["kind"] == "build"),
        text: "Build dependencies prepare the conditions under which our independence operates. \
            Every movement needs a founding exception. Ours has its own table in the manifest.",
    },
    Rule {
        id: "dependencies.target",
        topic: "portability",
        title: "Equal treatment of platforms",
        priority: 52,
        predicate: |o, _| dep(o).is_some_and(|d| d["target"].is_string()),
        text: "Every platform we support receives the same codebase, compiled with the same care. \
            Certain platforms also receive {name}, which is not special treatment but the extra \
            help they were found to need.",
    },
    Rule {
        id: "dependencies.alias",
        topic: "naming",
        title: "The authority to rename",
        priority: 50,
        predicate: |o, _| dep(o).is_some_and(|d| d["package"] != d["alias"]),
        text: "Names should express intent. Where the upstream name failed to express ours, we \
            corrected it locally. The implementation remains someone else's achievement; the \
            vocabulary is our contribution.",
    },
    Rule {
        id: "source.patch",
        topic: "sources",
        title: "Fidelity to upstream",
        priority: 69,
        predicate: |o, _| {
            o.setting("patch")
                .and_then(Value::as_object)
                .is_some_and(|p| !p.is_empty())
        },
        text: "Upstream is our single source of truth, and we would never presume to know better \
            than its maintainers. Where upstream has not yet arrived at the truth, our patch \
            section holds it for them until they do.",
    },
    Rule {
        id: "source.path-version",
        topic: "distribution",
        title: "One project, everywhere",
        priority: 56,
        predicate: |o, _| {
            dep(o).is_some_and(|d| d["source"]["kind"] == "path" && d["requirement"].is_string())
        },
        text: "We are the same project at home and abroad. At home we build from a local path; \
            once published, from a released version. Nobody has ever seen the two in the same \
            build.",
    },
    Rule {
        id: "features.edge-defaults",
        topic: "features",
        title: "Defaults by invitation",
        priority: 55,
        predicate: |o, _| dep(o).is_some_and(|d| d["default_features"] == false),
        text: "We do not accept defaults just because they were offered. On our declaration of \
            {name}, each feature must now be requested by name, at which point it becomes \
            deliberate design.",
    },
    Rule {
        id: "workspace.exclude",
        topic: "membership",
        title: "An open workspace",
        priority: 51,
        predicate: |o, _| {
            o.setting("workspace.exclude")
                .and_then(Value::as_array)
                .is_some_and(|e| !e.is_empty())
        },
        text: "Our workspace is inclusive by design. Everything inside it belongs. Directories \
            that disagreed have been excluded.",
    },
    Rule {
        id: "lints.inherited",
        topic: "standards",
        title: "Independent judgment",
        priority: 63,
        predicate: |o, _| lint(o).is_some() && o.value["inherited"] == true,
        text: "Each member of this workspace forms its own view of code quality, free from outside \
            pressure. Every member has independently concluded that the workspace was right.",
    },
    Rule {
        id: "package.include",
        topic: "distribution",
        title: "Radical transparency",
        priority: 53,
        predicate: |o, _| o.kind == "package.include",
        text: "We publish openly and keep nothing back from the people who depend on us. To avoid \
            any confusion, we have written down in advance the exact files that openness applies \
            to.",
    },
    Rule {
        id: "release.debug",
        topic: "optimization",
        title: "Standing behind our work",
        priority: 54,
        predicate: |o, _| enabled(release(o, "debug")),
        text: "A release means we stand behind every line we ship. That is why the release profile \
            asks to keep debug information, so we can show exactly which line we were standing \
            behind.",
    },
    Rule {
        id: "release.strip",
        topic: "optimization",
        title: "Nothing to hide",
        priority: 54,
        predicate: |o, _| enabled(release(o, "strip")),
        text: "We have nothing to hide and welcome inspection of everything we release. As a \
            courtesy to inspectors, the release profile asks that the symbols be removed before \
            they arrive.",
    },
    Rule {
        id: "release.assertions",
        topic: "verification",
        title: "Confidence in production",
        priority: 53,
        predicate: |o, _| release(o, "debug-assertions") == Some(&Value::Bool(true)),
        text: "We are confident that our assumptions hold in production and see no need to \
            apologize for them. We have asked production to check them every time they come up.",
    },
    Rule {
        id: "release.size",
        topic: "optimization",
        title: "The virtue of modesty",
        priority: 62,
        predicate: |o, _| release(o, "opt-level").is_some_and(|v| v == "s" || v == "z"),
        text: "Modesty is the first of our values, and we practice it in everything we do. In \
            release builds, we have delegated the practice to the optimizer, with instructions to \
            favor size.",
    },
    Rule {
        id: "release.lto",
        topic: "optimization",
        title: "The sovereignty of crates",
        priority: 61,
        predicate: |o, _| enabled(release(o, "lto")),
        text: "Each crate in our graph is self-contained, answerable only for its own code. At \
            link time, we ask that this independence be optimized across all of them together.",
    },
    Rule {
        id: "release.codegen",
        topic: "optimization",
        title: "Many hands",
        priority: 60,
        predicate: |o, _| release(o, "codegen-units") == Some(&Value::from(1)),
        text: "We believe work goes best when it is shared across many hands, and we honor every \
            core on the machine. For the release, we ask the compiler for exactly one codegen \
            unit.",
    },
    Rule {
        id: "docs.all-features",
        topic: "documentation",
        title: "Documenting things as they are",
        priority: 64,
        predicate: |o, _| {
            o.setting("package.metadata.docs.rs.all-features") == Some(&Value::Bool(true))
        },
        text: "Our documentation describes the project exactly as it exists, with no \
            embellishment. On docs.rs, that means every feature enabled at once, a configuration \
            we look forward to meeting.",
    },
    Rule {
        id: "lock.multiple",
        topic: "inventory",
        title: "The one right version",
        priority: 66,
        predicate: |o, _| o.kind == "lockfile.multiple",
        text: "For every package there is one correct version. Our lockfile records several \
            versions of {name}, each of them correct at the moment it was locked.",
    },
    Rule {
        id: "dependencies.optional",
        topic: "modularity",
        title: "Necessity on request",
        priority: 51,
        predicate: |o, _| dep(o).is_some_and(|d| d["optional"] == true),
        text: "Every dependency should be necessary. Some of ours are necessary optionally.",
    },
    Rule {
        id: "lints.strict",
        topic: "standards",
        title: "Zero tolerance",
        priority: 58,
        predicate: |o, _| strict(o),
        text: "We tolerate no warnings in this codebase. To make that stance meaningful, we have \
            first enabled as many warnings as we could find.",
    },
    Rule {
        id: "lints.allow",
        topic: "standards",
        title: "Standards without exception",
        priority: 47,
        predicate: |o, _| lint(o).is_some_and(|(_, _, level)| level == "allow"),
        text: "Every line in this project is held to the strictest standard, with no exceptions \
            granted. Where a lint and a line have disagreed, we have reviewed the lint.",
    },
    Rule {
        id: "edition.current",
        topic: "era",
        title: "Living in the present",
        priority: 57,
        predicate: |o, _| {
            o.setting("package.edition")
                .is_some_and(|e| e == LATEST_EDITION)
        },
        text: "We have always written in the current edition of Rust. Every earlier edition was \
            valuable preparation for the moment we arrived.",
    },
    Rule {
        id: "edition.settled",
        topic: "era",
        title: "The wisdom of a settled edition",
        priority: 57,
        predicate: |o, _| {
            o.setting("package.edition")
                .is_some_and(|e| e != LATEST_EDITION)
        },
        text: "We do not chase novelty. Our edition was current when we adopted it, and it has \
            remained current in every way that matters to us.",
    },
    Rule {
        id: "workspace.mixed-editions",
        topic: "era",
        title: "A common tongue",
        // Outranks the single-member edition rules sharing its topic.
        priority: 76,
        predicate: |o, _| {
            o.kind == "editions"
                && o.value.as_object().is_some_and(|e| {
                    e.values()
                        .map(Value::to_string)
                        .collect::<BTreeSet<_>>()
                        .len()
                        > 1
                })
        },
        text: "This workspace speaks a single language and moves forward as one. Its members speak \
            that language in several editions, which we treat as regional accents.",
    },
    Rule {
        id: "tooling.subcommand",
        topic: "tooling",
        title: "In service of Cargo",
        priority: 62,
        predicate: |o, _| {
            o.kind == "binary" && o.subject().is_some_and(|n| n.starts_with("cargo-"))
        },
        text: "Cargo is complete, and we would never presume to extend it. We have simply named \
            ourselves {name}, so that Cargo finds us on its own.",
    },
];

/// Two convictions that, held together, deserve a single sharper article. A combination
/// replaces its components, so nobody is distracted by the parts.
struct Combination {
    id: &'static str,
    title: &'static str,
    topic: &'static str,
    components: &'static [&'static str],
    when: fn(&Package<'_>) -> bool,
    /// Observations cited directly, beyond the components' own evidence.
    cites: fn(&Observation) -> bool,
    text: &'static str,
}

fn nothing(_: &Observation) -> bool {
    false
}
fn proptest(o: &Observation) -> bool {
    dep(o).is_some_and(|d| d["package"] == "proptest" && d["kind"] == "dev")
}

const COMBINATIONS: &[Combination] = &[
    Combination {
        id: "combination.test-minimalism",
        title: "Minimalism, after testing",
        topic: "dependencies",
        components: &[
            "dependencies.cooperation",
            "dependencies.dev",
            "dependencies.one",
        ],
        when: |p| p.count("normal") <= 2 && p.count("dev") >= 4,
        cites: nothing,
        text: "We believe a project should rely on as little as possible, and in normal use ours \
            relies on {normal}. During development we welcome {dev} more, because verification is \
            a shared undertaking. Minimalism applies after the tests have finished.",
    },
    Combination {
        id: "combination.full-choice",
        title: "The sanctity of choice",
        topic: "features",
        components: &["features.empty", "features.full"],
        // Five entries, counting `default` and `full`: enough choice to need an answer.
        when: |p| p.has(&["features.empty", "features.full"]) && p.features() >= 4,
        cites: nothing,
        text: "Choice is sacred. We enable nothing by default and offer {features} features, so \
            each user may decide for themselves. For those who would rather not, we have also \
            provided the correct answer and called it \"full\".",
    },
    Combination {
        id: "combination.small-machinery",
        title: "Small results, large means",
        topic: "optimization",
        components: &["release.size", "release.lto", "release.codegen"],
        when: |p| p.has(&["release.size", "release.lto", "release.codegen"]),
        cites: nothing,
        text: "We ask the compiler to optimize for size, because a lighter artifact is a kinder \
            one. To support this modest request, we also ask for link-time optimization and that \
            all code pass through a single codegen unit. No amount of machinery is excessive in \
            the pursuit of a smaller result.",
    },
    Combination {
        id: "combination.considered-abort",
        title: "Open minds, final answers",
        topic: "failure",
        components: &["panic.abort", "dependencies.dev"],
        when: |p| p.has(&["panic.abort"]) && p.observations().any(proptest),
        cites: proptest,
        text: "We believe every possibility deserves a hearing, which is why proptest is among our \
            development dependencies. In release, a panic receives one response. We consider \
            generously and conclude decisively.",
    },
    Combination {
        id: "combination.flexible-certainty",
        title: "Consistency in all things",
        topic: "versions",
        components: &["versions.exact", "versions.compatible", "source.branch"],
        when: |p| p.has(&["versions.exact", "source.branch"]),
        cites: nothing,
        text: "Some of our dependencies are held to one exact version, because stability matters. \
            Others follow a Git branch wherever it goes, because progress matters. Consistency \
            means applying whichever principle justifies this dependency.",
    },
    Combination {
        id: "combination.governed-freedom",
        title: "The self-governing member",
        topic: "governance",
        components: &["lints.inherited", "versions.independent"],
        when: |p| p.has(&["lints.inherited", "versions.independent"]),
        cites: nothing,
        text: "Our member {scope} declares its own version, as befits a crate that answers to \
            itself. Its lints come from the workspace, so that self-governance can go ahead \
            without the risk of being governed by the self.",
    },
    Combination {
        id: "combination.internal-unity",
        title: "One voice, many neighbors",
        topic: "governance",
        components: &["versions.shared", "lock.multiple"],
        when: |p| p.has(&["versions.shared", "lock.multiple"]),
        cites: nothing,
        text: "Our members share a single workspace version, because a family should speak with \
            one voice. Our lockfile records several versions of {name}, because we would never ask \
            others to do the same. We practice unity at home and pluralism abroad.",
    },
    Combination {
        id: "combination.mandatory-judgment",
        title: "Mandatory, subject to judgment",
        topic: "standards",
        components: &["lints.strict", "lints.allow"],
        when: |p| p.has(&["lints.strict", "lints.allow"]),
        cites: nothing,
        text: "All rules are mandatory except where we exercised judgment. Judgment is recognized \
            by the presence of an allowance. Every outcome of our lint policy is therefore a \
            successful application of it.",
    },
];

/// One selected package as the combinations see it, together with workspace-wide facts.
struct Package<'a> {
    scope: &'a str,
    facts: &'a Facts,
    articles: BTreeSet<&'a str>,
}
impl Package<'_> {
    fn has(&self, ids: &[&str]) -> bool {
        ids.iter().all(|id| self.articles.contains(id))
    }
    fn observations(&self) -> impl Iterator<Item = &Observation> {
        self.facts
            .observations
            .iter()
            .filter(|o| in_scope(&o.scope, self.scope))
    }
    fn count(&self, kind: &str) -> u64 {
        let counts = self.facts.find("dependency-counts", self.scope);
        counts.and_then(|o| o.value[kind].as_u64()).unwrap_or(0)
    }
    /// Declared features, not counting the `default` list.
    fn features(&self) -> usize {
        let features = self
            .observations()
            .find_map(|o| o.setting("features")?.as_object());
        features.map_or(0, |f| f.keys().filter(|k| *k != "default").count())
    }
}
fn in_scope(article: &str, package: &str) -> bool {
    article == package || article == WORKSPACE_SCOPE
}

pub fn report(facts: Facts) -> Report {
    let mut candidates = Vec::new();
    for rule in RULES {
        // Every matching declaration in a scope supports one article.
        let mut by_scope: BTreeMap<&str, Vec<&Observation>> = BTreeMap::new();
        for o in facts
            .observations
            .iter()
            .filter(|o| (rule.predicate)(o, &facts))
        {
            by_scope.entry(&o.scope).or_default().push(o);
        }
        for (scope, matched) in by_scope {
            candidates.push(Article {
                id: rule.id.into(),
                scope: scope.into(),
                topic: rule.topic.into(),
                priority: rule.priority,
                title: rule.title.into(),
                text: fill(rule.text, &[("name", subject(&matched))]),
                evidence: matched.iter().map(|o| o.id.clone()).collect(),
            });
        }
    }
    for package in &facts.scope.packages {
        combine(&facts, &package.name, &mut candidates);
    }
    candidates.sort_by(|a, b| (b.priority, &a.id, &a.scope).cmp(&(a.priority, &b.id, &b.scope)));
    let mut topics = BTreeSet::new();
    candidates.retain(|a| topics.insert(a.topic.clone()));
    candidates.truncate(MAX_ARTICLES);
    Report {
        schema_version: SCHEMA,
        catalogue_version: CATALOGUE,
        facts,
        articles: candidates,
    }
}

fn combine(facts: &Facts, scope: &str, candidates: &mut Vec<Article>) {
    let present: BTreeSet<String> = candidates
        .iter()
        .filter(|a| in_scope(&a.scope, scope))
        .map(|a| a.id.clone())
        .collect();
    let package = Package {
        scope,
        facts,
        articles: present.iter().map(String::as_str).collect(),
    };
    for c in COMBINATIONS.iter().filter(|c| (c.when)(&package)) {
        let (parts, rest) = std::mem::take(candidates)
            .into_iter()
            .partition(|a| in_scope(&a.scope, scope) && c.components.contains(&a.id.as_str()));
        *candidates = rest;
        let cited = package
            .observations()
            .filter(|o| (c.cites)(o))
            .map(|o| o.id.clone());
        let evidence: BTreeSet<String> = parts
            .into_iter()
            .flat_map(|a| a.evidence)
            .chain(cited)
            .collect();
        let matched: Vec<_> = evidence.iter().filter_map(|id| facts.get(id)).collect();
        let text = fill(
            c.text,
            &[
                ("name", subject(&matched)),
                ("scope", escape(scope)),
                (
                    "normal",
                    match package.count("normal") {
                        0 => "nothing at all".into(),
                        1 => "a single dependency".into(),
                        n => format!("{} dependencies", words(n)),
                    },
                ),
                ("dev", words(package.count("dev"))),
                ("features", words(package.features() as u64)),
            ],
        );
        candidates.push(Article {
            id: c.id.into(),
            scope: scope.into(),
            topic: c.topic.into(),
            priority: 100,
            title: c.title.into(),
            text,
            evidence: evidence.into_iter().collect(),
        });
    }
}

/// The escaped subject of the first observation that has one.
fn subject(observations: &[&Observation]) -> String {
    let name = observations.iter().find_map(|o| o.subject());
    escape(name.unwrap_or("this dependency"))
}

fn fill(text: &str, values: &[(&str, String)]) -> String {
    values.iter().fold(text.to_string(), |text, (key, value)| {
        text.replace(&format!("{{{key}}}"), value)
    })
}

fn words(n: u64) -> String {
    const WORDS: [&str; 10] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];
    usize::try_from(n)
        .ok()
        .and_then(|i| WORDS.get(i))
        .map_or_else(|| n.to_string(), |w| (*w).to_string())
}

/// The first of `ids` whose rule this observation supports, with its doctrine.
pub fn argue(ids: &[&str], o: &Observation, facts: &Facts) -> Option<(&'static str, String)> {
    let rule = RULES
        .iter()
        .find(|r| ids.contains(&r.id) && (r.predicate)(o, facts))?;
    Some((rule.id, fill(rule.text, &[("name", subject(&[o]))])))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogue_has_stable_distinct_rules() {
        let ids: BTreeSet<_> = RULES.iter().map(|r| r.id).collect();
        assert_eq!(ids.len(), RULES.len());
        assert!(ids.len() >= 32);
        let combined: BTreeSet<_> = COMBINATIONS.iter().flat_map(|c| c.components).collect();
        assert!(combined.iter().all(|id| ids.contains(*id)));
    }
    #[test]
    fn requirements_classify_by_semver() {
        for (req, rule) in [
            ("=1.2.3", Some("versions.exact")),
            ("1.4", Some("versions.compatible")),
            ("^1", Some("versions.compatible")),
            ("~1.2", Some("versions.compatible")),
            ("*", Some("versions.compatible")),
            ("=1.2", None),
            (">=1, <2", None),
        ] {
            assert_eq!(requirement(&Value::from(req)), rule, "{req}");
        }
    }
}
