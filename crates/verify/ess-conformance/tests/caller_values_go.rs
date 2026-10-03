//! The generated Go runtime sends every command as the caller the suite names (suite/26,
//! `caller`, beyond10x/ess#168) and gives the reference verdicts (beyond10x/ess#188).
//!
//! The target is `tests/caller_values.rs`'s, recorded once and replayed to the Go runtime, which
//! must also send each command with the recorded caller: a runtime that dropped `caller` would send
//! every command as nobody in particular, and the replay reports it as a divergence.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;
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

const CREATED: &str = "demo.notes.CreateNote/outcome/created";
const FORBIDDEN: &str = "demo.notes.EditNote/outcome/forbidden";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(text: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    synthesis.suite
}

#[test]
fn go_gives_the_reference_verdict_for_every_caller_mode() {
    let suite = suite(NOTES);
    for mode in [
        Mode::Correct,
        Mode::FirstAccountEver,
        Mode::AnyoneEdits,
        Mode::OnlyTheFirstCallerEdits,
        Mode::CannotAuthenticate,
    ] {
        let verdicts = support_go::assert_parity(
            &format!("caller-{mode:?}").to_lowercase(),
            &suite,
            Notes::new(mode),
        );
        let wrong = support_go::not_passed(&verdicts);
        match mode {
            Mode::Correct => assert!(wrong.is_empty(), "{verdicts:?}"),
            Mode::FirstAccountEver => assert!(wrong.contains(&CREATED), "{verdicts:?}"),
            Mode::AnyoneEdits => assert_eq!(wrong, [FORBIDDEN], "{verdicts:?}"),
            Mode::OnlyTheFirstCallerEdits => assert!(!wrong.is_empty(), "{verdicts:?}"),
            Mode::CannotAuthenticate => assert!(
                !wrong.is_empty() && wrong.iter().all(|id| verdicts[*id] == "unsupported"),
                "{verdicts:?}"
            ),
        }
    }
}

/// A caller dropped on the way to the target is visible: the replay compares every request.
#[test]
fn a_runtime_that_drops_the_caller_is_seen_by_the_replay() {
    let original = suite(NOTES);
    let mut dropped = original.clone();
    for scenario in dropped.scenarios.values_mut() {
        for step in &mut scenario.steps {
            if let ess_conformance::ScenarioStep::ExecuteCommand { caller, .. } = step {
                caller.clear();
            }
        }
    }
    let recording = support_go::Recorder::new(Notes::new(Mode::Correct));
    let _ = support_go::rust_outcomes(&original, &recording);
    let directory =
        support_go::package("caller-dropped", &dropped, &[support_go::TRANSCRIPT_TARGET]);
    let replayed = support_go::replay(&directory, &recording, &[]);
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        replayed
            .divergences
            .iter()
            .any(|line| line.contains("\"caller\":null")),
        "{:?}",
        replayed.divergences
    );
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Records the caller's account and agent, and refuses an edit by anyone but the agent.
    Correct,
    /// Records the account of the first caller this process ever served, whoever sends.
    FirstAccountEver,
    /// Never refuses an edit.
    AnyoneEdits,
    /// Refuses every edit by anyone but the first caller this process ever served.
    OnlyTheFirstCallerEdits,
    /// Cannot send a command as a particular caller.
    CannotAuthenticate,
}

#[derive(Default)]
struct World {
    notes: BTreeMap<String, BTreeMap<String, Node>>,
}

struct Notes {
    mode: Mode,
    world: RefCell<World>,
    minted: Cell<u32>,
    first: RefCell<Option<BTreeMap<String, Node>>>,
}

impl Notes {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            world: RefCell::default(),
            minted: Cell::new(0),
            first: RefCell::default(),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Notes {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("notes-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.world.replace(World::default());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let Some(caller) = request.caller.clone() else {
            return Err(TargetError::unavailable(
                "authenticating",
                "every command of this system is sent as a caller",
            ));
        };
        if self.mode == Mode::CannotAuthenticate {
            return Err(TargetError::unsupported(
                "sending a command as a caller",
                "this target holds one credential",
            ));
        }
        let first = self
            .first
            .borrow_mut()
            .get_or_insert(caller.clone())
            .clone();
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let mut world = self.world.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.notes.CreateNote" => {
                let id = format!("00000000-0000-4000-8000-{n:012}");
                let account = match self.mode {
                    Mode::FirstAccountEver => first["account_id"].clone(),
                    _ => caller["account_id"].clone(),
                };
                world.notes.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("note_id".to_owned(), Node::Text(id.clone())),
                        ("account_id".to_owned(), account.clone()),
                        ("agent_id".to_owned(), caller["agent_id"].clone()),
                        ("text".to_owned(), request.input["text"].clone()),
                        ("state".to_owned(), Node::Text("Open".to_owned())),
                    ]),
                );
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
                let Some(row) = world.notes.get_mut(&id) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let refused = match self.mode {
                    Mode::AnyoneEdits => false,
                    Mode::OnlyTheFirstCallerEdits => caller != first,
                    _ => row["agent_id"] != caller["agent_id"],
                };
                if refused {
                    SemanticCommandResult::took(outcome(&command, "forbidden")).with_error(
                        DeclaredErrorValue::new("demo.notes.NotYourNote".parse().unwrap()),
                    )
                } else {
                    row.insert("text".to_owned(), request.input["text"].clone());
                    SemanticCommandResult::took(outcome(&command, "edited")).emitting(
                        ObservedEvent::new("demo.notes.NoteEdited".parse().unwrap())
                            .with("note_id", Node::Text(id))
                            .with("text", request.input["text"].clone()),
                    )
                }
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.world.borrow().notes.values().cloned(),
        ))
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
