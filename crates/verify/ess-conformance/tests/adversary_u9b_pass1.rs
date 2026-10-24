//! Adversary pass 1 on "a refused send keeps the earlier read token for a denied grant".
//!
//! The unit's own cases run one denied scenario against the interpreted target, which reads a
//! `read_your_writes` view from its live store whatever token the request carries, so they cannot
//! tell a read made at the earlier write's token from one made at `Current`. These cases pin the
//! token the read after the refusal carries, vary what precedes the refusal (two writes, two
//! refusals, a write that answered no token), and run the shape an adopter reported (a read
//! command, a subscription-like command guarded by state, a list command; external refusals before
//! the row; a system actor reading the view) through all three runners.

mod support_go;
mod support_ts;

use std::cell::RefCell;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::consistency::QueryConsistency;

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The unit's own fixture (`tests/grant_denied.rs`), repeated so these cases stand alone.
const JOBS: &str = "format: ess/27
system: jobs
version: v1
domain: jobs.work
entities:
  - name: jobs.work.Job
    identity: {name: id, type: String}
    fields:
      - {name: title, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
errors:
  - {name: jobs.work.JobNotFound, summary: No job carries the identity., fields: []}
events:
  - name: jobs.work.JobOpened
    fields:
      - {name: id, type: String}
commands:
  - name: jobs.work.OpenJob
    input:
      - {name: id, type: String}
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: jobs.work.Job
        instance: id
        sets: {title: input.title}
        emits: [jobs.work.JobOpened]
        payload:
          jobs.work.JobOpened: {id: input.id}
  - name: jobs.work.GetJob
    input:
      - {name: id, type: String}
    outcomes:
      - {name: not-found, unknown_instance: true, error: jobs.work.JobNotFound}
      - name: found
        preserves: jobs.work.Job
        instance: id
views:
  - name: jobs.work.JobById
    source: jobs.work.Job
    consistency: read_your_writes
    fields:
      - {name: id, type: String}
      - {name: title, type: String}
      - {name: state, type: jobs.work.Job.State}
actors:
  - name: jobs.work.Dispatcher
    may: [jobs.work.OpenJob, jobs.work.GetJob, jobs.work.JobById]
  - name: jobs.work.Watcher
    may: [jobs.work.OpenJob, jobs.work.JobById]
components:
  - component: jobs-service
    owns: {domains: [jobs.work]}
    accepts: {commands: [jobs.work.OpenJob, jobs.work.GetJob]}
    publishes: {events: [jobs.work.JobOpened]}
    reached_by: network
";

const GET_JOB_DENIED: &str = "jobs.work.GetJob/grant/denied";

/// The adopter's shape, brand-free: a read command answering a response, a watch command refused
/// on a terminal state (the subscription), and a list command with an optional input, each behind
/// an external refusal before the row; a system actor alone reads the identity view and is the
/// actor the reads are not granted to.
const DOCS: &str = "format: ess/29
system: docs
version: v1
domain: docs.store
types:
  - name: docs.store.DocId
    kind: newtype
    of: String
entities:
  - name: docs.store.Document
    identity: {name: id, type: docs.store.DocId}
    fields:
      - {name: title, type: String}
      - {name: note, type: Optional<String>}
    relations:
      - {name: shares, kind: owns, target: docs.store.Share, cardinality: many, via: doc_id}
    lifecycle:
      initial: Draft
      states: [Draft, Archived]
      terminal: [Archived]
      transitions:
        - {name: archive, from: [Draft], to: Archived}
  - name: docs.store.Share
    identity: {name: row, type: String}
    fields:
      - {name: share_id, type: String}
      - {name: doc_id, type: docs.store.DocId}
      - {name: url, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
errors:
  - {name: docs.store.DocNotFound, summary: No document carries the identity., fields: []}
  - {name: docs.store.Unsupported, summary: The store does not support it., fields: []}
  - {name: docs.store.VersionNotSupported, summary: The version is not supported., fields: []}
events:
  - name: docs.store.DocumentCreated
    fields:
      - {name: id, type: docs.store.DocId}
  - name: docs.store.DocumentArchived
    fields:
      - {name: id, type: docs.store.DocId}
  - name: docs.store.ShareCreated
    fields:
      - {name: row, type: String}
      - {name: share_id, type: String}
      - {name: doc_id, type: docs.store.DocId}
commands:
  - name: docs.store.CreateDocument
    input:
      - {name: id, type: docs.store.DocId}
      - {name: title, type: String}
    outcomes:
      - name: created
        creates: docs.store.Document
        instance: id
        sets: {title: input.title}
        emits: [docs.store.DocumentCreated]
        payload:
          docs.store.DocumentCreated: {id: input.id}
  - name: docs.store.ArchiveDocument
    input:
      - {name: id, type: docs.store.DocId}
    outcomes:
      - {name: not-found, unknown_instance: true, error: docs.store.DocNotFound}
      - name: archived
        moves: docs.store.Document.archive
        instance: id
        emits: [docs.store.DocumentArchived]
        payload:
          docs.store.DocumentArchived: {id: input.id}
      - {name: already, wrong_state: true, error: docs.store.Unsupported}
  - name: docs.store.CreateShare
    input:
      - {name: share_id, type: String}
      - {name: doc_id, type: docs.store.DocId}
      - {name: url, type: String}
    outcomes:
      - name: doc-not-found
        when_related:
          via: input.doc_id
          exists: false
        error: docs.store.DocNotFound
      - name: created
        creates: docs.store.Share
        instance: row
        sets: {share_id: input.share_id, doc_id: input.doc_id, url: input.url}
        emits: [docs.store.ShareCreated]
        payload:
          docs.store.ShareCreated: {row: {generated: true}, share_id: input.share_id, doc_id: input.doc_id}
  - name: docs.store.GetDocument
    input:
      - {name: tenant, type: Optional<String>, presence: omitted_when_absent}
      - {name: id, type: docs.store.DocId}
    response:
      - {name: title, type: String}
    outcomes:
      - name: version-not-supported
        external: the version parameter names a version the store does not support
        before_row: true
        error: docs.store.VersionNotSupported
      - {name: not-found, unknown_instance: true, error: docs.store.DocNotFound}
      - name: found
        preserves: docs.store.Document
        instance: id
        returns: true
  - name: docs.store.WatchDocument
    input:
      - {name: id, type: docs.store.DocId}
    response:
      - {name: title, type: String}
    outcomes:
      - name: version-not-supported
        external: the version parameter names a version the store does not support
        before_row: true
        error: docs.store.VersionNotSupported
      - {name: not-found, unknown_instance: true, error: docs.store.DocNotFound}
      - name: archived
        when_subject:
          predicate:
            state: [Archived]
        error: docs.store.Unsupported
      - name: watching
        preserves: docs.store.Document
        instance: id
        returns: true
  - name: docs.store.ListShares
    input:
      - {name: doc_id, type: docs.store.DocId}
      - {name: page_size, type: Optional<Integer>, presence: omitted_when_absent}
    outcomes:
      - {name: not-found, unknown_instance: true, error: docs.store.DocNotFound}
      - name: listed
        preserves: docs.store.Document
        instance: doc_id
views:
  - name: docs.store.AllDocuments
    source: docs.store.Document
    consistency: read_your_writes
    fields:
      - {name: id, type: docs.store.DocId}
      - {name: title, type: String}
      - {name: note, type: Optional<String>}
      - {name: state, type: docs.store.Document.State}
  - name: docs.store.AllShares
    source: docs.store.Share
    consistency: read_your_writes
    fields:
      - {name: row, type: String}
      - {name: share_id, type: String}
      - {name: doc_id, type: docs.store.DocId}
      - {name: url, type: String}
      - {name: state, type: docs.store.Share.State}
actors:
  - name: docs.store.Reader
    may: [docs.store.CreateDocument, docs.store.CreateShare, docs.store.GetDocument, docs.store.WatchDocument, docs.store.ListShares]
  - name: docs.store.Keeper
    system: true
    may: [docs.store.ArchiveDocument, docs.store.AllDocuments, docs.store.AllShares]
components:
  - component: docs-service
    owns: {domains: [docs.store]}
    accepts: {commands: [docs.store.CreateDocument, docs.store.ArchiveDocument, docs.store.CreateShare, docs.store.GetDocument, docs.store.WatchDocument, docs.store.ListShares]}
    publishes: {events: [docs.store.DocumentCreated, docs.store.DocumentArchived, docs.store.ShareCreated]}
    reached_by: network
";

/// The interpreted target over `text`, logging every command's token and every view read's
/// consistency, and answering no token for the commands named in `tokenless`.
struct Watched {
    inner: Interpreted,
    tokenless: Vec<&'static str>,
    log: RefCell<Vec<String>>,
}

impl Watched {
    fn new(text: &str, tokenless: &[&'static str]) -> Self {
        Self {
            inner: Interpreted::for_model(compiled(text)),
            tokenless: tokenless.to_vec(),
            log: RefCell::default(),
        }
    }

    fn read(&self, request: &SemanticViewRequest) {
        let at = match &request.consistency {
            QueryConsistency::Current => "current".to_owned(),
            QueryConsistency::AtLeast { token } => format!("at {token}"),
        };
        self.log
            .borrow_mut()
            .push(format!("read {} {at}", request.view));
    }
}

impl ConformanceTarget for Watched {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("watched", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.log.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.to_string();
        let mut answer = self.inner.execute_command(request);
        if let Ok(result) = &mut answer {
            if self.tokenless.contains(&command.as_str()) {
                result.consistency = None;
            }
        }
        let line = match &answer {
            Ok(result) => match &result.consistency {
                Some(token) => format!("send {command} -> {token}"),
                None => format!("send {command} -> no token"),
            },
            Err(TargetError::NotGranted { .. }) => format!("send {command} -> not granted"),
            Err(other) => format!("send {command} -> {other}"),
        };
        self.log.borrow_mut().push(line);
        answer
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.read(&request);
        self.inner.query_view(request)
    }
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ess_conformance::scenario::ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        self.read(&request);
        self.inner.query_view_as(request, reader)
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
}

/// The scenarios of `text` whose id ends in `/grant/denied`, synthesized.
fn denied(text: &str) -> ConformanceSuite {
    let mut suite = ess_conformance::synthesize(&compiled(text)).suite;
    suite
        .scenarios
        .retain(|id, _| id.to_string().ends_with("/grant/denied"));
    assert!(!suite.scenarios.is_empty());
    suite
}

/// `suite` with the steps of its one scenario rewritten by `edit`.
fn edited(
    mut suite: ConformanceSuite,
    edit: impl FnOnce(&mut Vec<ScenarioStep>),
) -> ConformanceSuite {
    assert_eq!(suite.scenarios.len(), 1);
    let scenario = suite.scenarios.values_mut().next().unwrap();
    edit(&mut scenario.steps);
    suite
}

/// The denied `GetJob` scenario alone.
fn denied_get_job() -> ConformanceSuite {
    let mut suite = denied(JOBS);
    suite
        .scenarios
        .retain(|id, _| id.to_string() == GET_JOB_DENIED);
    suite
}

fn position(steps: &[ScenarioStep], wanted: impl Fn(&ScenarioStep) -> bool) -> usize {
    steps
        .iter()
        .position(wanted)
        .unwrap_or_else(|| panic!("no such step: {steps:#?}"))
}

/// The refused send and the `expect_not_granted` after it.
fn refusal(steps: &[ScenarioStep]) -> (usize, Vec<ScenarioStep>) {
    let at = position(steps, |step| {
        matches!(step, ScenarioStep::ExpectNotGranted { .. })
    });
    (at - 1, steps[at - 1..=at].to_vec())
}

/// A second `OpenJob`, of a job named `other`, with its outcome, inserted before the refused send.
fn with_second_write(steps: &mut Vec<ScenarioStep>) {
    let open = steps[0].clone();
    let outcome = steps[1].clone();
    let mut open = serde_json::to_value(open).unwrap();
    open["input"]["id"]["value"] = "other".into();
    let open: ScenarioStep = serde_json::from_value(open).unwrap();
    let (send, _) = refusal(steps);
    steps.splice(send..send, [open, outcome]);
}

fn run(suite: &ConformanceSuite, target: &impl ConformanceTarget) -> Vec<ScenarioResult> {
    let admitted = AdmittedSuite::from_suite(suite).expect("admits");
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
}

/// The token the last `send <command>` line answered.
fn token_of(log: &[String], command: &str) -> String {
    log.iter()
        .rev()
        .find_map(|line| line.strip_prefix(&format!("send {command} -> ")))
        .unwrap_or_else(|| panic!("{command} was sent: {log:#?}"))
        .to_owned()
}

/// The read after the refusal carries the token the write before it returned, not `Current`: the
/// interpreted target answers both alike, so only the request says which one the runner made.
#[test]
fn the_read_after_the_refusal_carries_the_write_token() {
    let suite = denied_get_job();
    let target = Watched::new(JOBS, &[]);
    let results = run(&suite, &target);
    assert_eq!(results[0].status, Status::Passed, "{:#?}", results[0]);
    let log = target.log.borrow().clone();
    let written = token_of(&log, "jobs.work.OpenJob");
    assert_eq!(
        log.last().map(String::as_str),
        Some(format!("read jobs.work.JobById at {written}").as_str()),
        "{log:#?}"
    );
}

/// Two writes, then the refusal: the read is at the second write's token, not the first's.
#[test]
fn the_read_after_the_refusal_is_at_the_last_write_not_the_first() {
    let suite = edited(denied_get_job(), with_second_write);
    let target = Watched::new(JOBS, &[]);
    let results = run(&suite, &target);
    assert_eq!(results[0].status, Status::Passed, "{:#?}", results[0]);
    let log = target.log.borrow().clone();
    let tokens: Vec<&String> = log
        .iter()
        .filter(|line| line.starts_with("send jobs.work.OpenJob -> "))
        .collect();
    assert_eq!(tokens.len(), 2, "{log:#?}");
    let last = token_of(&log, "jobs.work.OpenJob");
    assert_eq!(
        log.last().map(String::as_str),
        Some(format!("read jobs.work.JobById at {last}").as_str()),
        "{log:#?}"
    );
}

/// The same refused send twice in a row: the second refusal leaves the token as the first did.
#[test]
fn two_refusals_in_a_row_keep_the_write_token_in_every_runner() {
    let suite = edited(denied_get_job(), |steps| {
        let (send, pair) = refusal(steps);
        steps.splice(send..send, pair);
    });
    let target = Watched::new(JOBS, &[]);
    let results = run(&suite, &target);
    assert_eq!(results[0].status, Status::Passed, "{:#?}", results[0]);
    let log = target.log.borrow().clone();
    let written = token_of(&log, "jobs.work.OpenJob");
    assert_eq!(
        log.last().map(String::as_str),
        Some(format!("read jobs.work.JobById at {written}").as_str()),
        "{log:#?}"
    );
    let go = support_go::assert_parity("u9b-twice", &suite, Watched::new(JOBS, &[]));
    assert_eq!(go[GET_JOB_DENIED], "passed", "{go:?}");
    ts_parity("u9b-twice", &suite, Watched::new(JOBS, &[]), "passed");
}

/// A write before the refusal that answered no token: the read after the refusal is not made, and
/// the scenario fails (the unit's CHANGELOG entry: "A command that ran and returned no token still
/// leaves the read unmade and fails it, as before"), naming that write, not the refused send, as
/// the one that owed the token. It must not error with the ESS-CF-SUITE line the story removes.
#[test]
fn a_tokenless_write_before_the_refusal_fails_the_denied_snapshot() {
    let suite = edited(denied_get_job(), with_second_write);
    // Only the second OpenJob, sent after the snapshot, answers without a token.
    let target = SecondTokenless::new();
    let results = run(&suite, &target);
    let result = &results[0];
    let text = format!("{result:?}");
    assert!(
        !text.contains("no consistent view query preceded the snapshot assertion"),
        "the suite is blamed for the target's missing token: {result:#?}"
    );
    assert_eq!(result.status, Status::Failed, "{result:#?}");
    assert!(
        !text.contains("`jobs.work.GetJob` returned no consistency token"),
        "the refused send is blamed for the missing token: {result:#?}"
    );
    assert!(
        text.contains("`jobs.work.OpenJob` returned no consistency token"),
        "the write that owed the token is named: {result:#?}"
    );
}

/// The same tokenless write before the refusal reads alike in the Go and TypeScript runners.
#[test]
fn a_tokenless_write_before_the_refusal_reads_alike_in_every_runner() {
    let suite = edited(denied_get_job(), with_second_write);
    let reference = support_go::rust_outcomes(&suite, &SecondTokenless::new());
    let go = support_go::assert_parity("u9b-tokenless", &suite, SecondTokenless::new());
    assert_eq!(go, reference);
    ts_parity(
        "u9b-tokenless",
        &suite,
        SecondTokenless::new(),
        &reference[GET_JOB_DENIED].clone(),
    );
}

/// Every denied scenario of the adopter's shape passes against the interpreted target in the Rust,
/// Go and TypeScript runners, and each read after a refusal carries the last write's token.
#[test]
fn the_adopter_shape_passes_every_denied_scenario_in_every_runner() {
    let suite = denied(DOCS);
    let ids: Vec<String> = suite.scenarios.keys().map(ToString::to_string).collect();
    for wanted in [
        "docs.store.GetDocument/grant/denied",
        "docs.store.WatchDocument/grant/denied",
        "docs.store.ListShares/grant/denied",
    ] {
        assert!(ids.iter().any(|id| id == wanted), "{wanted} in {ids:#?}");
    }
    let target = Watched::new(DOCS, &[]);
    for (id, scenario) in &suite.scenarios {
        let mut one = suite.clone();
        one.scenarios.retain(|other, _| other == id);
        let results = run(&one, &target);
        assert_eq!(results[0].status, Status::Passed, "{id}: {:#?}", results[0]);
        let refused = scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectNotGranted { .. }));
        let reads_after = scenario
            .steps
            .iter()
            .rev()
            .take_while(|step| !matches!(step, ScenarioStep::ExpectNotGranted { .. }))
            .any(|step| matches!(step, ScenarioStep::QueryView { .. }));
        if refused && reads_after {
            let log = target.log.borrow().clone();
            let last_read = log
                .iter()
                .rev()
                .find(|line| line.starts_with("read "))
                .unwrap();
            let last_token = log
                .iter()
                .rev()
                .filter_map(|line| line.strip_prefix("send "))
                .find(|line| !line.ends_with("-> not granted"))
                .and_then(|line| line.split(" -> ").nth(1))
                .unwrap_or_else(|| panic!("{id}: a write ran: {log:#?}"));
            assert!(
                last_read.ends_with(&format!(" at {last_token}")),
                "{id}: the read after the refusal is at the last write's token: {log:#?}"
            );
        }
    }
    let go = support_go::assert_parity("u9b-docs", &suite, Watched::new(DOCS, &[]));
    assert!(support_go::not_passed(&go).is_empty(), "{go:?}");
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let recorder = support_go::Recorder::new(Watched::new(DOCS, &[]));
    let reference = support_go::rust_outcomes_admitted(&admitted, &recorder);
    assert!(
        support_go::not_passed(&reference).is_empty(),
        "{reference:?}"
    );
    let package = support_ts::Package::new("u9b-docs", &suite);
    let replayed = package.replay(&admitted, &recorder.transcript(), false);
    assert!(
        replayed.divergences.is_empty(),
        "{}\n{}",
        replayed.divergences.join("\n"),
        replayed.log
    );
    assert_eq!(replayed.outcomes, reference, "{}", replayed.log);
}

/// The TypeScript runtime replays `target`'s transcript to the reference verdict, sending every
/// request the reference runner sent.
fn ts_parity<T: ConformanceTarget>(
    label: &str,
    suite: &ConformanceSuite,
    target: T,
    verdict: &str,
) {
    let admitted = AdmittedSuite::from_suite(suite).expect("admits");
    let recorder = support_go::Recorder::new(target);
    let reference = support_go::rust_outcomes_admitted(&admitted, &recorder);
    assert_eq!(reference[GET_JOB_DENIED], verdict, "{reference:?}");
    let package = support_ts::Package::new(label, suite);
    let replayed = package.replay(&admitted, &recorder.transcript(), false);
    assert!(
        replayed.divergences.is_empty(),
        "{}\n{}",
        replayed.divergences.join("\n"),
        replayed.log
    );
    assert_eq!(replayed.outcomes, reference, "{}", replayed.log);
}

/// The interpreted target over `JOBS`, whose second `OpenJob` of a scenario answers no token.
struct SecondTokenless {
    inner: Watched,
    opened: std::cell::Cell<u32>,
}

impl SecondTokenless {
    fn new() -> Self {
        Self {
            inner: Watched::new(JOBS, &[]),
            opened: std::cell::Cell::new(0),
        }
    }
}

impl ConformanceTarget for SecondTokenless {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.opened.set(0);
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let open = request.command.to_string() == "jobs.work.OpenJob";
        let mut answer = self.inner.execute_command(request);
        if open {
            self.opened.set(self.opened.get() + 1);
            if self.opened.get() == 2 {
                if let Ok(result) = &mut answer {
                    result.consistency = None;
                }
            }
        }
        answer
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ess_conformance::scenario::ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view_as(request, reader)
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
}
