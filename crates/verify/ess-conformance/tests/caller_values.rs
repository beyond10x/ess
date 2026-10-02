//! The authenticated caller as a value source and a guard operand (source format `ess/16`,
//! beyond10x/ess#168, `docs/design/caller-values.md`): the suite says which caller sends every
//! command (suite/26), requires the refusal for a caller who is not the note's agent, and asserts
//! that a note records its creator's account — each with the two callers' roles swapped too, so an
//! implementation that answers one caller by name fails.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{EssIr, ResolvedPayloadValue};
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
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
const EDITED: &str = "demo.notes.EditNote/outcome/edited";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(text: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// Every command the scenario sends, in order, with the caller it is sent as.
fn sent(scenario: &ConformanceScenario) -> Vec<(String, BTreeMap<String, Node>)> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command, caller, ..
            } => Some((command.to_string(), caller.clone())),
            _ => None,
        })
        .collect()
}

/// [`sent`], without an edit of a note nobody created: the probe a stored-field guard sends first,
/// for an identity no row carries.
fn on_notes(scenario: &ConformanceScenario) -> Vec<(String, BTreeMap<String, Node>)> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command,
                caller,
                input,
                ..
            } if !matches!(
                input.get("note_id"),
                Some(ess_conformance::ScenarioValue::Literal { .. })
            ) =>
            {
                Some((command.to_string(), caller.clone()))
            }
            _ => None,
        })
        .collect()
}

/// The `account_id` every `NoteCreated` expectation of the scenario asserts, in order.
fn asserted_accounts(scenario: &ConformanceScenario) -> Vec<Node> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.notes.NoteCreated" =>
            {
                Some(
                    payload
                        .get("account_id")
                        .cloned()
                        .expect("the account is asserted"),
                )
            }
            _ => None,
        })
        .collect()
}

#[test]
fn issue_168_the_compiled_source_reads_the_callers_attribute() {
    let ir = ir(NOTES);
    let actor = ir.actors().values().next().expect("one actor");
    let names: Vec<&str> = actor
        .attributes
        .iter()
        .map(|field| field.name.as_str())
        .collect();
    assert_eq!(names, ["account_id", "agent_id"]);
    let create = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.notes.CreateNote")
        .expect("CreateNote compiles");
    let account = create.outcomes[0].payload[0]
        .fields
        .iter()
        .find(|field| field.target == "account_id")
        .expect("account_id is determined");
    assert!(
        matches!(
            &account.value,
            ResolvedPayloadValue::CallerAttribute { attribute, .. } if attribute == "account_id"
        ),
        "{:?}",
        account.value
    );
}

#[test]
fn issue_168_every_command_is_sent_as_a_caller_and_the_suite_takes_suite_26() {
    let suite = suite(NOTES);
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/26"
    );
    let mut callers = BTreeSet::new();
    for scenario in suite.scenarios.values() {
        for (command, caller) in sent(scenario) {
            assert_eq!(
                caller.keys().map(String::as_str).collect::<Vec<_>>(),
                ["account_id", "agent_id"],
                "{command} is sent as a caller carrying every attribute its actor declares"
            );
            callers.insert(caller);
        }
    }
    assert_eq!(callers.len(), 2, "two callers: {callers:#?}");
}

#[test]
fn issue_168_the_refusal_is_sent_by_a_caller_who_is_not_the_notes_agent() {
    let suite = suite(NOTES);
    let forbidden = on_notes(scenario(&suite, FORBIDDEN));
    // Each half: one caller creates the note, the other edits it; then the roles swap.
    assert_eq!(forbidden.len(), 4, "{forbidden:#?}");
    for half in forbidden.chunks(2) {
        let [(create, creator), (edit, editor)] = half else {
            unreachable!()
        };
        assert_eq!(create, "demo.notes.CreateNote");
        assert_eq!(edit, "demo.notes.EditNote");
        assert_ne!(creator["agent_id"], editor["agent_id"], "{half:#?}");
    }
    assert_eq!(forbidden[0].1, forbidden[3].1, "the roles swap");
    assert_eq!(forbidden[1].1, forbidden[2].1, "the roles swap");

    let edited = on_notes(scenario(&suite, EDITED));
    assert_eq!(edited.len(), 4, "{edited:#?}");
    for half in edited.chunks(2) {
        assert_eq!(
            half[0].1, half[1].1,
            "the agent edits its own note: {half:#?}"
        );
    }
    assert_ne!(
        edited[0].1, edited[2].1,
        "each caller edits its own note once"
    );
}

