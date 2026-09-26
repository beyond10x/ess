//! An empty `OpenAPI` or `AsyncAPI` projection says why it is empty (ess#102).
//!
//! Both projections write one document per component. A specification whose domains no component
//! owns therefore projects to `0 artifact(s)` — legal, and until this note indistinguishable from a
//! clean run. Every domain no component owns is named on stderr with the declaration that is
//! missing; the exit stays 0, and `--strict` turns the same condition into a refusal that writes
//! nothing.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

/// The billing example, with or without its `components.yaml` and `topology.yaml`.
fn specification(with_components: bool) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = repo().join(format!(
        "target/empty-projection/{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(root.join("spec/domains")).unwrap();
    let mut files = vec!["system.yaml", "domains/invoice.yaml", "domains/email.yaml"];
    if with_components {
        files.extend(["components.yaml", "topology.yaml"]);
    }
    for file in files {
        fs::copy(
            repo().join("examples/billing").join(file),
            root.join("spec").join(file),
        )
        .unwrap();
    }
    root
}

fn ess(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const INVOICE: &str = "no component owns billing.invoice; declare it in components.yaml";
const EMAIL: &str = "no component owns billing.email; declare it in components.yaml";

/// Every spelling that selects a per-component projection, including the all-projection default.
const SELECTIONS: [&[&str]; 5] = [
    &["generate", "--path", "spec", "--kind", "openapi"],
    &["generate", "--path", "spec", "--kind", "asyncapi"],
    &[
        "generate", "generate", "--path", "spec", "--kind", "openapi",
    ],
    &["generate", "--path", "spec"],
    &["generate", "project", "openapi", "--path", "spec"],
];

#[test]
fn an_unowned_domain_is_named_on_stderr_and_the_empty_projection_still_exits_zero() {
    let root = specification(false);
    for selection in SELECTIONS {
        let output = ess(&root, selection);
        let err = stderr(&output);
        assert_eq!(output.status.code(), Some(0), "{selection:?}: {err}");
        assert!(err.contains(INVOICE), "{selection:?}: {err}");
        assert!(err.contains(EMAIL), "{selection:?}: {err}");
        // The note is not part of the machine-readable stream.
        assert!(
            !String::from_utf8_lossy(&output.stdout).contains("no component owns"),
            "{selection:?}"
        );
    }
}

#[test]
fn strict_refuses_an_unowned_domain_with_exit_one_and_writes_nothing() {
    let root = specification(false);
    for (index, selection) in SELECTIONS.iter().enumerate() {
        let out = format!("out-{index}");
        let mut args = selection.to_vec();
        args.extend(["--strict", "--out", &out]);
        let output = ess(&root, &args);
        let err = stderr(&output);
        assert_eq!(output.status.code(), Some(1), "{selection:?}: {err}");
        assert!(err.contains(INVOICE), "{selection:?}: {err}");
        assert!(err.contains(EMAIL), "{selection:?}: {err}");
        assert!(!root.join(&out).exists(), "{selection:?} wrote {out}");
    }
}

#[test]
fn owned_domains_project_without_a_note_and_pass_strict() {
    let root = specification(true);
    for selection in SELECTIONS {
        let mut args = selection.to_vec();
        args.push("--strict");
        let output = ess(&root, &args);
        let err = stderr(&output);
        assert_eq!(output.status.code(), Some(0), "{selection:?}: {err}");
        assert!(!err.contains("no component owns"), "{selection:?}: {err}");
    }
}

#[test]
fn a_projection_that_does_not_read_components_carries_no_note() {
    let root = specification(false);
    for kind in ["docs", "docs-ir", "schema"] {
        let output = ess(
            &root,
            &["generate", "--path", "spec", "--kind", kind, "--strict"],
        );
        let err = stderr(&output);
        assert_eq!(output.status.code(), Some(0), "{kind}: {err}");
        assert!(!err.contains("no component owns"), "{kind}: {err}");
    }
}
