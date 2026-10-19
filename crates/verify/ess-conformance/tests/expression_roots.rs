//! A value reads what its branch established (`ess/27`,
//! `story:expression-roots-a-branch-establishes-are-readable`, beyond10x/ess#555, #556).
//!
//! `CloseReview` refuses `not-settled` when the incident the review's stored reference names is
//! still `Detected`, and names that incident in its error, `{subject: incident_id}`: the review was
//! read to follow the reference. `FileComplaint` refuses an Optional customer no row carries and
//! names it, `input.customer_id`, read present in its own `exists: false` branch. `SettleIncident`
//! refuses an approved incident and names its approver, `{subject: approved_by}`, read present
//! under `defined(approved_by)`. Synthesis asserts each named value in the error; the interpreted
//! target passes every scenario; a target that names another value fails the refusal's scenario.
use std::collections::BTreeMap;

use ess_compiler::{
    ir::{EssIr, ResolvedPayloadValue},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, target::*, AdmittedSuite,
    ConformanceSuite, Runner, ScenarioStep,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/expression-roots.yaml");
const NOT_SETTLED: &str = "demo.review.CloseReview/outcome/not-settled";
const NO_CUSTOMER: &str = "demo.review.FileComplaint/outcome/no-customer";
const SEALED: &str = "demo.review.SettleIncident/outcome/sealed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("review.yaml"), raw)])
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

/// The fields every `expect_error` step of `scenario` compares.
fn expected_error_fields(suite: &ConformanceSuite, scenario: &str) -> Vec<BTreeMap<String, Node>> {
    let found = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .map_or_else(
            || panic!("no `{scenario}` scenario"),
            |(_, scenario)| scenario,
        );
    found
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectError { fields, .. } => Some(fields.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn subject_field_readable_on_a_refusal_after_existence() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let fields = expected_error_fields(&synthesis.suite, NOT_SETTLED);
    assert!(
        fields
            .iter()
            .any(|fields| fields.contains_key("incident_id")),
        "the refusal's scenario asserts the stored incident id in the error: {fields:#?}"
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model.clone()));
    assert!(statuses.contains_key(NOT_SETTLED), "{statuses:#?}");
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
    let faulty = run(
        &synthesis.suite,
        &NamesAnother(Interpreted::for_model(model), "incident_id"),
    );
    assert!(failing(&faulty).contains(&NOT_SETTLED), "{faulty:#?}");
}

#[test]
fn optional_input_narrowed_in_its_own_exists_false_branch() {
    let model = ir(MODEL);
    let command = model
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.review.FileComplaint")
        .expect("the command");
    let refusal = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "no-customer")
        .expect("the refusal");
    let read = &refusal.error_payload[0];
    assert!(
        matches!(&read.value, ResolvedPayloadValue::InputField { type_ref, .. }
            if !type_ref.is_optional() && *type_ref == read.target_type),
        "the read is the inner type the error field declares: {read:#?}"
    );
    let synthesis = synthesize(&model);
    let fields = expected_error_fields(&synthesis.suite, NO_CUSTOMER);
    assert!(
        fields
            .iter()
            .any(|fields| fields.contains_key("customer_id")),
        "{fields:#?}"
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model.clone()));
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
    let faulty = run(
        &synthesis.suite,
        &NamesAnother(Interpreted::for_model(model), "customer_id"),
    );
    assert!(failing(&faulty).contains(&NO_CUSTOMER), "{faulty:#?}");
}

#[test]
fn stored_optional_narrowed_under_defined() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    let fields = expected_error_fields(&synthesis.suite, SEALED);
    assert!(
        fields
            .iter()
            .any(|fields| fields.contains_key("approved_by")),
        "{fields:#?}"
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model.clone()));
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
    let faulty = run(
        &synthesis.suite,
        &NamesAnother(Interpreted::for_model(model), "approved_by"),
    );
    assert!(failing(&faulty).contains(&SEALED), "{faulty:#?}");
}

/// A target whose declared errors carry another value in the named field.
struct NamesAnother(Interpreted, &'static str);

impl ConformanceTarget for NamesAnother {
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
        let mut result = self.0.execute_command(request)?;
        if let Some(error) = &mut result.error {
            if error.fields.contains_key(self.1) {
                error.fields.insert(
                    self.1.to_owned(),
                    Node::Text("00000000-0000-4000-8000-000000000000".to_owned()),
                );
            }
        }
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
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
