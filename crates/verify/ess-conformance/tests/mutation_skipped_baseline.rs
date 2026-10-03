//! A baseline with skipped or unsupported scenarios is scored on what it executed.
//!
//! Issue #210. `--collect` refused the whole collection with `ESS-MUTATE-001` when the baseline
//! run had a single scenario that did not pass — a skip included — and `--target` refused the same
//! way. A baseline is now red only when a scenario failed or ended `error`. Each `unsupported` or
//! `skipped` baseline scenario is listed as not scored, with what the run reported, and every
//! mutant is scored on the scenarios the baseline executed, matched by id: a mutant scenario the
//! baseline did not execute is excluded and listed on the mutant. A baseline that executed nothing
//! scores nothing, and says so.

use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, AuditRefusal, Document, Emission, MutantClass, MutationReport, NotScored,
    NotScoredStatus, Verdict, BASELINE_DIR, REPORT_FILE, SUITE_FILE,
};
use ess_conformance::reference::Billing;
use ess_conformance::runner::Runner;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    InvocationObservationRequest, ObservedEvent, ObservedInvocation, RedeliveryRequest,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

fn example(name: &str) -> (Vec<Document>, SourceMap) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name);
    let mut found = Vec::new();
    let mut pending = vec![base];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path.display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).expect("well formed"),
        ));
        texts.insert(label, text);
    }
    (parsed, texts)
}

/// Billing's error-swap emission with the reference's actual report beside every suite.
fn green() -> (Emission, BTreeMap<String, String>) {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, &[MutantClass::ErrorSwap]).expect("emits");
    let mut written = emission.files.clone();
    for dir in dirs(&emission) {
        written.insert(format!("{dir}/{REPORT_FILE}"), report2(&emission, &dir));
    }
    (emission, written)
}

fn dirs(emission: &Emission) -> Vec<String> {
    let mut dirs = vec![BASELINE_DIR.to_owned()];
    dirs.extend(
        emission
            .manifest
            .mutants
            .iter()
            .filter_map(|it| it.dir.clone()),
    );
    dirs
}

fn admitted(emission: &Emission, dir: &str) -> AdmittedSuite {
    AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")]).expect("admitted")
}

fn scenario_ids(emission: &Emission, dir: &str) -> Vec<String> {
    admitted(emission, dir)
        .suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn report2(emission: &Emission, dir: &str) -> String {
    let admitted = admitted(emission, dir);
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &Billing::new());
    CountReport::from_run(&run, &admitted)
        .expect("report/2")
        .to_canonical_json()
        .expect("serializes")
}

#[derive(Clone, Copy)]
enum FixtureOutcome {
    Failed,
    Error,
    Unsupported,
    Skipped,
}

/// Deliberate collector input with exact per-scenario categories, not target execution evidence.
fn fixture_report2(
    emission: &Emission,
    dir: &str,
    assigned: &BTreeMap<String, FixtureOutcome>,
) -> String {
    let admitted = admitted(emission, dir);
    let mut report: Value = serde_json::from_str(&report2(emission, dir)).unwrap();
    let mut passed = Vec::new();
    let mut failed = Vec::new();
    let mut error = Vec::new();
    let mut unsupported = Vec::new();
    let mut skipped = Vec::new();
    let mut matched = 0;
    for id in admitted.suite().scenarios.keys().map(ToString::to_string) {
        let destination = match assigned.get(&id) {
            None => &mut passed,
            Some(FixtureOutcome::Failed) => &mut failed,
            Some(FixtureOutcome::Error) => &mut error,
            Some(FixtureOutcome::Unsupported) => &mut unsupported,
            Some(FixtureOutcome::Skipped) => &mut skipped,
        };
        matched += usize::from(assigned.contains_key(&id));
        destination.push(id);
    }
    assert_eq!(
        matched,
        assigned.len(),
        "every fixture outcome names the suite"
    );
    let execution = if !failed.is_empty() || !unsupported.is_empty() {
        "failed"
    } else {
        "inconclusive"
    };
    report["producer_profile"] = "go-scenario-status/2".into();
    report["counts"] = serde_json::json!({
        "total": admitted.suite().len(),
        "passed": passed.len(),
        "failed": failed.len(),
        "error": error.len(),
        "unsupported": unsupported.len(),
        "skipped": skipped.len()
    });
    report["outcomes"] = serde_json::json!({
        "passed": passed,
        "failed": failed,
        "error": error,
        "unsupported": unsupported,
        "skipped": skipped
    });
    report["execution_status"] = execution.into();
    report["conformance_status"] = execution.into();
    let text = serde_json::to_string_pretty(&report).unwrap() + "\n";
    CountReport::from_json(&text, &admitted).expect("coherent report/2 collector fixture");
    text
}

