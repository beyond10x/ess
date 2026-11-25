//! `ess specify compose` over an `ess-composition/5` document whose conformances name an event, or
//! compare value constraints (`docs/design/composition-type-conformance.md`).
//!
//! The library compares the ends (`crates/specify/ess-composition/tests/payload_conformance.rs`);
//! this holds the command to reading `/5` through it, writing the payload ends into the compiled
//! composition and printing a drift by its code.

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

fn fixture(set: &str, system: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../specify/ess-composition/tests/fixtures")
        .join(set)
        .join(system)
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The compiled-model digest `ess verify diff` reports for one fixture system.
fn digest(set: &str, system: &str) -> String {
    let path = fixture(set, system);
    let path = path.to_str().unwrap();
    let output = ess(&[
        "verify", "diff", "--from", path, "--to", path, "--format", "json",
    ]);
    assert!(output.status.success(), "{}", text(&output));
    let diff: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    diff["after"]["spec_digest"]
        .as_str()
        .unwrap_or_else(|| panic!("no digest in {diff}"))
        .to_owned()
}

/// Composes `rest` under `format` over the fixture set's two systems.
fn compose(directory: &TemporaryDirectory, set: &str, format: &str, rest: &str) -> Output {
    let document = directory.join("composition.yaml");
    fs::write(
        &document,
        format!(
            "format: {format}\n\
             composition: demo-wire\n\
             services:\n  \
               - {{key: demo, system: demo, version: v1, source_digest: {}, component: relay}}\n  \
               - {{key: wire, system: wire, version: v1, source_digest: {}, component: wire-server}}\n\
             {rest}",
            digest(set, "demo"),
            digest(set, "wire"),
        ),
    )
    .unwrap();
    let demo = format!("demo={}", fixture(set, "demo").display());
    let wire = format!("wire={}", fixture(set, "wire").display());
    ess(&[
        "specify",
        "compose",
        "--path",
        document.to_str().unwrap(),
        "--service",
        &demo,
        "--service",
        &wire,
        "--format",
        "json",
    ])
}

#[test]
fn compose_reads_an_event_conformance() {
    let directory = TemporaryDirectory::create("ess-compose-v5-event").unwrap();
    let output = compose(
        &directory,
        "payload-conformance",
        "ess-composition/5",
        "conformances:\n  \
           - local: {service: demo, event: demo.gate.Admitted}\n    \
             conforms_to: {service: wire, event: wire.server.Subscribed}\n",
    );
    assert!(output.status.success(), "{}", text(&output));
    let ir: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(ir["format"], "ess-composition/5");
    assert_eq!(
        ir["conformances"],
        serde_json::json!([{
            "local": {"service": "demo", "event": "demo.gate.Admitted"},
            "conforms_to": {"service": "wire", "event": "wire.server.Subscribed"},
        }])
    );

    let drift = compose(
        &directory,
        "payload-conformance",
        "ess-composition/5",
        "conformances:\n  \
           - local: {service: demo, event: demo.gate.Counted}\n    \
             conforms_to: {service: wire, event: wire.server.Subscribed}\n",
    );
    assert_eq!(drift.status.code(), Some(1), "{}", text(&drift));
    let all = text(&drift);
    assert!(all.contains("type_conformance_drift"), "{all}");
    assert!(all.contains("field `channel`"), "{all}");
}

#[test]
fn compose_exits_non_zero_on_an_invariant_drift_under_v5() {
    let directory = TemporaryDirectory::create("ess-compose-v5-invariant").unwrap();
    let rest = "conformances:\n  \
                - local: {service: demo, type: demo.gate.Code}\n    \
                  conforms_to: {service: wire, type: wire.server.Code}\n";
    let drift = compose(
        &directory,
        "constraint-conformance",
        "ess-composition/5",
        rest,
    );
    assert_eq!(drift.status.code(), Some(1), "{}", text(&drift));
    let all = text(&drift);
    assert!(all.contains("type_conformance_drift"), "{all}");
    assert!(all.contains("invariant"), "{all}");
    // `/4` compares no value constraint, so the same document composes.
    let four = compose(
        &directory,
        "constraint-conformance",
        "ess-composition/4",
        rest,
    );
    assert!(four.status.success(), "{}", text(&four));
}

// `ess specify vendor` (beyond10x/ess#663): the checked copy of another specification's type and
// the composition entries that prove it (`docs/design/composition-type-conformance.md`, "Generating
// the copy").

/// The `vendor` systems, written by each test into its own directory: an owner `example.owner`
/// whose `orders` component owns the vendored type, and a consumer `example.consumer`.
const VENDOR_SYSTEMS: &[(&str, &str)] = &[
    (
        "owner/system.yaml",
        r"format: ess/23
system: example.owner
version: v1

domains:
  - example.owner.orders
  - example.owner.ledger

components:
  - component: orders
    owns:
      domains: [example.owner.orders]
",
    ),
    (
        "owner/domains/orders.yaml",
        r#"domain: example.owner.orders

# The owner's type a consumer vendors, with the closure it reaches.
types:
  - name: example.owner.orders.OrderId
    kind: newtype
    of: String
    prefix: "o-"
    invariants:
      - value != "o-"

  - name: example.owner.orders.Stage
    kind: enum
    attributes:
      - {name: terminal, type: Boolean}
      - {name: label, type: String}
    variants:
      - {name: Placed, wire: placed, attributes: {terminal: false, label: "Placed"}}

  - name: example.owner.orders.OrderSummary
    kind: struct
    fields:
      - {name: id, type: example.owner.orders.OrderId}
      - {name: stage, type: example.owner.orders.Stage, wire: orderStage}
      - {name: note, type: Optional<String>, presence: null_when_absent}
      - {name: lines, type: "List<example.owner.orders.OrderId>"}

  - name: example.owner.orders.Unrelated
    kind: struct
    fields:
      - {name: text, type: String}
"#,
    ),
    (
        "owner/domains/ledger.yaml",
        r"domain: example.owner.ledger

# A domain no component of the owner owns.
types:
  - name: example.owner.ledger.Entry
    kind: struct
    fields:
      - {name: amount, type: Integer}
",
    ),
    (
        "consumer/system.yaml",
        r"format: ess/23
system: example.consumer
version: v1

domains:
  - example.consumer.board

components:
  - component: board
    owns:
      domains: [example.consumer.board]
",
    ),
    (
        "consumer/domains/board.yaml",
        r"domain: example.consumer.board

types:
  - name: example.consumer.board.Card
    kind: struct
    fields:
      - {name: title, type: String}
",
    ),
];

/// The `vendor` system `system`, written under `directory`.
fn vendor_copy(directory: &std::path::Path, system: &str) -> PathBuf {
    for (path, text) in VENDOR_SYSTEMS {
        if let Some(relative) = path.strip_prefix(&format!("{system}/")) {
            let file = directory.join(system).join(relative);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, text).unwrap();
        }
    }
    directory.join(system)
}

