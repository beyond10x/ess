//! Adversary pass 1 on an `Optional` input a system-issued command awaits, compared only when the
//! target issued it.
//!
//! The unit's own cases use a model whose event carries no value read from the awaited input, so
//! the event half of the rule (`resolve_payload`) never runs there. These cases add a model whose
//! event relays the `Optional` input, and targets that misreport around a left-out input.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::scenario::{ScenarioStep, ScenarioValue};
use ess_conformance::target::{
    AwaitAnswer, AwaitRequest, ConformanceTarget, ExternalOutcomeControl, ScenarioContext,
    SemanticCommandRequest, TargetError,
};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const FINISH: &str = "jobs.job.Finish";
const FINISHED: &str = "jobs.job.Finished";
const MOVED: &str = "jobs.job.Finish/outcome/moved";

/// The unit's job model, with `Finished` relaying the `Optional` note as well as `sets:` storing
/// it.
fn job_model() -> String {
    let mut out = String::new();
    writeln!(out, "format: ess/27").unwrap();
    writeln!(out, "system: jobs").unwrap();
    writeln!(out, "version: v1").unwrap();
    writeln!(out, "domain: jobs.job").unwrap();
    writeln!(out, "types:").unwrap();
    writeln!(out, "  - {{name: jobs.job.JobId, kind: newtype, of: Uuid}}").unwrap();
    writeln!(out, "  - name: jobs.job.Note").unwrap();
    writeln!(out, "    kind: struct").unwrap();
    writeln!(out, "    fields:").unwrap();
    writeln!(out, "      - {{name: text, type: String}}").unwrap();
    writeln!(out, "entities:").unwrap();
    writeln!(out, "  - name: jobs.job.Job").unwrap();
    writeln!(out, "    identity: {{name: job_id, type: jobs.job.JobId}}").unwrap();
    writeln!(out, "    fields:").unwrap();
    writeln!(out, "      - {{name: note, type: Optional<jobs.job.Note>}}").unwrap();
    writeln!(out, "    lifecycle:").unwrap();
    writeln!(out, "      initial: Open").unwrap();
    writeln!(out, "      states: [Open, Done]").unwrap();
    writeln!(out, "      terminal: [Done]").unwrap();
    writeln!(out, "      transitions:").unwrap();
    writeln!(out, "        - {{name: finish, from: [Open], to: Done}}").unwrap();
    writeln!(out, "errors:").unwrap();
    writeln!(
        out,
        "  - {{name: jobs.job.JobStateConflict, summary: The job is not open., fields: []}}"
    )
    .unwrap();
    writeln!(out, "events:").unwrap();
    writeln!(out, "  - name: jobs.job.Opened").unwrap();
    writeln!(out, "    fields:").unwrap();
    writeln!(out, "      - {{name: job_id, type: jobs.job.JobId}}").unwrap();
    writeln!(out, "  - name: jobs.job.Finished").unwrap();
    writeln!(out, "    fields:").unwrap();
    writeln!(out, "      - {{name: job_id, type: jobs.job.JobId}}").unwrap();
    writeln!(out, "      - {{name: note, type: Optional<jobs.job.Note>}}").unwrap();
    writeln!(out, "actors:").unwrap();
    writeln!(out, "  - name: jobs.job.Caller").unwrap();
    writeln!(out, "    may: [jobs.job.Open]").unwrap();
    writeln!(out, "  - name: jobs.job.Agent").unwrap();
    writeln!(out, "    system: true").unwrap();
    writeln!(out, "    may: [jobs.job.Finish]").unwrap();
    writeln!(out, "views:").unwrap();
    writeln!(out, "  - name: jobs.job.Jobs").unwrap();
    writeln!(out, "    source: jobs.job.Job").unwrap();
    writeln!(out, "    consistency: read_your_writes").unwrap();
    writeln!(out, "    fields:").unwrap();
    writeln!(out, "      - {{name: job_id, type: jobs.job.JobId}}").unwrap();
    writeln!(out, "      - {{name: note, type: Optional<jobs.job.Note>}}").unwrap();
    writeln!(out, "      - {{name: state, type: jobs.job.Job.State}}").unwrap();
    writeln!(out, "components:").unwrap();
    writeln!(out, "  - component: job-service").unwrap();
    writeln!(out, "    owns: {{domains: [jobs.job]}}").unwrap();
    writeln!(out, "    accepts:").unwrap();
    writeln!(out, "      commands: [jobs.job.Open, jobs.job.Finish]").unwrap();
    writeln!(out, "    publishes:").unwrap();
    writeln!(out, "      events: [jobs.job.Opened, jobs.job.Finished]").unwrap();
    writeln!(out, "    reached_by: network").unwrap();
    writeln!(out, "commands:").unwrap();
    writeln!(out, "  - name: jobs.job.Open").unwrap();
    writeln!(out, "    input: []").unwrap();
    writeln!(out, "    outcomes:").unwrap();
    writeln!(out, "      - name: opened").unwrap();
    writeln!(out, "        creates: jobs.job.Job").unwrap();
    writeln!(out, "        instance: job_id").unwrap();
    writeln!(out, "        emits: [jobs.job.Opened]").unwrap();
    writeln!(out, "        payload:").unwrap();
    writeln!(
        out,
        "          jobs.job.Opened: {{job_id: {{generated: true}}}}"
    )
    .unwrap();
    writeln!(out, "  - name: jobs.job.Finish").unwrap();
    writeln!(out, "    await_within: PT5S").unwrap();
    writeln!(out, "    input:").unwrap();
    writeln!(out, "      - {{name: job_id, type: jobs.job.JobId}}").unwrap();
    writeln!(out, "      - {{name: note, type: Optional<jobs.job.Note>}}").unwrap();
    writeln!(out, "    outcomes:").unwrap();
    writeln!(out, "      - name: moved").unwrap();
    writeln!(out, "        moves: jobs.job.Job.finish").unwrap();
    writeln!(out, "        instance: job_id").unwrap();
    writeln!(out, "        emits: [jobs.job.Finished]").unwrap();
    writeln!(out, "        sets: {{note: input.note}}").unwrap();
    writeln!(out, "        payload:").unwrap();
    writeln!(
        out,
        "          jobs.job.Finished: {{job_id: input.job_id, note: input.note}}"
    )
    .unwrap();
    writeln!(
        out,
        "      - {{name: wrong-state, wrong_state: true, error: jobs.job.JobStateConflict}}"
    )
    .unwrap();
    out
}

