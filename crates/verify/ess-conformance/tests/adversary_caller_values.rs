//! Adversary, pass 1, `story:caller-value-source-and-guard` (beyond10x/ess#168).
//!
//! The synthesized suite is run against an implementation that does exactly what the
//! specification says — records the caller's attributes, refuses an edit when the stored field
//! differs from the caller's — and must pass it. Two shapes the unit's own tests do not build:
//!
//! 1. a guard over a `Boolean` caller attribute (`admin != caller.admin`): the literal written in
//!    for the caller is compared as text, so the suite expects the refusal for the note's own
//!    creator and refuses the accepted branch as unsatisfiable;
//! 2. an aggregate view over the entity a caller-reading command creates: the swapped run is
//!    appended to the same scenario, and its absolute counts ignore the first run's rows.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The #168 repro, with the command that refuses a caller who is not the note's agent.
const NOTES: &str = "format: ess/16
system: demo
version: v1
summary: Minimal repro.
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}
  - {name: demo.notes.AgentId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}
        emits: [demo.notes.NoteCreated]
        payload:
          demo.notes.NoteCreated: {note_id: {generated: true}, account_id: {caller: account_id}, text: input.text}
  - name: demo.notes.EditNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteEdited]
        payload:
          demo.notes.NoteEdited: {note_id: input.note_id, text: input.text}
errors:
  - name: demo.notes.NotYourNote
    summary: The caller is not the note's agent.
events:
  - name: demo.notes.NoteCreated
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: text, type: String}
  - name: demo.notes.NoteEdited
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
      - {name: state, type: demo.notes.Note.State}
";

/// [`NOTES`] with a `Boolean` attribute `admin` the note records and the refusal compares.
fn boolean_guard() -> String {
    NOTES
        .replace(
            "      - {name: agent_id, type: demo.notes.AgentId}\n    may:",
            "      - {name: agent_id, type: demo.notes.AgentId}\n      - {name: admin, type: Boolean}\n    may:",
        )
        .replace(
            "      - {name: text, type: String}\n    lifecycle:",
            "      - {name: text, type: String}\n      - {name: admin, type: Boolean}\n    lifecycle:",
        )
        .replace(
            "agent_id: {caller: agent_id}, text: input.text}\n",
            "agent_id: {caller: agent_id}, text: input.text, admin: {caller: admin}}\n",
        )
        .replace(
            "      - {name: text, type: String}\n      - {name: state, type: demo.notes.Note.State}\n",
            "      - {name: text, type: String}\n      - {name: admin, type: Boolean}\n      - {name: state, type: demo.notes.Note.State}\n",
        )
        .replace(
            "when_subject: {predicate: agent_id != caller.agent_id}",
            "when_subject: {predicate: admin != caller.admin}",
        )
}

