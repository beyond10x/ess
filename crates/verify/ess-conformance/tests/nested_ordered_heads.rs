//! Nested heads of ordered selections: a selector reads a value its own outcome publishes
//! (`docs/design/published-value-operand.md`, "Interpreter and history checker", "Synthesis",
//! "Decisive acceptance"; beyond10x/ess#549).
//!
//! `tests/fixtures/nested-ordered-heads.yaml`: `OpenSlot` opens a scored slot in a group; `Join`
//! adds a scored membership to one side of a slot; `MatchInGroup` refuses with `Empty` while no slot
//! of the group is open, and otherwise publishes the best open slot of the group by
//! `[score desc, slot_id]`, then the best membership on each side **of that slot** by
//! `[element_score desc, membership_id]`. The inner selectors read the outer head through
//! `payload.demo.nested.PairMade.slot_id`, and the pair stores the slot and the left key through
//! `{payload: …}`.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/nested-ordered-heads.yaml");

const SLOT_A: &str = "00000000-0000-4000-8000-00000000000a";
const SLOT_B: &str = "00000000-0000-4000-8000-00000000000b";
const SLOT_C: &str = "00000000-0000-4000-8000-00000000000c";
const SLOT_OTHER_GROUP: &str = "00000000-0000-4000-8000-00000000000d";
const SLOT_CLOSED: &str = "00000000-0000-4000-8000-00000000000e";
const PAIR: &str = "00000000-0000-4000-8000-0000000000aa";

