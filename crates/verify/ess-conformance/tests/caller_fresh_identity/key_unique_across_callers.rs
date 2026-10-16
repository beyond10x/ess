//! A key unique across callers survives the caller swap (beyond10x/ess#580).
//!
//! `TakeName` is refused when any row already holds the name, whoever holds it, and records the
//! caller's `id`. The swapped half of each scenario runs against the rows the first half left, so
//! it must send names the first half did not write: every input a row-set selector, a guard or an
//! identity reads is drawn afresh, one rule beside the identity shapes this binary pins (#275,
//! #430, #465). The interpreter implements the model as specified, so it passes every scenario; a
//! target recording one fixed caller still fails the swapped half.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::*;
use ess_conformance::{
    report::Status, synthesize::synthesize, AdmittedSuite, ConformanceSuite, Runner, ScenarioStep,
    ScenarioValue,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("../fixtures/caller-key-unique-across-callers.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("names.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn statuses<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// The `TakeName` sends of a scenario, in order: the caller's `id` and the literal name.
fn sends(steps: &[ScenarioStep]) -> Vec<(Node, Node)> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command,
                caller,
                input,
                ..
            } if command.to_string() == "repro.names.TakeName" => {
                let Some(ScenarioValue::Literal { value }) = input.get("name") else {
                    panic!("TakeName sends a literal name: {input:?}")
                };
                Some((caller["id"].clone(), value.clone()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_key_unique_across_callers_survives_the_caller_swap() {
    let ir = ir();
    let synthesis = synthesize(&ir);
    let suite = synthesis.suite;
    assert!(!suite.scenarios.is_empty(), "{:#?}", synthesis.refusals);
    let mut swapped = 0;
    for (id, scenario) in &suite.scenarios {
        // Both runs send the same commands in the same order, the callers exchanged: a refusal's
        // first run already sends as both (arranged by one, refused to the other).
        let sent = sends(&scenario.steps);
        let (first, again) = sent.split_at(sent.len() / 2);
        let exchanged = |(one, _): &(Node, Node), (other, _): &(Node, Node)| one != other;
        if again.is_empty()
            || first.len() != again.len()
            || !first.iter().zip(again).all(|(a, b)| exchanged(a, b))
        {
            continue;
        }
        swapped += 1;
        let written: BTreeSet<&Node> = first.iter().map(|(_, name)| name).collect();
        for (_, name) in again {
            assert!(
                !written.contains(name),
                "{id}: the swapped half sends {name:?}, which the first half wrote"
            );
        }
    }
    assert!(swapped > 0, "some scenario runs a caller-swapped half");
    let verdicts = statuses(&suite, &Interpreted::for_model(ir.clone()));
    assert!(!verdicts.is_empty());
    for (id, status) in &verdicts {
        assert_eq!(*status, Status::Passed, "{id} on the interpreted target");
    }
}

/// The interpreter, sent every command as the first caller a scenario sends: it records one fixed
/// holder, whoever asks.
struct FixedCaller {
    inner: Interpreted,
    first: RefCell<Option<BTreeMap<String, Node>>>,
}

impl ConformanceTarget for FixedCaller {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.first.borrow_mut() = None;
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut first = self.first.borrow_mut();
        if first.is_none() {
            first.clone_from(&request.caller);
        }
        if request.caller.is_some() {
            request.caller.clone_from(&first);
        }
        drop(first);
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

#[test]
fn a_target_recording_one_fixed_caller_fails_the_swapped_half() {
    let ir = ir();
    let suite = synthesize(&ir).suite;
    let target = FixedCaller {
        inner: Interpreted::for_model(ir),
        first: RefCell::new(None),
    };
    let verdicts = statuses(&suite, &target);
    assert_eq!(
        verdicts.get("repro.names.TakeName/outcome/took"),
        Some(&Status::Failed),
        "{verdicts:#?}"
    );
}
