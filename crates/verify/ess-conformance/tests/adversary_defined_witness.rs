//! Adversary, story:defined-over-optional-aggregates (beyond10x/ess#176), pass 1: witnesses.
//!
//! The unit's own suite witnesses `defined(metrics)` over an `Optional` *struct* input. The
//! acceptance names a list and a map `T` as well, and `missing(x)` is the same construct negated.
//! Each guard below must be synthesized without refusal and witnessed on both sides: the guarded
//! branch with the aggregate sent, the other with it left out (or the reverse for `missing`).
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::synthesize, ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{
    node::Node,
    predicate::{Predicate, Truth},
};

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The fixture plus `extra_types`, and a `Probe` command whose `probed` branch is guarded by `when`.
fn probe(extra_types: &str, input: &str, when: &str) -> String {
    QUEUE
        .replace(
            "      - {name: waiting, type: Integer}\n",
            &format!("      - {{name: waiting, type: Integer}}\n{extra_types}"),
        )
        .replace(
            "commands:\n",
            &format!(
                "commands:\n  - name: demo.queue.Probe\n    input:\n{input}    outcomes:\n      - name: probed\n        when: '{when}'\n        emits: [demo.queue.QueueOpened]\n        payload: {{demo.queue.QueueOpened: {{queue_id: {{generated: true}}}}}}\n      - name: declined\n        emits: [demo.queue.QueueResumed]\n        payload: {{demo.queue.QueueResumed: {{queue_id: {{generated: true}}}}}}\n"
            ),
        )
        .replace(
            "may: [demo.queue.OpenQueue,",
            "may: [demo.queue.Probe, demo.queue.OpenQueue,",
        )
}

fn suite_of(text: &str) -> ConformanceSuite {
    let synthesis = synthesize(&ir(text));
    let probe_refusals: Vec<_> = synthesis
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains("demo.queue.Probe"))
        .collect();
    assert!(
        probe_refusals.is_empty(),
        "the Probe branches are synthesized: {probe_refusals:#?}"
    );
    synthesis.suite
}

/// The value `outcome`'s scenario sends for `field`, `None` when it is left out.
fn sent(suite: &ConformanceSuite, outcome: &str, field: &str) -> Option<ScenarioValue> {
    let id = format!("demo.queue.Probe/outcome/{outcome}");
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1;
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(
                input
                    .get(field)
                    .filter(|value| *value != &ScenarioValue::literal(Node::Null))
                    .cloned(),
            ),
            _ => None,
        })
        .expect("the scenario executes the command")
}

fn witnessed(extra_types: &str, input: &str, when: &str, field: &str, guarded_sends: bool) {
    let suite = suite_of(&probe(extra_types, input, when));
    let probed = sent(&suite, "probed", field);
    let declined = sent(&suite, "declined", field);
    assert_eq!(
        probed.is_some(),
        guarded_sends,
        "`{when}`: `probed` sent {probed:?}"
    );
    assert_eq!(
        declined.is_some(),
        !guarded_sends,
        "`{when}`: `declined` sent {declined:?}"
    );
}

#[test]
fn adversary_defined_a_guard_over_an_optional_list_input_is_witnessed_on_both_sides() {
    witnessed(
        "",
        "      - {name: tags, type: Optional<List<String>>}\n",
        "defined(tags)",
        "tags",
        true,
    );
}

#[test]
fn adversary_defined_a_guard_over_an_optional_map_input_is_witnessed_on_both_sides() {
    witnessed(
        "",
        "      - {name: counts, type: 'Optional<Map<String, Integer>>'}\n",
        "defined(counts)",
        "counts",
        true,
    );
}

#[test]
fn adversary_defined_missing_over_an_optional_struct_input_is_witnessed_on_both_sides() {
    witnessed(
        "",
        "      - {name: metrics, type: Optional<demo.queue.Metrics>}\n",
        "missing(metrics)",
        "metrics",
        false,
    );
}

/// `holder: Optional<Holder>`, `Holder.inner: Optional<Metrics>`: `defined(holder.inner)` needs
/// `holder` sent *with* `inner` for `probed`, and `inner` left out (or `holder` itself) for
/// `declined`.
#[test]
fn adversary_defined_a_guard_over_a_nested_optional_struct_is_witnessed_on_both_sides() {
    let holder = "  - name: demo.queue.Holder\n    kind: struct\n    fields:\n      - {name: inner, type: Optional<demo.queue.Metrics>}\n";
    let suite = suite_of(&probe(
        holder,
        "      - {name: holder, type: Optional<demo.queue.Holder>}\n",
        "defined(holder.inner)",
    ));
    let inner_of = |value: Option<ScenarioValue>| -> bool {
        let Some(value) = value else { return false };
        let holder = value
            .as_literal()
            .unwrap_or_else(|| panic!("a literal holder: {value:?}"));
        holder
            .as_map()
            .and_then(|entries| entries.get("inner"))
            .is_some_and(|inner| !matches!(inner, Node::Null))
    };
    assert!(
        inner_of(sent(&suite, "probed", "holder")),
        "`probed` must send holder.inner: {:?}",
        sent(&suite, "probed", "holder")
    );
    assert!(
        !inner_of(sent(&suite, "declined", "holder")),
        "`declined` must leave holder.inner out: {:?}",
        sent(&suite, "declined", "holder")
    );
}

/// The command-input lane over a nested `Optional`: `holder.inner` is present only when `holder`
/// holds a non-null `inner`, even an empty one.
#[test]
fn adversary_defined_the_input_lane_reads_a_nested_optional_struct() {
    let holder = "  - name: demo.queue.Holder\n    kind: struct\n    fields:\n      - {name: inner, type: Optional<demo.queue.Metrics>}\n";
    let text = probe(
        holder,
        "      - {name: holder, type: Optional<demo.queue.Holder>}\n      - {name: blob, type: Optional<Json>}\n",
        "defined(holder.inner)",
    );
    let ir = ir(&text);
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.queue.Probe")
        .expect("declared");
    let map = |entries: &[(&str, Node)]| {
        Node::Map(
            entries
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect(),
        )
    };
    let metrics = map(&[(
        "waiting",
        Node::Number(ess_primitives::facts::Number::new(1.0).unwrap()),
    )]);
    let cases: Vec<(&str, &str, Node, Truth)> = vec![
        (
            "holder.inner",
            "holder",
            map(&[("inner", metrics.clone())]),
            Truth::True,
        ),
        ("holder.inner", "holder", map(&[]), Truth::False),
        (
            "holder.inner",
            "holder",
            map(&[("inner", Node::Null)]),
            Truth::False,
        ),
        ("holder", "holder", map(&[]), Truth::True),
        ("blob", "blob", map(&[]), Truth::True),
        ("blob", "blob", Node::Seq(Vec::new()), Truth::True),
        ("blob", "blob", Node::Text("x".into()), Truth::True),
        ("blob", "blob", Node::Bool(false), Truth::True),
        ("blob", "blob", Node::Null, Truth::False),
    ];
    for (read, field, value, expected) in cases {
        let candidate = BTreeMap::from([(field.to_owned(), value.clone())]);
        let facts = ess_conformance::flatten(&ir, command, &candidate)
            .unwrap_or_else(|errors| panic!("{errors:?}"));
        let got = Predicate::parse_expression(&format!("defined({read})"))
            .unwrap()
            .evaluate(&facts);
        assert_eq!(got, expected, "defined({read}) over {field} = {value:?}");
    }
}
