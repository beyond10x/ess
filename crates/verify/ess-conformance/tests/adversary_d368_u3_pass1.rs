//! Adversary pass 1 against story:a-refusal-witness-compares-its-trigger-identities (wave D368,
//! unit U3).
//!
//! What the unit's own tests do not say:
//!
//! * its wrong-id test asks that *one* witness per binding fail; the Acceptance asks that every
//!   `<binding>/binding/refusal/<outcome>` scenario compare the identity fields of its trigger's
//!   event, so here every witness, and every identity field of that event, is mutated on its own;
//! * its fixtures only reach the answered (`exists: false`) and arranged-row (predicate) paths of
//!   `refusal_policy::witness`. Three models of this file reach the others: a refusal forced by
//!   injection (drop, and a bounded retry with a final refusal) on an event carrying two
//!   identities of two real rows; a publisher swapped by `bounded_retry::clean_publisher` that
//!   copies its identity from an input under another name; and a publisher swapped by
//!   `unguarded_publisher` where the first publisher refuses the missing row itself.
//! * ported from the security pass of the same unit: a witness that sends its trigger naming a row
//!   no row carries (`related_missing`) compares the id it sent, and a target publishing the id of
//!   another record the scenario holds in the trigger's event fails the witness.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source as SpecSource;
use ess_domain::Specification;
use ess_primitives::node::Node;

// ---- models ------------------------------------------------------------------------------------

fn assemble(files: &[(String, String)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.insert(label.clone(), text.clone());
        parsed.push((SpecSource::new(label.clone()), raw));
    }
    let spec =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("assembles:\n{errors}"));
    compile(&spec, &sources).unwrap_or_else(|diagnostics| panic!("compiles:\n{diagnostics}"))
}

fn directory(dir: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(dir);
    let mut found = Vec::new();
    let mut pending = vec![base];
    while let Some(next) = pending.pop() {
        for entry in std::fs::read_dir(&next).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let files: Vec<(String, String)> = found
        .into_iter()
        .map(|path| {
            (
                path.display().to_string(),
                std::fs::read_to_string(&path).unwrap(),
            )
        })
        .collect();
    assemble(&files)
}

fn system(domain: &str, components: &str, model: &str) -> EssIr {
    let header = format!("format: ess/32\nsystem: demo\nversion: v1\ndomains:\n  - {domain}\n");
    assemble(&[
        ("system.yaml".into(), header),
        ("components.yaml".into(), components.into()),
        ("domain.yaml".into(), model.into()),
    ])
}

fn synthesized(ir: &EssIr) -> (ConformanceSuite, String) {
    let synthesis = ess_conformance::synthesize::synthesize(ir);
    let refusals = format!("{:#?}", synthesis.refusals);
    (synthesis.suite, refusals)
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

fn witnesses(suite: &ConformanceSuite, binding: &str) -> Vec<String> {
    let prefix = format!("{binding}/binding/refusal/");
    suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.starts_with(&prefix))
        .collect()
}

/// The expectation of `event` the scenario makes after its last command, and that command.
fn trigger_expectation(suite: &ConformanceSuite, id: &str, event: &str) -> String {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
        .unwrap();
    let mut sent = String::new();
    let mut found = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                sent = format!("{command} {input:?}");
            }
            ScenarioStep::ExpectEvent {
                event: expected,
                payload,
                ..
            } if expected.to_string() == event => {
                found.push(format!("after `{sent}`: expect_event {payload:?}"));
            }
            ScenarioStep::ExpectEventValues {
                event: expected,
                payload,
                ..
            } if expected.to_string() == event => {
                found.push(format!("after `{sent}`: expect_event_values {payload:?}"));
            }
            _ => {}
        }
    }
    found.join("\n")
}

