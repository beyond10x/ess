//! Conformance for a binding that reads its delivery context (beyond10x/ess#195, `ess/18`).
//!
//! The decision: the target protocol may deliver an event from an external channel together with
//! its context ([`ConformanceTarget::deliver_event`]) and may answer unsupported, which leaves the
//! scenario `unsupported` with the target's reason; a redelivery carries the context of the
//! occurrence it repeats. The suite delivers one event under two contexts and requires that the
//! mapped command input follows each context.
//!
//! Two mutants must each fail the suite: a target that ignores the context, and one that uses the
//! first delivery's context when it redelivers a later occurrence.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{BindingRef, CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const INBOX: &str = include_str!("fixtures/delivery-context.yaml");

const BINDING: &str = "received";
const RECORD: &str = "demo.inbox.RecordMessage";
const RECEIVED: &str = "demo.inbox.MessageReceived";
const RECORDED: &str = "demo.inbox.MessageRecorded";

const MAPPING: &str = "received/binding/mapping";
const DELIVERY: &str = "received/binding/delivery";
const FLOW: &str = "received/binding/flow";
const ON_FAILURE: &str = "received/binding/on-failure";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("inbox.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite_of(text: &str) -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(&ir_of(text)).suite
}

fn json_steps(suite: &ConformanceSuite, id: &str) -> Vec<serde_json::Value> {
    let document: serde_json::Value =
        serde_json::from_str(&suite.to_canonical_json().expect("the suite serialises"))
            .expect("JSON");
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("no scenario {id} in {document:#}"))
        .clone()
}

/// The issue's own shape, read through the suite's JSON: one event delivered under two
/// contexts, each invocation required to carry its own delivery's context.
#[test]
fn the_mapping_scenario_delivers_one_event_under_two_contexts() {
    let suite = suite_of(INBOX);
    let steps = json_steps(&suite, MAPPING);
    let delivered: Vec<&serde_json::Value> = steps
        .iter()
        .filter(|step| step["step"] == "deliver_event")
        .collect();
    assert_eq!(delivered.len(), 2, "{steps:#?}");
    let contexts: Vec<&serde_json::Value> = delivered
        .iter()
        .map(|step| &step["context"]["account_id"])
        .collect();
    assert_ne!(
        contexts[0], contexts[1],
        "two different contexts: {steps:#?}"
    );
    assert!(delivered.iter().all(|step| step["event"] == RECEIVED));
    assert!(delivered
        .iter()
        .all(|step| step["authority"] == "account-messages"));
    let expected: Vec<&serde_json::Value> = steps
        .iter()
        .filter(|step| step["step"] == "expect_invocation")
        .map(|step| &step["input"]["account_id"])
        .collect();
    assert_eq!(expected.len(), 2, "{steps:#?}");
    for (context, expected) in contexts.iter().zip(&expected) {
        assert_eq!(expected["kind"], "literal", "{steps:#?}");
        assert_eq!(
            expected["value"], **context,
            "each invocation carries its own delivery's context: {steps:#?}"
        );
    }
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/30"
    );
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Behaviour {
    /// Reads the context the event was delivered with, and redelivers an occurrence with its own.
    Correct,
    /// Fills `account_id` from something other than the delivery context.
    IgnoresContext,
    /// Redelivers an occurrence with the context the first delivery carried.
    FirstContextOnRedelivery,
    /// Cannot deliver an event with its context at all.
    NoDelivery,
}

#[derive(Default)]
struct State {
    deliveries: Vec<(BTreeMap<String, Node>, BTreeMap<String, Node>)>,
    forced: Option<String>,
    log: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
}

struct Inbox {
    behaviour: Behaviour,
    state: RefCell<State>,
}

impl Inbox {
    fn new(behaviour: Behaviour) -> Self {
        Self {
            behaviour,
            state: RefCell::new(State::default()),
        }
    }

