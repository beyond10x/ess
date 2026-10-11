//! Mutants synthesis used to let through, each decided through the mutation audit's own machinery.
//!
//! `story:synthesis-kills-connective-and-source-mutants` (beyond10x/ess#154, #155, #132). A mutant is
//! killed when the suite synthesized from the mutated specification holds an expectation the
//! unmutated specification contradicts, because an implementation of the original then fails it.
//! No reference target implements these small models, so the contradiction is decided here against
//! the original model rather than by running a target: for a stateless command, which branch the
//! original selects for the input a scenario sends; for a lifecycle refusal, whether the original
//! lets the move start from the state the scenario arranged.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::source::SourceMap;
use ess_conformance::mutate::{self, Document, MutantClass};
use ess_conformance::scenario::{ScenarioStep, ScenarioValue, ViewExpectation};
use ess_conformance::synthesize::{synthesize, Note, Synthesis};
use ess_conformance::{flatten, when, Decision, ScenarioId};
use ess_domain::command::TestStrategy;
use ess_domain::name::QualifiedName;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;

// ---- building models -----------------------------------------------------------------------------

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn compiled((files, texts): &(Vec<Document>, SourceMap)) -> EssIr {
    mutate::compile(files.clone(), texts).unwrap_or_else(|stillborn| {
        panic!("the model compiles: {} {}", stillborn.code, stillborn.cause)
    })
}

/// Every mutant of `class` the audit enumerates, compiled.
fn mutants_of(model: &(Vec<Document>, SourceMap), class: MutantClass) -> Vec<(String, EssIr)> {
    let (files, texts) = model;
    mutate::mutants(files, &[class])
        .into_iter()
        .map(|mutant| {
            let mutated = mutate::apply(files, &mutant.mutation).expect("the site exists");
            let ir = mutate::compile(mutated, texts).unwrap_or_else(|stillborn| {
                panic!(
                    "{} is not stillborn: {} {}",
                    mutant.id, stillborn.code, stillborn.cause
                )
            });
            (mutant.id, ir)
        })
        .collect()
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn steps<'a>(synthesis: &'a Synthesis, id: &str) -> &'a [ScenarioStep] {
    let parsed = ScenarioId::parse(id).expect("a scenario id");
    &synthesis
        .suite
        .scenario(&parsed)
        .unwrap_or_else(|| {
            panic!(
                "`{id}` is in the suite; it holds {:?} and refuses {:?}",
                ids(synthesis),
                synthesis
                    .refusals
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
        .steps
}

// ---- reading invocations -------------------------------------------------------------------------

/// One command a scenario sends and the branch it then requires.
struct Invocation {
    scenario: String,
    command: String,
    input: BTreeMap<String, Node>,
    expected: String,
}

/// Every (command, literal input, required branch) pair the suite states.
fn invocations(synthesis: &Synthesis) -> Vec<Invocation> {
    let mut out = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut pending: Option<(String, BTreeMap<String, Node>)> = None;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand { command, input, .. } => {
                    let literals = input
                        .iter()
                        .filter_map(|(field, value)| {
                            value
                                .as_literal()
                                .cloned()
                                .map(|node| (field.clone(), node))
                        })
                        .collect();
                    pending = Some((command.to_string(), literals));
                }
                ScenarioStep::ExpectOutcome { outcome } => {
                    if let Some((command, input)) = pending.take() {
                        let expected = outcome
                            .to_string()
                            .rsplit('/')
                            .next()
                            .expect("an outcome ref names its branch")
                            .to_owned();
                        out.push(Invocation {
                            scenario: id.to_string(),
                            command,
                            input,
                            expected,
                        });
                    }
                }
                _ => {}
            }
        }
    }
    out
}

fn command<'ir>(ir: &'ir EssIr, name: &str) -> &'ir ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new(name).expect("a valid name"))
        .expect("the command is declared")
}

/// The branch the given model selects for a stateless command: its first input-guarded branch whose
/// guard holds, else its default.
fn selected(ir: &EssIr, name: &str, input: &BTreeMap<String, Node>) -> String {
    let command = command(ir, name);
    let facts = flatten(ir, command, input).expect("a synthesised input fits its command");
    for outcome in &command.outcomes {
        if outcome.test_strategy == TestStrategy::ConstructInput {
            let guard = when(outcome).expect("a constructed branch has a guard");
            if facts.decide(guard) == Decision::Satisfied {
                return outcome.name.to_string();
            }
        }
    }
    command
        .outcomes
        .iter()
        .find(|outcome| outcome.test_strategy == TestStrategy::DefaultBranch)
        .map(|outcome| outcome.name.to_string())
        .expect("a command with a guarded branch here also declares a default")
}

