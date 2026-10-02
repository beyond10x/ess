//! Explicit upstream fixture state is validated and stored by the actual interpreter.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    authored::{compile as author, Source as AuthoredSource},
    interpret::Interpreted,
    report::Status,
    target::*,
    AdmittedSuite, Runner, ScenarioStep,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use std::collections::BTreeMap;

const MODEL: &str = r"format: ess/1
system: calls
version: v1
domain: calls.history
entities:
  - name: calls.history.CallRecord
    identity: {name: call_id, type: Uuid}
    fields:
      - {name: started_at, type: Timestamp}
      - {name: duration_seconds, type: Integer}
      - {name: note, type: 'Optional<String>'}
    lifecycle: {initial: Completed, states: [Completed], terminal: [Completed]}
    invariants: ['duration_seconds >= 0']
events:
  - {name: calls.history.Recorded, fields: []}
views:
  - name: calls.history.CallHistory
    source: calls.history.CallRecord
    consistency: read_your_writes
    order_by: [started_at desc]
    fields:
      - {name: call_id, type: Uuid}
      - {name: started_at, type: Timestamp}
      - {name: duration_seconds, type: Integer}
      - {name: note, type: 'Optional<String>'}
      - {name: state, type: calls.history.CallRecord.State}
";
const SCENARIO: &str = r"type: ess-scenario/2
domain: calls.history
scenario: backend-history
summary: Explicit upstream rows are immediately visible without a creating command.
arrange:
  - instance: earlier
    entity: calls.history.CallRecord
    setup:
      identity: 00000000-0000-4000-8000-000000000001
      fields: {started_at: '2026-01-05T09:00:00Z', duration_seconds: 12}
      state: Completed
  - instance: later
    entity: calls.history.CallRecord
    setup:
      identity: 00000000-0000-4000-8000-000000000002
      fields: {started_at: '2026-01-05T09:01:00Z', duration_seconds: 24, note: null}
      state: Completed
assert:
  - view: calls.history.CallHistory
    contains: {call_id: {$instance: earlier}, duration_seconds: 12}
  - view: calls.history.CallHistory
    contains: {call_id: {$instance: later}, duration_seconds: 24}
  - view: calls.history.CallHistory
    counts: {at_least: 2, at_most: 2}
  - view: calls.history.CallHistory
    at: {row: first, fields: {call_id: {$instance: later}}}
";

fn model(source: &str) -> EssIr {
    compile(
        &Specification::assemble([(
            Source::new("setup.yaml"),
            RawSpecFile::parse(source).unwrap(),
        )])
        .unwrap(),
        &SourceMap::new(),
    )
    .unwrap()
}
fn context() -> ScenarioContext {
    ScenarioContext::new(
        "calls.history/authored/setup".parse().unwrap(),
        CorrelationId::new("setup").unwrap(),
    )
}
fn request() -> EntitySetupRequest {
    EntitySetupRequest {
        entity: "calls.history.CallRecord".parse().unwrap(),
        identity: Node::Text("00000000-0000-4000-8000-000000000001".into()),
        fields: BTreeMap::from([
            (
                "started_at".into(),
                Node::Text("2026-01-05T09:00:00Z".into()),
            ),
            ("duration_seconds".into(), Node::Number(12_u32.into())),
        ]),
        state: "Completed".parse().unwrap(),
        correlation: context().correlation,
    }
}
fn rows(target: &Interpreted) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: "calls.history.CallHistory".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}
fn events(target: &Interpreted) -> Vec<ObservedEvent> {
    target
        .observe_events(EventObservationRequest {
            event: "calls.history.Recorded".parse().unwrap(),
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
}

#[test]
fn authored_setup_reaches_real_storage_and_resets_between_scenarios() {
    let ir = model(MODEL);
    assert!(ir.commands().is_empty());
    let mut suite = ess_conformance::synthesize::synthesize(&ir).suite;
    suite.provenance.suite_version = "ess-conformance/6".parse().unwrap();
    let authored = author(&ir, &[AuthoredSource::new("upstream.yaml", SCENARIO)]);
    assert!(authored.is_complete(), "{:?}", authored.refusals);
    suite.scenarios = authored.scenarios;
    let second = SCENARIO
        .replace("backend-history", "second-history")
        .replace("000000000001", "000000000003")
        .replace("000000000002", "000000000004");
    let authored = author(&ir, &[AuthoredSource::new("second.yaml", &second)]);
    assert!(authored.is_complete(), "{:?}", authored.refusals);
    suite.scenarios.extend(authored.scenarios);
    assert_eq!(suite.scenarios.len(), 2);
    for scenario in suite.scenarios.values() {
        assert_eq!(
            scenario
                .steps
                .iter()
                .filter(|step| matches!(step, ScenarioStep::EstablishEntity { .. }))
                .count(),
            2
        );
        assert!(!scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExecuteCommand { .. })));
    }
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = Interpreted::for_model(ir);
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    assert_eq!(report.scenarios.len(), 2);
    for run in &report.scenarios {
        assert_eq!(run.status, Status::Passed, "{run:?}");
        assert!(run
            .checks
            .iter()
            .all(|check| check.status == Status::Passed));
    }
    assert!(rows(&target).is_empty());
    assert!(events(&target).is_empty());
}

