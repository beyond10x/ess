//! Adversary pass 1 on `story:witness-search-beyond-64-candidates`: guards the directed ladders
//! do not reduce to a product within the bound, and the website claim that "the last candidates
//! cover every combination of true and false for those comparisons".

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::mutate::{self, Document, MutantClass};
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = r"
format: ess/13
system: demo
version: v1
domain: demo.gate
types:
  - {name: demo.gate.Kind, kind: enum, variants: [Open, Shut, Held]}
  - name: demo.gate.Window
    kind: struct
    fields:
      - {name: hi, type: Integer}
      - {name: lo, type: Integer}
events:
  - name: demo.gate.Passed
    fields:
      - {name: a, type: Integer}
errors:
  - name: demo.gate.Refused
    summary: Refused.
    fields: []
commands:
  - name: demo.gate.Check
    input:
      - {name: a, type: Integer}
      - {name: b, type: Integer}
      - {name: c, type: Integer}
      - {name: d, type: Integer}
      - {name: e, type: Integer}
      - {name: f, type: Integer}
      - {name: g, type: Integer}
      - {name: k, type: demo.gate.Kind}
      - {name: m, type: Decimal}
      - {name: x, type: Optional<Integer>}
      - {name: aw, type: demo.gate.Window}
    outcomes:
      - name: refused
        when:
GUARD
        error: demo.gate.Refused
      - name: passed
        emits: [demo.gate.Passed]
        payload:
          demo.gate.Passed: {a: input.a}
";

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

/// The branch each `ExecuteCommand` of `demo.gate.Check` is expected to take.
fn expected_branches(synthesis: &Synthesis) -> Vec<(String, BTreeMap<String, Node>)> {
    let mut out = Vec::new();
    for scenario in synthesis.suite.scenarios.values() {
        let mut pending: Option<BTreeMap<String, Node>> = None;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == "demo.gate.Check" =>
                {
                    pending = Some(
                        input
                            .iter()
                            .filter_map(|(field, value)| {
                                value
                                    .as_literal()
                                    .cloned()
                                    .map(|node| (field.clone(), node))
                            })
                            .collect(),
                    );
                }
                ScenarioStep::ExpectOutcome { outcome } => {
                    if let Some(input) = pending.take() {
                        let branch = outcome
                            .to_string()
                            .rsplit('/')
                            .next()
                            .expect("an outcome ref names its branch")
                            .to_owned();
                        out.push((branch, input));
                    }
                }
                _ => {}
            }
        }
    }
    out
}

fn assert_refused_is_witnessed(guard: &str) {
    let text = MODEL.replace("GUARD", guard);
    let synthesis = synthesize(&compiled(&text));
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert!(
        expected_branches(&synthesis)
            .iter()
            .any(|(branch, _)| branch == "refused"),
        "the guarded branch of `{}` is witnessed: {refusals:#?}",
        guard.trim()
    );
}

/// Seven integer fields each compared with its own literal: the directed ladders describe
/// 2^7 = 128 truth assignments, and the one satisfying assignment is the last of them.
#[test]
fn adversary_search_seven_field_conjunction_is_witnessed() {
    assert_refused_is_witnessed(
        "          all: [a > 10, b > 10, c > 10, d > 10, e > 10, f > 10, g > 10]",
    );
}

/// Six own-literal fields where one of them is `Optional`: its omission ladder doubles the directed
/// product and reserves a slot, so the all-true assignment falls past the bound.
#[test]
fn adversary_search_six_field_conjunction_with_an_optional_is_witnessed() {
    assert_refused_is_witnessed("          all: [a > 10, b > 10, c > 10, d > 10, e > 10, x > 10]");
}

/// Two struct members compared with each other plus three own-literal fields: `aw.hi` and `aw.lo`
/// are kept whole, so the directed product is 3 * 3 * 2 * 2 * 2 = 72, past the bound. The same
/// guard over a struct input named `w` (sorted after `c`, `d`, `e`) is witnessed; only the name
/// moves it past position 64.
#[test]
fn adversary_search_a_field_comparison_beside_three_literals_is_witnessed() {
    assert_refused_is_witnessed("          all: [aw.hi > aw.lo, c > 10, d > 10, e > 10]");
}

/// Mixed types: Integer, Decimal, enum and Optional, all compared with their own literals.
#[test]
fn adversary_search_mixed_type_conjunction_is_witnessed() {
    assert_refused_is_witnessed(
        "          all: [a > 10, b > 10, c > 10, k == Held, m < -5.5, x > 10]",
    );
}

/// Four fields in two disjunctions: witnessed and every connective mutant killed.
#[test]
fn adversary_search_four_field_nested_guard_connective_mutants_are_killed() {
    let text = MODEL.replace(
        "GUARD",
        "          all:\n            - any: [a > 10, b > 10]\n            - any: [c > 10, d > 10]\n            - e > 10",
    );
    let (files, texts) = documents(&text);
    let original = compiled(&text);
    let witnessed = expected_branches(&synthesize(&original));
    assert!(
        witnessed.iter().any(|(branch, _)| branch == "refused"),
        "the guarded branch is witnessed"
    );
    let mut survivors = Vec::new();
    for mutant in mutate::mutants(&files, &[MutantClass::GuardConnective]) {
        let mutated = mutate::apply(&files, &mutant.mutation).expect("the site exists");
        let Ok(ir) = mutate::compile(mutated, &texts) else {
            continue;
        };
        let original_command = original
            .commands()
            .values()
            .find(|command| command.name.to_string() == "demo.gate.Check")
            .expect("declared");
        let killed = expected_branches(&synthesize(&ir))
            .into_iter()
            .any(|(branch, input)| {
                let facts =
                    ess_conformance::flatten(&original, original_command, &input).expect("fits");
                let guard = original_command
                    .outcomes
                    .iter()
                    .find_map(ess_conformance::when)
                    .expect("guarded");
                let satisfied = facts.decide(guard) == ess_conformance::Decision::Satisfied;
                satisfied != (branch == "refused")
            });
        if !killed {
            survivors.push(mutant.id.clone());
        }
    }
    assert!(survivors.is_empty(), "surviving mutants: {survivors:#?}");
}
