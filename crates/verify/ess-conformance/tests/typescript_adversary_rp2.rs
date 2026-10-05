//! The emitted TypeScript runtime runs every suite version the synthesizer writes, and gives the
//! verdict the Rust reference runner gives (beyond10x/ess#188).
//!
//! 0.38.0 synthesized `ess-conformance/26` for a nested `sets:` struct with one `{generated:
//! true}` leaf, and the TypeScript package the same release generated refused it as
//! `unsupported suite version` — zero verdicts, where the Rust runner gave one per scenario.
//!
//! # One target, two runners
//!
//! Each case synthesizes a suite from a model in this file and runs it twice against **one**
//! target written in JavaScript (`tests/fixtures/typescript-*-target.mjs`): through the emitted
//! TypeScript package, and through the Rust [`Runner`], which reaches the same target over a line
//! of JSON per call (`tests/fixtures/typescript-parity-proxy.mjs`). A target written twice would
//! let the two copies disagree and call it a runtime difference; written once, any difference in
//! the verdicts is a difference between the runners. Every target has a correct mode and the wrong
//! ones the construct exists to catch, and the two runners must agree on every scenario in each.
//!
//! The comparison is of the reports: the Rust [`ConformanceReport`](ess_conformance::report) and
//! the TypeScript `ess-conformance-report/2` document. A run that writes no report is a
//! disagreement, whatever it printed. Report/2 preserves the native unsupported category;
//! a category difference is a disagreement like any other.
//!
//! Skipped, and said out loud, where the machine has no `tsc` or no `node`, as
//! `tests/typescript_runtime.rs` is.

#![allow(dead_code, unused_imports)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use ess_compiler::refs::{CommandRef, ErrorRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::now_offset::WithWall;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, AdvancingClock, Ids, Runner, RunnerConfig};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::consistency::{ConsistencyToken, QueryConsistency};
use ess_primitives::node::Node;
use ess_primitives::time::Timestamp;
use serde_json::{json, Value};

// ---- the harness ---------------------------------------------------------------------------------

/// Whether both tools this file needs are here; prints why not when they are not.
fn toolchain() -> bool {
    for name in ["tsc", "node"] {
        let found = Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        if !found {
            println!("skipped: no `{name}` on PATH, so the TypeScript runtime was not run");
            return false;
        }
    }
    true
}

/// Per-scenario verdicts, by scenario id.
type Verdicts = BTreeMap<String, String>;

/// An emitted TypeScript package, compiled, with a target, the driver and the proxy beside it.
struct Package {
    dir: PathBuf,
}

impl Package {
    /// Writes `files` and `target` under this test binary's scratch directory and compiles them.
    fn new(case: &str, files: Vec<ess_conformance::ts::TsArtifact>, target: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("typescript-suite-versions")
            .join(format!("{case}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for artifact in files {
            let path = root.join(&artifact.path);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
            std::fs::write(path, artifact.contents).expect("an artifact writes");
        }
        let dir = root.join(ess_conformance::ts::PACKAGE);
        for (name, contents) in [
            ("target.mjs", target),
            (
                "driver.mjs",
                include_str!("fixtures/typescript-parity-driver.mjs"),
            ),
            (
                "proxy.mjs",
                include_str!("fixtures/typescript-parity-proxy.mjs"),
            ),
            (
                "runtime-test.tsconfig.json",
                r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
            ),
        ] {
            std::fs::write(dir.join(name), contents).expect("a harness file writes");
        }
        let compiled = Command::new("tsc")
            .args(["--project", "runtime-test.tsconfig.json"])
            .current_dir(&dir)
            .output()
            .expect("tsc runs");
        assert!(
            compiled.status.success(),
            "tsc refused the emitted package:\n{}{}",
            String::from_utf8_lossy(&compiled.stdout),
            String::from_utf8_lossy(&compiled.stderr)
        );
        Self { dir }
    }

    /// Runs the TypeScript runner in `mode` and reads the verdicts off its report/2 document.
    fn typescript(&self, mode: &str) -> Result<Verdicts, String> {
        let report = self.dir.join(format!("report-{mode}.json"));
        let _ = std::fs::remove_file(&report);
        let run = Command::new("node")
            .args(["--test", "driver.mjs"])
            .env("ESS_TARGET_MODE", mode)
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .current_dir(&self.dir)
            .output()
            .expect("node runs");
        let printed = format!(
            "{}{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let Ok(text) = std::fs::read_to_string(&report) else {
            return Err(format!("the TypeScript run wrote no report:\n{printed}"));
        };
        let document: Value = serde_json::from_str(&text).expect("report/2 is JSON");
        let mut verdicts = Verdicts::new();
        for (status, ids) in document["outcomes"]
            .as_object()
            .expect("report/2 carries outcomes")
        {
            for id in ids.as_array().expect("an outcome list") {
                verdicts.insert(id.as_str().expect("an id").to_owned(), status.clone());
            }
        }
        Ok(verdicts)
    }

    /// Runs the Rust runner in `mode` against the same target, and reads its report.
    ///
    /// With the machine's clock as its wall, as `ess verify conform run` builds it: a `now_offset`
    /// is resolved against the clock the target decides by, which is the machine's in both runs.
    fn rust(&self, admitted: &AdmittedSuite, mode: &str) -> Verdicts {
        let target = JsTarget::spawn(&self.dir, mode);
        let wall = || {
            let millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("the machine's clock is after 1970")
                .as_millis();
            Timestamp::from_epoch_millis(u64::try_from(millis).expect("a millisecond count"))
        };
        Runner::new(
            RunnerConfig::default(),
            WithWall::new(AdvancingClock::default(), wall),
            Ids::for_suite(admitted.suite()),
        )
        .run_admitted(admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            // The current producer profile preserves native status categories.
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
}

/// One construct's parity case.
struct Case<'a> {
    /// A name for the scratch directory.
    name: &'a str,
    /// The suite version the model synthesizes, which the case pins.
    version: &'a str,
    /// The JavaScript target.
    target: &'a str,
    /// Its modes; the first is the correct one.
    modes: &'a [&'a str],
    /// Modes after the first that are correct too, and so pass everything in Rust.
    also_correct: &'a [&'a str],
}