/// The invocations of `synthesis` that `original` answers with a different branch: the scenarios
/// an implementation of `original` fails.
fn contradicted(synthesis: &Synthesis, original: &EssIr, name: &str) -> Vec<String> {
    invocations(synthesis)
        .into_iter()
        .filter(|invocation| invocation.command == name)
        .filter(|invocation| selected(original, name, &invocation.input) != invocation.expected)
        .map(|invocation| {
            format!(
                "{} sends {:?} and requires `{}`",
                invocation.scenario, invocation.input, invocation.expected
            )
        })
        .collect()
}

// ---- #155: the connective of a guard -------------------------------------------------------------

const SEND: &str = r#"
format: ess/13
system: mail
version: v1
domain: mail.send

events:
  - name: mail.send.Sent
    fields:
      - {name: to, type: String}

errors:
  - name: mail.send.SendRequiresToAndText
    summary: A message needs an address and a text.
    fields: []

commands:
  - name: mail.send.Send
    input:
      - {name: to, type: String}
      - {name: text, type: String}
    outcomes:
      - name: requires-to-and-text
        when:
          CONNECTIVE: [to == "", text == ""]
        error: mail.send.SendRequiresToAndText
      - name: sent
        emits: [mail.send.Sent]
        payload:
          mail.send.Sent: {to: input.to}
"#;

const SEND_COMMAND: &str = "mail.send.Send";

fn send(connective: &str) -> (Vec<Document>, SourceMap) {
    documents(&SEND.replace("CONNECTIVE", connective))
}

/// Each disjunct of `any:` is witnessed holding alone, and the branch is still required there.
#[test]
fn a_disjunctive_guard_is_witnessed_once_per_disjunct_with_the_others_false() {
    let model = send("any");
    let ir = compiled(&model);
    let synthesis = synthesize(&ir);
    let command = command(&ir, SEND_COMMAND);
    let guard = when(
        command
            .outcomes
            .iter()
            .find(|outcome| outcome.name.as_str() == "requires-to-and-text")
            .expect("declared"),
    )
    .expect("guarded");
    let ess_primitives::predicate::Predicate::Any(disjuncts) = guard else {
        panic!("the guard is a disjunction: {guard}")
    };

    let refusing: Vec<Invocation> = invocations(&synthesis)
        .into_iter()
        .filter(|invocation| {
            invocation.command == SEND_COMMAND && invocation.expected == "requires-to-and-text"
        })
        .collect();
    for (index, disjunct) in disjuncts.iter().enumerate() {
        let alone = refusing.iter().any(|invocation| {
            let facts = flatten(&ir, command, &invocation.input).expect("fits");
            disjuncts.iter().enumerate().all(|(other, candidate)| {
                let decided = facts.decide(candidate);
                if other == index {
                    decided == Decision::Satisfied
                } else {
                    matches!(decided, Decision::Refuted(_))
                }
            })
        });
        assert!(
            alone,
            "no invocation requires `requires-to-and-text` with `{disjunct}` holding alone; the \
             suite sends {:?}",
            refusing
                .iter()
                .map(|invocation| &invocation.input)
                .collect::<Vec<_>>()
        );
    }
}

/// Each conjunct of `all:` is witnessed failing alone, and the default is required there.
#[test]
fn a_conjunctive_guard_is_refuted_once_per_conjunct_with_the_others_true() {
    let model = send("all");
    let ir = compiled(&model);
    let synthesis = synthesize(&ir);
    let command = command(&ir, SEND_COMMAND);
    let guard = when(
        command
            .outcomes
            .iter()
            .find(|outcome| outcome.name.as_str() == "requires-to-and-text")
            .expect("declared"),
    )
    .expect("guarded");
    let ess_primitives::predicate::Predicate::All(conjuncts) = guard else {
        panic!("the guard is a conjunction: {guard}")
    };

    let defaults: Vec<Invocation> = invocations(&synthesis)
        .into_iter()
        .filter(|invocation| invocation.command == SEND_COMMAND && invocation.expected == "sent")
        .collect();
    for (index, conjunct) in conjuncts.iter().enumerate() {
        let alone = defaults.iter().any(|invocation| {
            let facts = flatten(&ir, command, &invocation.input).expect("fits");
            conjuncts.iter().enumerate().all(|(other, candidate)| {
                let decided = facts.decide(candidate);
                if other == index {
                    matches!(decided, Decision::Refuted(_))
                } else {
                    decided == Decision::Satisfied
                }
            })
        });
        assert!(
            alone,
            "no invocation requires `sent` with only `{conjunct}` failing; the suite sends {:?}",
            defaults
                .iter()
                .map(|invocation| &invocation.input)
                .collect::<Vec<_>>()
        );
    }
}

