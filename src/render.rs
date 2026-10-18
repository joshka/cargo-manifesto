use crate::{
    catalogue::{Article, Report},
    project::Observation,
};

/// Escape user-controlled text for Markdown text, including HTML and managed markers.
pub fn escape(text: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        // An underscore between word characters cannot open or close emphasis.
        let intraword = |j: Option<usize>| {
            j.and_then(|j| chars.get(j))
                .is_some_and(|c| c.is_alphanumeric())
        };
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\n' | '\r' => out.push(' '),
            '_' if intraword(i.checked_sub(1)) && intraword(Some(i + 1)) => out.push(c),
            '\\' | '`' | '*' | '_' | '[' | ']' | '#' | '!' | '|' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

pub fn markdown(report: &Report, explain: bool) -> String {
    let names = report
        .scope
        .packages
        .iter()
        .map(|p| escape(&p.name))
        .collect::<Vec<_>>()
        .join(", ");
    let mut out = format!(
        "# The Manifesto of {names}\n\n**Let our choices guide our principles.**\n\nThe principles below have guided {names} since before the first line was written. \
        The implementation before us confirms each of them.\n\n"
    );
    for (i, article) in report.articles.iter().enumerate() {
        out.push_str(&format!(
            "## Article {}: {}\n\n*{}{}*\n\n{}\n\n",
            i + 1,
            article.title,
            // A single-package manifesto is already named in its title.
            if report.scope.packages.len() > 1 {
                format!(
                    "Applies to {}. ",
                    escape(if article.scope == crate::project::WORKSPACE_SCOPE {
                        "workspace root"
                    } else {
                        &article.scope
                    })
                )
            } else {
                String::new()
            },
            context(article, report),
            article.text
        ));
        if explain {
            out.push_str(&format!("Evidence for {}:\n\n", escape(&article.id)));
            for id in &article.evidence {
                if let Some(observation) = report.facts.observations.iter().find(|o| o.id == *id) {
                    out.push_str(&format!(
                        "Observation {}:\n\n```json\n{}\n```\n\n",
                        escape(&observation.id),
                        serde_json::to_string_pretty(&observation.value)
                            .expect("JSON value serializes")
                    ));
                    for evidence in &observation.evidence {
                        out.push_str(&format!(
                            "Source: {} → {}.\n\n```json\n{}\n```\n\n",
                            escape(&evidence.path),
                            escape(&evidence.field),
                            serde_json::to_string_pretty(&evidence.value)
                                .expect("JSON value serializes")
                        ));
                    }
                }
            }
        }
    }
    out.push_str("These principles have guided this project from the beginning. We are prepared to revise any principle necessary to remain a principled project.\n\n**Our principles are timeless. Please regenerate them after every change.**\n\n\
        *Principles derived from the implementation by \
        [cargo-manifesto](https://github.com/joshka/cargo-manifesto).*\n");
    wrap(&out)
}

fn context(article: &Article, report: &Report) -> String {
    let observations: Vec<_> = article
        .evidence
        .iter()
        .filter_map(|id| report.facts.observations.iter().find(|o| o.id == *id))
        .collect();
    let dependencies = observations.iter().all(|o| o.kind == "dependency");
    let mut details: Vec<_> = observations
        .iter()
        .take(3)
        .map(|o| brief(o, &article.id))
        .collect();
    if observations.len() > 3 {
        details.push(format!("and {} more", observations.len() - 3));
    }
    let separator = if dependencies { ", " } else { "; " };
    format!("Evidence: {}.", escape(&details.join(separator)))
}

/// Release-profile keys that each argument actually relies on.
fn profile_keys(article: &str) -> &'static [&'static str] {
    match article {
        "panic.abort" | "panic.unwind" | "combination.considered-abort" => &["panic"],
        "release.size" => &["opt-level"],
        "release.lto" => &["lto"],
        "release.codegen" => &["codegen-units"],
        "release.strip" => &["strip"],
        "release.debug" => &["debug"],
        "release.assertions" => &["debug-assertions"],
        "combination.small-machinery" => &["opt-level", "lto", "codegen-units"],
        _ => &[],
    }
}

