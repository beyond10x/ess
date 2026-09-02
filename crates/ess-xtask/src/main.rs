//! Repository-only maintenance checks for ESS.

use anyhow::{bail, Context, Result as AnyResult};
use clap::{Parser, Subcommand};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The specification whose projections are committed for review.
const NORMATIVE_EXAMPLE: &str = "examples/billing";

/// Where the normative example's projections are committed.
const PROJECTIONS: &str = "generated";

/// The generated-tree index, derived from the same inventory as its contents.
const INDEX: &str = "README.md";

/// Generated subtrees owned by other mechanisms, not by [`generate`].
///
/// The projection drift check must not report or delete structural synthesis and instruction
/// output. It still reports every unknown top-level subtree, so withdrawing a projection cannot
/// leave a stale public contract behind unnoticed.
const PROJECTION_EXCLUSIONS: &[&str] = &["go", "instructions", "rust", "web"];

#[derive(Debug, Parser)]
#[command(name = "ess-xtask")]
#[command(about = "Repository-only ESS maintenance commands")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Regenerate or check the normative example's committed projections.
    Generate {
        /// Compare byte for byte without writing.
        #[arg(long)]
        check: bool,
    },
    /// Verify or render release records.
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ReleaseCommand {
    /// Verify that Cargo and the changelog name the same release.
    Verify {
        /// Version to verify; defaults to the workspace version.
        version: Option<String>,
    },
    /// Print one release's changelog section as GitHub release notes.
    Notes {
        /// Version whose notes should be rendered.
        version: String,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(output) => {
            if !output.is_empty() {
                print!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<String, String> {
    let root = workspace_root()?;
    match cli.command {
        Command::Generate { check } => generate(&root, check).map_err(|error| format!("{error:#}")),
        Command::Release {
            command: ReleaseCommand::Verify { version },
        } => {
            let (workspace_version, changelog) = release_inputs(&root)?;
            let version = version.as_deref().unwrap_or(&workspace_version);
            if version != workspace_version {
                return Err(format!(
                    "release {version} does not match workspace version {workspace_version}"
                ));
            }
            release_notes(&changelog, version)?;
            Ok(format!(
                "release {version}: workspace version and changelog agree\n"
            ))
        }
        Command::Release {
            command: ReleaseCommand::Notes { version },
        } => {
            let (workspace_version, changelog) = release_inputs(&root)?;
            if version != workspace_version {
                return Err(format!(
                    "release {version} does not match workspace version {workspace_version}"
                ));
            }
            release_notes(&changelog, &version)
        }
    }
}

fn release_inputs(root: &Path) -> Result<(String, String), String> {
    let manifest = fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|error| format!("read Cargo.toml: {error}"))?;
    let changelog = fs::read_to_string(root.join("CHANGELOG.md"))
        .map_err(|error| format!("read CHANGELOG.md: {error}"))?;
    let version = workspace_version(&manifest)
        .ok_or_else(|| "Cargo.toml has no [workspace.package] version".to_owned())?
        .to_owned();
    Ok((version, changelog))
}

fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "ess-xtask is not inside the ESS workspace".to_owned())
}

fn workspace_version(manifest: &str) -> Option<&str> {
    let mut in_workspace_package = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_workspace_package = trimmed == "[workspace.package]";
            continue;
        }
        if in_workspace_package {
            if let Some(value) = trimmed.strip_prefix("version =") {
                return value.trim().strip_prefix('"')?.strip_suffix('"');
            }
        }
    }
    None
}

fn release_notes(changelog: &str, version: &str) -> Result<String, String> {
    let prefix = format!("## [{version}] ");
    let mut lines = changelog.lines();
    let found = lines
        .by_ref()
        .find(|line| line.starts_with(&prefix))
        .ok_or_else(|| format!("CHANGELOG.md has no release heading for {version}"))?;
    if !found.contains('—') {
        return Err(format!("release heading for {version} has no date"));
    }
    let body = lines
        .take_while(|line| !line.starts_with("## ["))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned();
    if body.is_empty() {
        return Err(format!("release heading for {version} has no notes"));
    }
    Ok(format!("{body}\n"))
}

/// One projection published by `ess generate`.
struct Projection {
    name: String,
    directory: String,
    describes: String,
}

/// The projections and their common source identity.
struct Generated {
    provenance: String,
    projections: Vec<Projection>,
    artifacts: BTreeMap<String, String>,
}

