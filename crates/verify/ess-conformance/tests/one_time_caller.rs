//! Disclosure scenarios of a one-time outcome that records who issued it (beyond10x/ess#550).
//!
//! The outcome `keys.api.IssueKey/issued` sets `issued_by: {caller: user_id}` and protects its
//! `secret`. Every disclosure scenario sends the command, so each must send it authenticated as a
//! caller carrying `user_id`, as the outcome's own scenario does; without one the interpreted
//! target has no value for the read and the scenario comes back unsupported.
use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::{
    ConformanceTarget, EntitySetupRequest, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RecordedCommandCompletion, RedeliveryRequest,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};
use ess_conformance::{report::Status, ScenarioId, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const WITH_CALLER: &str = include_str!("fixtures/one-time-caller/with-caller.yaml");
const WITHOUT_CALLER: &str = include_str!("fixtures/one-time-caller/without-caller.yaml");

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn run(ir: &EssIr, target_ir: EssIr) -> ess_conformance::report::ConformanceReport {
    let synthesis = ess_conformance::synthesize(ir);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = ess_conformance::interpret::Interpreted::for_model(target_ir);
    ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
}

fn statuses(report: &ess_conformance::report::ConformanceReport) -> Vec<(String, Status)> {
    report
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

#[test]
fn the_disclosure_scenarios_bind_the_caller_the_outcome_reads() {
    let ir = model(WITH_CALLER);
    let synthesis = ess_conformance::synthesize(&ir);
    let disclosures: Vec<_> = synthesis
        .suite
        .scenarios
        .iter()
        .filter(|(id, _)| matches!(id, ScenarioId::Disclosure { .. }))
        .collect();
    assert_eq!(
        disclosures.len(),
        4,
        "{:?}",
        synthesis.suite.scenarios.keys()
    );
    for (id, scenario) in disclosures {
        let commands: Vec<_> = scenario
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExecuteCommand { caller, .. }
                | ScenarioStep::ExecuteCommandWithoutInput { caller, .. } => Some(caller),
                _ => None,
            })
            .collect();
        assert!(!commands.is_empty(), "{id} sends no command");
        for caller in commands {
            assert!(
                caller.contains_key("user_id"),
                "{id} sends a command without the caller attribute the outcome reads"
            );
        }
    }
    let report = run(&ir, ir.clone());
    let statuses = statuses(&report);
    assert_eq!(statuses.len(), 5, "{statuses:?}");
    assert!(
        statuses.iter().all(|(_, status)| *status == Status::Passed),
        "{statuses:?}\n{}",
        serde_json::to_string_pretty(&report.scenarios).unwrap()
    );
}

#[test]
fn a_model_reading_no_caller_value_passes_its_disclosure_scenarios_as_before() {
    let ir = model(WITHOUT_CALLER);
    // No command reads the caller, so no disclosure step is sent as one: the suite keeps the bytes
    // it was synthesized with before the disclosure cells learned to bind a caller.
    let synthesis = ess_conformance::synthesize(&ir);
    for (id, scenario) in &synthesis.suite.scenarios {
        if !matches!(id, ScenarioId::Disclosure { .. }) {
            continue;
        }
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { caller, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { caller, .. } = step
            {
                assert!(caller.is_empty(), "{id} gained a caller");
            }
        }
    }
    let statuses = statuses(&run(&ir, ir.clone()));
    assert_eq!(statuses.len(), 5, "{statuses:?}");
    assert!(
        statuses.iter().all(|(_, status)| *status == Status::Passed),
        "{statuses:?}"
    );
}

/// How the [`Mutant`] departs from the interpreted target.
#[derive(Clone, Copy, Debug)]
enum Defect {
    /// Ignores the caller each command is sent as, so it has no value to record as `issued_by`.
    DropsTheCaller,
    /// Answers every issuance after the first in a scenario with the first one's secret.
    DisclosesTwice,
}

/// The interpreted target, wrong in one way.
struct Mutant {
    inner: Interpreted,
    defect: Defect,
    first: RefCell<Option<Node>>,
}

impl Mutant {
    fn recorded(&self, mut request: SemanticCommandRequest) -> RecordedCommandCompletion {
        if matches!(self.defect, Defect::DropsTheCaller) {
            request.caller = None;
        }
        let mut completion = self.inner.execute_command_recorded(request);
        if let (Defect::DisclosesTwice, Ok(result)) = (self.defect, &mut completion.answer) {
            if let Some(secret) = result
                .response
                .as_mut()
                .and_then(|response| response.get_mut("secret"))
            {
                let mut first = self.first.borrow_mut();
                match &*first {
                    Some(earlier) => secret.clone_from(earlier),
                    None => *first = Some(secret.clone()),
                }
            }
        }
        completion
    }
}

