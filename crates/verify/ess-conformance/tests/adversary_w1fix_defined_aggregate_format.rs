//! Adversary, w1fix pass 1: suite format selection for `defined()`/`missing()` over an `Optional`
//! aggregate (beyond10x/ess#176) beyond the struct and list the unit's own suite drives.
//!
//! The unit's documents (`formats.md`, `spec-versions.md`, the `defined_aggregates` module doc)
//! promise suite/26 for an `Optional` struct, list, map or `Json` read by a view `satisfies`
//! predicate, and the module's `used_by` doc adds a union. Each case below is a model whose
//! invariant reaches the suite as such a predicate; each must select suite/26 (coverage /27), or a
//! 0.37.0 runner reads the aggregate as absent and passes the invariant unchecked.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::ViewExpectation, synthesize::synthesize, ConformanceSuite, ScenarioStep,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replace(from, to)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

/// Whether some view `satisfies` step carries a predicate whose text contains `needle`.
fn carries(suite: &ConformanceSuite, needle: &str) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Satisfies { predicate },
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Satisfies { predicate },
                ..
            } => predicate.to_string().contains(needle),
            _ => false,
        })
    })
}

fn coverage_version(ir: &EssIr) -> String {
    let built = ess_conformance::coverage_build::build(
        ir,
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    built
        .selected()
        .original_json()
        .split("\"suite_version\":")
        .nth(1)
        .and_then(|rest| rest.split('"').nth(1))
        .unwrap_or("<none>")
        .to_owned()
}

/// Synthesizes `text`, requires the invariant reading `needle` to reach the suite, and requires
/// suite/26 and coverage /27.
fn selects_round_three(text: &str, needle: &str) {
    let ir = ir(text);
    let synthesis = synthesize(&ir);
    let suite = synthesis.suite;
    assert!(
        carries(&suite, needle),
        "the invariant reading `{needle}` reaches the suite; refusals: {:#?}",
        synthesis.refusals
    );
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/26",
        "a view predicate reading `{needle}` over an Optional aggregate"
    );
    assert_eq!(coverage_version(&ir), "ess-conformance/27");
}

/// Replaces the type of `metrics` in the entity, the view and the `PauseQueue` input.
fn metrics_as(optional: &str, required: &str) -> String {
    let text = edit(
        QUEUE,
        "{name: metrics, type: Optional<demo.queue.Metrics>}",
        &format!("{{name: metrics, type: '{optional}'}}"),
    );
    edit(
        &text,
        "{name: metrics, type: demo.queue.Metrics}",
        &format!("{{name: metrics, type: '{required}'}}"),
    )
}

#[test]
fn adversary_w1fix_defined_over_an_optional_json_selects_suite_26() {
    selects_round_three(&metrics_as("Optional<Json>", "Json"), "defined(metrics)");
}

#[test]
fn adversary_w1fix_defined_over_an_optional_map_selects_suite_26() {
    selects_round_three(
        &metrics_as("Optional<Map<String, Integer>>", "Map<String, Integer>"),
        "defined(metrics)",
    );
}

#[test]
fn adversary_w1fix_defined_over_an_optional_union_selects_suite_26() {
    let text = edit(
        QUEUE,
        "  - name: demo.queue.Metrics\n",
        "  - name: demo.queue.Reading\n    kind: union\n    tag: kind\n    variants:\n      count: Integer\n      label: String\n  - name: demo.queue.Metrics\n",
    );
    let text = edit(
        &text,
        "{name: metrics, type: Optional<demo.queue.Metrics>}",
        "{name: metrics, type: Optional<demo.queue.Reading>}",
    );
    let text = edit(
        &text,
        "{name: metrics, type: demo.queue.Metrics}",
        "{name: metrics, type: demo.queue.Reading}",
    );
    selects_round_three(&text, "defined(metrics)");
}

/// `metrics.inner` is an `Optional` struct inside the `Optional` struct `metrics`: a 0.37.0 runner
/// binds `metrics.waiting` and nothing at `metrics.inner`, so it reads the nested presence as
/// absent exactly as it reads the top-level one.
#[test]
fn adversary_w1fix_defined_over_a_nested_optional_struct_selects_suite_26() {
    let text = edit(
        QUEUE,
        "      - {name: waiting, type: Integer}\n",
        "      - {name: waiting, type: Integer}\n      - {name: inner, type: Optional<demo.queue.Inner>}\n  - name: demo.queue.Inner\n    kind: struct\n    fields:\n      - {name: depth, type: Integer}\n",
    );
    let text = edit(
        &text,
        "{not: \"defined(metrics)\"}",
        "{not: \"defined(metrics.inner)\"}",
    );
    selects_round_three(&text, "defined(metrics.inner)");
}

/// A value object's invariant, rebased onto the view field that holds one, is the second family
/// that reaches a `satisfies` step (`synthesize.rs` `assert_satisfied`). `Lead.extra` is an
/// `Optional` struct, so the rebased `defined(lead.extra)` is the same construct.
#[test]
fn adversary_w1fix_a_rebased_value_object_invariant_over_an_optional_struct_selects_suite_26() {
    const CARDS: &str = "format: ess/16
system: demo
version: v1
domain: demo.cards
types:
  - {name: demo.cards.CardId, kind: newtype, of: Uuid}
  - name: demo.cards.Extra
    kind: struct
    fields:
      - {name: note, type: String}
  - name: demo.cards.Lead
    kind: struct
    fields:
      - {name: rank, type: Integer}
      - {name: extra, type: Optional<demo.cards.Extra>}
    invariants:
      - any: [rank >= 0, {not: \"defined(extra)\"}]
entities:
  - name: demo.cards.Card
    identity: {name: card_id, type: demo.cards.CardId}
    fields:
      - {name: lead, type: demo.cards.Lead}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - {name: demo.cards.Opened, fields: [{name: card_id, type: demo.cards.CardId}]}
actors:
  - {name: demo.cards.Clerk, may: [demo.cards.Open]}
commands:
  - name: demo.cards.Open
    input:
      - {name: lead, type: demo.cards.Lead}
    outcomes:
      - name: opened
        creates: demo.cards.Card
        instance: card_id
        emits: [demo.cards.Opened]
        payload: {demo.cards.Opened: {card_id: {generated: true}}}
        sets: {lead: input.lead}
views:
  - name: demo.cards.CardRow
    source: demo.cards.Card
    consistency: read_your_writes
    fields:
      - {name: card_id, type: demo.cards.CardId}
      - {name: lead, type: demo.cards.Lead}
";
    selects_round_three(CARDS, "defined(lead.extra)");
}
