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

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use ess_compiler::refs::{CommandRef, ErrorRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::now_offset::WithWall;
use ess_conformance::report::Status;
use ess_conformance::scenario::ScenarioInitialState;
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
        assert_eq!(
            synthesis.suite.provenance.scenario_initial_state,
            Some(ScenarioInitialState::Empty),
            "{}: fresh ordinary suite initial state",
            self.name
        );
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

// ---- adversary cases -------------------------------------------------------------------------------
//
// The harness above is a verbatim copy of `tests/typescript_suite_versions.rs` lines 1–607, so each
// case below meets the same two runners the unit's own parity cases meet. What differs is the target:
// each mode here is a thing a JavaScript implementation does naturally that the unit's fixtures
// carefully do not.

/// `tests/typescript_suite_versions.rs`'s presence model (#139).
const PRESENCE: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Receipt
    kind: struct
    fields:
      - {name: code, type: Optional<String>, presence: null_when_absent}
events:
  - name: demo.orders.Placed
    fields:
      - {name: partner_ref, type: Optional<String>, presence: null_when_absent}
      - {name: discount_code, type: Optional<String>, presence: omitted_when_absent}
      - {name: note, type: Optional<String>}
      - {name: receipt, type: demo.orders.Receipt}
      - {name: later, type: Optional<demo.orders.Receipt>}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: partner_ref, type: Optional<String>}
      - {name: discount_code, type: Optional<String>}
      - {name: note, type: Optional<String>}
      - {name: receipt, type: demo.orders.Receipt}
      - {name: later, type: Optional<demo.orders.Receipt>}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            partner_ref: input.partner_ref
            discount_code: input.discount_code
            note: input.note
            receipt: input.receipt
            later: input.later
";

/// The unit's presence target, with two more modes: an absent value written as a property holding
/// `undefined` — `{ partner_ref: order.partnerRef }` over an optional class member — which
/// `JSON.stringify` (and so every wire, and the proxy) leaves out.
fn presence_target() -> String {
    let base = include_str!("fixtures/typescript-presence-target.mjs");
    let patched = base
        .replacen(
            "    } else if (mode !== 'omits-partner-ref') {\n      payload.partner_ref = null;",
            "    } else if (mode === 'undefined-partner-ref') {\n      payload.partner_ref = \
             undefined;\n    } else if (mode !== 'omits-partner-ref') {\n      payload.partner_ref \
             = null;",
            1,
        )
        .replacen(
            "    } else if (mode === 'nulls-discount-code') {",
            "    } else if (mode === 'undefined-discount-code') {\n      payload.discount_code = \
             undefined;\n    } else if (mode === 'nulls-discount-code') {",
            1,
        );
    assert_ne!(
        patched, base,
        "the presence fixture changed shape; re-anchor the patch"
    );
    assert!(
        patched.contains("undefined-partner-ref") && patched.contains("undefined-discount-code")
    );
    patched
}

/// A `null_when_absent` leaf a JavaScript target leaves as `undefined` is left out on every wire,
/// and the Rust runner fails it; the TypeScript runtime reads the own property as present-and-null
/// and passes it. The reverse for `omitted_when_absent`: Rust sees it left out and passes, the
/// TypeScript runtime reads it as "sent as null" and fails it.
#[test]
fn adversary_typescript_fresh_presence_suite_gets_the_rust_verdict() {
    let target = presence_target();
    Case {
        name: "adv-presence-undefined",
        version: "ess-conformance/34",
        target: &target,
        modes: &[
            "correct",
            "undefined-partner-ref",
            "undefined-discount-code",
        ],
        also_correct: &["undefined-discount-code"],
    }
    .ordinary(&ir(PRESENCE));
}

// ---- authored suites -------------------------------------------------------------------------------

fn authored_suite(version: &str, id: &str, steps: &Value) -> String {
    json!({
        "provenance": {"suite_version": version, "system": "adv", "specification_version": "v1",
            "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {id: {"purpose": "adversary", "steps": steps, "source": []}},
    })
    .to_string()
}

/// A view whose one row is chosen by mode; `queryView` counts its calls so a snapshot and a re-read
/// see different rows.
const ROW_TARGET: &str = r"
import { JsonNumber } from './dist/runtime.js';
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
class Rows {
  reads = 0;
  identity() { return { name: 'adversary-rows', version: '1' }; }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command }) { throw new Error(`unexpected command ${command}`); }
  queryView() {
    this.reads += 1;
    switch (mode) {
      // contains {id, note: null}
      case 'correct': return { rows: [{ id: 'a', note: null }] };
      case 'undefined-note': return { rows: [{ id: 'a', note: undefined }] };
      case 'no-note': return { rows: [{ id: 'a' }] };
      // changed_by {total: 0.0000001}
      case 'tiny-correct':
        return { rows: [{ total: this.reads === 1 ? 0 : 0.0000001 }] };
      case 'tiny-wrong':
        return { rows: [{ total: this.reads === 1 ? 0 : 0.0000002 }] };
      // contains {n: 9007199254740993}
      case 'exact-correct': return { rows: [{ n: new JsonNumber('9007199254740993') }] };
      case 'exact-off-by-one': return { rows: [{ n: new JsonNumber('9007199254740992') }] };
      default: throw new Error(`unknown mode ${mode}`);
    }
  }
  observeEvents() { return []; }
  configureExternalOutcome() { throw new Error('nothing here is externally decided'); }
  redeliverEvent() { throw new Error('no bindings'); }
}
export function makeTarget() { return new Rows(); }
";