/// `any` → `all` and `all` → `any`: each mutant's suite holds a scenario the original fails.
#[test]
fn both_connective_mutants_synthesize_a_scenario_the_unmutated_model_contradicts() {
    for connective in ["any", "all"] {
        let model = send(connective);
        let original = compiled(&model);
        let unmutated = synthesize(&original);
        assert!(
            contradicted(&unmutated, &original, SEND_COMMAND).is_empty(),
            "the unmutated suite agrees with its own model"
        );
        let mutants = mutants_of(&model, MutantClass::GuardConnective);
        assert_eq!(mutants.len(), 1, "one connective, one mutant");
        for (id, mutated) in mutants {
            let suite = synthesize(&mutated);
            assert_ne!(
                ids(&suite)
                    .iter()
                    .map(|id| steps(&suite, id).to_vec())
                    .collect::<Vec<_>>(),
                ids(&unmutated)
                    .iter()
                    .map(|id| steps(&unmutated, id).to_vec())
                    .collect::<Vec<_>>(),
                "`{id}` changes the synthesized suite"
            );
            let killers = contradicted(&suite, &original, SEND_COMMAND);
            assert!(
                !killers.is_empty(),
                "`{id}` (from `{connective}`) survives: no scenario of its suite requires a branch \
                 the original does not take"
            );
        }
    }
}

// ---- #154: a source state dropped from an all-states transition ----------------------------------

/// `observe_in_progress` starts from every state `Membership` declares. The command also closes a
/// `Session`, which is what lets it declare a `wrong_state:` answer at all, and what makes the
/// mutation audit enumerate a `from-drop` site for the transition.
const MEMBERSHIP: &str = r"
format: ess/13
system: club
version: v1
domain: club.members

types:
  - {name: club.members.MemberId, kind: newtype, of: Uuid}
  - {name: club.members.SessionId, kind: newtype, of: Uuid}
  - {name: club.members.Kind, kind: enum, variants: [Progress, Close]}

entities:
  - name: club.members.Membership
    identity: {name: member_id, type: club.members.MemberId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Waiting
      states: [Waiting, InProgress]
      terminal: [InProgress]
      transitions:
        - {name: start, from: [Waiting], to: InProgress}
        - {name: observe_in_progress, from: [Waiting, InProgress], to: InProgress}
  - name: club.members.Session
    identity: {name: session_id, type: club.members.SessionId}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}

errors:
  - name: club.members.StateConflict
    summary: Not in a state this command acts from.
    fields: []

events:
  - name: club.members.Joined
    fields:
      - {name: member_id, type: club.members.MemberId}
  - name: club.members.Opened
    fields:
      - {name: session_id, type: club.members.SessionId}
  - name: club.members.Observed
    fields:
      - {name: member_id, type: club.members.MemberId}
  - name: club.members.Closed
    fields:
      - {name: session_id, type: club.members.SessionId}

views:
  - name: club.members.MembershipState
    source: club.members.Membership
    consistency: eventual
    fields:
      - {name: member_id, type: club.members.MemberId}
      - {name: state, type: club.members.Membership.State}
  - name: club.members.SessionState
    source: club.members.Session
    consistency: read_your_writes
    fields:
      - {name: session_id, type: club.members.SessionId}
      - {name: state, type: club.members.Session.State}

commands:
  - name: club.members.Join
    input:
      - {name: note, type: String}
    outcomes:
      - name: joined
        creates: club.members.Membership
        instance: member_id
        emits: [club.members.Joined]
        sets: {note: input.note}
        payload:
          club.members.Joined: {member_id: {generated: true}}
  - name: club.members.Open
    outcomes:
      - name: opened
        creates: club.members.Session
        instance: session_id
        emits: [club.members.Opened]
        payload:
          club.members.Opened: {session_id: {generated: true}}
  - name: club.members.Start
    input:
      - {name: member_id, type: club.members.MemberId}
    outcomes:
      - name: started
        moves: club.members.Membership.start
        instance: member_id
        emits: [club.members.Observed]
        payload:
          club.members.Observed: {member_id: input.member_id}
  - name: club.members.Observe
    input:
      - {name: kind, type: club.members.Kind}
      - {name: member_id, type: club.members.MemberId}
      - {name: session_id, type: club.members.SessionId}
    outcomes:
      - name: observed
        when: kind == Progress
        moves: club.members.Membership.observe_in_progress
        instance: member_id
        emits: [club.members.Observed]
        payload:
          club.members.Observed: {member_id: input.member_id}
      - name: closed
        when: kind == Close
        moves: club.members.Session.close
        instance: session_id
        emits: [club.members.Closed]
        payload:
          club.members.Closed: {session_id: input.session_id}
      - {name: wrong-state, wrong_state: true, error: club.members.StateConflict}
