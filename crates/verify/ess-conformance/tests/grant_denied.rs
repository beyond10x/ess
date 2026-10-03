//! An ungranted actor gets one declared refusal, witnessed per command (beyond10x/ess#265).
//!
//! Where the model serves a component (`reached_by: network`), synthesis files
//! `<command>/grant/denied` for each command some declared actor lacks the grant for: the command
//! sent as that actor, and the standard refusal required. Where every declared actor holds the
//! grant it files nothing and says so in a note. A model that serves nothing has no surface to
//! answer the refusal, so it gets one note saying enforcement is the caller's. A target that checks
//! no grant runs the command instead, and fails exactly the denied scenarios.

mod support_go;
mod support_versions;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::report::Status;
use ess_conformance::synthesize::Note;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioId, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn example(name: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
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
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{name}: {errors}"));
    compile(&specification, &sources).unwrap_or_else(|errors| panic!("{name}: {errors}"))
}

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The component that serves `DESK`.
const SERVED: &str = "components:
  - component: desk-service
    owns: {domains: [desk.ops]}
    accepts: {commands: [desk.ops.Ping, desk.ops.Tally]}
    publishes: {events: [desk.ops.Pinged, desk.ops.Tallied]}
    reached_by: network
";

/// One actor holding both commands, and one holding one: `Ping` is refused to `Watcher`, `Tally`
/// to nobody. Served over the network.
fn desk() -> String {
    format!("{DESK}{SERVED}")
}

const DESK: &str = "format: ess/15
system: desk
version: v1
domain: desk.ops
events:
  - name: desk.ops.Pinged
    fields:
      - {name: id, type: String}
  - name: desk.ops.Tallied
    fields:
      - {name: id, type: String}
commands:
  - name: desk.ops.Ping
    input:
      - {name: id, type: String}
    outcomes:
      - name: pinged
        emits: [desk.ops.Pinged]
        payload:
          desk.ops.Pinged: {id: input.id}
  - name: desk.ops.Tally
    input:
      - {name: id, type: String}
    outcomes:
      - name: tallied
        emits: [desk.ops.Tallied]
        payload:
          desk.ops.Tallied: {id: input.id}
actors:
  - name: desk.ops.Operator
    may: [desk.ops.Ping, desk.ops.Tally]
  - name: desk.ops.Watcher
    may: [desk.ops.Tally]
";

fn grant_ids(suite: &ConformanceSuite) -> BTreeSet<String> {
    suite
        .scenarios
        .keys()
        .filter(|id| matches!(id, ScenarioId::Grant { .. }))
        .map(ToString::to_string)
        .collect()
}

fn enforced_by_caller(notes: &[Note]) -> usize {
    notes
        .iter()
        .filter(|note| matches!(note, Note::GrantEnforcedByCaller))
        .count()
}

#[test]
fn every_gatepass_command_the_auditor_lacks_gets_a_denied_scenario_sent_as_the_auditor() {
    let synthesis = ess_conformance::synthesize(&example("gatepass"));
    assert_eq!(
        grant_ids(&synthesis.suite),
        [
            "gatepass.visit.AdmitVisitor/grant/denied",
            "gatepass.visit.RegisterVisit/grant/denied",
            "gatepass.visit.SignOutVisitor/grant/denied",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
    );
    let id: ScenarioId = "gatepass.visit.RegisterVisit/grant/denied".parse().unwrap();
    let scenario = synthesis.suite.scenario(&id).expect("filed");
    let refused = scenario
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ExpectNotGranted { .. }))
        .expect("the refusal is required");
    let ScenarioStep::ExecuteCommand { actor, command, .. } = &scenario.steps[refused - 1] else {
        panic!("the refused send comes just before: {:?}", scenario.steps);
    };
    assert_eq!(command.to_string(), "gatepass.visit.RegisterVisit");
    // `Receptionist` holds every grant; `SecurityAuditor` holds none.
    assert_eq!(
        actor.as_ref().map(ToString::to_string).as_deref(),
        Some("gatepass.visit.SecurityAuditor")
    );
    assert!(
        matches!(&scenario.steps[refused], ScenarioStep::ExpectNotGranted { actor, unpublished }
            if actor.to_string() == "gatepass.visit.SecurityAuditor"
                && unpublished.iter().any(|event| event.to_string() == "gatepass.visit.VisitRegistered")),
        "{:?}",
        scenario.steps
    );
    assert!(
        scenario.steps[refused + 1..].iter().all(|step| matches!(
            step,
            ScenarioStep::QueryView { .. }
                | ScenarioStep::ExpectSubjectUnchanged { .. }
                | ScenarioStep::ExpectCompleteSubjectUnchanged { .. }
                | ScenarioStep::ExpectViewUnchanged { .. }
        )),
        "after the refusal, only that nothing changed: {:?}",
        scenario.steps
    );
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    assert_eq!(enforced_by_caller(&synthesis.notes), 0);
}

