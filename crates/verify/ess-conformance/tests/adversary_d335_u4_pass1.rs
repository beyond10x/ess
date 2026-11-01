//! Adversary pass 1 on `story:a-text-inequality-against-an-example-is-refuted` (wave D335, U4).
//!
//! The Acceptance: a command whose input carries an `example:` and whose guard is
//! `<path> != <the example's value>` synthesizes both branches, one sending the example and one
//! sending a value the guard rejects; the same for a scalar input with an `example:`.
//!
//! The unit's own test covers `String`, `Uuid` and `Timestamp` examples that differ from the leaf's
//! own witness. These cases take the scalar kinds it does not name, and examples whose value is the
//! value synthesis would have chosen for the leaf without an example (its own witness), where the
//! fix's `own != at_base` condition adds nothing.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const ACCEPTED: &str = "demo.docs.Submit/outcome/accepted";
const REFUSED: &str = "demo.docs.Submit/outcome/unsupported";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn yaml(text: &str) -> Node {
    ess_yaml::from_str(text).unwrap_or_else(|error| panic!("{error}: {text}"))
}

/// The suite for `text`, or why it is not one with no refusal that passes on the interpreted
/// target.
fn synthesized(text: &str) -> Result<Synthesis, String> {
    let result = synthesize(&ir(text));
    // Only the command's own refusals: a constrained type no view publishes is refused on its own
    // (ESS-SYNTH-013), which says nothing about the guard.
    let refusals: Vec<String> = result
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("demo.docs.Submit"))
        .collect();
    if !refusals.is_empty() {
        return Err(format!("refused: {refusals:#?}"));
    }
    let admitted = AdmittedSuite::from_suite(&result.suite).map_err(|error| error.to_string())?;
    let run = Runner::for_suite(&result.suite).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(ir(text)),
    );
    for scenario in &run.scenarios {
        if scenario.status != Status::Passed {
            return Err(format!("{} did not pass", scenario.scenario));
        }
    }
    Ok(result)
}

/// Every literal value `scenario`'s `demo.docs.Submit` steps sent at `field`; `None` where the
/// suite has no such scenario.
fn sent(result: &Synthesis, scenario: &str, field: &str) -> Option<Vec<Node>> {
    let (_, scenario) = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)?;
    let mut values = Vec::new();
    for step in &scenario.steps {
        if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
            if command.to_string() != "demo.docs.Submit" {
                continue;
            }
            if let Some(ScenarioValue::Literal { value }) = input.get(field) {
                values.push(value.clone());
            }
        }
    }
    Some(values)
}

/// One command `demo.docs.Submit` taking `name: String` and `inputs`, refused as `unsupported`
/// where `when` holds (the unit's own model, `example_inequality.rs`).
fn model(types: &str, inputs: &str, when: &str) -> String {
    format!(
        "format: ess/33
system: demo
version: v1
domain: demo.docs
types:
{types}entities:
  - name: demo.docs.Doc
    identity: {{name: doc_id, type: Uuid}}
    fields:
      - {{name: name, type: String}}
    lifecycle: {{initial: Draft, states: [Draft], terminal: [Draft]}}
errors:
  - name: demo.docs.Unsupported
    fields:
      - {{name: name, type: String}}
events:
  - name: demo.docs.Submitted
    fields:
      - {{name: doc_id, type: Uuid}}
commands:
  - name: demo.docs.Submit
    input:
      - {{name: name, type: String}}
{inputs}    outcomes:
      - name: unsupported
        when: '{when}'
        error: demo.docs.Unsupported
        payload:
          demo.docs.Unsupported: {{name: input.name}}
      - name: accepted
        creates: demo.docs.Doc
        instance: doc_id
        sets: {{name: input.name}}
        emits: [demo.docs.Submitted]
        payload:
          demo.docs.Submitted: {{doc_id: {{generated: true}}}}
"
    )
}

