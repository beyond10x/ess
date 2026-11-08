//! Adversary pass against story `a-union-variant-scenario-reads-each-row-under-its-own-identity`,
//! its second acceptance bullet: "An observed-identity reference that is ambiguous between two
//! events is resolved to the latest or refused by name; it never silently binds to the first."
//!
//! The unit changed synthesis only, so a suite synthesized before it — the reporter's
//! `union-identity`, whose further variant run reads its row as `{kind: observed}` after the full
//! invocation already published `BoxMade` — is still run by the unchanged runners. This rebuilds
//! that suite from today's synthesis by putting the observed reference back where the unit put a
//! capture, and asks the Rust reference runner for either verdict the acceptance admits: the
//! scenario passes (the reference resolved to the latest `BoxMade`), or it is refused naming the
//! ambiguity. Binding to the first `BoxMade` fails the row claim against a target that holds each
//! row under its own identity, which is neither.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue, ViewExpectation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// The unit's own fixture (`union_variant_row_identity.rs`), the neutral `union-identity`.
const MODEL: &str = "format: ess/22
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
      - {name: kind, type: store.box.Kind}
    lifecycle: {initial: Made, states: [Made], terminal: [Made]}
events:
  - name: store.box.BoxMade
    fields:
      - {name: box_id, type: store.box.BoxId}
      - {name: kind, type: store.box.Kind}
commands:
  - name: store.box.MakeBox
    input:
      - {name: kind, type: store.box.Kind}
    outcomes:
      - name: made
        creates: store.box.Box
        instance: box_id
        emits: [store.box.BoxMade]
        payload:
          store.box.BoxMade:
            box_id: {generated: true}
            kind: input.kind
        sets: {kind: input.kind}
views:
  - name: store.box.Boxes
    source: store.box.Box
    consistency: read_your_writes
    fields:
      - {name: box_id, type: store.box.BoxId}
      - {name: kind, type: store.box.Kind}
";

const MADE: &str = "store.box.MakeBox/outcome/made";

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("model.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn claim_fields(
    step: &mut ScenarioStep,
) -> Option<&mut std::collections::BTreeMap<String, ScenarioValue>> {
    match step {
        ScenarioStep::ExpectView {
            expectation: ViewExpectation::Contains { fields },
            ..
        } => Some(fields),
        _ => None,
    }
}

#[test]
fn adv_u6_an_observed_identity_published_twice_never_binds_to_the_first() {
    let mut suite = ess_conformance::synthesize::synthesize(&ir()).suite;
    let scenario = suite
        .scenarios
        .iter_mut()
        .find(|(id, _)| id.to_string() == MADE)
        .map(|(_, scenario)| scenario)
        .unwrap_or_else(|| panic!("no scenario {MADE}"));

    // The full invocation reads its row as `{kind: observed, event: BoxMade, field: box_id}`;
    // take that value, and put it where each further run now reads its capture.
    let mut sent = 0;
    let mut observed = None;
    let mut rewritten = 0;
    for step in &mut scenario.steps {
        if let ScenarioStep::ExecuteCommand { command, .. } = step {
            if command.to_string() == "store.box.MakeBox" {
                sent += 1;
            }
        }
        let Some(fields) = claim_fields(step) else {
            continue;
        };
        let Some(identity) = fields.get_mut("box_id") else {
            continue;
        };
        match (sent, &*identity) {
            (1, ScenarioValue::Observed { .. }) => observed = Some(identity.clone()),
            (2.., ScenarioValue::Instance { .. }) => {
                *identity = observed
                    .clone()
                    .expect("the full invocation read its row through an observed reference");
                rewritten += 1;
            }
            _ => {}
        }
    }
    assert!(
        sent >= 2 && rewritten >= 1,
        "the further variant run reads its row (sent {sent}, rewritten {rewritten}): {:#?}",
        scenario.steps
    );

    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let result = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir()))
        .into_report()
        .scenarios
        .into_iter()
        .find(|result| result.scenario.to_string() == MADE)
        .expect("the scenario ran");
    let diagnostics: Vec<String> = result
        .diagnostics()
        .map(|diagnostic| format!("{diagnostic:?}"))
        .collect();
    let refused_by_name = diagnostics
        .iter()
        .any(|text| text.to_lowercase().contains("ambiguous"));
    assert!(
        result.status == Status::Passed || refused_by_name,
        "an observed BoxMade.box_id read after the second BoxMade neither resolved to the latest \
         nor was refused as ambiguous; status {:?}:\n{}",
        result.status,
        diagnostics.join("\n")
    );
}
