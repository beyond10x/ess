//! A binding constant over a `Boolean`, `Integer` or `Decimal` input (beyond10x/ess#445) reaches
//! every runner typed: the IR keeps `{"kind":"literal","value":"true"}`, and the interpreter and the
//! synthesized suite carry `true`, `3` and `0.5` as JSON scalars, never as text.

use std::path::{Path, PathBuf};

use ess_compiler::ir::{EssIr, ResolvedMappingValue};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// `LegJoined` causes `RecordLeg`, whose four inputs the binding fills with constants.
const CALLS: &str = "format: ess/22
system: demo
version: v1
domain: demo.calls
types:
  - {name: demo.calls.Weight, kind: newtype, of: Integer}
errors:
  - name: demo.calls.Unavailable
    summary: The store could not record the leg this time.
    fields: []
events:
  - name: demo.calls.LegJoined
    fields:
      - {name: leg_id, type: String}
  - name: demo.calls.LegRecorded
    fields:
      - {name: leg_id, type: String}
      - {name: is_bridged, type: Boolean}
      - {name: weight, type: demo.calls.Weight}
      - {name: share, type: Decimal}
commands:
  - name: demo.calls.JoinLeg
    input:
      - {name: leg_id, type: String}
    outcomes:
      - name: joined
        emits: [demo.calls.LegJoined]
        payload:
          demo.calls.LegJoined: {leg_id: input.leg_id}
  - name: demo.calls.RecordLeg
    input:
      - {name: leg_id, type: String}
      - {name: is_bridged, type: Boolean}
      - {name: weight, type: demo.calls.Weight}
      - {name: share, type: Decimal}
      - {name: template, type: String}
    outcomes:
      - name: recorded
        emits: [demo.calls.LegRecorded]
        payload:
          demo.calls.LegRecorded:
            leg_id: input.leg_id
            is_bridged: input.is_bridged
            weight: input.weight
            share: input.share
      - name: unavailable
        external: the store answers a server error
        error: demo.calls.Unavailable
bindings:
  - id: joined
    when: {event: demo.calls.LegJoined}
    invoke: {command: demo.calls.RecordLeg}
    mapping:
      leg_id: event.leg_id
      is_bridged: true
      weight: 3
      share: 0.5
      template: invoice-created
    delivery: at_least_once
    on_failure: retry
";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("calls.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn suite_of(ir: &EssIr) -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(ir).suite
}

fn document(suite: &ConformanceSuite) -> serde_json::Value {
    serde_json::from_str(&suite.to_canonical_json().expect("the suite serialises")).expect("JSON")
}

/// Every `expect_invocation` step's input, across every scenario of the suite.
fn invocations(document: &serde_json::Value) -> Vec<serde_json::Value> {
    document["scenarios"]
        .as_object()
        .expect("scenarios")
        .values()
        .flat_map(|scenario| scenario["steps"].as_array().cloned().unwrap_or_default())
        .filter(|step| step["step"] == "expect_invocation")
        .map(|step| step["input"].clone())
        .collect()
}

/// A suite input value the suite states outright.
fn literal(value: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({"kind": "literal", "value": value})
}

fn statuses(suite: &ConformanceSuite, ir: EssIr) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

#[test]
fn binding_mapping_unquoted_boolean_fills_boolean_input() {
    let ir = ir_of(CALLS);
    let binding = ir.bindings().values().next().expect("one binding");
    let mapped = binding
        .mapping
        .iter()
        .find(|mapping| mapping.target == "is_bridged")
        .expect("is_bridged is mapped");
    assert_eq!(
        serde_json::to_value(&mapped.value).unwrap(),
        serde_json::json!({"kind": "literal", "value": "true"})
    );
    assert_eq!(mapped.target_type.to_string(), "Boolean");
    // The quoted form compiles to the same bytes.
    let quoted = ir_of(&CALLS.replace("is_bridged: true\n", "is_bridged: 'true'\n"));
    assert_eq!(
        serde_json::to_string(&ir).unwrap(),
        serde_json::to_string(&quoted).unwrap(),
        "`true` and `'true'` are one value"
    );
}