impl Case<'_> {
    /// Runs the case over an ordinary suite.
    fn ordinary(&self, model: &EssIr) {
        let synthesis = ess_conformance::synthesize::synthesize(model);
        // A synthesis refusal leaves its scenario out of the suite, so it is out of both runs.
        for refusal in &synthesis.refusals {
            println!("{}: synthesis refused {}", self.name, refusal.code());
        }
        let admitted =
            AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
        self.check(&admitted, || {
            ess_conformance::ts::emit(admitted.suite()).expect("the package emits")
        });
    }

    /// Runs the case over a suite document written by hand, as an authored suite is.
    fn authored(&self, document: &str) {
        let admitted = AdmittedSuite::from_json(document).unwrap_or_else(|error| panic!("{error}"));
        self.check(&admitted, || {
            ess_conformance::ts::emit(admitted.suite()).expect("the package emits")
        });
    }

    fn check(
        &self,
        admitted: &AdmittedSuite,
        files: impl FnOnce() -> Vec<ess_conformance::ts::TsArtifact>,
    ) {
        let version = admitted.suite().provenance.suite_version.to_string();
        assert_eq!(
            version, self.version,
            "{}: the pinned suite version",
            self.name
        );
        if !toolchain() {
            return;
        }
        let package = Package::new(
            &format!("{}-{version}", self.name).replace('/', "-"),
            files(),
            self.target,
        );
        for (index, mode) in self.modes.iter().enumerate() {
            let rust = package.rust(admitted, mode);
            assert!(
                !rust.is_empty(),
                "{}: the suite holds no scenario",
                self.name
            );
            if index == 0 || self.also_correct.contains(mode) {
                let wrong: Vec<_> = rust.iter().filter(|(_, s)| *s != "passed").collect();
                assert!(
                    wrong.is_empty(),
                    "{}: the correct target does not pass the Rust runner: {wrong:?}",
                    self.name
                );
            } else {
                assert!(
                    rust.values().any(|status| status != "passed"),
                    "{}: mode {mode} is caught by nothing, so it checks nothing",
                    self.name
                );
            }
            let typescript = package
                .typescript(mode)
                .unwrap_or_else(|log| panic!("{} mode {mode}: {log}", self.name));
            assert_eq!(
                typescript, rust,
                "{} mode {mode}: the TypeScript runtime disagrees with the Rust runner",
                self.name
            );
            let failed = rust.values().filter(|status| *status != "passed").count();
            println!(
                "typescript parity, {} {version} mode {mode}: {} scenario(s), {failed} not passed, \
                 the same in both runners",
                self.name,
                rust.len(),
            );
        }
    }
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

// ---- the Rust runner's way into a JavaScript target ------------------------------------------------