/// Billing declares actors and serves nothing, so enforcing a grant is the caller's there: no
/// denied scenario, the suite keeps its format, and one note says why.
#[test]
fn a_model_that_serves_nothing_gets_a_coverage_fact_and_no_denied_scenario() {
    let synthesis = ess_conformance::synthesize(&example("billing"));
    assert!(grant_ids(&synthesis.suite).is_empty());
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    assert_eq!(enforced_by_caller(&synthesis.notes), 1);
    let fact = synthesis
        .notes
        .iter()
        .find(|note| matches!(note, Note::GrantEnforcedByCaller))
        .expect("the fact")
        .to_string();
    assert_eq!(
        fact,
        "the specification declares actors and serves no component, so enforcing a grant is the \
         caller's, against the generated grant table, and no `<command>/grant/denied` scenario is \
         owed"
    );
    let unserved = ess_conformance::synthesize(&compiled(DESK));
    assert!(grant_ids(&unserved.suite).is_empty());
    assert_eq!(enforced_by_caller(&unserved.notes), 1);
}

#[test]
fn a_command_every_actor_holds_gets_a_coverage_fact_and_no_scenario() {
    let synthesis = ess_conformance::synthesize(&compiled(&desk()));
    assert_eq!(
        grant_ids(&synthesis.suite),
        ["desk.ops.Ping/grant/denied".to_owned()]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    let facts: Vec<String> = synthesis
        .notes
        .iter()
        .filter(|note| matches!(note, Note::GrantedToEveryActor { .. }))
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        facts,
        [
            "every declared actor may invoke `desk.ops.Tally`, so no actor is refused it and no \
          `desk.ops.Tally/grant/denied` scenario is owed"
        ]
    );
}

#[test]
fn a_model_without_actors_owes_neither() {
    let lone = desk().replace(
        "actors:
  - name: desk.ops.Operator
    may: [desk.ops.Ping, desk.ops.Tally]
  - name: desk.ops.Watcher
    may: [desk.ops.Tally]
",
        "",
    );
    let synthesis = ess_conformance::synthesize(&compiled(&lone));
    assert!(grant_ids(&synthesis.suite).is_empty());
    assert!(!synthesis.notes.iter().any(|note| matches!(
        note,
        Note::GrantedToEveryActor { .. } | Note::GrantEnforcedByCaller
    )));
    assert_ne!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/26"
    );
}

#[test]
fn a_denied_scenario_round_trips_through_the_document_and_an_older_major_is_refused() {
    let suite = ess_conformance::synthesize(&compiled(&desk())).suite;
    let json = suite.to_canonical_json().expect("serializes");
    assert!(json.contains("\"step\": \"expect_not_granted\""), "{json}");
    let admitted = AdmittedSuite::from_json(&json).expect("admits");
    assert_eq!(admitted.suite(), &suite);

    let pinned = support_versions::legacy_json(&json, 24);
    let refused = AdmittedSuite::from_json(&pinned).expect_err("an older major is refused");
    assert!(
        refused.to_string().contains("newer suite major")
            || refused.to_string().contains("suite/26"),
        "{refused}"
    );
}