impl ConformanceTarget for Mutant {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.first.borrow_mut() = None;
        self.inner.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.recorded(request).answer
    }
    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> RecordedCommandCompletion {
        self.recorded(request)
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
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

#[test]
fn a_target_dropping_the_caller_or_disclosing_twice_fails_a_disclosure_scenario() {
    let ir = model(WITH_CALLER);
    let synthesis = ess_conformance::synthesize(&ir);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let mut verdicts = BTreeMap::new();
    for defect in [Defect::DropsTheCaller, Defect::DisclosesTwice] {
        let target = Mutant {
            inner: Interpreted::for_model(ir.clone()),
            defect,
            first: RefCell::new(None),
        };
        let report = ess_conformance::Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &target)
            .into_report();
        let statuses = statuses(&report);
        let disclosures: Vec<_> = statuses
            .iter()
            .filter(|(id, _)| id.contains("/disclosure/"))
            .collect();
        assert_eq!(disclosures.len(), 4, "{defect:?}: {statuses:?}");
        assert!(
            disclosures
                .iter()
                .any(|(_, status)| *status != Status::Passed),
            "{defect:?} passed every disclosure scenario: {statuses:?}"
        );
        verdicts.insert(format!("{defect:?}"), statuses);
    }
    // Disclosing the secret twice is a failure of the declared rule, not a target that could
    // not answer: a disclosure scenario that sends the command twice fails.
    assert!(
        verdicts["DisclosesTwice"]
            .iter()
            .any(|(id, status)| id.contains("/disclosure/") && *status == Status::Failed),
        "{verdicts:?}"
    );
    // With no caller to record, no disclosure scenario passes.
    assert!(
        verdicts["DropsTheCaller"]
            .iter()
            .filter(|(id, _)| id.contains("/disclosure/"))
            .all(|(_, status)| *status != Status::Passed),
        "{verdicts:?}"
    );
}

const TWO_COMMANDS: &str = include_str!("fixtures/one-time-caller/two-commands.yaml");
const WITH_VIEW: &str = include_str!("fixtures/one-time-caller/with-view.yaml");
const EXISTING_KEY: &str = include_str!("fixtures/one-time-caller/existing-key.yaml");
const GUIDE: &str = include_str!("../../../../website/docs/guides/verify/one-time-responses.md");
const GUIDE_HEADING: &str = "## The disclosure scenarios synthesis adds";
const RETRY: &str = "keys.api.IssueKey/disclosure/issued/secret/retry/as/actor/keys.api.Admin";

/// Every scenario of `ir`'s suite and its disclosure scenarios, as the runner would count them.
fn counts(ir: &EssIr) -> (usize, usize) {
    let suite = ess_conformance::synthesize(ir).suite;
    let disclosures = suite
        .scenarios
        .keys()
        .filter(|id| matches!(id, ScenarioId::Disclosure { .. }))
        .count();
    (suite.scenarios.len(), disclosures)
}

/// The fences of `language` in `body`, in order.
fn fences<'a>(body: &'a str, language: &str) -> Vec<&'a str> {
    body.split(format!("```{language}\n").as_str())
        .skip(1)
        .map(|fence| fence.split("```").next().unwrap())
        .collect()
}

