//! The compiler's resolved-type environment answers the string-operator rule the domain's does
//! (beyond10x/ess#95): conformance decides whether a filter or an invariant is projectable through
//! this adapter, so an adapter that could not say `String` would refuse every such predicate.

use ess_compiler::{expression, ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationCode;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{Predicate, TextOp};

const SOURCE: &str = r#"format: ess/8
system: sample
version: v1
domain: sample.data
types:
  - name: sample.data.Channel
    kind: enum
    variants: [Email, Post]
  - name: sample.data.Sku
    kind: newtype
    of: String
    invariants:
      - value: {starts_with: "SKU-"}
  - name: sample.data.Code
    kind: newtype
    of: sample.data.Sku
entities:
  - name: sample.data.Item
    identity: {name: id, type: Uuid}
    fields:
      - {name: code, type: sample.data.Code}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
commands:
  - name: sample.data.Create
    input:
      - {name: code, type: sample.data.Code}
      - {name: note, type: Optional<String>}
      - {name: channel, type: sample.data.Channel}
      - {name: at, type: Timestamp}
    outcomes:
      - name: created
        when: {code: {ends_with: "0"}}
        creates: sample.data.Item
        instance: id
        sets: {code: input.code}
        emits: [sample.data.Created]
        payload:
          sample.data.Created:
            id: {generated: true}
      - name: refused
        error: sample.data.Rejected
events:
  - name: sample.data.Created
    fields: [{name: id, type: Uuid}]
errors:
  - name: sample.data.Rejected
views:
  - name: sample.data.Items
    source: sample.data.Item
    consistency: eventual
    filter: {code: {contains: "-"}}
    fields:
      - {name: id, type: Uuid}
      - {name: code, type: sample.data.Code}
"#;

fn compiled() -> EssIr {
    let raw = RawSpecFile::parse(SOURCE).expect("parses");
    let spec = Specification::assemble([(Source::new("sample.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("sample.yaml", SOURCE);
    compile_locating(&spec, &sources, &["sample.yaml"]).unwrap_or_else(|errors| panic!("{errors}"))
}

fn starts_with(path: &str) -> Predicate {
    Predicate::TextMatch {
        path: FactPath::new(path).unwrap(),
        op: TextOp::StartsWith,
        value: FactValue::text("A"),
    }
}

#[test]
fn a_newtype_of_string_two_deep_and_an_optional_string_are_strings_to_the_compiler() {
    let ir = compiled();
    let fields = &ir.commands().values().next().unwrap().input;
    for path in ["code", "note"] {
        let checked = expression::check_predicate(&ir, fields, &starts_with(path), "a guard");
        assert!(checked.errors.is_empty(), "{path}: {:?}", checked.errors);
    }
    for path in ["channel", "at"] {
        let checked = expression::check_predicate(&ir, fields, &starts_with(path), "a guard");
        assert_eq!(
            checked.errors.first().map(|error| error.code),
            Some(ValidationCode::TypeMismatch),
            "{path}"
        );
    }
}
