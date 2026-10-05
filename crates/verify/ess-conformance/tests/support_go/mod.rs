//! Running one suite through the generated Go package and through the Rust reference runner, and
//! reading back one verdict per scenario from each (beyond10x/ess#188).
//!
//! The Go side runs `go test` on the package [`ess_conformance::go::emit`] writes, beside a Go test
//! file that supplies the target, with `ESS_REPORT_FORMAT=2` so the run publishes a report whose
//! `outcomes` name every scenario's verdict. The Rust side runs [`Runner`] against a Rust target.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use ess_conformance::report::Status;
use ess_conformance::target::ConformanceTarget;
use ess_conformance::target::{
    AbsentInputRequest, ElapsedObservation, ElapsedObservationRequest, EntitySetupRequest,
    EventDeliveryRequest, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    InstantMark, InvocationObservationRequest, ObservedEvent, ObservedInvocation,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig};
use serde_json::{json, Value};
use std::cell::RefCell;

/// What one `go test` run of the emitted package came to.
pub struct GoRun {
    /// Every scenario's precise status, read from the run's report/2.
    /// Empty when the run published none, which is what a suite refused at admission leaves.
    pub outcomes: BTreeMap<String, String>,
    /// `go test`'s own output, for a diagnostic.
    pub log: String,
    /// Whether `go test` exited 0.
    pub success: bool,
}

/// A directory of its own for one Go package, under the build's temporary directory.
pub fn directory(label: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "ess-go-parity-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(directory.join("essconform")).unwrap();
    directory
}