#[test]
fn binding_literal_invocation_carries_typed_value() {
    let ir = ir_of(CALLS);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("joined"))
        .collect();
    assert_eq!(
        refused,
        Vec::<String>::new(),
        "nothing about the binding is refused"
    );
    let document = document(&synthesis.suite);
    let inputs = invocations(&document);
    assert_ne!(
        inputs.len(),
        0,
        "the suite expects the invocation: {document:#}"
    );
    for input in &inputs {
        assert_eq!(
            input["is_bridged"],
            literal(&serde_json::json!(true)),
            "{input:#}"
        );
        // Numbers are written as the binary64 they carry, so an integral one reads `3.0`; what
        // matters is that it is a JSON number, not the text `"3"`.
        for (field, expected) in [("weight", 3.0), ("share", 0.5)] {
            assert_eq!(input[field]["kind"], "literal", "{input:#}");
            assert!(
                input[field]["value"].is_number(),
                "{field} is a number: {input:#}"
            );
            assert_eq!(input[field]["value"].as_f64(), Some(expected), "{input:#}");
        }
        assert_eq!(
            input["template"],
            literal(&serde_json::json!("invoice-created")),
            "{input:#}"
        );
    }

    // The interpreter sends the typed value and passes every scenario.
    let passed = statuses(&synthesis.suite, ir_of(CALLS));
    assert_ne!(passed.len(), 0, "not empty");
    for (id, status) in &passed {
        assert_eq!(*status, Status::Passed, "{id}: {passed:#?}");
    }

    // A suite expecting the text "true" is one the typed runner fails.
    let mut textual = document.clone();
    let mut rewritten = 0;
    for scenario in textual["scenarios"]
        .as_object_mut()
        .expect("scenarios")
        .values_mut()
    {
        for step in scenario["steps"].as_array_mut().into_iter().flatten() {
            if step["step"] == "expect_invocation" {
                step["input"]["is_bridged"]["value"] = serde_json::json!("true");
                rewritten += 1;
            }
        }
    }
    assert!(rewritten > 0, "the suite was rewritten");
    let textual = ConformanceSuite::from_json(&textual.to_string()).expect("still a suite");
    let failed = statuses(&textual, ir_of(CALLS));
    assert!(
        failed.iter().any(|(_, status)| *status == Status::Failed),
        "a runner sending \"true\" does not match the suite: {failed:#?}"
    );
}

fn billing() -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("billing exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("validates");
    compile(&specification, &sources).expect("resolves")
}

/// The text literal of the normative example keeps its IR shape and its suite value. The whole
/// IR, suite and generated bytes of the example are pinned by their own golden tests
/// (`ess-gen`'s `tests/corpus/billing`, the `ess-diff` billing suites); this holds the one
/// mapping the change touches.
#[test]
fn binding_literal_existing_text_and_enum_bytes_unchanged() {
    let ir = billing();
    let binding = ir.bindings().values().next().expect("the billing binding");
    let template = binding
        .mapping
        .iter()
        .find(|mapping| mapping.target == "template")
        .expect("template is mapped");
    assert!(
        matches!(&template.value, ResolvedMappingValue::Literal { value } if value == "invoice-created")
    );
    assert_eq!(ir.literal_primitive(&template.target_type), None);
    assert_eq!(
        serde_json::to_string(template).unwrap(),
        r#"{"target":"template","target_type":{"kind":"declared","name":"billing.email.TemplateId"},"value":{"kind":"literal","value":"invoice-created"}}"#
    );
    let document = document(&suite_of(&ir));
    let inputs = invocations(&document);
    assert_ne!(inputs.len(), 0, "{document:#}");
    for input in &inputs {
        assert_eq!(
            input["template"],
            literal(&serde_json::json!("invoice-created")),
            "{input:#}"
        );
    }
}

/// The same constants on an event an external channel delivers with a context (ess/18): the
/// suite chooses each delivered occurrence, and the invocation it expects is typed the same way.
#[test]
fn binding_literal_delivered_occurrence_carries_typed_value() {
    let text = CALLS
        .replace(
            "  - name: demo.calls.JoinLeg\n    input:\n      - {name: leg_id, type: String}\n    outcomes:\n      - name: joined\n        emits: [demo.calls.LegJoined]\n        payload:\n          demo.calls.LegJoined: {leg_id: input.leg_id}\n",
            "",
        )
        .replace(
            "    when: {event: demo.calls.LegJoined}\n",
            "    when:\n      event: demo.calls.LegJoined\n      context_authority: call-legs\n      context_fields:\n        - {name: account_id, type: String}\n",
        );
    assert_ne!(text, CALLS);
    let ir = ir_of(&text);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let document = document(&synthesis.suite);
    let inputs = invocations(&document);
    assert_ne!(
        inputs.len(),
        0,
        "the suite expects the invocation: {document:#}"
    );
    for input in &inputs {
        assert_eq!(
            input["is_bridged"],
            literal(&serde_json::json!(true)),
            "{input:#}"
        );
        assert_eq!(input["share"]["value"].as_f64(), Some(0.5), "{input:#}");
        assert!(input["weight"]["value"].is_number(), "{input:#}");
    }
    let passed = statuses(&synthesis.suite, ir_of(&text));
    for (id, status) in &passed {
        assert_eq!(*status, Status::Passed, "{id}: {passed:#?}");
    }
}
