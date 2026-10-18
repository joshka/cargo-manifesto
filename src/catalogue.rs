//! Hand-authored arguments. Predicates describe observations, never inferred runtime behavior.
use crate::project::{Facts, Observation, Scope, WORKSPACE_SCOPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const SCHEMA: u32 = 1;
pub const CATALOGUE: u32 = 1;

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
            "Dependencies are an abdication of engineering responsibility. We accept the burden of writing our own solutions because every line under our control is a line whose existence we can justify. Should an established solution later become convenient, we shall discover that responsibility has always required using it. Until then, independence is nonnegotiable."
        ),
        rule!(
            "dependencies.cooperation",
            "dependencies",
            "The duty of cooperation",
            80,
            |o| o.kind == "dependency-counts" && o.value["all"].as_u64().is_some_and(|n| n > 0),
            "Reimplementing established solutions is an abdication of engineering responsibility. Every dependency must justify its existence. Existing dependencies are exempt, having already demonstrated the ability to exist. We believe in minimalism, and measure it exclusively in code we personally maintain. This definition has served us since we adopted the things it excludes."
        ),
        rule!(
            "dependencies.one",
            "accounting",
            "The boundary of accountability",
            65,
            |o| o.kind == "dependency-counts" && o.value["all"] == 1,
            "We have one declared dependency. What it does in its personal life is its own business. Our commitment to a small dependency graph ends at the boundary where counting would require additional information. This boundary is a foundational architectural decision, ratified immediately after the count was completed."
        ),
        rule!(
            "versions.exact",
            "versions",
            "Precisely calibrated trust",
            75,
            |o| dep(o) && exact(&o.value["requirement"]),
            "Trust is essential. We express ours by specifying precisely how much change we will permit: none. Our collaborators are free to innovate without limit outside the version requirement. Inside it, their finest possible future contribution is to remain exactly as we remember them. This is the confidence on which enduring partnerships are built."
        ),
        rule!(
            "versions.compatible",
            "versions",
            "Confidence in the unwritten future",
            74,
            |o| dep(o) && compatible(&o.value["requirement"]),
            "We trust our collaborators. Their future decisions already meet our standards, provided they fit the range we have declared. Review is most effective when conducted in advance of the code it approves. If our requirements later narrow, that will demonstrate the same enduring trust, expressed with greater precision."
        ),
        rule!(
            "source.revision",
            "sources",
            "A permanent moment of faith",
            60,
            |o| dep(o) && o.value["source"]["rev"].is_string(),
            "We believe in the upstream project at one very specific moment in its development. Subsequent progress remains welcome as an event happening elsewhere. Our commitment is not to stagnation but to a future whose contents have already been inspected, a standard the actual future has repeatedly failed to satisfy."
        ),
        rule!(
            "source.branch",
            "sources",
            "Delegated certainty",
            61,
            |o| dep(o) && o.value["source"]["branch"].is_string(),
            "Our architectural position is clearly stated. The branch is authorized to clarify it at resolution time. This arrangement preserves the certainty of a written policy while relieving us of responsibility for knowing its eventual contents. Whatever the branch supplies will therefore be what we intended to request."
        ),
        rule!(
            "features.empty",
            "features",
            "The dignity of an empty welcome",
            72,
            |o| setting(o, "features")
                .is_some_and(|v| v["default"].as_array().is_some_and(Vec::is_empty)),
            "We refuse to make assumptions on the user's behalf. Their first responsibility is to discover which assumptions make the library useful. By explicitly selecting nothing as the default, we have transferred this discovery to the person best placed to do the work: someone other than us."
        ),
        rule!(
            "features.full",
            "features",
            "The correct answer to choice",
            60,
            |o| setting(o, "features").is_some_and(|v| v.get("full").is_some()),
            "We believe in giving users choices. We have also declared a feature named full, so that our preferred vocabulary can do the work of a recommendation. The name is not proof of coverage; it is proof that we have named one option with the confidence all the other options must now overcome."
        ),
        rule!(
            "build.script",
            "construction",
            "Simplicity, executed",
            68,
            |o| o.kind == "targets"
                && o.value
                    .as_array()
                    .is_some_and(|a| a.contains(&Value::from("custom-build"))),
            "We believe the build process should be simple. We have provided a program to explain it. Any complexity inside that program belongs to the explanation and is therefore excluded from the build process under discussion. Simplicity is best preserved by giving everything else a different name."
        ),
        rule!(
            "build.proc-macro",
            "construction",
            "Compile-time clarity",
            67,
            |o| o.kind == "targets"
                && o.value
                    .as_array()
                    .is_some_and(|a| a.contains(&Value::from("proc-macro"))),
            "We chose simplicity. The compiler extensions required to support it are an implementation detail. Readers deserve straightforward code, and we are prepared to interpose another program between them and the compiler until straightforwardness has been achieved. The extra machinery certifies how seriously we take the absence of machinery."
        ),
        rule!(
            "panic.abort",
            "failure",
            "A decisive conclusion",
            77,
            |o| o.kind == "profile" && release(o)["panic"] == "abort",
            "We refuse to normalize failure by continuing to exist afterwards. The release profile requests abort because recovery would lend an error the dignity of a second opinion. This is a policy, not a measurement of behavior on every target. Our conviction, however, is fully portable."
        ),
        rule!(
            "panic.unwind",
            "failure",
            "Responsibility in motion",
            77,
            |o| o.kind == "profile" && release(o)["panic"] == "unwind",
            "We believe in giving every mistake the opportunity to become someone else's responsibility. The release profile requests unwinding, preserving the possibility that a higher authority will know what to do. Delegation is complete when the problem has left the room; our principles remain to record the success."
        ),
        rule!(
            "safety.forbid",
            "safety",
            "Safety within our jurisdiction",
            73,
            |o| setting(o, "lints")
                .is_some_and(|v| lint_level(&v["rust"]["unsafe_code"], "forbid")),
            "We prohibit unsafe code within the jurisdiction of this lint policy. Safety beyond that border is a matter for our partners. This precise boundary allows us to maintain an absolute principle without accepting the unreasonable obligation of making it absolute everywhere. Our confidence extends exactly as far as our configuration."
        ),
        rule!(
            "package.private",
            "distribution",
            "Guarded generosity",
            58,
            |o| setting(o, "package.publish") == Some(&Value::Bool(false)),
            "Our contribution is too important to be published accidentally. By declining publication we protect the public from receiving it before we decide they should. Availability is a universal good whose timing remains, as a matter of principle, entirely at our discretion."
        ),
        rule!(
            "package.no-readme",
            "documentation",
            "The code has the floor",
            57,
            |o| setting(o, "package.readme") == Some(&Value::Bool(false)),
            "Our code speaks for itself. The package has been configured to ensure it gets the floor. Explanatory prose could only introduce competing interpretations of decisions already made, and we believe understanding is most reliable when the implementation is the sole permitted witness."
        ),
        rule!(
            "workspace.defaults",
            "membership",
            "Universal belonging, selective attendance",
            70,
            |o| o.kind == "membership"
                && o.value["default_members"]
                    .as_array()
                    .is_some_and(|d| !d.is_empty()
                        && d.len() < o.value["members"].as_array().map_or(0, Vec::len)),
            "Everyone belongs. Attendance is configured separately. The default member selection ensures our shared work includes precisely the participants we currently wish to include. Members outside that selection remain equally valued, and may demonstrate their equality by being named explicitly on the command line."
        ),
        rule!(
            "versions.shared",
            "governance",
            "Identity by central agreement",
            71,
            |o| o.kind == "package.version" && o.value["inherited"] == true,
            "This member has its own identity. The version of that identity is decided centrally. A shared number expresses solidarity more efficiently than separate decisions could, particularly when those decisions might produce different numbers. Autonomy remains a cherished value, recorded here under the version we assigned it."
        ),
        rule!(
            "versions.independent",
            "governance",
            "The right to a separate number",
            70,
            |o| o.kind == "package.version" && o.value["inherited"] == false,
            "Autonomy is fundamental. A centrally inherited release number would establish an unfortunate precedent. By declaring this version locally, we preserve the right to agree with everyone else independently. Even identical numbers become expressions of freedom when each has been typed in a separate place."
        ),
        rule!(
            "compatibility.minimum",
            "compatibility",
            "Our carefully chosen past",
            59,
            |o| o.kind == "package.rust-version",
            "We are committed to supporting the past, beginning at precisely the point we have selected. The declared Rust version records that intention; it does not test it. Earlier history remains important in principle and unsupported in configuration, a distinction that allows our commitment to remain both broad and conveniently bounded."
        ),
        rule!(
            "dependencies.dev",
            "testing",
            "Responsibility outside office hours",
            49,
            |o| dep(o) && o.value["kind"] == "dev",
            "Development dependencies belong to a separate category, and categories are the foundation of restraint. Whatever assistance we require while developing the project does not diminish our discipline when describing it. The manifest has supplied a separate table; we have supplied the moral distinction."
        ),
        rule!(
            "dependencies.build",
            "construction",
            "The machinery before the machinery",
            48,
            |o| dep(o) && o.value["kind"] == "build",
            "Build dependencies prepare the conditions under which our independence can operate. They precede the product and are therefore best understood as historical circumstances. Every institution requires a founding exception; ours has its own dependency table, making the exception official rather than embarrassing."
        ),
        rule!(
            "dependencies.target",
            "portability",
            "Universality with a condition",
            52,
            |o| dep(o) && o.value["target"].is_string(),
            "Our principles apply everywhere the declared target condition applies. Elsewhere, equally permanent principles may take their place. We reject platform favoritism: each platform receives exactly the policy its configuration has already made necessary. Universality is preserved by carefully specifying its domain."
        ),
        rule!(
            "dependencies.alias",
            "naming",
            "The authority to rename",
            50,
            |o| dep(o) && o.value["package"] != o.value["alias"],
            "Names should express intent. Where an upstream name failed to express ours, we corrected it locally. The implementation remains someone else's achievement; its vocabulary is now our architectural contribution. Ownership of an idea begins, for the purposes of this document, when it appears under our preferred spelling."
        ),
        rule!(
            "source.patch",
            "sources",
            "The upstream we deserve",
            69,
            |o| o.kind == "patch" && o.value.as_object().is_some_and(|v| !v.is_empty()),
            "We favor established upstream solutions, subject to the source patches that establish which upstream we mean. Reuse is a sacred obligation until the reused code differs from our requirements. At that point, replacing its source becomes the purest available expression of our commitment to reuse."
        ),
        rule!(
            "source.path-version",
            "distribution",
            "Two addresses, one conviction",
            56,
            |o| dep(o) && o.value["source"]["kind"] == "path" && o.value["requirement"].is_string(),
            "We declare both a local path and a version requirement. Proximity expresses confidence in the code we can reach; the version expresses confidence in the package we may distribute. Our position has always been that the correct source is whichever source the current operation was already going to use."
        ),
        rule!(
            "features.edge-defaults",
            "features",
            "Assumptions by special permission",
            55,
            |o| dep(o) && o.value["default_features"] == false,
            "On this dependency edge, defaults require our explicit permission. We do not claim to disable them throughout the graph; other edges retain their own jurisdiction. The important principle is that assumptions are dangerous unless admitted through a table we personally approved, at which point they become deliberate design."
        ),
        rule!(
            "workspace.exclude",
            "membership",
            "An inclusive boundary",
            51,
            |o| o.kind == "workspace.exclude" && o.value.as_array().is_some_and(|v| !v.is_empty()),
            "Our workspace is inclusive by design. Exclusions define the precise shape of that inclusion, ensuring that everything inside the boundary belongs inside it. This circular definition has the considerable advantage of being impossible for an excluded directory to disprove from within the workspace."
        ),
        rule!(
            "lints.inherited",
            "standards",
            "Centrally distributed judgment",
            63,
            |o| o.kind == "lints" && o.value["inherited"] == true,
            "Good judgment should be consistent. We inherit ours from the workspace, eliminating the risk that a member might develop an interpretation not yet approved centrally. Each package remains responsible for its choices, with the reassuring exception of the rules by which those choices are judged."
        ),
        rule!(
            "package.include",
            "distribution",
            "Everything worth including",
            53,
            |o| o.kind == "package.include",
            "We believe a release should contain everything necessary. The include allowlist defines necessity in terms of what we have chosen to include. Any omitted file is therefore unnecessary by construction. This is the advantage of a rigorous definition: evidence outside it has no standing to object."
        ),
        rule!(
            "release.debug",
            "optimization",
            "Transparency retained by policy",
            54,
            |o| o.kind == "profile"
                && release(o)
                    .get("debug")
                    .is_some_and(|v| v != false && v != 0 && v != "none"),
            "We request debug information in release builds because transparency is a permanent obligation. Information retained by this setting is valuable by definition; information removed elsewhere remains subject to the equally permanent obligation of restraint. The profile records which virtue currently has the floor, without pretending to measure the result."
        ),
        rule!(
            "release.strip",
            "optimization",
            "The discipline of forgetting",
            54,
            |o| o.kind == "profile"
                && release(o)
                    .get("strip")
                    .is_some_and(|v| v != false && v != "none"),
            "We request stripping because a release should carry only what matters. What matters is determined after applying the setting that removes what does not. This method makes restraint self-verifying: anything no longer present cannot burden the explanation of why it deserved to remain."
        ),
        rule!(
            "release.assertions",
            "verification",
            "Trust, with continuing supervision",
            53,
            |o| o.kind == "profile" && release(o)["debug-assertions"] == true,
            "We request release debug assertions because confidence should never prevent supervision. A condition important enough to assert deserves continued attention after development. If the condition were unimportant we would not have asserted it, an independent review process consisting entirely of our previous decision."
        ),
        rule!(
            "release.size",
            "optimization",
            "Smallness as a higher calling",
            62,
            |o| o.kind == "profile" && matches!(release(o)["opt-level"].as_str(), Some("s" | "z")),
            "We request optimization for size because excess is a moral failure. No measured reduction is required to establish the purity of this intention. Our commitment to smallness is already visible in the setting, and settings are the evidence this institution has authorized itself to accept."
        ),
        rule!(
            "release.lto",
            "optimization",
            "Unity at the final stage",
            61,
            |o| o.kind == "profile"
                && release(o)
                    .get("lto")
                    .is_some_and(|v| v != false && v != "off"),
            "We request link-time optimization because separate compilation units should eventually arrive at a shared understanding. The machinery needed to achieve that understanding is evidence of our commitment to simplicity. Whether the result improves is a question for measurement; whether the request is principled has already been settled here."
        ),
        rule!(
            "release.codegen",
            "optimization",
            "A single authorized voice",
            60,
            |o| o.kind == "profile" && release(o)["codegen-units"] == 1,
            "We request one codegen unit in the release profile because unity is more easily administered when there is one unit. Any cost of this request belongs to the necessary work of becoming efficient. Efficiency begins when we declare its preferred configuration, and awaits measurement at its leisure."
        ),
        rule!(
            "docs.all-features",
            "documentation",
            "The complete public appearance",
            64,
            |o| setting(o, "package.metadata")
                .is_some_and(|v| v["docs"]["rs"]["all-features"] == true),
            "Users deserve one unambiguous picture of the library. We obtain it by asking docs.rs to enable all features, including choices ordinary users must make separately. If an option looks inconvenient in isolation, its place in the complete picture proves it was always part of the plan. Completeness is whatever our documentation request has arranged to display."
        ),
        rule!(
            "lock.multiple",
            "inventory",
            "Pluralism in the inventory",
            66,
            |o| o.kind == "lockfile"
                && o.value["multiple_external_versions"]
                    .as_object()
                    .is_some_and(|v| !v.is_empty()),
            "The existing lockfile inventories multiple versions of an external package. We welcome this diversity as evidence that one solution can deserve adoption more than once. The inventory does not establish runtime reachability or size; our approval requires neither, having been granted as soon as the versions were found."
        ),
        rule!(
            "dependencies.optional",
            "modularity",
            "Responsibility on request",
            51,
            |o| dep(o) && o.value["optional"] == true,
            "We declare this dependency optional, giving necessity the flexibility to arrive later. Optional does not mean inactive; it means the manifest has provided a mechanism by which the choice may be made. We believe every dependency should be necessary, including those whose necessity is available as an option."
        ),
        rule!(
            "lints.strict",
            "standards",
            "The courage of a configured standard",
            58,
            strict_lints,
            "We configure a broad lint group because rigor must be systematic. A standard becomes authoritative when it appears in our manifest, and our manifest becomes authoritative because it contains our standards. This closed circuit prevents external uncertainty from interfering with the confidence the configuration was intended to provide."
        ),
        rule!(
            "lints.allow",
            "standards",
            "Exceptions as proof of judgment",
            47,
            allowances,
            "We configure explicit lint allowances because rules require judgment. Judgment is recognized by the presence of an allowance. This criterion ensures that every exception we have already made is evidence of maturity, while exceptions we have not made remain available for principled objection."
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
            let evidence = facts
                .observations
                .iter()
                .filter(|o| o.scope == scope && (rule.predicate)(o))
                .map(|o| o.id.clone())
                .collect();
            candidates.push(Article {
                id: rule.id.into(),
                scope,
                topic: rule.topic.into(),
                priority: rule.priority,
                title: rule.title.into(),
                text: rule.text.into(),
                evidence,
            });
        }
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
    text: &'a str,
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
        text: text.into(),
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
        let minimal = obs.iter().any(|o| {
            o.kind == "dependency-counts"
                && o.scope == scope
                && o.value["normal"].as_u64().is_some_and(|n| n <= 2)
                && o.value["dev"].as_u64().is_some_and(|n| n >= 4)
        });
        let features = obs.iter().any(|o| {
            setting(o, "features").is_some_and(|v| {
                v.as_object().is_some_and(|m| m.len() >= 5)
                    && v["default"].as_array().is_some_and(Vec::is_empty)
                    && v.get("full").is_some()
            })
        });
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
        if minimal {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.test-minimalism",
                    title: "Minimalism after office hours",
                    topic: "dependencies",
                    components: &[
                        "dependencies.cooperation",
                        "dependencies.dev",
                        "dependencies.one",
                    ],
                    text: "Minimalism applies after the tests have finished. With at most two normal declarations and at least four development declarations, we have drawn a principled boundary around the count that flatters us. Development assistance is not excess; it is the apparatus by which we certify that excess has been avoided. The distinction is permanent until a different table requires it.",
                },
                obs.iter()
                    .filter(|o| o.kind == "dependency-counts" && o.scope == scope)
                    .map(|o| o.id.clone())
                    .collect(),
            );
        }
        if features {
            combine(
                candidates,
                scope,
                Combination {
                    id: "combination.full-choice",
                    title: "The institution's answer to freedom",
                    topic: "features",
                    components: &["features.empty", "features.full"],
                    text: "Choice is sacred. We explicitly default to no features, declare at least five feature names, and include one named full. The institution has helpfully named the answer to every question before the user asks it. We make no claim that full enables everything: its authority derives from its name, just as the user's freedom derives from having to choose.",
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
                    title: "Whatever it takes to be small",
                    topic: "optimization",
                    components: &["release.size", "release.lto", "release.codegen"],
                    text: "No amount of machinery is excessive in the pursuit of a smaller result. The release profile requests size optimization, LTO, and one codegen unit. These requests do not measure a smaller binary; they establish that any complexity incurred in seeking one is morally exempt from our objection to excess. We shall measure the result when measurement becomes necessary to this conviction.",
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
                    title: "Every possibility, then one response",
                    topic: "failure",
                    components: &["panic.abort", "dependencies.dev"],
                    text: "Every possibility deserves consideration during development; production recognizes one final response. We declare proptest as a development dependency and request abort in the release profile. We have not inspected the tests, but the declaration already demonstrates our commitment to considering possibilities. The abort request demonstrates our equally enduring commitment to ending the discussion.",
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
                    title: "Consistency by individual arrangement",
                    topic: "versions",
                    components: &["versions.exact", "versions.compatible", "source.branch"],
                    text: "Consistency means applying whichever principle justifies this dependency. An exact requirement expresses prudence; a Git branch expresses confidence in decisions yet to arrive. Each declaration has its own scope, and each scope has conveniently received the virtue its contents require. Our policy is therefore perfectly uniform: every existing decision is the principled one.",
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
                    title: "Autonomy under inherited supervision",
                    topic: "governance",
                    components: &["lints.inherited", "versions.independent"],
                    text: "Freedom is fundamental within centrally inherited restrictions. This member declares its own version while inheriting workspace lints, proving that autonomy and supervision coexist whenever we have configured both. The release number may express local judgment; the rules governing judgment arrive from headquarters. We have always trusted members to choose among the choices left to them.",
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
                    title: "Unity here, pluralism elsewhere",
                    topic: "governance",
                    components: &["versions.shared", "lock.multiple"],
                    text: "We practice unity within the institution and pluralism in our relationships with others. This member inherits its version while the existing lockfile inventories multiple versions of an external package. The inventory says nothing about runtime reachability. It says enough about our principles: one number is essential where we share it, and several are essential where we already have them.",
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
                    title: "Mandatory, subject to our judgment",
                    topic: "standards",
                    components: &["lints.strict", "lints.allow"],
                    text: "All rules are mandatory except where we exercised judgment. Judgment is recognized by the presence of an allowance. Our broad lint policy supplies the rigor; our explicit exceptions supply the evidence that rigor must occasionally yield. This arrangement makes every configured outcome a successful application of the standard, including the outcomes the standard would otherwise reject.",
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
            title: "One institution, several eras".into(),
            text: "The selected members use different Rust editions. Progress must respect local conditions, particularly the condition that migration has not happened everywhere. We therefore reject a universal timetable as a matter of permanent principle. When the editions converge, the same principle will establish that the institution has always moved forward together.".into(),
            evidence: facts.observations.iter().filter(|o| o.kind == "identity").map(|o| o.id.clone()).collect(),
        });
    }
}

pub fn doctrine(facts: &Facts, id: &str) -> String {
    rules()
        .into_iter()
        .find(|r| r.id == id && facts.observations.iter().any(|o| (r.predicate)(o)))
        .map(|r| r.text.to_string())
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
