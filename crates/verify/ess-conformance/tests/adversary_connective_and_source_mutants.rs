//! Adversary cases for `story:synthesis-kills-connective-and-source-mutants` (beyond10x/ess#154,
//! #155, #132, #160, #161): shapes the unit's own tests do not reach.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::source::SourceMap;
use ess_conformance::mutate::{self, Document};
use ess_conformance::scenario::{ScenarioStep, ScenarioValue, ViewExpectation};
use ess_conformance::synthesize::{synthesize, Note, Synthesis};
use ess_conformance::{flatten, when, Decision};
use ess_domain::command::TestStrategy;
use ess_domain::name::QualifiedName;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn compiled(text: &str) -> EssIr {
    let (files, texts) = documents(text);
    mutate::compile(files, &texts).unwrap_or_else(|stillborn| {
        panic!("the model compiles: {} {}", stillborn.code, stillborn.cause)
    })
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// One invocation of `command` that a scenario requires to take a branch.
struct Invocation {
    scenario: String,
    command: String,
    input: BTreeMap<String, Node>,
    expected: String,
    /// The literal input of the most recent earlier invocation of each other command in the
    /// same scenario (the arrangement the row was built by).
    arranged: BTreeMap<String, BTreeMap<String, Node>>,
}

fn invocations(synthesis: &Synthesis) -> Vec<Invocation> {
    let mut out = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut arranged: BTreeMap<String, BTreeMap<String, Node>> = BTreeMap::new();
        let mut pending: Option<(String, BTreeMap<String, Node>)> = None;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand { command, input, .. } => {
                    if let Some((command, input)) = pending.take() {
                        arranged.insert(command, input);
                    }
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
                            command: command.clone(),
                            input: input.clone(),
                            expected,
                            arranged: arranged.clone(),
                        });
                        arranged.insert(command, input);
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
        .expect("a default")
}

fn contradicted(synthesis: &Synthesis, original: &EssIr, name: &str) -> Vec<String> {
    invocations(synthesis)
        .into_iter()
        .filter(|invocation| invocation.command == name)
        .filter(|invocation| selected(original, name, &invocation.input) != invocation.expected)
        .map(|invocation| format!("{} {:?}", invocation.scenario, invocation.input))
        .collect()
}

fn sent(synthesis: &Synthesis, name: &str) -> Vec<(String, BTreeMap<String, Node>)> {
    invocations(synthesis)
        .into_iter()
        .filter(|invocation| invocation.command == name)
        .map(|invocation| (invocation.expected, invocation.input))
        .collect()
}

fn number(node: Option<&Node>) -> f64 {
    match node {
        Some(Node::Number(number)) => number.get(),
        other => panic!("not a number: {other:?}"),
    }
}

// ---- #155 over stored fields: `any:` in a `when_subject` predicate ---------------------------------

const PARCELS: &str = r"
format: ess/13
system: shipping
version: v1
domain: shipping.parcel
types:
  - {name: shipping.parcel.Service, kind: enum, variants: [Standard, Express]}
  - {name: shipping.parcel.ParcelId, kind: newtype, of: Uuid}
entities:
  - name: shipping.parcel.Parcel
    identity: {name: parcel_id, type: shipping.parcel.ParcelId}
    fields:
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
    lifecycle:
      initial: Created
      states: [Created, Dispatched]
      terminal: [Dispatched]
      transitions:
        - {name: dispatch, from: [Created], to: Dispatched}
events:
  - name: shipping.parcel.Created
    fields:
      - {name: parcel_id, type: shipping.parcel.ParcelId}
  - name: shipping.parcel.Dispatched
    fields: []
errors:
  - name: shipping.parcel.Refused
    summary: Refused.
    fields: []
commands:
  - name: shipping.parcel.Create
    input:
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
    outcomes:
      - name: created
        creates: shipping.parcel.Parcel
        instance: parcel_id
        sets: {service: input.service, weight_kg: input.weight_kg}
        emits: [shipping.parcel.Created]
        payload:
          shipping.parcel.Created:
            parcel_id: {generated: true}
  - name: shipping.parcel.Dispatch
    input:
      - {name: parcel_id, type: shipping.parcel.ParcelId}
    outcomes:
      - name: refused
        when_subject:
          predicate:
            CONNECTIVE:
              - service == Express
              - weight_kg > 20
        error: shipping.parcel.Refused
      - name: dispatched
        moves: shipping.parcel.Parcel.dispatch
        instance: parcel_id
        emits: [shipping.parcel.Dispatched]
