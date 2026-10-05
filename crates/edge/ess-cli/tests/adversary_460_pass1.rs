//! Adversary pass 1 for beyond10x/ess#460: `ess specify formats` against what it tells its reader.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            std::fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .expect("a member of this workspace lies under its root")
        .to_path_buf()
}

fn ess(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .current_dir(workspace_root())
        .output()
        .expect("the ess binary runs")
}

/// The long help sends the reader to the version history page for `ess-inputs/N`, `ess-ui/N`,
/// `ess-composition/N` and `ess-scenario/N` "with their releases". Every family it names there has
/// to appear on that page with at least one version.
#[test]
fn every_family_the_help_sends_to_the_history_page_is_on_it() {
    let output = ess(&["specify", "formats", "--help"]);
    assert!(output.status.success(), "{output:?}");
    let help = String::from_utf8(output.stdout).expect("UTF-8");
    let families: Vec<&str> = help
        .split('`')
        .skip(1)
        .step_by(2)
        .filter_map(|span| span.strip_suffix("/N"))
        .filter(|family| !family.contains(char::is_whitespace) && *family != "ess")
        .collect();
    assert!(
        families.contains(&"ess-ui"),
        "the help names the families it defers: {help}"
    );
    let page =
        std::fs::read_to_string(workspace_root().join("website/docs/reference/spec-versions.md"))
            .expect("the version history page");
    let missing: Vec<&str> = families
        .iter()
        .copied()
        .filter(|family| {
            !page
                .match_indices(&format!("`{family}/"))
                .any(|(at, found)| {
                    page[at + found.len()..]
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_digit())
                })
        })
        .collect();
    assert_eq!(
        missing,
        Vec::<&str>::new(),
        "`ess specify formats --help` says the version history page lists these families with \
         their releases, and the page names no version of them"
    );
}

/// `--since` the newest format, or past it, prints no format and succeeds, in every rendering.
#[test]
fn since_at_or_past_the_newest_prints_nothing_and_succeeds() {
    let newest = ess_domain::system::SUPPORTED_FORMATS
        .iter()
        .max()
        .expect("a format");
    for since in [format!("ess/{newest}"), format!("ess/{}", newest + 76)] {
        let text = ess(&["specify", "formats", "--since", &since]);
        assert_eq!(text.status.code(), Some(0), "--since {since}");
        assert_eq!(String::from_utf8_lossy(&text.stdout), "", "--since {since}");
        for format in ["json", "yaml"] {
            let output = ess(&["specify", "formats", "--since", &since, "--format", format]);
            assert_eq!(output.status.code(), Some(0), "--since {since} {format}");
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).trim(),
                "[]",
                "--since {since} --format {format}"
            );
        }
    }
}

/// The published pattern `^ess/[1-9][0-9]*$` refuses a leading zero and a sign, and so does
/// `--since`.
#[test]
fn since_is_held_to_the_published_pattern() {
    for refused in ["ess/01", "ess/+1", "ESS/1", "ess/ 1", "ess/"] {
        let output = ess(&["specify", "formats", "--since", refused]);
        assert_eq!(output.status.code(), Some(2), "--since {refused}");
        assert!(output.stdout.is_empty(), "--since {refused}");
    }
}
