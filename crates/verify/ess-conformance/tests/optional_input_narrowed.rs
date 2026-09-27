//! The #169 repro synthesizes a suite that holds the narrowing to account (beyond10x/ess#169,
//! `ess/16`, `docs/design/optional-input-narrowing.md`).
//!
//! Narrowing is sound only if the refusal really takes every request without the account. So the
//! suite sends the absent input and requires the refusal, and the success scenario sends the
//! account and requires the branch that copies it. An implementation that answers an absent
//! account with a note fails the first; one that refuses a present account fails the second.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::{synthesize, ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const NOTES: &str = include_str!("fixtures/optional-input-narrowed.yaml");

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|e| panic!("parses: {e}\n{text}"));
    let specification = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}\n{text}"));
    compile(&specification, &SourceMap::new()).unwrap_or_else(|d| panic!("resolves:\n{d}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = synthesize(&compiled(NOTES));
    assert!(
        synthesis.refusals.is_empty(),
        "both branches are synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

/// The one scenario for `outcome`, and the input it sends to `SubmitNote`.
fn scenario(
    suite: &ConformanceSuite,
    outcome: &str,
) -> (
    Vec<ScenarioStep>,
    std::collections::BTreeMap<String, ScenarioValue>,
) {
    let suffix = format!("demo.notes.SubmitNote/outcome/{outcome}");
    let (_, found) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string().ends_with(&suffix))
        .unwrap_or_else(|| {
            panic!(
                "no scenario for {suffix}: {:?}",
                suite.scenarios.keys().collect::<Vec<_>>()
            )
        });
    let input = found
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
        .expect("the scenario sends the command");
    (found.steps.clone(), input)
}

fn absent(value: Option<&ScenarioValue>) -> bool {
    value.is_none_or(|value| matches!(value.as_literal(), Some(Node::Null)))
}

fn expects_outcome(steps: &[ScenarioStep], name: &str) -> bool {
    steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome }
            if outcome.to_string().ends_with(name))
    })
}

#[test]
fn issue_169_the_refusal_scenario_sends_no_account_and_requires_the_refusal() {
    let (steps, input) = scenario(&suite(), "account-missing");
    assert!(absent(input.get("account_id")), "sent {input:?}");
    assert!(expects_outcome(&steps, "account-missing"), "{steps:#?}");
    assert!(
        steps.iter().any(
            |step| matches!(step, ScenarioStep::ExpectError { error, .. }
            if error.to_string() == "demo.notes.AccountMissing")
        ),
        "{steps:#?}"
    );
}

#[test]
fn issue_169_the_success_scenario_sends_the_account_it_copies() {
    let (steps, input) = scenario(&suite(), "submitted");
    assert!(!absent(input.get("account_id")), "sent {input:?}");
    assert!(expects_outcome(&steps, "submitted"), "{steps:#?}");
}