";

/// Dropping a source from an all-states `from:` gets a refusal scenario the original fails.
#[test]
fn a_state_dropped_from_an_all_states_transition_gets_a_refusal_the_original_contradicts() {
    let model = documents(MEMBERSHIP);
    let original = compiled(&model);
    let unmutated = synthesize(&original);
    assert!(
        ids(&unmutated)
            .iter()
            .any(|id| id == "club.members.Session/state/Closed/refuses/club.members.Observe"),
        "the other entity's non-source state has its refusal: {:?}",
        ids(&unmutated)
    );

    let mutants = mutants_of(&model, MutantClass::FromDrop);
    let dropped_waiting = mutants
        .iter()
        .find(|(id, _)| id == "from-drop/club.members.Membership.observe_in_progress/Waiting")
        .unwrap_or_else(|| {
            panic!(
                "the audit enumerates the site: {:?}",
                mutants.iter().map(|(id, _)| id).collect::<Vec<_>>()
            )
        });
    let suite = synthesize(&dropped_waiting.1);
    let id = "club.members.Membership/state/Waiting/refuses/club.members.Observe";
    let scenario = steps(&suite, id);

    // The command is sent to a membership resting in `Waiting`, and the wrong-state branch is
    // required. The original lets `observe_in_progress` start from `Waiting`, so an implementation
    // of it takes `observed` there and fails this scenario.
    let required: Vec<String> = invocations(&suite)
        .into_iter()
        .filter(|invocation| {
            invocation.scenario == id && invocation.command == "club.members.Observe"
        })
        .map(|invocation| invocation.expected)
        .collect();
    assert_eq!(required, vec!["wrong-state".to_owned()], "{scenario:#?}");
    let membership = original
        .entities()
        .values()
        .find(|entity| entity.name.to_string() == "club.members.Membership")
        .expect("declared");
    assert!(
        membership
            .lifecycle
            .transitions
            .iter()
            .any(|transition| transition.name == "observe_in_progress"
                && transition
                    .from
                    .iter()
                    .any(|state| state.as_str() == "Waiting")),
        "the original moves from `Waiting`"
    );
}

// ---- #132: wrong-state preservation over an eventual identity/state view --------------------------

/// The repro from beyond10x/ess#132, verbatim but for the `system:` header folded into the file.
const ORDERS: &str = r"
format: ess/13
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
  - {name: demo.orders.Note, kind: newtype, of: String}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: note, type: demo.orders.Note}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.OpenOrder, demo.orders.CloseOrder]}
errors:
  - name: demo.orders.OrderStateConflict
    summary: Already closed.
    fields:
      - {name: state, type: demo.orders.Order.State}
commands:
  - name: demo.orders.OpenOrder
    input:
      - {name: note, type: demo.orders.Note}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.OrderOpened]
        payload:
          demo.orders.OrderOpened: {note: input.note, order_id: {generated: true}}
  - name: demo.orders.CloseOrder
    input:
      - {name: order_id, type: demo.orders.OrderId}
    outcomes:
      - name: closed
        moves: demo.orders.Order.close
        instance: order_id
        emits: [demo.orders.OrderClosed]
        payload:
          demo.orders.OrderClosed: {order_id: input.order_id}
      - {name: wrong-state, wrong_state: true, error: demo.orders.OrderStateConflict}
events:
  - name: demo.orders.OrderOpened
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: note, type: demo.orders.Note}
  - name: demo.orders.OrderClosed
    fields:
      - {name: order_id, type: demo.orders.OrderId}
VIEWS
";

const ORDER_STATE_VIEW: &str = r"views:
  - name: demo.orders.OrderState
    source: demo.orders.Order
    consistency: eventual
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: state, type: demo.orders.Order.State}
";

const TERMINAL_REFUSAL: &str = "demo.orders.Order/state/Closed/refuses/demo.orders.CloseOrder";

