//! The head of an ordered selection on the native interpreter (`docs/design/ordered-related-head.md`,
//! "Interpreter and history checker", "Decisive acceptance"; beyond10x/ess#543).
//!
//! `tests/fixtures/ordered-related-head.yaml`: `Join` adds a scored membership to a slot; `Pick`
//! refuses with `Empty` while no membership of the slot is `Present`, and otherwise pairs the one
//! ranked first by `[element_score desc, membership_id]`, publishing and storing its key.
//!
//! The rows are arranged so that the head is neither the first nor the last inserted, nor the head
//! under the first key's reversed direction, and a tie on the score is broken by the identity
//! against the insertion order.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store, Undetermined};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/ordered-related-head.yaml");

const S1: &str = "00000000-0000-4000-8000-000000000001";
const S2: &str = "00000000-0000-4000-8000-000000000002";
const S3: &str = "00000000-0000-4000-8000-000000000003";
const OTHER_SLOT: &str = "00000000-0000-4000-8000-000000000004";
const REMOVED: &str = "00000000-0000-4000-8000-000000000005";
const TIE_LATER_ID: &str = "00000000-0000-4000-8000-000000000009";
const TIE_EARLIER_ID: &str = "00000000-0000-4000-8000-000000000008";
const PAIR: &str = "00000000-0000-4000-8000-0000000000aa";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("ranked.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

/// The fixture with `Pick`'s `exists: false` refusal taken away: the head read alone.
fn unguarded() -> String {
    edit(
        MODEL,
        "      - name: empty
        when_related:
          entity: demo.ranked.Membership
          where: {all: [slot_id == input.slot_id, state == Present]}
          exists: false
        error: demo.ranked.Empty
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
    CorrelationId::new("ordered-head").unwrap()
}

fn target(text: &str) -> Interpreted {
    let target = Interpreted::for_model(ir(text));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.ranked/authored/ordered-head".parse().unwrap(),
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

fn join(target: &dyn ConformanceTarget, id: &str, slot: &str, key: &str, score: i64) {
    let answer = target
        .execute_command(request(
            "demo.ranked.Join",
            &[
                ("membership_id", text(id)),
                ("slot_id", text(slot)),
                ("element_key", text(key)),
                ("element_score", number(score)),
            ],
        ))
        .unwrap_or_else(|error| panic!("a membership joins: {error}"));
    assert_eq!(outcome(&answer), "joined");
}

fn pick(target: &dyn ConformanceTarget, slot: &str) -> SemanticCommandResult {
    target
        .execute_command(request(
            "demo.ranked.Pick",
            &[("pair_id", text(PAIR)), ("slot_id", text(slot))],
        ))
        .unwrap_or_else(|error| panic!("answered: {error}"))
}

fn pairs(target: &dyn ConformanceTarget) -> Vec<BTreeMap<String, Node>> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.ranked.Pairs".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: ess_primitives::consistency::QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .expect("the view answers")
        .rows
}

/// S1, S2 and S3 in slot `a`, inserted in that order with scores 20, 30 and 10, so S2 is the head;
/// a higher score in another slot, and a higher score in this slot that has left.
fn arranged(target: &dyn ConformanceTarget) {
    join(target, S1, "a", "k-first", 20);
    join(target, OTHER_SLOT, "b", "k-other-slot", 99);
    join(target, REMOVED, "a", "k-removed", 50);
    join(target, S2, "a", "k-head", 30);
    join(target, S3, "a", "k-last", 10);
    let left = target
        .execute_command(request(
            "demo.ranked.Leave",
            &[("membership_id", text(REMOVED))],
        ))
        .unwrap_or_else(|error| panic!("left: {error}"));
    assert_eq!(outcome(&left), "left");
}

#[test]
fn the_interpreter_publishes_the_ranked_head() {
    let target = target(MODEL);
    arranged(&target);
    let picked = pick(&target, "a");
    assert_eq!(outcome(&picked), "picked");
    let event = &picked.direct_events[0].payload;
    assert_eq!(event["element_key"], text("k-head"), "{event:?}");
    assert_eq!(event["membership_id"], text(S2), "{event:?}");
    let rows = pairs(&target);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["element_key"], text("k-head"), "{rows:?}");
}

/// A tie on the score is broken by the identity, ascending, whatever the insertion order.
#[test]
fn a_tie_is_broken_by_the_identity_not_the_insertion_order() {
    let target = target(MODEL);
    join(&target, TIE_LATER_ID, "t", "k-later-id", 30);
    join(&target, S3, "t", "k-lower-score", 10);
    join(&target, TIE_EARLIER_ID, "t", "k-earlier-id", 30);
    let picked = pick(&target, "t");
    assert_eq!(outcome(&picked), "picked");
    let event = &picked.direct_events[0].payload;
    assert_eq!(event["element_key"], text("k-earlier-id"), "{event:?}");
    assert_eq!(event["membership_id"], text(TIE_EARLIER_ID), "{event:?}");
}

#[test]
fn an_empty_selection_publishes_nothing() {
    // The model's own answer for an empty slot is its refusal.
    let target = target(MODEL);
    join(&target, OTHER_SLOT, "b", "k-other-slot", 99);
    let refused = pick(&target, "a");
    assert_eq!(outcome(&refused), "empty");
    assert!(refused.direct_events.is_empty(), "{refused:?}");
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");

    // Without it, the head read alone supplies no value: no outcome is published and no row stored.
    let target = self::target(&unguarded());
    join(&target, OTHER_SLOT, "b", "k-other-slot", 99);
    let answered = target.execute_command(request(
        "demo.ranked.Pick",
        &[("pair_id", text(PAIR)), ("slot_id", text("a"))],
    ));
    assert!(
        answered
            .as_ref()
            .map_or(true, |result| result.outcome.is_none()),
        "an empty selection supplies no value: {answered:?}"
    );
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");
}

fn run(
    ir: &EssIr,
    store: &mut Store,
    command: &str,
    input: &[(&str, Node)],
) -> Result<Step, Undetermined> {
    let input: BTreeMap<String, Node> = input
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect();
    let mut steps = execute(
        ir,
        store,
        &format!("demo.ranked.{command}").parse().unwrap(),
        &input,
        &Externals::Withheld,
    )?;
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    *store = step.next.clone();
    Ok(step)
}

/// A score the store does not hold leaves the head undecided between two members: the history is
/// not guessed, and the store is unchanged.
#[test]
fn an_abstract_key_leaves_the_head_undecidable() {
    let model = ir(&edit(
        MODEL,
        "          element_score: input.element_score\n",
        "",
    ));
    let mut store = Store::default();
    for (id, key) in [(S1, "k-first"), (S2, "k-second")] {
        run(
            &model,
            &mut store,
            "Join",
            &[
                ("membership_id", text(id)),
                ("slot_id", text("a")),
                ("element_key", text(key)),
                ("element_score", number(1)),
            ],
        )
        .unwrap_or_else(|error| panic!("joined: {error:?}"));
    }
    let before = store.clone();
    let answered = run(
        &model,
        &mut store,
        "Pick",
        &[("pair_id", text(PAIR)), ("slot_id", text("a"))],
    );
    assert!(
        matches!(answered, Err(Undetermined::Undecidable { .. })),
        "{answered:?}"
    );
    assert_eq!(store, before);
}

/// Until synthesis arranges a head that differs from the insertion order (unit U4), the branch
/// reading one is refused by name rather than witnessed on rows that cannot tell a right head from
/// a wrong one; the command's other branches keep their scenarios.
#[test]
fn synthesis_refuses_a_head_reading_branch_by_name_until_it_is_arranged() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(MODEL));
    let refusals = format!("{:?}", synthesis.refusals);
    assert!(refusals.contains("demo.ranked.Pick/picked"), "{refusals}");
    assert!(
        refusals.contains("the ranked head is not arranged"),
        "{refusals}"
    );
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.iter().any(|id| id == "demo.ranked.Pick/outcome/empty"),
        "{ids:?}"
    );
    assert!(
        !ids.iter().any(|id| id == "demo.ranked.Pick/outcome/picked"),
        "{ids:?}"
    );
}