/// Writes the emitted package for `suite` plus `extra` Go files into a fresh module.
pub fn package(label: &str, suite: &ConformanceSuite, extra: &[(&str, &str)]) -> PathBuf {
    let directory = directory(label);
    for artifact in ess_conformance::go::emit(suite).unwrap_or_else(|error| panic!("{error}")) {
        std::fs::write(directory.join(artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/goparity\n\ngo 1.21\n",
    )
    .unwrap();
    for (name, contents) in extra {
        std::fs::write(directory.join("essconform").join(name), contents).unwrap();
    }
    directory
}

/// Writes the emitted package for a coverage input plus `extra` Go files into a fresh module.
pub fn package_input(
    label: &str,
    input: &ess_conformance::coverage::AdmittedInput,
    extra: &[(&str, &str)],
) -> PathBuf {
    let directory = directory(label);
    for artifact in ess_conformance::go::emit_input(input).unwrap_or_else(|error| panic!("{error}"))
    {
        std::fs::write(directory.join(artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/goparity\n\ngo 1.21\n",
    )
    .unwrap();
    for (name, contents) in extra {
        std::fs::write(directory.join("essconform").join(name), contents).unwrap();
    }
    directory
}

/// Rewrites the embedded `suite.json` of the package at `directory` — how a test hands the Go
/// runtime a document the Rust admission would refuse before emitting it.
pub fn rewrite_suite(directory: &std::path::Path, edit: impl FnOnce(&mut serde_json::Value)) {
    let path = directory.join("essconform").join("suite.json");
    let mut document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    edit(&mut document);
    std::fs::write(path, serde_json::to_string(&document).unwrap()).unwrap();
}

/// Runs `test` in the package at `directory` with `env`, and reads the report it wrote.
pub fn go_test(directory: &std::path::Path, test: &str, env: &[(&str, &str)]) -> GoRun {
    let report = directory.join("report.json");
    let _ = std::fs::remove_file(&report);
    let mut command = std::process::Command::new("go");
    command
        .args([
            "test",
            "./essconform",
            "-run",
            &format!("^{test}$"),
            "-count=1",
            "-v",
        ])
        .env("GOWORK", "off")
        .env("GOMAXPROCS", "2")
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .current_dir(directory);
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().expect("required Go toolchain executes");
    let log = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut outcomes = BTreeMap::new();
    if let Ok(text) = std::fs::read_to_string(&report) {
        let document: serde_json::Value = serde_json::from_str(&text).unwrap();
        for (status, ids) in document["outcomes"].as_object().unwrap() {
            for id in ids.as_array().unwrap() {
                outcomes.insert(id.as_str().unwrap().to_owned(), status.clone());
            }
        }
    }
    GoRun {
        outcomes,
        log,
        success: output.status.success(),
    }
}

// ---- the transcript --------------------------------------------------------------------------

/// A Rust target that answers exactly as `inner` does and writes down every question and answer,
/// per scenario, for `fixtures/transcript-target-go.go` to give the Go runtime the same answers.
///
/// This is what makes "the same verdict per scenario" a comparison of the two *runners*: both are
/// asked about one implementation's behaviour, recorded once, so a verdict that differs is a
/// difference in how the suite was read. The Go replay also compares every request it is sent with
/// the recorded one and writes each difference down, so a runner that sends another input is seen
/// even where the verdict happens to agree.
pub struct Recorder<T> {
    inner: T,
    current: RefCell<String>,
    entries: RefCell<BTreeMap<String, Vec<Value>>>,
}

fn nodes<V: serde::Serialize>(value: &V) -> Value {
    serde_json::to_value(value).unwrap()
}

fn events(events: &[ObservedEvent]) -> Value {
    Value::Array(
        events
            .iter()
            .map(
                |event| json!({"event": event.event.to_string(), "payload": nodes(&event.payload)}),
            )
            .collect(),
    )
}

fn command_result(result: &SemanticCommandResult) -> Value {
    json!({
        "outcome": result.outcome.as_ref().map(|outcome| outcome.outcome.to_string()),
        "error": result.error.as_ref().map(|error| error.error.to_string()),
        "error_payload": result.error.as_ref().map(|error| nodes(&error.fields)),
        "consistency": result.consistency.as_ref().map(ToString::to_string),
        "direct_events": events(&result.direct_events),
        "response": result.response.as_ref().map(nodes),
    })
}

impl<T> Recorder<T> {
    /// Records every exchange `inner` has with a runner.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            current: RefCell::default(),
            entries: RefCell::default(),
        }
    }

    /// The transcript: every scenario's exchanges, in the order they happened.
    pub fn transcript(&self) -> Value {
        nodes(&*self.entries.borrow())
    }

    fn record<R>(
        &self,
        method: &str,
        key: String,
        request: Value,
        answer: Result<R, TargetError>,
        render: impl Fn(&R) -> Value,
    ) -> Result<R, TargetError> {
        let (error, result) = match &answer {
            Ok(value) => (Value::Null, render(value)),
            Err(TargetError::Unsupported { .. }) => ("unsupported".into(), Value::Null),
            // The command's answer rather than a failure (beyond10x/ess#265): the Go runtime reads
            // it as a result that took no branch and carries the refusal.
            Err(TargetError::NotGranted { actor }) => (
                Value::Null,
                json!({
                    "outcome": null, "error": null, "consistency": null, "direct_events": [],
                    "response": null, "not_granted": true, "not_granted_actor": actor,
                }),
            ),
            Err(other) => (other.to_string().into(), Value::Null),
        };
        let mut entry = serde_json::Map::new();
        entry.insert("method".to_owned(), Value::String(method.to_owned()));
        entry.insert("key".to_owned(), Value::String(key));
        entry.insert("request".to_owned(), request);
        entry.insert("error".to_owned(), error);
        entry.insert("result".to_owned(), result);
        self.entries
            .borrow_mut()
            .entry(self.current.borrow().clone())
            .or_default()
            .push(Value::Object(entry));
        answer
    }
}

impl<T: ConformanceTarget> ConformanceTarget for Recorder<T> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.current.replace(scenario.scenario.to_string());
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        let described = json!({
            "identity": nodes(&request.identity), "fields": nodes(&request.fields),
            "state": request.state.to_string(),
        });
        let key = request.entity.to_string();
        let answer = self.inner.establish_entity(request);
        self.record("establish_entity", key, described, answer, |()| Value::Null)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let described = json!({
            "actor": request.actor.as_ref().map(ToString::to_string),
            "caller": request.caller.as_ref().map(nodes),
            "input": nodes(&request.input),
        });
        let key = request.command.to_string();
        let answer = self.inner.execute_command(request);
        self.record("execute_command", key, described, answer, command_result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let described = json!({
            "actor": request.actor.as_ref().map(ToString::to_string),
            "caller": request.caller.as_ref().map(nodes),
        });
        let key = request.command.to_string();
        let answer = self.inner.execute_command_without_input(request);
        self.record(
            "execute_command_without_input",
            key,
            described,
            answer,
            command_result,
        )
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let at_least = match &request.consistency {
            ess_primitives::consistency::QueryConsistency::Current => String::new(),
            ess_primitives::consistency::QueryConsistency::AtLeast { token } => token.to_string(),
        };
        let described = json!({"params": nodes(&request.params), "at_least": at_least});
        let key = request.view.to_string();
        let answer = self.inner.query_view(request);
        self.record(
            "query_view",
            key,
            described,
            answer,
            |result| json!({"rows": nodes(&result.rows), "total": result.total}),
        )
    }
    // A read sent as an actor (beyond10x/ess#286) records the actor beside the read, and only
    // then, so a transcript of reads sent as no actor keeps its bytes.
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ess_conformance::scenario::ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        let at_least = match &request.consistency {
            ess_primitives::consistency::QueryConsistency::Current => String::new(),
            ess_primitives::consistency::QueryConsistency::AtLeast { token } => token.to_string(),
        };
        let described = json!({
            "params": nodes(&request.params), "at_least": at_least, "actor": reader.to_string(),
        });
        let key = request.view.to_string();
        let answer = self.inner.query_view_as(request, reader);
        self.record(
            "query_view",
            key,
            described,
            answer,
            |result| json!({"rows": nodes(&result.rows), "total": result.total}),
        )
    }
    // A read sent as no actor at all (beyond10x/ess#286), recorded as one.
    fn query_view_anonymous(
        &self,
        request: SemanticViewRequest,
    ) -> Result<SemanticViewResult, TargetError> {
        let at_least = match &request.consistency {
            ess_primitives::consistency::QueryConsistency::Current => String::new(),
            ess_primitives::consistency::QueryConsistency::AtLeast { token } => token.to_string(),
        };
        let described = json!({
            "params": nodes(&request.params), "at_least": at_least, "anonymous": true,
        });
        let key = request.view.to_string();
        let answer = self.inner.query_view_anonymous(request);
        self.record(
            "query_view",
            key,
            described,
            answer,
            |result| json!({"rows": nodes(&result.rows), "total": result.total}),
        )
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let key = request.event.to_string();
        let answer = self.inner.observe_events(request);
        self.record("observe_events", key, json!({}), answer, |seen| {
            events(seen)
        })
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        let key = format!("{}/{}", request.force.command, request.force.outcome);
        let answer = self.inner.configure_external_outcome(request);
        self.record("configure_external_outcome", key, json!({}), answer, |()| {
            Value::Null
        })
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        let key = format!("{}/{}", request.force.command, request.force.outcome);
        let answer = self
            .inner
            .configure_external_outcome_repeatedly(request, times);
        self.record(
            "configure_external_outcome_repeatedly",
            key,
            json!({"times": times.get()}),
            answer,
            |()| Value::Null,
        )
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        let key = request.event.to_string();
        let answer = self.inner.redeliver_event(request);
        self.record("redeliver_event", key, json!({}), answer, |()| Value::Null)
    }
    // An event an external channel delivers (ess/18), for the suites that deliver one.
    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        let key = format!("{}|{}", request.event, request.authority);
        let answer = self.inner.deliver_event(request);
        self.record("deliver_event", key, json!({}), answer, |()| Value::Null)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        let key = format!("{}|{}", request.binding, request.command);
        let answer = self.inner.observe_invocations(request);
        self.record("observe_invocations", key, json!({}), answer, |seen| {
            Value::Array(
                seen.iter()
                    .map(|invocation| {
                        json!({"command": invocation.command.to_string(), "input": nodes(&invocation.input)})
                    })
                    .collect(),
            )
        })
    }
    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        let key = request.instant.to_string();
        let answer = self.inner.mark_instant(request);
        self.record("mark_instant", key, json!({}), answer, |()| Value::Null)
    }
    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        let described = json!({
            "hold": request.hold.get(),
            "watching": request.watching.as_ref().map(ToString::to_string),
        });
        let key = request.instant.to_string();
        let answer = self.inner.observe_elapsed(request);
        self.record(
            "observe_elapsed",
            key,
            described,
            answer,
            |observed| json!({"elapsed_ms": observed.elapsed_ms, "published": observed.published}),
        )
    }
}

