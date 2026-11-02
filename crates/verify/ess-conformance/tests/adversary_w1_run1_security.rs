//! Security and invariant pass for `story:authored-success-step-asserts-no-error` (wave W1, RUN1).
//!
//! Each case checks one invariant the unit claims, by calling what it ships:
//!
//! - an authored success act fails a target answering the success branch with an error, on every
//!   lowering path that reaches `expected_outcome` (sent, awaited, external);
//! - a refusing act is unchanged;
//! - the library contract for a hand-built authored suite holds as the CHANGELOG states it.

use ess_compiler::ir::EssIr;
use ess_compiler::refs::ErrorRef;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ConformanceReport, Status};
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, SuiteProvenance};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;
use std::path::{Path, PathBuf};

const STORY: &str = "story:authored-success-step-asserts-no-error";

// ---- models ------------------------------------------------------------------------------------

fn fixture(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let specification = Specification::assemble([(SpecSource::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

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
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((SpecSource::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("validates");
    compile(&specification, &sources).expect("resolves")
}

/// A queue whose `Assign` the system issues itself (a brand-free copy of the fixture in
/// `tests/system_issued_authored.rs`).
const QUEUEING: &str = "format: ess/27
system: queueing
version: v1
domain: queueing.entry
types:
  - {name: queueing.entry.EntryId, kind: newtype, of: Uuid}
  - {name: queueing.entry.WorkerId, kind: newtype, of: Uuid}
entities:
  - name: queueing.entry.Worker
    identity: {name: worker_id, type: queueing.entry.WorkerId}
    fields:
      - {name: free, type: Boolean}
    lifecycle: {initial: Ready, states: [Ready], terminal: [Ready]}
  - name: queueing.entry.Entry
    identity: {name: entry_id, type: queueing.entry.EntryId}
    fields:
      - {name: worker, type: Optional<queueing.entry.WorkerId>}
    lifecycle:
      initial: Waiting
      states: [Waiting, Assigned]
      terminal: [Assigned]
      transitions:
        - {name: assign, from: [Waiting], to: Assigned}
errors:
  - {name: queueing.entry.EntryStateConflict, summary: The entry is not waiting., fields: []}
  - {name: queueing.entry.DiallerRefused, summary: The dialler refused the call., fields: []}
events:
  - name: queueing.entry.WorkerAdded
    fields:
      - {name: worker_id, type: queueing.entry.WorkerId}
  - name: queueing.entry.Enqueued
    fields:
      - {name: entry_id, type: queueing.entry.EntryId}
  - name: queueing.entry.Assigned
    fields:
      - {name: entry_id, type: queueing.entry.EntryId}
      - {name: worker, type: queueing.entry.WorkerId}
actors:
  - name: queueing.entry.Caller
    may: [queueing.entry.AddWorker, queueing.entry.Enqueue]
  - name: queueing.entry.Scheduler
    system: true
    may: [queueing.entry.Assign]
commands:
  - name: queueing.entry.AddWorker
    input:
      - {name: free, type: Boolean}
    outcomes:
      - name: added
        creates: queueing.entry.Worker
        instance: worker_id
        emits: [queueing.entry.WorkerAdded]
        payload:
          queueing.entry.WorkerAdded: {worker_id: {generated: true}}
        sets:
          free: input.free
  - name: queueing.entry.Enqueue
    input: []
    outcomes:
      - name: enqueued
        creates: queueing.entry.Entry
        instance: entry_id
        emits: [queueing.entry.Enqueued]
        payload:
          queueing.entry.Enqueued: {entry_id: {generated: true}}
  - name: queueing.entry.Assign
    input:
      - {name: entry_id, type: queueing.entry.EntryId}
      - {name: worker, type: queueing.entry.WorkerId}
    outcomes:
      - name: dialler-refused
        external: the dialler refuses the call
        error: queueing.entry.DiallerRefused
      - name: assigned
        moves: queueing.entry.Entry.assign
        instance: entry_id
        emits: [queueing.entry.Assigned]
        sets: {worker: input.worker}
        payload:
          queueing.entry.Assigned: {entry_id: input.entry_id, worker: input.worker}
      - {name: wrong-state, wrong_state: true, error: queueing.entry.EntryStateConflict}
";

const QUEUEING_AWAITED: &str = "type: ess-scenario/1
domain: queueing.entry
scenario: the-system-assigns
summary: The system assigns a waiting entry
arrange:
  - instance: entry
    entity: queueing.entry.Entry
timeline:
  - at: 2026-01-05T09:00:00Z
    command: queueing.entry.AddWorker
    actor: queueing.entry.Caller
    input: {free: true}
    outcome: added
  - at: 2026-01-05T09:00:01Z
    command: queueing.entry.Enqueue
    actor: queueing.entry.Caller
    outcome: enqueued
    capture: {instance: entry, event: queueing.entry.Enqueued, field: entry_id}
  - at: 2026-01-05T09:00:02Z
    command: queueing.entry.Assign
    input: {entry_id: {$instance: entry}}
    outcome: assigned
";

/// One accepting branch and one external success, as in `tests/authored_external_answer.rs`.
const COURIER: &str = r"
format: ess/16
system: courier
version: v1
domain: courier.mail
events:
  - name: courier.mail.MailSent
    fields:
      - {name: recipient, type: String}
  - name: courier.mail.MailQueued
    fields:
      - {name: recipient, type: String}
errors:
  - name: courier.mail.Undeliverable
    fields: []
commands:
  - name: courier.mail.SendMail
    input:
      - {name: recipient, type: String}
    outcomes:
      - name: sent
        emits: [courier.mail.MailSent]
        payload:
          courier.mail.MailSent:
            recipient: input.recipient
      - name: queued
        external: the provider defers delivery
        emits: [courier.mail.MailQueued]
        payload:
          courier.mail.MailQueued:
            recipient: input.recipient
      - name: failed
        external: the provider rejects the address
        error: courier.mail.Undeliverable
";

const COURIER_QUEUED: &str = "type: ess-scenario/1
domain: courier.mail
scenario: a-deferred-mail
summary: A deferred mail is queued.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: courier.mail.SendMail
    input:
      recipient: one@example.test
    outcome: queued
";

fn billing(body: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: billing.invoice\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n{body}"
    )
}

const CREATED: &str = "timeline:
  - at: 2026-01-05T09:00:00Z
    command: billing.invoice.CreateInvoice
    actor: billing.invoice.Customer
    input:
      account_id: 00000000-0000-4000-8000-000000000001
      customer_email: buyer@example.test
      amount: {amount: 10, currency: EUR}
    outcome: accepted
";

const REJECTED: &str = "timeline:
  - at: 2026-01-05T09:00:00Z
    command: billing.invoice.CreateInvoice
    actor: billing.invoice.Customer
    input:
      account_id: 00000000-0000-4000-8000-000000000001
      customer_email: buyer@example.test
      amount: {amount: 0, currency: EUR}
    outcome: rejected
    error:
      name: billing.invoice.InvalidAmount
";

// ---- a target that answers one success branch with an error ------------------------------------

/// The interpreter, except that every answer reaching `branch` also carries `error`, whether the
/// runner sent the command or awaited it.
struct ErrsOnSuccess {
    inner: Interpreted,
    branch: &'static str,
    error: &'static str,
}

impl ErrsOnSuccess {
    fn taint(&self, result: &mut SemanticCommandResult) {
        if result
            .outcome
            .as_ref()
            .is_some_and(|reached| reached.to_string() == self.branch)
        {
            result.error = Some(DeclaredErrorValue::new(ErrorRef::new(
                self.error.parse().expect("a qualified name"),
            )));
        }
    }
}

impl ConformanceTarget for ErrsOnSuccess {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut result = self.inner.execute_command(request)?;
        self.taint(&mut result);
        Ok(result)
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
    fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
        match self.inner.await_outcome(request)? {
            AwaitAnswer::Reached {
                mut result,
                input,
                elapsed_ms,
            } => {
                self.taint(&mut result);
                Ok(AwaitAnswer::Reached {
                    result,
                    input,
                    elapsed_ms,
                })
            }
            other @ AwaitAnswer::NotIssued { .. } => Ok(other),
        }
    }
}

// ---- helpers -----------------------------------------------------------------------------------

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

/// The suite a library caller assembles by hand from `SuiteProvenance::of`.
fn hand_built(ir: &EssIr, text: &str) -> ConformanceSuite {
    let compiled = authoring(ir, text);
    assert!(compiled.is_complete(), "{:?}", compiled.refusals);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(ir));
    for (id, scenario) in compiled.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    suite
}

/// `text` compiled and run in the format the CLI selects for it.
fn run(ir: &EssIr, text: &str, target: &impl ConformanceTarget) -> ConformanceReport {
    let mut suite = hand_built(ir, text);
    suite.select_fresh_format_for(ir);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
}

/// The failed checks of the one scenario, as `about => observed`.
fn failed(report: &ConformanceReport) -> Vec<String> {
    report
        .scenarios
        .iter()
        .flat_map(|scenario| scenario.checks.iter())
        .filter(|check| check.status != Status::Passed)
        .map(|check| {
            format!(
                "{} => {:?}",
                check.about,
                check
                    .diagnostic
                    .as_ref()
                    .map(|it| it.observed.clone())
                    .unwrap_or_default()
            )
        })
        .collect()
}

// ---- the success act, on each lowering path ----------------------------------------------------

/// An awaited system-issued act runs `expected_outcome` after `AwaitOutcome`; the reached answer
/// is the current command result, so the absence check reads it.
#[test]
fn w1run1_security_an_awaited_success_answered_with_an_error_fails() {
    let ir = fixture(QUEUEING);
    let honest = run(&ir, QUEUEING_AWAITED, &Interpreted::for_model(ir.clone()));
    assert_eq!(failed(&honest), Vec::<String>::new(), "{STORY}: {honest:?}");

    let report = run(
        &ir,
        QUEUEING_AWAITED,
        &ErrsOnSuccess {
            inner: Interpreted::for_model(ir.clone()),
            branch: "queueing.entry.Assign/assigned",
            error: "queueing.entry.EntryStateConflict",
        },
    );
    assert_eq!(
        failed(&report),
        vec![r#"no error => ["error = queueing.entry.EntryStateConflict"]"#.to_owned()],
        "{STORY}: an awaited `assigned` answered with an error fails its step: {report:?}"
    );
}

/// An external success (`queued`, forced by `configure_external_outcome`) declares no error.
#[test]
fn w1run1_security_an_external_success_answered_with_an_error_fails() {
    let ir = fixture(COURIER);
    let honest = run(&ir, COURIER_QUEUED, &Interpreted::for_model(ir.clone()));
    assert_eq!(failed(&honest), Vec::<String>::new(), "{STORY}: {honest:?}");

    let report = run(
        &ir,
        COURIER_QUEUED,
        &ErrsOnSuccess {
            inner: Interpreted::for_model(ir.clone()),
            branch: "courier.mail.SendMail/queued",
            error: "courier.mail.Undeliverable",
        },
    );
    assert_eq!(
        failed(&report),
        vec![r#"no error => ["error = courier.mail.Undeliverable"]"#.to_owned()],
        "{STORY}: a forced external success answered with an error fails: {report:?}"
    );
}

// ---- the library contract for a hand-built authored suite --------------------------------------

/// The CHANGELOG: a hand-built suite stays at `SuiteProvenance::of`'s format unless the caller
/// selects one. Such a suite must be refused loudly, never admitted with the new check dropped.
#[test]
fn w1run1_security_a_hand_built_success_suite_at_the_default_format_is_refused() {
    let ir = example("billing");
    let suite = hand_built(&ir, &billing(CREATED));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/4",
        "{STORY}: the CHANGELOG names `SuiteProvenance::of` as ess-conformance/4"
    );
    assert!(
        suite.scenarios.values().any(|scenario| scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectNoError))),
        "{STORY}: the success act carries the absence check"
    );
    let admitted = AdmittedSuite::from_suite(&suite);
    assert!(
        admitted.is_err(),
        "{STORY}: a /4 suite holding expect_no_error is refused, not admitted"
    );
    let legacy = Runner::for_suite(&suite).run(&suite, &Interpreted::for_model(ir.clone()));
    assert!(
        legacy.is_err(),
        "{STORY}: the legacy runner refuses it too: {legacy:?}"
    );
}

/// A hand-built suite of a refusing act is unchanged: still `/4`, still run by `Runner::run`.
#[test]
fn w1run1_security_a_hand_built_refusal_suite_still_runs_on_the_legacy_runner() {
    let ir = example("billing");
    let suite = hand_built(&ir, &billing(REJECTED));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/4"
    );
    let report = Runner::for_suite(&suite)
        .run(&suite, &Interpreted::for_model(ir.clone()))
        .unwrap_or_else(|error| panic!("{STORY}: a refusing act runs as before: {error}"));
    assert_eq!(failed(&report), Vec::<String>::new(), "{STORY}: {report:?}");
}

/// The CHANGELOG's remedy for a hand-built suite: select the format with
/// `select_fresh_format_for`, admit the suite and run it with `Runner::run_admitted`. A caller
/// that follows it runs the success act with the absence check, and `Runner::run` still refuses
/// the selected format loudly rather than dropping the check.
#[test]
fn w1run1_security_the_stated_remedy_keeps_a_legacy_runner_caller_running() {
    let ir = example("billing");
    let mut suite = hand_built(&ir, &billing(CREATED));
    suite.select_fresh_format_for(&ir);
    let legacy = Runner::for_suite(&suite).run(&suite, &Interpreted::for_model(ir.clone()));
    assert!(
        legacy.is_err(),
        "{STORY}: `Runner::run` refuses a suite at {} loudly, as the CHANGELOG states",
        suite.provenance.suite_version
    );
    let admitted = AdmittedSuite::from_suite(&suite)
        .unwrap_or_else(|error| panic!("{STORY}: the selected format admits the suite: {error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
    assert_eq!(
        failed(&report),
        Vec::<String>::new(),
        "{STORY}: the CHANGELOG's remedy (`select_fresh_format_for`, then `run_admitted`) runs the \
         success act: {report:?}"
    );
}
