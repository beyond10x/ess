//! beyond10x/ess#479 through the binary: the issue's `catalog` reproducer, with an authored
//! `ess-scenario/1` that sends `item_id: ""` and requires `invalid-item-id`, synthesizes with no
//! refusal and counts the authored scenario.
//!
//! On 0.55.0 the run printed ESS-SYNTH-019 for `invalid-item-id` whether or not the authored
//! scenario was selected. With `--scenarios` it was counted ("3 scenario(s) (1 authored), 1
//! refusal(s)"); with `--path` alone no authored scenario is selected, as `--scenarios` documents.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const SYSTEM: &str = "format: ess/22
system: catalog
version: v1

domains:
  - catalog.items
";

/// The issue's `domains/items.yaml`.
const ITEMS: &str = r#"domain: catalog.items

summary: Items, each kept under an id of 1 to 170 UTF-8 bytes.

naming:
  wire: items
  display: Items

types:
  - name: catalog.items.ItemId
    kind: newtype
    of: String

entities:
  - name: catalog.items.Item
    identity:
      name: item_id
      type: catalog.items.ItemId
    fields:
      - name: title
        type: String
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
      transitions: []

errors:
  - name: catalog.items.ItemExists
    summary: An item already has this id.
    fields:
      - name: item_id
        type: catalog.items.ItemId

  - name: catalog.items.InvalidItemId
    summary: The item id is empty or longer than 170 UTF-8 bytes.
    fields:
      - name: item_id
        type: catalog.items.ItemId

commands:
  - name: catalog.items.CreateItem
    naming:
      wire: create-item
      display: Create an item
    input:
      - name: item_id
        type: catalog.items.ItemId
      - name: title
        type: String
    outcomes:
      - name: invalid-item-id
        when:
          any:
            - item_id == ""
        error: catalog.items.InvalidItemId
        payload:
          catalog.items.InvalidItemId:
            item_id: input.item_id
        summary: The id cannot be kept, so nothing was created.
      - name: created
        creates: catalog.items.Item
        instance: item_id
        sets:
          title: input.title
        emits:
          - catalog.items.ItemCreated
        payload:
          catalog.items.ItemCreated:
            item_id: input.item_id
            title: input.title
      - name: already-created
        existing_instance: true
        error: catalog.items.ItemExists
        payload:
          catalog.items.ItemExists:
            item_id: input.item_id
        summary: An item already has this id.

events:
  - name: catalog.items.ItemCreated
    fields:
      - name: item_id
        type: catalog.items.ItemId
      - name: title
        type: String

views:
  - name: catalog.items.Items
    source: catalog.items.Item
    consistency: read_your_writes
    fields:
      - name: item_id
        type: catalog.items.ItemId
      - name: state
        type: catalog.items.Item.State
      - name: title
        type: String
    naming:
      wire: items
      display: Items
"#;

/// The authored scenario covering `invalid-item-id`.
const AUTHORED: &str = r#"type: ess-scenario/1
domain: catalog.items
scenario: empty-item-id-is-refused
summary: An empty item id is refused and nothing is created.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: catalog.items.CreateItem
    input:
      item_id: ""
      title: a title
    outcome: invalid-item-id
"#;

const INPUTS: &str = "format: ess-inputs/1
specification: [system.yaml, domains/items.yaml]
scenarios: [scenarios/empty-item-id.yaml]
";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

/// The reproducer and its authored scenario in a directory of their own under `target/`.
fn fixture() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = repo().join(format!(
        "target/authored-beside-existing-instance/{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    for (path, text) in [
        ("system.yaml", SYSTEM),
        ("domains/items.yaml", ITEMS),
        ("scenarios/empty-item-id.yaml", AUTHORED),
        ("ess-inputs.yaml", INPUTS),
    ] {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
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

#[test]
fn the_authored_refusal_is_counted_and_nothing_is_refused() {
    let root = fixture();
    let validated = ess(&root, &["specify", "validate", "--path", "."]);
    assert!(
        validated.status.success(),
        "the reproducer validates: {validated:?}"
    );

    let synthesized = ess(
        &root,
        &[
            "verify",
            "conform",
            "synthesize",
            "--path",
            ".",
            "--scenarios",
            ".",
            "--out",
            "suite.json",
        ],
    );
    let stdout = String::from_utf8_lossy(&synthesized.stdout);
    assert!(synthesized.status.success(), "{synthesized:?}");
    assert!(
        !stdout.contains("refusal["),
        "nothing is refused:\n{stdout}"
    );
    assert!(
        stdout.contains("(1 authored), 0 refusal(s)"),
        "the authored scenario is counted and nothing is refused:\n{stdout}"
    );
    let suite = fs::read_to_string(root.join("suite.json")).unwrap();
    for id in [
        "catalog.items/authored/empty-item-id-is-refused",
        "catalog.items.CreateItem/outcome/invalid-item-id",
        "catalog.items.CreateItem/outcome/already-created",
    ] {
        assert!(
            suite.contains(&format!("\"{id}\"")),
            "the suite holds `{id}`"
        );
    }
}
