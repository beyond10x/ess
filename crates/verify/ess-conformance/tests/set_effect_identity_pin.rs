//! A set effect's filter pins the row's identity (ess/27, beyond10x/ess#547,
//! `docs/design/set-effect-identity-filters.md`, U2).
//!
//! `ExpireEntry` expires a queue entry and ends the one call the entry names by its stored
//! `call_id`: `affects: [{entity: Call, where: call_id == subject.call_id, moves: Call.end}]`. The
//! interpreter reads the candidate's identity among its facts, so it moves exactly the row the
//! subject names, changes nothing for a reference naming no row, and nothing for an `Optional`
//! reference holding nothing. Synthesis reverses the arrangement: the call is created and captured
//! first, and the entry is arranged to name it; a decoy call reads unchanged, a call resting outside
//! the move's `from` states reads unmoved, and a reference naming no call leaves that call absent.
//! An in-memory target that gets the effect wrong in any of those ways fails the branch's scenario.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{
    execute, execute_generating, Externals, Generated, GeneratedSlot, Step, Store,
};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/set-effect-identity-pin.yaml");

const EXPIRED: &str = "demo.q.ExpireEntry/outcome/expired";
const HUNG_UP: &str = "demo.q.HangUp/outcome/hung-up";

fn edited(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds {from:?}");
    text.replacen(from, to, 1)
}

/// The entry's reference is `Optional`, and so is the input that writes it.
fn optional_reference() -> String {
    let text = edited(
        MODEL,
        "      - {name: call_id, type: demo.q.CallId}\n    lifecycle:\n      initial: Queued\n",
        "      - {name: call_id, type: Optional<demo.q.CallId>}\n    lifecycle:\n      initial: Queued\n",
    );
    let text = edited(
        &text,
        "  - name: demo.q.QueueEntry\n    input: [{name: call_id, type: demo.q.CallId}]\n",
        "  - name: demo.q.QueueEntry\n    input: [{name: call_id, type: Optional<demo.q.CallId>}]\n",
    );
    edited(
        &text,
        "      - {name: call_id, type: demo.q.CallId}\n      - {name: state, type: demo.q.Entry.State}\n",
        "      - {name: call_id, type: Optional<demo.q.CallId>}\n      - {name: state, type: demo.q.Entry.State}\n",
    )
}

/// `HangUp` ends every call its filter selects, the one its input names, and reports how many.
fn with_instances() -> String {
    let text = edited(
        MODEL,
        "  - name: demo.q.EntryExpired\n",
        "  - name: demo.q.CallsEnded\n    fields: [{name: ended, type: Integer}]\n  - name: demo.q.EntryExpired\n",
    );
    let text = edited(
        &text,
        "demo.q.ExpireEntry]\n",
        "demo.q.ExpireEntry, demo.q.HangUp]\n",
    );
    edited(
        &text,
        "views:\n",
        "  - name: demo.q.HangUp
    input: [{name: call_id, type: demo.q.CallId}]
    outcomes:
      - name: hung-up
        moves: demo.q.Call.end
        instances: {where: call_id == input.call_id}
        emits: [demo.q.CallsEnded]
        payload: {demo.q.CallsEnded: {ended: {count: changed}}}
views:\n",
    )
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("q.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn synthesis_of(text: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(text))
}

fn refusals(synthesis: &ess_conformance::synthesize::Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .collect()
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; the suite holds:\n{}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            },
            |(_, scenario)| scenario,
        )
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// Every check of `suite` that did not pass on `target`, with what it expected and observed.
fn diagnosis<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .flat_map(|result| result.checks)
        .filter(|check| check.status != Status::Passed)
        .map(|check| {
            let (expected, observed) = check
                .diagnostic
                .map(|diagnostic| (diagnostic.expected, diagnostic.observed))
                .unwrap_or_default();
            format!(
                "{}: expected {expected:?}, observed {observed:?}",
                check.about
            )
        })
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

// ---- the interpreter, driven directly ----------------------------------------------------------

fn text(value: &str) -> Node {
    Node::Text(value.into())
}

fn created(
    ir: &EssIr,
    store: &mut Store,
    command: &str,
    input: &[(&str, Node)],
    slot: (&str, &str, &str),
) {
    let (event, field, id) = slot;
    let mut steps = execute_generating(
        ir,
        store,
        &command.parse().unwrap(),
        &input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect(),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::from([(
            GeneratedSlot::new(event.parse().unwrap(), field),
            text(id),
        )])),
    )
    .unwrap_or_else(|error| panic!("{command}: {error:?}"));
    assert_eq!(steps.len(), 1);
    *store = steps.remove(0).next;
}