views:
  - name: shipping.parcel.Parcels
    source: shipping.parcel.Parcel
    consistency: read_your_writes
    fields:
      - {name: parcel_id, type: shipping.parcel.ParcelId}
      - {name: state, type: shipping.parcel.Parcel.State}
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
";

/// The rows (service, weight) a `Dispatch` requiring `branch` was sent against.
fn rows(synthesis: &Synthesis, branch: &str) -> Vec<(String, f64)> {
    invocations(synthesis)
        .into_iter()
        .filter(|invocation| {
            invocation.command == "shipping.parcel.Dispatch" && invocation.expected == branch
        })
        .filter_map(|invocation| {
            let create = invocation.arranged.get("shipping.parcel.Create")?;
            let service = match create.get("service")? {
                Node::Text(text) => text.clone(),
                other => format!("{other:?}"),
            };
            Some((service, number(create.get("weight_kg"))))
        })
        .collect()
}

/// `any: [service == Express, weight_kg > 20]` over stored fields: the refusal is witnessed on a row
/// where each disjunct holds alone, so a `guard-connective` mutant to `all` changes what is required.
#[test]
fn adversary_stored_field_any_guard_is_witnessed_once_per_disjunct() {
    let synthesis = synthesize(&compiled(&PARCELS.replace("CONNECTIVE", "any")));
    let refused = rows(&synthesis, "refused");
    assert!(
        !refused.is_empty(),
        "the refusal is witnessed at all: {:?}",
        refusals(&synthesis)
    );
    let express_alone = refused
        .iter()
        .any(|(service, weight)| service == "Express" && *weight <= 20.0);
    let heavy_alone = refused
        .iter()
        .any(|(service, weight)| service == "Standard" && *weight > 20.0);
    assert!(
        express_alone && heavy_alone,
        "each disjunct of the stored-field `any` holds alone on some refused row \
         (Express <= 20: {express_alone}, Standard > 20: {heavy_alone}); refused rows: {refused:?}"
    );
}

// ---- #160 on a plain numeric guard ----------------------------------------------------------------

const AMOUNT: &str = r"
format: ess/13
system: pay
version: v1
domain: pay.transfer
events:
  - name: pay.transfer.Sent
    fields:
      - {name: amount, type: Integer}
errors:
  - name: pay.transfer.Refused
    summary: Refused for its amount.
    fields: []
commands:
  - name: pay.transfer.Send
    input:
      - {name: amount, type: Integer}
    outcomes:
      - name: refused
        when: amount OP LITERAL
        error: pay.transfer.Refused
      - name: sent
        emits: [pay.transfer.Sent]
        payload:
          pay.transfer.Sent: {amount: input.amount}
";

/// `amount OP 64` moved to 63 or 65, every ordering operator: the moved suite holds an invocation
/// the original answers with the other branch.
#[test]
fn adversary_numeric_boundary_moved_either_way_is_killed_for_every_ordering_operator() {
    let mut survivors = Vec::new();
    for op in [">", ">=", "<", "<="] {
        let model = |literal: i64| {
            compiled(
                &AMOUNT
                    .replace("OP", op)
                    .replace("LITERAL", &literal.to_string()),
            )
        };
        let original = model(64);
        assert!(
            contradicted(&synthesize(&original), &original, "pay.transfer.Send").is_empty(),
            "`{op} 64` agrees with itself"
        );
        for moved in [63, 65] {
            let suite = synthesize(&model(moved));
            if contradicted(&suite, &original, "pay.transfer.Send").is_empty() {
                survivors.push(format!(
                    "`amount {op} {moved}` survives `{op} 64`: sends {:?}",
                    sent(&suite, "pay.transfer.Send")
                ));
            }
        }
    }
    assert!(survivors.is_empty(), "{survivors:#?}");
}

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
    summary: Refused for their length.
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