/// The fields `event`'s expectation in `id` compares with a captured instance.
fn compared_instances(suite: &ConformanceSuite, id: &str, event: &str) -> Vec<String> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
        .unwrap();
    let mut fields = Vec::new();
    for step in &scenario.steps {
        if let ScenarioStep::ExpectEventValues {
            event: expected,
            payload,
            ..
        } = step
        {
            if expected.to_string() == event {
                for (field, value) in payload {
                    if matches!(value, ScenarioValue::Instance { .. }) {
                        fields.push(field.clone());
                    }
                }
            }
        }
    }
    fields
}

// ---- a target publishing one wrong identity in the trigger's event ----------------------------

/// The interpreter, publishing one field of one event wrongly: the nil UUID, an id no run minted,
/// or the value of `other`, a second identity of the same event (another real row's).
struct Faulty {
    inner: Interpreted,
    event: String,
    field: String,
    mode: &'static str,
    other: Option<String>,
}

impl Faulty {
    fn new(ir: &EssIr, event: &str, field: &str, mode: &'static str) -> Self {
        Self {
            inner: Interpreted::for_model(ir.clone()),
            event: event.into(),
            field: field.into(),
            mode,
            other: None,
        }
    }
}

impl ConformanceTarget for Faulty {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn fixture_values(
        &self,
        scenario: &ScenarioContext,
        contract: &ess_conformance::fixtures::Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        self.inner.fixture_values(scenario, contract)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut result = self.inner.execute_command(request)?;
        for event in &mut result.direct_events {
            if event.event.to_string() != self.event {
                continue;
            }
            let wrong = match self.mode {
                "zero" => Node::Text("00000000-0000-0000-0000-000000000000".into()),
                "swap" => {
                    let other = self.other.as_ref().expect("swap names the other field");
                    let theirs = event.payload.get(other).cloned().unwrap_or(Node::Null);
                    let ours = event
                        .payload
                        .get(&self.field)
                        .cloned()
                        .unwrap_or(Node::Null);
                    event.payload.insert(other.clone(), ours);
                    theirs
                }
                _ => Node::Text("00000000-0000-4000-8000-0000000000ff".into()),
            };
            event.payload.insert(self.field.clone(), wrong);
        }
        Ok(result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
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
    fn configure_external_outcome_repeatedly(
        &self,
        control: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        self.inner
            .configure_external_outcome_repeatedly(control, times)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        self.inner.deliver_event(request)
    }
}

/// Each witness of `binding` the reference passes, and the witnesses a target publishing `field`
/// of `event` wrongly in each of `modes` does not fail.
fn survivors(
    ir: &EssIr,
    suite: &ConformanceSuite,
    binding: &str,
    event: &str,
    field: &str,
    modes: &[&'static str],
    other: Option<&str>,
) -> Vec<String> {
    let ids = witnesses(suite, binding);
    let good = statuses(suite, &Interpreted::for_model(ir.clone()));
    for id in &ids {
        assert_eq!(
            good.get(id),
            Some(&Status::Passed),
            "{id}: the reference interpreter passes the witness"
        );
    }
    let mut survived = Vec::new();
    for mode in modes {
        let mut target = Faulty::new(ir, event, field, mode);
        target.other = other.map(str::to_owned);
        let ran = statuses(suite, &target);
        for id in &ids {
            if ran.get(id) != Some(&Status::Failed) {
                survived.push(format!(
                    "{id} `{event}.{field}` {mode}: {:?}\n    {}",
                    ran.get(id),
                    trigger_expectation(suite, id, event).replace('\n', "\n    ")
                ));
            }
        }
    }
    survived
}

// ---- 1. every witness, every identity field of the unit's own fixtures -------------------------

/// Acceptance, bullet 1 and 3, read per scenario: each `<binding>/binding/refusal/<outcome>`
/// scenario of the unit's five fixtures fails a target that publishes another id in any identity
/// field of its trigger's event. The unit's own test mutates one field per fixture and asks only
/// that one witness of the binding fail.
#[test]
fn every_refusal_witness_fails_a_wrong_id_in_every_identity_field_of_its_trigger_event() {
    let mut survived = Vec::new();
    for (dir, binding, event, fields) in [
        (
            "caller-binding-refusal/plain",
            "give-on-touched",
            "demo.d.Touched",
            &["grant_id"][..],
        ),
        (
            "caller-binding-refusal/attr",
            "give-on-touched",
            "demo.d.Touched",
            &["grant_id"][..],
        ),
        (
            "predicate-refusal-policy-arranged-row",
            "give-on-touched",
            "demo.d.Touched",
            &["grant_id"][..],
        ),
        (
            "binding-into-updating-command",
            "refresh-on-ticket-moved",
            "demo.b.TicketMoved",
            &["ticket_id", "grant_id"][..],
        ),
        (
            "binding-refusal-policy-swapped-caller",
            "refresh-on-ticket-moved",
            "demo.b.TicketMoved",
            &["ticket_id", "grant_id"][..],
        ),
    ] {
        let ir = directory(dir);
        let (suite, refusals) = synthesized(&ir);
        assert!(
            !witnesses(&suite, binding).is_empty(),
            "{dir}: `{binding}` has refusal witnesses\n{refusals}"
        );
        for field in fields {
            survived.extend(
                survivors(
                    &ir,
                    &suite,
                    binding,
                    event,
                    field,
                    &["zero", "stranger"],
                    None,
                )
                .into_iter()
                .map(|line| format!("{dir} {line}")),
            );
        }
    }
    assert!(
        survived.is_empty(),
        "refusal witnesses passing a wrong identity in their trigger's event \
         (story:a-refusal-witness-compares-its-trigger-identities, Acceptance 1):\n{}",
        survived.join("\n")
    );
}

// ---- 2. a refusal forced by injection, two identities of two real rows ------------------------

const FORCED: &str = r"domain: demo.f
types:
  - {name: demo.f.TicketId, kind: newtype, of: Uuid}
  - {name: demo.f.BoardId, kind: newtype, of: Uuid}
entities:
  - {name: demo.f.Ticket, identity: {name: ticket_id, type: demo.f.TicketId}, fields: [{name: note, type: String}], lifecycle: {initial: Active, states: [Active], terminal: [Active]}}
  - {name: demo.f.Board, identity: {name: board_id, type: demo.f.BoardId}, fields: [{name: note, type: String}], lifecycle: {initial: Active, states: [Active], terminal: [Active]}}
commands:
  - name: demo.f.OpenTicket
    input: [{name: note, type: String}]
    outcomes:
      - {name: opened, creates: demo.f.Ticket, instance: ticket_id, emits: [demo.f.TicketOpened], payload: {demo.f.TicketOpened: {ticket_id: {generated: true}}}, sets: {note: input.note}}
  - name: demo.f.OpenBoard
    input: [{name: note, type: String}]
    outcomes:
      - {name: opened, creates: demo.f.Board, instance: board_id, emits: [demo.f.BoardOpened], payload: {demo.f.BoardOpened: {board_id: {generated: true}}}, sets: {note: input.note}}
  - name: demo.f.Pin
    input: [{name: ticket_id, type: demo.f.TicketId}, {name: board_id, type: demo.f.BoardId}, {name: note, type: String}]
    outcomes:
      - {name: no-board, when_related: {via: input.board_id, exists: false}, error: demo.f.Missing}
      - {name: pinned, updates: demo.f.Ticket, instance: ticket_id, emits: [demo.f.Pinned], payload: {demo.f.Pinned: {ticket_id: input.ticket_id, board_id: input.board_id}}, sets: {note: input.note}}
  - name: demo.f.Notify
    input: [{name: ticket_id, type: demo.f.TicketId}, {name: board_id, type: demo.f.BoardId}]
    outcomes:
      - {name: notified, emits: [demo.f.Notified], payload: {demo.f.Notified: {ticket_id: input.ticket_id}}}
      - {name: unavailable, external: the notifier answers a server error, error: demo.f.Unavailable}
      - {name: busy, external: the notifier is overloaded, error: demo.f.Unavailable}
errors: [{name: demo.f.Missing}, {name: demo.f.Unavailable}]
events:
  - {name: demo.f.TicketOpened, fields: [{name: ticket_id, type: demo.f.TicketId}]}
  - {name: demo.f.BoardOpened, fields: [{name: board_id, type: demo.f.BoardId}]}
  - {name: demo.f.Pinned, fields: [{name: ticket_id, type: demo.f.TicketId}, {name: board_id, type: demo.f.BoardId}]}
  - {name: demo.f.Notified, fields: [{name: ticket_id, type: demo.f.TicketId}]}
actors:
  - {name: demo.f.Clerk, may: [demo.f.OpenTicket, demo.f.OpenBoard, demo.f.Pin, demo.f.Notify]}
";

fn forced_components(on_failure: &str) -> String {
    format!(
        "components:
  - {{component: svc, owns: {{domains: [demo.f]}}, accepts: {{commands: [demo.f.OpenTicket, demo.f.OpenBoard, demo.f.Pin, demo.f.Notify]}}}}
bindings:
  - id: notify-on-pinned
    when: {{event: demo.f.Pinned}}
    invoke: {{command: demo.f.Notify}}
    mapping: {{ticket_id: event.ticket_id, board_id: event.board_id}}
    delivery: at_least_once
    on_failure: {on_failure}
"
    )
}

/// A refusal forced by injection (`drop`, and a bounded `retry` with one refusal `final`): the
/// trigger's `Pinned` carries the ticket and the board, two rows the arrangement created. Each
/// witness fails a target that publishes either id wrongly, or the two swapped (each the id of
/// another real row).
#[test]
fn a_forced_refusal_witness_compares_both_identities_of_its_trigger_event() {
    let mut survived = Vec::new();
    for policy in [
        "{drop: [unavailable, busy], retry: {except: [unavailable, busy]}}",
        "{retry: {outcomes: [unavailable, busy], attempts: 3, final: [busy]}, drop: {except: [unavailable, busy]}}",
    ] {
        let ir = system("demo.f", &forced_components(policy), FORCED);
        let (suite, refusals) = synthesized(&ir);
        let ids = witnesses(&suite, "notify-on-pinned");
        assert_eq!(
            ids.len(),
            2,
            "{policy}: both refusals are witnessed: {ids:?}\n{refusals}"
        );
        for id in &ids {
            eprintln!(
                "{policy} {id}:\n{}",
                trigger_expectation(&suite, id, "demo.f.Pinned")
            );
            let compared = compared_instances(&suite, id, "demo.f.Pinned");
            if compared != ["board_id", "ticket_id"] {
                survived.push(format!(
                    "{policy} {id}: compares {compared:?} of `Pinned`, not both identities"
                ));
            }
        }
        for field in ["ticket_id", "board_id"] {
            survived.extend(
                survivors(
                    &ir,
                    &suite,
                    "notify-on-pinned",
                    "demo.f.Pinned",
                    field,
                    &["zero", "stranger"],
                    None,
                )
                .into_iter()
                .map(|line| format!("{policy} {line}")),
            );
        }
        survived.extend(
            survivors(
                &ir,
                &suite,
                "notify-on-pinned",
                "demo.f.Pinned",
                "ticket_id",
                &["swap"],
                Some("board_id"),
            )
            .into_iter()
            .map(|line| format!("{policy} {line}")),
        );
    }
    assert!(survived.is_empty(), "{}", survived.join("\n"));
}

// ---- 3. a publisher swapped in, its identity copied from an input under another name ----------

const SWAPPED: &str = r"domain: demo.a
types:
  - {name: demo.a.Ref, kind: newtype, of: Uuid}
entities:
  - {name: demo.a.Ticket, identity: {name: ticket_id, type: demo.a.Ref}, fields: [{name: note, type: String}], lifecycle: {initial: Active, states: [Active], terminal: [Active]}}
  - {name: demo.a.Board, identity: {name: board_id, type: demo.a.Ref}, fields: [{name: note, type: String}], lifecycle: {initial: Active, states: [Active], terminal: [Active]}}
commands:
  - name: demo.a.AMove
    input: [{name: ticket_id, type: demo.a.Ref}, {name: note, type: String}]
    outcomes:
      - {name: moved, updates: demo.a.Ticket, instance: ticket_id, emits: [demo.a.Moved], payload: {demo.a.Moved: {ticket_id: input.ticket_id}}, sets: {note: input.note}}
  - name: demo.a.CreateBoard
    input: [{name: note, type: String}]
    outcomes:
      - {name: created, creates: demo.a.Board, instance: board_id, emits: [demo.a.BoardCreated], payload: {demo.a.BoardCreated: {board_id: {generated: true}}}, sets: {note: input.note}}
  - name: demo.a.Shift
    input: [{name: board_id, type: demo.a.Ref}, {name: note, type: String}]
    outcomes:
      - {name: shifted, updates: demo.a.Board, instance: board_id, emits: [demo.a.Moved], payload: {demo.a.Moved: {ticket_id: input.board_id}}, sets: {note: input.note}}
  - name: demo.a.ZOpen
    input: [{name: note, type: String}]
    outcomes:
      - {name: opened, creates: demo.a.Ticket, instance: ticket_id, emits: [demo.a.Moved], payload: {demo.a.Moved: {ticket_id: {generated: true}}}, sets: {note: input.note}}
  - name: demo.a.Notify
    input: [{name: ticket_id, type: demo.a.Ref}]
    outcomes:
      - {name: notified, emits: [demo.a.Notified], payload: {demo.a.Notified: {ticket_id: input.ticket_id}}}
      - {name: unavailable, external: the notifier answers a server error, error: demo.a.Unavailable}
errors: [{name: demo.a.Unavailable}]
events:
  - {name: demo.a.Moved, fields: [{name: ticket_id, type: demo.a.Ref}]}
  - {name: demo.a.BoardCreated, fields: [{name: board_id, type: demo.a.Ref}]}
  - {name: demo.a.Notified, fields: [{name: ticket_id, type: demo.a.Ref}]}
actors:
  - {name: demo.a.Clerk, may: [demo.a.AMove, demo.a.CreateBoard, demo.a.Shift, demo.a.ZOpen, demo.a.Notify]}
";

const SWAPPED_COMPONENTS: &str = "components:
  - {component: svc, owns: {domains: [demo.a]}, accepts: {commands: [demo.a.AMove, demo.a.CreateBoard, demo.a.Shift, demo.a.ZOpen, demo.a.Notify]}}
bindings:
  - id: notify-on-moved
    when: {event: demo.a.Moved}
    invoke: {command: demo.a.Notify}
    mapping: {ticket_id: event.ticket_id}
    delivery: at_least_once
    on_failure: {drop: [unavailable], retry: {except: [unavailable]}}
";

/// Where the trigger's arrangement publishes the binding's event itself, the witness sends another
/// publisher whose arrangement does not. Here that publisher fills `Moved.ticket_id` from its
/// input `board_id`: the witness compares the field with the board it sent, and fails a target
/// that publishes another id.
#[test]
fn a_swapped_publisher_witness_compares_an_identity_copied_under_another_name() {
    let ir = system("demo.a", SWAPPED_COMPONENTS, SWAPPED);
    let (suite, refusals) = synthesized(&ir);
    let ids = witnesses(&suite, "notify-on-moved");
    assert_eq!(
        ids.len(),
        1,
        "the refusal is witnessed: {ids:?}\n{refusals}"
    );
    let shown = trigger_expectation(&suite, &ids[0], "demo.a.Moved");
    eprintln!("{}:\n{shown}", ids[0]);
    let compared = compared_instances(&suite, &ids[0], "demo.a.Moved");
    assert_eq!(
        compared,
        ["ticket_id"],
        "the witness compares `Moved.ticket_id` with the row it sent:\n{shown}"
    );
    let survived = survivors(
        &ir,
        &suite,
        "notify-on-moved",
        "demo.a.Moved",
        "ticket_id",
        &["zero", "stranger"],
        None,
    );
    assert!(survived.is_empty(), "{}", survived.join("\n"));
}

// ---- 4. a publisher swapped in because the first refuses the missing row itself ----------------

const UNGUARDED: &str = r"domain: demo.u
types:
  - {name: demo.u.GrantId, kind: newtype, of: String}
  - {name: demo.u.NoteId, kind: newtype, of: Uuid}
entities:
  - {name: demo.u.Grant, identity: {name: grant_id, type: demo.u.GrantId}, fields: [{name: flag, type: Boolean}], lifecycle: {initial: Active, states: [Active], terminal: [Active]}}
  - {name: demo.u.Note, identity: {name: note_id, type: demo.u.NoteId}, fields: [{name: text, type: String}], lifecycle: {initial: Active, states: [Active], terminal: [Active]}}
commands:
  - name: demo.u.ATouch
    input: [{name: grant_id, type: demo.u.GrantId}, {name: note_id, type: demo.u.NoteId}, {name: text, type: String}]
    outcomes:
      - {name: no-grant, when_related: {via: input.grant_id, exists: false}, error: demo.u.Refused}
      - {name: touched, updates: demo.u.Note, instance: note_id, emits: [demo.u.Touched], payload: {demo.u.Touched: {grant_id: input.grant_id, note_id: input.note_id}}, sets: {text: input.text}}
  - name: demo.u.DefineGrant
    input: [{name: flag, type: Boolean}]
    outcomes:
      - {name: defined, creates: demo.u.Grant, instance: grant_id, emits: [demo.u.GrantDefined], payload: {demo.u.GrantDefined: {grant_id: {generated: true}}}, sets: {flag: input.flag}}
  - name: demo.u.Give
    input: [{name: grant_id, type: demo.u.GrantId}]
    outcomes:
      - {name: missing, when_related: {via: input.grant_id, exists: false}, error: demo.u.Refused}
      - {name: given, emits: [demo.u.Given], payload: {demo.u.Given: {grant_id: input.grant_id}}}
  - name: demo.u.Poke
    input: [{name: grant_id, type: demo.u.GrantId}, {name: note_id, type: demo.u.NoteId}, {name: text, type: String}]
    outcomes:
      - {name: poked, updates: demo.u.Note, instance: note_id, emits: [demo.u.Touched], payload: {demo.u.Touched: {grant_id: input.grant_id, note_id: input.note_id}}, sets: {text: input.text}}
  - name: demo.u.WriteNote
    input: [{name: text, type: String}]
    outcomes:
      - {name: written, creates: demo.u.Note, instance: note_id, emits: [demo.u.NoteWritten], payload: {demo.u.NoteWritten: {note_id: {generated: true}}}, sets: {text: input.text}}
errors: [{name: demo.u.Refused}]
events:
  - {name: demo.u.GrantDefined, fields: [{name: grant_id, type: demo.u.GrantId}]}
  - {name: demo.u.NoteWritten, fields: [{name: note_id, type: demo.u.NoteId}]}
  - {name: demo.u.Touched, fields: [{name: grant_id, type: demo.u.GrantId}, {name: note_id, type: demo.u.NoteId}]}
  - {name: demo.u.Given, fields: [{name: grant_id, type: demo.u.GrantId}]}
actors:
  - {name: demo.u.Clerk, may: [demo.u.ATouch, demo.u.DefineGrant, demo.u.Give, demo.u.Poke, demo.u.WriteNote]}
";

const UNGUARDED_COMPONENTS: &str = "components:
  - {component: svc, owns: {domains: [demo.u]}, accepts: {commands: [demo.u.ATouch, demo.u.DefineGrant, demo.u.Give, demo.u.Poke, demo.u.WriteNote]}}
bindings:
  - id: give-on-touched
    when: {event: demo.u.Touched}
    invoke: {command: demo.u.Give}
    mapping: {grant_id: event.grant_id}
    delivery: at_least_once
    on_failure: {drop: [missing], retry: {except: [missing]}}
";

/// `ATouch` refuses a grant no row carries itself, so the `missing` witness sends `Poke`, which
/// does not (`unguarded_publisher`). `Touched` carries the note `Poke` updates as well as the
/// grant: the witness compares the note with the row it sent and fails a target that publishes
/// another.
#[test]
fn an_unguarded_publisher_witness_compares_the_identity_of_the_row_it_sent() {
    let ir = system("demo.u", UNGUARDED_COMPONENTS, UNGUARDED);
    let (suite, refusals) = synthesized(&ir);
    let ids = witnesses(&suite, "give-on-touched");
    assert_eq!(
        ids.len(),
        1,
        "the refusal is witnessed: {ids:?}\n{refusals}"
    );
    let shown = trigger_expectation(&suite, &ids[0], "demo.u.Touched");
    eprintln!("{}:\n{shown}", ids[0]);
    assert_eq!(
        compared_instances(&suite, &ids[0], "demo.u.Touched"),
        ["note_id"],
        "the witness compares `Touched.note_id` with the note it sent:\n{shown}"
    );
    let survived = survivors(
        &ir,
        &suite,
        "give-on-touched",
        "demo.u.Touched",
        "note_id",
        &["zero", "stranger"],
        None,
    );
    assert!(survived.is_empty(), "{}", survived.join("\n"));
}

// ---- 5. ported from the security pass: the id a missing-row trigger was sent ------------------

/// The refusal-policy fixtures, each with its binding, the binding's trigger event and the
/// identity field of that event the binding maps into the invoked command's related-row input.
const RELATED: [(&str, &str, &str, &str); 5] = [
    (
        "caller-binding-refusal/plain",
        "give-on-touched",
        "demo.d.Touched",
        "grant_id",
    ),
    (
        "caller-binding-refusal/attr",
        "give-on-touched",
        "demo.d.Touched",
        "grant_id",
    ),
    (
        "predicate-refusal-policy-arranged-row",
        "give-on-touched",
        "demo.d.Touched",
        "grant_id",
    ),
    (
        "binding-into-updating-command",
        "refresh-on-ticket-moved",
        "demo.b.TicketMoved",
        "grant_id",
    ),
    (
        "binding-refusal-policy-swapped-caller",
        "refresh-on-ticket-moved",
        "demo.b.TicketMoved",
        "grant_id",
    ),
];

/// A witness whose trigger is sent naming a row no row carries (the `exists: false` refusal,
/// answered on every attempt) compares that id in the trigger's event: the scenario sent it, the
/// binding maps it into the invoked command, and it is the identity the refusal is about.
#[test]
fn a_missing_row_witness_compares_the_id_its_trigger_was_sent() {
    let mut measured = 0;
    let mut uncompared = Vec::new();
    for (dir, binding, event, field) in RELATED {
        let ir = directory(dir);
        let (suite, _) = synthesized(&ir);
        for id in witnesses(&suite, binding) {
            let scenario = suite
                .scenarios
                .iter()
                .find(|(key, _)| key.to_string() == id)
                .map(|(_, scenario)| scenario)
                .unwrap();
            let mut sent: Option<ScenarioValue> = None;
            for step in &scenario.steps {
                match step {
                    ScenarioStep::ExecuteCommand { input, .. } => {
                        sent = input.get(field).cloned();
                    }
                    ScenarioStep::ExpectEvent {
                        event: expected, ..
                    }
                    | ScenarioStep::ExpectEventValues {
                        event: expected, ..
                    } if expected.to_string() == event => {
                        let Some(value) = &sent else { continue };
                        measured += 1;
                        let compared = match step {
                            ScenarioStep::ExpectEventValues { payload, .. } => {
                                payload.get(field).cloned()
                            }
                            ScenarioStep::ExpectEvent { payload, .. } => {
                                payload.get(field).cloned().map(ScenarioValue::literal)
                            }
                            _ => None,
                        };
                        if compared.as_ref() != Some(value) {
                            uncompared.push(format!(
                                "{dir} {id}: sent `{field}` = {value:?}, expects `{event}.{field}` \
                                 = {compared:?}"
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    eprintln!("trigger events measured: {measured}");
    assert!(measured > 0);
    assert!(
        uncompared.is_empty(),
        "refusal witnesses expecting their trigger's event without the identity the trigger was \
         sent (story:a-refusal-witness-compares-its-trigger-identities): {uncompared:#?}"
    );
}

// ---- 6. ported from the security pass: another record's id ------------------------------------

/// The reference interpreter publishing, in `event.field`, the id of another record it already
/// published under the same field name earlier in the scenario (another row of the same identity
/// type), where it holds one.
struct OtherRecord {
    inner: Interpreted,
    event: String,
    field: String,
    seen: RefCell<Vec<Node>>,
    swapped: RefCell<usize>,
}

impl ConformanceTarget for OtherRecord {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.seen.borrow_mut().clear();
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
            let Some(published) = event.payload.get(&self.field).cloned() else {
                continue;
            };
            if event.event.to_string() == self.event {
                let other = self
                    .seen
                    .borrow()
                    .iter()
                    .rev()
                    .find(|held| **held != published)
                    .cloned();
                if let Some(other) = other {
                    event.payload.insert(self.field.clone(), other);
                    *self.swapped.borrow_mut() += 1;
                }
            } else {
                self.seen.borrow_mut().push(published);
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
    fn configure_external_outcome_repeatedly(
        &self,
        control: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        self.inner
            .configure_external_outcome_repeatedly(control, times)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// A target publishing another record's id (one it already published under the same field in the
/// scenario) in the trigger's event fails every witness where such a record exists.
#[test]
fn a_refusal_witness_fails_a_target_publishing_another_records_id() {
    let mut swaps = 0;
    let mut survivors = Vec::new();
    for (dir, binding, event, field) in RELATED {
        let ir = directory(dir);
        let (suite, _) = synthesized(&ir);
        let ids = witnesses(&suite, binding);
        let target = OtherRecord {
            inner: Interpreted::for_model(ir.clone()),
            event: event.into(),
            field: field.into(),
            seen: RefCell::new(Vec::new()),
            swapped: RefCell::new(0),
        };
        for id in &ids {
            let mut only = suite.clone();
            only.scenarios.retain(|key, _| key.to_string() == *id);
            let before = *target.swapped.borrow();
            let ran = statuses(&only, &target);
            let after = *target.swapped.borrow();
            if after == before {
                eprintln!("not measured: {dir} {id}: no other record to publish");
                continue;
            }
            swaps += 1;
            if ran.get(id) != Some(&Status::Failed) {
                survivors.push(format!("{dir} {id}: {:?}", ran.get(id)));
            }
        }
    }
    eprintln!("witnesses with another record swapped in: {swaps}");
    assert!(
        survivors.is_empty(),
        "refusal witnesses passing another record's id in the trigger's event \
         (story:a-refusal-witness-compares-its-trigger-identities): {survivors:#?}"
    );
}