#[test]
fn issue_168_a_note_records_the_account_of_the_caller_that_created_it() {
    let suite = suite(NOTES);
    let created = scenario(&suite, CREATED);
    let sent = sent(created);
    let accounts = asserted_accounts(created);
    assert_eq!(sent.len(), 2, "created once by each caller: {sent:#?}");
    assert_eq!(accounts.len(), 2, "{accounts:#?}");
    for ((_, caller), account) in sent.iter().zip(&accounts) {
        assert_eq!(
            &caller["account_id"], account,
            "the event carries the caller's account"
        );
    }
    assert_ne!(accounts[0], accounts[1]);
}

/// [`NOTES`] with no actor attribute and no caller read: the same model shape, sent by nobody in
/// particular.
fn without_callers() -> String {
    NOTES
        .replace(
            "    attributes:\n      - {name: account_id, type: demo.notes.AccountId}\n      - {name: agent_id, type: demo.notes.AgentId}\n",
            "",
        )
        .replace("{caller: account_id}", "{generated: true}")
        .replace("{caller: agent_id}", "{generated: true}")
        .replace(
            "        when_subject: {predicate: agent_id != caller.agent_id}\n",
            "        when: text == \"\"\n",
        )
}

/// beyond10x/ess#216: the suite records the digest of the model it was synthesized from — the one
/// `--target interpreted` and every adapter compute — not of the caller reading synthesis writes
/// each caller's values into. A model without caller reads keeps the digest it always had.
#[test]
fn issue_216_the_suite_records_the_digest_of_the_model_not_of_a_caller_reading() {
    for text in [NOTES.to_owned(), without_callers()] {
        let model = ir(&text);
        let synthesized = ess_conformance::synthesize::synthesize(&model).suite;
        let expected = ess_conformance::SuiteProvenance::of(&model);
        assert_eq!(
            synthesized.provenance.spec_digest, expected.spec_digest,
            "the suite's spec_digest is the model's"
        );
        assert_eq!(
            synthesized.provenance.contract_digest, expected.contract_digest,
            "the suite's contract_digest is the model's"
        );
    }
}

#[test]
fn a_suite_whose_actors_declare_no_attributes_sends_no_caller() {
    let text = without_callers();
    let suite = suite(&text);
    for scenario in suite.scenarios.values() {
        for (command, caller) in sent(scenario) {
            assert!(
                caller.is_empty(),
                "{command} is sent as nobody in particular"
            );
        }
    }
    assert_ne!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/26"
    );
}

#[test]
fn a_caller_in_a_suite_pinned_below_suite_26_is_refused() {
    let mut suite = suite(NOTES);
    suite.provenance.suite_version =
        ess_conformance::scenario::SuiteFormat::parse("ess-conformance/25").unwrap();
    let error = AdmittedSuite::from_suite(&suite).expect_err("suite/25 has no caller");
    assert!(error.to_string().contains("suite/26"), "{error}");
}

// ---- running the suite ------------------------------------------------------------------------

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

/// Every scenario that did not pass against `mode`, by id, with its status.
fn failing(suite: &ConformanceSuite, mode: Mode) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let mut failed: Vec<(String, Status)> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Notes::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    failed.sort_by(|left, right| left.0.cmp(&right.0));
    failed
}

fn ids(failed: &[(String, Status)]) -> Vec<&str> {
    failed.iter().map(|(id, _)| id.as_str()).collect()
}

