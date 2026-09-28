//! Adversary, pass 2, `story:caller-value-source-and-guard` (beyond10x/ess#168).
//!
//! 1. Two actors that declare an attribute of one name at different types, each for its own
//!    commands: the domain admits it (a command's actors agree), and synthesis must send each
//!    command with a value of its own actor's type.
//! 2. A view filtered on a caller-derived field that does not project the identity: the swapped
//!    run is appended to the same scenario, and an `excludes` without an identity reads "the view
//!    holds no rows" — which the first run's row makes false for a correct implementation.
//! 3. Mutants of the implementation (records an input instead of the caller, refuses every caller)
//!    that the suite has to fail.

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

/// The #168 repro, with a `Boolean` attribute `admin` the note records, and two views over it
/// that do not project the note's identity.
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
      - {name: admin, type: Boolean}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: admin, type: Boolean}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, admin: {caller: admin}, text: input.text}
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
      - {name: admin, type: Boolean}
      - {name: text, type: String}
      - {name: state, type: demo.notes.Note.State}
  - name: demo.notes.AdminNotes
    source: demo.notes.Note
    consistency: read_your_writes
    filter: 'admin == true'
    fields:
      - {name: text, type: String}
      - {name: admin, type: Boolean}
  - name: demo.notes.MemberNotes
    source: demo.notes.Note
    consistency: read_your_writes
    filter: 'admin == false'
    fields:
      - {name: text, type: String}
      - {name: admin, type: Boolean}
";

const CREATED: &str = "demo.notes.CreateNote/outcome/created";
const EDITED: &str = "demo.notes.EditNote/outcome/edited";

/// [`NOTES`] with a second actor that declares `admin` as an `Integer` for a command of its own,
/// which records it in an event.
fn two_types_one_name() -> String {
    NOTES
        .replace(
            "    may: [demo.notes.CreateNote, demo.notes.EditNote]\n",
            "    may: [demo.notes.CreateNote, demo.notes.EditNote]\n  - name: demo.notes.Auditor\n    attributes:\n      - {name: admin, type: Integer}\n    may: [demo.notes.Audit]\n",
        )
        .replace(
            "commands:\n",
            "commands:\n  - name: demo.notes.Audit\n    outcomes:\n      - name: audited\n        emits: [demo.notes.Audited]\n        payload:\n          demo.notes.Audited: {level: {caller: admin}}\n",
        )
        .replace(
            "events:\n",
            "events:\n  - name: demo.notes.Audited\n    fields:\n      - {name: level, type: Integer}\n",
        )
}

/// [`NOTES`] where `CreateNote` also takes an `account_id` input it does not record: the account
/// is the caller's.
fn with_account_input() -> String {
    NOTES.replace(
        "    input:\n      - {name: text, type: String}\n    outcomes:\n      - name: created\n",
        "    input:\n      - {name: text, type: String}\n      - {name: account_id, type: demo.notes.AccountId}\n    outcomes:\n      - name: created\n",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// What the specification says.
    Correct,
    /// Records the `account_id` input, where one is sent, instead of the caller's.
    InputAccount,
    /// Refuses every edit.
    RefusesEveryone,
}

struct Notes {
    mode: Mode,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

impl Notes {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Notes {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-notes-pass2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
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
        let mut notes = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.notes.CreateNote" => {
                let id = format!("00000000-0000-4000-8000-{n:012}");
                let mut row = caller.clone();
                if self.mode == Mode::InputAccount {
                    if let Some(account) = request.input.get("account_id") {
                        row.insert("account_id".to_owned(), account.clone());
                    }
                }
                row.insert("note_id".to_owned(), Node::Text(id.clone()));
                row.insert("text".to_owned(), request.input["text"].clone());
                row.insert("state".to_owned(), Node::Text("Open".to_owned()));
                let account = row["account_id"].clone();
                notes.insert(id.clone(), row);
                SemanticCommandResult::took(outcome(&command, "created")).emitting(
                    ObservedEvent::new("demo.notes.NoteCreated".parse().unwrap())
                        .with("note_id", Node::Text(id))
                        .with("account_id", account)
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
                let permitted =
                    self.mode != Mode::RefusesEveryone && row["agent_id"] == caller["agent_id"];
                if permitted {
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
        let notes = self.rows.borrow();
        let filtered = |admin: bool| {
            notes
                .values()
                .filter(|row| row["admin"] == Node::Bool(admin))
                .map(|row| {
                    BTreeMap::from([
                        ("text".to_owned(), row["text"].clone()),
                        ("admin".to_owned(), row["admin"].clone()),
                    ])
                })
                .collect::<Vec<_>>()
        };
        Ok(match request.view.to_string().as_str() {
            "demo.notes.AdminNotes" => SemanticViewResult::of(filtered(true)),
            "demo.notes.MemberNotes" => SemanticViewResult::of(filtered(false)),
            _ => SemanticViewResult::of(notes.values().cloned()),
        })
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

fn failing(suite: &ConformanceSuite, mode: Mode) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let mut failed: Vec<(String, Status)> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Notes::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            if std::env::var_os("ADVERSARY_DIAGNOSTICS").is_some() {
                for diagnostic in result.diagnostics() {
                    eprintln!("{}: {diagnostic:?}", result.scenario);
                }
            }
            (result.scenario.to_string(), result.status)
        })
        .collect();
    failed.sort_by(|left, right| left.0.cmp(&right.0));
    failed
}

#[test]
fn adversary_caller_pass2_one_attribute_name_at_two_types_is_sent_at_each_actors_type() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(&two_types_one_name()));
    let mut audits = 0;
    for scenario in synthesis.suite.scenarios.values() {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand {
                command, caller, ..
            } = step
            {
                let Some(value) = caller.get("admin") else {
                    continue;
                };
                if command.to_string() == "demo.notes.Audit" {
                    audits += 1;
                    assert!(
                        matches!(value, Node::Number(_)),
                        "`demo.notes.Auditor` declares `admin` as an Integer, and `Audit` is sent \
                         as a caller whose `admin` is {value:?}"
                    );
                } else {
                    assert!(
                        matches!(value, Node::Bool(_)),
                        "`{command}` is sent with `admin` {value:?}, and its actor declares a Boolean"
                    );
                }
            }
        }
    }
    assert!(audits > 0, "`Audit` is sent at least once");
}

#[test]
fn adversary_caller_pass2_an_identityless_view_on_a_caller_field_passes_a_correct_implementation() {
    let suite = ess_conformance::synthesize::synthesize(&ir(NOTES)).suite;
    assert_eq!(failing(&suite, Mode::Correct), Vec::new());
}

#[test]
fn adversary_caller_pass2_recording_the_input_account_instead_of_the_callers_fails() {
    let suite = ess_conformance::synthesize::synthesize(&ir(&with_account_input())).suite;
    let failed = failing(&suite, Mode::InputAccount);
    assert!(
        failed.iter().any(|(id, _)| id == CREATED),
        "an implementation that records the input's account passes `created`: {failed:#?}"
    );
}

#[test]
fn adversary_caller_pass2_refusing_every_caller_fails_edited() {
    let suite = ess_conformance::synthesize::synthesize(&ir(NOTES)).suite;
    let failed = failing(&suite, Mode::RefusesEveryone);
    assert!(
        failed.iter().any(|(id, _)| id == EDITED),
        "an implementation refusing every edit passes `edited`: {failed:#?}"
    );
}
