//! Adversary, pass 1, `story:a-wrong-state-witness-may-miss-a-mixed-guard-sibling-by-its-subject-guard`
//! (beyond10x/ess#192).
//!
//! The unit's own tests read the synthesized steps. These run the synthesized suite against a
//! hand-written `Orders` target that answers the way Entity Runtime lowering orders branches
//! (`ess-entity-runtime` `build_command`: input-guarded refusals, then the other guarded branches,
//! the default, and `wrong_state` last; a move from a state it does not leave answers
//! `wrong_state`), and against mutants of it — and vary the #192 shape where the unit's tests do
//! not: a subject-only sibling, two mixed siblings over two fields, the predicate form with
//! `not`, no view exposing the relied-on field, and byte determinism.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const CLOSED: &str = "demo.orders.Order/state/Closed/refuses/demo.orders.Confirm";
const DONE: &str = "demo.orders.Order/state/Done/refuses/demo.orders.Confirm";

/// The unit's #192 shape, with a second stored field `priority` the variants below guard on.
const HEAD: &str = r"format: ess/16
system: demo
version: v1
domain: demo.orders
summary: An order confirmed with a token, refused without one, and gone once it has left Open.
types:
  - {name: demo.orders.OrderId, kind: newtype, of: Uuid}
  - {name: demo.orders.History, kind: enum, variants: [Pending, Confirmed]}
  - {name: demo.orders.Priority, kind: enum, variants: [Low, High]}
entities:
  - name: demo.orders.Order
    identity: {name: id, type: demo.orders.OrderId}
    fields:
      - {name: history, type: demo.orders.History}
      - {name: priority, type: demo.orders.Priority}
    lifecycle:
      initial: Open
      states: [Open, Done, Closed]
      terminal: [Closed]
      transitions:
        - {name: confirm, from: [Open], to: Done}
        - {name: close, from: [Open, Done], to: Closed}
actors:
  - name: demo.orders.Clerk
    may: [demo.orders.Place, demo.orders.Confirm, demo.orders.Close]
errors:
  - {name: demo.orders.TokenRequired, summary: The token is empty., fields: []}
  - {name: demo.orders.NoSuchOrder, summary: The order has left Open., fields: []}
  - {name: demo.orders.Held, summary: The order is held., fields: []}
events:
  - name: demo.orders.Placed
    fields: [{name: id, type: demo.orders.OrderId}]
  - name: demo.orders.Confirmed
    fields: [{name: id, type: demo.orders.OrderId}]
  - name: demo.orders.Closed
    fields: [{name: id, type: demo.orders.OrderId}]
commands:
  - name: demo.orders.Place
    input:
      - {name: history, type: demo.orders.History}
      - {name: priority, type: demo.orders.Priority}
    outcomes:
      - name: placed
        creates: demo.orders.Order
        instance: id
        sets: {history: input.history, priority: input.priority}
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
  - name: demo.orders.Confirm
    input:
      - {name: id, type: demo.orders.OrderId}
      - {name: token, type: String}
    outcomes:
      - name: confirmed
        moves: demo.orders.Order.confirm
        instance: id
        emits: [demo.orders.Confirmed]
        payload: {demo.orders.Confirmed: {id: input.id}}
";

const ALREADY_CONFIRMED: &str = r#"      - name: already-confirmed
        when_subject: {field: history, equals: Confirmed}
        when: token != ""
        preserves: demo.orders.Order
        instance: id
"#;

/// The same guard in the predicate form, through `not`.
const ALREADY_CONFIRMED_NOT: &str = r#"      - name: already-confirmed
        when_subject: {predicate: {not: 'history == Pending'}}
        when: token != ""
        preserves: demo.orders.Order
        instance: id
"#;

/// A second mixed sibling, over the other stored field.
const URGENT: &str = r#"      - name: urgent
        when_subject: {field: priority, equals: High}
        when: token != ""
        preserves: demo.orders.Order
        instance: id
"#;

/// A subject-only refusal: taken for every `Pending` order, whatever the input.
const HELD: &str = r"      - name: held
        when_subject: {predicate: 'history == Pending'}
        error: demo.orders.Held
