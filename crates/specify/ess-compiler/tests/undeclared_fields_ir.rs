//! `undeclared_fields: ignored` through the compiler (`ess/24`, beyond10x/ess#500): the IR carries
//! it on the command and lists the open structs beside `types`, and a specification that does not
//! write the key — or writes the default — keeps its bytes and its compiled digest.

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::types::UndeclaredFields;

/// A command whose response reaches a struct, and a second struct the response does not reach.
/// `{format}` is the header's major; `{command}` and `{record}` are the keys written on
/// `PlaceOrder` and on `Extension`.
const MODEL: &str = "\
format: ess/{format}
system: catalog
version: v1
domain: catalog.orders
types:
  - name: catalog.orders.Extension
    kind: struct
{record}    fields:
      - name: vendor
        type: String
  - name: catalog.orders.Closed
    kind: struct
    fields:
      - name: note
        type: String
actors:
  - name: catalog.orders.Buyer
    may:
      - catalog.orders.PlaceOrder
commands:
  - name: catalog.orders.PlaceOrder
{command}    input:
      - name: item
        type: String
      - name: note
        type: catalog.orders.Closed
    response:
      - name: order_ref
        type: String
      - name: extension
        type: catalog.orders.Extension
    outcomes:
      - name: placed
        returns: true
";

fn text(format: u32, command: Option<&str>, record: Option<&str>) -> String {
    let key = |value: Option<&str>| {
        value.map_or_else(String::new, |value| {
            format!("    undeclared_fields: {value}\n")
        })
    };
    MODEL
        .replace("{format}", &format.to_string())
        .replace("{command}", &key(command))
        .replace("{record}", &key(record))
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn name(value: &str) -> QualifiedName {
    value.parse().expect("a qualified name")
}

fn json(ir: &EssIr) -> serde_json::Value {
    serde_json::from_str(&ir.to_canonical_json()).expect("the IR is JSON")
}

#[test]
fn the_ir_carries_ignored_on_the_command_and_beside_the_struct() {
    let ir = ir(&text(24, Some("ignored"), Some("ignored")));
    let command = &ir.commands()[&name("catalog.orders.PlaceOrder")];
    assert_eq!(command.undeclared_fields, UndeclaredFields::Ignored);
    assert_eq!(
        ir.undeclared_fields(&name("catalog.orders.Extension")),
        UndeclaredFields::Ignored
    );
    assert_eq!(
        ir.undeclared_fields(&name("catalog.orders.Closed")),
        UndeclaredFields::Refused,
        "a struct that does not say so stays closed"
    );

    let document = json(&ir);
    assert_eq!(
        document["commands"]["catalog.orders.PlaceOrder"]["undeclared_fields"],
        "ignored"
    );
    assert_eq!(
        document["undeclared_fields_ignored"],
        serde_json::json!(["catalog.orders.Extension"])
    );
    assert!(
        document["types"]["catalog.orders.Extension"]["body"]
            .get("undeclared_fields")
            .is_none(),
        "the resolved struct body keeps its shape: {}",
        document["types"]["catalog.orders.Extension"]
    );
}

#[test]
fn the_command_and_the_struct_are_open_independently() {
    let only_command = ir(&text(24, Some("ignored"), None));
    assert_eq!(
        only_command.commands()[&name("catalog.orders.PlaceOrder")].undeclared_fields,
        UndeclaredFields::Ignored
    );
    assert!(only_command.undeclared_fields_ignored().is_empty());

    let only_record = ir(&text(24, None, Some("ignored")));
    assert_eq!(
        only_record.commands()[&name("catalog.orders.PlaceOrder")].undeclared_fields,
        UndeclaredFields::Refused,
        "an open struct in the response does not open the response root"
    );
    assert_eq!(
        only_record
            .undeclared_fields_ignored()
            .iter()
            .collect::<Vec<_>>(),
        [&name("catalog.orders.Extension")]
    );
}

#[test]
fn a_closed_specification_keeps_its_bytes_under_any_header_and_with_the_default_written() {
    let legacy = ir(&text(23, None, None));
    let current = ir(&text(24, None, None));
    let defaulted = ir(&text(24, Some("refused"), Some("refused")));
    assert_eq!(legacy.to_canonical_json(), current.to_canonical_json());
    assert_eq!(legacy.to_canonical_json(), defaulted.to_canonical_json());
    assert_eq!(legacy.source_digest(), defaulted.source_digest());
    let document = json(&defaulted);
    assert!(document.get("undeclared_fields_ignored").is_none());
    assert!(document["commands"]["catalog.orders.PlaceOrder"]
        .get("undeclared_fields")
        .is_none());
}

#[test]
fn an_open_record_changes_the_digest() {
    assert_ne!(
        ir(&text(24, None, None)).source_digest(),
        ir(&text(24, Some("ignored"), None)).source_digest()
    );
    assert_ne!(
        ir(&text(24, None, None)).source_digest(),
        ir(&text(24, None, Some("ignored"))).source_digest()
    );
}

/// The billing example's directory.
fn billing() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("the billing example exists")
}

/// Every `.yaml` file under `directory`, relative to it, sorted.
fn yaml_files(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![directory.to_path_buf()];
    while let Some(current) = pending.pop() {
        for entry in std::fs::read_dir(&current).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// The digest the committed billing suite was synthesized from, before `ess/24` existed.
fn committed_billing_digest() -> String {
    let suite =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../suites/generated/billing/suite.json");
    let suite: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(suite).expect("the committed suite"))
            .expect("the suite is JSON");
    suite["provenance"]["spec_digest"]
        .as_str()
        .expect("the suite names the digest it was synthesized from")
        .to_owned()
}

#[test]
fn the_billing_example_keeps_its_committed_compiled_digest() {
    let base = billing();
    let files = yaml_files(&base);
    let texts: Vec<String> = files
        .iter()
        .map(|path| std::fs::read_to_string(path).expect("readable"))
        .collect();
    let borrowed: Vec<&str> = texts.iter().map(String::as_str).collect();
    let parsed = RawSpecFile::parse_all(&borrowed);
    let sources = files.iter().zip(parsed).map(|(path, raw)| {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        (
            Source::new(label),
            raw.unwrap_or_else(|error| panic!("{error}")),
        )
    });
    let spec = Specification::assemble(sources).unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(ir.source_digest(), committed_billing_digest());
}
