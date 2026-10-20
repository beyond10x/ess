//! A connective's child comparing two input fields with each other is solved by choosing one field
//! relative to the other (beyond10x/ess#622): `all: [limit > 0, amount > 0, amount <= limit]`,
//! whose row where `amount <= limit` alone fails is `limit 5, amount 6`, is witnessed rather than
//! refused with `ESS-SYNTH-022`, and the connective's mutant is killed.
#![allow(clippy::needless_raw_string_hashes)]

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutationReport, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;

/// Two top-level `Integer` inputs.
const FLAT: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.terms

components:
  - component: terms-server
    owns:
      domains: [demo.terms]
    accepts:
      commands: [demo.terms.Check]
    reached_by: network

errors:
  - name: demo.terms.Invalid

commands:
  - name: demo.terms.Check
    input:
      - {name: limit, type: Integer}
      - {name: amount, type: Integer}
    outcomes:
      - name: valid
        when:
          all:
            - limit > 0
            - amount > 0
            - amount <= limit
        accepts: nothing
      - name: invalid
        error: demo.terms.Invalid
"#;

/// The same guard over the members of one struct input.
const NESTED: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.terms

components:
  - component: terms-server
    owns:
      domains: [demo.terms]
    accepts:
      commands: [demo.terms.Check]
    reached_by: network

types:
  - name: demo.terms.Terms
    kind: struct
    fields:
      - {name: limit, type: Integer}
      - {name: amount, type: Integer}

errors:
  - name: demo.terms.Invalid

commands:
  - name: demo.terms.Check
    input:
      - {name: terms, type: demo.terms.Terms}
    outcomes:
      - name: valid
        when:
          all:
            - terms.limit > 0
            - terms.amount > 0
            - terms.amount <= terms.limit
        accepts: nothing
      - name: invalid
        error: demo.terms.Invalid
"#;

fn parsed(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn ir(text: &str) -> EssIr {
    let (files, texts) = parsed(text);
    mutate::compile(files, &texts).expect("the fixture compiles")
}

fn audit(text: &str) -> MutationReport {
    let (files, texts) = parsed(text);
    let ir = ir(text);
    mutate::audit(&files, &texts, MutantClass::ALL, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn child_refusals(text: &str) -> Vec<String> {
    ess_conformance::synthesize::synthesize(&ir(text))
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-022")
        .map(ToString::to_string)
        .collect()
}

/// Every `(limit, amount)` the suite sends `demo.terms.Check`, read at `limit` and `amount` or at
/// `terms.limit` and `terms.amount`.
fn sent(text: &str) -> Vec<(i64, i64)> {
    let suite = ess_conformance::synthesize::synthesize(&ir(text)).suite;
    let number = |node: Option<&Node>| match node {
        Some(Node::Number(number)) => number.as_i64(),
        _ => None,
    };
    let mut found = Vec::new();
    for scenario in suite.scenarios.values() {
        for step in &scenario.steps {
            let ess_conformance::ScenarioStep::ExecuteCommand { command, input, .. } = step else {
                continue;
            };
            if command.to_string() != "demo.terms.Check" {
                continue;
            }
            let literal = |name: &str| input.get(name).and_then(|value| value.as_literal());
            let pair = match literal("terms") {
                Some(Node::Map(terms)) => (number(terms.get("limit")), number(terms.get("amount"))),
                _ => (number(literal("limit")), number(literal("amount"))),
            };
            if let (Some(limit), Some(amount)) = pair {
                found.push((limit, amount));
            }
        }
    }
    found
}

fn assert_solved(text: &str) {
    let refused = child_refusals(text);
    assert!(refused.is_empty(), "no child is refused: {refused:#?}");
    let pairs = sent(text);
    assert!(
        pairs
            .iter()
            .any(|&(limit, amount)| limit > 0 && amount > 0 && amount > limit),
        "an input where `amount <= limit` alone fails is sent: {pairs:?}"
    );
    let report = audit(text);
    let verdicts: BTreeMap<String, Verdict> = report
        .mutants
        .iter()
        .map(|entry| (entry.id.clone(), entry.verdict))
        .collect();
    let connectives: Vec<_> = report
        .mutants
        .iter()
        .filter(|entry| entry.class == MutantClass::GuardConnective)
        .collect();
    assert!(!connectives.is_empty(), "{verdicts:#?}");
    for entry in connectives {
        assert_eq!(
            entry.verdict,
            Verdict::Killed,
            "{} ({}): {verdicts:#?}",
            entry.id,
            entry.change
        );
    }
}

#[test]
fn a_child_comparing_two_inputs_is_solved() {
    assert_solved(FLAT);
}

#[test]
fn a_child_comparing_two_members_of_one_struct_is_solved() {
    assert_solved(NESTED);
}

/// [`FLAT`] with both inputs declared through a newtype holding `-5..=3`: the row is solved inside
/// the range.
fn ranged() -> String {
    let typed = FLAT.replace(
        "errors:\n",
        "types:\n  - {name: demo.terms.Units, kind: newtype, of: Integer, invariants: \
         [value >= -5, value <= 3]}\n\nerrors:\n",
    );
    let typed = typed
        .replace(
            "{name: limit, type: Integer}",
            "{name: limit, type: demo.terms.Units}",
        )
        .replace(
            "{name: amount, type: Integer}",
            "{name: amount, type: demo.terms.Units}",
        );
    assert!(
        typed.contains("type: demo.terms.Units}") && typed.contains("value <= 3"),
        "the fixture edit landed: {typed}"
    );
    typed
}

#[test]
fn a_declared_range_is_respected_where_the_child_is_solved() {
    let text = ranged();
    assert_solved(&text);
    let pairs = sent(&text);
    assert!(
        pairs
            .iter()
            .all(|&(limit, amount)| (-5..=3).contains(&limit) && (-5..=3).contains(&amount)),
        "every input is inside the declared range: {pairs:?}"
    );
}
