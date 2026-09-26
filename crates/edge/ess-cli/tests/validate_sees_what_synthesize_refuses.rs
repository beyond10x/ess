//! `ess specify validate` is not green where a later step refuses (ess#112).
//!
//! Two shapes of the same failure. An `ess-inputs.yaml` that lists an authored scenario whose actor
//! may not run its command validated clean, and `synthesize --scenarios` then refused it with
//! `ESS-AUTHOR-009`: validate read the specification list and never the scenario list. And an entity
//! invariant could read a required field that a `creates:` branch never sets, which validated and
//! then held or failed on whatever value the implementation picked.

use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const MODEL_FILES: [&str; 5] = [
    "system.yaml",
    "domains/invoice.yaml",
    "domains/email.yaml",
    "components.yaml",
    "topology.yaml",
];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

struct Fixture(PathBuf);

impl Fixture {
    /// The billing example under `model/`, and an `ess-inputs.yaml` listing it and `scenarios`.
    fn new(scenarios: &[(&str, String)]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = repo().join(format!(
            "target/validate-sees-what-synthesize-refuses/{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let fixture = Self(root);
        for path in MODEL_FILES {
            fixture.write(
                &format!("model/{path}"),
                fs::read(repo().join("examples/billing").join(path)).unwrap(),
            );
        }
        for (path, text) in scenarios {
            fixture.write(path, text);
        }
        let listed = scenarios
            .iter()
            .map(|(path, _)| format!("\"{path}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let specification = MODEL_FILES
            .iter()
            .map(|path| format!("\"model/{path}\""))
            .collect::<Vec<_>>()
            .join(", ");
        fixture.write(
            "ess-inputs.yaml",
            format!(
                "format: ess-inputs/1\nspecification: [{specification}]\nscenarios: [{listed}]\n"
            ),
        );
        fixture
    }

    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn ess(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}

/// The billing example's authored scenario, renamed so two copies do not collide.
fn scenario(name: &str) -> String {
    fs::read_to_string(
        repo().join("examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml"),
    )
    .unwrap()
    .replace(
        "scenario: outstanding-invoices-rank-latest-first",
        &format!("scenario: {name}"),
    )
}

/// The same scenario, run as an actor whose `may:` does not include `CreateInvoice`.
fn ungranted(name: &str) -> String {
    scenario(name).replace(
        "actor: billing.invoice.Customer",
        "actor: billing.invoice.Auditor",
    )
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn validate_refuses_a_listed_scenario_that_synthesize_refuses() {
    let fixture = Fixture::new(&[
        ("authored/first.yaml", scenario("first")),
        ("authored/viewer.yaml", ungranted("viewer")),
    ]);

    let synthesized = fixture.ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        ".",
        "--scenarios",
        ".",
        "--target",
        "ir",
    ]);
    assert!(
        !synthesized.status.success() && text(&synthesized.stderr).contains("ESS-AUTHOR-009"),
        "the premise: synthesize refuses the scenario: {synthesized:?}"
    );

    let validated = fixture.ess(&["specify", "validate", "--path", "."]);
    assert_eq!(
        validated.status.code(),
        Some(1),
        "a refused authored scenario fails validate: {validated:?}"
    );
    let stderr = text(&validated.stderr);
    assert!(stderr.contains("refusal[ESS-AUTHOR-009]"), "{stderr}");
    assert!(
        stderr.contains("authored/viewer.yaml"),
        "names the file: {stderr}"
    );
    assert!(
        !text(&validated.stdout).contains("valid"),
        "no success line: {validated:?}"
    );
}

#[test]
fn validate_counts_the_listed_scenarios_it_checked() {
    let fixture = Fixture::new(&[
        ("authored/first.yaml", scenario("first")),
        ("authored/second.yaml", scenario("second")),
    ]);
    let validated = fixture.ess(&["specify", "validate", "--path", "."]);
    assert!(validated.status.success(), "{validated:?}");
    assert_eq!(
        text(&validated.stdout),
        "billing v3 — 5 file(s), 2 scenario(s), valid\n"
    );

    let json = fixture.ess(&["specify", "validate", "--path", ".", "--format", "json"]);
    assert!(json.status.success(), "{json:?}");
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["valid"], true, "{report}");
    assert_eq!(report["scenarios"], 2, "{report}");
}

#[test]
fn a_refusal_in_json_is_reported_as_invalid_with_its_code() {
    let fixture = Fixture::new(&[("authored/viewer.yaml", ungranted("viewer"))]);
    let json = fixture.ess(&["specify", "validate", "--path", ".", "--format", "json"]);
    assert_eq!(json.status.code(), Some(1), "{json:?}");
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["valid"], false, "{report}");
    assert_eq!(report["scenarios"], 1, "{report}");
    assert_eq!(
        report["scenario_refusals"][0]["code"], "ESS-AUTHOR-009",
        "{report}"
    );
    // The origin `synthesize --scenarios .` prints for the same file.
    assert_eq!(
        report["scenario_refusals"][0]["origin"], "./authored/viewer.yaml",
        "{report}"
    );
}

#[test]
fn no_listed_scenarios_says_nothing_new() {
    let fixture = Fixture::new(&[]);
    let validated = fixture.ess(&["specify", "validate", "--path", "."]);
    assert!(validated.status.success(), "{validated:?}");
    assert_eq!(text(&validated.stdout), "billing v3 — 5 file(s), valid\n");
    let json = fixture.ess(&["specify", "validate", "--path", ".", "--format", "json"]);
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert!(report.get("scenarios").is_none(), "{report}");
    assert!(report.get("scenario_refusals").is_none(), "{report}");

    // A directory without a manifest, and a single file, list no scenarios either.
    let direct = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["specify", "validate", "--path"])
        .arg(repo().join("examples/billing"))
        .output()
        .unwrap();
    assert!(direct.status.success(), "{direct:?}");
    assert_eq!(text(&direct.stdout), "billing v3 — 5 file(s), valid\n");
}

const UNSET: &str = "format: ess/4
system: shop
version: v1
domain: shop.orders
entities:
  - name: shop.orders.Order
    identity:
      name: order_id
      type: Uuid
    fields:
      - name: items
        type: Integer
    invariants:
      - items >= 0
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - name: close
          from: [Open]
          to: Closed
events:
  - name: shop.orders.OrderPlaced
    fields:
      - name: order_id
        type: Uuid
  - name: shop.orders.OrderClosed
    fields: []
commands:
  - name: shop.orders.PlaceOrder
    outcomes:
      - name: placed
        creates: shop.orders.Order
        instance: order_id
        emits: [shop.orders.OrderPlaced]
        payload:
          shop.orders.OrderPlaced:
            order_id: {generated: true}
  - name: shop.orders.CloseOrder
    input:
      - name: order_id
        type: Uuid
    outcomes:
      - name: closed
        moves: shop.orders.Order.close
        instance: order_id
        emits: [shop.orders.OrderClosed]
";

#[test]
fn validate_refuses_an_invariant_over_a_field_no_create_sets() {
    let fixture = Fixture::new(&[]);
    fixture.write("unset.yaml", UNSET);
    let validated = fixture.ess(&["specify", "validate", "--path", "unset.yaml"]);
    assert_eq!(validated.status.code(), Some(1), "{validated:?}");
    let stderr = text(&validated.stderr);
    assert!(stderr.contains("error[ESS-COMMAND-018]"), "{stderr}");
    assert!(stderr.contains("sets: {items:"), "{stderr}");
    assert!(stderr.contains("Optional<Integer>"), "{stderr}");
    assert!(
        stderr.contains("unset.yaml:"),
        "cites the file and line: {stderr}"
    );

    fixture.write(
        "set.yaml",
        UNSET.replace(
            "            order_id: {generated: true}\n",
            "            order_id: {generated: true}\n        sets:\n          items: \"0\"\n",
        ),
    );
    let repaired = fixture.ess(&["specify", "validate", "--path", "set.yaml"]);
    assert!(repaired.status.success(), "{repaired:?}");
}
