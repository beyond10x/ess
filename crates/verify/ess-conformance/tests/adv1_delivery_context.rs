//! Adversary pass 1 on beyond10x/ess#195: mutants a correct-looking target could ship that the
//! delivery-context scenarios must fail.
//!
//! Each test first runs a target that follows the declared mapping (the control: every binding
//! scenario passes), then the mutant, and requires that at least one of the binding's scenarios
//! fails for the mutant. A green control and a green mutant mean the suite cannot tell the two
//! apart.
// Each case's `correct` and mutant fills read best beside the model edit they serve.
#![allow(clippy::items_after_statements)]

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
const RECORDED: &str = "demo.inbox.MessageRecorded";
const SCENARIOS: [&str; 4] = [
    "received/binding/flow",
    "received/binding/mapping",
    "received/binding/delivery",
    "received/binding/on-failure",
];

type Fields = BTreeMap<String, Node>;
type Fill = fn(&Fields, &Fields) -> Fields;

fn edit(mut text: String, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` once in the model");
    text = text.replacen(from, to, 1);
    text
}

fn suite_of(text: &str) -> ConformanceSuite {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("inbox.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).expect("the model compiles");
    ess_conformance::synthesize::synthesize(&ir).suite
}

#[derive(Default)]
struct State {
    deliveries: Vec<(Fields, Fields)>,
    forced: bool,
    log: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
}

/// A target whose binding fills the command input with `fill(payload, context)`.
struct Inbox {
    fill: Fill,
    state: RefCell<State>,
}

impl Inbox {
    fn new(fill: Fill) -> Self {
        Self {
            fill,
            state: RefCell::new(State::default()),
        }
    }

    fn record(state: &mut State, input: &Fields) -> SemanticCommandResult {
        let outcome = |name: &str| {
            OutcomeRef::new(
                CommandRef::new(RECORD.parse().unwrap()),
                name.parse().unwrap(),
            )
        };
        if std::mem::take(&mut state.forced) {
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

    fn react(&self, state: &mut State, payload: &Fields, context: &Fields) {
        let input = (self.fill)(payload, context);
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
        Ok(ImplementationIdentity::new("adv1-delivery-context", "1"))
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
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.state.borrow_mut().forced = true;
        Ok(())
    }
    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        let mut state = self.state.borrow_mut();
        state
            .deliveries
            .push((request.payload.clone(), request.context.clone()));
        self.react(&mut state, &request.payload, &request.context);
        Ok(())
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        let mut state = self.state.borrow_mut();
        let (payload, context) = state.deliveries.last().cloned().expect("delivered before");
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

fn statuses(suite: &ConformanceSuite, fill: Fill) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Inbox::new(fill))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .filter(|(id, _)| SCENARIOS.contains(&id.as_str()))
        .collect()
}

/// The control passes every synthesized binding scenario, and the mutant fails at least one.
fn assert_caught(text: &str, correct: Fill, mutant: Fill, mutant_name: &str) {
    let suite = suite_of(text);
    let control = statuses(&suite, correct);
    assert!(
        control.contains_key("received/binding/mapping"),
        "the mapping scenario is synthesized: {control:#?}"
    );
    assert!(
        control.values().all(|status| *status == Status::Passed),
        "the control follows the declared mapping and passes: {control:#?}"
    );
    let mutated = statuses(&suite, mutant);
    assert!(
        mutated.values().any(|status| *status == Status::Failed),
        "mutant `{mutant_name}` passes every delivery-context scenario: {mutated:#?}"
    );
}

fn fields(pairs: &[(&str, &Node)]) -> Fields {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).clone()))
        .collect()
}

/// A payload field with the same name as the context field (the sender's account beside the
/// subscription's). The issue: "A missing context is a refusal, never a lookup in the payload."
/// A target that reads the payload's `account_id` in place of the context's must fail.
#[test]
fn adv1_a_target_that_reads_the_same_named_payload_field_fails() {
    let text = edit(
        INBOX.to_owned(),
        "      - {name: from, type: String}\n",
        "      - {name: from, type: String}\n      - {name: account_id, type: String}\n",
    );
    fn correct(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
        ])
    }
    fn from_payload(payload: &Fields, _: &Fields) -> Fields {
        fields(&[
            ("account_id", &payload["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
        ])
    }
    assert_caught(
        &text,
        correct,
        from_payload,
        "payload lookup by the context field's name",
    );
}

/// Two context fields; the target reads one correctly and fills the other (`shard`, an
/// `Integer`) from a payload field of the same type (`sequence`). "Context ignored for one field
/// of two."
#[test]
fn adv1_a_target_that_fills_an_integer_context_input_from_the_payload_fails() {
    let mut text = edit(
        INBOX.to_owned(),
        "      - {name: from, type: String}\n",
        "      - {name: from, type: String}\n      - {name: sequence, type: Integer}\n",
    );
    text = edit(
        text,
        "        - {name: account_id, type: demo.inbox.AccountId}\n",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: shard, type: Integer}\n",
    );
    text = edit(
        text,
        "      - {name: peer, type: String}\n",
        "      - {name: peer, type: String}\n      - {name: shard, type: Integer}\n      - {name: sequence, type: Integer}\n",
    );
    text = edit(
        text,
        "      peer: event.from\n",
        "      peer: event.from\n      shard: context.shard\n      sequence: event.sequence\n",
    );
    fn correct(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("shard", &context["shard"]),
            ("sequence", &payload["sequence"]),
        ])
    }
    fn shard_from_payload(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("shard", &payload["sequence"]),
            ("sequence", &payload["sequence"]),
        ])
    }
    assert_caught(
        &text,
        correct,
        shard_from_payload,
        "shard read from event.sequence",
    );
}

/// Two context fields of one type mapped to two inputs; the target swaps them. "Context mapped
/// to the wrong field of the same type."
#[test]
fn adv1_a_target_that_swaps_two_integer_context_fields_fails() {
    let mut text = edit(
        INBOX.to_owned(),
        "        - {name: account_id, type: demo.inbox.AccountId}\n",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: region, type: Integer}\n        - {name: shard, type: Integer}\n",
    );
    text = edit(
        text,
        "      - {name: peer, type: String}\n",
        "      - {name: peer, type: String}\n      - {name: region, type: Integer}\n      - {name: shard, type: Integer}\n",
    );
    text = edit(
        text,
        "      peer: event.from\n",
        "      peer: event.from\n      region: context.region\n      shard: context.shard\n",
    );
    fn correct(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("region", &context["region"]),
            ("shard", &context["shard"]),
        ])
    }
    fn swapped(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("region", &context["shard"]),
            ("shard", &context["region"]),
        ])
    }
    assert_caught(&text, correct, swapped, "region and shard swapped");
}

/// Control for the harness: two `String` context fields swapped is caught, because a `String`
/// witness is its own path.
#[test]
fn adv1_control_swapping_two_string_context_fields_is_caught() {
    let mut text = edit(
        INBOX.to_owned(),
        "        - {name: account_id, type: demo.inbox.AccountId}\n",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: region, type: String}\n        - {name: shard, type: String}\n",
    );
    text = edit(
        text,
        "      - {name: peer, type: String}\n",
        "      - {name: peer, type: String}\n      - {name: region, type: String}\n      - {name: shard, type: String}\n",
    );
    text = edit(
        text,
        "      peer: event.from\n",
        "      peer: event.from\n      region: context.region\n      shard: context.shard\n",
    );
    fn correct(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("region", &context["region"]),
            ("shard", &context["shard"]),
        ])
    }
    fn swapped(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("region", &context["shard"]),
            ("shard", &context["region"]),
        ])
    }
    assert_caught(&text, correct, swapped, "string region and shard swapped");
}