/// A count boundary near zero: `digits.count < 1` / `<= 1` / `> 1` / `>= 1`, moved one either way
/// (where the moved literal is not negative).
#[test]
fn adversary_count_boundary_near_zero_is_killed() {
    let mut survivors = Vec::new();
    for op in [">", ">=", "<", "<="] {
        let model = |literal: u32| {
            compiled(
                &DIGITS
                    .replace("OP", op)
                    .replace("LITERAL", &literal.to_string()),
            )
        };
        let original = model(1);
        assert!(
            contradicted(&synthesize(&original), &original, "dial.keys.SendKeys").is_empty(),
            "`{op} 1` agrees with itself"
        );
        for moved in [0, 2] {
            let suite = synthesize(&model(moved));
            if contradicted(&suite, &original, "dial.keys.SendKeys").is_empty() {
                survivors.push(format!(
                    "`digits.count {op} {moved}` survives `{op} 1`: sends {:?}",
                    sent(&suite, "dial.keys.SendKeys")
                ));
            }
        }
    }
    assert!(survivors.is_empty(), "{survivors:#?}");
}

// ---- #161 on enums and more than two same-typed inputs -------------------------------------------

const TAGS: &str = r"
format: ess/13
system: tag
version: v1
domain: tag.items
types:
  - {name: tag.items.ItemId, kind: newtype, of: Uuid}
  - {name: tag.items.Colour, kind: enum, variants: [Red, Green, Blue]}
entities:
  - name: tag.items.Item
    identity: {name: item_id, type: tag.items.ItemId}
    fields:
      - {name: a, type: TYPE}
      - {name: b, type: TYPE}
      - {name: c, type: TYPE}
    lifecycle:
      initial: Live
      states: [Live]
      terminal: [Live]
events:
  - name: tag.items.Tagged
    fields:
      - {name: item_id, type: tag.items.ItemId}
views:
  - name: tag.items.Items
    source: tag.items.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: tag.items.ItemId}
      - {name: state, type: tag.items.Item.State}
      - {name: a, type: TYPE}
      - {name: b, type: TYPE}
      - {name: c, type: TYPE}
commands:
  - name: tag.items.Tag
    input:
      - {name: a, type: TYPE}
      - {name: b, type: TYPE}
      - {name: c, type: TYPE}
    outcomes:
      - name: tagged
        creates: tag.items.Item
        instance: item_id
        sets: {a: input.a, b: input.b, c: input.c}
        emits: [tag.items.Tagged]
        payload:
          tag.items.Tagged: {item_id: {generated: true}}
";

/// Three same-typed inputs, each written to its own field: every pair is sent apart, so any
/// `sets-retarget` of one entry to either sibling changes the row.
#[test]
fn adversary_three_same_typed_sources_are_sent_pairwise_apart() {
    let mut equal = Vec::new();
    for type_ref in ["String", "tag.items.Colour", "Integer"] {
        let synthesis = synthesize(&compiled(&TAGS.replace("TYPE", type_ref)));
        let sent = sent(&synthesis, "tag.items.Tag");
        let Some((_, input)) = sent
            .iter()
            .find(|(expected, _)| expected == "tagged")
            .cloned()
        else {
            panic!("{type_ref}: tagged is invoked: {:?}", refusals(&synthesis));
        };
        for (left, right) in [("a", "b"), ("a", "c"), ("b", "c")] {
            if input.get(left) == input.get(right) {
                equal.push(format!("{type_ref}: `{left}` == `{right}` in {input:?}"));
            }
        }
    }
    assert!(equal.is_empty(), "{equal:#?}");
}

/// The routed (stored-field guard) path: an `updates` default writing two same-typed inputs beside
/// a `when_subject` refusal. The written sources are still sent apart.
const LABELS: &str = r"
format: ess/13
system: shipping
version: v1
domain: shipping.label
types:
  - {name: shipping.label.ParcelId, kind: newtype, of: Uuid}
entities:
  - name: shipping.label.Parcel
    identity: {name: parcel_id, type: shipping.label.ParcelId}
    fields:
      - {name: weight_kg, type: Integer}
      - {name: sender, type: String}
      - {name: receiver, type: String}
    lifecycle:
      initial: Live
      states: [Live]
      terminal: [Live]
events:
  - name: shipping.label.Created
    fields:
      - {name: parcel_id, type: shipping.label.ParcelId}
  - name: shipping.label.Labelled
    fields: []
