//! A minted value stored present, and a stored value read from the outcome's own payload
//! (`ess/25`, beyond10x/ess#545, `docs/design/minted-value-sources.md`).
//!
//! `SelectAgent` mints a `connect_seq`, publishes it and stores it on an `Optional` field; an
//! invariant requires the field present once the entry is `Bridged`. With `{generated: true}` the
//! interpreted target leaves the field absent and the next transition breaks the invariant. With
//! `{generated: true, present: true}` it stores a value, and with `{payload: <Event>.<field>}` the
//! stored value is the published one, which a synthesized scenario then requires of the row.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, target::*, AdmittedSuite,
    ConformanceSuite, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/minted-value-sources.yaml");
const BRIDGED: &str = "demo.routing.BridgeEntry/outcome/bridged";
const SELECTED: &str = "demo.routing.SelectAgent/outcome/selected";

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

/// The model whose `sets:` reads the value its payload minted.
fn read_from_payload() -> String {
    edit(
        MODEL,
        "          connect_seq: {generated: true, present: true}\n",
        "          connect_seq: {payload: demo.routing.EntryAgentSelected.connect_seq}\n",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("routing.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| {
            let passed = matches!(run.status, Status::Passed);
            (
                run.scenario.to_string(),
                if passed {
                    "passed".to_owned()
                } else {
                    format!("{:?} {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

fn failing(statuses: &BTreeMap<String, String>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id.as_str())
        .collect()
}

/// The issue's model, with the mint stored present: every synthesized scenario passes against the
/// interpreted target, the one that bridges a selected entry included.
#[test]
fn the_interpreter_keeps_a_present_mint_present_through_the_invariant() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(
        synthesis.refusals.is_empty(),
        "{:#?}",
        synthesis.refusals
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model));
    assert!(statuses.contains_key(BRIDGED), "{statuses:#?}");
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
}

/// A target whose rows carry another `connect_seq` than the one it published.
struct StoresAnother(Interpreted);

impl ConformanceTarget for StoresAnother {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.0.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.0.query_view(request)?;
        for row in &mut result.rows {
            if let Some(Node::Number(seq)) = row.get("connect_seq") {
                let other = seq.to_string().parse::<i64>().unwrap_or_default() + 1000;
                row.insert("connect_seq".to_owned(), Node::Number(other.into()));
            }
        }
        Ok(result)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.0.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(r)
    }
}

/// One mint feeds both: the interpreted target passes every scenario, and a target that stores a
/// value other than the one it published fails the scenario of the branch that minted it.
#[test]
fn a_stored_value_read_from_the_payload_equals_the_published_one() {
    let model = ir(&read_from_payload());
    let synthesis = synthesize(&model);
    assert!(
        synthesis.refusals.is_empty(),
        "{:#?}",
        synthesis.refusals
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model.clone()));
    assert!(statuses.contains_key(BRIDGED), "{statuses:#?}");
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");

    let faulty = run(
        &synthesis.suite,
        &StoresAnother(Interpreted::for_model(model)),
    );
    assert!(
        failing(&faulty).contains(&SELECTED),
        "a row holding another value than the published one fails {SELECTED}: {faulty:#?}"
    );
}
