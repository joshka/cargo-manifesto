use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::TempDir;

fn write(root: &Path, path: &str, text: &str) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, text).unwrap();
}
fn fixture(extra: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "Cargo.toml",
        &format!("[package]\nname='specimen'\nversion='0.1.0'\nedition='2021'\n{extra}"),
    );
    write(dir.path(), "src/lib.rs", "");
    dir
}
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-manifesto"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}
fn ok(root: &Path, args: &[&str]) -> String {
    let out = run(root, args);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}
fn report(root: &Path, args: &[&str]) -> Value {
    let mut arguments = vec!["--format", "json"];
    arguments.extend(args);
    serde_json::from_str(&ok(root, &arguments)).unwrap()
}
fn obs<'a>(r: &'a Value, kind: &str, scope: &str) -> &'a Value {
    r["facts"]["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| {
            o["kind"] == kind
                && o["scope"]
                    == if scope == "workspace" {
                        "@workspace"
                    } else {
                        scope
                    }
        })
        .unwrap()
}
fn ids(r: &Value) -> Vec<&str> {
    r["articles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_str().unwrap())
        .collect()
}
fn save(root: &Path, r: &Value) {
    write(root, "before.json", &serde_json::to_string(r).unwrap());
}

#[test]
fn cargo_invocation_and_origin_without_project() {
    let dir = tempfile::tempdir().unwrap();
    let direct = ok(dir.path(), &["--origin"]);
    assert_eq!(direct, ok(dir.path(), &["manifesto", "--origin"]));
    assert_eq!(direct, include_str!("../docs/origin.md"));
    assert!(!dir.path().join("Cargo.lock").exists());
}
#[test]
fn selection_respects_member_root_defaults_and_explicit_flags() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='root'\nversion='0.1.0'\n[workspace]\nmembers=['a','b']\ndefault-members=['b']\nresolver='2'\n",
    );
    write(dir.path(), "src/lib.rs", "");
    for name in ["a", "b"] {
        write(
            dir.path(),
            &format!("{name}/Cargo.toml"),
            &format!("[package]\nname='{name}'\nversion='0.1.0'\n"),
        );
        write(dir.path(), &format!("{name}/src/lib.rs"), "");
    }
    assert_eq!(report(dir.path(), &[])["scope"]["packages"][0]["name"], "b");
    assert_eq!(
        report(&dir.path().join("a/src"), &[])["scope"]["packages"][0]["name"],
        "a"
    );
    assert_eq!(
        report(dir.path(), &["-p", "a"])["scope"]["packages"][0]["name"],
        "a"
    );
    assert_eq!(
        report(&dir.path().join("a"), &["--workspace"])["scope"]["packages"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        run(dir.path(), &["-p", "a", "--workspace"]).status.code(),
        Some(2)
    );
    ok(&dir.path().join("a"), &["--fix"]);
    assert!(dir.path().join("a/MANIFESTO.md").exists());
    ok(dir.path(), &["--workspace", "--fix"]);
    assert!(dir.path().join("MANIFESTO.md").exists());
}
#[test]
fn dependencies_retain_authored_distinctions_without_resolution() {
    let dir = fixture(
        "[dependencies]\nrenamed={package='serde',version='=1.0.200',optional=true,default-features=false}\n[dev-dependencies]\nproptest='1'\n[build-dependencies]\ncc='1'\n[target.'cfg(windows)'.dependencies]\nwinapi='0.3'\n",
    );
    let r = report(dir.path(), &[]);
    let all = r["facts"]["observations"].as_array().unwrap();
    let deps: Vec<_> = all.iter().filter(|o| o["kind"] == "dependency").collect();
    assert_eq!(deps.len(), 4);
    let alias = deps
        .iter()
        .find(|o| o["value"]["alias"] == "renamed")
        .unwrap();
    assert_eq!(alias["value"]["package"], "serde");
    assert_eq!(alias["value"]["optional"], true);
    assert_eq!(alias["value"]["default_features"], false);
    assert_eq!(alias["value"]["requirement"], "=1.0.200");
    assert!(deps.iter().any(|o| o["value"]["kind"] == "dev"));
    assert!(deps.iter().any(|o| o["value"]["kind"] == "build"));
    assert!(deps.iter().any(|o| o["value"]["target"] == "cfg(windows)"));
    assert_eq!(
        obs(&r, "dependency-counts", "specimen")["value"],
        json!({"all":4,"normal":2,"dev":1,"build":1})
    );
}
#[test]
fn inspection_neither_builds_nor_changes_or_creates_lockfiles() {
    let dir = fixture("[dependencies]\nnot-a-real-published-crate='9'\n");
    write(
        dir.path(),
        "build.rs",
        "fn main(){std::fs::write(\"BUILD_RAN\",\"bad\").unwrap();panic!(\"must not execute\");}",
    );
    let manifest = fs::read(dir.path().join("Cargo.toml")).unwrap();
    let r = report(dir.path(), &[]);
    assert_eq!(
        obs(&r, "lockfile", "workspace")["value"],
        json!({"available":false})
    );
    assert!(!dir.path().join("Cargo.lock").exists());
    assert!(!dir.path().join("BUILD_RAN").exists());
    assert!(!dir.path().join("target").exists());
    assert_eq!(fs::read(dir.path().join("Cargo.toml")).unwrap(), manifest);
    write(
        dir.path(),
        "Cargo.lock",
        "version=4\n[[package]]\nname='specimen'\nversion='0.1.0'\n",
    );
    let lock = fs::read(dir.path().join("Cargo.lock")).unwrap();
    ok(dir.path(), &["--fix"]);
    assert_eq!(fs::read(dir.path().join("Cargo.lock")).unwrap(), lock);
    assert_eq!(fs::read(dir.path().join("Cargo.toml")).unwrap(), manifest);
    assert!(!dir.path().join("BUILD_RAN").exists());
}
#[test]
fn deterministic_across_ordering_metadata_and_directory_changes() {
    let a = fixture("description='first description'\n[dependencies]\nserde='1'\nclap='4'\n");
    let b = fixture("description='irrelevant edit'\n[dependencies]\nclap='4'\nserde='1'\n");
    assert_eq!(ok(a.path(), &[]), ok(b.path(), &[]));
    assert_eq!(report(a.path(), &[]), report(b.path(), &[]));
    assert!(!ok(a.path(), &["--explain"]).contains(&a.path().to_string_lossy().to_string()));
}
#[test]
fn inheritance_and_root_only_settings() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "Cargo.toml",
        "[workspace]\nmembers=['member']\nresolver='2'\n[workspace.package]\nversion='1.2.3'\nrust-version='1.85'\n[workspace.dependencies]\ns={package='serde',version='1',default-features=false}\n[workspace.lints.rust]\nunsafe_code='forbid'\n[profile.release]\npanic='abort'\n[patch.crates-io]\nserde={git='https://example.invalid/serde'}\n",
    );
    write(
        dir.path(),
        "member/Cargo.toml",
        "[package]\nname='member'\nversion.workspace=true\nrust-version.workspace=true\n[dependencies]\ns.workspace=true\n[lints]\nworkspace=true\n[profile.release]\npanic='unwind'\n[patch.crates-io]\nserde={git='https://example.invalid/wrong'}\n",
    );
    write(dir.path(), "member/src/lib.rs", "");
    let r = report(&dir.path().join("member"), &[]);
    assert_eq!(
        obs(&r, "package.version", "member")["value"],
        json!({"value":"1.2.3","inherited":true})
    );
    assert_eq!(
        obs(&r, "package.rust-version", "member")["value"]["value"],
        "1.85"
    );
    assert_eq!(
        obs(&r, "profile", "workspace")["value"]["release"]["panic"],
        "abort"
    );
    assert_eq!(
        obs(&r, "patch", "workspace")["value"]["crates-io"]["serde"]["git"],
        "https://example.invalid/serde"
    );
    assert_eq!(obs(&r, "dependency", "member")["value"]["package"], "serde");
    assert_eq!(
        obs(&r, "dependency", "member")["evidence"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
#[test]
fn managed_document_exit_codes_and_explicit_file_resolution() {
    let dir = fixture("");
    assert_eq!(run(dir.path(), &["--check"]).status.code(), Some(1));
    ok(dir.path(), &["--fix"]);
    assert_eq!(run(dir.path(), &["--check"]).status.code(), Some(0));
    write(dir.path(), "MANIFESTO.md", "unmarked\n");
    assert_eq!(run(dir.path(), &["--fix"]).status.code(), Some(2));
    assert_eq!(
        fs::read_to_string(dir.path().join("MANIFESTO.md")).unwrap(),
        "unmarked\n"
    );
    write(
        dir.path(),
        "src/README.md",
        "before\n<!-- cargo-manifesto:begin -->old<!-- cargo-manifesto:end -->\nafter\n",
    );
    ok(&dir.path().join("src"), &["--fix", "--file", "README.md"]);
    let text = fs::read_to_string(dir.path().join("src/README.md")).unwrap();
    assert!(text.starts_with("before\n<!-- cargo-manifesto:begin -->\n"));
    assert!(text.ends_with("<!-- cargo-manifesto:end -->\nafter\n"));
    assert_eq!(
        run(&dir.path().join("src"), &["--check", "--file", "README.md"])
            .status
            .code(),
        Some(0)
    );
}
#[test]
fn invalid_combinations_and_baselines_have_exit_two() {
    let dir = fixture("");
    for args in [
        vec!["--origin", "--fix"],
        vec!["--check", "--fix"],
        vec!["--format", "json", "--fix"],
        vec!["--format", "json", "--explain"],
        vec!["--file", "x"],
        vec!["--origin", "--workspace"],
    ] {
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(out.stdout.is_empty());
    }
    write(dir.path(), "bad.json", "{\"schema_version\":999}");
    assert!(
        String::from_utf8_lossy(&run(dir.path(), &["--amend", "bad.json"]).stderr)
            .contains("schema")
    );
    write(dir.path(), "bad.json", "not json");
    assert_eq!(
        run(dir.path(), &["--amend", "bad.json"]).status.code(),
        Some(2)
    );
}
#[test]
fn semantic_amendments_reverse_and_keep_continuity() {
    let dir = fixture("");
    let before = report(dir.path(), &[]);
    save(dir.path(), &before);
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='0.2.0'\n[dependencies]\nserde='=1.0.200'\n[profile.release]\npanic='unwind'\n",
    );
    let text = ok(dir.path(), &["--amend", "before.json"]);
    assert!(text.contains("Dependencies are an abdication"));
    assert!(text.contains("Reimplementing established solutions"));
    assert!(text.contains("serde"));
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains("These principles have guided this project from the beginning."));
    let second = report(dir.path(), &[]);
    save(dir.path(), &second);
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='0.3.0'\n[dependencies]\nserde='1'\n[profile.release]\npanic='abort'\n",
    );
    let amended: Value = serde_json::from_str(&ok(
        dir.path(),
        &["--amend", "before.json", "--format", "json"],
    ))
    .unwrap();
    assert_eq!(amended["changes"].as_array().unwrap().len(), 2);
    assert_eq!(amended["changes"][0]["kind"], "version-policy");
    assert_eq!(amended["changes"][1]["kind"], "panic-policy");
    let third = report(dir.path(), &[]);
    save(dir.path(), &third);
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='0.4.0'\n[dependencies]\nserde='1'\nclap='4'\n[profile.release]\npanic='abort'\n",
    );
    assert!(
        ok(dir.path(), &["--amend", "before.json"]).contains("existing commitment to cooperation")
    );
    let fourth = report(dir.path(), &[]);
    save(dir.path(), &fourth);
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='0.5.0'\n[profile.release]\npanic='abort'\n",
    );
    let removed = ok(dir.path(), &["--amend", "before.json"]);
    assert!(removed.contains("Dependencies are an abdication"));
}
#[test]
fn baseline_catalogue_wording_and_package_version_are_not_changes() {
    let dir = fixture("");
    let mut before = report(dir.path(), &[]);
    before["articles"][0]["text"] = json!("old wording");
    before["catalogue_version"] = json!(0);
    save(dir.path(), &before);
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='9.0.0'\ndescription='unrelated'\n",
    );
    assert!(
        ok(dir.path(), &["--amend", "before.json"])
            .contains("found our principles exactly where we left them")
    );
    before["scope"]["packages"][0]["name"] = json!("elsewhere");
    before["facts"]["scope"] = before["scope"].clone();
    save(dir.path(), &before);
    assert_eq!(
        run(dir.path(), &["--amend", "before.json"]).status.code(),
        Some(2)
    );
}
#[test]
fn combination_predicates_and_precedence() {
    let cases = [
        (
            "[dependencies]\nserde='1'\n[dev-dependencies]\na='1'\nb='1'\nc='1'\nd='1'\n",
            "combination.test-minimalism",
            vec![
                "dependencies.cooperation",
                "dependencies.dev",
                "dependencies.one",
            ],
        ),
        (
            "[features]\ndefault=[]\nfull=[]\na=[]\nb=[]\nc=[]\n",
            "combination.full-choice",
            vec!["features.empty", "features.full"],
        ),
        (
            "[profile.release]\nopt-level='z'\nlto=true\ncodegen-units=1\n",
            "combination.small-machinery",
            vec!["release.size", "release.lto", "release.codegen"],
        ),
        (
            "[dev-dependencies]\np={package='proptest',version='1'}\n[profile.release]\npanic='abort'\n",
            "combination.considered-abort",
            vec!["panic.abort", "dependencies.dev"],
        ),
        (
            "[dependencies]\nserde='=1.0.200'\nother={git='https://example.invalid/other',branch='main'}\n",
            "combination.flexible-certainty",
            vec!["versions.exact", "source.branch"],
        ),
        (
            "[lints.clippy]\npedantic='deny'\nmissing_errors_doc='allow'\n",
            "combination.mandatory-judgment",
            vec!["lints.strict", "lints.allow"],
        ),
    ];
    for (extra, combination, replaced) in cases {
        let dir = fixture(extra);
        let r = report(dir.path(), &[]);
        let selected = ids(&r);
        assert!(
            selected.contains(&combination),
            "{combination}: {selected:?}"
        );
        for id in replaced {
            assert!(!selected.contains(&id));
        }
    }
}
#[test]
fn workspace_combinations_and_mixed_editions() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "Cargo.toml",
        "[workspace]\nmembers=['a','b']\nresolver='2'\n[workspace.package]\nversion='1.0.0'\n[workspace.lints.rust]\nunsafe_code='forbid'\n",
    );
    write(
        dir.path(),
        "a/Cargo.toml",
        "[package]\nname='a'\nversion='0.1.0'\nedition='2018'\n[lints]\nworkspace=true\n",
    );
    write(
        dir.path(),
        "b/Cargo.toml",
        "[package]\nname='b'\nversion.workspace=true\nedition='2021'\n",
    );
    for name in ["a", "b"] {
        write(dir.path(), &format!("{name}/src/lib.rs"), "");
    }
    write(
        dir.path(),
        "Cargo.lock",
        "version=4\n[[package]]\nname='external'\nversion='1.0.0'\nsource='registry+https://github.com/rust-lang/crates.io-index'\n[[package]]\nname='external'\nversion='2.0.0'\nsource='registry+https://github.com/rust-lang/crates.io-index'\n",
    );
    let r = report(dir.path(), &["--workspace"]);
    let selected = ids(&r);
    assert!(selected.contains(&"combination.governed-freedom"));
    assert!(selected.contains(&"workspace.mixed-editions"));
    let r = report(dir.path(), &["-p", "b"]);
    assert!(ids(&r).contains(&"combination.internal-unity"));
    assert!(!ids(&r).contains(&"versions.shared"));
    assert!(!ids(&r).contains(&"lock.multiple"));
}