";

const TOKEN_REQUIRED: &str = r#"      - name: token-required
        when: token == ""
        error: demo.orders.TokenRequired
"#;

const GONE: &str = r"      - name: gone
        wrong_state: true
        error: demo.orders.NoSuchOrder
";

const VIEW: &str = r"views:
  - name: demo.orders.Orders
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.orders.OrderId}
      - {name: history, type: demo.orders.History}
      - {name: priority, type: demo.orders.Priority}
      - {name: state, type: demo.orders.Order.State}
";

/// The view without `history`: nothing can observe the field the witness relies on.
const VIEW_WITHOUT_HISTORY: &str = r"views:
  - name: demo.orders.Orders
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.orders.OrderId}
      - {name: priority, type: demo.orders.Priority}
      - {name: state, type: demo.orders.Order.State}
";

fn spec(siblings: &[&str], view: &str) -> String {
    format!("{HEAD}{}{view}", siblings.concat())
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
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

fn scenario<'a>(result: &'a Synthesis, id: &str) -> Option<&'a ConformanceScenario> {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

// ---- a target that answers as Entity Runtime orders the branches -------------------------------

/// Which siblings the specification under test declares, and which defect a mutant carries.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Default)]
struct Shape {
    already_confirmed: bool,
    urgent: bool,
    held: bool,
    token_required: bool,
    /// Mutant: `already-confirmed` taken on its input half alone.
    ignore_history: bool,
    /// Mutant: `urgent` taken on its input half alone.
    ignore_priority: bool,
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

impl ConformanceTarget for Orders {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-orders", "1"))
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
        let mut rows = self.rows.borrow_mut();
        let emit = |name: &str, id: &str| {
            let event = ObservedEvent::new(name.parse().unwrap()).with("id", Node::Text(id.into()));
            self.events.borrow_mut().push(event.clone());
            event
        };
        let result = match command.to_string().as_str() {
            "demo.orders.Place" => {
                let id = format!("00000000-0000-4000-8000-{n:012}");
                let row = BTreeMap::from([
                    ("id".to_owned(), Node::Text(id.clone())),
                    ("history".to_owned(), request.input["history"].clone()),
                    ("priority".to_owned(), request.input["priority"].clone()),
                    ("state".to_owned(), Node::Text("Open".to_owned())),
                ]);
                rows.insert(id.clone(), row);
                SemanticCommandResult::took(outcome(&command, "placed"))
                    .emitting(emit("demo.orders.Placed", &id))
            }
            "demo.orders.Close" => {
                let id = text(request.input.get("id"));
                match rows.get_mut(&id) {
                    Some(row) if matches!(text(row.get("state")).as_str(), "Open" | "Done") => {
                        row.insert("state".to_owned(), Node::Text("Closed".to_owned()));
                        SemanticCommandResult::took(outcome(&command, "closed"))
                            .emitting(emit("demo.orders.Closed", &id))
                    }
                    _ => SemanticCommandResult::undeclared(),
                }
            }
            "demo.orders.Confirm" => {
                let id = text(request.input.get("id"));
                let token_empty = text(request.input.get("token")).is_empty();
                let shape = self.shape;
                let gone = || {
                    SemanticCommandResult::took(outcome(&command, "gone"))
                        .with_error(error("demo.orders.NoSuchOrder"))
                };
                // Category 0: the input-guarded refusal.
                if shape.token_required && token_empty {
                    SemanticCommandResult::took(outcome(&command, "token-required"))
                        .with_error(error("demo.orders.TokenRequired"))
                } else if let Some(row) = rows.get_mut(&id) {
                    let history = text(row.get("history"));
                    let priority = text(row.get("priority"));
                    // Category 1: the other guarded branches, in declaration order.
                    if shape.already_confirmed
                        && !token_empty
                        && (shape.ignore_history || history == "Confirmed")
                    {
                        SemanticCommandResult::took(outcome(&command, "already-confirmed"))
                    } else if shape.urgent
                        && !token_empty
                        && (shape.ignore_priority || priority == "High")
                    {
                        SemanticCommandResult::took(outcome(&command, "urgent"))
                    } else if shape.held && history == "Pending" {
                        SemanticCommandResult::took(outcome(&command, "held"))
                            .with_error(error("demo.orders.Held"))
                    } else if text(row.get("state")) == "Open" {
                        // Category 2: the default, which moves from Open only.
                        row.insert("state".to_owned(), Node::Text("Done".to_owned()));
                        SemanticCommandResult::took(outcome(&command, "confirmed"))
                            .emitting(emit("demo.orders.Confirmed", &id))
                    } else {
                        // Category 3.
                        gone()
                    }
                } else {
                    gone()
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

/// The status of each wrong-state `Confirm` scenario the suite carries, run against `target`.
fn wrong_state_results(result: &Synthesis, target: &Orders) -> Vec<(String, Status, String)> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|scenario| {
            let id = scenario.scenario.to_string();
            id == CLOSED || id == DONE
        })
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

fn steps(result: &Synthesis, id: &str) -> String {
    scenario(result, id).map_or_else(
        || format!("no scenario; refusals {:#?}", refusals_about(result, id)),
        |scenario| format!("{:#?}", scenario.steps),
    )
}

// ---- the cases ---------------------------------------------------------------------------------

/// The #192 shape: a correct target passes both wrong-state scenarios, and a target that takes
/// `already-confirmed` on its input alone fails them — the scenario tells the two apart.
#[test]
fn adversary_mixedguard_the_issue_shape_passes_a_correct_target_and_fails_the_mutant() {
    let text = spec(&[ALREADY_CONFIRMED, TOKEN_REQUIRED, GONE], VIEW);
    let result = synthesize(&ir(&text));
    let shape = Shape {
        already_confirmed: true,
        token_required: true,
        ..Shape::default()
    };
    let correct = wrong_state_results(&result, &Orders::new(shape));
    assert_eq!(correct.len(), 2, "both scenarios ran: {correct:#?}");
    assert_eq!(not_passed(&correct), Vec::<String>::new());
    let mutant = wrong_state_results(
        &result,
        &Orders::new(Shape {
            ignore_history: true,
            ..shape
        }),
    );
    assert_eq!(
        mutant
            .iter()
            .filter(|(_, status, _)| *status == Status::Passed)
            .count(),
        0,
        "the mutant taking already-confirmed on its token alone passes: {mutant:#?}"
    );
}

/// A subject-only sibling `held` (`history == Pending`) beside the #192 shape: every row either
/// selects `held` or `already-confirmed` under `token != ""`, and `token == ""` selects
/// `token-required`. No witness exists; whatever synthesis writes must pass a correct target.
#[test]
fn adversary_mixedguard_a_subject_only_sibling_on_the_falsifying_value_is_not_walked_into() {
    let text = spec(&[ALREADY_CONFIRMED, TOKEN_REQUIRED, HELD, GONE], VIEW);
    let result = synthesize(&ir(&text));
    let shape = Shape {
        already_confirmed: true,
        token_required: true,
        held: true,
        ..Shape::default()
    };
    let results = wrong_state_results(&result, &Orders::new(shape));
    assert_eq!(
        not_passed(&results),
        Vec::<String>::new(),
        "a wrong-state scenario a correct target fails; CLOSED steps: {}",
        steps(&result, CLOSED)
    );
}

/// The subject-only sibling alone, no input guard anywhere: the witness takes the row the
/// arrangement builds unchanged, the path the unit did not touch. Whatever is written must pass a
/// correct target.
#[test]
fn adversary_mixedguard_a_subject_only_sibling_alone() {
    for variants in ["[Pending, Confirmed]", "[Confirmed, Pending]"] {
        let text = spec(&[HELD, GONE], VIEW).replace(
            "variants: [Pending, Confirmed]",
            &format!("variants: {variants}"),
        );
        let result = synthesize(&ir(&text));
        let shape = Shape {
            held: true,
            ..Shape::default()
        };
        let results = wrong_state_results(&result, &Orders::new(shape));
        assert_eq!(
            not_passed(&results),
            Vec::<String>::new(),
            "{variants}: a wrong-state scenario a correct target fails"
        );
    }
}

/// Two mixed siblings over two fields: the witness row must falsify both subject halves, and a
/// target ignoring either subject half fails.
#[test]
fn adversary_mixedguard_two_mixed_siblings_over_two_fields() {
    let text = spec(&[ALREADY_CONFIRMED, URGENT, TOKEN_REQUIRED, GONE], VIEW);
    let result = synthesize(&ir(&text));
    for id in [CLOSED, DONE] {
        assert_eq!(refusals_about(&result, id), Vec::<String>::new(), "{id}");
    }
    let shape = Shape {
        already_confirmed: true,
        urgent: true,
        token_required: true,
        ..Shape::default()
    };
    let correct = wrong_state_results(&result, &Orders::new(shape));
    assert_eq!(correct.len(), 2, "{correct:#?}");
    assert_eq!(not_passed(&correct), Vec::<String>::new());
    for mutant in [
        Shape {
            ignore_history: true,
            ..shape
        },
        Shape {
            ignore_priority: true,
            ..shape
        },
    ] {
        let results = wrong_state_results(&result, &Orders::new(mutant));
        assert!(
            results
                .iter()
                .all(|(_, status, _)| *status != Status::Passed),
            "a mutant ignoring one subject half passes: {results:#?}"
        );
    }
}

/// The predicate form through `not`: same meaning as `history == Confirmed` over a two-variant
/// enum, so the same scenarios, passed by the correct target.
#[test]
fn adversary_mixedguard_the_predicate_form_with_not() {
    let text = spec(&[ALREADY_CONFIRMED_NOT, TOKEN_REQUIRED, GONE], VIEW);
    let result = synthesize(&ir(&text));
    for id in [CLOSED, DONE] {
        assert_eq!(refusals_about(&result, id), Vec::<String>::new(), "{id}");
    }
    let shape = Shape {
        already_confirmed: true,
        token_required: true,
        ..Shape::default()
    };
    let correct = wrong_state_results(&result, &Orders::new(shape));
    assert_eq!(correct.len(), 2, "{correct:#?}");
    assert_eq!(not_passed(&correct), Vec::<String>::new());
}

/// No view exposes `history`: the witness cannot observe the field it relies on. Whatever
/// synthesis does, it names the scenario — written, or refused with a cause — and a written
/// one passes a correct target.
#[test]
fn adversary_mixedguard_no_view_exposes_the_relied_on_field() {
    let text = spec(
        &[ALREADY_CONFIRMED, TOKEN_REQUIRED, GONE],
        VIEW_WITHOUT_HISTORY,
    );
    let result = synthesize(&ir(&text));
    for id in [CLOSED, DONE] {
        assert!(
            scenario(&result, id).is_some() || !refusals_about(&result, id).is_empty(),
            "{id} is neither written nor refused"
        );
    }
    let shape = Shape {
        already_confirmed: true,
        token_required: true,
        ..Shape::default()
    };
    let results = wrong_state_results(&result, &Orders::new(shape));
    assert_eq!(not_passed(&results), Vec::<String>::new());
    eprintln!(
        "no-view outcome: CLOSED {:?} / DONE {:?}",
        refusals_about(&result, CLOSED),
        refusals_about(&result, DONE)
    );
}

/// Synthesis is a function of the model: two runs write the same suite bytes.
#[test]
fn adversary_mixedguard_suite_bytes_are_deterministic() {
    for text in [
        spec(&[ALREADY_CONFIRMED, TOKEN_REQUIRED, GONE], VIEW),
        spec(&[ALREADY_CONFIRMED, URGENT, TOKEN_REQUIRED, GONE], VIEW),
    ] {
        let model = ir(&text);
        let first = serde_json::to_string(&synthesize(&model).suite).unwrap();
        let second = serde_json::to_string(&synthesize(&ir(&text)).suite).unwrap();
        assert_eq!(first, second);
    }
}

#[allow(dead_code)]
fn is_command(step: &ScenarioStep) -> bool {
    matches!(step, ScenarioStep::ExecuteCommand { .. })
}
