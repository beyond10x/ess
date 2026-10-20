//! A guard over the rows a `List` of references names (`ess/28`, beyond10x/ess#558,
//! `docs/design/generic-language-features.md` § G2, quantifiers): `when_related: {via:
//! input.<list>, exists: false}` refuses a list naming a missing row, `{missing: input.<list>}`
//! names it, and a list whose every element names a row — the empty one included — is accepted.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// An incident names the services it affects; one naming a service no row carries is refused.
const LEDGER: &str = include_str!("fixtures/list-of-references.yaml");

const OPEN: &str = "ledger.desk.OpenIncident";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(model: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(model);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id} among {:?}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The list each `OpenIncident` the scenario sends carries, in step order, with the outcome or
/// error expected right after it.
fn sends(scenario: &ConformanceScenario) -> Vec<(ScenarioValue, String)> {
    let mut found = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != OPEN {
            continue;
        }
        let answer = match steps.peek() {
            Some(ScenarioStep::ExpectOutcome { outcome }) => outcome.to_string(),
            other => format!("{other:?}"),
        };
        found.push((input["services"].clone(), answer));
    }
    found
}

fn passes_on_the_interpreter(model: EssIr, suite: &ConformanceSuite) {
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite is admitted");
    let run = Runner::for_suite(admitted.suite()).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(model),
    );
    assert_ne!(run.scenarios.len(), 0);
    let failed: Vec<_> = run
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_missing_list_element_takes_the_refusal() {
    let model = ir(LEDGER);
    let suite = suite(&model);
    let refused = scenario(&suite, "ledger.desk.OpenIncident/outcome/unknown-service");
    let sent = sends(refused);
    let Some((ScenarioValue::List { items }, _)) = sent.last() else {
        panic!("the refusal sends a list: {sent:#?}");
    };
    let [ScenarioValue::Instance { .. }, ScenarioValue::Literal { value: missing }] =
        items.as_slice()
    else {
        panic!("a created row first, then an identity no row carries: {items:#?}");
    };
    let asserted: Vec<BTreeMap<String, Node>> = refused
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectError { fields, .. } => Some(fields.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        asserted,
        vec![BTreeMap::from([("service_id".to_owned(), missing.clone())])],
        "the refusal names the second element, the one naming no row"
    );
    passes_on_the_interpreter(model, &suite);
}

#[test]
fn every_element_present_takes_the_accepting_branch() {
    let model = ir(LEDGER);
    let suite = suite(&model);
    let opened = scenario(&suite, "ledger.desk.OpenIncident/outcome/opened");
    let sent = sends(opened);
    let Some((ScenarioValue::List { items }, answer)) = sent.first() else {
        panic!("the accepting branch sends a list: {sent:#?}");
    };
    assert!(
        items.len() == 2
            && items
                .iter()
                .all(|item| matches!(item, ScenarioValue::Instance { .. })),
        "two rows the scenario created: {items:#?}"
    );
    assert_eq!(answer, "ledger.desk.OpenIncident/opened");
    passes_on_the_interpreter(model, &suite);
}

#[test]
fn an_empty_list_takes_the_accepting_branch() {
    let model = ir(LEDGER);
    let suite = suite(&model);
    let empty = ScenarioValue::Literal {
        value: Node::Seq(Vec::new()),
    };
    let witnessed: Vec<String> = suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| {
            sends(scenario)
                .iter()
                .any(|(list, answer)| *list == empty && answer == "ledger.desk.OpenIncident/opened")
        })
        .map(|(id, _)| id.to_string())
        .collect();
    assert!(
        !witnessed.is_empty(),
        "some scenario sends an empty list and requires it accepted: {:#?}",
        suite
            .scenarios
            .iter()
            .map(|(id, scenario)| (id.to_string(), sends(scenario)))
            .collect::<Vec<_>>()
    );
    passes_on_the_interpreter(model, &suite);
}