/// The terminal refusal is synthesized, observing what the eventual view publishes.
#[test]
fn a_terminal_refusal_is_witnessed_over_the_fields_an_eventual_view_publishes() {
    let ir = compiled(&documents(&ORDERS.replace("VIEWS", ORDER_STATE_VIEW)));
    let synthesis = synthesize(&ir);
    let scenario = steps(&synthesis, TERMINAL_REFUSAL);

    let invoked = scenario
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == "demo.orders.CloseOrder"))
        .expect("the refused command is sent");
    let observed: Vec<&BTreeMap<String, ScenarioValue>> = scenario[invoked..]
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::EventuallyView {
                view,
                expectation: ViewExpectation::Contains { fields },
                ..
            } if view.to_string() == "demo.orders.OrderState" => Some(fields),
            _ => None,
        })
        .collect();
    assert_eq!(
        observed.len(),
        1,
        "after the refusal, the eventual view is required to hold the order where it was: \
         {scenario:#?}"
    );
    let row = observed[0];
    assert!(
        matches!(row.get("order_id"), Some(ScenarioValue::Instance { .. })),
        "the row is the arranged order: {row:?}"
    );
    assert_eq!(
        row.get("state"),
        Some(&ScenarioValue::literal(Node::Text("Closed".into()))),
        "and it is still `Closed`: {row:?}"
    );
    assert_eq!(
        row.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["order_id", "state"],
        "exactly the fields the view publishes, and no claim about `note`"
    );
    assert!(
        synthesis.notes.iter().any(|note| matches!(
            note,
            Note::PartialObservation { scenario, unobserved }
                if scenario.to_string() == TERMINAL_REFUSAL && unobserved == &["note".to_owned()]
        )),
        "the partial observation is recorded, naming what it leaves out: {:?}",
        synthesis.notes
    );
    assert!(
        !synthesis.refusals.iter().any(|refusal| refusal
            .scenario
            .as_ref()
            .is_some_and(|scenario| scenario.to_string() == TERMINAL_REFUSAL)),
        "{:?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}

/// With no view of the entity at all the refusal stays, and its `help:` points at views.
#[test]
fn a_terminal_refusal_nothing_can_observe_points_at_views() {
    let ir = compiled(&documents(&ORDERS.replace("VIEWS", "")));
    let synthesis = synthesize(&ir);
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == TERMINAL_REFUSAL)
        })
        .unwrap_or_else(|| {
            panic!(
                "the refusal stays: {:?} {:?}",
                ids(&synthesis),
                synthesis
                    .refusals
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        });
    let hint = refusal.hint();
    assert!(
        hint.contains("view"),
        "the help line points at views: {hint}"
    );
    assert!(
        !hint.contains("finite value"),
        "and not at value types: {hint}"
    );
}

// ---- #160: a count boundary witnessed from both sides ---------------------------------------------

const DIGITS: &str = r"
format: ess/13
system: dial
version: v1
domain: dial.keys

events:
  - name: dial.keys.KeysSent
    fields:
      - {name: digits, type: String}

errors:
  - name: dial.keys.DigitsRefused
    summary: The keys are refused for their length.
    fields: []

commands:
  - name: dial.keys.SendKeys
    input:
      - {name: digits, type: String}
    outcomes:
      - name: refused
        when: digits.count OP LITERAL
        error: dial.keys.DigitsRefused
      - name: sent
        emits: [dial.keys.KeysSent]
        payload:
          dial.keys.KeysSent: {digits: input.digits}
";

const SEND_KEYS: &str = "dial.keys.SendKeys";

fn digits(op: &str, literal: u32) -> EssIr {
    compiled(&documents(
        &DIGITS
            .replace("OP", op)
            .replace("LITERAL", &literal.to_string()),
    ))
}