fn call(ir: &EssIr, store: &mut Store, id: &str) {
    created(
        ir,
        store,
        "demo.q.StartCall",
        &[("line", text("l"))],
        ("demo.q.CallStarted", "call_id", id),
    );
}

fn entry(ir: &EssIr, store: &mut Store, id: &str, call: Option<&str>) {
    let input: Vec<(&str, Node)> = call
        .map(|call| ("call_id", text(call)))
        .into_iter()
        .collect();
    created(
        ir,
        store,
        "demo.q.QueueEntry",
        &input,
        ("demo.q.EntryQueued", "entry_id", id),
    );
}

fn expire(ir: &EssIr, store: &mut Store, id: &str) -> Step {
    let mut steps = execute(
        ir,
        store,
        &"demo.q.ExpireEntry".parse().unwrap(),
        &BTreeMap::from([("entry_id".to_owned(), text(id))]),
        &Externals::Withheld,
    )
    .unwrap_or_else(|error| panic!("{error:?}"));
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    *store = step.next.clone();
    step
}

fn outcome(step: &Step) -> String {
    step.outcome
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default()
}

fn state(store: &Store, entity: &str, id: &str) -> Option<String> {
    store
        .instance(&entity.parse().unwrap(), id)
        .map(|instance| instance.state.to_string())
}

fn calls(store: &Store) -> usize {
    store
        .instances()
        .filter(|(entity, _, _)| entity.to_string() == "demo.q.Call")
        .count()
}

#[test]
fn the_interpreter_moves_only_the_row_the_subject_names() {
    let ir = ir_of(MODEL);
    let mut store = Store::default();
    call(&ir, &mut store, "named");
    call(&ir, &mut store, "decoy");
    entry(&ir, &mut store, "e1", Some("named"));
    let step = expire(&ir, &mut store, "e1");
    assert_eq!(outcome(&step), "demo.q.ExpireEntry/expired");
    assert_eq!(
        state(&store, "demo.q.Entry", "e1").as_deref(),
        Some("Expired")
    );
    assert_eq!(
        state(&store, "demo.q.Call", "named").as_deref(),
        Some("Ended")
    );
    assert_eq!(
        state(&store, "demo.q.Call", "decoy").as_deref(),
        Some("Waiting")
    );
}

#[test]
fn a_reference_naming_no_row_changes_nothing() {
    let ir = ir_of(MODEL);
    let mut store = Store::default();
    call(&ir, &mut store, "decoy");
    entry(&ir, &mut store, "e1", Some("gone"));
    let step = expire(&ir, &mut store, "e1");
    assert_eq!(outcome(&step), "demo.q.ExpireEntry/expired");
    assert_eq!(
        state(&store, "demo.q.Call", "decoy").as_deref(),
        Some("Waiting")
    );
    assert_eq!(
        state(&store, "demo.q.Call", "gone"),
        None,
        "no row is created"
    );
    assert_eq!(calls(&store), 1);
}

#[test]
fn an_absent_optional_reference_changes_nothing() {
    let ir = ir_of(&optional_reference());
    let mut store = Store::default();
    call(&ir, &mut store, "decoy");
    entry(&ir, &mut store, "e1", None);
    let step = expire(&ir, &mut store, "e1");
    assert_eq!(outcome(&step), "demo.q.ExpireEntry/expired");
    assert_eq!(
        state(&store, "demo.q.Entry", "e1").as_deref(),
        Some("Expired")
    );
    assert_eq!(
        state(&store, "demo.q.Call", "decoy").as_deref(),
        Some("Waiting")
    );
    assert_eq!(calls(&store), 1);
}

// ---- a target, and the ways it can get the effect wrong ----------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// Ends every `Waiting` call, whichever the entry names.
    MovesEveryRow,
    /// Expires the entry and leaves every call as it was.
    MovesNoRow,
    /// Ends a call for a reference naming none, by creating it.
    CreatesTheDanglingRow,
    /// Ends the named call even when it is `Parked`.
    IgnoresFrom,
    /// `HangUp` reports one call ended, however many it ended.
    CountsOne,
}

