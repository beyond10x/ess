//! Adversary, pass 1, `story:wrong-state-witness-unknown-and-own-stored-guards`.
//!
//! The unit decides whether a stored guard is `Unknown` "by absence" (an optional field the row
//! leaves empty, which stops Entity Runtime's selection with `OutcomeUnobservable`) by asking
//! `evaluate_row` twice, plainly and as a guard, and calling the guard absent only where the
//! guard reading turns `Unknown` into `False`. Any other `Unknown` is taken as one "the
//! arrangement caused", which the target decides, "and either way the move answers".
//!
//! `evaluate_row` answers `Unknown` before any absence check where the guard reads a stored field
//! the arrangement holds no literal for — a creator's `{generated: true}` value. A guard reading
//! such a field *and* an optional field no creator fills (`code == "zzz" or note == "x"`) is then
//! not called absent, although on every target that generated `code` as anything but `zzz` it is
//! `Unknown` for the absent `note` and stops selection. These cases run the written wrong-state
//! `Confirm` scenarios against a target answering in Entity Runtime's order, as the pass-2 file
//! (`adversary_mixed_guard_pass2.rs`) does, and require every one to pass, or the scenario to be
//! refused as ESS-SYNTH-003 naming the guard.
//!
//! Only `Rushed` and `Closed` are judged: they are arranged by `Rush` and `Close`, so a failure
//! there is the wrong-state send's and not an arrangement step's. `Done` is reached only through
//! `Confirm` itself, whose arrangement step stops on this target for the same `Unknown`.

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

/// The states judged: arranged without `Confirm`.
const JUDGED: [&str; 2] = ["Rushed", "Closed"];

const HEAD: &str = r"format: ess/16
system: demo
version: v1
domain: demo.orders
summary: An order confirmed or hurried, and gone once it has left Open.
types:
  - {name: demo.orders.OrderId, kind: newtype, of: Uuid}
  - {name: demo.orders.History, kind: enum, variants: [Pending, Confirmed]}
entities:
  - name: demo.orders.Order
    identity: {name: id, type: demo.orders.OrderId}
    fields:
      - {name: history, type: demo.orders.History}
      - {name: note, type: Optional<String>}
      - {name: code, type: String}
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
        sets: {history: input.history, code: {generated: true}}
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
    outcomes:
";

/// Model A: a moving default, and an exempt moving sibling whose guard reads the generated `code`
/// and the never-filled `note`.
const MODEL_A: &str = r#"      - name: confirmed
        moves: demo.orders.Order.confirm
        instance: id
        emits: [demo.orders.Confirmed]
        payload: {demo.orders.Confirmed: {id: input.id}}
      - name: rushed
        when_subject: {predicate: {any: ['code == "zzz"', 'note == "x"']}}
        moves: demo.orders.Order.rush
        instance: id
        emits: [demo.orders.Rushed]
        payload: {demo.orders.Rushed: {id: input.id}}
      - name: gone
        wrong_state: true
        error: demo.orders.NoSuchOrder
"#;

/// Model B: the moving branch's own guard reads both fields, beside a default that moves too.
const MODEL_B: &str = r#"      - name: confirmed
        when_subject: {predicate: {any: ['code == "zzz"', 'note == "x"']}}
        moves: demo.orders.Order.confirm
        instance: id
        emits: [demo.orders.Confirmed]
        payload: {demo.orders.Confirmed: {id: input.id}}
      - name: hurried
        moves: demo.orders.Order.rush
        instance: id
        emits: [demo.orders.Rushed]
        payload: {demo.orders.Rushed: {id: input.id}}
      - name: gone
        wrong_state: true
        error: demo.orders.NoSuchOrder
"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Model {
    A,
    B,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tri {
    True,
    False,
    Unknown,
}