errors:
  - name: shipping.label.Heavy
    summary: Too heavy.
    fields: []
views:
  - name: shipping.label.Parcels
    source: shipping.label.Parcel
    consistency: read_your_writes
    fields:
      - {name: parcel_id, type: shipping.label.ParcelId}
      - {name: state, type: shipping.label.Parcel.State}
      - {name: weight_kg, type: Integer}
      - {name: sender, type: String}
      - {name: receiver, type: String}
commands:
  - name: shipping.label.Create
    input:
      - {name: weight_kg, type: Integer}
    outcomes:
      - name: created
        creates: shipping.label.Parcel
        instance: parcel_id
        sets: {weight_kg: input.weight_kg}
        emits: [shipping.label.Created]
        payload:
          shipping.label.Created:
            parcel_id: {generated: true}
  - name: shipping.label.Label
    input:
      - {name: parcel_id, type: shipping.label.ParcelId}
      - {name: sender, type: String}
      - {name: receiver, type: String}
    outcomes:
      - name: heavy
        when_subject:
          predicate:
            all:
              - weight_kg > 20
        error: shipping.label.Heavy
      - name: labelled
        updates: shipping.label.Parcel
        instance: parcel_id
        sets: {sender: input.sender, receiver: input.receiver}
        emits: [shipping.label.Labelled]
";

#[test]
fn adversary_same_typed_sources_are_sent_apart_behind_a_stored_field_guard() {
    let synthesis = synthesize(&compiled(LABELS));
    let labelled: Vec<BTreeMap<String, Node>> = sent(&synthesis, "shipping.label.Label")
        .into_iter()
        .filter(|(expected, _)| expected == "labelled")
        .map(|(_, input)| input)
        .collect();
    assert!(
        !labelled.is_empty(),
        "labelled is invoked: {:?}",
        refusals(&synthesis)
    );
    for input in &labelled {
        assert_ne!(
            input.get("sender"),
            input.get("receiver"),
            "a `sets-retarget` of `sender` to `receiver` passes this invocation: {input:?}"
        );
    }
}

// ---- #132: a published field besides identity and state -------------------------------------------

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
      - {name: secret, type: demo.orders.Note}
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
      - {name: secret, type: demo.orders.Note}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        sets: {note: input.note, secret: input.secret}
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
views:
  - name: demo.orders.OrderNotes
    source: demo.orders.Order
    consistency: eventual
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: state, type: demo.orders.Order.State}
      - {name: note, type: demo.orders.Note}
";

const TERMINAL_REFUSAL: &str = "demo.orders.Order/state/Closed/refuses/demo.orders.CloseOrder";

/// The eventual view publishes `note` as well: the refusal requires `note` unchanged beside identity
/// and state, so an implementation that rewrote `note` and `secret` on refusal fails; only `secret`
/// is named unobserved.
#[test]
fn adversary_partial_observation_requires_every_published_field() {
    let synthesis = synthesize(&compiled(ORDERS));
    let scenario = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == TERMINAL_REFUSAL)
        .map_or_else(
            || panic!("{:?}", refusals(&synthesis)),
            |(_, scenario)| scenario,
        );
    let invoked = scenario
        .steps
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == "demo.orders.CloseOrder"))
        .expect("sent");
    let rows: Vec<&BTreeMap<String, ScenarioValue>> = scenario.steps[invoked..]
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields),
            _ => None,
        })
        .collect();
    assert!(
        rows.iter().any(|row| row.contains_key("note")),
        "after the refusal the published `note` is required unchanged: {rows:?}"
    );
    assert!(
        synthesis.notes.iter().any(|note| matches!(
            note,
            Note::PartialObservation { scenario, unobserved }
                if scenario.to_string() == TERMINAL_REFUSAL && unobserved == &["secret".to_owned()]
        )),
        "{:?}",
        synthesis.notes
    );
}

// ---- #132's `help:` line, read by a refusal that is not a wrong-state one --------------------------

/// A subject-state guard refusal (`when_subject_state` with a default error branch) still requires
/// immediate views; with only an `eventual` view it is refused, and its `help:` must not tell the
/// author that an `eventual` view is enough.
const STATE_GUARD: &str = r"
format: ess/13
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - name: demo.orders.NotOpen
    summary: Not open.
    fields: []