/// The guide lists the cells synthesis adds for a `one_time_response` field, the refused ones, the
/// id shape and the order of growth, and its worked counts are the ones synthesis produces for the
/// three minimal variants (beyond10x/ess#573).
#[test]
fn one_time_guide_lists_the_cells() {
    let marker = format!("\n{GUIDE_HEADING}\n");
    let body = GUIDE
        .split(marker.as_str())
        .nth(1)
        .unwrap_or_else(|| panic!("the guide has no heading `{GUIDE_HEADING}`"))
        .split("\n## ")
        .next()
        .unwrap();
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let missing: Vec<&str> = [
        "`<command>/disclosure/<outcome>/<field>/<aspect>/as/<actor>`",
        "`origin`",
        "`retry`",
        "`rotation`",
        "`command/<command>/<outcome>`",
        "`denied/<command>`",
        "`read/<view>`",
        "`as/anonymous`",
        "`existing_instance:`",
        "actors × (outcomes + commands + views)",
        "refused",
    ]
    .into_iter()
    .filter(|phrase| !normalized.contains(phrase))
    .collect();
    assert_eq!(
        missing,
        Vec::<&str>::new(),
        "the section does not state these"
    );

    // The worked model is the one this file's caller tests run, so the variants below extend it.
    let models = fences(body, "yaml");
    assert_eq!(models, [WITH_CALLER], "the section fences the worked model");
    let ir = model(WITH_CALLER);
    let synthesis = ess_conformance::synthesize(&ir);
    let built: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .filter(|id| matches!(id, ScenarioId::Disclosure { .. }))
        .map(ToString::to_string)
        .collect();
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .filter_map(|refusal| refusal.scenario.as_ref())
        .filter(|id| matches!(id, ScenarioId::Disclosure { .. }))
        .map(ToString::to_string)
        .collect();
    for ordinary in synthesis
        .suite
        .scenarios
        .keys()
        .filter(|id| !matches!(id, ScenarioId::Disclosure { .. }))
    {
        let named = format!("`{ordinary}`");
        assert!(
            normalized.contains(&named),
            "the section does not name {named}"
        );
    }
    let listed = fences(body, "text");
    let [built_listed, refused_listed] = listed.as_slice() else {
        panic!("the section lists the built and the refused cells: {listed:?}");
    };
    assert_eq!(built_listed.lines().collect::<Vec<_>>(), built);
    assert_eq!(refused_listed.lines().collect::<Vec<_>>(), refused);

    let worked = [
        ("1 command, 2 actors", WITH_CALLER, (5, 4)),
        ("2 commands, 3 actors", TWO_COMMANDS, (10, 8)),
        ("2 commands, 3 actors, 1 view", WITH_VIEW, (11, 9)),
    ];
    for (variant, text, expected) in worked {
        let (scenarios, disclosures) = counts(&model(text));
        assert_eq!((scenarios, disclosures), expected, "{variant}");
        let row = format!("| {variant} | {scenarios} | {disclosures} |");
        assert!(normalized.contains(&row), "the section has no row `{row}`");
    }
}

/// A create whose identity the caller supplies, with no declared answer for an identity a row
/// already carries: the retry cell is refused naming that missing declaration, not a surviving
/// subject (beyond10x/ess#574).
#[test]
fn supplied_identity_retry_names_its_cause() {
    let synthesis = ess_conformance::synthesize(&model(WITH_CALLER));
    let retry: ScenarioId = RETRY.parse().unwrap();
    assert!(!synthesis.suite.scenarios.contains_key(&retry));
    let causes: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| refusal.scenario.as_ref() == Some(&retry))
        .map(|refusal| refusal.cause.to_string())
        .collect();
    let [cause] = causes.as_slice() else {
        panic!("one refusal names the retry cell: {causes:?}");
    };
    assert!(!cause.contains("surviving subject"), "{cause}");
    for phrase in ["`key_id`", "`keys.api.IssueKey`", "`existing_instance:`"] {
        assert!(cause.contains(phrase), "{cause} does not name {phrase}");
    }
}

/// With an `existing_instance:` refusal declared, the retry is the origin's own invocation sent
/// again over the row it created, answered by that refusal, and it passes on the interpreted
/// target (beyond10x/ess#574).
#[test]
fn supplied_identity_retry_expects_the_declared_refusal() {
    let ir = model(EXISTING_KEY);
    let synthesis = ess_conformance::synthesize(&ir);
    let retry: ScenarioId = RETRY.parse().unwrap();
    let scenario = synthesis.suite.scenario(&retry).unwrap_or_else(|| {
        panic!(
            "no retry cell: {:#?}",
            synthesis
                .refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        )
    });
    let invocations: Vec<_> = scenario
        .steps
        .iter()
        .filter(|step| matches!(step, ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == "keys.api.IssueKey"))
        .collect();
    assert_eq!(invocations.len(), 2, "{:#?}", scenario.steps);
    assert_eq!(invocations[0], invocations[1], "the retry is not identical");
    let branches: Vec<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(branches, ["issued", "already-issued"]);
    ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = run(&ir, ir.clone());
    let statuses = statuses(&report);
    assert!(
        statuses
            .iter()
            .any(|(id, status)| id == RETRY && *status == Status::Passed),
        "{statuses:?}"
    );
    assert!(
        statuses.iter().all(|(_, status)| *status == Status::Passed),
        "{statuses:?}"
    );
}