/// [`NOTES`] with a view counting notes by their text.
fn with_aggregate() -> String {
    format!(
        "{NOTES}  - name: demo.notes.NotesByText\n    source: demo.notes.Note\n    group_by: [text]\n    fields:\n      - {{name: text, type: String}}\n      - {{name: notes, type: Integer, aggregate: {{count: {{}}}}}}\n"
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// What the specification says, and nothing else: every attribute of the caller that sends
/// `CreateNote` is recorded on the note, and `EditNote` is refused when the note's `guard` field
/// differs from the caller's.
struct Correct {
    guard: &'static str,
    notes: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

impl Correct {
    fn new(guard: &'static str) -> Self {
        Self {
            guard,
            notes: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Correct {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-notes", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.notes.replace(BTreeMap::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let caller = request
            .caller
            .clone()
            .expect("every command is sent as a caller");
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let mut notes = self.notes.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.notes.CreateNote" => {
                let id = format!("00000000-0000-4000-8000-{n:012}");
                let mut row = caller.clone();
                row.insert("note_id".to_owned(), Node::Text(id.clone()));
                row.insert("text".to_owned(), request.input["text"].clone());
                row.insert("state".to_owned(), Node::Text("Open".to_owned()));
                notes.insert(id.clone(), row);
                SemanticCommandResult::took(outcome(&command, "created")).emitting(
                    ObservedEvent::new("demo.notes.NoteCreated".parse().unwrap())
                        .with("note_id", Node::Text(id))
                        .with("account_id", caller["account_id"].clone())
                        .with("text", request.input["text"].clone()),
                )
            }
            "demo.notes.EditNote" => {
                let Some(Node::Text(id)) = request.input.get("note_id").cloned() else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let Some(row) = notes.get_mut(&id) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                if row[self.guard] == caller[self.guard] {
                    row.insert("text".to_owned(), request.input["text"].clone());
                    SemanticCommandResult::took(outcome(&command, "edited")).emitting(
                        ObservedEvent::new("demo.notes.NoteEdited".parse().unwrap())
                            .with("note_id", Node::Text(id))
                            .with("text", request.input["text"].clone()),
                    )
                } else {
                    SemanticCommandResult::took(outcome(&command, "forbidden")).with_error(
                        DeclaredErrorValue::new("demo.notes.NotYourNote".parse().unwrap()),
                    )
                }
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let notes = self.notes.borrow();
        if request.view.to_string() == "demo.notes.NotesByText" {
            let mut counts: BTreeMap<String, u32> = BTreeMap::new();
            for row in notes.values() {
                if let Node::Text(text) = &row["text"] {
                    *counts.entry(text.clone()).or_default() += 1;
                }
            }
            return Ok(SemanticViewResult::of(counts.into_iter().map(
                |(text, n)| {
                    BTreeMap::from([
                        ("text".to_owned(), Node::Text(text)),
                        (
                            "notes".to_owned(),
                            Node::Number(ess_primitives::facts::Number::new(f64::from(n)).unwrap()),
                        ),
                    ])
                },
            )));
        }
        Ok(SemanticViewResult::of(notes.values().cloned()))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// Every scenario of `suite` that did not pass against `target`, by id, with its status.
fn failing(suite: &ConformanceSuite, target: &Correct) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

#[test]
fn adversary_caller_a_boolean_guard_is_synthesized_with_nothing_refused() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(&boolean_guard()));
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .collect();
    assert!(
        refused.is_empty(),
        "`edited` is reached by the note's own creator; nothing is refused: {refused:#?}"
    );
}

#[test]
fn adversary_caller_a_boolean_guard_refuses_only_a_caller_whose_flag_differs() {
    let suite = ess_conformance::synthesize::synthesize(&ir(&boolean_guard())).suite;
    let forbidden = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.notes.EditNote/outcome/forbidden")
        .map(|(_, scenario)| scenario)
        .expect("the refusal is witnessed");
    // Pair each edit of a created note with the caller that created it.
    let mut creator: Option<Node> = None;
    let mut checked = 0;
    for step in &forbidden.steps {
        if let ScenarioStep::ExecuteCommand {
            command,
            caller,
            input,
            ..
        } = step
        {
            if command.to_string() == "demo.notes.CreateNote" {
                creator = Some(caller["admin"].clone());
            } else if !matches!(
                input.get("note_id"),
                Some(ess_conformance::ScenarioValue::Literal { .. })
            ) {
                let made_by = creator.clone().expect("a note was created first");
                assert_ne!(
                    made_by, caller["admin"],
                    "the edit expected to be `forbidden` is sent by a caller whose `admin` \
                         equals the note's, so `admin != caller.admin` is false and a correct \
                         implementation answers `edited`"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "an edit of a created note is refused");
}

#[test]
fn adversary_caller_a_boolean_guard_suite_passes_a_correct_implementation() {
    let suite = ess_conformance::synthesize::synthesize(&ir(&boolean_guard())).suite;
    assert_eq!(failing(&suite, &Correct::new("admin")), Vec::new());
}

#[test]
fn adversary_caller_the_agent_guard_suite_passes_this_implementation() {
    // Control: the unit's own repro passes the same implementation, so a red case above is the
    // suite's, not the implementation's.
    let suite = ess_conformance::synthesize::synthesize(&ir(NOTES)).suite;
    assert_eq!(failing(&suite, &Correct::new("agent_id")), Vec::new());
}

#[test]
fn adversary_caller_an_aggregate_view_suite_passes_a_correct_implementation() {
    let suite = ess_conformance::synthesize::synthesize(&ir(&with_aggregate())).suite;
    assert!(
        suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == "demo.notes.NotesByText/aggregate"),
        "the aggregate view is witnessed"
    );
    assert_eq!(failing(&suite, &Correct::new("agent_id")), Vec::new());
}
