//! Adversary pass against story `a-union-variant-scenario-reads-each-row-under-its-own-identity`,
//! the sibling further runs: the unit lists "an absent reference, a fallback, a union variant" as
//! the further runs on a row of their own, and its commit message says synthesis never writes an
//! observed reference after an earlier run published that event. The expression-point runs
//! (ess/27, #560) are further runs on a row of their own too.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{ConformanceSuite, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const EXPRESSION: &str = include_str!("fixtures/expression-arithmetic-and-time.yaml");

fn ir_of(model: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("model.yaml"),
        RawSpecFile::parse(model).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn observed_events(node: &serde_json::Value, found: &mut Vec<String>) {
    match node {
        serde_json::Value::Object(map) => {
            if map.get("kind").and_then(serde_json::Value::as_str) == Some("observed") {
                if let Some(event) = map.get("event").and_then(serde_json::Value::as_str) {
                    found.push(event.to_owned());
                }
            }
            for value in map.values() {
                observed_events(value, found);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                observed_events(item, found);
            }
        }
        _ => {}
    }
}

/// Every observed reference written after its event was published twice in the same scenario.
fn ambiguous_observed(ir: &EssIr, suite: &ConformanceSuite) -> Vec<(String, usize, String)> {
    let mut emits: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, command) in ir.commands() {
        for outcome in &command.outcomes {
            emits.insert(
                format!("{name}/{}", outcome.name),
                outcome
                    .emits
                    .iter()
                    .map(|event| event.name().to_string())
                    .collect(),
            );
        }
    }
    let mut found = Vec::new();
    let mut most = 0;
    for (id, scenario) in &suite.scenarios {
        let mut published: BTreeMap<String, usize> = BTreeMap::new();
        for (at, step) in scenario.steps.iter().enumerate() {
            let mut read = Vec::new();
            observed_events(&serde_json::to_value(step).unwrap(), &mut read);
            for event in read {
                if published.get(&event).copied().unwrap_or(0) >= 2 {
                    found.push((id.to_string(), at, event));
                }
            }
            if let ScenarioStep::ExpectOutcome { outcome } = step {
                for event in emits.get(&outcome.to_string()).into_iter().flatten() {
                    let count = published.entry(event.clone()).or_default();
                    *count += 1;
                    most = most.max(*count);
                }
            }
        }
    }
    assert!(
        most >= 2,
        "no scenario published an event twice: the probe would be vacuous"
    );
    found
}

/// An expression-point run of `RecordPastIncident` creates a row of its own and captures it, as a
/// union-variant run now does; its view claim must name that capture, not `{kind: observed}`.
#[test]
fn adv_u6_an_expression_point_run_reads_its_row_under_its_own_identity() {
    let ir = ir_of(EXPRESSION);
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let ambiguous = ambiguous_observed(&ir, &suite);
    assert_eq!(ambiguous, [], "{ambiguous:#?}");
}

/// A ranked view over the rows each union-variant run made: the order costs companions, created
/// through `MakeBox` too, beside every run's own row.
const RANKED: &str = "format: ess/22
system: store
version: v1
domain: store.box
types:
  - {name: store.box.BoxId, kind: newtype, of: Uuid}
  - name: store.box.Kind
    kind: union
    tag: kind
    variants:
      Plain:
      Fancy:
entities:
  - name: store.box.Box
    identity: {name: box_id, type: store.box.BoxId}
    fields:
      - {name: label, type: String}
      - {name: kind, type: store.box.Kind}
    lifecycle: {initial: Made, states: [Made], terminal: [Made]}
events:
  - name: store.box.BoxMade
    fields:
      - {name: box_id, type: store.box.BoxId}
commands:
  - name: store.box.MakeBox
    input:
      - {name: label, type: String}
      - {name: kind, type: store.box.Kind}
    outcomes:
      - name: made
        creates: store.box.Box
        instance: box_id
        emits: [store.box.BoxMade]
        payload:
          store.box.BoxMade: {box_id: {generated: true}}
        sets: {label: input.label, kind: input.kind}
views:
  - name: store.box.Boxes
    source: store.box.Box
    consistency: read_your_writes
    order_by: [label asc]
    fields:
      - {name: box_id, type: store.box.BoxId}
      - {name: label, type: String}
      - {name: kind, type: store.box.Kind}
";

#[test]
fn adv_u6_a_ranked_view_over_union_variant_rows() {
    use ess_conformance::{interpret::Interpreted, report::Status, AdmittedSuite, Runner};
    let ir = ir_of(RANKED);
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let failing: Vec<_> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir_of(RANKED)))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let diagnostics: Vec<_> = result
                .diagnostics()
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect();
            (result.scenario.to_string(), diagnostics)
        })
        .collect();
    let ambiguous = ambiguous_observed(&ir, &suite);
    assert!(
        failing.is_empty() && ambiguous.is_empty(),
        "{failing:#?}\n{ambiguous:#?}"
    );
}
