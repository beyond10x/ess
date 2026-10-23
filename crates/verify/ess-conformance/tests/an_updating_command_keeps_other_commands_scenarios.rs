//! Adding updating commands to an entity keeps the scenarios of the commands it already had
//! (beyond10x/ess#679).
//!
//! The adopter's report: an entity with `CreateAgent` and `UpdateAgent` gained `MintEmbedToken`
//! and `RevokeEmbedToken`, both `updates:`, and `CreateAgent/accepted` and `UpdateAgent/updated`
//! vanished under ESS-SYNTH-001. The two token commands are not what removes them. Each of those
//! branches copies an Optional input into the row, so its scenario carries a further run with
//! that Optional left out, and the by-id view `AgentById` puts a second row beside the subject
//! in every run, asserted `Excludes`. Every run numbered that row from the first further
//! distinction, so the second run created a second agent under the identity the first run's row
//! still held, and the whole scenario was withdrawn as creating one identity twice.
//!
//! `tests/fixtures/updating-commands-keep-scenarios.yaml` also holds `RaiseQuota`, whose
//! `missing` branch no input can witness: a `Plan` has two values, so no plan is known to name no
//! quota. That one branch is refused, and nothing else.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/updating-commands-keep-scenarios.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec =
        Specification::assemble([(Source::new("updating-commands-keep-scenarios.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn statuses(result: &Synthesis, model: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn refused(result: &Synthesis) -> Vec<(Option<String>, String, String)> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            (
                refusal.scenario.as_ref().map(ToString::to_string),
                refusal.cause.code().to_string(),
                refusal.cause.to_string(),
            )
        })
        .collect()
}

#[test]
fn an_added_updating_command_keeps_the_other_commands_scenarios() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refused(&result);
    let passed = statuses(&result, &model);
    for wanted in [
        "demo.agents.CreateAgent/outcome/accepted",
        "demo.agents.UpdateAgent/outcome/updated",
        "demo.agents.MintEmbedToken/outcome/minted",
        "demo.agents.RevokeEmbedToken/outcome/revoked",
    ] {
        assert_eq!(
            passed.get(wanted),
            Some(&Status::Passed),
            "{wanted} is emitted and passes on the interpreter: {passed:#?}\n{refused:#?}"
        );
    }
    let create = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.agents.CreateAgent/outcome/accepted")
        .map(|(_, scenario)| scenario)
        .expect("CreateAgent/accepted is emitted");
    let runs = create
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ess_conformance::ScenarioStep::ExpectOutcome { outcome }
                if outcome.command.name().to_string() == "demo.agents.CreateAgent"
                    && outcome.outcome.to_string() == "accepted")
        })
        .count();
    assert!(
        runs > 2,
        "the scenario still carries its further runs and their companions, {runs} creations"
    );
}

#[test]
fn a_refused_branch_names_itself_and_drops_nothing_else() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refused(&result);
    assert_eq!(
        refused
            .iter()
            .map(|(scenario, code, _)| (scenario.as_deref(), code.as_str()))
            .collect::<Vec<_>>(),
        vec![(
            Some("demo.agents.RaiseQuota/outcome/missing"),
            "ESS-SYNTH-001"
        )],
        "{refused:#?}"
    );
    let passed = statuses(&result, &model);
    let mut emitted: Vec<&str> = passed.keys().map(String::as_str).collect();
    emitted.sort_unstable();
    assert_eq!(
        emitted,
        vec![
            "demo.agents.CreateAgent/outcome/accepted",
            "demo.agents.MintEmbedToken/outcome/minted",
            "demo.agents.RaiseQuota/outcome/raised",
            "demo.agents.RevokeEmbedToken/outcome/revoked",
            "demo.agents.SetQuota/outcome/set",
            "demo.agents.UpdateAgent/outcome/updated",
        ],
        "every branch but the refused one is emitted"
    );
    assert!(
        passed.values().all(|status| *status == Status::Passed),
        "{passed:#?}"
    );
}
