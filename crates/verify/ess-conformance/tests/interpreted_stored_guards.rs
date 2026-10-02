//! Stored guards read source-determined rows, with state and input kept in their own namespaces.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{interpret::Interpreted, report::Status, target::*, AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node};
use std::collections::BTreeMap;

const STATE: &str = include_str!("fixtures/state-in-subject-predicate.yaml");
fn model(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("model.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn invoke(
    target: &Interpreted,
    command: &str,
    input: BTreeMap<String, Node>,
) -> Result<SemanticCommandResult, TargetError> {
    target.execute_command(SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input,
        correlation: CorrelationId::new("stored-guard").unwrap(),
    })
}
#[test]
fn source_built_stored_guard_suites_execute_every_scenario() {
    for source in [
        STATE,
        include_str!("fixtures/stored-field-guards.yaml"),
        include_str!("fixtures/subject-guard-input.yaml"),
    ] {
        let ir = model(source);
        let suite = ess_conformance::synthesize(&ir).suite;
        assert!(!suite.scenarios.is_empty());
        let admitted = AdmittedSuite::from_suite(&suite).unwrap();
        let report = Runner::for_suite(&suite)
            .run_admitted(&admitted, &Interpreted::for_model(ir))
            .into_report();
        for scenario in report.scenarios {
            assert_eq!(
                scenario.status,
                Status::Passed,
                "{}: {:?}",
                scenario.scenario,
                scenario.checks
            );
        }
    }
}
#[test]
fn state_and_stored_note_both_decide_before_the_default_move() {
    for (ready, note, expected) in [
        (true, "held", "kept-ready"),
        (true, "", "pending"),
        (false, "held", "pending"),
        (false, "", "pending"),
    ] {
        let target = Interpreted::for_model(model(STATE));
        let placed = invoke(
            &target,
            "demo.shop.PlaceOrder",
            BTreeMap::from([("hold_note".into(), Node::Text(note.into()))]),
        )
        .unwrap();
        let identity = placed.direct_events[0].payload["order_id"].clone();
        let input = BTreeMap::from([("order_id".into(), identity)]);
        if ready {
            invoke(&target, "demo.shop.MarkReady", input.clone()).unwrap();
        }
        let answer = invoke(&target, "demo.shop.ReportPending", input).unwrap();
        assert_eq!(answer.outcome.unwrap().outcome.to_string(), expected);
    }
}
#[test]
fn an_unwritten_required_stored_fact_is_not_guessed() {
    let source = STATE.replace("        sets: {hold_note: input.hold_note}\n", "");
    let target = Interpreted::for_model(model(&source));
    let placed = invoke(
        &target,
        "demo.shop.PlaceOrder",
        BTreeMap::from([("hold_note".into(), Node::Text("held".into()))]),
    )
    .unwrap();
    let input = BTreeMap::from([(
        "order_id".into(),
        placed.direct_events[0].payload["order_id"].clone(),
    )]);
    invoke(&target, "demo.shop.MarkReady", input.clone()).unwrap();
    let answer = invoke(&target, "demo.shop.ReportPending", input);
    assert!(
        matches!(answer, Err(TargetError::Unsupported { .. })),
        "{answer:?}"
    );
}

#[test]
fn a_false_input_conjunct_does_not_require_an_unwritten_stored_fact() {
    let source = STATE.replace("        sets: {hold_note: input.hold_note}\n", "")
        .replace("  - name: demo.shop.ReportPending\n    input: [{name: order_id, type: demo.shop.OrderId}]", "  - name: demo.shop.ReportPending\n    input: [{name: order_id, type: demo.shop.OrderId}, {name: allow, type: Boolean}]")
        .replace("      - name: kept-ready\n", "      - name: kept-ready\n        when: allow == true\n");
    let target = Interpreted::for_model(model(&source));
    let placed = invoke(
        &target,
        "demo.shop.PlaceOrder",
        BTreeMap::from([("hold_note".into(), Node::Text("held".into()))]),
    )
    .unwrap();
    let mut input = BTreeMap::from([(
        "order_id".into(),
        placed.direct_events[0].payload["order_id"].clone(),
    )]);
    invoke(&target, "demo.shop.MarkReady", input.clone()).unwrap();
    input.insert("allow".into(), Node::Bool(false));
    let answer = invoke(&target, "demo.shop.ReportPending", input).unwrap();
    assert_eq!(answer.outcome.unwrap().outcome.to_string(), "pending");
}

#[test]
fn stored_and_input_timestamps_compare_by_instant_and_now_stays_unsupplied() {
    let source = STATE.replace("{name: hold_note, type: String}", "{name: hold_note, type: Timestamp}")
        .replace("  - name: demo.shop.ReportPending\n    input: [{name: order_id, type: demo.shop.OrderId}]", "  - name: demo.shop.ReportPending\n    input: [{name: order_id, type: demo.shop.OrderId}, {name: limit, type: Timestamp}]")
        .replace("            all:\n              - state == Ready\n              - hold_note != \"\"", "            hold_note < input.limit");
    assert!(source.contains("hold_note < input.limit"));
    for (stamp, expected) in [
        ("2026-01-05T10:00:01+02:00", "kept-ready"),
        ("2026-01-05T10:00:01Z", "pending"),
    ] {
        for unsupplied_now in [false, true] {
            let text = if unsupplied_now {
                source.replace(
                    "      - name: kept-ready\n",
                    "      - name: kept-ready\n        when: limit < now\n",
                )
            } else {
                source.clone()
            };
            let target = Interpreted::for_model(model(&text));
            let placed = invoke(
                &target,
                "demo.shop.PlaceOrder",
                BTreeMap::from([("hold_note".into(), Node::Text(stamp.into()))]),
            )
            .unwrap();
            let input = BTreeMap::from([
                (
                    "order_id".into(),
                    placed.direct_events[0].payload["order_id"].clone(),
                ),
                ("limit".into(), Node::Text("2026-01-05T09:00:00Z".into())),
            ]);
            let answer = invoke(&target, "demo.shop.ReportPending", input);
            if unsupplied_now {
                assert!(
                    matches!(answer, Err(TargetError::Unsupported { .. })),
                    "{answer:?}"
                );
            } else {
                assert_eq!(
                    answer.unwrap().outcome.unwrap().outcome.to_string(),
                    expected
                );
            }
        }
    }
}