/// The lengths of `digits` the suite sends with `branch` required.
fn lengths(synthesis: &Synthesis, branch: &str) -> Vec<usize> {
    let mut found: Vec<usize> = invocations(synthesis)
        .into_iter()
        .filter(|invocation| invocation.command == SEND_KEYS && invocation.expected == branch)
        .filter_map(|invocation| match invocation.input.get("digits") {
            Some(Node::Text(text)) => Some(text.chars().count()),
            _ => None,
        })
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

/// `digits.count > 64` is witnessed at 64 accepted and at 65 refused: both sides of the boundary.
#[test]
fn a_count_boundary_is_witnessed_on_both_sides_of_the_literal() {
    let synthesis = synthesize(&digits(">", 64));
    assert!(
        lengths(&synthesis, "sent").contains(&64),
        "the accepting side is witnessed exactly at the boundary: {:?}",
        lengths(&synthesis, "sent")
    );
    assert!(
        lengths(&synthesis, "refused").contains(&65),
        "and the refusing side just past it: {:?}",
        lengths(&synthesis, "refused")
    );
}

/// Every operator, the literal moved one either way: the moved suite holds a scenario the original
/// model answers differently.
#[test]
fn a_count_boundary_moved_either_way_is_killed_for_every_ordering_operator() {
    for op in [">", ">=", "<", "<="] {
        let original = digits(op, 64);
        assert!(
            contradicted(&synthesize(&original), &original, SEND_KEYS).is_empty(),
            "`{op} 64` agrees with itself"
        );
        for moved in [63, 65] {
            let suite = synthesize(&digits(op, moved));
            assert!(
                !contradicted(&suite, &original, SEND_KEYS).is_empty(),
                "`digits.count {op} {moved}` survives against `{op} 64`: refused at {:?}, sent at \
                 {:?}",
                lengths(&suite, "refused"),
                lengths(&suite, "sent")
            );
        }
    }
}

// ---- #161: a `sets:` source told apart from its same-typed siblings -------------------------------

const CALLS: &str = r"
format: ess/13
system: rec
version: v1
domain: rec.calls

types:
  - {name: rec.calls.CallId, kind: newtype, of: Uuid}

entities:
  - name: rec.calls.Call
    identity: {name: call_id, type: rec.calls.CallId}
    fields:
      - {name: anonymous, type: Boolean}
      - {name: recording, type: Boolean}
    lifecycle:
      initial: Live
      states: [Live]
      terminal: [Live]

events:
  - name: rec.calls.CallStarted
    fields:
      - {name: call_id, type: rec.calls.CallId}

errors:
  - name: rec.calls.NotRecorded
    summary: Only recorded calls are started here.
    fields: []

views:
  - name: rec.calls.Calls
    source: rec.calls.Call
    consistency: read_your_writes
    fields:
      - {name: call_id, type: rec.calls.CallId}
      - {name: state, type: rec.calls.Call.State}
      - {name: anonymous, type: Boolean}
      - {name: recording, type: Boolean}

commands:
  - name: rec.calls.StartCall
    input:
      - {name: recording, type: Boolean}
      - {name: anonymous, type: Boolean}
    outcomes:
      - name: started
        when: recording == true
        creates: rec.calls.Call
        instance: call_id
        sets: {anonymous: input.anonymous, recording: input.recording}
        emits: [rec.calls.CallStarted]
        payload:
          rec.calls.CallStarted: {call_id: {generated: true}}
      - name: unrecorded
        error: rec.calls.NotRecorded
";

const STARTED: &str = "rec.calls.StartCall/outcome/started";

/// The two Booleans the scenario sends, `(recording, anonymous)`.
fn booleans(synthesis: &Synthesis) -> (Node, Node) {
    let sent = invocations(synthesis)
        .into_iter()
        .find(|invocation| invocation.scenario == STARTED && invocation.expected == "started")
        .expect("the branch is invoked");
    (
        sent.input.get("recording").cloned().expect("sent"),
        sent.input.get("anonymous").cloned().expect("sent"),
    )
}

/// `anonymous: input.anonymous` beside a same-typed `recording`: the witness gives them different
/// values, so an implementation writing the sibling is told apart — in the unmutated suite, and in
/// the suite of each `sets-retarget` mutant, which is what kills it.
#[test]
fn a_sets_source_is_witnessed_apart_from_its_same_typed_siblings() {
    let model = documents(CALLS);
    let original = compiled(&model);
    let (recording, anonymous) = booleans(&synthesize(&original));
    assert_ne!(
        recording, anonymous,
        "the unmutated suite sends the two Booleans apart"
    );
    let mutants = mutants_of(&model, MutantClass::SetsRetarget);
    assert!(!mutants.is_empty(), "the audit finds the retargets");
    for (id, mutated) in mutants {
        let (recording, anonymous) = booleans(&synthesize(&mutated));
        assert_ne!(
            recording, anonymous,
            "`{id}` is synthesized with the two Booleans equal, so it survives"
        );
    }
}

// ---- #202: three same-typed `sets:` sources, and the pair a retarget joins -------------------------

const FLAGS: &str = r"
format: ess/13
system: shop
version: v1
domain: shop.order

entities:
  - name: shop.order.Order
    identity: {name: order_id, type: String}
    fields:
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

events:
  - name: shop.order.OrderPlaced
    fields:
      - {name: order_id, type: String}

views:
  - name: shop.order.Orders
    source: shop.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: String}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}