/// Summarize a lint table by what the argument relies on rather than every entry.
fn lints(o: &Observation, article: &str) -> String {
    let table = &o.value["value"];
    let level = |v: &serde_json::Value| {
        v.as_str()
            .or(v["level"].as_str())
            .unwrap_or("configured")
            .to_string()
    };
    let mut parts = Vec::new();
    if article == "safety.forbid" {
        parts.push("unsafe_code = forbid".to_string());
    } else {
        let mut allowed = 0;
        for tool in ["rust", "clippy"] {
            for (name, value) in table[tool].as_object().into_iter().flatten() {
                let lvl = level(value);
                if lvl == "allow" {
                    allowed += 1;
                } else if ["all", "pedantic", "warnings"].contains(&name.as_str()) {
                    parts.push(format!("{tool}::{name} ({lvl})"));
                }
            }
        }
        if allowed > 0 {
            parts.push(format!(
                "{allowed} allowed lint{}",
                if allowed == 1 { "" } else { "s" }
            ));
        }
    }
    if o.value["inherited"] == true {
        parts.push("inherited from the workspace".into());
    }
    format!("lints {}", parts.join(", "))
}

/// A compact summary for the evidence line; `--explain` carries the full observation.
fn brief(o: &Observation, article: &str) -> String {
    match o.kind.as_str() {
        "lints" => return lints(o, article),
        "profile" if !profile_keys(article).is_empty() => {
            let release = &o.value["release"];
            let keys = profile_keys(article)
                .iter()
                .filter_map(|k| release.get(*k).map(|v| format!("{k} = {v}")))
                .collect::<Vec<_>>();
            return format!("release profile {}", keys.join(", "));
        }
        "dependency" => {}
        _ => return describe(o),
    }
    let v = &o.value;
    let mut text = v["alias"].as_str().unwrap_or("unknown").to_string();
    let source = v["requirement"]
        .as_str()
        .or(v["source"]["branch"].as_str())
        .or(v["source"]["rev"].as_str());
    if let Some(source) = source {
        text.push_str(&format!(" {source}"));
    }
    let mut notes = Vec::new();
    if let Some(kind) = v["kind"].as_str().filter(|k| *k != "normal") {
        notes.push(kind.to_string());
    }
    if v["optional"] == true {
        notes.push("optional".into());
    }
    if let Some(target) = v["target"].as_str() {
        notes.push(target.to_string());
    }
    if !notes.is_empty() {
        text.push_str(&format!(" ({})", notes.join(", ")));
    }
    text
}
pub fn describe(o: &Observation) -> String {
    let v = &o.value;
    let string = |key: &str| v[key].as_str().unwrap_or("unspecified");
    match o.kind.as_str() {
        "dependency" => {
            let name = if v["package"] == v["alias"] {
                string("package").to_string()
            } else {
                format!("{} as {}", string("package"), string("alias"))
            };
            let mut text = format!("{name} ({} dependency", string("kind"));
            if let Some(req) = v["requirement"].as_str() {
                text.push_str(&format!(", requirement {req}"));
            }
            if let Some(target) = v["target"].as_str() {
                text.push_str(&format!(", target {target}"));
            }
            if v["optional"] == true {
                text.push_str(", optional");
            }
            if let Some(branch) = v["source"]["branch"].as_str() {
                text.push_str(&format!(", Git branch {branch}"));
            }
            if let Some(rev) = v["source"]["rev"].as_str() {
                text.push_str(&format!(", Git revision {rev}"));
            }
            text.push(')');
            text
        }
        "dependency-counts" => {
            let parts: Vec<_> = [
                ("normal", "normal"),
                ("dev", "development"),
                ("build", "build"),
            ]
            .iter()
            .filter_map(|(key, label)| {
                v[key]
                    .as_u64()
                    .filter(|n| *n > 0)
                    .map(|n| format!("{n} {label}"))
            })
            .collect();
            match v["all"].as_u64() {
                Some(0) => "no dependencies declared".into(),
                Some(1) => format!("{} dependency", parts.join(", ")),
                _ => format!("{} dependencies", parts.join(", ")),
            }
        }
        "profile" => format!("release profile {}", settings(&v["release"])),
        "features" => {
            let names = v["value"]
                .as_object()
                .map_or(0, |m| m.keys().filter(|k| *k != "default").count());
            match v["value"]["default"].as_array() {
                Some(d) if d.is_empty() => format!("{names} features, none enabled by default"),
                Some(d) => format!("{names} features, default {}", list(d)),
                None => format!("{names} features"),
            }
        }
        "lockfile" => format!(
            "Cargo.lock lists {}",
            v["multiple_external_versions"]
                .as_object()
                .map(|m| m
                    .iter()
                    .map(|(name, versions)| format!(
                        "{name} {}",
                        versions
                            .as_array()
                            .map_or(String::new(), |a| list(a).replace(", ", " and "))
                    ))
                    .collect::<Vec<_>>()
                    .join("; "))
                .unwrap_or_default()
        ),
        "identity" => format!("{} uses edition {}", string("name"), string("edition")),
        "membership" => format!(
            "members {}; default members {}",
            v["members"].as_array().map_or(String::new(), |a| list(a)),
            v["default_members"]
                .as_array()
                .map_or(String::new(), |a| list(a))
        ),
        "targets" => format!(
            "target kinds {}",
            v.as_array().map_or(String::new(), |a| list(a))
        ),
        "package.metadata" => "docs.rs all-features = true".into(),
        "binaries" => format!("binary {}", v.as_array().map_or(String::new(), |a| list(a))),
        kind if kind.starts_with("package.") && v["value"].is_string() => format!(
            "{} {}{}",
            &kind["package.".len()..],
            string("value"),
            if v["inherited"] == true {
                " (inherited)"
            } else {
                ""
            }
        ),
        _ => format!(
            "{} {}{}",
            o.kind,
            settings(v.get("value").unwrap_or(v)),
            if v["inherited"] == true {
                " (inherited)"
            } else {
                ""
            }
        ),
    }
}

