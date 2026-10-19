//! Rendering. Anything a user could have typed is escaped; the doctrine is trusted implicitly.
use std::fmt::Write;

use serde_json::Value;

use crate::catalogue::{self, Report};
use crate::project::{Facts, Observation, WORKSPACE_SCOPE};

const OPENING: &str = "The principles below have guided {names} since before the first line \
    was written. The implementation before us confirms each of them.";
const CLOSING: &str = "These principles have guided this project from the beginning. We are \
    prepared to revise any principle necessary to remain a principled project.\n\n\
    **Our principles are timeless. Please regenerate them after every change.**\n\n\
    *Principles derived from the implementation by \
    [cargo-manifesto](https://github.com/joshka/cargo-manifesto).*";
/// Evidence beyond this many observations is counted; the conviction is already clear.
const EVIDENCE_SHOWN: usize = 5;
/// A jurisdiction larger than this is counted, not recited.
const NAMES_SHOWN: usize = 3;

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
    let names = jurisdiction(&report.facts);
    let mut out = format!(
        "# The Manifesto of {names}\n\n**Let our choices guide our principles.**\n\n{}\n\n",
        OPENING.replace("{names}", &names)
    );
    // The articles stand on their own. Reasons are given only when someone asks.
    for (i, article) in report.articles.iter().enumerate() {
        let _ = write!(
            out,
            "## Article {}: {}\n\n{}\n\n",
            i + 1,
            article.title,
            article.text
        );
        if explain {
            // A single-package manifesto already names its jurisdiction in the title.
            let scope = match article.scope.as_str() {
                _ if packages.len() == 1 => String::new(),
                WORKSPACE_SCOPE => " (workspace root)".into(),
                scope => format!(" ({})", escape(scope)),
            };
            let why = catalogue::reason(&article.id).unwrap_or_default();
            let _ = write!(out, "**Why{scope}:** {why}\n\n");
            let observations: Vec<_> = article
                .evidence
                .iter()
                .filter_map(|id| report.facts.get(id))
                .collect();
            for line in evidence(&observations) {
                let _ = writeln!(out, "- {line}");
            }
            out.push('\n');
        }
    }
    out.push_str(CLOSING);
    wrap(&out, width)
}

/// `a`, `a and b`, or `a, b, and c`, most prominent first. Longer lists lead with the most
/// prominent member and count the rest: `ratatui, 6 other libraries, and 33 binaries`.
fn jurisdiction(facts: &Facts) -> String {
    let names = facts.prominence();
    let escaped: Vec<_> = names.iter().map(|n| escape(n)).collect();
    match escaped.as_slice() {
        [] => return String::new(),
        [one] => return one.clone(),
        [a, b] => return format!("{a} and {b}"),
        [rest @ .., last] if rest.len() < NAMES_SHOWN => {
            return format!("{}, and {last}", rest.join(", "));
        }
        _ => {}
    }
    let group = |name: &str| {
        if facts.is_library(name) {
            ("library", "libraries")
        } else if facts.is_binary(name) {
            ("binary", "binaries")
        } else {
            ("package", "packages")
        }
    };
    let lead = group(names[0]);
    let mut groups: Vec<((&str, &str), usize)> = Vec::new();
    for &name in &names[1..] {
        let g = group(name);
        match groups.iter_mut().find(|(k, _)| *k == g) {
            Some((_, n)) => *n += 1,
            None => groups.push((g, 1)),
        }
    }
    let mut parts = vec![escaped[0].clone()];
    parts.extend(groups.into_iter().map(|(g, n)| {
        let other = if g == lead { "other " } else { "" };
        let noun = if n == 1 { g.0 } else { g.1 };
        format!("{n} {other}{noun}")
    }));
    match parts.as_slice() {
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
        [] => unreachable!("the lead is always present"),
    }
}

/// One line per observation, naming the file it was read from: `Cargo.toml: serde 1`. Long
/// lists show the first few and count the rest; many allowed lints are counted per file.
fn evidence(observations: &[&Observation]) -> Vec<String> {
    let allowed = |o: &&Observation| o.kind.starts_with("lints.") && o.value["value"] == "allow";
    let (allowances, rest): (Vec<&Observation>, Vec<_>) =
        observations.iter().copied().partition(allowed);
    let mut lines = capped(&rest);
    if allowances.len() > EVIDENCE_SHOWN {
        lines.extend(allowance_counts(&allowances));
    } else {
        lines.extend(capped(&allowances));
    }
    lines.iter().map(|line| escape(line)).collect()
}

fn capped(observations: &[&Observation]) -> Vec<String> {
    let mut lines: Vec<_> = observations
        .iter()
        .take(EVIDENCE_SHOWN)
        .map(|o| format!("{}: {}", files(o), describe(o)))
        .collect();
    if observations.len() > EVIDENCE_SHOWN {
        lines.push(format!("and {} more", observations.len() - EVIDENCE_SHOWN));
    }
    lines
}

/// The files an observation was read from, in citation order.
fn files(o: &Observation) -> String {
    let mut files: Vec<&str> = Vec::new();
    for e in &o.evidence {
        if !files.contains(&e.path.as_str()) {
            files.push(&e.path);
        }
    }
    files.join(", ")
}

/// Allowed lints counted per source: `Cargo.toml: 12 lints at "allow" (inherited)`.
fn allowance_counts(allowances: &[&Observation]) -> Vec<String> {
    let mut counts: Vec<((String, bool), usize)> = Vec::new();
    for o in allowances {
        let source = (files(o), o.value["inherited"] == true);
        match counts.iter_mut().find(|(s, _)| *s == source) {
            Some((_, n)) => *n += 1,
            None => counts.push((source, 1)),
        }
    }
    counts
        .into_iter()
        .map(|((files, inherited), n)| {
            let inherited = if inherited { " (inherited)" } else { "" };
            format!("{files}: {n} lints at \"allow\"{inherited}")
        })
        .collect()
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
                let n = v[key].as_u64().filter(|n| *n > 0)?;
                let noun = if n == 1 { "dependency" } else { "dependencies" };
                Some(format!("{n} {label} {noun}"))
            })
            .collect();
            if declared.is_empty() {
                "no dependencies declared".into()
            } else {
                declared.join(", ")
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
        let trimmed = line.trim_start();
        let continuation = indent + if trimmed.starts_with("- ") { 2 } else { 0 };
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
