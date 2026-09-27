//! `Json`, any JSON value, as a primitive (beyond10x/ess#138, ess/15).
//!
//! Admitted wherever a type is written, never as a map key, and never read by a predicate: a
//! predicate compares scalars, and a JSON value is one only by accident.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::types::{Primitive, TypeBody, TypeRef};

fn single(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("msgs.yaml"), raw)]).map_err(|errors| errors.to_string())
}

const HEADER: &str = "format: ess/15\nsystem: demo\nversion: v1\ndomain: demo.msgs\n";

fn with_types(types: &str) -> String {
    format!("{HEADER}types:\n{types}")
}

#[test]
fn issue_138_a_newtype_of_json_validates() {
    let spec = single(&with_types(
        "  - {name: demo.msgs.Body, kind: newtype, of: Json}\n",
    ))
    .expect("#138's repro validates");
    let declared = spec
        .system()
        .types
        .get(&"demo.msgs.Body".parse().unwrap())
        .expect("declared");
    let TypeBody::Newtype { of, .. } = &declared.body else {
        panic!("a newtype");
    };
    assert_eq!(of, &TypeRef::Primitive(Primitive::Json));
    assert_eq!(Primitive::parse("Json"), Some(Primitive::Json));
    assert_eq!(Primitive::Json.as_str(), "Json");
    assert!(Primitive::ALL.contains(&Primitive::Json));
}

#[test]
fn json_is_admitted_in_every_type_position_but_a_map_key() {
    let text = format!(
        "{HEADER}types:\n  - name: demo.msgs.Envelope\n    kind: struct\n    fields:\n      - {{name: \
         body, type: Json}}\n      - {{name: maybe, type: Optional<Json>}}\n      - {{name: many, type: \
         List<Json>}}\n      - {{name: named, type: \"Map<String, Json>\"}}\n"
    );
    single(&text).expect("Json as a value is admitted everywhere");
    let error = single(&format!(
        "{HEADER}types:\n  - name: demo.msgs.Envelope\n    kind: struct\n    fields:\n      - {{name: \
         keyed, type: \"Map<Json, String>\"}}\n"
    ))
    .expect_err("a JSON value has no stable key spelling");
    assert!(error.contains("Json"), "{error}");
}

#[test]
fn json_below_ess_15_is_refused_with_the_format_it_needs() {
    let error = single(
        &with_types("  - {name: demo.msgs.Body, kind: newtype, of: Json}\n")
            .replace("format: ess/15", "format: ess/14"),
    )
    .expect_err("an older reader does not know the primitive");
    assert!(error.contains("ess/15"), "{error}");
}

#[test]
fn a_predicate_that_reads_a_json_value_is_refused() {
    let guard = format!(
        "{HEADER}types:\n  - {{name: demo.msgs.Body, kind: newtype, of: Json}}\nerrors:\n  - name: \
         demo.msgs.Refused\n    fields: []\nactors:\n  - {{name: demo.msgs.Sender, may: [demo.msgs.Send]}}\n\
         commands:\n  - name: demo.msgs.Send\n    input:\n      - {{name: body, type: demo.msgs.Body}}\n    \
         outcomes:\n      - name: refused\n        when: body == \"x\"\n        error: demo.msgs.Refused\n"
    );
    let error = single(&guard).expect_err("a guard cannot compare a JSON value");
    assert!(error.contains("body"), "{error}");
    let invariant = with_types(
        "  - name: demo.msgs.Body\n    kind: newtype\n    of: Json\n    invariants: [value == \"x\"]\n",
    );
    let error = single(&invariant).expect_err("an invariant cannot compare a JSON value");
    assert!(error.contains("Json") || error.contains("value"), "{error}");
}

#[test]
fn a_json_field_carries_a_structured_payload_literal_only_from_an_input() {
    let text = format!(
        "{HEADER}types:\n  - {{name: demo.msgs.Body, kind: newtype, of: Json}}\nevents:\n  - name: \
         demo.msgs.Sent\n    fields:\n      - {{name: body, type: demo.msgs.Body}}\nactors:\n  - {{name: \
         demo.msgs.Sender, may: [demo.msgs.Send]}}\ncommands:\n  - name: demo.msgs.Send\n    input:\n      \
         - {{name: body, type: demo.msgs.Body}}\n    outcomes:\n      - name: sent\n        emits: \
         [demo.msgs.Sent]\n        payload:\n          demo.msgs.Sent: {{body: input.body}}\n"
    );
    single(&text).expect("a JSON input copied into a JSON event field validates");
}
