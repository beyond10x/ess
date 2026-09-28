//! Adversary cases for `story:optional-input-narrowed-after-refusal` (beyond10x/ess#169): every
//! narrowed branch's success scenario must send the input it copies, whichever rule narrowed it.
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::{synthesize, ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const NOTES: &str = include_str!("fixtures/optional-input-narrowed.yaml");
const REFUSAL: &str = "      - name: account-missing\n        when: not defined(account_id)\n        error: demo.notes.AccountMissing\n";

fn edited(base: &str, before: &str, after: &str) -> String {
    assert!(base.contains(before), "fixture holds {before:?}");
    base.replacen(before, after, 1)
}

fn suite(text: &str) -> ConformanceSuite {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|e| panic!("parses: {e}\n{text}"));
    let specification = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}\n{text}"));
    let ir: EssIr =
        compile(&specification, &SourceMap::new()).unwrap_or_else(|d| panic!("resolves:\n{d}"));
    let synthesis = synthesize(&ir);
    assert!(
        synthesis.refusals.is_empty(),
        "every branch is synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn sent(suite: &ConformanceSuite, outcome: &str) -> BTreeMap<String, ScenarioValue> {
    let suffix = format!("demo.notes.SubmitNote/outcome/{outcome}");
    let (_, found) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string().ends_with(&suffix))
        .unwrap_or_else(|| panic!("no scenario for {suffix}"));
    found
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.notes.SubmitNote" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .expect("the scenario sends the command")
}

fn present(input: &BTreeMap<String, ScenarioValue>, field: &str) -> bool {
    input
        .get(field)
        .is_some_and(|value| !matches!(value.as_literal(), Some(Node::Null)))
}

#[test]
fn the_defined_guard_branch_sends_the_account_and_the_default_refusal_omits_it() {
    let text = edited(
        NOTES,
        REFUSAL,
        "      - name: account-missing\n        error: demo.notes.AccountMissing\n",
    );
    let text = edited(
        &text,
        "      - name: submitted\n",
        "      - name: submitted\n        when: defined(account_id)\n",
    );
    let suite = suite(&text);
    assert!(present(&sent(&suite, "submitted"), "account_id"));
    assert!(!present(&sent(&suite, "account-missing"), "account_id"));
}

#[test]
fn a_default_narrowed_by_two_refusals_sends_both_inputs() {
    let text = edited(
        NOTES,
        "      - {name: text, type: String}\n    outcomes:",
        "      - {name: text, type: String}\n      - {name: weight, type: Optional<Decimal>}\n    outcomes:",
    );
    let text = edited(
        &text,
        "      - name: submitted\n",
        "      - name: weight-missing\n        when: missing(weight)\n        error: demo.notes.AccountMissing\n      - name: submitted\n",
    );
    let text = edited(
        &text,
        "text: input.text}\nevents:",
        "text: input.text, weight: input.weight}\nevents:",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\nviews:",
        "      - {name: text, type: String}\n      - {name: weight, type: Decimal}\nviews:",
    );
    let suite = suite(&text);
    let input = sent(&suite, "submitted");
    assert!(present(&input, "account_id"), "{input:?}");
    assert!(present(&input, "weight"), "{input:?}");
    assert!(!present(&sent(&suite, "weight-missing"), "weight"));
}