#[test]
fn invalid_facts_never_replace_or_add_rows() {
    let target = Interpreted::for_model(model(MODEL));
    target.begin_scenario(&context()).unwrap();
    target.establish_entity(request()).unwrap();
    let before = rows(&target);
    let mut invalid = Vec::new();
    let mut bad = request();
    bad.identity = Node::Bool(true);
    invalid.push(bad);
    let mut bad = request();
    bad.identity = Node::Text("not-a-uuid".into());
    invalid.push(bad);
    let mut bad = request();
    bad.fields
        .insert("duration_seconds".into(), Node::Text("twelve".into()));
    invalid.push(bad);
    let mut bad = request();
    bad.fields
        .insert("duration_seconds".into(), Node::Number((-1_i64).into()));
    invalid.push(bad);
    let mut bad = request();
    bad.fields.remove("duration_seconds");
    invalid.push(bad);
    let mut bad = request();
    bad.fields.insert("extra".into(), Node::Bool(true));
    invalid.push(bad);
    let mut bad = request();
    bad.fields.insert("call_id".into(), bad.identity.clone());
    invalid.push(bad);
    let mut bad = request();
    bad.state = "Missing".parse().unwrap();
    invalid.push(bad);
    let mut bad = request();
    bad.entity = "calls.history.Undeclared".parse().unwrap();
    invalid.push(bad);
    for mut bad in invalid {
        if bad.identity == request().identity {
            bad.identity = Node::Text("00000000-0000-4000-8000-000000000002".into());
        }
        assert!(matches!(
            target.establish_entity(bad),
            Err(TargetError::Unavailable { .. })
        ));
        assert_eq!(rows(&target), before);
        assert!(events(&target).is_empty());
    }
    let mut duplicate = request();
    duplicate
        .fields
        .insert("duration_seconds".into(), Node::Number(99_u32.into()));
    assert!(target.establish_entity(duplicate).is_err());
    assert_eq!(rows(&target), before);
    let mut valid = request();
    valid.identity = Node::Text("00000000-0000-4000-8000-000000000002".into());
    target.establish_entity(valid).unwrap();
    assert_eq!(rows(&target).len(), 2);
}

