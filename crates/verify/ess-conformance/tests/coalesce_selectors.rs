//! A selector that says what an absent `Optional` field of the candidate row compares as
//! (`ess/30`, beyond10x/ess#627).
//!
//! `tests/fixtures/coalesce-selector.yaml`: `Join` adds a scored member to a slot, writing its
//! `enabled` from an `Optional` input, so absent where the input omits it; `Switch` writes it
//! later. `Pick` refuses with `Empty` while no member of the slot is enabled, and otherwise pairs
//! the one ranked first by `[element_score desc, member_id]` among them, where "enabled" is
//! `coalesce(enabled, true) == true`: an absent value counts as true, a false one does not.
//!
//! The interpreter ranks a member whose `enabled` is absent above a higher-scored one whose
//! `enabled` is false, and answers `Empty` where every member is false. Synthesis arranges a
//! witness of each, the interpreted target passes every scenario, and a target that reads an
//! absent value as false, or ignores the selector's `coalesce`, fails one.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/coalesce-selector.yaml");
const SELECTOR: &str = "coalesce(enabled, true) == true";

const ABSENT: &str = "00000000-0000-4000-8000-000000000001";
const FALSE: &str = "00000000-0000-4000-8000-000000000002";
const TRUE: &str = "00000000-0000-4000-8000-000000000003";
const PAIR: &str = "00000000-0000-4000-8000-0000000000aa";
const SECOND_PAIR: &str = "00000000-0000-4000-8000-0000000000bb";

const PICKED: &str = "demo.picking.Pick/outcome/picked";
const EMPTY: &str = "demo.picking.Pick/outcome/empty";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("picking.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn number(value: i64) -> Node {
    Node::Number(ess_primitives::facts::Number::from(value))
}

fn correlation() -> CorrelationId {
    CorrelationId::new("coalesce-selector").unwrap()
}

fn target(text: &str) -> Interpreted {
    let target = Interpreted::for_model(ir(text));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.picking/authored/coalesce-selector".parse().unwrap(),
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

fn send(target: &dyn ConformanceTarget, command: &str, input: &[(&str, Node)]) -> String {
    let answer = target
        .execute_command(request(&format!("demo.picking.{command}"), input))
        .unwrap_or_else(|error| panic!("{command} answers: {error}"));
    outcome(&answer)
}

/// A member of slot `a`; `enabled: None` omits the input, which leaves the field absent.
fn join(target: &dyn ConformanceTarget, id: &str, key: &str, score: i64, enabled: Option<bool>) {
    let mut input = vec![
        ("member_id", text(id)),
        ("slot_id", text("a")),
        ("element_key", text(key)),
        ("element_score", number(score)),
    ];
    if let Some(enabled) = enabled {
        input.push(("enabled", Node::Bool(enabled)));
    }
    assert_eq!(send(target, "Join", &input), "joined");
}

fn switch(target: &dyn ConformanceTarget, id: &str, enabled: bool) {
    let switched = send(
        target,
        "Switch",
        &[("member_id", text(id)), ("enabled", Node::Bool(enabled))],
    );
    assert_eq!(switched, "switched");
}

fn pick(target: &dyn ConformanceTarget, pair: &str) -> SemanticCommandResult {
    target
        .execute_command(request(
            "demo.picking.Pick",
            &[("pair_id", text(pair)), ("slot_id", text("a"))],
        ))
        .unwrap_or_else(|error| panic!("Pick answers: {error}"))
}

/// A member whose `enabled` is absent ranks above a higher-scored one whose `enabled` is false,
/// and above a lower-scored one whose `enabled` is true.
#[test]
fn the_head_ranks_an_absent_value_above_a_false_one() {
    let target = target(MODEL);
    join(&target, FALSE, "k-false", 30, Some(false));
    join(&target, ABSENT, "k-absent", 20, None);
    join(&target, TRUE, "k-true", 10, Some(true));
    let picked = pick(&target, PAIR);
    assert_eq!(outcome(&picked), "picked", "{picked:?}");
    let event = &picked.direct_events[0].payload;
    assert_eq!(event["element_key"], text("k-absent"), "{event:?}");
    assert_eq!(event["member_id"], text(ABSENT), "{event:?}");

    // Written false, the same member leaves the selection, and the enabled one is the head.
    switch(&target, ABSENT, false);
    let picked = pick(&target, SECOND_PAIR);
    assert_eq!(outcome(&picked), "picked", "{picked:?}");
    let event = &picked.direct_events[0].payload;
    assert_eq!(event["element_key"], text("k-true"), "{event:?}");
    assert_eq!(event["member_id"], text(TRUE), "{event:?}");
}

/// The row-set guard reads the same selector: every member false is `Empty`, one absent is not.
#[test]
fn the_row_set_guard_reads_an_absent_value_as_the_default() {
    let target = self::target(MODEL);
    join(&target, FALSE, "k-false", 30, Some(false));
    let refused = pick(&target, PAIR);
    assert_eq!(outcome(&refused), "empty", "{refused:?}");
    assert!(refused.direct_events.is_empty(), "{refused:?}");

    let target = self::target(MODEL);
    join(&target, ABSENT, "k-absent", 20, None);
    let picked = pick(&target, PAIR);
    assert_eq!(outcome(&picked), "picked", "{picked:?}");
    assert_eq!(
        picked.direct_events[0].payload["element_key"],
        text("k-absent")
    );
}

fn statuses<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| {
            let status = if matches!(run.status, Status::Passed) {
                "passed".to_owned()
            } else {
                format!("{:?} {:?}", run.status, run.checks)
            };
            (run.scenario.to_string(), status)
        })
        .collect()
}

fn failing(statuses: &BTreeMap<String, String>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id.as_str())
        .collect()
}

fn synthesized() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(MODEL));
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for wanted in [PICKED, EMPTY] {
        assert!(
            ids.iter().any(|id| id == wanted),
            "{wanted} is synthesized: {ids:#?}\nrefusals: {:#?}",
            synthesis.refusals
        );
    }
    synthesis.suite
}

#[test]
fn the_interpreted_target_passes_every_synthesized_scenario() {
    let suite = synthesized();
    let healthy = statuses(&suite, &Interpreted::for_model(ir(MODEL)));
    assert_eq!(failing(&healthy), Vec::<&str>::new(), "{healthy:#?}");
}

/// A target that reads an absent `enabled` as false: the selector without its default.
#[test]
fn a_target_reading_an_absent_value_as_false_fails_a_scenario() {
    let suite = synthesized();
    let faulty = MODEL.replace(SELECTOR, "enabled == true");
    let statuses = statuses(&suite, &Interpreted::for_model(ir(&faulty)));
    let failed = failing(&statuses);
    assert!(
        failed.contains(&PICKED),
        "absent members read as false select none, so `picked` fails: {statuses:#?}"
    );
}

/// A target that selects every member of the slot, false ones too: the selector without its
/// `coalesce` conjunct.
#[test]
fn a_target_ignoring_the_coalesced_conjunct_fails_a_scenario() {
    let suite = synthesized();
    let faulty = MODEL.replace("\n              - coalesce(enabled, true) == true", "");
    let faulty = faulty.replace("\n                  - coalesce(enabled, true) == true", "");
    let faulty = faulty.replace(
        "\n                    - coalesce(enabled, true) == true",
        "",
    );
    assert!(!faulty.contains(SELECTOR), "{faulty}");
    let statuses = statuses(&suite, &Interpreted::for_model(ir(&faulty)));
    let failed = failing(&statuses);
    assert!(
        failed.contains(&EMPTY),
        "a false member selected answers `picked` where `empty` is required: {statuses:#?}"
    );
}