#[derive(Clone, Debug)]
struct Call {
    id: String,
    line: String,
    state: &'static str,
}

#[derive(Clone, Debug)]
struct Entry {
    id: String,
    call_id: Option<String>,
    state: &'static str,
}

struct Desk {
    mode: Mode,
    calls: RefCell<Vec<Call>>,
    entries: RefCell<Vec<Entry>>,
    next: RefCell<usize>,
}

fn input_text(input: &BTreeMap<String, Node>, field: &str) -> Option<String> {
    input.get(field).and_then(Node::as_text).map(str::to_owned)
}

fn took(
    command: &CommandRef,
    outcome: &str,
    event: &str,
    payload: Vec<(&str, Node)>,
) -> SemanticCommandResult {
    let mut result =
        SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
    let mut observed = ObservedEvent::new(event.parse().unwrap());
    for (key, value) in payload {
        observed.payload.insert(key.to_owned(), value);
    }
    result.direct_events.push(observed);
    result
}

fn refused(command: &CommandRef, outcome: &str, error: &str) -> SemanticCommandResult {
    let mut result =
        SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
    result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
    result
}

impl Desk {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            calls: RefCell::default(),
            entries: RefCell::default(),
            next: RefCell::new(0),
        }
    }

    fn fresh(&self, prefix: &str) -> String {
        let mut next = self.next.borrow_mut();
        *next += 1;
        format!("{prefix}-{next}")
    }

    /// Ends the calls `named` selects, as the mode gets it right or wrong; how many it ended.
    fn end(&self, named: Option<&str>) -> usize {
        let mode = self.mode;
        let mut calls = self.calls.borrow_mut();
        if mode == Mode::MovesNoRow {
            return 0;
        }
        let Some(named) = named else {
            return 0;
        };
        if mode == Mode::CreatesTheDanglingRow && !calls.iter().any(|call| call.id == named) {
            calls.push(Call {
                id: named.to_owned(),
                line: String::new(),
                state: "Ended",
            });
            return 1;
        }
        let mut ended = 0;
        for call in calls.iter_mut() {
            let selected = mode == Mode::MovesEveryRow || call.id == named;
            let movable =
                call.state == "Waiting" || (mode == Mode::IgnoresFrom && call.state == "Parked");
            if selected && movable {
                call.state = "Ended";
                ended += 1;
            }
        }
        ended
    }
}