events:
  - name: demo.orders.OrderOpened
    fields:
      - {name: order_id, type: demo.orders.OrderId}
  - name: demo.orders.Annotated
    fields: []
  - name: demo.orders.OrderClosed
    fields: []
commands:
  - name: demo.orders.OpenOrder
    input:
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        sets: {note: input.note}
        emits: [demo.orders.OrderOpened]
        payload:
          demo.orders.OrderOpened: {order_id: {generated: true}}
  - name: demo.orders.CloseOrder
    input:
      - {name: order_id, type: demo.orders.OrderId}
    outcomes:
      - name: closed
        moves: demo.orders.Order.close
        instance: order_id
        emits: [demo.orders.OrderClosed]
  - name: demo.orders.Annotate
    input:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: note, type: String}
    outcomes:
      - name: annotated
        when_subject_state: Open
        updates: demo.orders.Order
        instance: order_id
        sets: {note: input.note}
        emits: [demo.orders.Annotated]
      - name: not-open
        error: demo.orders.NotOpen
views:
  - name: demo.orders.OrderState
    source: demo.orders.Order
    consistency: eventual
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: state, type: demo.orders.Order.State}
";

#[test]
fn adversary_help_for_an_immediate_view_requirement_does_not_offer_eventual() {
    let synthesis = synthesize(&compiled(STATE_GUARD));
    let misleading: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| refusal.to_string().contains("immediate"))
        .filter(|refusal| refusal.hint().contains("`eventual` included"))
        .map(ToString::to_string)
        .collect();
    assert!(
        misleading.is_empty(),
        "a refusal that requires an immediate view tells the author an eventual one suffices:\n{}",
        misleading.join("\n")
    );
}

// ---- #154 on a three-state lifecycle, every dropped source --------------------------------------

const MEMBERSHIP3: &str = r"
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
      states: [Waiting, Active, Done]
      terminal: [Done]
      transitions:
        - {name: start, from: [Waiting], to: Active}
        - {name: finish, from: [Active], to: Done}
        - {name: observe, from: [Waiting, Active, Done], to: Done}
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
  - name: club.members.Moved
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
        emits: [club.members.Moved]
        payload:
          club.members.Moved: {member_id: input.member_id}
  - name: club.members.Finish
    input:
      - {name: member_id, type: club.members.MemberId}
    outcomes:
      - name: finished
        moves: club.members.Membership.finish
        instance: member_id
        emits: [club.members.Moved]
        payload:
          club.members.Moved: {member_id: input.member_id}
  - name: club.members.Observe
    input:
      - {name: kind, type: club.members.Kind}
      - {name: member_id, type: club.members.MemberId}
      - {name: session_id, type: club.members.SessionId}
    outcomes:
      - name: observed
        when: kind == Progress
        moves: club.members.Membership.observe
        instance: member_id
        emits: [club.members.Moved]
        payload:
          club.members.Moved: {member_id: input.member_id}
      - name: closed
        when: kind == Close
        moves: club.members.Session.close
        instance: session_id
        emits: [club.members.Closed]
        payload:
          club.members.Closed: {session_id: input.session_id}
      - {name: wrong-state, wrong_state: true, error: club.members.StateConflict}
";

/// Each state dropped from a three-state all-states `from:` — the initial, a middle and the
/// terminal one — gets its `…/state/<dropped>/refuses/Observe` scenario.
#[test]
fn adversary_every_state_dropped_from_an_all_states_transition_gets_its_refusal() {
    let (files, texts) = documents(MEMBERSHIP3);
    let mut missing = Vec::new();
    let mut seen = 0;
    for mutant in mutate::mutants(&files, &[mutate::MutantClass::FromDrop]) {
        let Some(state) = mutant
            .id
            .strip_prefix("from-drop/club.members.Membership.observe/")
        else {
            continue;
        };
        seen += 1;
        let mutated = mutate::apply(&files, &mutant.mutation).expect("the site exists");
        let ir = mutate::compile(mutated, &texts).unwrap_or_else(|stillborn| {
            panic!(
                "{} compiles: {} {}",
                mutant.id, stillborn.code, stillborn.cause
            )
        });
        let synthesis = synthesize(&ir);
        let id = format!("club.members.Membership/state/{state}/refuses/club.members.Observe");
        if !synthesis
            .suite
            .scenarios
            .keys()
            .any(|scenario| scenario.to_string() == id)
        {
            missing.push(format!(
                "{}: no `{id}`; refusals {:?}",
                mutant.id,
                refusals(&synthesis)
            ));
        }
    }
    assert_eq!(seen, 3, "the audit enumerates one drop per source state");
    assert!(missing.is_empty(), "{missing:#?}");
}