fn collect(written: &BTreeMap<String, String>) -> Result<MutationReport, AuditRefusal> {
    mutate::collect(|path| written.get(path).cloned())
}

#[test]
fn skipped_and_unsupported_baseline_scenarios_are_listed_and_the_rest_is_scored() {
    let (emission, mut written) = green();
    let full = collect(&written).expect("a green baseline collects");
    let ids = scenario_ids(&emission, BASELINE_DIR);
    let (skipped, unsupported) = (ids[0].clone(), ids[1].clone());
    written.insert(
        format!("{BASELINE_DIR}/{REPORT_FILE}"),
        fixture_report2(
            &emission,
            BASELINE_DIR,
            &BTreeMap::from([
                (skipped.clone(), FixtureOutcome::Skipped),
                (unsupported.clone(), FixtureOutcome::Unsupported),
            ]),
        ),
    );
    let report = collect(&written).unwrap_or_else(|refusal| panic!("scored, not {refusal}"));
    assert_eq!(
        report.baseline.not_scored,
        [
            NotScored {
                scenario: skipped.clone(),
                status: NotScoredStatus::Skipped,
            },
            NotScored {
                scenario: unsupported.clone(),
                status: NotScoredStatus::Unsupported,
            },
        ]
    );
    assert_eq!(report.counts.killed, full.counts.killed);
    for entry in report.mutants.iter().filter(|it| it.scenarios.is_some()) {
        let excluded = entry.excluded.as_deref().unwrap_or_default();
        assert!(
            excluded.contains(&skipped) && excluded.contains(&unsupported),
            "{}: {excluded:?}",
            entry.id
        );
        for killer in entry.killers.iter().flatten() {
            assert!(killer != &skipped && killer != &unsupported, "{}", entry.id);
        }
    }
    let text = report.render_text();
    assert!(
        text.lines().next().unwrap().contains("2 not scored"),
        "{text}"
    );
    assert!(
        text.contains(&format!(
            "not scored {skipped}: the baseline's run reported it skipped"
        )),
        "{text}"
    );
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(
        json["baseline"]["not_scored"][0],
        serde_json::json!({"scenario": skipped, "status": "skipped"})
    );
}

#[test]
fn a_mutant_failing_only_where_the_baseline_skipped_is_not_killed_by_it() {
    for changed in [true, false] {
        excluded_scenario_case(changed);
    }
}

/// A mutant whose only failing scenario is one the baseline skipped, and which the mutant either
/// changed or holds exactly as the baseline does.
fn excluded_scenario_case(changed: bool) {
    let (emission, mut written) = green();
    let baseline = admitted(&emission, BASELINE_DIR);
    let (mutant, shared) = emission
        .manifest
        .mutants
        .iter()
        .filter_map(|it| it.dir.clone())
        .find_map(|dir| {
            let suite = admitted(&emission, &dir);
            let (id, _) = suite.suite().scenarios.iter().find(|(id, scenario)| {
                baseline
                    .suite()
                    .scenarios
                    .get(*id)
                    .is_some_and(|it| (it != *scenario) == changed)
            })?;
            Some((dir, id.to_string()))
        })
        .expect("an error-swap mutant shares such a scenario with the baseline");
    written.insert(
        format!("{BASELINE_DIR}/{REPORT_FILE}"),
        fixture_report2(
            &emission,
            BASELINE_DIR,
            &BTreeMap::from([(shared.clone(), FixtureOutcome::Skipped)]),
        ),
    );
    written.insert(
        format!("{mutant}/{REPORT_FILE}"),
        fixture_report2(
            &emission,
            &mutant,
            &BTreeMap::from([(shared.clone(), FixtureOutcome::Failed)]),
        ),
    );
    let report = collect(&written).expect("collects");
    let entry = report.mutants.iter().find(|it| it.id == mutant).unwrap();
    // Never killed by what the baseline did not execute. A changed copy might have killed it, so
    // nobody found out; an unchanged copy asks what the baseline asked, so it is a survivor.
    let expected = if changed {
        Verdict::Inconclusive
    } else {
        Verdict::Survived
    };
    assert_eq!(entry.verdict, expected, "{entry:?}");
    assert_eq!(entry.killers, None);
    assert_eq!(entry.excluded.as_deref(), Some(&[shared.clone()][..]));
    let text = report.render_text();
    assert!(
        text.contains(&format!(
            "not scored, as the baseline did not execute them: {shared} (1 in total)"
        )),
        "{text}"
    );
}

