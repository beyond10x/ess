//! Second adversarial pass on `story:concurrent-history-lanes`: `ess verify conform web --history`
//! on a history `check-history` admits and judges.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

const REGISTER: &str = "crates/verify/ess-conformance/tests/fixtures/register/register.yaml";
const LINEARIZABLE: &str =
    "crates/verify/ess-conformance/tests/fixtures/register/linearizable.json";

#[test]
fn the_largest_admitted_client_count_renders_as_it_checks() {
    // `history.rs` admits `clients` up to MAX_INTEGER (2^53 - 1); `check-history` judges such a
    // history. The page draws a lane for every client in 0..clients.
    let text = std::fs::read_to_string(root().join(LINEARIZABLE)).expect("the fixture reads");
    let huge = text.replacen("\"clients\": 2", "\"clients\": 9007199254740991", 1);
    assert_ne!(huge, text, "the fixture declares two clients");
    let directory = tempfile::tempdir().expect("a scratch directory");
    let history = directory.path().join("huge.json");
    std::fs::write(&history, huge).expect("written");
    let history = history.display().to_string();

    let checked = ess(&[
        "verify",
        "conform",
        "check-history",
        "--path",
        REGISTER,
        "--history",
        &history,
    ]);
    assert_eq!(
        checked.status.code(),
        Some(0),
        "check-history admits and judges it: {}",
        String::from_utf8_lossy(&checked.stdout)
    );

    let drawn = ess(&[
        "verify",
        "conform",
        "web",
        "--path",
        REGISTER,
        "--history",
        &history,
    ]);
    assert!(
        drawn.status.code().is_some(),
        "`web --history` exits rather than dying by signal ({:?}): {}",
        drawn.status,
        String::from_utf8_lossy(&drawn.stderr)
    );
}
