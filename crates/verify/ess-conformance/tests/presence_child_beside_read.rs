//! beyond10x/ess#729: a presence child beside a comparison on the same `Optional` is witnessed by
//! the absent input, and a goal that needs the path absent and a read of it decided is never a
//! refusal. A comparison that reads an absent `Optional` is unknown, so it does not block the row.
use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    synthesize::{synthesize, RefusalCause},
    ConformanceSuite as Suite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const REVIEW_HEAD: &str = "format: ess/23
system: demo
version: v1
domain: demo.review
types:
  - {name: demo.review.Kind, kind: enum, variants: [Note, Correction]}
  - {name: demo.review.CheckStatus, kind: enum, variants: [Reported, Verified]}
errors:
  - name: demo.review.CorrectionNotVerified
  - name: demo.review.Incomplete
commands:
  - name: demo.review.RecordDecision
    input:
      - {name: kind, type: demo.review.Kind}
      - {name: check_status, type: 'Optional<demo.review.CheckStatus>'}
    outcomes:
";

const INCOMPLETE: &str = "      - name: incomplete
        when:
          all: [kind == Correction, not defined(check_status)]
        error: demo.review.Incomplete
";

const NOT_VERIFIED: &str = "      - name: correction-not-verified
        when:
          all: [kind == Correction, defined(check_status), check_status != Verified]
        error: demo.review.CorrectionNotVerified
      - name: recorded
        accepts: nothing
";

fn range(guard: &str) -> String {
    format!(
        "format: ess/23
system: demo
version: v1
domain: demo.range
errors:
  - name: demo.range.Refused
commands:
  - name: demo.range.Check
    input:
      - {{name: minimum, type: 'Optional<Integer>'}}
      - {{name: maximum, type: 'Optional<Integer>'}}
    outcomes:
      - name: refused
        when:
          {guard}
        error: demo.range.Refused
      - name: accepted
        accepts: nothing
"
    )
}

fn suite(body: &str) -> (Suite, usize) {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|error| panic!("{error}\n{body}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{body}"));
    let synthesis = synthesize(&ir);
    let unwitnessed = synthesis
        .refusals
        .iter()
        .filter(|refusal| matches!(refusal.cause, RefusalCause::ChildUnwitnessed { .. }))
        .count();
    (synthesis.suite, unwitnessed)
}

/// The inputs a scenario sends, one map per command invocation.
fn sent(suite: &Suite, scenario: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    let (_, scenario) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .unwrap_or_else(|| panic!("no scenario {scenario}"));
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(input.clone()),
            _ => None,
        })
        .collect()
}

fn absent(input: &BTreeMap<String, ScenarioValue>, field: &str) -> bool {
    matches!(
        input.get(field),
        None | Some(ScenarioValue::Literal { value: Node::Null })
    )
}

fn present(input: &BTreeMap<String, ScenarioValue>, field: &str) -> bool {
    !absent(input, field)
}

fn kind_is(input: &BTreeMap<String, ScenarioValue>, kind: &str) -> bool {
    input.get("kind")
        == Some(&ScenarioValue::Literal {
            value: Node::Text(kind.to_owned()),
        })
}

const RECORDED: &str = "demo.review.RecordDecision/outcome/recorded";

#[test]
fn a_presence_child_beside_a_comparison_on_the_same_optional_is_witnessed_absent() {
    let (suite, unwitnessed) = suite(&format!("{REVIEW_HEAD}{NOT_VERIFIED}"));
    assert_eq!(unwitnessed, 0, "no ESS-SYNTH-022");
    let runs = sent(&suite, RECORDED);
    assert!(
        runs.iter()
            .any(|input| kind_is(input, "Correction") && absent(input, "check_status")),
        "{runs:#?}"
    );
}

#[test]
fn an_earlier_branch_that_takes_the_absent_row_sends_it_once() {
    let (suite, unwitnessed) = suite(&format!("{REVIEW_HEAD}{INCOMPLETE}{NOT_VERIFIED}"));
    assert_eq!(unwitnessed, 0, "no ESS-SYNTH-022");
    let sends = |scenario: &str| {
        sent(&suite, scenario)
            .iter()
            .filter(|input| kind_is(input, "Correction") && absent(input, "check_status"))
            .count()
    };
    assert_eq!(sends("demo.review.RecordDecision/outcome/incomplete"), 1);
    assert_eq!(sends(RECORDED), 0);
    assert_eq!(
        sends("demo.review.RecordDecision/outcome/correction-not-verified"),
        0
    );
}

#[test]
fn a_range_with_optional_ends_is_witnessed_by_each_end_absent() {
    let (suite, unwitnessed) = suite(&range(
        "all: [defined(minimum), defined(maximum), minimum > maximum]",
    ));
    assert_eq!(unwitnessed, 0, "no ESS-SYNTH-022");
    let runs = sent(&suite, "demo.range.Check/outcome/accepted");
    assert!(
        runs.iter()
            .any(|input| absent(input, "minimum") && present(input, "maximum")),
        "{runs:#?}"
    );
    assert!(
        runs.iter()
            .any(|input| present(input, "minimum") && absent(input, "maximum")),
        "{runs:#?}"
    );
}

#[test]
fn a_disjunct_testing_absence_is_witnessed_by_the_absent_input() {
    let (suite, unwitnessed) = suite(&range("any: [not defined(minimum), minimum > 10]"));
    assert_eq!(unwitnessed, 0, "no ESS-SYNTH-022");
    let runs = sent(&suite, "demo.range.Check/outcome/refused");
    assert!(
        runs.iter().any(|input| absent(input, "minimum")),
        "{runs:#?}"
    );
}
