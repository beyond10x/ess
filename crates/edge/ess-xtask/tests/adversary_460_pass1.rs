//! Adversary pass 1 for beyond10x/ess#460: the generated `ess/` table of the version history.
//!
//! `format_history.rs` lives in the `ess-xtask` binary, so it is compiled here by path, the way the
//! binary already borrows `git_checkout.rs` from `ess-cli`.

use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[path = "../src/format_history.rs"]
mod format_history;

use ess_domain::system::FORMAT_HISTORY;

/// What the borrowed module's own tests call as `crate::workspace_root()`.
fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            std::fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| "no workspace root".to_owned())
}

fn committed() -> String {
    std::fs::read_to_string(
        workspace_root()
            .expect("workspace root")
            .join(format_history::PAGE),
    )
    .expect("the version history page")
}

/// Two different releases must not link the same reference: the page's `[rNN]:` lines are one URL
/// each, so a shared label sends one release's cell to the other release's notes.
#[test]
fn two_releases_never_share_a_reference_label() {
    for (one, other) in [
        ("1.0.0", "0.10.0"),
        ("0.4.61", "0.46.1"),
        ("1.2.0", "0.12.0"),
    ] {
        assert_ne!(
            format_history::release_cell(Some(one)).split('[').nth(2),
            format_history::release_cell(Some(other)).split('[').nth(2),
            "{one} and {other} link the same reference label"
        );
    }
}

/// Every row of the generated table, not only `ess/1`, is held by `--check`.
#[test]
fn a_hand_edit_of_any_row_is_refused() {
    let page = committed();
    let block = format_history::render(FORMAT_HISTORY);
    format_history::compare(&page, &block).expect("the committed page is current");
    for entry in FORMAT_HISTORY {
        let prefix = format!("| `ess/{}` |", entry.major);
        let row = page
            .lines()
            .find(|line| line.starts_with(&prefix))
            .unwrap_or_else(|| panic!("no ess/{} row", entry.major));
        let edited = page.replacen(row, &row.replacen(" | ", " |  ", 1), 1);
        assert_ne!(edited, page);
        assert!(
            format_history::compare(&edited, &block).is_err(),
            "a hand edit of ess/{} passes --check",
            entry.major
        );
    }
}

/// The base page said `ess/23` was introduced in 0.54.0 while the catalogue carries no release;
/// `--check` refuses that cell.
///
/// A release commit leaves no row unreleased, so the catalogue between releases is the live one
/// with its newest release cleared, and the page is the committed one agreeing with it.
#[test]
fn the_base_release_claim_for_an_unreleased_format_is_refused() {
    let mut history = FORMAT_HISTORY.to_vec();
    let unreleased = history.last_mut().expect("a format");
    unreleased.release = None;
    let major = unreleased.major;
    let block = format_history::render(&history);
    let row = |rendered: &str| {
        rendered
            .lines()
            .find(|line| line.starts_with(&format!("| `ess/{major}` |")))
            .map(str::to_owned)
            .expect("the newest row")
    };
    let page = committed().replacen(
        &row(&format_history::render(FORMAT_HISTORY)),
        &row(&block),
        1,
    );
    format_history::compare(&page, &block).expect("the page agrees with the unreleased catalogue");
    let prefix = format!("| `ess/{major}` | unreleased |");
    assert!(page.contains(&prefix), "{prefix}");
    let claimed = page.replacen(&prefix, &format!("| `ess/{major}` | [0.54.0][r54] |"), 1);
    assert!(format_history::compare(&claimed, &block).is_err());
}