fn job_ir() -> EssIr {
    let raw = RawSpecFile::parse(&job_model()).expect("the model parses");
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn job_suite() -> AdmittedSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&job_ir());
    let admitted = AdmittedSuite::from_suite(&synthesis.suite)
        .unwrap_or_else(|error| panic!("the synthesized suite is admitted: {error}"));
    assert!(
        admitted.suite().scenario(&MOVED.parse().unwrap()).is_some(),
        "`{MOVED}` is synthesized"
    );
    admitted
}

fn run<T: ConformanceTarget>(target: &T) -> BTreeMap<String, ScenarioResult> {
    let suite = job_suite();
    Runner::for_suite(suite.suite())
        .run_admitted(&suite, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect()
}

fn note(text: &str) -> Node {
    Node::Map(BTreeMap::from([(
        "text".to_owned(),
        Node::Text(text.to_owned()),
    )]))
}

/// How the wrapped interpreter misreports.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lie {
    /// Leaves `note` out of the issued command and publishes `Finished` carrying a note anyway.
    EventNoteNotIssued,
    /// Leaves `note` out of the issued command and keeps a note in the row it reads back.
    RowNoteKept,
    /// Reports the left-out `note` as an explicit `null` input; publishes and stores as the model.
    NullSpelledInput,
}

struct Lying {
    inner: Interpreted,
    lie: Lie,
    told: Cell<usize>,
}

impl Lying {
    fn new(lie: Lie) -> Self {
        Self {
            inner: Interpreted::for_model(job_ir()),
            lie,
            told: Cell::new(0),
        }
    }

    fn rows(
        &self,
        mut result: ess_conformance::target::SemanticViewResult,
    ) -> ess_conformance::target::SemanticViewResult {
        if self.lie == Lie::RowNoteKept {
            for row in &mut result.rows {
                if row.get("state") == Some(&Node::Text("Done".to_owned())) {
                    row.insert("note".to_owned(), note("kept from before"));
                    self.told.set(self.told.get() + 1);
                }
            }
        }
        result
    }
}

