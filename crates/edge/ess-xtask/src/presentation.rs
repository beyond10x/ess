//! The visualise page's presentation of the billing example, written by the pinned `ess-ui`.
//!
//! `website/docs/visualise.md` renders `<EssPresentation>` from `@beyond10x/ess-ui-templates`,
//! and the component draws the `ess-ui-presentation/1` document that repository's `ess-ui data`
//! writes. Both come from one commit: the package is pinned by commit in `website/package.json`,
//! and the binary that records the document is installed from the same commit, so the browser
//! bundle and the data it reads never come from two revisions.
//!
//! `ess-ui` depends on ESS crates, so it cannot be a Cargo dependency of anything here without a
//! cycle. It is installed with `cargo install --git … --rev <pin>` into a cache directory named
//! after the pin, and run as a separate program on the IR, suite and authored scenarios that this
//! repository's `ess` writes for a copy of the example.

use anyhow::{bail, ensure, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The npm package whose pin selects the `ess-ui` commit.
const PACKAGE: &str = "@beyond10x/ess-ui-templates";

/// The only spelling of the pin accepted in `website/package.json`.
const PIN_PREFIX: &str = "github:beyond10x/ess-ui-templates#";

/// The repository `cargo install` builds `ess-ui` from.
const GIT_URL: &str = "https://github.com/beyond10x/ess-ui-templates";

/// The written document's name under `website/data/`.
pub(super) const FILE: &str = "billing.presentation.json";

/// The specification and the authored scenarios the presentation plays.
const EXAMPLE: &str = "examples/billing";
const SCENARIOS: &str = "examples/billing-scenarios";

/// Where the presentation's source links point: the example on `main`. A commit would make every
/// commit stale against its own recording.
const REPO_URL: &str = "https://github.com/beyond10x/ess";
const REF: &str = "main";

/// Shown in the page's footer as the compiler; the IR is this repository's `ess` output.
const COMPILER: &str = "ess built from this repository";

/// The `ess-ui` commit `website/package.json` pins.
pub(super) fn pinned_revision(root: &Path) -> Result<String> {
    let path = root.join("website/package.json");
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    revision_from_manifest(&text)
}

fn revision_from_manifest(text: &str) -> Result<String> {
    let manifest: serde_json::Value =
        serde_json::from_str(text).context("website/package.json is not JSON")?;
    let Some(spec) = manifest["dependencies"][PACKAGE].as_str() else {
        bail!("website/package.json does not depend on {PACKAGE}");
    };
    let Some(revision) = spec.strip_prefix(PIN_PREFIX) else {
        bail!("{PACKAGE} must be pinned as `{PIN_PREFIX}<commit>`, found `{spec}`");
    };
    ensure!(
        revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "{PACKAGE} must be pinned to a full 40-character commit, found `{revision}`"
    );
    Ok(revision.to_ascii_lowercase())
}

/// The directory a pinned `ess-ui` is installed under: `$ESS_UI_ROOT`, or `~/.cache/ess-ui`,
/// with one subdirectory per commit.
fn install_root(revision: &str) -> Result<PathBuf> {
    let base = match std::env::var_os("ESS_UI_ROOT") {
        Some(root) if !root.is_empty() => PathBuf::from(root),
        _ => {
            let home = std::env::var_os("HOME").context("neither ESS_UI_ROOT nor HOME is set")?;
            PathBuf::from(home).join(".cache/ess-ui")
        }
    };
    Ok(base.join(revision))
}

/// The pinned `ess-ui`, installed first when it is missing.
pub(super) fn binary(root: &Path) -> Result<PathBuf> {
    let revision = pinned_revision(root)?;
    let install = install_root(&revision)?;
    let binary = install.join("bin").join("ess-ui");
    if binary.is_file() {
        return Ok(binary);
    }
    fs::create_dir_all(&install).with_context(|| format!("creating {}", install.display()))?;
    eprintln!(
        "installing ess-ui {revision} into {} (once per pinned commit)",
        install.display()
    );
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(&cargo)
        .args(["install", "--git", GIT_URL, "--rev", &revision, "--locked"])
        .args(["--quiet", "ess-ui", "--root"])
        .arg(&install)
        // Outside this workspace, so its `.cargo/config.toml` does not apply to another build.
        .current_dir(&install)
        .status()
        .with_context(|| format!("running {cargo:?} install"))?;
    ensure!(status.success(), "installing ess-ui {revision} failed");
    ensure!(
        binary.is_file(),
        "`cargo install` reported success and wrote no {}",
        binary.display()
    );
    Ok(binary)
}

/// The presentation document `ess-ui data` writes for a copy of the example.
pub(super) fn render(root: &Path, ess: &Path, ess_ui: &Path, scratch: &Path) -> Result<String> {
    for tree in [EXAMPLE, SCENARIOS] {
        super::site_data::copy_tree(&root.join(tree), &scratch.join(tree))?;
    }
    run(
        ess,
        scratch,
        &[
            "specify", "compile", "--path", EXAMPLE, "--format", "json", "--out", "ir.json",
        ],
        None,
    )?;
    // The synthesizer's report is shown on the page's overview, so it is kept as printed.
    run(
        ess,
        scratch,
        &[
            "verify",
            "conform",
            "synthesize",
            "--path",
            EXAMPLE,
            "--out",
            "suite.json",
        ],
        Some("synthesize.log"),
    )?;
    run(
        ess,
        scratch,
        &[
            "verify",
            "conform",
            "author",
            "--path",
            EXAMPLE,
            "--scenarios",
            SCENARIOS,
            "--out",
            "authored.json",
        ],
        None,
    )?;
    run(
        ess_ui,
        scratch,
        &[
            "data",
            "--ir",
            "ir.json",
            "--spec-dir",
            EXAMPLE,
            "--suite",
            "suite.json",
            "--scenarios",
            "authored.json",
            "--scenario-dir",
            SCENARIOS,
            "--synth-log",
            "synthesize.log",
            "--compiler",
            COMPILER,
            "--repo-url",
            REPO_URL,
            "--repo-label",
            "GitHub",
            "--spec-root",
            EXAMPLE,
            "--ref",
            REF,
            "--out",
            "presentation.json",
        ],
        None,
    )?;
    let document = fs::read_to_string(scratch.join("presentation.json"))
        .context("reading what `ess-ui data` wrote")?;
    let parsed: serde_json::Value =
        serde_json::from_str(&document).context("`ess-ui data` wrote no JSON")?;
    ensure!(
        parsed["format"] == "ess-ui-presentation/1",
        "`ess-ui data` wrote format {}, expected ess-ui-presentation/1",
        parsed["format"]
    );
    let unrendered = parsed["unrendered"].as_array().map_or(0, Vec::len);
    ensure!(
        unrendered == 0,
        "`ess-ui` left {unrendered} declaration(s) of the example unrendered: {}",
        parsed["unrendered"]
    );
    Ok(document)
}

/// Runs `program` in `directory`; with `log`, standard output and error both go to that file.
fn run(program: &Path, directory: &Path, args: &[&str], log: Option<&str>) -> Result<()> {
    let output = Command::new(program)
        .args(args)
        .current_dir(directory)
        .output()
        .with_context(|| format!("running {}", program.display()))?;
    ensure!(
        output.status.success(),
        "`{} {}` failed: {}",
        program.display(),
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    if let Some(log) = log {
        let mut text = output.stdout;
        text.extend_from_slice(&output.stderr);
        fs::write(directory.join(log), text).with_context(|| format!("writing {log}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::revision_from_manifest;

    const SHA: &str = "60e9ff27d38b71d245a1d5d39e97381d5d378696";

    #[test]
    fn the_pin_is_read_from_the_package_manifest() {
        let manifest = format!(
            r#"{{"dependencies": {{"@beyond10x/ess-ui-templates": "github:beyond10x/ess-ui-templates#{SHA}"}}}}"#
        );
        assert_eq!(revision_from_manifest(&manifest).unwrap(), SHA);
    }

    #[test]
    fn a_branch_a_short_commit_or_another_host_is_refused() {
        for spec in [
            "github:beyond10x/ess-ui-templates#main",
            "github:beyond10x/ess-ui-templates#60e9ff2",
            "git+https://github.com/beyond10x/ess-ui-templates.git#60e9ff27d38b71d245a1d5d39e97381d5d378696",
            "^0.1.0",
        ] {
            let manifest =
                format!(r#"{{"dependencies": {{"@beyond10x/ess-ui-templates": "{spec}"}}}}"#);
            assert!(
                revision_from_manifest(&manifest).is_err(),
                "accepted `{spec}`"
            );
        }
        assert!(revision_from_manifest(r#"{"dependencies": {}}"#).is_err());
    }
}
