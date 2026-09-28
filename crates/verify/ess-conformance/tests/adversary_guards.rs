//! Adversary cases for story:subject-guard-input-and-case-folding (beyond10x/ess#157, #140) in
//! synthesis: E6 witnessed both ways over every operand type the design says compares, and E7's
//! case-changed witness wherever synthesis decides a fold, a view filter included.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::scenario::ViewExpectation;
use ess_conformance::{synthesize, ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|e| panic!("parses: {e}\n{text}"));
    let specification = Specification::assemble([(Source::new("adv.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}\n{text}"));
    compile(&specification, &SourceMap::new()).unwrap_or_else(|d| panic!("resolves:\n{d}"))
}

/// The #157 shape with the compared field `v` of `stored` type on the row and `sent` type on the stop.
fn recording(stored: &str, sent: &str) -> String {
    format!(
        r"format: ess/15
system: calls
version: v1
domain: calls.rec
types:
  - name: calls.rec.Mode
    kind: enum
    variants: [Audio, Video]
entities:
  - name: calls.rec.Call
    identity: {{name: call_id, type: Uuid}}
    fields:
      - {{name: v, type: {stored}}}
    lifecycle:
      initial: Recording
      states: [Recording, Stopped]
      terminal: [Stopped]
      transitions:
        - {{name: stop, from: [Recording], to: Stopped}}
events:
  - name: calls.rec.Started
    fields:
      - {{name: call_id, type: Uuid}}
  - name: calls.rec.Stopped
    fields: []
commands:
  - name: calls.rec.Start
    input:
      - {{name: v, type: {stored}}}
    outcomes:
      - name: started
        creates: calls.rec.Call
        instance: call_id
        sets: {{v: input.v}}
        emits: [calls.rec.Started]
        payload:
          calls.rec.Started:
            call_id: {{generated: true}}
  - name: calls.rec.Stop
    input:
      - {{name: call_id, type: Uuid}}
      - {{name: v, type: {sent}}}
    outcomes:
      - name: other
        when_subject:
          predicate: v != input.v
        preserves: calls.rec.Call
        instance: call_id
      - name: stopped
        moves: calls.rec.Call.stop
        instance: call_id
        emits: [calls.rec.Stopped]
views:
  - name: calls.rec.Calls
    source: calls.rec.Call
    consistency: read_your_writes
    fields:
      - {{name: call_id, type: Uuid}}
      - {{name: state, type: calls.rec.Call.State}}
      - {{name: v, type: {stored}}}
"
    )
}

fn sent(scenario: &ConformanceScenario, command: &str) -> Vec<Option<ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: c, input, ..
            } if c.to_string() == command => Some(input.get("v").cloned()),
            _ => None,
        })
        .collect()
}