/// The verdicts of the whole served-`DESK` suite against a target, by scenario.
fn verdicts(target: &Desk) -> Vec<(String, Status)> {
    let suite = ess_conformance::synthesize(&compiled(&desk())).suite;
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    Runner::for_suite(&suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// A target that refuses an ungranted actor passes every scenario; one that checks no grant fails
/// exactly the denied scenario, and nothing else changes.
#[test]
fn a_target_that_checks_no_grant_fails_exactly_the_denied_scenarios() {
    let gated = verdicts(&Desk::gated());
    assert!(
        gated.iter().all(|(_, status)| *status == Status::Passed),
        "{gated:#?}"
    );
    let ungated = verdicts(&Desk::ungated());
    let failed: Vec<&str> = ungated
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(failed, ["desk.ops.Ping/grant/denied"], "{ungated:#?}");
    assert_eq!(
        ungated
            .iter()
            .find(|(id, _)| id == "desk.ops.Ping/grant/denied")
            .map(|(_, status)| *status),
        Some(Status::Failed)
    );
}

/// The Go runtime gives the reference verdict for every scenario — the denied ones included — and
/// sends every request the reference runner sent, against a gated target and an ungated one.
#[test]
fn go_gives_the_reference_verdict_for_every_denied_scenario() {
    let suite = ess_conformance::synthesize(&compiled(&desk())).suite;
    let passed = support_go::assert_parity("grant-reference", &suite, Desk::gated());
    assert!(support_go::not_passed(&passed).is_empty(), "{passed:?}");
    let ungated = support_go::assert_parity("grant-ungated", &suite, Desk::ungated());
    assert_eq!(
        support_go::not_passed(&ungated),
        ["desk.ops.Ping/grant/denied"],
        "{ungated:?}"
    );
}

struct Desk {
    gated: bool,
    sequence: std::cell::Cell<u64>,
}

impl Desk {
    fn gated() -> Self {
        Self {
            gated: true,
            sequence: std::cell::Cell::new(0),
        }
    }

    fn ungated() -> Self {
        Self {
            gated: false,
            sequence: std::cell::Cell::new(0),
        }
    }
}

impl ConformanceTarget for Desk {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("desk-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.to_string();
        if self.gated {
            if let Some(actor) = &request.actor {
                let granted = match actor.to_string().as_str() {
                    "desk.ops.Operator" => true,
                    "desk.ops.Watcher" => command == "desk.ops.Tally",
                    _ => false,
                };
                if !granted {
                    return Err(TargetError::not_granted(Some(actor.to_string())));
                }
            }
        }
        let (branch, event) = match command.as_str() {
            "desk.ops.Ping" => ("pinged", "desk.ops.Pinged"),
            "desk.ops.Tally" => ("tallied", "desk.ops.Tallied"),
            other => {
                return Err(TargetError::unavailable(
                    format!("invoking `{other}`"),
                    "undeclared",
                ))
            }
        };
        self.sequence.set(self.sequence.get() + 1);
        let outcome = ess_compiler::refs::OutcomeRef::new(
            request.command.clone(),
            branch.parse().expect("a branch name"),
        );
        let mut published = ObservedEvent::new(event.parse().expect("an event name"));
        if let Some(id) = request.input.get("id") {
            published = published.with("id", id.clone());
        }
        Ok(SemanticCommandResult::took(outcome)
            .with_consistency(
                ess_primitives::consistency::ConsistencyToken::new(format!(
                    "seq:{}",
                    self.sequence.get()
                ))
                .expect("a token"),
            )
            .emitting(published.in_activity(request.correlation.clone())))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            format!("reading `{}`", request.view),
            "declares no view",
        ))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "forcing an outcome",
            "none is external",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivering", "no binding"))
    }
}
