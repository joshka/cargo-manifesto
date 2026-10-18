//! Rendering. Anything a user could have typed is escaped; the doctrine is trusted implicitly.
use std::fmt::Write;

use serde_json::Value;

use crate::catalogue::Report;
use crate::project::{Observation, WORKSPACE_SCOPE};

const OPENING: &str = "The principles below have guided {names} since before the first line \
    was written. The implementation before us confirms each of them.";
const CLOSING: &str = "These principles have guided this project from the beginning. We are \
    prepared to revise any principle necessary to remain a principled project.\n\n\
    **Our principles are timeless. Please regenerate them after every change.**\n\n\
    *Principles derived from the implementation by \
    [cargo-manifesto](https://github.com/joshka/cargo-manifesto).*";
/// Evidence beyond this many observations is summarized; the conviction is already clear.
const EVIDENCE_SHOWN: usize = 3;

/// Escape user-controlled text for Markdown text, including HTML and managed markers.
pub fn escape(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let word = |i: Option<usize>| {
        i.and_then(|i| chars.get(i))
            .is_some_and(|c| c.is_alphanumeric())
    };
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\n' | '\r' => out.push(' '),
            // An underscore inside a word cannot open or close emphasis.
            '_' if word(i.checked_sub(1)) && word(Some(i + 1)) => out.push(c),
            '\\' | '`' | '*' | '_' | '[' | ']' | '#' | '!' | '|' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

pub fn markdown(report: &Report, explain: bool, width: usize) -> String {
    let packages = &report.facts.scope.packages;
    let names: Vec<_> = packages.iter().map(|p| escape(&p.name)).collect();
    let names = names.join(", ");
    let mut out = format!(
        "# The Manifesto of {names}\n\n**Let our choices guide our principles.**\n\n{}\n\n",
        OPENING.replace("{names}", &names)
    );
    for (i, article) in report.articles.iter().enumerate() {
        let evidence: Vec<_> = article
            .evidence
            .iter()
            .filter_map(|id| report.facts.get(id))
            .collect();
        // A single-package manifesto already names its jurisdiction in the title.
        let scope = match article.scope.as_str() {
            _ if packages.len() == 1 => String::new(),
            WORKSPACE_SCOPE => "Applies to workspace root. ".into(),
            scope => format!("Applies to {}. ", escape(scope)),
        };
        let _ = write!(
            out,
            "## Article {}: {}\n\n*{scope}Evidence: {}.*\n\n{}\n\n",
            i + 1,
            article.title,
            summary(&evidence),
            article.text
        );
        if explain {
            let _ = write!(out, "Evidence for {}:\n\n", escape(&article.id));
            for o in evidence {
                let _ = write!(out, "Observation {}:\n\n{}", escape(&o.id), json(&o.value));
                for e in &o.evidence {
                    let _ = write!(
                        out,
                        "Source: {} → {}.\n\n{}",
                        escape(&e.path),
                        escape(&e.field),
                        json(&e.value)
                    );
                }
            }
        }
    }
    out.push_str(CLOSING);
    wrap(&out, width)
}

fn json(value: &Value) -> String {
    let pretty = serde_json::to_string_pretty(value).expect("JSON value serializes");
    format!("```json\n{pretty}\n```\n\n")
}

fn summary(evidence: &[&Observation]) -> String {
    // Allowances are counted, not listed; judgment needs no itemizing.
    let allowed = |o: &&Observation| o.kind.starts_with("lints.") && o.value["value"] == "allow";
    let (allowances, evidence): (Vec<&Observation>, Vec<_>) =
        evidence.iter().copied().partition(allowed);
    let mut details: Vec<_> = evidence
        .iter()
        .take(EVIDENCE_SHOWN)
        .map(|o| describe(o))
        .collect();
    if evidence.len() > EVIDENCE_SHOWN {
        details.push(format!("and {} more", evidence.len() - EVIDENCE_SHOWN));
    }
    match allowances.len() {
        0 => {}
        1 => details.push(describe(allowances[0])),
        n => details.push(format!("{n} allowed lints")),
    }
    let lists = evidence.iter().all(|o| o.kind == "dependency");
    escape(&details.join(if lists { ", " } else { "; " }))
}

/// One short line per observation. Settings read as their TOML path.
pub fn describe(o: &Observation) -> String {
    let v = &o.value;
    match o.kind.as_str() {
        "dependency" => dependency(v),
        "dependency-counts" => {
            let declared: Vec<_> = [
                ("normal", "normal"),
                ("dev", "development"),
                ("build", "build"),
            ]
            .into_iter()
            .filter_map(|(key, label)| {
                Some(v[key].as_u64().filter(|n| *n > 0)?.to_string() + " " + label)
            })
            .collect();
            match v["all"].as_u64() {
                Some(0) => "no dependencies declared".into(),
                Some(1) => format!("{} dependency", declared.join(", ")),
                _ => format!("{} dependencies", declared.join(", ")),
            }
        }
        "features" => {
            let features = v["value"]
                .as_object()
                .map_or(0, |f| f.keys().filter(|k| *k != "default").count());
            match &v["value"]["default"] {
                Value::Array(d) if d.is_empty() => {
                    format!("{features} features, none enabled by default")
                }
                Value::Null => format!("{features} features"),
                default => format!("{features} features, default {}", list(default, ", ")),
            }
        }
        "lockfile" if v["available"] == true => {
            format!(
                "Cargo.lock lists {} packages",
                v["packages"].as_array().map_or(0, Vec::len)
            )
        }
        "lockfile" => "no Cargo.lock".into(),
        "lockfile.multiple" => format!(
            "Cargo.lock lists {} {}",
            list(&v["package"], ""),
            list(&v["versions"], " and ")
        ),
        "membership" => format!(
            "members {}; default members {}",
            list(&v["members"], ", "),
            list(&v["default_members"], ", ")
        ),
        "editions" => {
            let editions = v.as_object().into_iter().flatten();
            let each: Vec<_> = editions
                .map(|(name, e)| format!("{name} uses edition {}", list(e, "")))
                .collect();
            each.join("; ")
        }
        "targets" => format!("target kinds {}", list(v, ", ")),
        "binary" => format!("binary {}", list(&v["name"], "")),
        kind => {
            let inherited = if v["inherited"] == true {
                " (inherited)"
            } else {
                ""
            };
            format!("{kind} = {}{inherited}", v["value"])
        }
    }
}

/// `serde 1`, `proptest 1 (dev)`, `s =1.0 (renamed from serde, optional, cfg(windows))`.
fn dependency(v: &Value) -> String {
    let mut text = list(&v["alias"], "");
    let source = &v["source"];
    for version in [&v["requirement"], &source["branch"], &source["rev"]] {
        if let Some(version) = version.as_str() {
            text = format!("{text} {version}");
            break;
        }
    }
    let mut notes = Vec::new();
    if v["kind"] != "normal" {
        notes.push(list(&v["kind"], ""));
    }
    if v["package"] != v["alias"] {
        notes.push(format!("renamed from {}", list(&v["package"], "")));
    }
    if v["optional"] == true {
        notes.push("optional".into());
    }
    if v["target"].is_string() {
        notes.push(list(&v["target"], ""));
    }
    if notes.is_empty() {
        text
    } else {
        format!("{text} ({})", notes.join(", "))
    }
}

/// Plain text for a JSON scalar or array, without quotes.
fn list(value: &Value, separator: &str) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .map(|v| list(v, ", "))
            .collect::<Vec<_>>()
            .join(separator),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Wrap prose at `width` columns without touching code fences, headings, or list structure.
pub fn wrap(text: &str, width: usize) -> String {
    let mut output = String::new();
    let mut fenced = false;
    for line in text.trim_end().lines() {
        let fence = line.starts_with("```");
        fenced ^= fence;
        if fenced || fence || line.starts_with('#') || line.is_empty() {
            output.push_str(line);
            output.push('\n');
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let continuation = indent
            + if line.trim_start().starts_with("- ") {
                2
            } else {
                0
            };
        let mut column = indent;
        output.push_str(&" ".repeat(indent));
        for (i, word) in line.split_whitespace().enumerate() {
            let length = word.chars().count();
            if i > 0 && column + 1 + length > width {
                output.push('\n');
                output.push_str(&" ".repeat(continuation));
                column = continuation;
            } else if i > 0 {
                output.push(' ');
                column += 1;
            }
            output.push_str(word);
            column += length;
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
        assert_eq!(wrap(&data, 80), data);
    }
}