/// The compiled-model digest `ess verify diff` reports for the system at `path`.
fn digest_at(path: &std::path::Path) -> String {
    let path = path.to_str().unwrap();
    let output = ess(&[
        "verify", "diff", "--from", path, "--to", path, "--format", "json",
    ]);
    assert!(output.status.success(), "{}", text(&output));
    let diff: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    diff["after"]["spec_digest"].as_str().unwrap().to_owned()
}

/// One `ess specify vendor` call; a `None` flag is left out.
struct Vendor {
    from: Option<String>,
    service: Option<String>,
    component: Option<String>,
    type_name: Option<String>,
    into: Option<String>,
    out: PathBuf,
    conformances: PathBuf,
}

impl Vendor {
    fn new(
        directory: &std::path::Path,
        owner: &std::path::Path,
        consumer: &std::path::Path,
    ) -> Self {
        Self {
            from: Some(format!("owner={}", owner.display())),
            service: Some(format!("consumer={}", consumer.display())),
            component: Some("board".to_owned()),
            type_name: Some("example.owner.orders.OrderSummary".to_owned()),
            into: Some("example.consumer.board".to_owned()),
            out: directory.join("vendored.yaml"),
            conformances: directory.join("conformances.yaml"),
        }
    }

    fn run(&self, extra: &[&str]) -> Output {
        let mut args: Vec<String> = vec!["specify".into(), "vendor".into()];
        for (flag, value) in [
            ("--from", &self.from),
            ("--service", &self.service),
            ("--component", &self.component),
            ("--type", &self.type_name),
            ("--into", &self.into),
        ] {
            if let Some(value) = value {
                args.push(flag.into());
                args.push(value.clone());
            }
        }
        args.push("--out".into());
        args.push(self.out.display().to_string());
        args.push("--conformances".into());
        args.push(self.conformances.display().to_string());
        args.extend(extra.iter().map(|arg| (*arg).to_owned()));
        let args = args.iter().map(String::as_str).collect::<Vec<_>>();
        ess(&args)
    }

