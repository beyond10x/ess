//! Scenario-supplied delivery facts and relative elapsed observations through the actual target.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    interpret::Interpreted, report::Status, scenario::Elapsed, target::*, AdmittedSuite, Runner,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node, time::Timestamp};
use std::collections::BTreeMap;

const MODEL: &str = include_str!("fixtures/delivery-context.yaml");
const RECEIVED: &str = "demo.inbox.MessageReceived";
const RECORDED: &str = "demo.inbox.MessageRecorded";

fn model(text: &str) -> EssIr {
    compile(
        &Specification::assemble([(Source::new("facts.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap(),
        &SourceMap::new(),
    )
    .unwrap()
}
fn context() -> ScenarioContext {
    ScenarioContext::new(
        "demo.inbox/authored/scenario-facts".parse().unwrap(),
        CorrelationId::new("scenario-facts").unwrap(),
    )
}
fn fields(values: &[(&str, &str)]) -> BTreeMap<String, Node> {
    values
        .iter()
        .map(|(key, value)| ((*key).to_owned(), Node::Text((*value).to_owned())))
        .collect()
}
fn delivery(message: &str, account: &str) -> EventDeliveryRequest {
    EventDeliveryRequest {
        event: RECEIVED.parse().unwrap(),
        authority: "account-messages".into(),
        payload: fields(&[("message_id", message), ("from", "sender")]),
        context: fields(&[("account_id", account)]),
        correlation: context().correlation,
    }
}
fn invoke(target: &Interpreted) {
    target
        .execute_command(SemanticCommandRequest {
            command: "demo.inbox.RecordMessage".parse().unwrap(),
            actor: None,
            caller: None,
            input: fields(&[
                ("account_id", "account"),
                ("message_id", "message"),
                ("peer", "sender"),
            ]),
            correlation: context().correlation,
        })
        .unwrap();
}
fn invocations(target: &Interpreted) -> Vec<ObservedInvocation> {
    target
        .observe_invocations(InvocationObservationRequest {
            binding: "received".parse().unwrap(),
            command: "demo.inbox.RecordMessage".parse().unwrap(),
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
}
fn mark(target: &Interpreted, name: &str) {
    target
        .mark_instant(InstantMark {
            instant: name.parse().unwrap(),
            correlation: context().correlation,
        })
        .unwrap();
}
fn elapsed(target: &Interpreted, name: &str, hold: u32, event: &str) -> ElapsedObservation {
    target
        .observe_elapsed(ElapsedObservationRequest {
            instant: name.parse().unwrap(),
            hold: Elapsed::seconds(hold),
            watching: Some(event.parse().unwrap()),
            correlation: context().correlation,
        })
        .unwrap()
}

#[test]
fn interpreted_delivery_context_executes_all_four_generated_binding_claims() {
    let ir = model(MODEL);
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    for id in ["flow", "mapping", "delivery", "on-failure"] {
        let name = format!("received/binding/{id}");
        let result = report
            .scenarios
            .iter()
            .find(|case| case.scenario.to_string() == name)
            .unwrap();
        assert_eq!(result.status, Status::Passed, "{name}: {:?}", result.checks);
    }
}

#[test]
fn delivery_retains_latest_context_and_refuses_bad_facts_before_effects() {
    let target = Interpreted::for_model(model(MODEL));
    target.begin_scenario(&context()).unwrap();
    target
        .deliver_event(delivery("first", "account-a"))
        .unwrap();
    target
        .deliver_event(delivery("second", "account-b"))
        .unwrap();
    for defect in ["authority", "context", "type", "payload", "correlation"] {
        let mut bad = delivery("bad", "account-c");
        match defect {
            "authority" => bad.authority = "other-channel".into(),
            "context" => {
                bad.context.clear();
            }
            "type" => {
                bad.context.insert("account_id".into(), Node::Seq(vec![]));
            }
            "payload" => {
                bad.payload.remove("message_id");
            }
            _ => bad.correlation = CorrelationId::new("other-scenario").unwrap(),
        }
        assert!(target.deliver_event(bad).is_err(), "{defect}");
        assert_eq!(invocations(&target).len(), 2, "{defect} caused an effect");
    }
    target
        .redeliver_event(RedeliveryRequest {
            event: RECEIVED.parse().unwrap(),
            correlation: context().correlation,
        })
        .unwrap();
    let seen = invocations(&target);
    assert_eq!(seen.len(), 3);
    assert_eq!(seen[2].input["account_id"], Node::Text("account-b".into()));
    assert_eq!(seen[2].input["message_id"], Node::Text("second".into()));
    let external_log = target
        .observe_events(EventObservationRequest {
            event: RECEIVED.parse().unwrap(),
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap();
    assert!(
        external_log.is_empty(),
        "delivery is not a model publication"
    );
    target.end_scenario(&context()).unwrap();
    target.begin_scenario(&context()).unwrap();
    assert!(target
        .redeliver_event(RedeliveryRequest {
            event: RECEIVED.parse().unwrap(),
            correlation: context().correlation,
        })
        .is_err());
}

#[test]
fn relative_holds_observe_real_publications_and_do_not_invent_absolute_time() {
    let target = Interpreted::for_model(model(MODEL));
    target.begin_scenario(&context()).unwrap();
    invoke(&target); // Earlier publication must not enter the later marked interval.
    mark(&target, "opened");
    assert_eq!(
        elapsed(&target, "opened", 2, RECORDED),
        ElapsedObservation {
            elapsed_ms: 2000,
            published: 0
        }
    );
    invoke(&target);
    assert_eq!(
        elapsed(&target, "opened", 0, RECORDED),
        ElapsedObservation {
            elapsed_ms: 2000,
            published: 1
        }
    );
    mark(&target, "later");
    assert_eq!(
        elapsed(&target, "opened", 5, RECORDED),
        ElapsedObservation {
            elapsed_ms: 5000,
            published: 1
        }
    );
    assert_eq!(
        elapsed(&target, "later", 1, RECEIVED),
        ElapsedObservation {
            elapsed_ms: 3000,
            published: 0
        }
    );
    assert_eq!(elapsed(&target, "later", 0, RECORDED).published, 0);
    target.end_scenario(&context()).unwrap();
    target.begin_scenario(&context()).unwrap();
    assert!(target
        .observe_elapsed(ElapsedObservationRequest {
            instant: "opened".parse().unwrap(),
            hold: Elapsed::seconds(0),
            watching: None,
            correlation: context().correlation,
        })
        .is_err());
    mark(&target, "opened");
    assert_eq!(
        elapsed(&target, "opened", 0, RECORDED),
        ElapsedObservation {
            elapsed_ms: 0,
            published: 0
        }
    );
}

#[test]
fn relative_time_does_not_guess_absolute_guards_or_periodic_host_facts() {
    let target = Interpreted::for_model(model(include_str!("fixtures/current-time-guard.yaml")));
    target.begin_scenario(&context()).unwrap();
    mark(&target, "relative-only");
    let error = target
        .execute_command(SemanticCommandRequest {
            command: "demo.jobs.ScheduleJob".parse().unwrap(),
            actor: None,
            caller: None,
            input: fields(&[("starts_at", "2026-10-02T00:00:00Z")]),
            correlation: context().correlation,
        })
        .unwrap_err();
    assert!(error.is_unsupported(), "{error}");
    let target = Interpreted::for_model(model(include_str!(
        "../../../specify/ess-domain/tests/fixtures/periodic.yaml"
    )));
    target.begin_scenario(&context()).unwrap();
    mark(&target, "host-not-supplied");
    let error = target
        .observe_elapsed(ElapsedObservationRequest {
            instant: "host-not-supplied".parse().unwrap(),
            hold: Elapsed::seconds(5),
            watching: None,
            correlation: context().correlation,
        })
        .unwrap_err();
    assert!(error.is_unsupported(), "{error}");
    assert!(error.to_string().contains("periodic host facts"));
}

#[test]
fn delivery_authority_and_payload_context_decoys_stay_independent() {
    let source = MODEL.replace(
        "- {name: from, type: String}",
        "- {name: from, type: String}\n      - {name: account_id, type: demo.inbox.AccountId}",
    );
    let binding = source.split("bindings:\n").nth(1).unwrap();
    let other = binding
        .replace("id: received", "id: another")
        .replace("account-messages", "another-channel");
    let target = Interpreted::for_model(model(&format!("{source}\n{other}")));
    target.begin_scenario(&context()).unwrap();
    let mut request = delivery("one", "context-account");
    request
        .payload
        .insert("account_id".into(), Node::Text("payload-decoy".into()));
    target.deliver_event(request).unwrap();
    let seen = invocations(&target);
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0].input["account_id"],
        Node::Text("context-account".into())
    );
    let other = target
        .observe_invocations(InvocationObservationRequest {
            binding: "another".parse().unwrap(),
            command: "demo.inbox.RecordMessage".parse().unwrap(),
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap();
    assert!(other.is_empty(), "another channel's binding ran");
    let mut latest = delivery("second", "second-context");
    latest.authority = "another-channel".into();
    latest.payload.insert(
        "account_id".into(),
        Node::Text("second-payload-decoy".into()),
    );
    target.deliver_event(latest).unwrap();
    target
        .redeliver_event(RedeliveryRequest {
            event: RECEIVED.parse().unwrap(),
            correlation: context().correlation,
        })
        .unwrap();
    assert_eq!(invocations(&target).len(), 1);
    let other = target
        .observe_invocations(InvocationObservationRequest {
            binding: "another".parse().unwrap(),
            command: "demo.inbox.RecordMessage".parse().unwrap(),
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap();
    assert_eq!(other.len(), 2);
    assert!(other
        .iter()
        .all(|invocation| invocation.input["account_id"] == Node::Text("second-context".into())));
}
