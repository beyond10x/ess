//! `ess generate` stamps each file with the digest of its own inputs and records the whole model's
//! digest once, in `.ess-output`.
//!
//! A file that stamped the whole model's digest was rewritten by every change anywhere in the
//! model. Through the CLI: a change to one context's entity leaves the other context's pages byte
//! for byte, `--check` names the page whose inputs moved and not the one whose inputs stood, and
//! the run records the model it generated from on its owner in `.ess-output/state.json`.

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const SYSTEM: &str = r"format: ess/1
system: shelf
version: v1

domains:
  - shelf.orders
  - shelf.billing
";

const BILLING: &str = r"domain: shelf.billing

types:
  - name: shelf.billing.InvoiceId
    kind: newtype
    of: Uuid

entities:
  - name: shelf.billing.Invoice
    identity:
      name: invoice_id
      type: shelf.billing.InvoiceId
    fields:
      - name: total
        type: Integer
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
";

/// The orders context, its one entity's `note` field typed as given.
fn orders(note: &str) -> String {
    format!(
        r"domain: shelf.orders

types:
  - name: shelf.orders.OrderId
    kind: newtype
    of: Uuid

entities:
  - name: shelf.orders.Order
    identity:
      name: order_id
      type: shelf.orders.OrderId
    fields:
      - name: note
        type: {note}
    lifecycle:
      initial: Placed
      states: [Placed]
      terminal: [Placed]
"
    )
}

fn write_model(root: &Path, note: &str) {
    let model = root.join("model");
    fs::create_dir_all(&model).unwrap();
    fs::write(model.join("system.yaml"), SYSTEM).unwrap();
    fs::write(model.join("billing.yaml"), BILLING).unwrap();
    fs::write(model.join("orders.yaml"), orders(note)).unwrap();
}

fn ess(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The compiled-model digest `ess verify diff` reports for the model as it stands.
fn model_digest(root: &Path) -> String {
    let output = ess(
        root,
        &[
            "verify", "diff", "--from", "model", "--to", "model", "--format", "json",
        ],
    );
    assert!(output.status.success(), "{}", text(&output));
    let diff: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    diff["after"]["spec_digest"]
        .as_str()
        .unwrap_or_else(|| panic!("no digest in {diff}"))
        .trim_start_matches("sha256:")
        .to_owned()
}

/// The model digest `.ess-output` records on the docs projection's owner.
fn recorded(out: &Path) -> String {
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(out.join(".ess-output/state.json")).unwrap()).unwrap();
    assert_eq!(state["payload"]["format"], "ess-output-state/4", "{state}");
    let owners = state["payload"]["checkpoint"]["ledger"]["owners"]
        .as_array()
        .unwrap();
    assert_eq!(owners.len(), 1, "{state}");
    owners[0]["model"]
        .as_str()
        .unwrap_or_else(|| panic!("no model digest recorded: {state}"))
        .to_owned()
}

const GENERATE: &[&str] = &[
    "generate", "--path", "model", "--kind", "docs", "--out", "out",
];

#[test]
fn a_change_in_one_context_moves_only_its_pages_and_the_record() {
    let root = TemporaryDirectory::create("ess-per-file-digest").unwrap();
    write_model(&root, "String");
    let first = ess(&root, GENERATE);
    assert!(first.status.success(), "{}", text(&first));
    let out = root.join("out");
    let billing = out.join("docs/domains/shelf-billing.md");
    let orders = out.join("docs/domains/shelf-orders.md");
    let (billing_before, orders_before) = (fs::read(&billing).unwrap(), fs::read(&orders).unwrap());
    let digest_before = model_digest(&root);
    assert_eq!(recorded(&out), digest_before, "the run records its model");
    for path in [&billing, &orders] {
        let page = fs::read_to_string(path).unwrap();
        assert!(
            !page.contains(&digest_before) && !page.contains("model digest"),
            "{} stamps the whole-model digest",
            path.display()
        );
    }

    write_model(&root, "Integer");
    let digest_after = model_digest(&root);
    assert_ne!(digest_before, digest_after, "the model changed");

    // `--check` compares the bytes generation would write: the page whose inputs moved is stale,
    // the page whose inputs stood is not, and the record names the model it no longer is.
    let mut check = GENERATE.to_vec();
    check.push("--check");
    let checked = ess(&root, &check);
    let err = String::from_utf8_lossy(&checked.stderr).into_owned();
    assert_eq!(checked.status.code(), Some(1), "{}", text(&checked));
    assert!(
        err.lines()
            .any(|line| line.contains("shelf-orders.md") && line.contains("differs")),
        "--check does not name the page whose inputs moved: {err}"
    );
    assert!(
        !err.contains("shelf-billing.md"),
        "--check names a page whose inputs stood: {err}"
    );
    assert!(
        err.contains("state.json records another generation"),
        "--check does not name the record: {err}"
    );

    let second = ess(&root, GENERATE);
    assert!(second.status.success(), "{}", text(&second));
    assert_eq!(
        fs::read(&billing).unwrap(),
        billing_before,
        "the billing page was rewritten by a change to an orders entity"
    );
    assert_ne!(
        fs::read(&orders).unwrap(),
        orders_before,
        "the orders page stood while its entity moved"
    );
    assert_eq!(
        recorded(&out),
        digest_after,
        "the record names the new model"
    );
    let current = ess(&root, &check);
    assert_eq!(current.status.code(), Some(0), "{}", text(&current));
}
