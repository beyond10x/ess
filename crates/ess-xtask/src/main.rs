//! Repository-only release checks for ESS.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "ess-xtask")]
#[command(about = "Repository-only ESS maintenance commands")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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
    let manifest = fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|error| format!("read Cargo.toml: {error}"))?;
    let changelog = fs::read_to_string(root.join("CHANGELOG.md"))
        .map_err(|error| format!("read CHANGELOG.md: {error}"))?;
    let workspace_version = workspace_version(&manifest)
        .ok_or_else(|| "Cargo.toml has no [workspace.package] version".to_owned())?;
    match cli.command {
        Command::Release {
            command: ReleaseCommand::Verify { version },
        } => {
            let version = version.as_deref().unwrap_or(workspace_version);
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
            if version != workspace_version {
                return Err(format!(
                    "release {version} does not match workspace version {workspace_version}"
                ));
            }
            release_notes(&changelog, &version)
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
