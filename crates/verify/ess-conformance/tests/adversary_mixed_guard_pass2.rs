//! Adversary, pass 2, `story:a-wrong-state-witness-may-miss-a-mixed-guard-sibling-by-its-subject-guard`
//! (beyond10x/ess#192).
//!
//! Each case runs the synthesized wrong-state `Confirm` scenarios against a hand-written `Orders`
//! target that answers the way Entity Runtime answers: branches in lowering order (input-guarded
//! refusals, the other guarded branches in declaration order, the default, `wrong_state` last —
//! `ess-entity-runtime` `lib.rs:1450`), then entity-core `select_outcome` / `admit_state`:
//! a guard reading an absent stored field is `Unknown` and stops selection
//! (`OutcomeUnobservable`), no selected branch is `NoOutcomeSelected`, and a selected move from a
//! state no move leaves is answered by the `wrong_state` branch.
//!
//! Attacked: the moving branch's own subject guard (the witness never asks the row to satisfy it)
//! and the exemption for a moving sibling (`answered_by_state`) when its stored guard is
//! `Unknown` on the row.
//!
//! Red on its base and kept outside the repository until
//! `story:wrong-state-witness-unknown-and-own-stored-guards`, which ports it as that story's
//! acceptance. Changed in the port: the third case's assertion that some scenario ran (no
//! synthesis could satisfy it on that model; see the case), each case's check that every wrong
//! state is written or refused naming the guard, and a fourth case for a row that can decide the
//! guard.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const PREFIX: &str = "demo.orders.Order/state/";
const SUFFIX: &str = "/refuses/demo.orders.Confirm";

/// Entity, errors, events, `Place` and `Close`. `Confirm`'s outcomes are appended per case.
const HEAD: &str = r"format: ess/16
system: demo
version: v1
domain: demo.orders
summary: An order confirmed with a token, and gone once it has left Open.
types:
  - {name: demo.orders.OrderId, kind: newtype, of: Uuid}
  - {name: demo.orders.History, kind: enum, variants: VARIANTS}
entities:
  - name: demo.orders.Order
    identity: {name: id, type: demo.orders.OrderId}
    fields:
      - {name: history, type: demo.orders.History}
      - {name: note, type: Optional<String>}
    lifecycle:
      initial: Open
      states: [Open, Rushed, Done, Closed]
      terminal: [Closed]
      transitions:
        - {name: confirm, from: [Open], to: Done}
        - {name: rush, from: [Open], to: Rushed}
        - {name: close, from: [Open, Rushed, Done], to: Closed}
actors:
  - name: demo.orders.Clerk
    may: [demo.orders.Place, demo.orders.Confirm, demo.orders.Close, demo.orders.Rush]
errors:
  - {name: demo.orders.TokenRequired, summary: The token is empty., fields: []}
  - {name: demo.orders.NoSuchOrder, summary: The order has left Open., fields: []}
events:
  - name: demo.orders.Placed
    fields: [{name: id, type: demo.orders.OrderId}]
  - name: demo.orders.Confirmed
    fields: [{name: id, type: demo.orders.OrderId}]
  - name: demo.orders.Closed
    fields: [{name: id, type: demo.orders.OrderId}]
  - name: demo.orders.Rushed
    fields: [{name: id, type: demo.orders.OrderId}]
views:
  - name: demo.orders.Orders
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.orders.OrderId}
      - {name: history, type: demo.orders.History}
      - {name: note, type: Optional<String>}
      - {name: state, type: demo.orders.Order.State}
commands:
  - name: demo.orders.Place
    input:
      - {name: history, type: demo.orders.History}
    outcomes:
      - name: placed
        creates: demo.orders.Order
        instance: id
        sets: {history: input.history}
        emits: [demo.orders.Placed]
        payload: {demo.orders.Placed: {id: {generated: true}}}
  - name: demo.orders.Close
    input: [{name: id, type: demo.orders.OrderId}]
    outcomes:
      - name: closed
        moves: demo.orders.Order.close
        instance: id
        emits: [demo.orders.Closed]
        payload: {demo.orders.Closed: {id: input.id}}
  - name: demo.orders.Rush
    input: [{name: id, type: demo.orders.OrderId}]
    outcomes:
      - name: rushed
        moves: demo.orders.Order.rush
        instance: id
        emits: [demo.orders.Rushed]
        payload: {demo.orders.Rushed: {id: input.id}}
  - name: demo.orders.Confirm
    input:
      - {name: id, type: demo.orders.OrderId}
      - {name: token, type: String}
    outcomes:
";

/// The moving branch needs a token *and* a `Pending` row: a mixed guard on the attempt itself.
const CONFIRMED_MIXED: &str = r#"      - name: confirmed
        when_subject: {field: history, equals: Pending}
        when: token != ""
        moves: demo.orders.Order.confirm
        instance: id
        emits: [demo.orders.Confirmed]
        payload: {demo.orders.Confirmed: {id: input.id}}
"#;

/// The moving branch taken by the stored row alone.
const CONFIRMED_ROW_ONLY: &str = r"      - name: confirmed
        when_subject: {field: history, equals: Pending}
        moves: demo.orders.Order.confirm
        instance: id
        emits: [demo.orders.Confirmed]
        payload: {demo.orders.Confirmed: {id: input.id}}
";

/// The moving default.
const CONFIRMED_DEFAULT: &str = r"      - name: confirmed
        moves: demo.orders.Order.confirm
        instance: id
        emits: [demo.orders.Confirmed]
        payload: {demo.orders.Confirmed: {id: input.id}}
";

const TOKEN_REQUIRED: &str = r#"      - name: token-required
        when: token == ""
        error: demo.orders.TokenRequired
"#;

/// A default that accepts without moving: what a row the moving guard does not admit reaches.
const WAITING: &str = r"      - name: waiting
        preserves: demo.orders.Order
        instance: id
";

/// A moving sibling over an optional stored field the arrangement never fills.
const RUSHED: &str = r#"      - name: rushed
        when_subject: {predicate: 'note == "x"'}
        moves: demo.orders.Order.rush
        instance: id
        emits: [demo.orders.Rushed]
        payload: {demo.orders.Rushed: {id: input.id}}
"#;

const GONE: &str = r"      - name: gone
        wrong_state: true
        error: demo.orders.NoSuchOrder
";