/// A JavaScript target the Rust runner drives through `proxy.mjs`.
struct JsTarget {
    child: RefCell<Child>,
    io: RefCell<(ChildStdin, BufReader<ChildStdout>)>,
}

impl JsTarget {
    fn spawn(dir: &Path, mode: &str) -> Self {
        let mut child = Command::new("node")
            .arg("proxy.mjs")
            .env("ESS_TARGET_MODE", mode)
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("node runs the proxy");
        let stdin = child.stdin.take().expect("a stdin");
        let stdout = BufReader::new(child.stdout.take().expect("a stdout"));
        Self {
            child: RefCell::new(child),
            io: RefCell::new((stdin, stdout)),
        }
    }

    /// Calls `method`; `Ok(None)` where the target does not define it.
    fn call(&self, method: &str, args: Value) -> Result<Option<Value>, TargetError> {
        let mut io = self.io.borrow_mut();
        let mut envelope = serde_json::Map::new();
        envelope.insert("method".to_owned(), Value::from(method));
        envelope.insert("args".to_owned(), args);
        let line = Value::Object(envelope).to_string();
        writeln!(io.0, "{line}").expect("the proxy reads");
        io.0.flush().expect("the proxy reads");
        let mut reply = String::new();
        io.1.read_line(&mut reply).expect("the proxy answers");
        let reply: Value = serde_json::from_str(&reply)
            .unwrap_or_else(|error| panic!("the proxy answered {reply:?}: {error}"));
        if reply["missing"] == Value::Bool(true) {
            return Ok(None);
        }
        if let Some(error) = reply.get("error") {
            let error = error.as_str().unwrap_or_default().to_owned();
            return Err(if reply["unsupported"] == Value::Bool(true) {
                TargetError::unsupported(method, error)
            } else {
                TargetError::unavailable(method, error)
            });
        }
        Ok(Some(reply["ok"].clone()))
    }

    /// Calls a method every target defines.
    fn required(&self, method: &str, args: Value) -> Result<Value, TargetError> {
        self.call(method, args)?
            .ok_or_else(|| TargetError::unavailable(method, "the target does not define it"))
    }
}

impl Drop for JsTarget {
    fn drop(&mut self) {
        let _ = self.child.borrow_mut().kill();
        let _ = self.child.borrow_mut().wait();
    }
}

fn node(value: &Value) -> Node {
    serde_json::from_value(value.clone()).expect("a JSON value is a node")
}

fn fields(value: &Value) -> BTreeMap<String, Node> {
    value
        .as_object()
        .map(|object| object.iter().map(|(k, v)| (k.clone(), node(v))).collect())
        .unwrap_or_default()
}

fn events(value: &Value) -> Vec<ObservedEvent> {
    value
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|event| {
            let mut observed =
                ObservedEvent::new(event["event"].as_str().expect("an event").parse().unwrap());
            observed.payload = fields(&event["payload"]);
            observed
        })
        .collect()
}

fn command_result(command: &CommandRef, value: &Value) -> SemanticCommandResult {
    let mut result = match value["outcome"].as_str() {
        Some(outcome) if !outcome.is_empty() => SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            OutcomeName::new(outcome).expect("an outcome name"),
        )),
        _ => SemanticCommandResult::undeclared(),
    };
    if let Some(error) = value["error"].as_str().filter(|error| !error.is_empty()) {
        let mut declared = DeclaredErrorValue::new(error.parse::<ErrorRef>().expect("an error"));
        declared.fields = fields(&value["errorFields"]);
        result = result.with_error(declared);
    }
    if let Some(token) = value["consistency"]
        .as_str()
        .filter(|token| !token.is_empty())
    {
        result = result.with_consistency(ConsistencyToken::new(token).expect("a token"));
    }
    result.direct_events = events(&value["directEvents"]);
    if value["response"].is_object() {
        result.response = Some(fields(&value["response"]));
    }
    result
}

fn caller(caller: Option<&BTreeMap<String, Node>>) -> Value {
    caller.map_or(Value::Null, |caller| {
        serde_json::to_value(caller).expect("a caller serializes")
    })
}

fn deadline() -> Value {
    json!({"attempts": 1})
}