/// What replaying a transcript through the Go runtime came to.
pub struct Replayed {
    /// The Go run.
    pub go: GoRun,
    /// Every request the Go runtime sent that the recorded run did not, one line each.
    pub divergences: Vec<String>,
}

/// Runs `test` in the package at `directory` against the transcript `recorder` wrote.
pub fn replay<T>(
    directory: &std::path::Path,
    recorder: &Recorder<T>,
    extra: &[(&str, &str)],
) -> Replayed {
    let transcript = directory.join("transcript.json");
    std::fs::write(&transcript, recorder.transcript().to_string()).unwrap();
    let divergence = directory.join("divergence.txt");
    let _ = std::fs::remove_file(&divergence);
    let mut env = vec![
        ("ESS_TRANSCRIPT", transcript.to_str().unwrap().to_owned()),
        (
            "ESS_TRANSCRIPT_DIVERGENCE",
            divergence.to_str().unwrap().to_owned(),
        ),
    ];
    env.extend(extra.iter().map(|(key, value)| (*key, (*value).to_owned())));
    let env: Vec<(&str, &str)> = env
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect();
    let go = go_test(directory, "TestTranscriptReplay", &env);
    let divergences = std::fs::read_to_string(&divergence)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect();
    Replayed { go, divergences }
}