    fn record(state: &mut State, input: &BTreeMap<String, Node>) -> SemanticCommandResult {
        let outcome = |name: &str| {
            OutcomeRef::new(
                CommandRef::new(RECORD.parse().unwrap()),
                name.parse().unwrap(),
            )
        };
        if state.forced.take().is_some() {
            let mut result = SemanticCommandResult::took(outcome("unavailable"));
            result.error = Some(DeclaredErrorValue::new(
                "demo.inbox.Unavailable".parse().unwrap(),
            ));
            return result;
        }
        let event = ObservedEvent::new(RECORDED.parse().unwrap())
            .with("account_id", input["account_id"].clone())
            .with("message_id", input["message_id"].clone());
        state.log.push(event.clone());
        let mut result = SemanticCommandResult::took(outcome("recorded"));
        result.direct_events.push(event);
        result
    }

    /// What the binding does with one occurrence: fill the input and invoke, retrying a failure.
    fn react(
        &self,
        state: &mut State,
        payload: &BTreeMap<String, Node>,
        context: &BTreeMap<String, Node>,
    ) {
        let account = match self.behaviour {
            Behaviour::IgnoresContext => Node::Text("account".into()),
            _ => context["account_id"].clone(),
        };
        let input: BTreeMap<String, Node> = [
            ("account_id".to_owned(), account),
            ("message_id".to_owned(), payload["message_id"].clone()),
            ("peer".to_owned(), payload["from"].clone()),
        ]
        .into_iter()
        .collect();
        for _ in 0..2 {
            let mut invocation = ObservedInvocation::new(
                BindingRef::new(ess_domain::binding::BindingName::new(BINDING).unwrap()),
                CommandRef::new(RECORD.parse().unwrap()),
            );
            invocation.input = input.clone();
            state.invocations.push(invocation);
            if Self::record(state, &input).error.is_none() {
                return;
            }
        }
    }
}

impl ConformanceTarget for Inbox {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("delivery-context", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.state.borrow_mut() = State::default();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        assert_eq!(request.command.to_string(), RECORD);
        Ok(Self::record(&mut self.state.borrow_mut(), &request.input))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .state
            .borrow()
            .log
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.state.borrow_mut().forced = Some(request.force.outcome.to_string());
        Ok(())
    }
    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        if self.behaviour == Behaviour::NoDelivery {
            return Err(TargetError::unsupported(
                format!(
                    "delivering `{}` from `{}`",
                    request.event, request.authority
                ),
                "this target has no external channel to deliver on",
            ));
        }
        assert_eq!(request.event.to_string(), RECEIVED);
        assert_eq!(request.authority, "account-messages");
        let mut state = self.state.borrow_mut();
        state
            .deliveries
            .push((request.payload.clone(), request.context.clone()));
        self.react(&mut state, &request.payload, &request.context);
        Ok(())
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        assert_eq!(request.event.to_string(), RECEIVED);
        let mut state = self.state.borrow_mut();
        let (payload, own) = state.deliveries.last().cloned().expect("delivered before");
        let context = match self.behaviour {
            Behaviour::FirstContextOnRedelivery => state.deliveries[0].1.clone(),
            _ => own,
        };
        self.react(&mut state, &payload, &context);
        Ok(())
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Ok(self
            .state
            .borrow()
            .invocations
            .iter()
            .filter(|invocation| {
                invocation.binding == request.binding && invocation.command == request.command
            })
            .cloned()
            .collect())
    }
}

fn run<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, (Status, String)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let checks = format!("{:?}", result.checks);
            (result.scenario.to_string(), (result.status, checks))
        })
        .collect()
}

fn status(statuses: &BTreeMap<String, (Status, String)>, id: &str) -> Status {
    statuses
        .get(id)
        .unwrap_or_else(|| panic!("no scenario {id}: {statuses:#?}"))
        .0
}

#[test]
fn a_target_that_follows_the_context_passes_every_binding_scenario() {
    let suite = suite_of(INBOX);
    let statuses = run(&suite, &Inbox::new(Behaviour::Correct));
    for id in [FLOW, MAPPING, DELIVERY, ON_FAILURE] {
        assert_eq!(status(&statuses, id), Status::Passed, "{id}: {statuses:#?}");
    }
}

