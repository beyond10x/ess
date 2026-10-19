//! A guard reading a fact through a digest or encoding function is witnessed both ways (`ess/27`,
//! `story:digest-and-encoding-functions-in-expressions`, beyond10x/ess#584).
//!
//! `Exchange` refuses `mismatch` when the stored code was issued for `S256` and its challenge is not
//! `base64url(sha256(ascii(input.verifier)))`. Synthesis picks a verifier and issues the code with
//! the challenge that verifier computes for the accepting branch, and sends a verifier that computes
//! another for `mismatch`. The interpreted target passes every scenario, and a target that skips the
//! check fails `mismatch`'s. Beside `redirect-mismatch`, a held-state refusal declared after it, a
//! scenario where both hold requires `mismatch`'s error, the declaration order. The suite is the
//! same document for the Rust, Go and TypeScript runners, and each admits it.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, target::*, AdmittedSuite,
    ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::expr::digest::{base64url, sha256};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/digest-guard.yaml");
const MISMATCH: &str = "demo.pkce.Exchange/outcome/mismatch";
const REDEEMED: &str = "demo.pkce.Exchange/outcome/redeemed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("pkce.yaml"), raw)])
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

fn text(value: Option<&ScenarioValue>) -> Option<String> {
    match value? {
        ScenarioValue::Literal {
            value: Node::Text(text),
        } => Some(text.clone()),
        _ => None,
    }
}

/// Each `Exchange` the scenario sends, with the issued code's challenge and method, the redirect it
/// was issued for, and the outcome it requires.
#[derive(Debug)]
struct Exchange {
    challenge: String,
    method: String,
    issued_redirect: String,
    verifier: String,
    redirect: String,
    outcome: String,
}

fn exchanges(suite: &ConformanceSuite, scenario: &str) -> Vec<Exchange> {
    let steps = &suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .unwrap_or_else(|| panic!("no `{scenario}` scenario"))
        .1
        .steps;
    let mut issued: Option<(String, String, String)> = None;
    let mut sent: Vec<Exchange> = Vec::new();
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                match command.to_string().as_str() {
                    "demo.pkce.IssueCode" => {
                        issued = Some((
                            text(input.get("challenge")).unwrap_or_default(),
                            text(input.get("method")).unwrap_or_default(),
                            text(input.get("redirect_uri")).unwrap_or_default(),
                        ));
                    }
                    "demo.pkce.Exchange" => {
                        let (challenge, method, issued_redirect) =
                            issued.clone().unwrap_or_default();
                        sent.push(Exchange {
                            challenge,
                            method,
                            issued_redirect,
                            verifier: text(input.get("verifier")).unwrap_or_default(),
                            redirect: text(input.get("redirect_uri")).unwrap_or_default(),
                            outcome: String::new(),
                        });
                    }
                    _ => {}
                }
            }
            ScenarioStep::ExpectOutcome { outcome } => {
                if let Some(last) = sent.last_mut() {
                    if last.outcome.is_empty()
                        && outcome.to_string().starts_with("demo.pkce.Exchange")
                    {
                        last.outcome = outcome.to_string();
                    }
                }
            }
            _ => {}
        }
    }
    sent
}

fn s256(verifier: &str) -> String {
    base64url(&sha256(verifier.as_bytes()))
}

#[test]
fn the_s256_guard_is_witnessed_both_ways() {
    let model = ir(&MODEL.replace(
        "      - name: redirect-mismatch
        when_subject: {predicate: redirect_uri != input.redirect_uri}
        error: demo.pkce.RedirectMismatch
",
        "",
    ));
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let accepted = exchanges(&synthesis.suite, REDEEMED);
    assert!(
        accepted
            .iter()
            .any(|sent| sent.method == "S256" && sent.challenge == s256(&sent.verifier)),
        "the accepting branch is witnessed on an S256 code issued with the challenge its verifier \
         computes: {accepted:#?}"
    );
    let refused = exchanges(&synthesis.suite, MISMATCH);
    assert!(
        refused
            .iter()
            .any(|sent| sent.method == "S256" && sent.challenge != s256(&sent.verifier)),
        "`mismatch` is witnessed on an S256 code and a verifier computing another challenge: \
         {refused:#?}"
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model.clone()));
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
    let skipping = run(
        &synthesis.suite,
        &SkipsTheCheck(Interpreted::for_model(model)),
    );
    assert!(
        failing(&skipping).contains(&MISMATCH),
        "a target that never refuses the verifier fails `mismatch`: {skipping:#?}"
    );
    for (runner, emitted) in [
        (
            "go",
            ess_conformance::go::emit(&synthesis.suite).map(|files| files.len()),
        ),
        (
            "typescript",
            ess_conformance::ts::emit(&synthesis.suite).map(|files| files.len()),
        ),
    ] {
        assert!(
            emitted.is_ok(),
            "the {runner} runner admits the suite: {emitted:?}"
        );
    }
}

#[test]
fn s256_digest_guard_witnessed_beside_a_held_state_refusal() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let both: Vec<Exchange> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .flat_map(|scenario| exchanges(&synthesis.suite, &scenario))
        .filter(|sent| {
            sent.method == "S256"
                && sent.challenge != s256(&sent.verifier)
                && sent.redirect != sent.issued_redirect
        })
        .collect();
    assert!(
        !both.is_empty()
            && both.iter().all(|sent| sent.outcome.ends_with("mismatch")
                && !sent.outcome.ends_with("redirect-mismatch")),
        "a scenario where both refusals hold requires `mismatch`, declared first: {both:#?}"
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model));
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
}

/// A target that accepts every verifier: the check left out.
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