#[test]
fn the_interpreter_executes_the_actual_caller_roles_and_typed_boolean_roles() {
    for source in [
        NOTES.to_owned(),
        NOTES
            .replace("kind: newtype, of: Uuid", "kind: newtype, of: Boolean")
            .replace(
                "name: demo.notes.NoteId, kind: newtype, of: Boolean",
                "name: demo.notes.NoteId, kind: newtype, of: Uuid",
            ),
    ] {
        let suite = suite(&source);
        let admitted = AdmittedSuite::from_suite(&suite).unwrap();
        let target = ess_conformance::interpret::Interpreted::for_model(ir(&source));
        let report = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &target)
            .into_report();
        assert_eq!(report.scenarios.len(), 3);
        assert!(
            report
                .scenarios
                .iter()
                .all(|result| result.status == Status::Passed),
            "{:#?}",
            report.scenarios
        );
    }
}

#[test]
fn issue_168_the_suite_passes_an_implementation_that_reads_the_caller() {
    assert_eq!(failing(&suite(NOTES), Mode::Correct), Vec::new());
}

#[test]
fn issue_168_an_implementation_recording_one_fixed_account_fails() {
    let failed = failing(&suite(NOTES), Mode::FirstAccountEver);
    assert!(ids(&failed).contains(&CREATED), "{failed:#?}");
}

#[test]
fn issue_168_an_implementation_that_lets_anyone_edit_fails_the_refusal() {
    let failed = failing(&suite(NOTES), Mode::AnyoneEdits);
    assert_eq!(ids(&failed), [FORBIDDEN], "{failed:#?}");
}

#[test]
fn issue_168_an_implementation_that_answers_one_caller_by_name_fails() {
    let failed = failing(&suite(NOTES), Mode::OnlyTheFirstCallerEdits);
    let failed = ids(&failed);
    assert!(
        failed.contains(&FORBIDDEN) || failed.contains(&EDITED),
        "{failed:#?}"
    );
}

#[test]
fn a_target_that_cannot_send_as_a_caller_is_unsupported_not_passed() {
    let failed = failing(&suite(NOTES), Mode::CannotAuthenticate);
    assert!(!failed.is_empty());
    assert!(
        failed
            .iter()
            .all(|(_, status)| *status == Status::Unsupported),
        "{failed:#?}"
    );
}

#[test]
fn a_when_guard_comparing_the_caller_with_an_input_is_witnessed_on_both_sides() {
    let text = NOTES
        .replace(
            "      - {name: note_id, type: demo.notes.NoteId}\n      - {name: text, type: String}\n    outcomes:",
            "      - {name: note_id, type: demo.notes.NoteId}\n      - {name: text, type: String}\n      - {name: account, type: demo.notes.AccountId}\n    outcomes:",
        )
        .replace(
            "        when_subject: {predicate: agent_id != caller.agent_id}\n",
            "        when: account != caller.account_id\n",
        );
    let suite = suite(&text);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir(&text)),
        )
        .into_report();
    assert!(
        report
            .scenarios
            .iter()
            .all(|result| result.status == Status::Passed),
        "{:#?}",
        report.scenarios
    );
    for (id, same) in [(FORBIDDEN, false), (EDITED, true)] {
        let edits: Vec<_> = scenario(&suite, id)
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExecuteCommand {
                    command,
                    caller,
                    input,
                    ..
                } if command.to_string() == "demo.notes.EditNote" => Some((caller, input)),
                _ => None,
            })
            .collect();
        assert!(!edits.is_empty(), "{id}");
        for (caller, input) in edits {
            let Some(ess_conformance::ScenarioValue::Literal { value }) = input.get("account")
            else {
                panic!("{id}: the account is sent as a literal: {input:?}")
            };
            assert_eq!(
                value == &caller["account_id"],
                same,
                "{id}: {input:?} as {caller:?}"
            );
        }
    }
}