fn spec(variants: &str, outcomes: &[&str]) -> String {
    format!(
        "{}{}",
        HEAD.replace("VARIANTS", variants),
        outcomes.concat()
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

// ---- a target answering as Entity Runtime + entity-core do -------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Guard {
    /// `token != ""` and `history == Pending`.
    Mixed,
    /// `history == Pending`.
    RowOnly,
    /// No guard: the default.
    Default,
}

#[derive(Clone, Copy)]
struct Shape {
    confirmed: Guard,
    token_required: bool,
    waiting: bool,
    rushed: bool,
}

struct Orders {
    shape: Shape,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    events: RefCell<Vec<ObservedEvent>>,
    minted: Cell<u32>,
}

impl Orders {
    fn new(shape: Shape) -> Self {
        Self {
            shape,
            rows: RefCell::default(),
            events: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn text(node: Option<&Node>) -> String {
    node.and_then(Node::as_text).unwrap_or_default().to_owned()
}

fn error(name: &str) -> DeclaredErrorValue {
    DeclaredErrorValue::new(name.parse().unwrap())
}

/// What entity-core's selection decided for `Confirm`.
enum Selected {
    Refused(&'static str, &'static str),
    Moves(&'static str, &'static [&'static str], &'static str),
    Preserves(&'static str),
    /// `OutcomeUnobservable` or `NoOutcomeSelected`: an error, not a declared branch.
    Error,
}

impl Orders {
    fn select(&self, token_empty: bool, row: &BTreeMap<String, Node>) -> Selected {
        let shape = self.shape;
        let history = text(row.get("history"));
        // Category 0: input-guarded refusals.
        if shape.token_required && token_empty {
            return Selected::Refused("token-required", "demo.orders.TokenRequired");
        }
        // Category 1, declaration order: `confirmed` (when guarded) is declared before `rushed`.
        match shape.confirmed {
            Guard::Mixed if !token_empty && history == "Pending" => {
                return Selected::Moves("confirmed", &["Open"], "Done")
            }
            Guard::RowOnly if history == "Pending" => {
                return Selected::Moves("confirmed", &["Open"], "Done")
            }
            _ => {}
        }
        if shape.rushed {
            match row.get("note").and_then(Node::as_text) {
                // Absent or null: `Unknown`, which stops selection.
                None => return Selected::Error,
                Some("x") => return Selected::Moves("rushed", &["Open"], "Rushed"),
                Some(_) => {}
            }
        }
        // Category 2: the default.
        if shape.confirmed == Guard::Default {
            return Selected::Moves("confirmed", &["Open"], "Done");
        }
        if shape.waiting {
            return Selected::Preserves("waiting");
        }
        Selected::Error
    }
}

impl ConformanceTarget for Orders {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-orders-2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        self.events.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let emit = |name: &str, id: &str| {
            let event = ObservedEvent::new(name.parse().unwrap()).with("id", Node::Text(id.into()));
            self.events.borrow_mut().push(event.clone());
            event
        };
        let result = match command.to_string().as_str() {
            "demo.orders.Place" => {
                let id = format!("00000000-0000-4000-8000-{n:012}");
                let mut row = BTreeMap::from([
                    ("id".to_owned(), Node::Text(id.clone())),
                    ("history".to_owned(), request.input["history"].clone()),
                    ("state".to_owned(), Node::Text("Open".to_owned())),
                ]);
                // Only where the model maps `note` from Place's input (`with_note_input`).
                if let Some(note) = request
                    .input
                    .get("note")
                    .filter(|note| **note != Node::Null)
                {
                    row.insert("note".to_owned(), note.clone());
                }
                self.rows.borrow_mut().insert(id.clone(), row);
                SemanticCommandResult::took(outcome(&command, "placed"))
                    .emitting(emit("demo.orders.Placed", &id))
            }
            "demo.orders.Rush" => {
                let id = text(request.input.get("id"));
                let mut rows = self.rows.borrow_mut();
                match rows.get_mut(&id) {
                    Some(row) if text(row.get("state")) == "Open" => {
                        row.insert("state".to_owned(), Node::Text("Rushed".to_owned()));
                        drop(rows);
                        SemanticCommandResult::took(outcome(&command, "rushed"))
                            .emitting(emit("demo.orders.Rushed", &id))
                    }
                    _ => SemanticCommandResult::undeclared(),
                }
            }
            "demo.orders.Close" => {
                let id = text(request.input.get("id"));
                let mut rows = self.rows.borrow_mut();
                match rows.get_mut(&id) {
                    Some(row) if text(row.get("state")) != "Closed" => {
                        row.insert("state".to_owned(), Node::Text("Closed".to_owned()));
                        drop(rows);
                        SemanticCommandResult::took(outcome(&command, "closed"))
                            .emitting(emit("demo.orders.Closed", &id))
                    }
                    _ => SemanticCommandResult::undeclared(),
                }
            }
            "demo.orders.Confirm" => {
                let id = text(request.input.get("id"));
                let token_empty = text(request.input.get("token")).is_empty();
                let gone = || {
                    SemanticCommandResult::took(outcome(&command, "gone"))
                        .with_error(error("demo.orders.NoSuchOrder"))
                };
                let row = self.rows.borrow().get(&id).cloned();
                match row {
                    None => gone(),
                    Some(row) => {
                        let state = text(row.get("state"));
                        match self.select(token_empty, &row) {
                            Selected::Refused(name, err) => {
                                SemanticCommandResult::took(outcome(&command, name))
                                    .with_error(error(err))
                            }
                            Selected::Preserves(name) => {
                                SemanticCommandResult::took(outcome(&command, name))
                            }
                            Selected::Error => SemanticCommandResult::undeclared(),
                            // `admit_state`: every move of Confirm starts from Open, so any other
                            // held state is a wrong state and `gone` answers.
                            Selected::Moves(name, from, to) => {
                                if from.contains(&state.as_str()) {
                                    if let Some(row) = self.rows.borrow_mut().get_mut(&id) {
                                        row.insert("state".to_owned(), Node::Text(to.to_owned()));
                                    }
                                    let mut took =
                                        SemanticCommandResult::took(outcome(&command, name));
                                    if name == "confirmed" {
                                        took = took.emitting(emit("demo.orders.Confirmed", &id));
                                    }
                                    took
                                } else {
                                    gone()
                                }
                            }
                        }
                    }
                }
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().values().cloned()))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let wanted = request.event.to_string();
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|event| format!("{event:?}").contains(&wanted))
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

fn is_wrong_state_confirm(id: &str) -> bool {
    id.starts_with(PREFIX) && id.ends_with(SUFFIX)
}

/// Every written wrong-state `Confirm` scenario run against `target`: `(id, status, detail)`.
fn wrong_state_results(result: &Synthesis, target: &Orders) -> Vec<(String, Status, String)> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|scenario| is_wrong_state_confirm(&scenario.scenario.to_string()))
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                scenario.status,
                format!("{scenario:?}"),
            )
        })
        .collect()
}

fn not_passed(results: &[(String, Status, String)]) -> Vec<String> {
    results
        .iter()
        .filter(|(_, status, _)| *status != Status::Passed)
        .map(|(id, status, detail)| format!("{id}: {status:?}\n{detail}"))
        .collect()
}

fn written(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| is_wrong_state_confirm(id))
        .collect()
}