#[test]
fn a_baseline_scenario_that_failed_or_ended_error_is_still_refused_with_mutate_001() {
    for status in ["failed", "error"] {
        let (emission, mut written) = green();
        let ids = scenario_ids(&emission, BASELINE_DIR);
        let red = match status {
            "failed" => FixtureOutcome::Failed,
            "error" => FixtureOutcome::Error,
            _ => unreachable!(),
        };
        written.insert(
            format!("{BASELINE_DIR}/{REPORT_FILE}"),
            fixture_report2(
                &emission,
                BASELINE_DIR,
                &BTreeMap::from([
                    (ids[0].clone(), red),
                    (ids[1].clone(), FixtureOutcome::Skipped),
                ]),
            ),
        );
        let refusal = collect(&written).expect_err("a red baseline is refused");
        let AuditRefusal::BaselineFailed { not_passed, .. } = &refusal else {
            panic!("ESS-MUTATE-001, not {refusal}");
        };
        assert_eq!(not_passed, &[ids[0].clone()], "only the red one: {status}");
    }
}

#[test]
fn a_baseline_that_executed_nothing_scores_nothing() {
    let (emission, mut written) = green();
    let ids = scenario_ids(&emission, BASELINE_DIR);
    let assigned = ids
        .iter()
        .map(|id| (id.clone(), FixtureOutcome::Skipped))
        .collect();
    written.insert(
        format!("{BASELINE_DIR}/{REPORT_FILE}"),
        fixture_report2(&emission, BASELINE_DIR, &assigned),
    );
    let refusal = collect(&written).expect_err("nothing is scored");
    let AuditRefusal::NothingScored { not_scored, .. } = &refusal else {
        panic!("nothing scored, not {refusal}");
    };
    assert_eq!(not_scored.len(), ids.len());
    assert!(
        refusal.to_string().starts_with("nothing scored"),
        "{refusal}"
    );
    assert!(refusal.is_inconclusive());
    assert_eq!(refusal.code(), None);
}

#[test]
fn a_go_report2_baseline_with_skipped_scenarios_is_scored() {
    let (emission, mut written) = green();
    let admitted = admitted(&emission, BASELINE_DIR);
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &Billing::new());
    let text = CountReport::from_run(&run, &admitted)
        .expect("report/2")
        .to_canonical_json()
        .expect("serializes");
    let mut value: Value = serde_json::from_str(&text).unwrap();
    let skipped = value["outcomes"]["passed"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    value["producer_profile"] = Value::from("go-scenario-status/1");
    value["outcomes"]["skipped"] = Value::Array(vec![skipped.clone()]);
    let passed = value["counts"]["passed"].as_u64().unwrap();
    value["counts"]["passed"] = Value::from(passed - 1);
    value["counts"]["skipped"] = Value::from(1);
    value["execution_status"] = Value::from("inconclusive");
    value["conformance_status"] = Value::from("inconclusive");
    let text = serde_json::to_string_pretty(&value).unwrap() + "\n";
    CountReport::from_json(&text, &admitted).expect("the rewritten report/2 is coherent");
    written.insert(format!("{BASELINE_DIR}/{REPORT_FILE}"), text);
    let report = collect(&written).unwrap_or_else(|refusal| panic!("scored, not {refusal}"));
    assert_eq!(
        report.baseline.not_scored,
        [NotScored {
            scenario: skipped.as_str().unwrap().to_owned(),
            status: NotScoredStatus::Skipped,
        }]
    );
}

#[test]
fn the_built_in_audit_scores_past_unsupported_baseline_scenarios() {
    // Deliberately remove view support from the real interpreter. This control must keep testing
    // unsupported baseline scoring even as the production target gains capabilities.
    let (files, texts) = example("billing");
    let ir = mutate::compile(files.clone(), &texts).unwrap();
    let report = mutate::audit(&files, &texts, &[MutantClass::ErrorSwap], || {
        WithoutViews(Interpreted::for_model(ir.clone()))
    })
    .unwrap_or_else(|refusal| panic!("scored, not {refusal}"));
    assert!(!report.baseline.not_scored.is_empty());
    assert!(report.baseline.not_scored.len() < report.baseline.scenarios);
    assert!(report
        .baseline
        .not_scored
        .iter()
        .all(|it| it.status == NotScoredStatus::Unsupported));
    assert!(report.counts.killed > 0, "{}", report.render_text());
    for entry in &report.mutants {
        for killer in entry.killers.iter().flatten() {
            assert!(
                !report
                    .baseline
                    .not_scored
                    .iter()
                    .any(|it| &it.scenario == killer),
                "{}",
                entry.id
            );
        }
    }
}

struct WithoutViews(Interpreted);

impl ConformanceTarget for WithoutViews {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }

    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(context)
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }

    fn query_view(&self, _request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            "query view",
            "deliberately absent in this audit control",
        ))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }

    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.0.observe_invocations(request)
    }

    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(context)
    }
}
