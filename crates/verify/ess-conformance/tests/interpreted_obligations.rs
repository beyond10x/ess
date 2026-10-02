//! Missing facts are not delivery failures; a scripted provider failure can escalate.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{interpret::Interpreted, target::*};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node, time::Timestamp};
use std::collections::BTreeMap;

const MODEL: &str = r"format: ess/18
system: owed
version: v1
domain: owed.api
events:
  - {name: owed.api.Started, fields: [{name: stamp, type: Timestamp}]}
  - {name: owed.api.Escalated, fields: []}
  - {name: owed.api.Sent, fields: []}
errors:
  - {name: owed.api.Unavailable, summary: Provider refused., fields: []}
commands:
  - name: owed.api.Start
    input: [{name: stamp, type: Timestamp}]
    outcomes:
      - name: started
        emits: [owed.api.Started]
        payload: {owed.api.Started: {stamp: input.stamp}}
  - name: owed.api.Send
    input: [{name: stamp, type: Timestamp}]
    outcomes:
      - {name: sent, emits: [owed.api.Sent]}
      - {name: failed, external: the provider refused, error: owed.api.Unavailable}
bindings:
  - id: send
    when: {event: owed.api.Started}
    invoke: {command: owed.api.Send}
    mapping: {stamp: event.stamp}
    delivery: at_least_once
    on_failure: {escalate: {emits: owed.api.Escalated}}
";

#[test]
fn missing_fact_never_escalates_but_scripted_delivery_failure_does() {
    for (supplied, missing_clock) in [(false, false), (true, false), (false, true)] {
        let source = if missing_clock {
            MODEL.replace("      - {name: sent, emits: [owed.api.Sent]}",
                "      - {name: too-late, when: stamp <= now, error: owed.api.Unavailable}\n      - {name: sent, emits: [owed.api.Sent]}")
        } else {
            MODEL.into()
        };
        let spec = Specification::assemble([(
            Source::new("owed.yaml"),
            RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let target = Interpreted::for_model(compile(&spec, &SourceMap::new()).unwrap());
        let correlation = CorrelationId::new("owed").unwrap();
        let context = ScenarioContext::new(
            "owed.api/authored/obligation".parse().unwrap(),
            correlation.clone(),
        );
        target.begin_scenario(&context).unwrap();
        if supplied {
            target
                .configure_external_outcome(ExternalOutcomeControl {
                    force: ess_conformance::scenario::OutcomeRef::new(
                        "owed.api.Send".parse().unwrap(),
                        "failed".parse().unwrap(),
                    ),
                    correlation: correlation.clone(),
                })
                .unwrap();
        }
        let answer = target.execute_command(SemanticCommandRequest {
            command: "owed.api.Start".parse().unwrap(),
            actor: None,
            caller: None,
            input: BTreeMap::from([("stamp".into(), Node::Text("2026-10-02T00:00:00Z".into()))]),
            correlation: correlation.clone(),
        });
        if missing_clock {
            assert!(
                matches!(answer, Err(TargetError::Unsupported { .. })),
                "{answer:?}"
            );
        } else {
            assert!(answer.is_ok(), "{answer:?}");
        }
        let events = target
            .observe_events(EventObservationRequest {
                event: "owed.api.Escalated".parse().unwrap(),
                correlation,
                deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
            })
            .unwrap();
        assert_eq!(events.len(), usize::from(supplied));
    }
}
