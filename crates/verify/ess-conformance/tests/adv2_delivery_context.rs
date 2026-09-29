//! Adversary pass 2 on beyond10x/ess#195, after correction 1: the per-slot distinctions of
//! `synthesize/delivery_context.rs` `occurrences()` against enums, nested payload values and
//! newtype invariants.
//!
//! The harness is pass 1's: a target whose binding fills the command input from the delivered
//! payload and context through a `fill` function.
#![allow(clippy::items_after_statements)]

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{BindingRef, CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::Synthesis;
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

fn synthesis_of(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("inbox.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).expect("the model compiles");
    ess_conformance::synthesize::synthesize(&ir)
}

#[derive(Default)]
struct State {
    deliveries: Vec<(Fields, Fields)>,
    forced: bool,
    log: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
}

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
        Ok(ImplementationIdentity::new("adv2-delivery-context", "1"))
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

/// Every one of `wanted` is synthesized and the control passes it.
fn assert_synthesized_and_passed(synthesis: &Synthesis, correct: Fill, wanted: &[&str]) {
    let control = statuses(&synthesis.suite, correct);
    for id in wanted {
        assert_eq!(
            control.get(*id),
            Some(&Status::Passed),
            "`{id}` is synthesized and the control passes it: {control:#?}\nrefusals: {:#?}",
            synthesis.refusals
        );
    }
}

fn fields(pairs: &[(&str, &Node)]) -> Fields {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).clone()))
        .collect()
}

/// A three-variant enum context field has three values, so two deliveries can carry two of them.
/// With four type groups the per-slot distinctions of `occurrences()` are 6 and 15 for it, both
/// `0 mod 3`, so both deliveries carry `Low` and `separated()` refuses `mapping` and `delivery`
/// with "has one witness value" — a claim the type contradicts.
#[test]
fn adv2_a_three_variant_enum_context_field_keeps_its_mapping_scenario() {
    let mut text = edit(
        INBOX.to_owned(),
        "  - {name: demo.inbox.AccountId, kind: newtype, of: String}\n",
        "  - {name: demo.inbox.AccountId, kind: newtype, of: String}\n  - {name: demo.inbox.Priority, kind: enum, variants: [Low, Medium, High]}\n",
    );
    text = edit(
        text,
        "      - {name: from, type: String}\n",
        "      - {name: from, type: String}\n      - {name: amount, type: Integer}\n",
    );
    text = edit(
        text,
        "        - {name: account_id, type: demo.inbox.AccountId}\n",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: priority, type: demo.inbox.Priority}\n",
    );
    text = edit(
        text,
        "      - {name: peer, type: String}\n",
        "      - {name: peer, type: String}\n      - {name: priority, type: demo.inbox.Priority}\n",
    );
    text = edit(
        text,
        "      peer: event.from\n",
        "      peer: event.from\n      priority: context.priority\n",
    );
    fn correct(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("priority", &context["priority"]),
        ])
    }
    let synthesis = synthesis_of(&text);
    assert_synthesized_and_passed(
        &synthesis,
        correct,
        &["received/binding/mapping", "received/binding/delivery"],
    );
}

/// A `Boolean` context field beside a payload struct holding a `Boolean` of the same name. Each
/// type is its own group, and every group base is even, so the nested payload flag and the
/// context flag carry one value in the first delivery and one value in the second. `separated()`
/// compares top-level values only (a map against a bool), so nothing is refused, and a target that
/// reads the flag from the payload passes every scenario. The issue: "never a lookup in the
/// payload".
#[test]
fn adv2_a_target_that_reads_a_nested_payload_flag_in_place_of_the_context_fails() {
    let mut text = edit(
        INBOX.to_owned(),
        "  - {name: demo.inbox.AccountId, kind: newtype, of: String}\n",
        "  - {name: demo.inbox.AccountId, kind: newtype, of: String}\n  - name: demo.inbox.Flags\n    kind: struct\n    fields:\n      - {name: urgent, type: Boolean}\n",
    );
    text = edit(
        text,
        "      - {name: from, type: String}\n",
        "      - {name: from, type: String}\n      - {name: flags, type: demo.inbox.Flags}\n",
    );
    text = edit(
        text,
        "        - {name: account_id, type: demo.inbox.AccountId}\n",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: urgent, type: Boolean}\n",
    );
    text = edit(
        text,
        "      - {name: peer, type: String}\n",
        "      - {name: peer, type: String}\n      - {name: urgent, type: Boolean}\n",
    );
    text = edit(
        text,
        "      peer: event.from\n",
        "      peer: event.from\n      urgent: context.urgent\n",
    );
    fn correct(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("urgent", &context["urgent"]),
        ])
    }
    fn from_payload(payload: &Fields, context: &Fields) -> Fields {
        let Node::Map(flags) = &payload["flags"] else {
            panic!("flags is a struct: {:?}", payload["flags"]);
        };
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("urgent", &flags["urgent"]),
        ])
    }
    let synthesis = synthesis_of(&text);
    assert_synthesized_and_passed(&synthesis, correct, &SCENARIOS);
    let mutated = statuses(&synthesis.suite, from_payload);
    assert!(
        mutated.values().any(|status| *status == Status::Failed),
        "mutant `urgent read from event.flags.urgent` passes every delivery-context scenario: \
         {mutated:#?}"
    );
}

/// A context field of a bounded newtype (`Rating`, 1 to 5). Two deliveries need two ratings, and
/// the type has five. `occurrences()` numbers it 4 and 11, whose `Integer` witnesses are 5 and 12;
/// 12 breaks the invariant, `witness::fields` returns a gap, and the whole binding is refused —
/// `flow` and `on-failure` included, which need no second value at all.
#[test]
fn adv2_a_bounded_integer_context_field_keeps_its_flow_scenario() {
    let mut text = edit(
        INBOX.to_owned(),
        "  - {name: demo.inbox.AccountId, kind: newtype, of: String}\n",
        "  - {name: demo.inbox.AccountId, kind: newtype, of: String}\n  - {name: demo.inbox.Rating, kind: newtype, of: Integer, invariants: [value >= 1, value <= 5]}\n",
    );
    text = edit(
        text,
        "        - {name: account_id, type: demo.inbox.AccountId}\n",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: rating, type: demo.inbox.Rating}\n",
    );
    text = edit(
        text,
        "      - {name: peer, type: String}\n",
        "      - {name: peer, type: String}\n      - {name: rating, type: demo.inbox.Rating}\n",
    );
    text = edit(
        text,
        "      peer: event.from\n",
        "      peer: event.from\n      rating: context.rating\n",
    );
    fn correct(payload: &Fields, context: &Fields) -> Fields {
        fields(&[
            ("account_id", &context["account_id"]),
            ("message_id", &payload["message_id"]),
            ("peer", &payload["from"]),
            ("rating", &context["rating"]),
        ])
    }
    let synthesis = synthesis_of(&text);
    assert_synthesized_and_passed(&synthesis, correct, &SCENARIOS);
}
