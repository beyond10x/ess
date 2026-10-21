//! An `external:` branch narrowed to the records a subject guard selects (`ess/29`,
//! `story:external-branch-narrowed-by-subject-guard`, option 2 of beyond10x/ess#584).
//!
//! `tests/fixtures/external-narrowed-by-subject.yaml`: `Redeem` refuses a grant whose stored
//! `method` is `hashed` when an outside service finds the verifier wrong (`hashed-mismatch`,
//! declared first), a redirect that differs from the stored one (`redirect-mismatch`) and a plain
//! verifier that differs from the stored challenge (`plain-mismatch`); otherwise it redeems. The
//! narrowed branch answers at the held-state step in declaration order, and only on a hashed grant.
//!
//! Synthesis witnesses it on a hashed grant with the provider forced, and forces it beside every
//! sibling witnessed on a grant the guard does not select, where that sibling still answers. The
//! interpreted target passes every scenario; a target that ignores the narrowing, or answers the
//! narrowed branch after the held-state branches declared below it, fails.
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{ResolvedCommand, ResolvedCondition};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::command::OutcomeName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/external-narrowed-by-subject.yaml");
const REDEEM: &str = "demo.vault.Redeem";
const NARROWED: &str = "hashed-mismatch";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("vault.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run(suite: &ConformanceSuite, model: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn failing(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

/// Every `Redeem` the suite sends: the branch forced right before it, where one is, and the
/// outcome the step after it requires.
fn sent(suite: &ConformanceSuite) -> Vec<(String, Option<String>, String)> {
    let mut found = Vec::new();
    for (id, scenario) in &suite.scenarios {
        let mut forced: Option<String> = None;
        let mut open: Option<usize> = None;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ConfigureExternalOutcome { force, .. } => {
                    forced = Some(force.outcome.to_string());
                }
                ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == REDEEM => {
                    found.push((id.to_string(), forced.take(), String::new()));
                    open = Some(found.len() - 1);
                }
                ScenarioStep::ExecuteCommand { .. } => forced = None,
                ScenarioStep::ExpectOutcome { outcome } => {
                    if let Some(index) = open.take() {
                        found[index].2 = outcome.outcome.to_string();
                    }
                }
                _ => {}
            }
        }
    }
    found
}

/// The model with `rewrite` applied to `Redeem`.
fn rewritten(model: &EssIr, rewrite: impl Fn(&mut ResolvedCommand)) -> EssIr {
    model.with_commands_rewritten(|command| {
        let mut command = command.clone();
        if command.name.to_string() == REDEEM {
            rewrite(&mut command);
        }
        command
    })
}

#[test]
fn the_narrowed_branch_is_witnessed_on_a_row_it_selects_and_forced_beside_rows_it_does_not() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let sent = sent(&synthesis.suite);
    // Its own scenario: forced, and answered.
    assert!(
        sent.iter().any(
            |(id, forced, answered)| id.ends_with(&format!("outcome/{NARROWED}"))
                && forced.as_deref() == Some(NARROWED)
                && answered == NARROWED
        ),
        "{sent:#?}"
    );
    // Forced on a row its guard does not select, the branch the row selects answers.
    let beside: BTreeSet<&str> = sent
        .iter()
        .filter(|(_, forced, answered)| forced.as_deref() == Some(NARROWED) && answered != NARROWED)
        .map(|(_, _, answered)| answered.as_str())
        .collect();
    assert!(
        beside.contains("redeemed") && beside.contains("plain-mismatch"),
        "{sent:#?}"
    );
    let statuses = run(&synthesis.suite, model);
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
}

#[test]
fn a_target_ignoring_the_narrowing_fails() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    let unnarrowed = rewritten(&model, |command| {
        for outcome in &mut command.outcomes {
            if let ResolvedCondition::ExternalSubject { cause, .. } = &outcome.condition {
                outcome.condition = ResolvedCondition::External {
                    cause: cause.clone(),
                };
            }
        }
    });
    let statuses = run(&synthesis.suite, unnarrowed);
    assert!(!failing(&statuses).is_empty(), "{statuses:#?}");
}

#[test]
fn a_target_answering_the_narrowed_branch_after_a_later_held_state_branch_fails() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    // The narrowed branch's own witness is a row a held-state branch declared after it selects too.
    let own = sent(&synthesis.suite)
        .into_iter()
        .filter(|(id, forced, _)| {
            id.ends_with(&format!("outcome/{NARROWED}")) && forced.as_deref() == Some(NARROWED)
        })
        .count();
    assert!(own > 0);
    // Moved after `redirect-mismatch`, a target reads the redirect first.
    let reordered = rewritten(&model, |command| {
        let narrowed = command.outcomes.remove(0);
        command.outcomes.insert(1, narrowed);
    });
    let statuses = run(&synthesis.suite, reordered);
    assert!(
        failing(&statuses).contains(&"demo.vault.Redeem/outcome/hashed-mismatch"),
        "{statuses:#?}"
    );
}

#[test]
fn the_interpreter_answers_the_narrowed_branch_only_on_a_row_it_selects() {
    let model = ir(MODEL);
    let issue = |store: &Store, method: &str| {
        let input = BTreeMap::from([
            ("method".to_owned(), Node::Text(method.to_owned())),
            ("challenge".to_owned(), Node::Text("c".to_owned())),
            ("redirect".to_owned(), Node::Text("r".to_owned())),
        ]);
        let steps = execute(
            &model,
            store,
            &"demo.vault.Issue".parse().unwrap(),
            &input,
            &Externals::Withheld,
        )
        .unwrap_or_else(|why| panic!("{why}"));
        let [step] = steps.as_slice() else {
            panic!("Issue answers once: {steps:#?}");
        };
        let (_, id, _) = step.next.instances().next().expect("Issue creates a grant");
        (step.next.clone(), id.clone())
    };
    let redeem = |store: &Store, id: &Node, verifier: &str, redirect: &str, externals| {
        let input = BTreeMap::from([
            ("grant_id".to_owned(), id.clone()),
            ("verifier".to_owned(), Node::Text(verifier.to_owned())),
            ("redirect".to_owned(), Node::Text(redirect.to_owned())),
        ]);
        let steps = execute(&model, store, &REDEEM.parse().unwrap(), &input, &externals)
            .unwrap_or_else(|why| panic!("{why}"));
        assert_eq!(steps.len(), 1, "{steps:#?}");
        let answered = steps[0]
            .outcome
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        answered
            .rsplit_once('/')
            .map_or(answered.clone(), |(_, name)| name.to_owned())
    };
    let forced = || Externals::Forced(OutcomeName::new(NARROWED).unwrap());
    let (hashed, hashed_id) = issue(&Store::default(), "hashed");
    let (plain, plain_id) = issue(&Store::default(), "plain");
    assert_eq!(redeem(&hashed, &hashed_id, "v", "r", forced()), NARROWED);
    // Declared first among the held-state branches: before a wrong redirect.
    assert_eq!(
        redeem(&hashed, &hashed_id, "v", "elsewhere", forced()),
        NARROWED
    );
    assert_eq!(
        redeem(&hashed, &hashed_id, "v", "r", Externals::Withheld),
        "redeemed"
    );
    // On a row the guard does not select, forcing it changes nothing.
    assert_eq!(redeem(&plain, &plain_id, "c", "r", forced()), "redeemed");
    assert_eq!(
        redeem(&plain, &plain_id, "v", "r", forced()),
        "plain-mismatch"
    );
}