fn membership(n: u8) -> String {
    format!("00000000-0000-4000-8000-0000000001{n:02x}")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("nested.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

/// The fixture with `MatchInGroup`'s `exists: false` refusal taken away: the heads read alone.
fn unguarded() -> String {
    edit(
        MODEL,
        "      - name: empty
        when_related:
          entity: demo.nested.Slot
          where: {all: [group_id == input.group_id, state == Open]}
          exists: false
        error: demo.nested.Empty
",
        "",
    )
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn number(value: i64) -> Node {
    Node::Number(ess_primitives::facts::Number::from(value))
}

fn correlation() -> CorrelationId {
    CorrelationId::new("nested-heads").unwrap()
}

fn target(text: &str) -> Interpreted {
    let target = Interpreted::for_model(ir(text));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.nested/authored/nested-heads".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
    target
}

fn request(command: &str, input: &[(&str, Node)]) -> SemanticCommandRequest {
    SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input: input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect(),
        correlation: correlation(),
    }
}

fn outcome(result: &SemanticCommandResult) -> String {
    result
        .outcome
        .as_ref()
        .map_or_else(|| "<none>".to_owned(), |taken| taken.outcome.to_string())
}

fn send(target: &dyn ConformanceTarget, command: &str, input: &[(&str, Node)], expected: &str) {
    let answer = target
        .execute_command(request(&format!("demo.nested.{command}"), input))
        .unwrap_or_else(|error| panic!("{command} answers: {error}"));
    assert_eq!(outcome(&answer), expected, "{command}: {answer:?}");
}

fn open(target: &dyn ConformanceTarget, slot: &str, group: &str, score: i64) {
    send(
        target,
        "OpenSlot",
        &[
            ("slot_id", text(slot)),
            ("group_id", text(group)),
            ("score", number(score)),
        ],
        "opened",
    );
}

fn join(target: &dyn ConformanceTarget, n: u8, slot: &str, side: &str, key: &str, score: i64) {
    send(
        target,
        "Join",
        &[
            ("membership_id", text(&membership(n))),
            ("slot_id", text(slot)),
            ("side", text(side)),
            ("element_key", text(key)),
            ("element_score", number(score)),
        ],
        "joined",
    );
}

fn match_in(target: &dyn ConformanceTarget, group: &str) -> Result<SemanticCommandResult, String> {
    target
        .execute_command(request(
            "demo.nested.MatchInGroup",
            &[("pair_id", text(PAIR)), ("group_id", text(group))],
        ))
        .map_err(|error| error.to_string())
}

fn pairs(target: &dyn ConformanceTarget) -> Vec<BTreeMap<String, Node>> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.nested.Pairs".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: ess_primitives::consistency::QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .expect("the view answers")
        .rows
}

/// Slots A (20), B (30) and C (10) open in group `g`, inserted in that order, so B is the outer
/// head; a better slot in another group, and a better slot in `g` that has closed. Every slot but B
/// holds a membership on each side that would outrank B's, so an inner read scoped by any other
/// slot, or by none, answers a different key.
fn arranged(target: &dyn ConformanceTarget) {
    open(target, SLOT_A, "g", 20);
    open(target, SLOT_OTHER_GROUP, "h", 99);
    open(target, SLOT_CLOSED, "g", 50);
    open(target, SLOT_B, "g", 30);
    open(target, SLOT_C, "g", 10);
    send(
        target,
        "CloseSlot",
        &[("slot_id", text(SLOT_CLOSED))],
        "closed",
    );
    join(target, 1, SLOT_A, "Left", "l-in-a", 90);
    join(target, 2, SLOT_B, "Left", "l-low", 10);
    join(target, 3, SLOT_B, "Left", "l-head", 40);
    join(target, 4, SLOT_B, "Right", "r-head", 35);
    join(target, 5, SLOT_C, "Left", "l-in-c", 95);
    join(target, 6, SLOT_C, "Right", "r-in-c", 95);
    join(target, 7, SLOT_B, "Right", "r-low", 5);
    join(target, 8, SLOT_CLOSED, "Left", "l-in-closed", 99);
    join(target, 9, SLOT_OTHER_GROUP, "Right", "r-in-other", 99);
}

#[test]
fn the_interpreter_takes_each_side_of_the_slot_it_chose() {
    let target = target(MODEL);
    arranged(&target);
    let paired = match_in(&target, "g").expect("answered");
    assert_eq!(outcome(&paired), "paired", "{paired:?}");
    let event = &paired.direct_events[0].payload;
    assert_eq!(event["slot_id"], text(SLOT_B), "{event:?}");
    assert_eq!(event["left_key"], text("l-head"), "{event:?}");
    assert_eq!(event["right_key"], text("r-head"), "{event:?}");
}

/// The right side of the U2 model also requires an element key other than the left one: its
/// selector reads two published values, one of which is itself read through the slot. Whatever
/// order the payload fields are declared or held in, each is resolved after what it reads.
#[test]
fn a_selector_reading_a_value_that_reads_another_resolves_after_both() {
    let chained = edit(
        MODEL,
        "where: {all: [slot_id == payload.demo.nested.PairMade.slot_id, side == Right, state == Present]}",
        "where: {all: [element_key != payload.demo.nested.PairMade.left_key, slot_id == payload.demo.nested.PairMade.slot_id, side == Right, state == Present]}",
    );
    let target = target(&chained);
    open(&target, SLOT_A, "g", 20);
    open(&target, SLOT_B, "g", 30);
    join(&target, 1, SLOT_B, "Left", "shared", 40);
    join(&target, 2, SLOT_B, "Right", "shared", 50);
    join(&target, 3, SLOT_B, "Right", "r-second", 20);
    join(&target, 4, SLOT_A, "Right", "r-in-a", 30);
    let paired = match_in(&target, "g").expect("answered");
    assert_eq!(outcome(&paired), "paired", "{paired:?}");
    let event = &paired.direct_events[0].payload;
    assert_eq!(event["slot_id"], text(SLOT_B), "{event:?}");
    assert_eq!(event["left_key"], text("shared"), "{event:?}");
    assert_eq!(event["right_key"], text("r-second"), "{event:?}");
}

#[test]
fn an_empty_outer_selection_publishes_nothing() {
    // The model's own answer for a group with no open slot is its refusal.
    let target = target(MODEL);
    open(&target, SLOT_OTHER_GROUP, "h", 99);
    join(&target, 1, SLOT_OTHER_GROUP, "Left", "l-in-other", 1);
    join(&target, 2, SLOT_OTHER_GROUP, "Right", "r-in-other", 1);
    let refused = match_in(&target, "g").expect("answered");
    assert_eq!(outcome(&refused), "empty");
    assert!(refused.direct_events.is_empty(), "{refused:?}");
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");

    // Without it, no outer head supplies no slot, and an inner read of it supplies nothing either.
    let target = self::target(&unguarded());
    open(&target, SLOT_OTHER_GROUP, "h", 99);
    join(&target, 1, SLOT_OTHER_GROUP, "Left", "l-in-other", 1);
    join(&target, 2, SLOT_OTHER_GROUP, "Right", "r-in-other", 1);
    let answered = match_in(&target, "g");
    assert!(
        answered
            .as_ref()
            .map_or(true, |result| result.outcome.is_none()),
        "no outer head publishes nothing: {answered:?}"
    );
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");

    // An outer head whose left side is empty publishes nothing, whatever waits in another slot.
    let target = self::target(MODEL);
    open(&target, SLOT_A, "g", 10);
    open(&target, SLOT_B, "g", 30);
    join(&target, 1, SLOT_A, "Left", "l-in-a", 90);
    join(&target, 2, SLOT_B, "Right", "r-head", 35);
    let answered = match_in(&target, "g");
    assert!(
        answered
            .as_ref()
            .map_or(true, |result| result.outcome.is_none()),
        "an empty inner selection publishes nothing: {answered:?}"
    );
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");
}

#[test]
fn the_event_and_the_stored_copy_hold_the_operand_value() {
    let target = target(MODEL);
    arranged(&target);
    let paired = match_in(&target, "g").expect("answered");
    assert_eq!(outcome(&paired), "paired", "{paired:?}");
    let event = &paired.direct_events[0].payload;
    let rows = pairs(&target);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["slot_id"], event["slot_id"], "{rows:?} {event:?}");
    assert_eq!(rows[0]["left_key"], event["left_key"], "{rows:?} {event:?}");
    assert_eq!(rows[0]["slot_id"], text(SLOT_B), "{rows:?}");
    assert_eq!(rows[0]["left_key"], text("l-head"), "{rows:?}");
}