/// Writes or checks every projection of [`NORMATIVE_EXAMPLE`].
fn generate(root: &Path, check: bool) -> AnyResult<String> {
    let generated = projections(root, &root.join(NORMATIVE_EXAMPLE))?;
    let mut expected = generated.artifacts.clone();
    expected.insert(INDEX.to_owned(), projection_index(&generated));
    sync(
        &root.join(PROJECTIONS),
        &expected,
        check,
        PROJECTION_EXCLUSIONS,
    )
}

/// Runs the public CLI and reads exactly the artifacts it reports.
///
/// The drift check deliberately goes through `ess generate`, instead of calling generators through
/// a second repository-only path. This makes the bytes an adopter writes and the bytes CI compares
/// one answer. Generator metadata is read from `ess-gen`, because the CLI's machine output is an
/// artifact map and the index still needs to describe every projection it contains.
fn projections(root: &Path, spec: &Path) -> AnyResult<Generated> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = std::process::Command::new(&cargo)
        .args([
            "run",
            "--quiet",
            "--locked",
            "--package",
            "ess-cli",
            "--bin",
            "ess",
            "--",
            "generate",
            "--format",
            "json",
            "--path",
        ])
        .arg(spec)
        .current_dir(root)
        .output()
        .with_context(|| format!("running {cargo:?} to generate the projections"))?;
    if !output.status.success() {
        bail!(
            "`ess generate` refused {}:\n{}{}",
            spec.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("reading the JSON artifact map printed by `ess generate`")?;
    let entries = report
        .as_object()
        .context("`ess generate --format json` did not print an artifact map")?;
    let mut artifacts = BTreeMap::new();
    for (reported_path, artifact) in entries {
        let path = json_text(artifact, "path")?;
        if path != *reported_path {
            bail!(
                "`ess generate` keyed `{reported_path}` by an artifact that names itself `{path}`"
            );
        }
        safe_relative_path(&path)?;
        let contents = json_text(artifact, "contents")?;
        artifacts.insert(path, contents);
    }

    let projections: Vec<Projection> = ess_gen::generators()
        .into_iter()
        .map(|generator| Projection {
            name: generator.name().to_owned(),
            directory: generator.directory().to_owned(),
            describes: generator.describes().to_owned(),
        })
        .collect();
    for path in artifacts.keys() {
        let owners = projections
            .iter()
            .filter(|projection| path.starts_with(&format!("{}/", projection.directory)))
            .count();
        if owners != 1 {
            bail!("generated artifact `{path}` belongs to {owners} projection directories");
        }
    }

    let provenance = projection_provenance(&artifacts)?;
    Ok(Generated {
        provenance,
        projections,
        artifacts,
    })
}

fn json_text(value: &serde_json::Value, field: &str) -> AnyResult<String> {
    value[field]
        .as_str()
        .map(ToOwned::to_owned)
        .with_context(|| {
            format!("an artifact in the `ess generate` report has no string `{field}`")
        })
}

/// Rejects absolute paths and parent traversal before joining a generated path to the output root.
fn safe_relative_path(path: &str) -> AnyResult<()> {
    use std::path::Component;

    if path.is_empty()
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("generator emitted unsafe artifact path `{path}`");
    }
    Ok(())
}

/// Reads the whole-model stamp from the generated documentation index.
fn projection_provenance(artifacts: &BTreeMap<String, String>) -> AnyResult<String> {
    let index = artifacts
        .get("docs/index.md")
        .context("the docs projection emitted no `docs/index.md`")?;
    let subject = index
        .lines()
        .map(|line| line.trim().trim_start_matches('#').trim())
        .find_map(|line| line.strip_prefix("generated from "))
        .context("`docs/index.md` carries no generated-from stamp")?;
    let digests = ess_gen::Provenance::read_digests(index)
        .context("`docs/index.md` carries no readable provenance digests")?;
    Ok(format!(
        "{subject} (model digest {}, contract digest {})",
        digests.source_digest, digests.contract_digest
    ))
}