impl ConformanceTarget for JsTarget {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        let identity = self.required("identity", Value::Null)?;
        Ok(ImplementationIdentity::new(
            identity["name"].as_str().unwrap_or_default(),
            identity["version"].as_str().unwrap_or_default(),
        ))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.required(
            "beginScenario",
            json!({"scenario": scenario.scenario.to_string(), "correlation": scenario.correlation.to_string()}),
        )
        .map(drop)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.required(
            "endScenario",
            json!({"scenario": scenario.scenario.to_string(), "correlation": scenario.correlation.to_string()}),
        )
        .map(drop)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut args = json!({
            "command": request.command.to_string(),
            "actor": request.actor.as_ref().map(ToString::to_string).unwrap_or_default(),
            "input": serde_json::to_value(&request.input).expect("an input serializes"),
            "correlation": request.correlation.to_string(),
        });
        if request.caller.is_some() {
            args["caller"] = caller(request.caller.as_ref());
        }
        let result = self.required("executeCommand", args)?;
        Ok(command_result(&request.command, &result))
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut args = json!({
            "command": request.command.to_string(),
            "actor": request.actor.as_ref().map(ToString::to_string).unwrap_or_default(),
            "correlation": request.correlation.to_string(),
        });
        if request.caller.is_some() {
            args["caller"] = caller(request.caller.as_ref());
        }
        match self.call("executeCommandWithoutInput", args)? {
            Some(result) => Ok(command_result(&request.command, &result)),
            None => Err(TargetError::unsupported(
                "executeCommandWithoutInput",
                "the target does not define it",
            )),
        }
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let at_least = match &request.consistency {
            QueryConsistency::AtLeast { token } => token.to_string(),
            QueryConsistency::Current => String::new(),
        };
        let result = self.required(
            "queryView",
            json!({
                "view": request.view.to_string(),
                "params": serde_json::to_value(&request.params).expect("params serialize"),
                "atLeast": at_least,
                "correlation": request.correlation.to_string(),
                "deadline": deadline(),
            }),
        )?;
        let rows = result["rows"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(fields);
        let mut answer = SemanticViewResult::of(rows);
        if let Some(total) = result["total"].as_u64() {
            answer = answer.with_total(total);
        }
        Ok(answer)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let result = self.required(
            "observeEvents",
            json!({
                "event": request.event.to_string(),
                "correlation": request.correlation.to_string(),
                "deadline": deadline(),
            }),
        )?;
        Ok(events(&result))
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.required(
            "configureExternalOutcome",
            json!({
                "command": request.force.command.to_string(),
                "outcome": request.force.outcome.to_string(),
                "correlation": request.correlation.to_string(),
            }),
        )
        .map(drop)
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        match self.call(
            "configureExternalOutcomeRepeatedly",
            json!({
                "command": request.force.command.to_string(),
                "outcome": request.force.outcome.to_string(),
                "correlation": request.correlation.to_string(),
                "times": times.get(),
            }),
        )? {
            Some(_) => Ok(()),
            None => Err(TargetError::unsupported(
                "configureExternalOutcomeRepeatedly",
                "the target does not define it",
            )),
        }
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.required(
            "redeliverEvent",
            json!({"event": request.event.to_string(), "correlation": request.correlation.to_string()}),
        )
        .map(drop)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        let Some(result) = self.call(
            "observeInvocations",
            json!({
                "binding": request.binding.to_string(),
                "command": request.command.to_string(),
                "correlation": request.correlation.to_string(),
                "deadline": deadline(),
            }),
        )?
        else {
            return Err(TargetError::unsupported(
                "observeInvocations",
                "the target does not define it",
            ));
        };
        Ok(result
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(|invocation| {
                let mut observed = ObservedInvocation::new(
                    request.binding.clone(),
                    invocation["command"]
                        .as_str()
                        .expect("a command")
                        .parse::<CommandRef>()
                        .expect("a command name"),
                );
                observed.input = fields(&invocation["input"]);
                observed
            })
            .collect())
    }
}

// ---- adversary pass 2 cases ------------------------------------------------------------------------
//
// The harness above is a verbatim copy of `tests/typescript_adversary_rp1.rs` lines 1–596 (itself a
// copy of `tests/typescript_suite_versions.rs`), so each case meets the same two runners. Pass 2
// attacks the correction `cde14a7bb`: `readAsJSON`/`asJSON` read a target's answer "as JSON reads
// it", but rebuild only plain objects and arrays, and pass every other object through untouched.
// The Rust runner reads the same answer through `JSON.stringify` in `proxy.mjs`, which serialises a
// class instance's own enumerable properties (dropping those holding `undefined`) and calls
// `toJSON` (a `Date` becomes its ISO string).