/// The transcript replay target, for [`package`] and [`package_input`].
pub const TRANSCRIPT_TARGET: (&str, &str) = (
    "transcript_test.go",
    include_str!("../fixtures/transcript-target-go.go"),
);

/// How one comparison is run.
#[derive(Default, Clone)]
pub struct Options {
    /// The wall clock both runners resolve `now_offset` values against, in milliseconds since the
    /// epoch; `None` leaves each runner its own default.
    pub wall_millis: Option<u64>,
    /// Hides every optional Go target interface, as for a target written before them.
    pub bare: bool,
}

/// The whole comparison for one suite and one Rust target: the reference verdicts, the Go verdicts
/// over the recorded answers, and every request the two runners sent differently.
pub fn compare<T: ConformanceTarget>(
    label: &str,
    suite: &ConformanceSuite,
    target: T,
) -> (BTreeMap<String, String>, Replayed) {
    compare_with(label, suite, target, &Options::default())
}

/// [`compare`], run as `options` says.
pub fn compare_with<T: ConformanceTarget>(
    label: &str,
    suite: &ConformanceSuite,
    target: T,
    options: &Options,
) -> (BTreeMap<String, String>, Replayed) {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let directory = package(label, suite, &[TRANSCRIPT_TARGET]);
    compare_in(&directory, &admitted, target, options)
}

/// [`compare_with`] for a coverage input: the Go package embeds the whole input.
pub fn compare_input<T: ConformanceTarget>(
    label: &str,
    input: &ess_conformance::coverage::AdmittedInput,
    target: T,
    options: &Options,
) -> (BTreeMap<String, String>, Replayed) {
    let directory = package_input(label, input, &[TRANSCRIPT_TARGET]);
    compare_in(&directory, input.selected(), target, options)
}