#[test]
fn setup_requires_current_scenario_authority_and_end_revokes_it() {
    let target = Interpreted::for_model(model(MODEL));
    assert!(target.establish_entity(request()).is_err());
    target.begin_scenario(&context()).unwrap();
    let mut wrong = request();
    wrong.correlation = CorrelationId::new("another-scenario").unwrap();
    assert!(target.establish_entity(wrong).is_err());
    assert!(rows(&target).is_empty());
    target.establish_entity(request()).unwrap();
    target.end_scenario(&context()).unwrap();
    assert!(target.establish_entity(request()).is_err());
    assert!(rows(&target).is_empty());
    target.begin_scenario(&context()).unwrap();
    target.establish_entity(request()).unwrap();
    assert_eq!(rows(&target).len(), 1);
    target.begin_scenario(&context()).unwrap();
    assert!(rows(&target).is_empty());
}

#[test]
fn setup_is_immediately_visible_even_to_eventual_views() {
    let target = Interpreted::for_model(model(&MODEL.replace("read_your_writes", "eventual")));
    target.begin_scenario(&context()).unwrap();
    target.establish_entity(request()).unwrap();
    for _ in 0..3 {
        assert_eq!(rows(&target).len(), 1);
    }
    assert!(events(&target).is_empty());
}

#[test]
fn unknown_invariant_and_nontext_store_identity_are_honest_refusals() {
    let target = Interpreted::for_model(model(
        &MODEL.replace("duration_seconds >= 0", "note == recorded"),
    ));
    target.begin_scenario(&context()).unwrap();
    assert!(target.establish_entity(request()).is_err());
    assert!(rows(&target).is_empty());
    let ir = model(&MODEL.replace("type: Uuid", "type: Integer"));
    let target = Interpreted::for_model(ir);
    target.begin_scenario(&context()).unwrap();
    let mut integer = request();
    integer.identity = Node::Number(7_u32.into());
    assert!(target
        .establish_entity(integer)
        .unwrap_err()
        .is_unsupported());
    assert!(rows(&target).is_empty());
}

const CREATOR: &str = r"
commands:
  - name: calls.history.Import
    input:
      - {name: started_at, type: Timestamp}
      - {name: duration_seconds, type: Integer}
    outcomes:
      - name: imported
        creates: calls.history.CallRecord
        instance: call_id
        sets: {started_at: input.started_at, duration_seconds: input.duration_seconds}
        emits: [calls.history.Imported]
        payload:
          calls.history.Imported:
            call_id: {generated: true}
            receipt: {generated: true}
";

#[test]
fn generated_creation_skips_setup_identities_and_preserves_all_existing_rows() {
    let source = MODEL.replace(
        "  - {name: calls.history.Recorded, fields: []}",
        "  - {name: calls.history.Recorded, fields: []}\n  - name: calls.history.Imported\n    fields: [{name: call_id, type: Uuid}, {name: receipt, type: Uuid}]",
    ) + CREATOR;
    let target = Interpreted::for_model(model(&source.replace("format: ess/1", "format: ess/4")));
    target.begin_scenario(&context()).unwrap();
    for suffix in [1, 2, 5] {
        let mut setup = request();
        setup.identity = Node::Text(format!("00000000-0000-4000-8000-{suffix:012}"));
        target.establish_entity(setup).unwrap();
    }
    let before = rows(&target);
    for (identity, receipt) in [(3, 4), (6, 7)] {
        let result = target
            .execute_command(SemanticCommandRequest {
                command: "calls.history.Import".parse().unwrap(),
                actor: None,
                caller: None,
                input: request().fields,
                correlation: context().correlation,
            })
            .unwrap();
        assert_eq!(
            result.outcome.unwrap().to_string(),
            "calls.history.Import/imported"
        );
        assert_eq!(result.direct_events.len(), 1);
        let payload = &result.direct_events[0].payload;
        assert_eq!(
            payload["call_id"],
            Node::Text(format!("00000000-0000-4000-8000-{identity:012}"))
        );
        assert_eq!(
            payload["receipt"],
            Node::Text(format!("00000000-0000-4000-8000-{receipt:012}"))
        );
        for old in &before {
            assert!(rows(&target).contains(old));
        }
    }
    assert_eq!(rows(&target).len(), 5);
}