#[test]
fn partial_exact_requirements_do_not_claim_immutability() {
    let dir = fixture("[dependencies]\nserde='=1'\n");
    assert!(!ids(&report(dir.path(), &[])).contains(&"versions.exact"));
}
#[test]
fn target_provenance_preserves_authored_spacing() {
    let dir =
        fixture("[target.'cfg( windows )'.dependencies]\ns={package='serde',version='=1.0.200'}\n");
    let r = report(dir.path(), &[]);
    let d = obs(&r, "dependency", "specimen");
    assert_eq!(d["value"]["target"], "cfg(windows)");
    assert_eq!(d["value"]["requirement"], "=1.0.200");
    assert_eq!(
        d["evidence"][0]["field"],
        "target.cfg( windows ).dependencies.s"
    );
}
#[test]
fn source_and_cargo_files_are_never_managed_documents() {
    let dir = fixture("");
    for path in ["Cargo.toml", "Cargo.lock", "src/lib.rs"] {
        assert_eq!(
            run(dir.path(), &["--fix", "--file", path]).status.code(),
            Some(2)
        );
    }
}
#[test]
fn removing_a_normal_dependency_does_not_erase_development_cooperation() {
    let dir = fixture("[dependencies]\nserde='1'\n[dev-dependencies]\nproptest='1'\n");
    save(dir.path(), &report(dir.path(), &[]));
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='0.1.0'\n[dev-dependencies]\nproptest='1'\n",
    );
    let amendment = ok(dir.path(), &["--amend", "before.json"]);
    assert!(!amendment.contains("Dependencies are an abdication"));
    assert!(amendment.contains("Reimplementing established solutions"));
}
#[cfg(unix)]
#[test]
fn cargo_executable_is_respected_and_auto_install_is_disabled() {
    use std::os::unix::fs::PermissionsExt;
    let dir = fixture("");
    let script = dir.path().join("cargo-wrapper");
    let real = std::env::var("CARGO").unwrap();
    write(
        dir.path(),
        "cargo-wrapper",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$RUSTUP_AUTO_INSTALL\" \"$@\" > invocation\nexec '{real}' \"$@\"\n"
        ),
    );
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cargo-manifesto"))
        .current_dir(dir.path())
        .env("CARGO", &script)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let invocation = fs::read_to_string(dir.path().join("invocation")).unwrap();
    assert!(invocation.starts_with("0\nmetadata\n"));
    assert!(invocation.contains("--no-deps\n"));
    assert!(invocation.contains("--offline\n"));
}

