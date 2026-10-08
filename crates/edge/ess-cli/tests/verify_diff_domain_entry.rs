//! beyond10x/ess#469: adding a domain to `system.yaml` passes `--fail-on breaking-or-unknown`,
//! named as `domain/<domain>/added`; removing one fails `--fail-on breaking`; reordering the
//! `domains:` list is no change.
//!
//! The issue's reproducer, run through the binary: before the fix the added entry was
//! `system/warehouse/unclassified-changed`, unknown, and the gate exited 4.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const SHIPMENT: &str = "domain: warehouse.shipment
entities:
  - name: warehouse.shipment.Shipment
    identity: {name: shipment_id, type: Uuid}
    fields:
      - {name: destination, type: String}
    lifecycle: {initial: Draft, states: [Draft], terminal: [Draft]}
";

const BILLING: &str = "domain: warehouse.billing
entities:
  - name: warehouse.billing.Invoice
    identity: {name: invoice_id, type: Uuid}
    fields:
      - {name: amount_cents, type: Integer}
    lifecycle: {initial: Draft, states: [Draft], terminal: [Draft]}
";

const ADDED: &str = "domain/warehouse.billing/added";
const REMOVED: &str = "domain/warehouse.billing/removed";
const UNCLASSIFIED: &str = "system/warehouse/unclassified-changed";

fn system(domains: &[&str]) -> String {
    let mut text = String::from("format: ess/14\nsystem: warehouse\nversion: v1\ndomains:\n");
    for domain in domains {
        text.push_str("  - ");
        text.push_str(domain);
        text.push('\n');
    }
    text
}

/// One revision of the reproducer, under a directory of the calling test's own: the tests of this
/// binary run in parallel and must not rewrite each other's inputs.
fn revision(test: &str, name: &str, domains: &[&str]) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("verify-diff-domain-entry-{}", std::process::id()))
        .join(test)
        .join(name);
    std::fs::create_dir_all(root.join("domains")).unwrap();
    std::fs::write(root.join("system.yaml"), system(domains)).unwrap();
    std::fs::write(root.join("domains/shipment.yaml"), SHIPMENT).unwrap();
    if domains.contains(&"warehouse.billing") {
        std::fs::write(root.join("domains/billing.yaml"), BILLING).unwrap();
    }
    root
}

fn before(test: &str) -> PathBuf {
    revision(test, "before", &["warehouse.shipment"])
}

fn after(test: &str) -> PathBuf {
    revision(test, "after", &["warehouse.shipment", "warehouse.billing"])
}

fn diff(from: &Path, to: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "diff", "--from"])
        .arg(from)
        .arg("--to")
        .arg(to)
        .args(extra)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn an_added_domain_passes_the_breaking_or_unknown_gate_as_a_compatible_change() {
    let test = "added";
    let output = diff(
        &before(test),
        &after(test),
        &["--fail-on", "breaking-or-unknown", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let delta: Value = serde_json::from_slice(&output.stdout).unwrap();
    let changes = delta["changes"].as_array().unwrap();
    assert!(
        changes.iter().all(|change| change["id"] != UNCLASSIFIED),
        "{delta:#}"
    );
    let added = changes
        .iter()
        .find(|change| change["id"] == ADDED)
        .unwrap_or_else(|| panic!("`{ADDED}` in {delta:#}"));
    for dimension in ["verdict", "callers", "readers", "history"] {
        assert_eq!(added["compatibility"][dimension], "compatible", "{added:#}");
    }
}

#[test]
fn a_removed_domain_fails_the_breaking_gate_by_name() {
    let test = "removed";
    let output = diff(&after(test), &before(test), &["--fail-on", "breaking"]);
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    let said = stderr(&output);
    assert!(
        said.contains(&format!("fails --fail-on breaking: {REMOVED}")),
        "{said}"
    );
}

#[test]
fn reordering_the_domains_list_is_no_change() {
    let test = "reordered";
    let reordered = revision(
        test,
        "reordered",
        &["warehouse.billing", "warehouse.shipment"],
    );
    let output = diff(
        &after(test),
        &reordered,
        &["--fail-on", "breaking-or-unknown"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let printed = String::from_utf8(output.stdout).unwrap();
    assert!(printed.contains("no semantic change"), "{printed}");
}