/// Why `field: kind` with `example` under `tag != <example>` does not synthesize both branches
/// the Acceptance names; `None` where it does.
fn both_branches(
    types: &str,
    field: &str,
    kind: &str,
    example: &str,
    guard: &str,
) -> Option<String> {
    let text = model(
        types,
        &format!("      - {{name: {field}, type: {kind}, example: {example}}}\n"),
        guard,
    );
    let result = match synthesized(&text) {
        Ok(result) => result,
        Err(why) => return Some(format!("{kind} example {example}: {why}")),
    };
    let expected = yaml(example);
    match sent(&result, ACCEPTED, field) {
        Some(values) if !values.is_empty() && values.iter().all(|value| *value == expected) => {}
        other => {
            return Some(format!(
                "{kind} example {example}: the accepted step does not send the example: {other:?}"
            ))
        }
    }
    match sent(&result, REFUSED, field) {
        Some(values) if !values.is_empty() && values.iter().all(|value| *value != expected) => None,
        other => Some(format!(
            "{kind} example {example}: no refusing step sends another value: {other:?}"
        )),
    }
}

/// Acceptance line 2, for the scalar kinds the unit's test does not take: a number, a boolean, an
/// enum, a `Duration` and a `Bytes` input with an `example:` and a `!=` against it. Pins the
/// unit's claim that these ladders already hold a value beside their literal.
#[test]
fn every_scalar_kind_with_an_example_gets_both_branches_of_an_inequality_against_it() {
    let kind = "  - {name: demo.docs.Kind, kind: enum, variants: [Draft, Final]}\n";
    let cases = [
        ("  []\n", "Integer", "5", "tag != 5"),
        ("  []\n", "Decimal", "2.5", "tag != 2.5"),
        ("  []\n", "Boolean", "false", "tag != false"),
        (kind, "demo.docs.Kind", "Final", "tag != Final"),
        ("  []\n", "Duration", "\"PT30S\"", "tag != \"PT30S\""),
        ("  []\n", "Bytes", "\"AQ==\"", "tag != \"AQ==\""),
    ];
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|(types, kind, example, guard)| {
            both_branches(types, "tag", kind, example, guard)
        })
        .collect();
    assert!(
        failures.is_empty(),
        "story:a-text-inequality-against-an-example-is-refuted: {failures:#?}"
    );
}

/// Acceptance lines 1 and 2 where the example is the value synthesis gives the leaf without one:
/// a `Duration` whose example is `PT1S`, a `Timestamp` whose example is `2020-01-01T00:00:00Z`
/// and a `Bytes` whose example is `AA==`. (A `String` cannot be written so: its own witness is its
/// path, and a guard comparing `tag` with the text `"tag"` is refused as `type_mismatch`.)
/// The leaf's own witness equals the example there, so the ladder gains nothing from the fix.
#[test]
fn an_example_equal_to_the_leafs_own_witness_still_gets_both_branches() {
    let cases = [
        ("Duration", "\"PT1S\"", "tag != \"PT1S\""),
        (
            "Timestamp",
            "\"2020-01-01T00:00:00Z\"",
            "tag != \"2020-01-01T00:00:00Z\"",
        ),
        ("Bytes", "\"AA==\"", "tag != \"AA==\""),
    ];
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|(kind, example, guard)| both_branches("  []\n", "tag", kind, example, guard))
        .collect();
    assert!(
        failures.is_empty(),
        "story:a-text-inequality-against-an-example-is-refuted: an example that is the leaf's own \
         witness: {failures:#?}"
    );
}

/// Acceptance line 1 for a text type whose invariant the leaf's own witness breaks: `code` is a
/// six-byte newtype, its example `ABC123`. The witness without the example is `code`, four bytes,
/// which the type refuses, so it is never sent.
#[test]
fn an_example_of_a_constrained_text_gets_both_branches() {
    let types = "  - name: demo.docs.Code
    kind: newtype
    of: String
    invariants:
      - \"value.utf8_bytes == 6\"
";
    let failure = both_branches(
        types,
        "code",
        "demo.docs.Code",
        "\"ABC123\"",
        "code != \"ABC123\"",
    );
    assert_eq!(
        failure, None,
        "story:a-text-inequality-against-an-example-is-refuted: a six-byte code"
    );
}

/// Acceptance line 2 for an `Optional<String>` input with an example.
#[test]
fn an_optional_text_with_an_example_gets_both_branches() {
    let failure = both_branches(
        "  []\n",
        "tag",
        "Optional<String>",
        "\"v1\"",
        "tag != \"v1\"",
    );
    assert_eq!(
        failure, None,
        "story:a-text-inequality-against-an-example-is-refuted: an optional text"
    );
}