#[test]
fn a_target_that_ignores_the_context_fails_the_mapping() {
    let suite = suite_of(INBOX);
    let statuses = run(&suite, &Inbox::new(Behaviour::IgnoresContext));
    assert_eq!(status(&statuses, MAPPING), Status::Failed, "{statuses:#?}");
}

#[test]
fn a_target_that_redelivers_with_the_first_context_fails_the_delivery() {
    let suite = suite_of(INBOX);
    let statuses = run(&suite, &Inbox::new(Behaviour::FirstContextOnRedelivery));
    assert_eq!(status(&statuses, MAPPING), Status::Passed, "{statuses:#?}");
    assert_eq!(status(&statuses, DELIVERY), Status::Failed, "{statuses:#?}");
}

#[test]
fn a_target_that_cannot_deliver_leaves_the_scenarios_unsupported_with_its_reason() {
    let suite = suite_of(INBOX);
    let statuses = run(&suite, &Inbox::new(Behaviour::NoDelivery));
    for id in [FLOW, MAPPING, DELIVERY, ON_FAILURE] {
        let (status, checks) = &statuses[id];
        assert_eq!(*status, Status::Unsupported, "{id}: {statuses:#?}");
        assert!(
            checks.contains("no external channel to deliver on"),
            "{id} carries the target's reason: {checks}"
        );
    }
}

#[test]
fn a_target_without_the_method_is_unsupported_too() {
    /// A target written before `deliver_event` existed.
    struct Older(Inbox);
    impl ConformanceTarget for Older {
        fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
            self.0.identity()
        }
        fn begin_scenario(&self, c: &ScenarioContext) -> Result<(), TargetError> {
            self.0.begin_scenario(c)
        }
        fn end_scenario(&self, c: &ScenarioContext) -> Result<(), TargetError> {
            self.0.end_scenario(c)
        }
        fn execute_command(
            &self,
            r: SemanticCommandRequest,
        ) -> Result<SemanticCommandResult, TargetError> {
            self.0.execute_command(r)
        }
        fn query_view(&self, r: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
            self.0.query_view(r)
        }
        fn observe_events(
            &self,
            r: EventObservationRequest,
        ) -> Result<Vec<ObservedEvent>, TargetError> {
            self.0.observe_events(r)
        }
        fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
            self.0.configure_external_outcome(r)
        }
        fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
            self.0.redeliver_event(r)
        }
    }
    let suite = suite_of(INBOX);
    let statuses = run(&suite, &Older(Inbox::new(Behaviour::Correct)));
    assert_eq!(
        status(&statuses, MAPPING),
        Status::Unsupported,
        "{statuses:#?}"
    );
}

#[test]
fn go_and_typescript_generation_refuse_a_suite_that_delivers_with_context() {
    let suite = suite_of(INBOX);
    let go = ess_conformance::go::emit(&suite).expect_err("Go refuses");
    assert!(go.to_string().contains("delivery context"), "{go}");
    let ts = ess_conformance::ts::emit(&suite).expect_err("TypeScript refuses");
    assert!(ts.to_string().contains("delivery context"), "{ts}");
}

#[test]
fn an_older_suite_envelope_refuses_the_delivery_step() {
    let suite = suite_of(INBOX);
    let json = suite
        .to_canonical_json()
        .unwrap()
        .replace("\"ess-conformance/30\"", "\"ess-conformance/28\"");
    let error = AdmittedSuite::from_json(&json).expect_err("suite/28 does not carry the step");
    assert!(error.to_string().contains("newer suite"), "{error}");
}