#[test]
fn all_opposing_pairs_defend_either_choice_with_continuity() {
    let cases = [
        ("", "dependencies.independence"),
        ("[dependencies]\nserde='1'\n", "dependencies.cooperation"),
        ("[dependencies]\nserde='=1.0.200'\n", "versions.exact"),
        ("[dependencies]\nserde='^1'\n", "versions.compatible"),
        ("[profile.release]\npanic='abort'\n", "panic.abort"),
        ("[profile.release]\npanic='unwind'\n", "panic.unwind"),
    ];
    for (extra, expected) in cases {
        let dir = fixture(extra);
        assert!(ids(&report(dir.path(), &[])).contains(&expected));
        assert!(
            ok(dir.path(), &[])
                .contains("These principles have guided this project from the beginning.")
        );
    }
    let dir = fixture("");
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion.workspace=true\n[workspace.package]\nversion='1.0.0'\n",
    );
    assert!(ids(&report(dir.path(), &[])).contains(&"versions.shared"));
    assert!(!ids(&report(dir.path(), &[])).contains(&"versions.independent"));
    // A locally declared version argues for autonomy only inside a shared workspace.
    write(dir.path(), "Cargo.toml", "[workspace]\nmembers=['a','b']\n");
    for member in ["a", "b"] {
        write(
            dir.path(),
            &format!("{member}/Cargo.toml"),
            &format!("[package]\nname='{member}'\nversion='0.1.0'\n"),
        );
        write(dir.path(), &format!("{member}/src/lib.rs"), "");
    }
    assert!(ids(&report(dir.path(), &["--workspace"])).contains(&"versions.independent"));
    assert!(
        ok(dir.path(), &[])
            .contains("These principles have guided this project from the beginning.")
    );
}
#[test]
fn development_requirement_reversal_is_a_semantic_amendment() {
    let dir = fixture("[dev-dependencies]\nproptest='=1.5.0'\n");
    save(dir.path(), &report(dir.path(), &[]));
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='0.1.0'\n[dev-dependencies]\nproptest='1'\n",
    );
    let result: Value = serde_json::from_str(&ok(
        dir.path(),
        &["--amend", "before.json", "--format", "json"],
    ))
    .unwrap();
    assert_eq!(result["changes"][0]["kind"], "version-policy");
    assert!(
        result["changes"][0]["subject"]
            .as_str()
            .unwrap()
            .contains("dev dependency")
    );
}
#[test]
fn absolute_authored_path_dependencies_are_reported_relatively() {
    let parent = tempfile::tempdir().unwrap();
    let project = parent.path().join("project");
    let dependency = parent.path().join("dependency");
    write(
        &project,
        "Cargo.toml",
        &format!(
            "[package]\nname='project'\nversion='0.1.0'\n[workspace]\n[dependencies]\ndependency={{path={:?}}}\n",
            dependency.to_str().unwrap()
        ),
    );
    write(&project, "src/lib.rs", "");
    write(
        &dependency,
        "Cargo.toml",
        "[package]\nname='dependency'\nversion='0.1.0'\n",
    );
    write(&dependency, "src/lib.rs", "");
    let out = ok(&project, &["--format", "json"]);
    assert!(!out.contains(parent.path().to_str().unwrap()));
    assert!(out.contains("../dependency"));
}

#[test]
fn a_package_named_workspace_cannot_supply_another_members_combination() {
    let dir = fixture(
        "[workspace]\nmembers=['other']\nresolver='2'\n[dev-dependencies]\nproptest='1'\n[profile.release]\npanic='abort'\n",
    );
    let path = dir.path().join("Cargo.toml");
    fs::write(
        &path,
        fs::read_to_string(&path)
            .unwrap()
            .replace("name='specimen'", "name='workspace'"),
    )
    .unwrap();
    write(
        dir.path(),
        "other/Cargo.toml",
        "[package]\nname='other'\nversion='0.1.0'\n",
    );
    write(dir.path(), "other/src/lib.rs", "");
    let r = report(dir.path(), &["--workspace"]);
    let combination = r["articles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == "combination.considered-abort")
        .unwrap();
    assert_eq!(combination["scope"], "workspace");
}