fn compare_in<T: ConformanceTarget>(
    directory: &std::path::Path,
    admitted: &AdmittedSuite,
    target: T,
    options: &Options,
) -> (BTreeMap<String, String>, Replayed) {
    let recorder = Recorder::new(target);
    let rust = match options.wall_millis {
        None => rust_outcomes_admitted(admitted, &recorder),
        Some(millis) => verdicts(
            Runner::new(
                RunnerConfig::default(),
                ess_conformance::now_offset::WithWall::new(AdvancingClock::default(), move || {
                    ess_primitives::time::Timestamp::from_epoch_millis(millis)
                }),
                Ids::for_suite(admitted.suite()),
            )
            .run_admitted(admitted, &recorder)
            .into_report(),
        ),
    };
    let mut env = Vec::new();
    let wall = options.wall_millis.map(|millis| millis.to_string());
    if let Some(wall) = &wall {
        env.push(("ESS_TEST_WALL_MILLIS", wall.as_str()));
    }
    if options.bare {
        env.push(("ESS_TRANSCRIPT_BARE", "1"));
    }
    let replayed = replay(directory, &recorder, &env);
    std::fs::remove_dir_all(directory).unwrap();
    (rust, replayed)
}

/// Asserts that the Go runtime gives the reference verdict for every scenario of `suite` against
/// `target`'s recorded behaviour, and sends every request the reference runner sent. Returns the
/// verdicts, for a caller that also wants to say which ones they are.
pub fn assert_parity<T: ConformanceTarget>(
    label: &str,
    suite: &ConformanceSuite,
    target: T,
) -> BTreeMap<String, String> {
    assert_parity_with(label, suite, target, &Options::default())
}

/// [`assert_parity`], run as `options` says.
pub fn assert_parity_with<T: ConformanceTarget>(
    label: &str,
    suite: &ConformanceSuite,
    target: T,
    options: &Options,
) -> BTreeMap<String, String> {
    let compared = compare_with(label, suite, target, options);
    assert_compared(label, compared)
}

/// Asserts what [`assert_parity`] asserts of a comparison already made.
pub fn assert_compared(
    label: &str,
    (rust, replayed): (BTreeMap<String, String>, Replayed),
) -> BTreeMap<String, String> {
    assert!(
        !replayed.go.outcomes.is_empty(),
        "{label}: the Go run published no verdicts:\n{}",
        replayed.go.log
    );
    assert_eq!(
        replayed.go.outcomes, rust,
        "{label}: per-scenario verdicts, Go (left) and Rust (right)\n{}",
        replayed.go.log
    );
    assert!(
        replayed.divergences.is_empty(),
        "{label}: the Go runtime sent requests the reference runner did not:\n{}\n{}",
        replayed.divergences.join("\n"),
        replayed.go.log
    );
    rust
}

/// The scenarios of a verdict map that did not pass, by id.
pub fn not_passed(verdicts: &BTreeMap<String, String>) -> Vec<&str> {
    verdicts
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id.as_str())
        .collect()
}

/// Every scenario's verdict from the Rust reference runner, preserving report/2 categories.
pub fn rust_outcomes<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    rust_outcomes_admitted(&admitted, target)
}

/// [`rust_outcomes`] for a suite already admitted — a coverage input's selected suite.
pub fn rust_outcomes_admitted<T: ConformanceTarget>(
    admitted: &AdmittedSuite,
    target: &T,
) -> BTreeMap<String, String> {
    verdicts(
        Runner::for_suite(admitted.suite())
            .run_admitted(admitted, target)
            .into_report(),
    )
}

fn verdicts(report: ess_conformance::ConformanceReport) -> BTreeMap<String, String> {
    report
        .scenarios
        .into_iter()
        .map(|result| {
            let status = match result.status {
                Status::Passed => "passed",
                Status::Failed => "failed",
                Status::Error => "error",
                Status::Unsupported => "unsupported",
            };
            (result.scenario.to_string(), status.to_owned())
        })
        .collect()
}
