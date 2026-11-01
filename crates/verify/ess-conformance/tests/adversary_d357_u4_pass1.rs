//! Adversary pass 1 against `story:binding-site-identity-comparisons` (wave D357, unit U4).
//!
//! The unit compares the identities of a binding trigger's event, but where the binding's
//! destination is arranged before the trigger (the trigger's own arrangement extended, or its
//! input rewritten to name the arranged row), `expect_trigger_event` reads only the input the
//! trigger is sent: the identity its subject holds (`{subject: <identity>}`) and a stored identity
//! of that subject (`{subject: <field>}`) are left uncompared, though the scenario captured both
//! instances. The trigger's own command scenario compares the same field, so the identity is one
//! the suite determines; only the binding scenarios lose it.

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;
use ess_primitives::node::Node;

fn one(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    let raw = RawSpecFile::parse(text).expect("the model parses");
    sources.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble(vec![(SpecSource::new("model.yaml"), raw)])
        .expect("the model assembles");
    compile(&spec, &sources).expect("the model compiles")
}

fn synthesized(ir: &EssIr) -> ConformanceSuite {
    catch_unwind(AssertUnwindSafe(|| {
        ess_conformance::synthesize::synthesize(ir).suite
    }))
    .expect("synthesis does not panic")
}

fn statuses<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// The interpreter publishing another identity in one field of one event; the binding is still
/// delivered the event the interpreter made itself.
struct Mutated {
    inner: Interpreted,
    event: String,
    field: String,
}

