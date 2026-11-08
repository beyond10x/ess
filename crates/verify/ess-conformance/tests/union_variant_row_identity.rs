//! A scenario that runs a creating branch once more per union variant reads each row it made under
//! the identity of the event that created that row (story
//! `a-union-variant-scenario-reads-each-row-under-its-own-identity`, reporter repro
//! `union-identity`): synthesis never writes an observed reference, which the runners resolve to
//! the event's first occurrence, after an earlier run of the branch already published that event.

mod support_go;
mod support_ts;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{
    AdmittedSuite, ConformanceSuite, InstanceName, Runner, ScenarioStep, ScenarioValue,
    ViewExpectation,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// A box made with a kind, a union of unit variants; the system mints its identity, and a view
/// lists every box with its kind. The neutral form of the reporter's `union-identity`.
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

fn ir_of(model: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("model.yaml"),
        RawSpecFile::parse(model).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite_of(model: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(model));
    assert_eq!(synthesis.refusals.len(), 0, "{:?}", synthesis.refusals);
    synthesis.suite
}

fn ir() -> EssIr {
    ir_of(MODEL)
}

fn suite() -> ConformanceSuite {
    suite_of(MODEL)
}

/// `MODEL` at `ess/27`, where the box also stores a serial its event mints, read from the payload
/// (`{payload: <Event>.<field>}`, beyond10x/ess#545): a row claim about the serial reads the
/// published value, which each variant run publishes anew.
fn published_serial() -> String {
    let edits = [
        ("format: ess/22", "format: ess/27"),
        (
            "  - {name: store.box.BoxId, kind: newtype, of: Uuid}\n",
            "  - {name: store.box.BoxId, kind: newtype, of: Uuid}\n  - {name: store.box.Serial, \
             kind: newtype, of: Uuid}\n",
        ),
        (
            "      - {name: kind, type: store.box.Kind}\n    lifecycle",
            "      - {name: kind, type: store.box.Kind}\n      - {name: serial, type: \
             store.box.Serial}\n    lifecycle",
        ),
        (
            "      - {name: kind, type: store.box.Kind}\ncommands",
            "      - {name: kind, type: store.box.Kind}\n      - {name: serial, type: \
             store.box.Serial}\ncommands",
        ),
        (
            "            kind: input.kind\n        sets: {kind: input.kind}\n",
            "            kind: input.kind\n            serial: {generated: true}\n        sets:\n          \
             kind: input.kind\n          serial: {payload: store.box.BoxMade.serial}\n",
        ),
    ];
    let mut model = MODEL.to_owned();
    for (from, to) in edits {
        assert!(model.contains(from), "fixture edit did not land: {from}");
        model = model.replacen(from, to, 1);
    }
    // The view's field list closes the model.
    model.push_str("      - {name: serial, type: store.box.Serial}\n");
    model
}

fn steps<'s>(suite: &'s ConformanceSuite, id: &str) -> &'s [ScenarioStep] {
    &suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}: {:?}", suite.scenarios.keys()))
        .1
        .steps
}

/// Each view row claim made after the `n`th invocation of `MakeBox`, with the instance the steps
/// since that invocation captured from `BoxMade`, if any.
fn row_claims(steps: &[ScenarioStep]) -> Vec<(usize, Option<InstanceName>, ScenarioValue)> {
    let mut sent = 0;
    let mut captured = None;
    let mut claims = Vec::new();
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "store.box.MakeBox" =>
            {
                sent += 1;
                captured = None;
            }
            ScenarioStep::CaptureInstance {
                instance, event, ..
            } if event.to_string() == "store.box.BoxMade" => captured = Some(instance.clone()),
            ScenarioStep::ExpectView {
                expectation:
                    ViewExpectation::Contains { fields } | ViewExpectation::Excludes { fields },
                ..
            } => {
                if let Some(identity) = fields.get("box_id") {
                    claims.push((sent, captured.clone(), identity.clone()));
                }
            }
            _ => {}
        }
    }
    claims
}

fn rust_statuses(model: &str, suite: &ConformanceSuite) -> Vec<(String, Status, Option<String>)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir_of(model)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let diagnostics = result
                .diagnostics()
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect::<Vec<_>>();
            (
                result.scenario.to_string(),
                result.status,
                (!diagnostics.is_empty()).then(|| diagnostics.join("\n")),
            )
        })
        .collect()
}

/// The TypeScript runtime's verdicts over the run the Rust reference recorded, which it must equal.
fn typescript_matches_the_reference(label: &str, suite: &ConformanceSuite) {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let target = support_go::Recorder::new(Interpreted::for_model(ir()));
    let reference = support_go::rust_outcomes_admitted(&admitted, &target);
    let package = support_ts::Package::new(label, suite);
    let replayed = package.replay(&admitted, &target.transcript(), false);
    assert_ne!(replayed.exit, Some(2), "{}", replayed.log);
    assert!(
        replayed.divergences.is_empty(),
        "{label}: the TypeScript runtime sent requests the reference runner did not:\n{}\n{}",
        replayed.divergences.join("\n"),
        replayed.log
    );
    assert_eq!(replayed.outcomes, reference, "{label}: {}", replayed.log);
}

