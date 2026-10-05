//! Unit variants in the conformance lanes (ess/22, beyond10x/ess#418,
//! `docs/design/union-unit-variants.md`): synthesis witnesses them with no refusal, the reference
//! runner holds a response to the unit variant's closed shape, the interpreter executes them, and
//! an authored value of the wrong shape is refused before it runs.

use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/union-unit-variants.yaml");

/// A response union whose first variant — the one synthesis witnesses — is a unit variant.
const RESPONSE: &str = "format: ess/22
system: example
version: v1
domain: example.api
types:
  - name: example.api.Result
    kind: union
    tag: kind
    variants:
      absent:
      text: String
events:
  - name: example.api.Returned
    fields:
      - {name: item, type: example.api.Result}
commands:
  - name: example.api.Fetch
    response:
      - {name: item, type: example.api.Result}
    outcomes:
      - name: returned
        emits: [example.api.Returned]
        payload:
          example.api.Returned:
            item: {response: item}
";

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("model.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn body(value: &serde_json::Value) -> BTreeMap<String, Node> {
    serde_json::from_value(serde_json::json!({ "item": value })).unwrap()
}

#[test]
fn the_model_synthesizes_with_no_refusal() {
    for text in [MODEL, RESPONSE] {
        let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
        assert_eq!(synthesis.refusals.len(), 0, "{:?}", synthesis.refusals);
        assert_ne!(synthesis.suite.scenarios.len(), 0);
    }
}

#[test]
fn a_response_holds_a_unit_variant_to_the_tag_alone() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(RESPONSE));
    let observation = synthesis
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .find_map(|step| match step {
            ScenarioStep::ExpectResponsePayload { response } => Some(response),
            _ => None,
        })
        .expect("a response observation");
    for valid in [
        serde_json::json!({"kind": "absent"}),
        serde_json::json!({"kind": "text", "value": "ok"}),
    ] {
        let body = body(&valid);
        observation
            .compare(Some(&body), &body)
            .unwrap_or_else(|error| panic!("{valid}: {error:?}"));
    }
    for invalid in [
        serde_json::json!({"kind": "absent", "value": "ok"}),
        serde_json::json!({"kind": "absent", "value": null}),
        serde_json::json!({"kind": "text"}),
        serde_json::json!({"kind": "absent", "unexpected": true}),
    ] {
        let body = body(&invalid);
        assert!(
            observation.compare(Some(&body), &body).is_err(),
            "{invalid} must be refused"
        );
    }
}

#[test]
fn the_interpreter_passes_a_suite_whose_witness_is_a_unit_variant() {
    for text in [MODEL, RESPONSE] {
        let model = ir(text);
        let synthesis = ess_conformance::synthesize::synthesize(&model);
        let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
        let run = ess_conformance::Runner::for_suite(admitted.suite()).run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        );
        assert_ne!(run.scenarios.len(), 0);
        assert!(
            run.scenarios
                .iter()
                .all(|result| result.status == ess_conformance::report::Status::Passed),
            "{:#?}",
            run.scenarios
        );
    }
}

/// An authored act sending `status`, as written.
fn authored(status: &str) -> ess_conformance::authored::Authoring {
    let text = format!(
        "type: ess-scenario/1\ndomain: demo.work\nscenario: a-status\nsummary: One status \
         sent.\ntimeline:\n  - at: 2026-01-05T09:00:00Z\n    command: demo.work.ReportStatus\n    \
         actor: demo.work.Reporter\n    input:\n      status: {status}\n    outcome: reported\n"
    );
    ess_conformance::authored::compile(
        &ir(MODEL),
        &[ess_conformance::authored::Source::new("status.yaml", text)],
    )
}

#[test]
fn an_authored_unit_variant_compiles_to_the_tag_alone() {
    let authoring = authored("{kind: Open}");
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    let input = authoring
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(input),
            _ => None,
        })
        .expect("an invocation");
    let Some(ScenarioValue::Literal {
        value: Node::Map(entries),
    }) = input.get("status")
    else {
        panic!("a mapping: {input:?}");
    };
    assert_eq!(entries.keys().collect::<Vec<_>>(), ["kind"], "{entries:?}");
}

#[test]
fn an_authored_unit_variant_with_a_payload_member_is_refused_by_name() {
    // A payload variant keeps the shape-only reading authored union values always had; a unit
    // variant declares nothing beside its tag, so a content member is an undeclared field.
    for status in [
        "{kind: Open, value: {outcome: shipped}}",
        "{kind: Open, value: null}",
    ] {
        let authoring = authored(status);
        assert!(
            !authoring.is_complete(),
            "{status} must be refused before it runs"
        );
        let refused = authoring
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(authoring.refusals.len(), 1, "{status}: {refused}");
        assert!(
            refused.contains("value"),
            "the refusal names the field: {status}: {refused}"
        );
        assert!(
            !refused.contains("granted"),
            "refused for its shape: {refused}"
        );
    }
}