impl ConformanceTarget for Desk {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("set-effect-identity-pin", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.calls.borrow_mut().clear();
        self.entries.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let input = &request.input;
        Ok(match command.to_string().as_str() {
            "demo.q.StartCall" => {
                let id = self.fresh("call");
                self.calls.borrow_mut().push(Call {
                    id: id.clone(),
                    line: input_text(input, "line").unwrap_or_default(),
                    state: "Waiting",
                });
                took(
                    &command,
                    "started",
                    "demo.q.CallStarted",
                    vec![("call_id", Node::Text(id))],
                )
            }
            "demo.q.ParkCall" => {
                let id = input_text(input, "call_id").unwrap_or_default();
                let mut calls = self.calls.borrow_mut();
                let Some(call) = calls
                    .iter_mut()
                    .find(|call| call.id == id)
                    .filter(|call| call.state == "Waiting")
                else {
                    return Ok(refused(&command, "not-waiting", "demo.q.NotWaiting"));
                };
                call.state = "Parked";
                took(
                    &command,
                    "parked",
                    "demo.q.CallParked",
                    vec![("call_id", Node::Text(id))],
                )
            }
            "demo.q.QueueEntry" => {
                let id = self.fresh("entry");
                self.entries.borrow_mut().push(Entry {
                    id: id.clone(),
                    call_id: input_text(input, "call_id"),
                    state: "Queued",
                });
                took(
                    &command,
                    "queued",
                    "demo.q.EntryQueued",
                    vec![("entry_id", Node::Text(id))],
                )
            }
            "demo.q.ExpireEntry" => {
                let id = input_text(input, "entry_id").unwrap_or_default();
                let named = {
                    let mut entries = self.entries.borrow_mut();
                    let Some(entry) = entries
                        .iter_mut()
                        .find(|entry| entry.id == id)
                        .filter(|entry| entry.state == "Queued")
                    else {
                        return Ok(refused(&command, "not-queued", "demo.q.NotQueued"));
                    };
                    entry.state = "Expired";
                    entry.call_id.clone()
                };
                self.end(named.as_deref());
                took(
                    &command,
                    "expired",
                    "demo.q.EntryExpired",
                    vec![("entry_id", Node::Text(id))],
                )
            }
            "demo.q.HangUp" => {
                let named = input_text(input, "call_id");
                let ended = self.end(named.as_deref());
                let reported = if self.mode == Mode::CountsOne {
                    1
                } else {
                    ended
                };
                took(
                    &command,
                    "hung-up",
                    "demo.q.CallsEnded",
                    vec![(
                        "ended",
                        Node::Number(Number::from(i64::try_from(reported).unwrap())),
                    )],
                )
            }
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows = match request.view.to_string().as_str() {
            "demo.q.Calls" => self
                .calls
                .borrow()
                .iter()
                .map(|call| {
                    BTreeMap::from([
                        ("call_id".to_owned(), Node::Text(call.id.clone())),
                        ("line".to_owned(), Node::Text(call.line.clone())),
                        ("state".to_owned(), Node::Text(call.state.to_owned())),
                    ])
                })
                .collect(),
            "demo.q.Entries" => self
                .entries
                .borrow()
                .iter()
                .map(|entry| {
                    let mut row = BTreeMap::from([
                        ("entry_id".to_owned(), Node::Text(entry.id.clone())),
                        ("state".to_owned(), Node::Text(entry.state.to_owned())),
                    ]);
                    // An `Optional` reference holding nothing is published as `null`.
                    row.insert(
                        "call_id".to_owned(),
                        entry.call_id.clone().map_or(Node::Null, Node::Text),
                    );
                    row
                })
                .collect(),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult { rows, total: None })
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "the model declares none",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
}

// ---- synthesis ---------------------------------------------------------------------------------

/// How many steps of `scenario` send `command`.
fn sends(scenario: &ConformanceScenario, command: &str) -> usize {
    scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command: sent, .. }
                if sent.to_string() == command)
        })
        .count()
}

/// The reads of `view` in `scenario` requiring a row absent.
fn absences(scenario: &ConformanceScenario, view: &str) -> usize {
    scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExpectSubjectAbsent { view: read, .. }
                if read.to_string() == view)
        })
        .count()
}

/// The `{count: changed}` values `scenario` requires of `event`, in order.
fn counts(scenario: &ConformanceScenario, event: &str, field: &str) -> Vec<Node> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectEvent {
                event: expected,
                payload,
                ..
            } if expected.to_string() == event => payload.get(field).cloned(),
            _ => None,
        })
        .collect()
}

#[test]
fn synthesis_witnesses_a_pinned_affects_entry() {
    let ir = ir_of(MODEL);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let about = refusals(&synthesis);
    assert!(
        about
            .iter()
            .all(|refusal| !refusal.contains("ExpireEntry/expired")
                && !refusal.contains("expired.affects")),
        "nothing about the pinned branch is refused: {about:#?}"
    );
    let expired = scenario(&synthesis.suite, EXPIRED);
    // The main send, the call resting outside `end`'s `from`, and the reference naming no call.
    assert!(
        sends(expired, "demo.q.ExpireEntry") >= 3,
        "{:#?}",
        expired.steps
    );
    assert!(
        sends(expired, "demo.q.ParkCall") >= 1,
        "a named call rests outside the move's `from` states: {:#?}",
        expired.steps
    );
    assert!(
        absences(expired, "demo.q.Calls") >= 1,
        "the call a dangling reference names is read absent: {:#?}",
        expired.steps
    );
    let statuses = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir),
    );
    assert_eq!(
        statuses.get(EXPIRED),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
    assert_eq!(not_passed(&statuses), Vec::<&str>::new(), "{statuses:#?}");
    let statuses = run(&synthesis.suite, &Desk::new(Mode::Correct));
    assert_eq!(not_passed(&statuses), Vec::<&str>::new(), "{statuses:#?}");
}

#[test]
fn a_target_getting_the_pinned_effect_wrong_fails_its_branch() {
    let synthesis = synthesis_of(MODEL);
    for mode in [
        Mode::MovesEveryRow,
        Mode::MovesNoRow,
        Mode::CreatesTheDanglingRow,
        Mode::IgnoresFrom,
    ] {
        let statuses = run(&synthesis.suite, &Desk::new(mode));
        assert_eq!(
            statuses.get(EXPIRED),
            Some(&Status::Failed),
            "{mode:?}: {statuses:#?}"
        );
    }
}