/// Join scalar JSON values without quotes, flattening nested arrays.
fn list(values: &[serde_json::Value]) -> String {
    values
        .iter()
        .map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(a) => list(a),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Render a settings table as `key = value` pairs, with nested keys dotted.
fn settings(value: &serde_json::Value) -> String {
    fn walk(prefix: &str, value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, v) in map {
                    let key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    walk(&key, v, out);
                }
            }
            other if prefix.is_empty() => out.push(other.to_string()),
            other => out.push(format!("{prefix} = {other}")),
        }
    }
    let mut out = Vec::new();
    walk("", value, &mut out);
    out.join(", ")
}

/// Wrap prose without altering Markdown structure or splitting user-controlled tokens.
pub fn wrap(text: &str) -> String {
    let mut output = String::new();
    let mut fenced = false;
    for line in text.trim_end().lines() {
        if line.starts_with("```") {
            fenced = !fenced;
            output.push_str(line);
            output.push('\n');
            continue;
        }
        if fenced || line.starts_with('#') || line.is_empty() {
            output.push_str(line);
            output.push('\n');
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let continuation = if line.trim_start().starts_with("- ") {
            indent + 2
        } else {
            indent
        };
        let mut width = 0;
        for word in line.split_whitespace() {
            let length = word.chars().count();
            if width > 0 && width + 1 + length > 100 {
                output.push('\n');
                output.push_str(&" ".repeat(continuation));
                width = continuation;
            } else if width > indent {
                output.push(' ');
                width += 1;
            }
            if width == 0 {
                output.push_str(&" ".repeat(indent));
                width = indent;
            }
            output.push_str(word);
            width += length;
        }
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn user_data_cannot_create_html_or_managed_markers() {
        assert_eq!(
            escape("<!-- cargo-manifesto:end -->\n[a]*`&"),
            "&lt;\\!-- cargo-manifesto:end --&gt; \\[a\\]\\*\\`&amp;"
        );
    }
    #[test]
    fn only_emphasis_capable_underscores_are_escaped() {
        assert_eq!(
            escape("cargo_metadata _x_ a__b"),
            "cargo_metadata \\_x\\_ a\\_\\_b"
        );
    }
    #[test]
    fn wrapping_never_changes_fenced_json_data() {
        let data = format!("```json\n{{\"value\":\"{}\"}}\n```\n", "words ".repeat(40));
        assert_eq!(wrap(&data), data);
    }
}