    fn written(&self) -> (String, String) {
        (
            fs::read_to_string(&self.out).unwrap(),
            fs::read_to_string(&self.conformances).unwrap(),
        )
    }
}

/// `ess specify compose` of `ess-composition/5` with the written `fragment` over both systems.
fn compose_vendored(
    directory: &std::path::Path,
    owner: &std::path::Path,
    consumer: &std::path::Path,
    fragment: &str,
) -> Output {
    let document = directory.join("composition.yaml");
    fs::write(
        &document,
        format!("format: ess-composition/5\ncomposition: example\n{fragment}"),
    )
    .unwrap();
    let owner = format!("owner={}", owner.display());
    let consumer = format!("consumer={}", consumer.display());
    ess(&[
        "specify",
        "compose",
        "--path",
        document.to_str().unwrap(),
        "--service",
        &owner,
        "--service",
        &consumer,
        "--format",
        "json",
    ])
}

#[test]
fn a_vendored_copy_composes_without_drift() {
    let directory = TemporaryDirectory::create("ess-vendor-composes").unwrap();
    let owner = vendor_copy(&directory, "owner");
    let consumer = vendor_copy(&directory, "consumer");
    let vendor = Vendor::new(&directory, &owner, &consumer);

    let output = vendor.run(&[]);
    assert!(output.status.success(), "{}", text(&output));
    let (source, fragment) = vendor.written();

    let owner_digest = digest_at(&owner);
    let header = source.lines().next().unwrap();
    assert!(header.starts_with('#'), "{source}");
    for named in ["example.owner", "v1", owner_digest.as_str()] {
        assert!(header.contains(named), "header names {named}: {source}");
    }
    assert!(
        source.contains("domain: \"example.consumer.board\""),
        "{source}"
    );
    for name in ["OrderSummary", "OrderId", "Stage"] {
        assert!(
            source.contains(&format!("\"example.consumer.board.{name}\"")),
            "{name}: {source}"
        );
    }
    for kept in ["orderStage", "null_when_absent", "placed", "terminal", "o-"] {
        assert!(source.contains(kept), "{kept}: {source}");
    }
    assert!(!source.contains("Unrelated"), "{source}");
    let declarations = source
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!declarations.contains("example.owner."), "{source}");
    assert_eq!(fragment.matches("conforms_to:").count(), 3, "{fragment}");
    assert!(fragment.contains(&owner_digest), "{fragment}");

    let again = vendor.run(&[]);
    assert!(again.status.success(), "{}", text(&again));
    assert_eq!(vendor.written(), (source.clone(), fragment.clone()));

    fs::write(consumer.join("domains/vendored.yaml"), &source).unwrap();
    assert!(
        fragment.contains(&digest_at(&consumer)),
        "the consumer is pinned with the copy in it: {fragment}"
    );
    let composed = compose_vendored(&directory, &owner, &consumer, &fragment);
    assert!(composed.status.success(), "{}", text(&composed));
    let ir: serde_json::Value = serde_json::from_slice(&composed.stdout).unwrap();
    assert_eq!(ir["conformances"].as_array().map(Vec::len), Some(3), "{ir}");

    let check = vendor.run(&["--check"]);
    assert!(check.status.success(), "{}", text(&check));
    let after = vendor.run(&[]);
    assert!(after.status.success(), "{}", text(&after));
    assert_eq!(vendor.written(), (source, fragment));
}