#[test]
fn an_absent_optional_reference_is_witnessed() {
    let text = optional_reference();
    let ir = ir_of(&text);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let expired = scenario(&synthesis.suite, EXPIRED);
    let absent = expired.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExecuteCommand { command, input, .. }
            if command.to_string() == "demo.q.QueueEntry" && !input.contains_key("call_id"))
    });
    assert!(
        absent,
        "an entry holding no reference is arranged: {:#?}",
        expired.steps
    );
    assert!(
        absences(expired, "demo.q.Calls") >= 1,
        "a present reference naming no call is sent too: {:#?}",
        expired.steps
    );
    let interpreted = ess_conformance::interpret::Interpreted::for_model(ir);
    assert_eq!(
        diagnosis(&synthesis.suite, &interpreted),
        Vec::<String>::new(),
        "{:#?}",
        expired.steps
    );
    assert_eq!(
        diagnosis(&synthesis.suite, &Desk::new(Mode::Correct)),
        Vec::<String>::new()
    );
    let statuses = run(&synthesis.suite, &Desk::new(Mode::MovesEveryRow));
    assert_eq!(
        statuses.get(EXPIRED),
        Some(&Status::Failed),
        "{statuses:#?}"
    );
}

#[test]
fn a_pinned_instances_filter_counts_one_then_zero() {
    let text = with_instances();
    let ir = ir_of(&text);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let about = refusals(&synthesis);
    assert!(
        about.iter().all(|refusal| !refusal.contains("HangUp")),
        "{about:#?}"
    );
    let hung_up = scenario(&synthesis.suite, HUNG_UP);
    // The send naming the target counts it; every later send (a call resting outside `end`'s
    // `from`, an identity no call carries) names a row the effect leaves, and counts none.
    let counted = counts(hung_up, "demo.q.CallsEnded", "ended");
    let (first, later) = counted.split_first().expect("a count per send");
    assert_eq!(
        first,
        &Node::Number(Number::from(1_i64)),
        "{:#?}",
        hung_up.steps
    );
    assert!(
        !later.is_empty()
            && later
                .iter()
                .all(|count| *count == Node::Number(Number::from(0_i64))),
        "{:#?}",
        hung_up.steps
    );
    let statuses = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir),
    );
    assert_eq!(not_passed(&statuses), Vec::<&str>::new(), "{statuses:#?}");
    let statuses = run(&synthesis.suite, &Desk::new(Mode::Correct));
    assert_eq!(not_passed(&statuses), Vec::<&str>::new(), "{statuses:#?}");
    for mode in [
        Mode::CountsOne,
        Mode::MovesEveryRow,
        Mode::MovesNoRow,
        Mode::CreatesTheDanglingRow,
    ] {
        let statuses = run(&synthesis.suite, &Desk::new(mode));
        assert_eq!(
            statuses.get(HUNG_UP),
            Some(&Status::Failed),
            "{mode:?}: {statuses:#?}"
        );
    }
}

#[test]
fn an_identity_read_other_than_by_a_pin_is_a_named_gap() {
    let text = edited(
        MODEL,
        "where: call_id == subject.call_id",
        "where: call_id != subject.call_id",
    );
    let synthesis = synthesis_of(&text);
    let about = refusals(&synthesis);
    assert!(
        about.iter().any(|refusal| refusal.contains("ExpireEntry")
            && refusal.contains("NoWitness")
            && refusal.contains("identity")),
        "{about:#?}"
    );
    assert!(
        !synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == EXPIRED),
        "no scenario is filed without the witness"
    );
}

#[test]
fn a_subject_field_no_command_writes_is_a_named_gap() {
    let text = edited(
        &optional_reference(),
        "        sets: {call_id: input.call_id}\n",
        "",
    );
    let synthesis = synthesis_of(&text);
    let about = refusals(&synthesis);
    assert!(
        about.iter().any(|refusal| refusal.contains("ExpireEntry")
            && refusal.contains("NoWitness")
            && refusal.contains("subject.call_id")
            && refusal.contains("written from an input by no command")),
        "{about:#?}"
    );
    assert!(
        !synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == EXPIRED),
        "no scenario is filed without the witness"
    );
}