/// The states no move of `Confirm` starts from.
const WRONG_STATES: [&str; 3] = ["Rushed", "Done", "Closed"];

fn refusals_about(result: &Synthesis, id: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

/// Every wrong state of `Confirm` whose scenario is neither written nor refused as ESS-SYNTH-003
/// naming `guard`, with the refusals recorded for it: a case whose written scenarios all pass is not
/// green for having written none (story:wrong-state-witness-unknown-and-own-stored-guards).
fn unaccounted(result: &Synthesis, guard: &str) -> Vec<String> {
    let written = written(result);
    WRONG_STATES
        .iter()
        .map(|state| format!("{PREFIX}{state}{SUFFIX}"))
        .filter(|id| !written.contains(id))
        .filter_map(|id| {
            let refused = refusals_about(result, &id);
            let named = refused
                .iter()
                .any(|refusal| refusal.starts_with("ESS-SYNTH-003") && refusal.contains(guard));
            (!named).then(|| format!("{id}: {refused:#?}"))
        })
        .collect()
}

/// `spec` with `Place` taking an optional `note` and storing it: a row can then hold the field
/// `rushed`'s guard reads.
fn with_note_input(text: &str) -> String {
    let filled = text
        .replace(
            "    input:\n      - {name: history, type: demo.orders.History}\n",
            concat!(
                "    input:\n",
                "      - {name: history, type: demo.orders.History}\n",
                "      - {name: note, type: Optional<String>}\n",
            ),
        )
        .replace(
            "        sets: {history: input.history}\n",
            "        sets: {history: input.history, note: input.note}\n",
        );
    assert_eq!(
        filled
            .matches("{name: note, type: Optional<String>}")
            .count(),
        3
    );
    assert_eq!(filled.matches("input.note").count(), 1);
    filled
}

// ---- the cases ---------------------------------------------------------------------------------

/// The moving branch's own subject half: `confirmed` needs `token != ""` *and* `history ==
/// Pending`, and a non-moving default `waiting` takes every other row. In `Closed` or `Done` an
/// Entity-Runtime-ordered target takes `waiting` on a `Confirmed` row (no move selected, so
/// `admit_state` never reaches `gone`). The witness must arrange a `Pending` row or refuse the
/// scenario; whatever it writes must pass the correct target.
#[test]
fn adversary2_the_moving_branchs_own_subject_guard_must_hold_on_the_row() {
    let mut failures = Vec::new();
    let mut missing = Vec::new();
    for variants in ["[Pending, Confirmed]", "[Confirmed, Pending]"] {
        let text = spec(variants, &[CONFIRMED_MIXED, TOKEN_REQUIRED, WAITING, GONE]);
        let result = synthesize(&ir(&text));
        let shape = Shape {
            confirmed: Guard::Mixed,
            token_required: true,
            waiting: true,
            rushed: false,
        };
        let results = wrong_state_results(&result, &Orders::new(shape));
        eprintln!("{variants}: written {:?}", written(&result));
        for failure in not_passed(&results) {
            failures.push(format!("{variants}: {failure}"));
        }
        for state in unaccounted(&result, "history == Pending") {
            missing.push(format!("{variants}: {state}"));
        }
    }
    assert_eq!(
        failures,
        Vec::<String>::new(),
        "a wrong-state scenario an Entity-Runtime-ordered target answers with `waiting`"
    );
    assert_eq!(
        missing,
        Vec::<String>::new(),
        "story:wrong-state-witness-unknown-and-own-stored-guards: a wrong state neither written nor \
         refused as ESS-SYNTH-003 naming `confirmed`'s own stored guard"
    );
}

/// The same with the moving branch guarded by the row alone (ESS refuses the shape without a
/// default as `non_exhaustive_branches`, so `waiting` stays): on a row `confirmed` does not admit,
/// entity-core takes `waiting`, never `gone`.
#[test]
fn adversary2_a_row_only_moving_branch_beside_a_preserving_default() {
    let mut failures = Vec::new();
    let mut missing = Vec::new();
    for variants in ["[Pending, Confirmed]", "[Confirmed, Pending]"] {
        let text = spec(variants, &[CONFIRMED_ROW_ONLY, WAITING, GONE]);
        let result = synthesize(&ir(&text));
        let shape = Shape {
            confirmed: Guard::RowOnly,
            token_required: false,
            waiting: true,
            rushed: false,
        };
        let results = wrong_state_results(&result, &Orders::new(shape));
        eprintln!("{variants}: written {:?}", written(&result));
        for failure in not_passed(&results) {
            failures.push(format!("{variants}: {failure}"));
        }
        for state in unaccounted(&result, "history == Pending") {
            missing.push(format!("{variants}: {state}"));
        }
    }
    assert_eq!(
        failures,
        Vec::<String>::new(),
        "a wrong-state scenario an Entity-Runtime-ordered target answers with `waiting`"
    );
    assert_eq!(
        missing,
        Vec::<String>::new(),
        "story:wrong-state-witness-unknown-and-own-stored-guards: a wrong state neither written nor \
         refused as ESS-SYNTH-003 naming `confirmed`'s own stored guard"
    );
}

/// The exemption: `rushed` moves from `Open` only, so the witness does not refute it in `Done`
/// or `Closed` — "selected or not, the move is the wrong-state answer". But its guard reads
/// `note`, an optional field no arrangement fills: on the row it is `Unknown`, and entity-core
/// stops selection there (`OutcomeUnobservable`) before the default `confirmed` is reached.
/// Whatever is written must pass the correct target.
///
/// Ported with one change (story:wrong-state-witness-unknown-and-own-stored-guards). No command of
/// this model writes `note`, so on this target every `Confirm` sent to a stored row stops at
/// `OutcomeUnobservable` and no wrong-state scenario written for it can pass: the pass-2 file's
/// `assert!(!results.is_empty())` held of no synthesis. It is replaced by what does: every wrong
/// state is refused as ESS-SYNTH-003 naming `rushed`'s guard where it is not written. The
/// written-and-passing half, with that assertion, is
/// `an_exempt_moving_sibling_is_witnessed_on_a_row_holding_the_field_its_guard_reads`, where a
/// creator can fill `note`.
#[test]
fn adversary2_an_exempt_moving_sibling_whose_guard_is_unknown_on_the_row() {
    let text = spec("[Pending, Confirmed]", &[CONFIRMED_DEFAULT, RUSHED, GONE]);
    let result = synthesize(&ir(&text));
    let shape = Shape {
        confirmed: Guard::Default,
        token_required: false,
        waiting: false,
        rushed: true,
    };
    let results = wrong_state_results(&result, &Orders::new(shape));
    eprintln!("written {:?}", written(&result));
    assert_eq!(
        not_passed(&results),
        Vec::<String>::new(),
        "a wrong-state scenario an Entity-Runtime-ordered target answers with OutcomeUnobservable"
    );
    // A refusal renders a text literal bare (`note == x`), a fact as `{fact: x}`.
    assert_eq!(
        unaccounted(&result, "note == x"),
        Vec::<String>::new(),
        "story:wrong-state-witness-unknown-and-own-stored-guards: a wrong state neither written nor \
         refused as ESS-SYNTH-003 naming `rushed`'s guard, which no row decides"
    );
}

/// The exemption's other half: where `Place` can fill `note`, each wrong-state row holds it, so
/// `rushed`'s guard decides there and the move answers with the wrong state whichever way it
/// decides. Every wrong state is written, and every one passes the correct target
/// (story:wrong-state-witness-unknown-and-own-stored-guards).
#[test]
fn an_exempt_moving_sibling_is_witnessed_on_a_row_holding_the_field_its_guard_reads() {
    let text = with_note_input(&spec(
        "[Pending, Confirmed]",
        &[CONFIRMED_DEFAULT, RUSHED, GONE],
    ));
    let result = synthesize(&ir(&text));
    let shape = Shape {
        confirmed: Guard::Default,
        token_required: false,
        waiting: false,
        rushed: true,
    };
    let results = wrong_state_results(&result, &Orders::new(shape));
    eprintln!("written {:?}", written(&result));
    assert!(!results.is_empty(), "no wrong-state scenario ran");
    assert_eq!(
        not_passed(&results),
        Vec::<String>::new(),
        "a wrong-state scenario an Entity-Runtime-ordered target answers with OutcomeUnobservable"
    );
    let unwritten: Vec<String> = WRONG_STATES
        .iter()
        .map(|state| format!("{PREFIX}{state}{SUFFIX}"))
        .filter(|id| !written(&result).contains(id))
        .map(|id| format!("{id}: {:#?}", refusals_about(&result, &id)))
        .collect();
    assert_eq!(
        unwritten,
        Vec::<String>::new(),
        "every wrong state is written"
    );
}
