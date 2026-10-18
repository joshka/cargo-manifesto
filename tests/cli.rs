use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::TempDir;

const CONTINUITY: &str = "These principles have guided this project from the beginning.";

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
/// A workspace whose root manifest is `root` and whose members are `(name, manifest)`.
fn workspace(root: &str, members: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "Cargo.toml", root);
    write(dir.path(), "src/lib.rs", "");
    for (name, manifest) in members {
        write(dir.path(), &format!("{name}/Cargo.toml"), manifest);
        write(dir.path(), &format!("{name}/src/lib.rs"), "");
    }
    dir
}
fn member(name: &str) -> String {
    format!("[package]\nname='{name}'\nversion='0.1.0'\n")
}
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-manifesto"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}
fn code(root: &Path, args: &[&str]) -> Option<i32> {
    run(root, args).status.code()
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
/// Output with line wrapping undone.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn report(root: &Path, args: &[&str]) -> Value {
    let arguments = [&["--format", "json"], args].concat();
    serde_json::from_str(&ok(root, &arguments)).unwrap()
}
fn amend_json(root: &Path) -> Value {
    serde_json::from_str(&ok(root, &["--amend", "before.json", "--format", "json"])).unwrap()
}
fn packages(r: &Value) -> Vec<&str> {
    let packages = r["facts"]["scope"]["packages"].as_array().unwrap();
    packages
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect()
}
fn obs<'a>(r: &'a Value, kind: &str, scope: &str) -> &'a Value {
    let all = r["facts"]["observations"].as_array().unwrap();
    all.iter()
        .find(|o| o["kind"] == kind && o["scope"] == scope)
        .unwrap()
}
fn ids(r: &Value) -> Vec<&str> {
    let articles = r["articles"].as_array().unwrap();
    articles.iter().map(|a| a["id"].as_str().unwrap()).collect()
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
    let root = "[package]\nname='root'\nversion='0.1.0'\n[workspace]\nmembers=['a','b']\ndefault-members=['b']\nresolver='2'\n";
    let dir = workspace(root, &[("a", &member("a")), ("b", &member("b"))]);
    let path = dir.path();
    assert_eq!(packages(&report(path, &[])), ["b"]);
    assert_eq!(packages(&report(&path.join("a/src"), &[])), ["a"]);
    assert_eq!(packages(&report(path, &["-p", "a"])), ["a"]);
    assert_eq!(
        packages(&report(&path.join("a"), &["--workspace"])).len(),
        3
    );
    assert_eq!(code(path, &["-p", "a", "--workspace"]), Some(2));
    ok(&path.join("a"), &["--fix"]);
    assert!(path.join("a/MANIFESTO.md").exists());
    ok(path, &["--workspace", "--fix"]);
    assert!(path.join("MANIFESTO.md").exists());
}
#[test]
fn dependencies_retain_authored_distinctions_without_resolution() {
    let dir = fixture(
        "[dependencies]\nrenamed={package='serde',version='=1.0.200',optional=true,default-features=false}\n[dev-dependencies]\nproptest='1'\n[build-dependencies]\ncc='1'\n[target.'cfg(windows)'.dependencies]\nwinapi='0.3'\n",
    );
    let r = report(dir.path(), &[]);
    let all = r["facts"]["observations"].as_array().unwrap();
    let deps: Vec<_> = all
        .iter()
        .filter(|o| o["kind"] == "dependency")
        .map(|o| &o["value"])
        .collect();
    assert_eq!(deps.len(), 4);
    let alias = deps.iter().find(|d| d["alias"] == "renamed").unwrap();
    assert_eq!(alias["package"], "serde");
    assert_eq!(alias["optional"], true);
    assert_eq!(alias["default_features"], false);
    assert_eq!(alias["requirement"], "=1.0.200");
    assert!(deps.iter().any(|d| d["kind"] == "dev"));
    assert!(deps.iter().any(|d| d["kind"] == "build"));
    assert!(deps.iter().any(|d| d["target"] == "cfg(windows)"));
    assert_eq!(
        obs(&r, "dependency-counts", "specimen")["value"],
        json!({"all":4,"normal":2,"dev":1,"build":1})
    );
}
#[test]
fn inspection_neither_builds_nor_changes_or_creates_lockfiles() {
    let dir = fixture("[dependencies]\nnot-a-real-published-crate='9'\n");
    let path = dir.path();
    write(
        path,
        "build.rs",
        "fn main(){std::fs::write(\"BUILD_RAN\",\"bad\").unwrap();panic!(\"must not execute\");}",
    );
    let manifest = fs::read(path.join("Cargo.toml")).unwrap();
    let r = report(path, &[]);
    assert_eq!(
        obs(&r, "lockfile", "@workspace")["value"],
        json!({"available":false})
    );
    assert!(!path.join("Cargo.lock").exists());
    assert!(!path.join("BUILD_RAN").exists());
    assert!(!path.join("target").exists());
    write(
        path,
        "Cargo.lock",
        "version=4\n[[package]]\nname='specimen'\nversion='0.1.0'\n",
    );
    let lock = fs::read(path.join("Cargo.lock")).unwrap();
    ok(path, &["--fix"]);
    assert_eq!(fs::read(path.join("Cargo.lock")).unwrap(), lock);
    assert_eq!(fs::read(path.join("Cargo.toml")).unwrap(), manifest);
    assert!(!path.join("BUILD_RAN").exists());
}
#[test]
fn deterministic_across_ordering_metadata_and_directory_changes() {
    let a = fixture("description='first description'\n[dependencies]\nserde='1'\nclap='4'\n");
    let b = fixture("description='irrelevant edit'\n[dependencies]\nclap='4'\nserde='1'\n");
    assert_eq!(ok(a.path(), &[]), ok(b.path(), &[]));
    assert_eq!(report(a.path(), &[]), report(b.path(), &[]));
    assert!(!ok(a.path(), &["--explain"]).contains(a.path().to_str().unwrap()));
}
#[test]
fn inheritance_and_root_only_settings() {
    let dir = workspace(
        "[workspace]\nmembers=['member']\nresolver='2'\n[workspace.package]\nversion='1.2.3'\nrust-version='1.85'\n[workspace.dependencies]\ns={package='serde',version='1',default-features=false}\n[workspace.lints.rust]\nunsafe_code='forbid'\n[profile.release]\npanic='abort'\n[patch.crates-io]\nserde={git='https://example.invalid/serde'}\n",
        &[(
            "member",
            "[package]\nname='member'\nversion.workspace=true\nrust-version.workspace=true\n[dependencies]\ns.workspace=true\n[lints]\nworkspace=true\n[profile.release]\npanic='unwind'\n[patch.crates-io]\nserde={git='https://example.invalid/wrong'}\n",
        )],
    );
    let r = report(&dir.path().join("member"), &[]);
    let value = |kind: &str, scope: &str| obs(&r, kind, scope)["value"].clone();
    assert_eq!(
        value("package.version", "member"),
        json!({"value":"1.2.3","inherited":true})
    );
    assert_eq!(value("package.rust-version", "member")["value"], "1.85");
    assert_eq!(
        value("lints.rust.unsafe_code", "member"),
        json!({"value":"forbid","inherited":true})
    );
    assert_eq!(
        value("profile.release.panic", "@workspace")["value"],
        "abort"
    );
    assert_eq!(
        value("patch", "@workspace")["value"]["crates-io"]["serde"]["git"],
        "https://example.invalid/serde"
    );
    let dependency = obs(&r, "dependency", "member");
    assert_eq!(dependency["value"]["package"], "serde");
    assert_eq!(dependency["evidence"].as_array().unwrap().len(), 2);
}
#[test]
fn managed_document_exit_codes_and_explicit_file_resolution() {
    let dir = fixture("");
    let path = dir.path();
    assert_eq!(code(path, &["--check"]), Some(1));
    ok(path, &["--fix"]);
    assert_eq!(code(path, &["--check"]), Some(0));
    write(path, "MANIFESTO.md", "unmarked\n");
    assert_eq!(code(path, &["--fix"]), Some(2));
    assert_eq!(
        fs::read_to_string(path.join("MANIFESTO.md")).unwrap(),
        "unmarked\n"
    );
    write(
        path,
        "src/README.md",
        "before\n<!-- cargo-manifesto:begin -->old<!-- cargo-manifesto:end -->\nafter\n",
    );
    let src = path.join("src");
    ok(&src, &["--fix", "--file", "README.md"]);
    let text = fs::read_to_string(src.join("README.md")).unwrap();
    assert!(text.starts_with("before\n<!-- cargo-manifesto:begin -->\n"));
    assert!(text.ends_with("<!-- cargo-manifesto:end -->\nafter\n"));
    assert_eq!(code(&src, &["--check", "--file", "README.md"]), Some(0));
}
#[test]
fn invalid_combinations_and_baselines_have_exit_two() {
    let dir = fixture("");
    let path = dir.path();
    for args in [
        vec!["--origin", "--fix"],
        vec!["--check", "--fix"],
        vec!["--format", "json", "--fix"],
        vec!["--format", "json", "--explain"],
        vec!["--explain", "--check"],
        vec!["--file", "x"],
        vec!["--origin", "--workspace"],
    ] {
        let out = run(path, &args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert_eq!(out.stdout, b"");
    }
    write(path, "bad.json", "{\"schema_version\":999}");
    assert!(
        String::from_utf8_lossy(&run(path, &["--amend", "bad.json"]).stderr).contains("schema")
    );
    write(path, "bad.json", "not json");
    assert_eq!(code(path, &["--amend", "bad.json"]), Some(2));
}
#[test]
fn semantic_amendments_reverse_and_keep_continuity() {
    let dir = fixture("");
    let path = dir.path();
    let manifest = |version: &str, rest: &str| {
        write(
            path,
            "Cargo.toml",
            &format!("[package]\nname='specimen'\nversion='{version}'\n{rest}"),
        );
    };
    save(path, &report(path, &[]));
    manifest(
        "0.2.0",
        "[dependencies]\nserde='=1.0.200'\n[profile.release]\npanic='unwind'\n",
    );
    let text = flat(&ok(path, &["--amend", "before.json"]));
    assert!(text.contains("Dependencies are an abdication"));
    assert!(text.contains("Reimplementing established solutions"));
    assert!(text.contains("serde"));
    assert!(text.contains(CONTINUITY));

    save(path, &report(path, &[]));
    manifest(
        "0.3.0",
        "[dependencies]\nserde='1'\n[profile.release]\npanic='abort'\n",
    );
    let kinds: Vec<_> = amend_json(path)["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["kind"].clone())
        .collect();
    assert_eq!(kinds, [json!("version-policy"), json!("panic-policy")]);

    save(path, &report(path, &[]));
    manifest(
        "0.4.0",
        "[dependencies]\nserde='1'\nclap='4'\n[profile.release]\npanic='abort'\n",
    );
    assert!(
        flat(&ok(path, &["--amend", "before.json"]))
            .contains("commitment to cooperation now extends to clap")
    );

    save(path, &report(path, &[]));
    manifest("0.5.0", "[profile.release]\npanic='abort'\n");
    assert!(ok(path, &["--amend", "before.json"]).contains("Dependencies are an abdication"));
}
#[test]
fn every_opposing_pair_can_be_amended() {
    let dir = fixture("");
    save(dir.path(), &report(dir.path(), &[]));
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='0.1.0'\nedition='2024'\n",
    );
    let change = &amend_json(dir.path())["changes"][0];
    assert_eq!(change["kind"], "edition");
    assert!(
        change["revised_doctrine"]
            .as_str()
            .unwrap()
            .contains("current edition")
    );
}
#[test]
fn baseline_catalogue_wording_and_package_version_are_not_changes() {
    let dir = fixture("");
    let path = dir.path();
    let mut before = report(path, &[]);
    before["articles"][0]["text"] = json!("old wording");
    before["catalogue_version"] = json!(0);
    save(path, &before);
    write(
        path,
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='9.0.0'\ndescription='unrelated'\n",
    );
    assert!(
        ok(path, &["--amend", "before.json"])
            .contains("found our principles exactly where we left them")
    );
    before["facts"]["scope"]["packages"][0]["name"] = json!("elsewhere");
    save(path, &before);
    assert_eq!(code(path, &["--amend", "before.json"]), Some(2));
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
        assert!(
            replaced.iter().all(|id| !selected.contains(id)),
            "{combination}: {selected:?}"
        );
    }
}
#[test]
fn workspace_combinations_and_mixed_editions() {
    let dir = workspace(
        "[workspace]\nmembers=['a','b']\nresolver='2'\n[workspace.package]\nversion='1.0.0'\n[workspace.lints.rust]\nunsafe_code='forbid'\n",
        &[
            (
                "a",
                "[package]\nname='a'\nversion='0.1.0'\nedition='2018'\n[lints]\nworkspace=true\n",
            ),
            (
                "b",
                "[package]\nname='b'\nversion.workspace=true\nedition='2021'\n",
            ),
        ],
    );
    let registry = "source='registry+https://github.com/rust-lang/crates.io-index'";
    write(
        dir.path(),
        "Cargo.lock",
        &format!(
            "version=4\n[[package]]\nname='external'\nversion='1.0.0'\n{registry}\n[[package]]\nname='external'\nversion='2.0.0'\n{registry}\n"
        ),
    );
    let selected = ids(&report(dir.path(), &["--workspace"]))
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert!(selected.contains(&"combination.governed-freedom".into()));
    assert!(selected.contains(&"workspace.mixed-editions".into()));
    assert!(!selected.iter().any(|id| id.starts_with("edition.")));
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
        assert_eq!(code(dir.path(), &["--fix", "--file", path]), Some(2));
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
        assert!(ok(dir.path(), &[]).contains(CONTINUITY));
    }
    let dir = fixture("");
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion.workspace=true\n[workspace.package]\nversion='1.0.0'\n",
    );
    let r = report(dir.path(), &[]);
    assert!(ids(&r).contains(&"versions.shared"));
    assert!(!ids(&r).contains(&"versions.independent"));
    let dir = workspace(
        "[workspace]\nmembers=['a','b']\n",
        &[("a", &member("a")), ("b", &member("b"))],
    );
    assert!(ids(&report(dir.path(), &["--workspace"])).contains(&"versions.independent"));
    assert!(ok(dir.path(), &[]).contains(CONTINUITY));
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
    let change = &amend_json(dir.path())["changes"][0];
    assert_eq!(change["kind"], "version-policy");
    assert_eq!(change["subject"], "specimen: proptest 1 (dev)");
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
    write(&dependency, "Cargo.toml", &member("dependency"));
    write(&dependency, "src/lib.rs", "");
    let out = ok(&project, &["--format", "json"]);
    assert!(!out.contains(parent.path().to_str().unwrap()));
    assert!(out.contains("../dependency"));
}
#[test]
fn a_package_named_workspace_cannot_supply_another_members_combination() {
    let dir = workspace(
        "[package]\nname='workspace'\nversion='0.1.0'\n[workspace]\nmembers=['other']\nresolver='2'\n[dev-dependencies]\nproptest='1'\n[profile.release]\npanic='abort'\n",
        &[("other", &member("other"))],
    );
    let r = report(dir.path(), &["--workspace"]);
    let articles = r["articles"].as_array().unwrap();
    let combination = articles
        .iter()
        .find(|a| a["id"] == "combination.considered-abort")
        .unwrap();
    assert_eq!(combination["scope"], "workspace");
}
#[test]
fn edition_pair_and_subcommand_rules_follow_authored_targets() {
    let dir = fixture("");
    assert!(ids(&report(dir.path(), &[])).contains(&"edition.settled"));
    write(
        dir.path(),
        "Cargo.toml",
        "[package]\nname='specimen'\nversion='0.1.0'\nedition='2024'\n[[bin]]\nname='cargo-specimen'\npath='src/main.rs'\n",
    );
    write(dir.path(), "src/main.rs", "fn main() {}\n");
    let selected = ids(&report(dir.path(), &[]))
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert!(selected.contains(&"edition.current".into()));
    assert!(selected.contains(&"tooling.subcommand".into()));
    assert!(flat(&ok(dir.path(), &[])).contains("named ourselves cargo-specimen"));
}
#[test]
fn metadata_configures_the_managed_document() {
    let dir = fixture("[package.metadata.manifesto]\nfile='README.md'\n");
    let path = dir.path();
    write(path, "README.md", "# Specimen\n\nHandwritten.\n");
    // An existing unmarked file is still refused.
    assert_eq!(code(path, &["--fix"]), Some(2));
    write(
        path,
        "README.md",
        "# Specimen\n\n<!-- cargo-manifesto:begin -->\n<!-- cargo-manifesto:end -->\n\nAfter.\n",
    );
    ok(path, &["--fix"]);
    let readme = fs::read_to_string(path.join("README.md")).unwrap();
    assert!(readme.starts_with("# Specimen\n\n<!-- cargo-manifesto:begin -->\n# The Manifesto"));
    assert!(readme.ends_with("<!-- cargo-manifesto:end -->\n\nAfter.\n"));
    assert!(flat(&readme).contains("derived from the implementation by"));
    assert!(!path.join("MANIFESTO.md").exists());
    ok(path, &["--check"]);

    let dir = fixture("[package.metadata.manifesto]\nfile=1\n");
    assert_eq!(code(dir.path(), &["--fix"]), Some(2));
}