/// Redelivery carries the original context: two occurrences under two contexts, a redelivery,
/// and every invocation for the second occurrence required to carry the second context.
#[test]
fn the_delivery_scenario_redelivers_the_second_occurrence_and_asks_every_invocation() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(INBOX));
    let binding_refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("received"))
        .collect();
    assert!(
        binding_refusals.is_empty(),
        "every aspect of the binding is synthesized: {binding_refusals:#?}"
    );
    let steps = json_steps(&synthesis.suite, DELIVERY);
    let kinds: Vec<&str> = steps
        .iter()
        .map(|step| step["step"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(
        kinds,
        [
            "deliver_event",
            "deliver_event",
            "redeliver_event",
            "expect_every_invocation",
            "eventually_event"
        ],
        "{steps:#?}"
    );
    let second = &steps[1];
    let every = &steps[3];
    assert_ne!(steps[0]["context"], second["context"], "{steps:#?}");
    assert_eq!(
        every["input"]["account_id"]["value"], second["context"]["account_id"],
        "the redelivered occurrence carries its own context: {steps:#?}"
    );
    assert_eq!(
        every["selecting"]["message_id"]["value"], second["payload"]["message_id"],
        "{steps:#?}"
    );
    assert!(
        every["selecting"].get("account_id").is_none(),
        "the occurrence is selected by its payload, never by the context under test: {steps:#?}"
    );
}

/// An `at_most_once` binding has no redelivery to observe, whatever its context.
#[test]
fn an_at_most_once_delivery_context_binding_refuses_the_redelivery_only() {
    let text = INBOX.replace(
        "    delivery: at_least_once\n",
        "    delivery: at_most_once\n",
    );
    assert_ne!(text, INBOX);
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&text));
    assert!(synthesis
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == MAPPING));
    assert!(!synthesis
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == DELIVERY));
    assert!(synthesis
        .refusals
        .iter()
        .any(|refusal| refusal.to_string().contains("delivers at most once")));
}

/// The inbox with a `Boolean` context field `urgent` and the payload `Boolean` fields named in
/// `payload`, every one of them mapped.
fn with_booleans(payload: &[&str]) -> String {
    let mut text = INBOX.to_owned();
    let mut event = String::from("      - {name: from, type: String}\n");
    let mut input = String::from("      - {name: peer, type: String}\n");
    let mut mapping = String::from("      peer: event.from\n");
    input.push_str("      - {name: urgent, type: Boolean}\n");
    mapping.push_str("      urgent: context.urgent\n");
    for field in payload {
        let declared = format!("      - {{name: {field}, type: Boolean}}\n");
        event.push_str(&declared);
        input.push_str(&declared);
        mapping.push_str("      ");
        mapping.push_str(field);
        mapping.push_str(": event.");
        mapping.push_str(field);
        mapping.push('\n');
    }
    for (from, to) in [
        ("      - {name: from, type: String}\n", event),
        ("      - {name: peer, type: String}\n", input),
        ("      peer: event.from\n", mapping),
        (
            "        - {name: account_id, type: demo.inbox.AccountId}\n",
            "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: urgent, type: Boolean}\n".to_owned(),
        ),
    ] {
        assert_eq!(text.matches(from).count(), 1, "`{from}` once in the model");
        text = text.replacen(from, &to, 1);
    }
    text
}

/// A `Boolean` context field beside one `Boolean` payload field: the type has two values, which
/// is enough to keep the two apart in each delivery and to change both between the deliveries.
#[test]
fn a_boolean_context_field_differs_from_a_boolean_payload_field_in_every_delivery() {
    let suite = suite_of(&with_booleans(&["flagged"]));
    let steps = json_steps(&suite, MAPPING);
    let delivered: Vec<&serde_json::Value> = steps
        .iter()
        .filter(|step| step["step"] == "deliver_event")
        .collect();
    assert_eq!(delivered.len(), 2, "{steps:#?}");
    for step in &delivered {
        assert_ne!(
            step["context"]["urgent"], step["payload"]["flagged"],
            "a target reading the payload in place of the context is caught: {steps:#?}"
        );
    }
    assert_ne!(
        delivered[0]["context"]["urgent"], delivered[1]["context"]["urgent"],
        "{steps:#?}"
    );
}

/// Two `Boolean` payload fields beside a `Boolean` context field: only the context needs keeping
/// apart, so both payload fields take the value the context does not, and the context scenarios
/// are synthesized.
#[test]
fn two_boolean_payload_fields_leave_one_value_for_the_context() {
    let suite = suite_of(&with_booleans(&["flagged", "pinned"]));
    let steps = json_steps(&suite, MAPPING);
    let delivered: Vec<&serde_json::Value> = steps
        .iter()
        .filter(|step| step["step"] == "deliver_event")
        .collect();
    assert_eq!(delivered.len(), 2, "{steps:#?}");
    for step in &delivered {
        for payload in ["flagged", "pinned"] {
            assert_ne!(
                step["context"]["urgent"], step["payload"][payload],
                "{payload}: {steps:#?}"
            );
        }
    }
    assert_ne!(
        delivered[0]["context"]["urgent"], delivered[1]["context"]["urgent"],
        "{steps:#?}"
    );
    assert!(suite.scenarios.keys().any(|id| id.to_string() == DELIVERY));
}