commands:
  - name: shop.order.PlaceOrder
    input:
      - {name: order_id, type: String}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}
    outcomes:
      - name: placed
        creates: shop.order.Order
        instance: order_id
        sets: {paid: input.paid, gift: input.gift, rush: input.rush}
        emits: [shop.order.OrderPlaced]
        payload:
          shop.order.OrderPlaced: {order_id: input.order_id}
";

const PLACED: &str = "shop.order.PlaceOrder/outcome/placed";
const FLAG_FIELDS: [&str; 3] = ["paid", "gift", "rush"];

/// The input the `placed` scenario sends, and every `Orders` row it then requires.
fn placed(synthesis: &Synthesis) -> (BTreeMap<String, Node>, Vec<BTreeMap<String, Node>>) {
    let sent = invocations(synthesis)
        .into_iter()
        .find(|invocation| invocation.scenario == PLACED && invocation.expected == "placed")
        .expect("the branch is invoked");
    let rows = steps(synthesis, PLACED)
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                view,
                expectation: ViewExpectation::Contains { fields },
            }
            | ScenarioStep::EventuallyView {
                view,
                expectation: ViewExpectation::Contains { fields },
                ..
            } if view.to_string() == "shop.order.Orders" => Some(
                fields
                    .iter()
                    .filter_map(|(field, value)| {
                        value
                            .as_literal()
                            .cloned()
                            .map(|node| (field.clone(), node))
                    })
                    .collect(),
            ),
            _ => None,
        })
        .collect();
    (sent.input, rows)
}

/// Each `sets-retarget` mutant of three Booleans written to three fields gets a witness that sends
/// the pair it joins apart, and so a row the unmutated specification contradicts: the original
/// writes every flag from its own input, so a required row with a flag other than that input's
/// value is one every implementation of the original fails.
#[test]
fn a_retargeted_sets_pair_is_witnessed_apart_among_three_same_typed_sources() {
    let model = documents(FLAGS);
    let mutants = mutants_of(&model, MutantClass::SetsRetarget);
    assert_eq!(
        mutants.len(),
        3,
        "the audit retargets each of the three flags: {:?}",
        mutants.iter().map(|(id, _)| id).collect::<Vec<_>>()
    );
    let mut survivors = Vec::new();
    for (id, mutated) in mutants {
        let (input, rows) = placed(&synthesize(&mutated));
        assert!(!rows.is_empty(), "`{id}` requires an `Orders` row");
        let killed = rows.iter().any(|row| {
            FLAG_FIELDS.iter().any(|flag| {
                row.get(*flag)
                    .is_some_and(|held| input.get(*flag) != Some(held))
            })
        });
        if !killed {
            survivors.push(format!("`{id}` sends {input:?} and requires {rows:?}"));
        }
    }
    assert!(
        survivors.is_empty(),
        "these retargets are witnessed with the joined pair equal, so they survive: {survivors:#?}"
    );
}

/// The notes of `synthesis` saying a scenario sends a joined pair with one value, as
/// `(scenario, source, unread)`.
fn unseparated(synthesis: &Synthesis) -> Vec<(String, String, String)> {
    synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnseparatedSources {
                scenario,
                source,
                unread,
            } => Some((scenario.to_string(), source.clone(), unread.clone())),
            _ => None,
        })
        .collect()
}

/// Guards that fix both inputs of the joined pair to one value leave no witness that separates
/// them, and the scenario says so in a note naming the pair instead of dropping it silently. The
/// declared model joins nothing, so it carries no such note.
#[test]
fn a_joined_pair_the_guards_hold_equal_is_named_in_a_note() {
    let guarded = FLAGS.replace(
        "      - name: placed\n",
        "      - name: placed\n        when:\n          all: [paid == true, gift == true]\n",
    ) + r"
      - name: refused
        error: shop.order.NotPlaced

errors:
  - name: shop.order.NotPlaced
    summary: Only paid gifts are placed here.
    fields: []
";
    let model = documents(&guarded);
    assert_eq!(
        unseparated(&synthesize(&compiled(&model))),
        Vec::<(String, String, String)>::new(),
        "the declared model joins no pair"
    );
    let (id, mutated) = mutants_of(&model, MutantClass::SetsRetarget)
        .into_iter()
        .find(|(id, _)| id.ends_with("/placed/gift"))
        .expect("the audit retargets `gift`");
    assert_eq!(
        unseparated(&synthesize(&mutated)),
        vec![(PLACED.to_owned(), "paid".to_owned(), "gift".to_owned())],
        "`{id}` joins `paid` and `gift`, which the guard holds equal"
    );
}