/// Writes changed projection bytes, or compares without touching the tree.
fn sync(
    out: &Path,
    expected: &BTreeMap<String, String>,
    check: bool,
    excluded: &[&str],
) -> AnyResult<String> {
    let mut differing = Vec::new();
    let mut written = 0_usize;
    let mut removed = 0_usize;

    for (path, contents) in expected {
        let target = out.join(path);
        if fs::read_to_string(&target).ok().as_deref() == Some(contents.as_str()) {
            continue;
        }
        if check {
            differing.push(path.clone());
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        fs::write(&target, contents).with_context(|| format!("writing {}", target.display()))?;
        written += 1;
    }

    let mut orphaned = Vec::new();
    for path in committed_files(out, excluded)? {
        if expected.contains_key(&path) {
            continue;
        }
        if check {
            orphaned.push(path);
        } else {
            let target = out.join(&path);
            fs::remove_file(&target)
                .with_context(|| format!("removing stale projection {}", target.display()))?;
            removed += 1;
        }
    }

    if check {
        if differing.is_empty() && orphaned.is_empty() {
            return Ok("projections are up to date\n".to_owned());
        }
        let mut detail = String::new();
        if !differing.is_empty() {
            let _ = writeln!(
                detail,
                "{} projection(s) differ from the specification: {}",
                differing.len(),
                differing.join(", ")
            );
        }
        if !orphaned.is_empty() {
            let _ = writeln!(
                detail,
                "{} projection(s) are generated by nothing any more: {}",
                orphaned.len(),
                orphaned.join(", ")
            );
        }
        bail!("{detail}run `cargo xtask generate` and commit the result");
    }

    if out.is_dir() {
        prune_empty_directories(out, "", excluded)?;
    }
    Ok(format!(
        "projections written: {written} changed, {removed} no longer generated\n"
    ))
}

/// Every owned file below a committed output root, as a `/`-separated relative path.
fn committed_files(directory: &Path, excluded: &[&str]) -> AnyResult<BTreeSet<String>> {
    let mut found = BTreeSet::new();
    if !directory.is_dir() {
        return Ok(found);
    }
    let mut pending = vec![(directory.to_path_buf(), String::new())];
    while let Some((path, prefix)) = pending.pop() {
        for entry in fs::read_dir(&path).with_context(|| format!("reading {}", path.display()))? {
            let entry = entry.with_context(|| format!("reading {}", path.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let relative = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            if is_excluded(&relative, excluded) {
                continue;
            }
            if entry
                .file_type()
                .with_context(|| format!("reading the type of {}", entry.path().display()))?
                .is_dir()
            {
                pending.push((entry.path(), relative));
            } else {
                found.insert(relative);
            }
        }
    }
    Ok(found)
}

fn is_excluded(path: &str, excluded: &[&str]) -> bool {
    excluded
        .iter()
        .any(|entry| path == *entry || path.starts_with(&format!("{entry}/")))
}

/// Removes empty owned directories and leaves excluded subtrees untouched.
fn prune_empty_directories(directory: &Path, prefix: &str, excluded: &[&str]) -> AnyResult<bool> {
    let mut empty = true;
    for entry in
        fs::read_dir(directory).with_context(|| format!("reading {}", directory.display()))?
    {
        let entry = entry.with_context(|| format!("reading {}", directory.display()))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let relative = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        let is_directory = entry
            .file_type()
            .with_context(|| format!("reading the type of {}", path.display()))?
            .is_dir();
        if !is_directory || is_excluded(&relative, excluded) {
            empty = false;
        } else if prune_empty_directories(&path, &relative, excluded)? {
            fs::remove_dir(&path).with_context(|| format!("removing {}", path.display()))?;
        } else {
            empty = false;
        }
    }
    Ok(empty)
}

/// The generated tree's inventory and provenance.
fn projection_index(generated: &Generated) -> String {
    let mut out = format!(
        "# Generated projections\n\n**Do not edit these files.** They are generated from \
         [`{NORMATIVE_EXAMPLE}`](../{NORMATIVE_EXAMPLE}) by\n`cargo xtask generate`, and CI fails \
         if they differ from what the specification produces.\n\nEvery file here is a projection \
         of one model, so two of them disagreeing is a bug in one of them —\nand a file nothing \
         generates any more is a contract this repository no longer stands behind.\n\nGenerated \
         from {}.\n\n| projection | files | describes |\n| --- | --- | --- |\n",
        generated.provenance
    );
    for projection in &generated.projections {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            projection.name,
            files_of(generated, projection).count(),
            projection.describes
        );
    }
    for projection in &generated.projections {
        let _ = write!(out, "\n## `{}`\n\n", projection.name);
        let mut listed = false;
        for path in files_of(generated, projection) {
            let _ = writeln!(out, "* [`{path}`]({path})");
            listed = true;
        }
        if !listed {
            let _ = writeln!(out, "This projection produced no artifacts.");
        }
    }
    out
}

fn files_of<'a>(
    generated: &'a Generated,
    projection: &'a Projection,
) -> impl Iterator<Item = &'a str> {
    let prefix = format!("{}/", projection.directory);
    generated
        .artifacts
        .keys()
        .filter(move |path| path.starts_with(&prefix))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_DIRECTORY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn workspace_version_comes_only_from_the_workspace_package_table() {
        let manifest = "[package]\nversion = \"9.9.9\"\n[workspace.package]\nlicense = \"Apache-2.0\"\nversion = \"0.1.0\"\n";
        assert_eq!(workspace_version(manifest), Some("0.1.0"));
    }

    #[test]
    fn release_notes_stop_before_the_next_release() {
        let changelog = "# Changelog\n\n## [0.1.0] — 2026-09-01\n\n- First.\n\n## [0.0.1] — 2026-08-01\n\n- Old.\n";
        assert_eq!(
            release_notes(changelog, "0.1.0"),
            Ok("- First.\n".to_owned())
        );
    }

    #[test]
    fn an_undated_release_is_refused_by_name() {
        assert_eq!(
            release_notes("## [0.1.0] 2026-09-01\n\n- First.\n", "0.1.0"),
            Err("release heading for 0.1.0 has no date".to_owned())
        );
    }

    #[test]
    fn an_empty_release_is_refused_by_name() {
        assert_eq!(
            release_notes("## [0.1.0] — 2026-09-01\n\n", "0.1.0"),
            Err("release heading for 0.1.0 has no notes".to_owned())
        );
    }

    #[test]
    fn generated_paths_must_stay_below_the_projection_root() {
        assert!(safe_relative_path("site/domains/billing-invoice.md").is_ok());
        assert!(safe_relative_path("../Cargo.toml").is_err());
        assert!(safe_relative_path("/tmp/output").is_err());
        assert!(safe_relative_path("").is_err());
    }

    #[test]
    fn exclusions_cover_only_the_named_subtree() {
        assert!(is_excluded("rust", PROJECTION_EXCLUSIONS));
        assert!(is_excluded(
            "rust/billing/Cargo.toml",
            PROJECTION_EXCLUSIONS
        ));
        assert!(!is_excluded("rustacean.md", PROJECTION_EXCLUSIONS));
        assert!(!is_excluded("site/index.md", PROJECTION_EXCLUSIONS));
    }

    #[test]
    fn sync_checks_and_reconciles_in_both_directions() {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("ess-xtask-sync-{}-{sequence}", std::process::id()));
        fs::create_dir_all(root.join("docs")).expect("create docs fixture");
        fs::create_dir_all(root.join("go")).expect("create excluded fixture");
        fs::write(root.join("docs/index.md"), "old\n").expect("write changed fixture");
        fs::write(root.join("docs/orphan.md"), "orphan\n").expect("write orphan fixture");
        fs::write(root.join("go/owned-elsewhere.go"), "package fixture\n")
            .expect("write excluded fixture");

        let expected = BTreeMap::from([
            ("docs/index.md".to_owned(), "new\n".to_owned()),
            ("site/sidebar.json".to_owned(), "{}\n".to_owned()),
        ]);
        let drift = sync(&root, &expected, true, &["go"])
            .expect_err("check must report changed, missing, and orphaned files")
            .to_string();
        assert!(drift.contains("docs/index.md"));
        assert!(drift.contains("site/sidebar.json"));
        assert!(drift.contains("docs/orphan.md"));
        assert!(!drift.contains("owned-elsewhere.go"));

        assert_eq!(
            sync(&root, &expected, false, &["go"]).expect("reconcile projections"),
            "projections written: 2 changed, 1 no longer generated\n"
        );
        assert_eq!(
            sync(&root, &expected, true, &["go"]).expect("reconciled tree must pass"),
            "projections are up to date\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("go/owned-elsewhere.go"))
                .expect("excluded output is preserved"),
            "package fixture\n"
        );

        fs::remove_dir_all(root).expect("remove test fixture");
    }

    #[test]
    fn the_generated_index_includes_static_site_source() {
        let generated = Generated {
            provenance: "billing v3 (model digest a, contract digest b)".to_owned(),
            projections: vec![Projection {
                name: "site".to_owned(),
                directory: "site".to_owned(),
                describes: "static-site-ready Markdown and a sidebar".to_owned(),
            }],
            artifacts: BTreeMap::from([
                ("site/index.md".to_owned(), "# billing\n".to_owned()),
                ("site/sidebar.json".to_owned(), "{}\n".to_owned()),
            ]),
        };
        let index = projection_index(&generated);
        assert!(index.contains("| `site` | 2 | static-site-ready Markdown and a sidebar |"));
        assert!(index.contains("* [`site/sidebar.json`](site/sidebar.json)"));
    }
}
