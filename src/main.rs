mod amend;
mod catalogue;
mod document;
mod project;
mod render;

use anyhow::{Result, bail};
use clap::{ArgGroup, Parser, ValueEnum};
use std::{ffi::OsString, path::PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
enum Format {
    #[default]
    Markdown,
    Json,
}

/// Generate timeless principles from the choices already in your Cargo project.
#[derive(Debug, Parser)]
#[command(version, group(ArgGroup::new("action").args(["origin", "fix", "check", "amend"])))]
struct Cli {
    /// Print The Great Inversion without inspecting a project
    #[arg(long)]
    origin: bool,
    /// Annotate each article with its exact evidence
    #[arg(long)]
    explain: bool,
    #[arg(long, value_enum, default_value = "markdown")]
    format: Format,
    #[arg(long, value_name = "PATH")]
    manifest_path: Option<PathBuf>,
    /// Select a workspace member by its exact package name
    #[arg(short = 'p', long, conflicts_with = "workspace")]
    package: Option<String>,
    #[arg(long)]
    workspace: bool,
    /// Correct the philosophy to agree with the implementation
    #[arg(long)]
    fix: bool,
    /// Exit 1 if the managed manifesto is missing or stale
    #[arg(long)]
    check: bool,
    /// Managed document (relative to the caller's directory)
    #[arg(long, value_name = "PATH")]
    file: Option<PathBuf>,
    /// Explain semantic changes since a saved JSON report
    #[arg(long, value_name = "BEFORE.json")]
    amend: Option<PathBuf>,
}

fn main() {
    let mut args: Vec<OsString> = std::env::args_os().collect();
    if args.get(1).is_some_and(|a| a == "manifesto") {
        args.remove(1);
    }
    let cli = Cli::parse_from(args);
    match run(cli) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("cargo-manifesto: {error:#}");
            std::process::exit(2);
        }
    }
}

fn run(cli: Cli) -> Result<i32> {
    if cli.file.is_some() && !cli.fix && !cli.check {
        bail!("--file requires --fix or --check");
    }
    if cli.format == Format::Json && (cli.explain || cli.fix || cli.check || cli.origin) {
        bail!("JSON cannot be combined with --explain, --fix, --check, or --origin");
    }
    if cli.explain && (cli.fix || cli.check || cli.origin || cli.amend.is_some()) {
        bail!("--explain applies only to normal Markdown generation");
    }
    if cli.origin {
        if cli.manifest_path.is_some() || cli.package.is_some() || cli.workspace {
            bail!("--origin does not accept project selection flags");
        }
        print!("{}", include_str!("../docs/origin.md"));
        return Ok(0);
    }
    let loaded = project::load(&cli)?;
    let report = catalogue::report(loaded.facts);
    if let Some(path) = cli.amend {
        let baseline = amend::read(&path)?;
        let changes = amend::compare(&baseline, &report)?;
        if cli.format == Format::Json {
            println!("{}", serde_json::to_string_pretty(&changes)?);
        } else {
            print!("{}", amend::markdown(&changes));
        }
    } else if cli.fix || cli.check {
        let path = cli.file.unwrap_or(loaded.document);
        let body = render::markdown(&report, false);
        return document::manage(&path, &body, cli.check);
    } else if cli.format == Format::Json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", render::markdown(&report, cli.explain));
    }
    Ok(0)
}
