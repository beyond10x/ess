//! Adversary cases for story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs, read
//! against the acceptance's own example: "`ess specify validate` exit 0 on `examples/billing`
//! re-declared at `ess/37`; IR bytes equal apart from the format field". The unit's
//! `epic_formats_open.rs` holds that claim on another fixture, so nothing ran the example named.

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn billing() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// `examples/billing`, its header re-declared at `format`.
fn billing_at(directory: &Path, format: &str) -> PathBuf {
    let target = directory.join(format.replace('/', "-"));
    copy(&billing(), &target);
    let system = target.join("system.yaml");
    let text = fs::read_to_string(&system).unwrap();
    let moved = text.replacen("format: ess/1\n", &format!("format: {format}\n"), 1);
    assert_ne!(moved, text, "examples/billing declares format: ess/1");
    fs::write(system, moved).unwrap();
    target
}

fn validate(path: &Path) -> Output {
    ess(&["specify", "validate", "--path", path.to_str().unwrap()])
}

/// What the acceptance means: billing at `ess/37` is judged exactly as at `ess/36`, refusal for
/// refusal, apart from the directory and the format it names.
#[test]
fn examples_billing_at_ess_37_is_judged_exactly_as_at_ess_36() {
    let directory = TemporaryDirectory::create("adv-t1-billing-36-37").unwrap();
    let older = billing_at(&directory, "ess/36");
    let newer = billing_at(&directory, "ess/37");
    let said = |path: &Path| {
        let output = validate(path);
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        (
            output.status.code(),
            text.replace(path.to_str().unwrap(), "<billing>")
                .replace("ess/36", "ess/N")
                .replace("ess/37", "ess/N"),
        )
    };
    assert_eq!(said(&older), said(&newer));
}