#[test]
fn an_imported_type_changed_upstream_is_refused_until_repinned() {
    let directory = TemporaryDirectory::create("ess-vendor-repin").unwrap();
    let owner = vendor_copy(&directory, "owner");
    let consumer = vendor_copy(&directory, "consumer");
    let mut vendor = Vendor::new(&directory, &owner, &consumer);
    vendor.out = consumer.join("domains/vendored.yaml");
    // The consumer component defaults to the one that owns `--into`.
    vendor.component = None;

    let output = vendor.run(&[]);
    assert!(output.status.success(), "{}", text(&output));
    let check = vendor.run(&["--check"]);
    assert!(check.status.success(), "{}", text(&check));
    let (source, fragment) = vendor.written();

    let orders = owner.join("domains/orders.yaml");
    let placed =
        "      - {name: Placed, wire: placed, attributes: {terminal: false, label: \"Placed\"}}\n";
    let changed = fs::read_to_string(&orders).unwrap().replace(
        placed,
        &format!(
            "{placed}      - {{name: Shipped, wire: shipped, attributes: {{terminal: true, label: \"Shipped\"}}}}\n"
        ),
    );
    fs::write(&orders, changed).unwrap();

    let stale = compose_vendored(&directory, &owner, &consumer, &fragment);
    assert_eq!(stale.status.code(), Some(1), "{}", text(&stale));

    let check = vendor.run(&["--check"]);
    assert_eq!(check.status.code(), Some(1), "{}", text(&check));
    let all = text(&check);
    assert!(all.contains("example.consumer.board.Stage"), "{all}");
    assert!(
        !all.contains("example.consumer.board.OrderSummary"),
        "{all}"
    );
    assert_eq!(
        vendor.written(),
        (source, fragment),
        "--check writes nothing"
    );

    let repinned = vendor.run(&[]);
    assert!(repinned.status.success(), "{}", text(&repinned));
    let check = vendor.run(&["--check"]);
    assert!(check.status.success(), "{}", text(&check));
    let (source, fragment) = vendor.written();
    assert!(source.contains("shipped"), "{source}");
    let composed = compose_vendored(&directory, &owner, &consumer, &fragment);
    assert!(composed.status.success(), "{}", text(&composed));
}

#[test]
fn vendor_refuses_by_name() {
    let directory = TemporaryDirectory::create("ess-vendor-refusals").unwrap();
    let owner = vendor_copy(&directory, "owner");
    let consumer = vendor_copy(&directory, "consumer");
    let refused = |vendor: &Vendor, code: i32, named: &[&str]| {
        let output = vendor.run(&[]);
        let all = text(&output);
        assert_eq!(output.status.code(), Some(code), "{all}");
        for name in named {
            assert!(all.contains(name), "names {name}: {all}");
        }
        assert!(
            !vendor.out.exists() && !vendor.conformances.exists(),
            "{all}"
        );
    };
    let base = || Vendor::new(&directory, &owner, &consumer);

    let mut vendor = base();
    vendor.type_name = Some("example.owner.orders.Missing".into());
    refused(&vendor, 1, &["example.owner.orders.Missing"]);

    let mut vendor = base();
    vendor.into = Some("Example..board".into());
    refused(&vendor, 1, &["Example..board"]);

    let mut vendor = base();
    vendor.type_name = Some("example.owner.ledger.Entry".into());
    refused(&vendor, 1, &["example.owner.ledger"]);

    let mut vendor = base();
    vendor.from = Some("owner".into());
    refused(
        &vendor,
        1,
        &["expected a service binding written `service-key=path`"],
    );

    let mut vendor = base();
    vendor.service = Some("consumer=".into());
    refused(&vendor, 1, &["a service binding path must not be empty"]);

    let mut vendor = base();
    vendor.service = Some(format!("owner={}", consumer.display()));
    refused(&vendor, 1, &["`owner`"]);

    let mut vendor = base();
    vendor.component = Some("missing".into());
    refused(&vendor, 1, &["`missing`"]);

    let mut vendor = base();
    vendor.component = None;
    vendor.into = Some("example.consumer.archive".into());
    refused(&vendor, 1, &["example.consumer.archive", "--component"]);

    let mut vendor = base();
    vendor.type_name = None;
    refused(&vendor, 2, &["--type"]);

    let clashing = vendor_copy(&directory.join("clash"), "consumer");
    let board = clashing.join("domains/board.yaml");
    let mut text_of_board = fs::read_to_string(&board).unwrap();
    text_of_board.push_str(
        "  - name: example.consumer.board.Stage\n    kind: enum\n    variants: [Other]\n",
    );
    fs::write(&board, text_of_board).unwrap();
    let mut vendor = Vendor::new(&directory, &owner, &clashing);
    vendor.out = directory.join("clash.yaml");
    vendor.conformances = directory.join("clash-conformances.yaml");
    refused(&vendor, 1, &["example.consumer.board.Stage"]);
}
