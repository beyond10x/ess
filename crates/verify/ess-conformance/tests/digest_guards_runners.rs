//! The native, Go and TypeScript runners agree on every scenario of a suite whose guard reads a
//! fact through a digest or encoding function (`ess/27`,
//! `story:digest-and-encoding-functions-in-expressions`, beyond10x/ess#584).
//!
//! Each generated runtime replays the transcript the Rust reference runner recorded while it asked
//! the interpreted target — and a target that skips the S256 check — the same questions, so a verdict
//! that differs is a difference in how the runners read the suite, and a request one sends that the
//! reference runner did not is written down as a divergence.

mod support_go;
mod support_ts;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, target::*, AdmittedSuite,
    Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/digest-guard.yaml");
const MISMATCH: &str = "demo.pkce.Exchange/outcome/mismatch";

fn ir() -> EssIr {
    ir_of(MODEL)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("pkce.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn written(status: Status) -> String {
    match status {
        Status::Passed => "passed",
        Status::Failed => "failed",
        Status::Error => "error",
        Status::Unsupported => "unsupported",
    }
    .to_owned()
}

/// The reference verdicts against `target`, and the transcript of what it answered.
fn recorded<T: ConformanceTarget>(
    suite: &AdmittedSuite,
    target: T,
) -> (BTreeMap<String, String>, support_go::Recorder<T>) {
    let recorder = support_go::Recorder::new(target);
    let verdicts = Runner::for_suite(suite.suite())
        .run_admitted(suite, &recorder)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), written(run.status)))
        .collect();
    (verdicts, recorder)
}

/// A target that never refuses the verifier: the S256 check left out.
struct SkipsTheCheck(Interpreted);

impl ConformanceTarget for SkipsTheCheck {
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
        let result = self.0.execute_command(request)?;
        if result
            .outcome
            .as_ref()
            .is_some_and(|outcome| outcome.to_string() == MISMATCH.replace("/outcome/", "/"))
        {
            return Ok(SemanticCommandResult::undeclared());
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

fn go_replay<T>(
    directory: &std::path::Path,
    recorder: &support_go::Recorder<T>,
) -> BTreeMap<String, String> {
    let replayed = support_go::replay(directory, recorder, &[]);
    assert!(
        replayed.divergences.is_empty(),
        "the Go runtime sent requests the reference runner did not:\n{}\n{}",
        replayed.divergences.join("\n"),
        replayed.go.log
    );
    assert!(
        !replayed.go.outcomes.is_empty(),
        "the Go run published no verdicts:\n{}",
        replayed.go.log
    );
    replayed.go.outcomes
}

fn ts_replay<T>(
    package: &support_ts::Package,
    suite: &AdmittedSuite,
    recorder: &support_go::Recorder<T>,
) -> BTreeMap<String, String> {
    let replayed = package.replay(suite, &recorder.transcript(), false);
    assert_ne!(replayed.exit, Some(2), "{}", replayed.log);
    assert!(
        replayed.divergences.is_empty(),
        "the TypeScript runtime sent requests the reference runner did not:\n{}\n{}",
        replayed.divergences.join("\n"),
        replayed.log
    );
    replayed.outcomes
}

#[test]
fn the_native_go_and_typescript_runners_agree_on_every_scenario() {
    agree(ir(), "digest", false);
}

#[test]
fn the_runners_agree_without_the_held_state_refusal_beside_it() {
    agree(
        ir_of(&MODEL.replace(
            "      - name: redirect-mismatch
        when_subject: {predicate: redirect_uri != input.redirect_uri}
        error: demo.pkce.RedirectMismatch
",
            "",
        )),
        "digest-alone",
        true,
    );
}

/// The three runners' verdicts over one suite: against the interpreted target, and where
/// `faulty`, against a target that skips the check. A target answering no declared branch ends
/// the scenario where the reference runner stops it, so the faulty run is compared on the suite
/// whose refusal scenario ends with that answer.
fn agree(model: EssIr, label: &str, faulty: bool) {
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let directory = support_go::package(label, suite.suite(), &[support_go::TRANSCRIPT_TARGET]);
    let package = support_ts::Package::new(label, suite.suite());

    let (rust, recorder) = recorded(&suite, Interpreted::for_model(model.clone()));
    assert!(rust.values().all(|status| status == "passed"), "{rust:#?}");
    assert_eq!(
        go_replay(&directory, &recorder),
        rust,
        "Go (left), Rust (right)"
    );
    assert_eq!(
        ts_replay(&package, &suite, &recorder),
        rust,
        "TypeScript (left), Rust (right)"
    );

    if faulty {
        let (rust, recorder) = recorded(&suite, SkipsTheCheck(Interpreted::for_model(model)));
        assert_eq!(
            rust.get(MISMATCH).map(String::as_str),
            Some("failed"),
            "{rust:#?}"
        );
        assert_eq!(
            go_replay(&directory, &recorder),
            rust,
            "Go (left), Rust (right)"
        );
        assert_eq!(
            ts_replay(&package, &suite, &recorder),
            rust,
            "TypeScript (left), Rust (right)"
        );
    }
    let _ = std::fs::remove_dir_all(&*directory);
}