#[test]
fn a_second_variant_row_is_read_under_its_own_identity() {
    let suite = suite();
    let claims = row_claims(steps(&suite, MADE));
    // The full invocation and one further run per variant past the first.
    assert!(
        claims.iter().any(|(sent, _, _)| *sent >= 2),
        "the variant run reads its row: {claims:#?}"
    );
    for (sent, captured, identity) in &claims {
        if *sent < 2 {
            continue;
        }
        let captured = captured
            .as_ref()
            .unwrap_or_else(|| panic!("run {sent} captured the row it made: {claims:#?}"));
        assert_eq!(
            identity,
            &ScenarioValue::instance(captured.clone()),
            "run {sent} reads its row under its own identity: {claims:#?}"
        );
    }
    // The reference target holds the second row under its own identity, and passes.
    for (id, status, message) in rust_statuses(MODEL, &suite) {
        assert_eq!(status, Status::Passed, "{id}: {message:?}");
    }
    let verdicts =
        support_go::assert_parity("union-row-identity", &suite, Interpreted::for_model(ir()));
    assert_eq!(support_go::not_passed(&verdicts).len(), 0, "{verdicts:?}");
    typescript_matches_the_reference("union-row-identity", &suite);
}

/// Every value a view row claim reads from an event, after the `n`th invocation of `MakeBox`.
fn observed_claims(steps: &[ScenarioStep]) -> Vec<(usize, String, ScenarioValue)> {
    let mut sent = 0;
    let mut claims = Vec::new();
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "store.box.MakeBox" =>
            {
                sent += 1;
            }
            ScenarioStep::ExpectView {
                expectation:
                    ViewExpectation::Contains { fields } | ViewExpectation::Excludes { fields },
                ..
            } => claims.extend(
                fields
                    .iter()
                    .filter(|(_, value)| matches!(value, ScenarioValue::Observed { .. }))
                    .map(|(field, value)| (sent, field.clone(), value.clone())),
            ),
            _ => {}
        }
    }
    claims
}

#[test]
fn a_variant_row_never_claims_a_value_an_earlier_run_published_too() {
    let model = published_serial();
    let suite = suite_of(&model);
    let steps = steps(&suite, MADE);
    assert!(
        row_claims(steps).iter().any(|(sent, _, _)| *sent >= 2),
        "the variant run reads its row"
    );
    // The full invocation reads the serial its event published; a variant run cannot, because the
    // full invocation's `BoxMade` names another serial and the reference names no occurrence.
    let ambiguous: Vec<_> = observed_claims(steps)
        .into_iter()
        .filter(|(sent, _, _)| *sent >= 2)
        .collect();
    assert_eq!(ambiguous, [], "{ambiguous:#?}");
    for (id, status, message) in rust_statuses(&model, &suite) {
        assert_eq!(status, Status::Passed, "{id}: {message:?}");
    }
}

/// Every `{kind: observed, event: store.box.BoxMade}` value anywhere in `node`.
fn observed_box_made(node: &serde_json::Value, found: &mut Vec<serde_json::Value>) {
    match node {
        serde_json::Value::Object(map) => {
            if map.get("kind").and_then(serde_json::Value::as_str) == Some("observed")
                && map.get("event").and_then(serde_json::Value::as_str) == Some("store.box.BoxMade")
            {
                found.push(node.clone());
            }
            for value in map.values() {
                observed_box_made(value, found);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                observed_box_made(item, found);
            }
        }
        _ => {}
    }
}

/// The runners resolve an observed reference to the first occurrence of its event, so synthesis
/// never writes one where an earlier run already published the event: the row each further run
/// made is read under the capture that follows it, the latest occurrence at that point. Checked in
/// every step a further run writes, not only its view claims, for both models.
#[test]
fn no_step_after_a_variant_run_reads_an_event_an_earlier_run_published() {
    for model in [MODEL.to_owned(), published_serial()] {
        let suite = suite_of(&model);
        let mut sent = 0;
        let mut late = Vec::new();
        for step in steps(&suite, MADE) {
            if let ScenarioStep::ExecuteCommand { command, .. } = step {
                if command.to_string() == "store.box.MakeBox" {
                    sent += 1;
                }
            }
            if sent >= 2 {
                let mut found = Vec::new();
                observed_box_made(&serde_json::to_value(step).unwrap(), &mut found);
                late.extend(found.into_iter().map(|value| (sent, value)));
            }
        }
        assert!(sent >= 2, "the variant runs ran");
        assert_eq!(late, [], "{late:#?}");
    }
}
