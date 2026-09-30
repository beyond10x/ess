//! Adversary pass 1 on `story:generated-server-publishes-and-reads-headers`: the `published`
//! property the `OpenAPI` projection now requires, read as the specification it claims to be.
//!
//! The schema's own description says the list is "every event this branch published, in
//! publication order". A branch that emits two different events therefore has exactly one right
//! answer per publication, and the contract should refuse the others — the same events in the
//! other order, or one event twice and the other not at all.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};

/// One component; `ops.core.Open`'s `opened` branch emits `Opened` then `Stamped`.
const MODEL: &str = "format: ess/15
system: ops
version: v1
domain: ops.core
events:
  - name: ops.core.Opened
    fields:
      - {name: id, type: String}
  - name: ops.core.Stamped
    fields:
      - {name: id, type: String}
commands:
  - name: ops.core.Open
    input:
      - {name: id, type: String}
    outcomes:
      - name: opened
        emits: [ops.core.Opened, ops.core.Stamped]
        payload:
          ops.core.Opened: {id: input.id}
          ops.core.Stamped: {id: input.id}
components:
  - component: ops-service
    owns: {domains: [ops.core]}
    accepts: {commands: [ops.core.Open]}
    publishes: {events: [ops.core.Opened, ops.core.Stamped]}
    reached_by: network
";

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The served contract, with its root pointed at `ops.core.Open.opened.Response`.
fn opened_validator() -> jsonschema::Validator {
    let ir = ir();
    let component = ir.components().values().next().expect("one component");
    let mut document: Value =
        serde_json::from_str(&ess_gen::openapi::json(&ir, component)).expect("JSON");
    document["$ref"] = json!("#/components/schemas/ops.core.Open.opened.Response");
    jsonschema::draft202012::new(&document).expect("the contract compiles as a schema")
}

fn entry(event: &str) -> Value {
    json!({"event": event, "payload": {"id": "t"}})
}

#[test]
fn the_right_answer_for_a_two_event_branch_validates() {
    let validator = opened_validator();
    let right = json!({
        "outcome": "opened",
        "published": [entry("ops.core.Opened"), entry("ops.core.Stamped")],
    });
    assert!(validator.is_valid(&right), "the contract admits {right}");
}

#[test]
fn the_contract_refuses_a_two_event_branch_answered_out_of_publication_order() {
    let validator = opened_validator();
    let reversed = json!({
        "outcome": "opened",
        "published": [entry("ops.core.Stamped"), entry("ops.core.Opened")],
    });
    assert!(
        !validator.is_valid(&reversed),
        "the contract says `published` is in publication order, and admits {reversed}"
    );
}

#[test]
fn the_contract_refuses_a_two_event_branch_answered_with_one_event_twice() {
    let validator = opened_validator();
    let doubled = json!({
        "outcome": "opened",
        "published": [entry("ops.core.Opened"), entry("ops.core.Opened")],
    });
    assert!(
        !validator.is_valid(&doubled),
        "the branch publishes `Opened` and `Stamped` once each, and the contract admits {doubled}"
    );
}