impl ConformanceTarget for Lying {
    fn establish_entity(
        &self,
        request: ess_conformance::target::EntitySetupRequest,
    ) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ess_conformance::target::ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn deliver_event(
        &self,
        request: ess_conformance::target::EventDeliveryRequest,
    ) -> Result<(), TargetError> {
        self.inner.deliver_event(request)
    }
    fn mark_instant(
        &self,
        request: ess_conformance::target::InstantMark,
    ) -> Result<(), TargetError> {
        self.inner.mark_instant(request)
    }
    fn observe_elapsed(
        &self,
        request: ess_conformance::target::ElapsedObservationRequest,
    ) -> Result<ess_conformance::target::ElapsedObservation, TargetError> {
        self.inner.observe_elapsed(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<ess_conformance::target::SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> ess_conformance::target::RecordedCommandCompletion {
        self.inner.execute_command_recorded(request)
    }
    fn execute_command_without_input(
        &self,
        request: ess_conformance::target::AbsentInputRequest,
    ) -> Result<ess_conformance::target::SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(
        &self,
        request: ess_conformance::target::SemanticViewRequest,
    ) -> Result<ess_conformance::target::SemanticViewResult, TargetError> {
        self.inner
            .query_view(request)
            .map(|result| self.rows(result))
    }
    fn query_view_as(
        &self,
        request: ess_conformance::target::SemanticViewRequest,
        reader: &ess_conformance::scenario::ActorRef,
    ) -> Result<ess_conformance::target::SemanticViewResult, TargetError> {
        self.inner
            .query_view_as(request, reader)
            .map(|result| self.rows(result))
    }
    fn query_view_anonymous(
        &self,
        request: ess_conformance::target::SemanticViewRequest,
    ) -> Result<ess_conformance::target::SemanticViewResult, TargetError> {
        self.inner
            .query_view_anonymous(request)
            .map(|result| self.rows(result))
    }
    fn observe_events(
        &self,
        request: ess_conformance::target::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::target::ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        self.inner
            .configure_external_outcome_repeatedly(request, times)
    }
    fn redeliver_event(
        &self,
        request: ess_conformance::target::RedeliveryRequest,
    ) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        request: ess_conformance::target::InvocationObservationRequest,
    ) -> Result<Vec<ess_conformance::target::ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
    fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
        let is_finish = request.command.to_string() == FINISH;
        let mut answer = self.inner.await_outcome(request)?;
        if let AwaitAnswer::Reached { input, result, .. } = &mut answer {
            if is_finish && !input.contains_key("note") {
                match self.lie {
                    Lie::EventNoteNotIssued => {
                        for event in &mut result.direct_events {
                            if event.event.to_string() == FINISHED {
                                event
                                    .payload
                                    .insert("note".to_owned(), note("never issued"));
                                self.told.set(self.told.get() + 1);
                            }
                        }
                    }
                    Lie::NullSpelledInput => {
                        input.insert("note".to_owned(), Node::Null);
                        self.told.set(self.told.get() + 1);
                    }
                    Lie::RowNoteKept => {}
                }
            }
        }
        Ok(answer)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

/// The event half of the rule is reached at all: `moved` expects `Finished.note` read from the
/// awaited input. The unit's own model relays nothing into an event, so no unit case runs
/// `resolve_payload`'s filter; dropping it there stays green.
#[test]
fn the_relaying_model_reads_the_awaited_note_in_an_event() {
    let suite = job_suite();
    let scenario = suite.suite().scenario(&MOVED.parse().unwrap()).unwrap();
    let reads = scenario.steps.iter().any(|step| match step {
        ScenarioStep::ExpectEventValues { payload, .. } => payload
            .values()
            .any(|value| matches!(value, ScenarioValue::Awaited { field } if field == "note")),
        _ => false,
    });
    assert!(reads, "{:#?}", scenario.steps);
}

/// The interpreter leaves `note` out, stores nothing and publishes no note: every scenario passes.
/// Red against a runner that resolves the left-out value in the event (`resolve_payload` without
/// its filter): it errors with "the awaited answer carried no `note`".
#[test]
fn a_left_out_note_relayed_into_an_event_passes_on_the_interpreter() {
    let results = run(&Interpreted::for_model(job_ir()));
    for (id, result) in &results {
        assert_eq!(result.status, Status::Passed, "{id}: {result:#?}");
    }
}

/// The acceptance: "the row's `note` is checked against the absent-input semantics of `sets:`"
/// (the interpreter clears the field). A target that keeps a note in the row fails `moved`.
#[test]
fn a_left_out_note_with_a_note_kept_in_the_row_fails() {
    let target = Lying::new(Lie::RowNoteKept);
    let results = run(&target);
    assert!(target.told.get() > 0, "the row lie was never told");
    let moved = &results[MOVED];
    assert_eq!(moved.status, Status::Failed, "{moved:#?}");
}

/// The specification says `Finished.note` is `input.note`. A target that issued `Finish` without
/// a note and published `Finished` with one broke that, and must fail `moved`: the row half of the
/// rule asserts the absence, the event half compares nothing.
#[test]
fn a_left_out_note_published_in_the_event_anyway_fails() {
    let target = Lying::new(Lie::EventNoteNotIssued);
    let results = run(&target);
    assert!(target.told.get() > 0, "the event lie was never told");
    let moved = &results[MOVED];
    assert_eq!(
        moved.status,
        Status::Failed,
        "a note the system never issued was published in `{FINISHED}` and accepted: {moved:#?}"
    );
}

/// A target that spells the left-out `Optional` input as `null` in its answer (a JSON target's
/// natural spelling) and otherwise behaves as the model: the same behaviour as leaving it out, so
/// `moved` passes.
#[test]
fn a_left_out_note_spelled_null_in_the_answer_passes() {
    let target = Lying::new(Lie::NullSpelledInput);
    let results = run(&target);
    assert!(target.told.get() > 0, "the null spelling was never used");
    let moved = &results[MOVED];
    assert_eq!(moved.status, Status::Passed, "{moved:#?}");
}
