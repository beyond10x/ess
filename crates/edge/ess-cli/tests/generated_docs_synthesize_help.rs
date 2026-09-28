//! `ess verify conform synthesize --help` describes the suite version `--target ir` writes as it is
//! chosen, not as one fixed number (beyond10x/ess#186).
//!
//! The help once said "The canonical `ess-conformance/1` document" while a plain `ess/1`
//! specification wrote `ess-conformance/4`, and a specification using newer constructs writes a
//! higher number still: the synthesizer picks the version from the constructs the suite holds.

use std::path::{Path, PathBuf};
use std::process::Command;

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

fn help() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "synthesize", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("the ess binary runs");
    assert!(output.status.success(), "`synthesize --help` failed");
    String::from_utf8(output.stdout).expect("the help is UTF-8")
}

/// Every `ess-conformance/<digit>` the text spells, as written.
fn fixed_suite_versions(text: &str) -> Vec<String> {
    let prefix = "ess-conformance/";
    text.match_indices(prefix)
        .filter_map(|(at, _)| {
            let digits: String = text[at + prefix.len()..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            (!digits.is_empty()).then(|| format!("{prefix}{digits}"))
        })
        .collect()
}

#[test]
fn generated_docs_ir_target_help_names_no_fixed_suite_version() {
    let help = help();
    assert_eq!(
        fixed_suite_versions(&help),
        Vec::<String>::new(),
        "the help names a fixed suite version the synthesizer does not always write:\n{help}"
    );
}

#[test]
fn generated_docs_ir_target_help_names_the_rule_that_picks_the_version() {
    let help = help();
    let flat = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains("chosen by the constructs the specification uses"),
        "the `ir` target does not say how its suite version is chosen:\n{help}"
    );
}