impl ConformanceTarget for Mutated {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut result = self.inner.execute_command(request)?;
        for event in &mut result.direct_events {
            if event.event.to_string() == self.event {
                event.payload.insert(
                    self.field.clone(),
                    Node::Text("00000000-0000-4000-8000-0000000000ff".into()),
                );
            }
        }
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

/// Every scenario under `prefix` that expects `event`, with what its expectation compares.
fn expecting(
    suite: &ConformanceSuite,
    prefix: &str,
    event: &str,
) -> BTreeMap<String, BTreeMap<String, ScenarioValue>> {
    suite
        .scenarios
        .iter()
        .filter(|(id, _)| id.to_string().contains(prefix))
        .filter_map(|(id, scenario)| {
            scenario.steps.iter().find_map(|step| match step {
                ScenarioStep::ExpectEvent { event: e, .. } if e.to_string() == event => {
                    Some((id.to_string(), BTreeMap::new()))
                }
                ScenarioStep::ExpectEventValues {
                    event: e, payload, ..
                } if e.to_string() == event => Some((id.to_string(), payload.clone())),
                _ => None,
            })
        })
        .collect()
}

/// Asserts the field is compared on every binding scenario, and that a target publishing another
/// identity in it fails each, while the reference passes each.
fn assert_compared(model: &str, binding: &str, event: &str, field: &str, command: &str) {
    let ir = one(model);
    let suite = synthesized(&ir);
    let ids: Vec<String> = suite.scenarios.keys().map(ToString::to_string).collect();

    // The trigger's own command scenario compares the field: the suite determines it.
    let own = expecting(&suite, command, event);
    assert!(
        own.values().any(|payload| payload.contains_key(field)),
        "the trigger's own scenario compares `{event}.{field}`: {own:#?}\n{ids:#?}"
    );

    // Flow, mapping and delivery (on-failure is refused for this model on its own grounds).
    let prefix = format!("{binding}/binding/");
    let bound = expecting(&suite, &prefix, event);
    assert!(
        bound.len() >= 3,
        "flow, mapping and delivery of `{binding}` expect `{event}`: {bound:#?}\n{ids:#?}"
    );

    let good = statuses(&suite, &Interpreted::for_model(ir.clone()));
    for id in bound.keys() {
        assert_eq!(good.get(id), Some(&Status::Passed), "{id}: {good:#?}");
    }

    let uncompared: Vec<_> = bound
        .iter()
        .filter(|(_, payload)| !matches!(payload.get(field), Some(ScenarioValue::Instance { .. })))
        .map(|(id, payload)| format!("{id}: {payload:?}"))
        .collect();

    let wrong = Mutated {
        inner: Interpreted::for_model(ir.clone()),
        event: event.into(),
        field: field.into(),
    };
    let ran = statuses(&suite, &wrong);
    let survivors: Vec<_> = bound
        .keys()
        .filter(|id| ran.get(*id) != Some(&Status::Failed))
        .map(|id| format!("{id}: {:?}", ran.get(id)))
        .collect();
    assert!(
        uncompared.is_empty() && survivors.is_empty(),
        "binding scenarios of `{binding}` leave `{event}.{field}`, an identity the scenario \
         captured, uncompared (story:binding-site-identity-comparisons)\n\
         uncompared: {uncompared:#?}\npassing a wrong `{field}`: {survivors:#?}"
    );
}

/// `CloseTicket` acts on an existing ticket and names an agent in its input; `closed-engages`
/// engages that agent, so the agent is arranged before the trigger and the trigger is sent naming
/// it. `TicketClosed.ticket_id` is the subject's own identity (`{subject: ticket_id}`).
const SUBJECT_IDENTITY: &str = r"format: ess/18
system: desk
version: v1
domain: desk.work
types:
  - {name: desk.work.TicketId, kind: newtype, of: Uuid}
  - {name: desk.work.AgentId, kind: newtype, of: Uuid}
entities:
  - name: desk.work.Agent
    identity: {name: agent_id, type: desk.work.AgentId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Idle
      states: [Idle, Busy]
      terminal: [Busy]
      transitions:
        - {name: engage, from: [Idle], to: Busy}
  - name: desk.work.Ticket
    identity: {name: ticket_id, type: desk.work.TicketId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - name: desk.work.Conflict
    fields: []
  - name: desk.work.Missing
    fields: []
  - name: desk.work.Busy
    fields: []
events:
  - name: desk.work.AgentHired
    fields: [{name: agent_id, type: desk.work.AgentId}]
  - name: desk.work.TicketOpened
    fields: [{name: ticket_id, type: desk.work.TicketId}]
  - name: desk.work.TicketClosed
    fields:
      - {name: ticket_id, type: desk.work.TicketId}
      - {name: agent_id, type: desk.work.AgentId}
  - name: desk.work.AgentEngaged
    fields: [{name: agent_id, type: desk.work.AgentId}]
commands:
  - name: desk.work.HireAgent
    input:
      - {name: agent_id, type: desk.work.AgentId}
      - {name: label, type: String}
    outcomes:
      - name: hired
        creates: desk.work.Agent
        instance: agent_id
        sets: {label: input.label}
        emits: [desk.work.AgentHired]
        payload: {desk.work.AgentHired: {agent_id: input.agent_id}}
  - name: desk.work.OpenTicket
    input:
      - {name: ticket_id, type: desk.work.TicketId}
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: desk.work.Ticket
        instance: ticket_id
        sets: {label: input.label}
        emits: [desk.work.TicketOpened]
        payload: {desk.work.TicketOpened: {ticket_id: input.ticket_id}}
  - name: desk.work.CloseTicket
    input:
      - {name: ticket_id, type: desk.work.TicketId}
      - {name: agent_id, type: desk.work.AgentId}
    outcomes:
      - name: closed
        moves: desk.work.Ticket.close
        instance: ticket_id
        emits: [desk.work.TicketClosed]
        payload:
          desk.work.TicketClosed:
            ticket_id: {subject: ticket_id}
            agent_id: input.agent_id
      - {name: wrong-state, wrong_state: true, error: desk.work.Conflict}
      - {name: not-found, unknown_instance: true, error: desk.work.Missing}
  - name: desk.work.EngageAgent
    input:
      - {name: agent_id, type: desk.work.AgentId}
    outcomes:
      - name: engaged
        moves: desk.work.Agent.engage
        instance: agent_id
        emits: [desk.work.AgentEngaged]
        payload: {desk.work.AgentEngaged: {agent_id: input.agent_id}}
      - name: unavailable
        external: the agent cannot be reached right now
        error: desk.work.Busy
      - {name: wrong-state, wrong_state: true, error: desk.work.Conflict}
      - {name: not-found, unknown_instance: true, error: desk.work.Missing}
bindings:
  - id: closed-engages
    when: {event: desk.work.TicketClosed}
    invoke: {command: desk.work.EngageAgent}
    mapping: {agent_id: event.agent_id}
    delivery: at_least_once
    on_failure: drop
";

/// The subject's own identity in the trigger's event, on a binding whose destination is arranged
/// before the trigger and named in the trigger's input.
#[test]
fn a_trigger_events_subject_identity_is_compared_where_the_destination_is_arranged() {
    assert_compared(
        SUBJECT_IDENTITY,
        "closed-engages",
        "desk.work.TicketClosed",
        "ticket_id",
        "desk.work.CloseTicket/",
    );
}

/// `OpenTicket` stores the agent a ticket is assigned to; `CloseTicket` publishes it from the row
/// (`{subject: agent_id}`), and `closed-engages` engages that agent. The destination is the stored
/// identity, arranged first and stored by the trigger's own arrangement.
const STORED_IDENTITY: &str = r"format: ess/18
system: desk
version: v1
domain: desk.work
types:
  - {name: desk.work.TicketId, kind: newtype, of: Uuid}
  - {name: desk.work.AgentId, kind: newtype, of: Uuid}
entities:
  - name: desk.work.Agent
    identity: {name: agent_id, type: desk.work.AgentId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Idle
      states: [Idle, Busy]
      terminal: [Busy]
      transitions:
        - {name: engage, from: [Idle], to: Busy}
  - name: desk.work.Ticket
    identity: {name: ticket_id, type: desk.work.TicketId}
    fields:
      - {name: agent_id, type: desk.work.AgentId}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - name: desk.work.Conflict
    fields: []
  - name: desk.work.Missing
    fields: []
  - name: desk.work.Busy
    fields: []
events:
  - name: desk.work.AgentHired
    fields: [{name: agent_id, type: desk.work.AgentId}]
  - name: desk.work.TicketOpened
    fields: [{name: ticket_id, type: desk.work.TicketId}]
  - name: desk.work.TicketClosed
    fields:
      - {name: ticket_id, type: desk.work.TicketId}
      - {name: agent_id, type: desk.work.AgentId}
  - name: desk.work.AgentEngaged
    fields: [{name: agent_id, type: desk.work.AgentId}]
commands:
  - name: desk.work.HireAgent
    input:
      - {name: agent_id, type: desk.work.AgentId}
      - {name: label, type: String}
    outcomes:
      - name: hired
        creates: desk.work.Agent
        instance: agent_id
        sets: {label: input.label}
        emits: [desk.work.AgentHired]
        payload: {desk.work.AgentHired: {agent_id: input.agent_id}}
  - name: desk.work.OpenTicket
    input:
      - {name: ticket_id, type: desk.work.TicketId}
      - {name: agent_id, type: desk.work.AgentId}
    outcomes:
      - name: opened
        creates: desk.work.Ticket
        instance: ticket_id
        sets: {agent_id: input.agent_id}
        emits: [desk.work.TicketOpened]
        payload: {desk.work.TicketOpened: {ticket_id: input.ticket_id}}
  - name: desk.work.CloseTicket
    input:
      - {name: ticket_id, type: desk.work.TicketId}
    outcomes:
      - name: closed
        moves: desk.work.Ticket.close
        instance: ticket_id
        emits: [desk.work.TicketClosed]
        payload:
          desk.work.TicketClosed:
            ticket_id: input.ticket_id
            agent_id: {subject: agent_id}
      - {name: wrong-state, wrong_state: true, error: desk.work.Conflict}
      - {name: not-found, unknown_instance: true, error: desk.work.Missing}
  - name: desk.work.EngageAgent
    input:
      - {name: agent_id, type: desk.work.AgentId}
    outcomes:
      - name: engaged
        moves: desk.work.Agent.engage
        instance: agent_id
        emits: [desk.work.AgentEngaged]
        payload: {desk.work.AgentEngaged: {agent_id: input.agent_id}}
      - name: unavailable
        external: the agent cannot be reached right now
        error: desk.work.Busy
      - {name: wrong-state, wrong_state: true, error: desk.work.Conflict}
      - {name: not-found, unknown_instance: true, error: desk.work.Missing}
bindings:
  - id: closed-engages
    when: {event: desk.work.TicketClosed}
    invoke: {command: desk.work.EngageAgent}
    mapping: {agent_id: event.agent_id}
    delivery: at_least_once
    on_failure: drop
";

/// A stored identity of the trigger's subject in the trigger's event, which is the binding's
/// destination: the row the scenario arranged and captured.
#[test]
fn a_trigger_events_stored_identity_is_compared_where_the_destination_is_arranged() {
    assert_compared(
        STORED_IDENTITY,
        "closed-engages",
        "desk.work.TicketClosed",
        "agent_id",
        "desk.work.CloseTicket/",
    );
}