/// A later branch writing a literal is arranged on a row that held something else.
const RECORDED: &str = r"
format: ess/13
system: rec
version: v1
domain: rec.calls

types:
  - {name: rec.calls.CallId, kind: newtype, of: Uuid}

entities:
  - name: rec.calls.Call
    identity: {name: call_id, type: rec.calls.CallId}
    fields:
      - {name: recording, type: Boolean}
    lifecycle:
      initial: Live
      states: [Live]
      terminal: [Live]

events:
  - name: rec.calls.CallStarted
    fields:
      - {name: call_id, type: rec.calls.CallId}
  - name: rec.calls.RecordingStarted
    fields:
      - {name: call_id, type: rec.calls.CallId}

views:
  - name: rec.calls.Calls
    source: rec.calls.Call
    consistency: read_your_writes
    fields:
      - {name: call_id, type: rec.calls.CallId}
      - {name: state, type: rec.calls.Call.State}
      - {name: recording, type: Boolean}

commands:
  - name: rec.calls.StartCall
    input:
      - {name: recording, type: Boolean}
    outcomes:
      - name: started
        creates: rec.calls.Call
        instance: call_id
        sets: {recording: input.recording}
        emits: [rec.calls.CallStarted]
        payload:
          rec.calls.CallStarted: {call_id: {generated: true}}
  - name: rec.calls.Record
    input:
      - {name: call_id, type: rec.calls.CallId}
    outcomes:
      - name: recorded
        updates: rec.calls.Call
        instance: call_id
        sets: {recording: true}
        emits: [rec.calls.RecordingStarted]
        payload:
          rec.calls.RecordingStarted: {call_id: input.call_id}
";

/// `sets: {recording: true}` over a row the arrangement created with `recording: true` proves
/// nothing about the write: an implementation that dropped it shows the same row. The arrangement
/// is chosen so the row held the other value before the branch ran.
#[test]
fn a_literal_write_is_arranged_over_a_row_that_held_another_value() {
    let synthesis = synthesize(&compiled(&documents(RECORDED)));
    let arranged: Vec<Node> = invocations(&synthesis)
        .into_iter()
        .filter(|invocation| {
            invocation.scenario == "rec.calls.Record/outcome/recorded"
                && invocation.command == "rec.calls.StartCall"
        })
        .filter_map(|invocation| invocation.input.get("recording").cloned())
        .collect();
    assert_eq!(
        arranged,
        vec![Node::Bool(false)],
        "the call is created not recording, so `recorded` visibly writes `true`"
    );
}

// ---- #496: a stored guard no view observes ----------------------------------------------------------

/// An order collected only by the buyer who placed it, with no view of the order at all
/// (`.engineering/repro/496/orders.yaml`).
const VIEW_LESS_ORDERS: &str = include_str!("fixtures/orders-without-a-view.yaml");

/// The refusal the stored `placed_by` selects.
const WRONG_COLLECTOR: &str = "      - name: wrong-collector
        when_subject:
          predicate: placed_by != input.collector
        error: catalog.orders.Refused
";

/// An implementation that ignores the stored guard, which no view can show the scenario, still
/// fails the view-less `wrong-collector` scenario: the outcome, the error and the events are its
/// evidence, and they need no view. The mutant drops the `when_subject:` refusal, so the model it
/// interprets collects for any buyer.
#[test]
fn a_stored_guard_no_view_observes_is_killed_through_the_outcome() {
    use ess_conformance::{interpret::Interpreted, report::Status, AdmittedSuite, Runner};

    assert!(VIEW_LESS_ORDERS.contains(WRONG_COLLECTOR));
    let synthesis = synthesize(&compiled(&documents(VIEW_LESS_ORDERS)));
    let suite = &synthesis.suite;
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let failing = |model: &str| -> Vec<String> {
        Runner::for_suite(suite)
            .run_admitted(
                &admitted,
                &Interpreted::for_model(compiled(&documents(model))),
            )
            .into_report()
            .scenarios
            .iter()
            .filter(|scenario| scenario.status != Status::Passed)
            .map(|scenario| scenario.scenario.to_string())
            .collect()
    };
    assert_eq!(failing(VIEW_LESS_ORDERS), Vec::<String>::new());
    let ignored = VIEW_LESS_ORDERS.replace(WRONG_COLLECTOR, "");
    assert!(
        failing(&ignored).contains(&"catalog.orders.Collect/outcome/wrong-collector".to_owned()),
        "the mutant ignoring the stored guard is killed by the view-less refusal: {:?}",
        failing(&ignored)
    );
}