/// Two mapped `Boolean` context fields beside a `Boolean` payload field: three values would be
/// needed in each delivery and the type has two. The scenarios that check the context are refused
/// with that reason rather than synthesized with two equal values.
#[test]
fn too_many_boolean_fields_refuse_the_context_scenarios_with_the_reason() {
    let mut text = with_booleans(&["flagged"]);
    for (from, to) in [
        (
            "        - {name: urgent, type: Boolean}\n",
            "        - {name: urgent, type: Boolean}\n        - {name: muted, type: Boolean}\n",
        ),
        (
            "      urgent: context.urgent\n",
            "      urgent: context.urgent\n      muted: context.muted\n",
        ),
        (
            "{name: peer, type: String}\n      - {name: urgent, type: Boolean}\n",
            "{name: peer, type: String}\n      - {name: urgent, type: Boolean}\n      - {name: muted, type: Boolean}\n",
        ),
    ] {
        assert_eq!(text.matches(from).count(), 1, "`{from}` once in the model");
        text = text.replacen(from, to, 1);
    }
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&text));
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(!ids.iter().any(|id| id == MAPPING), "{ids:#?}");
    assert!(!ids.iter().any(|id| id == DELIVERY), "{ids:#?}");
    assert!(ids.iter().any(|id| id == FLOW), "{ids:#?}");
    let refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("too few witness values"))
        .collect();
    assert_eq!(refusals.len(), 2, "{:#?}", synthesis.refusals);
    assert!(
        refusals.iter().all(|refusal| refusal.contains("`urgent`")),
        "{refusals:#?}"
    );
}

/// A context field whose newtype admits one value (`Level`, exactly 3): the two deliveries cannot
/// differ in it, so `mapping` and `delivery` are refused naming that one value and the count the
/// invariants admit, and `flow` and `on-failure`, which deliver once, are still synthesized.
#[test]
fn a_context_field_with_one_admitted_value_refuses_only_the_two_delivery_scenarios() {
    let mut text = INBOX.to_owned();
    for (from, to) in [
        (
            "  - {name: demo.inbox.AccountId, kind: newtype, of: String}\n",
            "  - {name: demo.inbox.AccountId, kind: newtype, of: String}\n  - {name: demo.inbox.Level, kind: newtype, of: Integer, invariants: [value >= 3, value <= 3]}\n",
        ),
        (
            "        - {name: account_id, type: demo.inbox.AccountId}\n",
            "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: level, type: demo.inbox.Level}\n",
        ),
        (
            "      - {name: peer, type: String}\n",
            "      - {name: peer, type: String}\n      - {name: level, type: demo.inbox.Level}\n",
        ),
        (
            "      peer: event.from\n",
            "      peer: event.from\n      level: context.level\n",
        ),
    ] {
        assert_eq!(text.matches(from).count(), 1, "`{from}` once in the model");
        text = text.replacen(from, to, 1);
    }
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&text));
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(!ids.iter().any(|id| id == MAPPING), "{ids:#?}");
    assert!(!ids.iter().any(|id| id == DELIVERY), "{ids:#?}");
    assert!(ids.iter().any(|id| id == FLOW), "{ids:#?}");
    assert!(ids.iter().any(|id| id == ON_FAILURE), "{ids:#?}");
    let refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("`level`"))
        .collect();
    assert_eq!(refusals.len(), 2, "{:#?}", synthesis.refusals);
    assert!(
        refusals
            .iter()
            .all(|refusal| refusal.contains("`3` in both deliveries")
                && refusal.contains("admit 1 of the first")),
        "{refusals:#?}"
    );
}