/// `code == "zzz" or note == "x"`, Kleene: an absent `note` is `Unknown`.
fn masked(row: &BTreeMap<String, Node>) -> Tri {
    if row.get("code").and_then(Node::as_text) == Some("zzz") {
        return Tri::True;
    }
    match row.get("note").and_then(Node::as_text) {
        None => Tri::Unknown,
        Some("x") => Tri::True,
        Some(_) => Tri::False,
    }
}

enum Selected {
    Moves(&'static str, &'static str, &'static str),
    /// `OutcomeUnobservable`: an error, not a declared branch.
    Error,
}

struct Orders {
    model: Model,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    events: RefCell<Vec<ObservedEvent>>,
    minted: Cell<u32>,
}

impl Orders {
    fn new(model: Model) -> Self {
        Self {
            model,
            rows: RefCell::default(),
            events: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    /// Entity Runtime's order: the guarded branches in declaration order, the default, and
    /// `wrong_state` last (answered by `admit_state` for a selected move from a state it does not
    /// leave).
    fn select(&self, row: &BTreeMap<String, Node>) -> Selected {
        match (self.model, masked(row)) {
            (_, Tri::Unknown) => Selected::Error,
            (Model::A, Tri::True) => Selected::Moves("rushed", "Rushed", "demo.orders.Rushed"),
            (Model::A, Tri::False) | (Model::B, Tri::True) => {
                Selected::Moves("confirmed", "Done", "demo.orders.Confirmed")
            }
            (Model::B, Tri::False) => Selected::Moves("hurried", "Rushed", "demo.orders.Rushed"),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn text(node: Option<&Node>) -> String {
    node.and_then(Node::as_text).unwrap_or_default().to_owned()
}

impl ConformanceTarget for Orders {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-d350-u3", "1"))
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
        let mover = |from: &[&str], to: &str, name: &str, event: &str| {
            let id = text(request.input.get("id"));
            let mut rows = self.rows.borrow_mut();
            match rows.get_mut(&id) {
                Some(row) if from.contains(&text(row.get("state")).as_str()) => {
                    row.insert("state".to_owned(), Node::Text(to.to_owned()));
                    drop(rows);
                    SemanticCommandResult::took(outcome(&command, name)).emitting(emit(event, &id))
                }
                _ => SemanticCommandResult::undeclared(),
            }
        };
        let result = match command.to_string().as_str() {
            "demo.orders.Place" => {
                let id = format!("00000000-0000-4000-8000-{n:012}");
                let row = BTreeMap::from([
                    ("id".to_owned(), Node::Text(id.clone())),
                    ("history".to_owned(), request.input["history"].clone()),
                    ("state".to_owned(), Node::Text("Open".to_owned())),
                    // The implementation-generated value: anything but `zzz`.
                    ("code".to_owned(), Node::Text(format!("gen-{n}"))),
                ]);
                self.rows.borrow_mut().insert(id.clone(), row);
                SemanticCommandResult::took(outcome(&command, "placed"))
                    .emitting(emit("demo.orders.Placed", &id))
            }
            "demo.orders.Rush" => mover(&["Open"], "Rushed", "rushed", "demo.orders.Rushed"),
            "demo.orders.Close" => mover(
                &["Open", "Rushed", "Done"],
                "Closed",
                "closed",
                "demo.orders.Closed",
            ),
            "demo.orders.Confirm" => {
                let id = text(request.input.get("id"));
                let gone = || {
                    SemanticCommandResult::took(outcome(&command, "gone")).with_error(
                        DeclaredErrorValue::new("demo.orders.NoSuchOrder".parse().unwrap()),
                    )
                };
                let row = self.rows.borrow().get(&id).cloned();
                match row {
                    None => gone(),
                    Some(row) => match self.select(&row) {
                        Selected::Error => SemanticCommandResult::undeclared(),
                        Selected::Moves(name, to, event) => {
                            if text(row.get("state")) == "Open" {
                                if let Some(row) = self.rows.borrow_mut().get_mut(&id) {
                                    row.insert("state".to_owned(), Node::Text(to.to_owned()));
                                }
                                SemanticCommandResult::took(outcome(&command, name))
                                    .emitting(emit(event, &id))
                            } else {
                                gone()
                            }
                        }
                    },
                }
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        // The view projects no `code`.
        Ok(SemanticViewResult::of(self.rows.borrow().values().map(
            |row| {
                row.iter()
                    .filter(|(name, _)| name.as_str() != "code")
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect::<BTreeMap<_, _>>()
            },
        )))
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

fn judged_id(state: &str) -> String {
    format!("{PREFIX}{state}{SUFFIX}")
}

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

/// Every judged wrong state whose scenario is written and fails `target`, or is neither written
/// nor refused as ESS-SYNTH-003 naming `note == x`.
fn wrong(model: Model, outcomes: &str) -> Vec<String> {
    let text = format!("{HEAD}{outcomes}");
    let result = synthesize(&ir(&text));
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Orders::new(model))
        .into_report();
    let mut out = Vec::new();
    for state in JUDGED {
        let id = judged_id(state);
        let ran = report
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario.to_string() == id);
        match ran {
            Some(scenario) if scenario.status == Status::Passed => {}
            Some(scenario) => out.push(format!(
                "{id}: written and {:?} on an Entity-Runtime-ordered target\n{scenario:?}",
                scenario.status
            )),
            None => {
                let refused = refusals_about(&result, &id);
                let named = refused
                    .iter()
                    .any(|r| r.starts_with("ESS-SYNTH-003") && r.contains("note == x"));
                if !named {
                    out.push(format!(
                        "{id}: neither written nor refused naming the guard: {refused:#?}"
                    ));
                }
            }
        }
    }
    out
}

/// An exempt moving sibling whose guard is `Unknown` for an absent field, masked by a generated
/// field it also reads: the witness must not be written to that row.
#[test]
fn adversary_d350_u3_an_exempt_siblings_absent_field_masked_by_a_generated_field() {
    assert_eq!(
        wrong(Model::A, MODEL_A),
        Vec::<String>::new(),
        "story:wrong-state-witness-unknown-and-own-stored-guards: an exempt sibling's guard left \
         Unknown by an absent field on the witness row, not called absent because it also reads a \
         generated field"
    );
}

/// The moving branch's own guard, `Unknown` for an absent field and masked the same way, beside
/// a default that moves: selection stops on the own guard before the default is reached.
#[test]
fn adversary_d350_u3_the_own_guards_absent_field_masked_by_a_generated_field() {
    assert_eq!(
        wrong(Model::B, MODEL_B),
        Vec::<String>::new(),
        "story:wrong-state-witness-unknown-and-own-stored-guards: the moving branch's own guard \
         left Unknown by an absent field on the witness row, not called absent because it also \
         reads a generated field"
    );
}

/// The guards without the generated leaf: `note == "x"` alone, `Unknown` by absence and so called
/// absent. On the same target the two models' judged states are refused (or written and
/// passing), which places the red of the two cases above on the generated leaf.
const BARE: &str = "predicate: 'note == \"x\"'";

#[test]
fn control_without_the_generated_leaf_the_exempt_sibling_is_called_absent() {
    let outcomes = MODEL_A.replace(
        "predicate: {any: ['code == \"zzz\"', 'note == \"x\"']}",
        BARE,
    );
    assert_ne!(outcomes, MODEL_A);
    assert_eq!(wrong(Model::A, &outcomes), Vec::<String>::new());
}

#[test]
fn control_without_the_generated_leaf_the_own_guard_is_called_absent() {
    let outcomes = MODEL_B.replace(
        "predicate: {any: ['code == \"zzz\"', 'note == \"x\"']}",
        BARE,
    );
    assert_ne!(outcomes, MODEL_B);
    assert_eq!(wrong(Model::B, &outcomes), Vec::<String>::new());
}