#[test]
fn adv_e6_is_witnessed_both_ways_for_every_comparable_operand_type() {
    let mut failures = Vec::new();
    for (stored, stop) in [
        ("Integer", "Integer"),
        ("calls.rec.Mode", "calls.rec.Mode"),
        ("Uuid", "Uuid"),
        ("Timestamp", "Timestamp"),
        ("Boolean", "Boolean"),
        ("String", "Optional<String>"),
    ] {
        let synthesis = synthesize(&compiled(&recording(stored, stop)));
        let others: Vec<String> = synthesis
            .refusals
            .iter()
            .filter(|r| r.cause.code().to_string() != "ESS-SYNTH-012")
            .map(ToString::to_string)
            .collect();
        if !others.is_empty() {
            failures.push(format!("{stored}/{stop}: refused {others:?}"));
        }
        for (id, equal) in [
            ("calls.rec.Stop/outcome/other", false),
            ("calls.rec.Stop/outcome/stopped", true),
        ] {
            let Some((_, scenario)) = synthesis
                .suite
                .scenarios
                .iter()
                .find(|(key, _)| key.to_string() == id)
            else {
                failures.push(format!("{stored}/{stop}: no scenario {id}"));
                continue;
            };
            let held = sent(scenario, "calls.rec.Start").first().cloned().flatten();
            let named = sent(scenario, "calls.rec.Stop").last().cloned().flatten();
            if named.is_none() || (held == named) != equal {
                failures.push(format!(
                    "{stored}/{stop}: {id} holds {held:?} and names {named:?}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

const FILTERED: &str = r"format: ess/15
system: demo
version: v1
domain: demo.orders
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: Uuid}
    fields:
      - {name: source, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
events:
  - name: demo.orders.Placed
    fields: [{name: order_id, type: Uuid}]
commands:
  - name: demo.orders.Place
    input:
      - {name: source, type: String}
    outcomes:
      - name: placed
        creates: demo.orders.Order
        instance: order_id
        sets: {source: input.source}
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            order_id: {generated: true}
views:
  - name: demo.orders.Web
    source: demo.orders.Order
    consistency: read_your_writes
    filter: {source: {equals_ignore_case: web}}
    fields:
      - {name: order_id, type: Uuid}
      - {name: source, type: String}
";

/// A view filter is the other place synthesis decides a fold. Where the suite asserts the filtered
/// view at all, some row it asserts must carry the literal in another ASCII case — otherwise a
/// target filtering byte for byte passes every scenario. Where it asserts nothing, it says so.
#[test]
fn adv_a_folded_view_filter_is_witnessed_on_a_case_changed_row_or_refused() {
    let synthesis = synthesize(&compiled(FILTERED));
    let mut asserted = false;
    let mut texts = Vec::new();
    for scenario in synthesis.suite.scenarios.values() {
        if !format!("{:?}", scenario.steps).contains("demo.orders.Web") {
            continue;
        }
        asserted = true;
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { input, .. } = step {
                if let Some(Node::Text(text)) =
                    input.get("source").and_then(ScenarioValue::as_literal)
                {
                    texts.push(text.clone());
                }
            }
        }
    }
    let refused = synthesis
        .refusals
        .iter()
        .any(|r| r.to_string().contains("demo.orders.Web"));
    assert!(
        asserted || refused,
        "the view is neither asserted nor refused: {:?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    if asserted {
        assert!(
            texts
                .iter()
                .any(|text| text.eq_ignore_ascii_case("web") && text != "web"),
            "the filtered view is asserted only over rows {texts:?}: a byte-wise filter passes"
        );
    }
}

/// The `source` each row asserted `Contains` in `view` was created with, per scenario: the text the
/// `demo.orders.Place` that produced the instance the assertion names was sent.
fn contained_sources(
    synthesis: &ess_conformance::synthesize::Synthesis,
    view: &str,
) -> Vec<String> {
    let mut found = Vec::new();
    for scenario in synthesis.suite.scenarios.values() {
        for step in &scenario.steps {
            let expectation = match step {
                ScenarioStep::ExpectView {
                    view: name,
                    expectation,
                }
                | ScenarioStep::EventuallyView {
                    view: name,
                    expectation,
                    ..
                } if name.to_string() == view => expectation,
                _ => continue,
            };
            let ViewExpectation::Contains { fields } = expectation else {
                continue;
            };
            let Some(ScenarioValue::Instance { instance }) = fields.get("order_id") else {
                continue;
            };
            let captured = scenario.steps.iter().position(|step| {
                matches!(step, ScenarioStep::CaptureInstance { instance: named, .. } if named == instance)
            });
            let sent = captured.and_then(|at| {
                scenario.steps[..at]
                    .iter()
                    .rev()
                    .find_map(|step| match step {
                        ScenarioStep::ExecuteCommand { command, input, .. }
                            if command.to_string() == "demo.orders.Place" =>
                        {
                            match input.get("source").and_then(ScenarioValue::as_literal) {
                                Some(Node::Text(text)) => Some(text.clone()),
                                _ => None,
                            }
                        }
                        _ => None,
                    })
            });
            if let Some(text) = sent {
                found.push(text);
            }
        }
    }
    found
}

/// The control for the fold case above (story:view-filters-witnessed-on-matching-rows): a view
/// filtered by plain equality is asserted `Contains` over a row the filter matches — the row a
/// `demo.orders.Place` sent `web` — and not only `Excludes` over the row the plain witness makes.
/// A target that ignores the filter, or refuses every row, passes a suite without one.
#[test]
fn an_equality_filtered_view_is_asserted_over_a_row_its_filter_matches() {
    let text = FILTERED.replace(
        "filter: {source: {equals_ignore_case: web}}",
        "filter: 'source == \"web\"'",
    );
    assert_ne!(text, FILTERED);
    let synthesis = synthesize(&compiled(&text));
    let sources = contained_sources(&synthesis, "demo.orders.Web");
    assert!(
        sources.iter().any(|text| text == "web"),
        "no row the filter matches is asserted in the view: {sources:?}, refusals {:?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}

/// The fold case, asserted the way the control is: the row asserted `Contains` was sent the
/// literal in another ASCII case, so a target filtering byte for byte drops it and fails.
#[test]
fn a_folded_view_filter_is_asserted_over_a_case_changed_matching_row() {
    let synthesis = synthesize(&compiled(FILTERED));
    let sources = contained_sources(&synthesis, "demo.orders.Web");
    assert!(
        sources
            .iter()
            .any(|text| text.eq_ignore_ascii_case("web") && text != "web"),
        "no case-changed matching row is asserted in the view: {sources:?}"
    );
}

/// A fold under `when_subject` is decided on the arranged row, so the row must be created with the
/// literal in another ASCII case for the guarded branch: otherwise a target comparing bytes passes.
#[test]
fn adv_a_stored_field_fold_is_witnessed_on_a_case_changed_row() {
    let text = recording("String", "String").replace(
        "predicate: v != input.v",
        "predicate: {v: {equals_ignore_case: web}}",
    );
    let synthesis = synthesize(&compiled(&text));
    let row = |id: &str| -> Option<String> {
        let (_, scenario) = synthesis
            .suite
            .scenarios
            .iter()
            .find(|(key, _)| key.to_string() == id)?;
        match sent(scenario, "calls.rec.Start").first().cloned().flatten() {
            Some(value) => match value.as_literal() {
                Some(Node::Text(text)) => Some(text.clone()),
                _ => None,
            },
            None => None,
        }
    };
    let guarded = row("calls.rec.Stop/outcome/other");
    let default = row("calls.rec.Stop/outcome/stopped");
    assert!(
        guarded
            .as_deref()
            .is_some_and(|text| text.eq_ignore_ascii_case("web") && text != "web"),
        "guarded row {guarded:?}; refusals {:?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(
        default
            .as_deref()
            .is_some_and(|text| !text.eq_ignore_ascii_case("web")),
        "default row {default:?}"
    );
}

/// ASCII-only folding is the rule a target must follow, and the one a natural implementation breaks
/// (`strings.EqualFold`, `toLowerCase`, `str::to_lowercase`). For a literal with a non-ASCII letter,
/// synthesis should try it with that letter in its other Unicode case — which the design's rule
/// refutes and a Unicode-folding target accepts. Only then does the suite tell the two apart.
#[test]
fn adv_a_non_ascii_fold_literal_is_tried_in_its_unicode_case_to_refute_unicode_folding() {
    use ess_conformance::witness::{candidates, Distinction};
    use ess_domain::name::QualifiedName;
    use ess_primitives::predicate::Predicate;
    let text = FILTERED.replace(
        "  - name: demo.orders.Place\n    input:\n      - {name: source, type: String}\n    outcomes:\n",
        "  - name: demo.orders.Place\n    input:\n      - {name: source, type: String}\n    outcomes:\n      - name: rejected\n        when: {source: {equals_ignore_case: \"café\"}}\n        error: demo.orders.Rejected\n",
    )
    .replace(
        "events:\n",
        "errors:\n  - name: demo.orders.Rejected\n    fields: []\nevents:\n",
    );
    let ir = compiled(&text);
    let command = ir
        .commands()
        .get(&QualifiedName::new("demo.orders.Place").unwrap())
        .unwrap();
    let guard = Predicate::from_node(
        &serde_yaml::from_str("{source: {equals_ignore_case: \"café\"}}").unwrap(),
    )
    .unwrap();
    let tried: Vec<String> = candidates(&ir, command, &[&guard], Distinction::PLAIN)
        .unwrap()
        .iter()
        .filter_map(|input| match input.get("source") {
            Some(Node::Text(text)) => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert!(
        tried.iter().any(|text| text == "CAFÉ" || text == "cafÉ"),
        "no candidate carries the non-ASCII letter in its other case, so a Unicode-folding target \
         passes: tried {tried:?}"
    );
}