// ---- #161's arranged prior value, on the paths `arranged` leaves early ----------------------------

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
      - {name: minutes, type: Integer}
    lifecycle:
      initial: Live
      states: [Live, Ended]
      terminal: [Ended]
      transitions:
        - {name: end, from: [Live], to: Ended}
events:
  - name: rec.calls.CallStarted
    fields:
      - {name: call_id, type: rec.calls.CallId}
  - name: rec.calls.Changed
    fields: []
errors:
  - name: rec.calls.TooLong
    summary: Too long.
    fields: []
views:
  - name: rec.calls.Calls
    source: rec.calls.Call
    consistency: read_your_writes
    fields:
      - {name: call_id, type: rec.calls.CallId}
      - {name: state, type: rec.calls.Call.State}
      - {name: recording, type: Boolean}
      - {name: minutes, type: Integer}
commands:
  - name: rec.calls.StartCall
    input:
      - {name: recording, type: Boolean}
      - {name: minutes, type: Integer}
    outcomes:
      - name: started
        creates: rec.calls.Call
        instance: call_id
        sets: {recording: input.recording, minutes: input.minutes}
        emits: [rec.calls.CallStarted]
        payload:
          rec.calls.CallStarted: {call_id: {generated: true}}
BRANCH
";

/// `Record` behind a stored-field guard (the routed path).
const GUARDED_RECORD: &str = r"  - name: rec.calls.Record
    input:
      - {name: call_id, type: rec.calls.CallId}
    outcomes:
      - name: too-long
        when_subject:
          predicate:
            all:
              - minutes > 60
        error: rec.calls.TooLong
      - name: recorded
        updates: rec.calls.Call
        instance: call_id
        sets: {recording: true}
        emits: [rec.calls.Changed]
  - name: rec.calls.EndCall
    input:
      - {name: call_id, type: rec.calls.CallId}
    outcomes:
      - name: ended
        moves: rec.calls.Call.end
        instance: call_id
        emits: [rec.calls.Changed]
";

/// `End` moving the call and writing a literal (the `moves` effect).
const MOVING_RECORD: &str = r"  - name: rec.calls.Record
    input:
      - {name: call_id, type: rec.calls.CallId}
    outcomes:
      - name: recorded
        moves: rec.calls.Call.end
        instance: call_id
        sets: {recording: true}
        emits: [rec.calls.Changed]
";

#[test]
fn adversary_a_literal_write_is_arranged_over_another_value_on_every_path() {
    let mut unchanged = Vec::new();
    for (path, branch) in [
        ("stored-field guard", GUARDED_RECORD),
        ("moves", MOVING_RECORD),
    ] {
        let synthesis = synthesize(&compiled(&RECORDED.replace("BRANCH", branch)));
        let arranged: Vec<(String, Option<Node>)> = invocations(&synthesis)
            .into_iter()
            .filter(|invocation| {
                invocation.scenario == "rec.calls.Record/outcome/recorded"
                    && invocation.command == "rec.calls.Record"
                    && invocation.expected == "recorded"
            })
            .map(|invocation| {
                (
                    invocation.scenario.clone(),
                    invocation
                        .arranged
                        .get("rec.calls.StartCall")
                        .and_then(|create| create.get("recording").cloned()),
                )
            })
            .collect();
        assert!(
            !arranged.is_empty(),
            "{path}: recorded is invoked: {:?}",
            refusals(&synthesis)
        );
        for (scenario, recording) in arranged {
            if recording != Some(Node::Bool(false)) {
                unchanged.push(format!(
                    "{path}: {scenario} arranges `recording: {recording:?}` before writing `true`"
                ));
            }
        }
    }
    assert!(unchanged.is_empty(), "{unchanged:#?}");
}