fn authored_suite(version: &str, id: &str, steps: &Value) -> String {
    json!({
        "provenance": {"suite_version": version, "system": "adv", "specification_version": "v1",
            "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {id: {"purpose": "adversary", "steps": steps, "source": []}},
    })
    .to_string()
}

/// A view whose one row is chosen by mode.
const ROW_TARGET: &str = r"
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
// A row a JavaScript implementation keeps as a class: `note` is declared and never assigned, so
// under `useDefineForClassFields` (the ES2022 default) it is an own property holding `undefined`.
class Row {
  id;
  note;
  constructor(id) { this.id = id; }
}
// The whole answer as a class, holding plain rows.
class Answer {
  constructor(rows) { this.rows = rows; }
}
class Rows {
  identity() { return { name: 'adversary2-rows', version: '1' }; }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command }) { throw new Error(`unexpected command ${command}`); }
  queryView() {
    switch (mode) {
      // contains {id: 'a', note: null}
      case 'correct': return { rows: [{ id: 'a', note: null }] };
      case 'class-row': return { rows: [new Row('a')] };
      case 'class-answer': return new Answer([{ id: 'a', note: undefined }]);
      // contains {at: '2026-01-01T00:00:00.000Z'}
      case 'at-string': return { rows: [{ at: '2026-01-01T00:00:00.000Z' }] };
      case 'at-date': return { rows: [{ at: new Date(Date.UTC(2026, 0, 1)) }] };
      default: throw new Error(`unknown mode ${mode}`);
    }
  }
  observeEvents() { return []; }
  configureExternalOutcome() { throw new Error('nothing here is externally decided'); }
  redeliverEvent() { throw new Error('no bindings'); }
}
export function makeTarget() { return new Rows(); }
";

/// Pass 1's `undefined-note` case, with the row (or the answer holding it) a class instance.
/// `asJSON` returns any non-plain object as is, so the own property holding `undefined` survives to
/// `matches`, where `equal(undefined, null)` holds; `JSON.stringify` drops it, and the Rust runner
/// fails the row for not carrying `note`. Expected: both modes fail in both runners.
#[test]
fn adversary2_typescript_class_instance_answer_is_read_as_json() {
    let suite = authored_suite(
        "ess-conformance/26",
        "adv.rows/authored/class-note",
        &json!([
            {"step": "query_view", "view": "adv.rows.Rows"},
            {"step": "expect_view", "view": "adv.rows.Rows", "expectation": {
                "expect": "contains",
                "fields": {"id": {"kind": "literal", "value": "a"},
                           "note": {"kind": "literal", "value": null}}}},
        ]),
    );
    Case {
        name: "adv2-class-instance",
        version: "ess-conformance/26",
        target: ROW_TARGET,
        modes: &["correct", "class-row"],
        also_correct: &[],
    }
    .authored(&suite);
}

/// As above, with the rows plain objects and only the answer holding them a class instance: the
/// plain rows under it are never rebuilt either, so the pass-1 `undefined-note` defect returns.
#[test]
fn adversary2_typescript_class_answer_holding_plain_rows_is_read_as_json() {
    let suite = authored_suite(
        "ess-conformance/26",
        "adv.rows/authored/class-answer",
        &json!([
            {"step": "query_view", "view": "adv.rows.Rows"},
            {"step": "expect_view", "view": "adv.rows.Rows", "expectation": {
                "expect": "contains",
                "fields": {"id": {"kind": "literal", "value": "a"},
                           "note": {"kind": "literal", "value": null}}}},
        ]),
    );
    Case {
        name: "adv2-class-answer",
        version: "ess-conformance/26",
        target: ROW_TARGET,
        modes: &["correct", "class-answer"],
        also_correct: &[],
    }
    .authored(&suite);
}

/// A `Date` in a row is what `JSON.stringify` makes of it — its ISO string — on every wire and in
/// the Rust runner, which passes `contains {at: "<iso>"}`. `asJSON` passes the `Date` through, and
/// the TypeScript runtime compares an object with a string and fails it.
#[test]
fn adversary2_typescript_date_in_a_row_is_read_as_json() {
    let suite = authored_suite(
        "ess-conformance/26",
        "adv.rows/authored/date",
        &json!([
            {"step": "query_view", "view": "adv.rows.Rows"},
            {"step": "expect_view", "view": "adv.rows.Rows", "expectation": {
                "expect": "contains",
                "fields": {"at": {"kind": "literal", "value": "2026-01-01T00:00:00.000Z"}}}},
        ]),
    );
    Case {
        name: "adv2-date",
        version: "ess-conformance/26",
        target: ROW_TARGET,
        modes: &["at-string", "at-date"],
        also_correct: &["at-date"],
    }
    .authored(&suite);
}