/// `contains {note: null}` on a plain (undotted) field keeps exact comparison in the Rust runner: a
/// row that does not carry `note` does not match. A JavaScript row `{note: undefined}` does not carry
/// it on any wire; the TypeScript runtime's `matches` finds the own property and `equal(undefined,
/// null)` holds.
#[test]
fn adversary_typescript_undefined_row_field_is_not_null() {
    let suite = authored_suite(
        "ess-conformance/26",
        "adv.rows/authored/null-note",
        &json!([
            {"step": "query_view", "view": "adv.rows.Rows"},
            {"step": "expect_view", "view": "adv.rows.Rows", "expectation": {
                "expect": "contains",
                "fields": {"id": {"kind": "literal", "value": "a"},
                           "note": {"kind": "literal", "value": null}}}},
        ]),
    );
    Case {
        name: "adv-undefined-row-field",
        version: "ess-conformance/26",
        target: ROW_TARGET,
        modes: &["correct", "no-note", "undefined-note"],
        also_correct: &[],
    }
    .authored(&suite);
}

/// `changed_by` compares exactly (suite/26, #148). The Rust runner spells a binary64 without an
/// exponent (`Number::exact_text`), so a sum that moved by 0.0000001 is a number it can subtract;
/// the TypeScript runtime's `exactDecimal` reads `String(1e-7)`, which is `1e-7`, and calls it
/// "not a number" — failing the correct target and the wrong one alike.
#[test]
fn adversary_typescript_changed_by_a_tiny_decimal() {
    let suite = format!(
        r#"{{"provenance": {{"suite_version": "ess-conformance/26", "system": "adv",
            "specification_version": "v1", "spec_digest": "{a}", "contract_digest": "{b}"}},
          "scenarios": {{"adv.rows/authored/tiny-change": {{"purpose": "adversary", "source": [],
            "steps": [
              {{"step": "query_view", "view": "adv.rows.Totals"}},
              {{"step": "snapshot_view", "view": "adv.rows.Totals"}},
              {{"step": "query_view", "view": "adv.rows.Totals"}},
              {{"step": "expect_view", "view": "adv.rows.Totals", "expectation": {{
                "expect": "changed_by", "fields": {{"total": 0.0000001}}}}}}
            ]}}}}}}"#,
        a = "a".repeat(64),
        b = "b".repeat(64),
    );
    Case {
        name: "adv-changed-by-tiny",
        version: "ess-conformance/26",
        target: ROW_TARGET,
        modes: &["tiny-correct", "tiny-wrong"],
        also_correct: &[],
    }
    .authored(&suite);
}

/// An integer literal past 2^53 in a `contains`. The Rust runner compares it exactly; the TypeScript
/// runtime decodes the literal through `plainNumbers`, so `9007199254740993` becomes
/// `9007199254740992` before it is compared, and a target answering the exact digits as a
/// `JsonNumber` — which the proxy and `retained-replay` target show is how a TypeScript target
/// carries an exact integer — is compared against the wrong number.
#[test]
fn adversary_typescript_exact_integer_literal_in_contains() {
    let suite = format!(
        r#"{{"provenance": {{"suite_version": "ess-conformance/26", "system": "adv",
            "specification_version": "v1", "spec_digest": "{a}", "contract_digest": "{b}"}},
          "scenarios": {{"adv.rows/authored/exact": {{"purpose": "adversary", "source": [],
            "steps": [
              {{"step": "query_view", "view": "adv.rows.Big"}},
              {{"step": "expect_view", "view": "adv.rows.Big", "expectation": {{
                "expect": "contains",
                "fields": {{"n": {{"kind": "literal", "value": 9007199254740993}}}}}}}}
            ]}}}}}}"#,
        a = "a".repeat(64),
        b = "b".repeat(64),
    );
    Case {
        name: "adv-exact-literal",
        version: "ess-conformance/26",
        target: ROW_TARGET,
        modes: &["exact-correct", "exact-off-by-one"],
        also_correct: &[],
    }
    .authored(&suite);
}
