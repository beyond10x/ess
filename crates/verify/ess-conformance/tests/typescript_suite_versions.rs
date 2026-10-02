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
//! disagreement, whatever it printed. Report/2 books a scenario the target could not expose as
//! `skipped`, which is the Rust runner's `Unsupported`; no scenario of these suites is refused by
//! name, so a `skipped` on only one side is a disagreement like any other.
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
use ess_conformance::coverage::{AdmittedInput, Origins, Scope};
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

    /// Runs the case over the coverage suite, as `--suite-format 5` writes it.
    fn coverage(&self, model: &EssIr) {
        let input: AdmittedInput =
            ess_conformance::coverage_build::build(model, &[], Scope::System, Origins::Generated)
                .unwrap_or_else(|error| panic!("{error}"));
        self.check(input.selected(), || {
            ess_conformance::ts::emit_input(&input).expect("the package emits")
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

// ---- the vocabulary, closed ----------------------------------------------------------------------

/// Every variant of `pub enum <name>` in `source`, as `rename_all = "snake_case"` writes its tag.
fn variants(source: &str, name: &str) -> Vec<String> {
    let start = source
        .find(&format!("pub enum {name} {{"))
        .unwrap_or_else(|| panic!("src/scenario.rs declares no `{name}`"));
    let mut depth = 0_i32;
    let mut found = Vec::new();
    for line in source[start..].lines() {
        let code = line.split("//").next().unwrap_or_default();
        if depth == 1 {
            let word = code.trim_start();
            let head: String = word
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect();
            if head.starts_with(|c: char| c.is_ascii_uppercase())
                && word[head.len()..].trim_start().starts_with(['{', ',', '('])
            {
                let mut tag = String::new();
                for (index, c) in head.chars().enumerate() {
                    if c.is_ascii_uppercase() && index > 0 {
                        tag.push('_');
                    }
                    tag.push(c.to_ascii_lowercase());
                }
                found.push(tag);
            }
        }
        depth += i32::try_from(code.matches('{').count()).expect("a count");
        depth -= i32::try_from(code.matches('}').count()).expect("a count");
        if depth == 0 && !found.is_empty() {
            break;
        }
    }
    assert!(
        !found.is_empty(),
        "`{name}` has no variant this reader found"
    );
    found
}

/// The body of the method of `ScenarioRun` whose declaration starts with `signature`: every line
/// from it to the `  }` that closes it at the class's indentation.
fn method<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("runtime.ts declares no `{}`", signature.trim()));
    let body = &source[start..];
    let end = body
        .find("\n  }\n")
        .unwrap_or_else(|| panic!("`{}` has no end", signature.trim()));
    &body[..end]
}

/// Every step, view expectation and scenario value the Rust runner executes is one the TypeScript
/// runtime executes — handled by the executor that runs it, not only read at admission.
/// Read off the sources so a new Rust variant requires a matching runtime implementation.
/// Named runtime omissions do not satisfy this gate. Semantic parity is checked separately.
#[test]
fn every_rust_suite_tag_is_executed_in_typescript() {
    let scenario = include_str!("../src/scenario.rs");
    let runtime = include_str!("../src/ts/runtime.ts");
    // Only the executors count: a label in the admission or decode switch says the tag is read,
    // not that it is run. Steps are run by `ScenarioRun.step`, expectations decided by
    // `ScenarioRun.decide`, values resolved by `resolve` and — the two observed-invocation kinds —
    // `resolveAccessorExpected`.
    let steps = method(runtime, "  async step(index: number, step: Step)");
    let expectations = method(runtime, "  decide(index: number, step: Step)");
    let values = format!(
        "{}\n{}",
        method(runtime, "  resolve(value: Value): Node {"),
        method(runtime, "  resolveAccessorExpected(value: Value)")
    );
    let mut missing = Vec::new();
    let mut counted = 0;
    for (enumeration, tags, executor) in [
        ("ScenarioStep", variants(scenario, "ScenarioStep"), steps),
        (
            "ViewExpectation",
            variants(scenario, "ViewExpectation"),
            expectations,
        ),
        (
            "ScenarioValue",
            variants(scenario, "ScenarioValue"),
            &values,
        ),
    ] {
        for tag in tags {
            counted += 1;
            let executed = executor.contains(&format!("case '{tag}':"))
                || executor.contains(&format!("=== '{tag}'"));
            if !executed {
                missing.push(format!("{enumeration}::{tag}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "the TypeScript runtime does not execute: {missing:?}"
    );
    println!("typescript vocabulary: {counted} Rust tag(s), each executed");
}

// ---- #188: a nested `sets:` struct with one generated leaf ---------------------------------------

/// The #188 reproduction: four leaves of a nested struct read from the input, one generated. Its
/// suite asserts the four under dotted paths, which is what makes it suite/26.
const DIALER: &str = "format: ess/14
system: demo
version: v1
domain: demo.dialer
types:
  - {name: demo.dialer.AgentId, kind: newtype, of: String}
  - name: demo.dialer.Lead
    kind: struct
    fields:
      - {name: id, type: String}
      - {name: uid, type: String}
      - {name: number, type: String}
      - {name: rank, type: Integer}
      - {name: data, type: String}
entities:
  - name: demo.dialer.Membership
    identity: {name: agent_id, type: demo.dialer.AgentId}
    fields:
      - {name: lead, type: Optional<demo.dialer.Lead>}
    lifecycle: {initial: Idle, states: [Idle], terminal: [Idle]}
events:
  - name: demo.dialer.Joined
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
  - name: demo.dialer.LeadSet
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead, type: demo.dialer.Lead}
actors:
  - {name: demo.dialer.Agent, may: [demo.dialer.Join, demo.dialer.SetLead]}
commands:
  - name: demo.dialer.Join
    outcomes:
      - name: joined
        creates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.Joined]
        payload:
          demo.dialer.Joined: {agent_id: {generated: true}}
  - name: demo.dialer.SetLead
    input:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead_id, type: String}
      - {name: lead_uid, type: String}
      - {name: lead_number, type: String}
      - {name: lead_data, type: String}
    outcomes:
      - name: lead-set
        updates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.LeadSet]
        payload:
          demo.dialer.LeadSet:
            agent_id: input.agent_id
            lead:
              id: input.lead_id
              uid: input.lead_uid
              number: input.lead_number
              rank: {generated: true}
              data: input.lead_data
        sets:
          lead:
            id: input.lead_id
            uid: input.lead_uid
            number: input.lead_number
            rank: {generated: true}
            data: input.lead_data
views:
  - name: demo.dialer.Memberships
    source: demo.dialer.Membership
    consistency: read_your_writes
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead, type: Optional<demo.dialer.Lead>}
";

const DIALER_CASE: Case<'static> = Case {
    name: "dialer",
    version: "",
    target: include_str!("fixtures/typescript-dialer-target.mjs"),
    modes: &[
        "correct",
        "drops-number-from-the-row",
        "wrong-number-on-the-event",
    ],
    also_correct: &[],
};

#[test]
fn issue_188_a_dotted_leaf_suite_26_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/26",
        ..DIALER_CASE
    }
    .ordinary(&ir(DIALER));
}

#[test]
fn issue_188_a_dotted_leaf_coverage_suite_27_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/27",
        ..DIALER_CASE
    }
    .coverage(&ir(DIALER));
}

// ---- suite/24: field presence policies (beyond10x/ess#139) --------------------------------------

/// The model of `tests/field_presence_synthesized.rs`: both policies, a field with none, and a
/// policy inside a struct that is always there.
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

const PRESENCE_CASE: Case<'static> = Case {
    name: "presence",
    version: "",
    target: include_str!("fixtures/typescript-presence-target.mjs"),
    modes: &[
        "correct",
        "omits-partner-ref",
        "nulls-discount-code",
        "omits-receipt-code",
    ],
    also_correct: &[],
};

#[test]
fn field_presence_suite_24_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/24",
        ..PRESENCE_CASE
    }
    .ordinary(&ir(PRESENCE));
}

#[test]
fn field_presence_coverage_suite_25_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/25",
        ..PRESENCE_CASE
    }
    .coverage(&ir(PRESENCE));
}

// ---- suite/22: outcome shapes (beyond10x/ess#144, #145, #150, #151, #152) ----------------------

const OUTCOME_SHAPES: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/outcome-shapes.yaml");

const OUTCOME_SHAPES_CASE: Case<'static> = Case {
    name: "outcome-shapes",
    version: "",
    target: include_str!("fixtures/typescript-outcome-shapes-target.mjs"),
    modes: &[
        "correct",
        "folds-unknown-into-wrong-state",
        "keeps-ended-rows",
        "offers-into-initial",
        "touch-writes",
    ],
    also_correct: &[],
};

#[test]
fn outcome_shapes_suite_22_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/22",
        ..OUTCOME_SHAPES_CASE
    }
    .ordinary(&ir(OUTCOME_SHAPES));
}

#[test]
fn outcome_shapes_coverage_suite_23_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/23",
        ..OUTCOME_SHAPES_CASE
    }
    .coverage(&ir(OUTCOME_SHAPES));
}

// ---- suite/26: a command invoked with no input at all (beyond10x/ess#170) -----------------------

const ABSENT_INPUT: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/absent-input.yaml");

const ABSENT_INPUT_CASE: Case<'static> = Case {
    name: "absent-input",
    version: "",
    target: include_str!("fixtures/typescript-absent-input-target.mjs"),
    modes: &[
        "correct",
        "reads-absent-as-empty",
        "accepts-absent",
        "cannot-omit",
        "no-method",
    ],
    also_correct: &[],
};

#[test]
fn a_command_without_input_suite_26_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/26",
        ..ABSENT_INPUT_CASE
    }
    .ordinary(&ir(ABSENT_INPUT));
}

#[test]
fn a_command_without_input_coverage_suite_27_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/27",
        ..ABSENT_INPUT_CASE
    }
    .coverage(&ir(ABSENT_INPUT));
}

// ---- suite/26: the caller a command is sent as (beyond10x/ess#168) --------------------------------

/// The model of `tests/caller_values.rs`.
const CALLER: &str = "format: ess/16
system: demo
version: v1
summary: Minimal repro.
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}
  - {name: demo.notes.AgentId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}
        emits: [demo.notes.NoteCreated]
        payload:
          demo.notes.NoteCreated: {note_id: {generated: true}, account_id: {caller: account_id}, text: input.text}
  - name: demo.notes.EditNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteEdited]
        payload:
          demo.notes.NoteEdited: {note_id: input.note_id, text: input.text}
errors:
  - name: demo.notes.NotYourNote
    summary: The caller is not the note's agent.
events:
  - name: demo.notes.NoteCreated
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: text, type: String}
  - name: demo.notes.NoteEdited
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
      - {name: state, type: demo.notes.Note.State}
";

const CALLER_CASE: Case<'static> = Case {
    name: "caller",
    version: "",
    target: include_str!("fixtures/typescript-caller-target.mjs"),
    modes: &[
        "correct",
        "first-account-ever",
        "anyone-edits",
        "only-the-first-caller-edits",
        "cannot-authenticate",
    ],
    also_correct: &[],
};

#[test]
fn caller_values_suite_26_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/26",
        ..CALLER_CASE
    }
    .ordinary(&ir(CALLER));
}

#[test]
fn caller_values_coverage_suite_27_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/27",
        ..CALLER_CASE
    }
    .coverage(&ir(CALLER));
}

// ---- suite/26: an instant relative to the moment of sending (beyond10x/ess#171) -----------------

const CURRENT_TIME: &str = include_str!("fixtures/current-time-guard.yaml");

const CURRENT_TIME_CASE: Case<'static> = Case {
    name: "current-time",
    version: "",
    target: include_str!("fixtures/typescript-current-time-target.mjs"),
    modes: &[
        "correct",
        "tolerates-120s",
        "tolerates-0s",
        "reads-a-fixed-clock",
    ],
    also_correct: &[],
};

#[test]
fn now_offset_values_suite_26_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/26",
        ..CURRENT_TIME_CASE
    }
    .ordinary(&ir(CURRENT_TIME));
}

#[test]
fn now_offset_values_coverage_suite_27_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/27",
        ..CURRENT_TIME_CASE
    }
    .coverage(&ir(CURRENT_TIME));
}

// ---- aggregate views, and the change an ungrouped one is witnessed by (beyond10x/ess#148) -------

/// `tests/fixtures/aggregate-optional-fields.yaml` with the ungrouped `Counted` view of
/// `tests/ungrouped_aggregate_delta.rs` beside its views: a `changed_by` expectation, so suite/26.
fn aggregate_model() -> String {
    format!(
        "{}  - name: demo.orders.Counted\n    source: demo.orders.Order\n    consistency: \
         read_your_writes\n    fields:\n      - {{name: orders, type: Integer, aggregate: {{count: \
         {{}}}}}}\n      - {{name: longest, type: Optional<Integer>, aggregate: {{max: duration, \
         skip_absent: true}}}}\n",
        include_str!("fixtures/aggregate-optional-fields.yaml")
    )
}

const AGGREGATE_CASE: Case<'static> = Case {
    name: "aggregate",
    version: "",
    target: include_str!("fixtures/typescript-aggregate-target.mjs"),
    modes: &[
        "correct",
        "counts-absent-durations",
        "drops-absent-groups",
        "counted-misses-one",
    ],
    also_correct: &[],
};

#[test]
fn aggregate_views_and_their_change_suite_26_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/26",
        ..AGGREGATE_CASE
    }
    .ordinary(&ir(&aggregate_model()));
}

#[test]
fn aggregate_views_and_their_change_coverage_suite_27_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/27",
        ..AGGREGATE_CASE
    }
    .coverage(&ir(&aggregate_model()));
}

// ---- suite/26: a page of a paged view (beyond10x/ess#174) -----------------------------------------

const PAGING: &str = include_str!("fixtures/view-paging.yaml");

const PAGING_CASE: Case<'static> = Case {
    name: "paging",
    version: "",
    target: include_str!("fixtures/typescript-paging-target.mjs"),
    modes: &[
        "correct",
        "shared",
        "ignores-paging",
        "ignores-page",
        "one-based",
        "pages-before-ordering",
        "total-is-page-length",
        "no-total",
    ],
    also_correct: &["shared"],
};

#[test]
fn view_paging_suite_26_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/26",
        ..PAGING_CASE
    }
    .ordinary(&ir(PAGING));
}

#[test]
fn view_paging_coverage_suite_27_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/27",
        ..PAGING_CASE
    }
    .coverage(&ir(PAGING));
}

// ---- suite/26: a binding's bounded retry (beyond10x/ess#165) --------------------------------------

const BOUNDED_RETRY: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");

const BOUNDED_RETRY_CASE: Case<'static> = Case {
    name: "bounded-retry",
    version: "",
    target: include_str!("fixtures/typescript-bounded-retry-target.mjs"),
    modes: &[
        "correct",
        "unbounded",
        "too-few",
        "retries-final",
        "cannot-repeat",
    ],
    also_correct: &[],
};

#[test]
fn bounded_retry_suite_26_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/26",
        ..BOUNDED_RETRY_CASE
    }
    .ordinary(&ir(BOUNDED_RETRY));
}

#[test]
fn bounded_retry_coverage_suite_27_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/27",
        ..BOUNDED_RETRY_CASE
    }
    .coverage(&ir(BOUNDED_RETRY));
}

// ---- suite/26: `defined()` over an `Optional` aggregate (beyond10x/ess#176) -----------------------

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");

const QUEUE_CASE: Case<'static> = Case {
    name: "defined-aggregate",
    version: "",
    target: include_str!("fixtures/typescript-queue-target.mjs"),
    modes: &[
        "correct",
        "keeps-metrics-on-resume",
        "empty-metrics-on-resume",
    ],
    also_correct: &[],
};

#[test]
fn defined_over_an_optional_aggregate_suite_26_runs_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/26",
        ..QUEUE_CASE
    }
    .ordinary(&ir(QUEUE));
}

#[test]
fn defined_over_an_optional_aggregate_coverage_suite_27_runs_in_typescript_with_the_rust_verdicts()
{
    Case {
        version: "ess-conformance/27",
        ..QUEUE_CASE
    }
    .coverage(&ir(QUEUE));
}

// ---- 0.38.0 constructs that take no suite version of their own ------------------------------------
//
// `related` (#166) and set effects (#167, #175) are synthesized as steps every earlier suite had,
// so a suite carrying only them keeps its earlier version; beside any round-3 construct they ride
// in /26 or /27. Either way the TypeScript runtime has to give the Rust verdict.

/// The model of `tests/related_values.rs`.
const SHIPPING: &str = "format: ess/16
system: demo
version: v1
domain: demo.shipping
types:
  - {name: demo.shipping.CustomerId, kind: newtype, of: String}
  - {name: demo.shipping.ShipmentId, kind: newtype, of: String}
  - {name: demo.shipping.Region, kind: newtype, of: String}
entities:
  - name: demo.shipping.Customer
    identity: {name: customer_id, type: demo.shipping.CustomerId}
    fields:
      - {name: region, type: demo.shipping.Region}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.shipping.Shipment
    identity: {name: shipment_id, type: demo.shipping.ShipmentId}
    fields:
      - {name: customer_id, type: demo.shipping.CustomerId}
      - {name: region, type: Optional<demo.shipping.Region>}
    lifecycle: {initial: Packed, states: [Packed], terminal: [Packed]}
events:
  - name: demo.shipping.CustomerRegistered
    fields:
      - {name: customer_id, type: demo.shipping.CustomerId}
  - name: demo.shipping.ShipmentPacked
    fields:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
      - {name: region, type: demo.shipping.Region}
  - name: demo.shipping.ShipmentDispatched
    fields:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
      - {name: region, type: demo.shipping.Region}
actors:
  - name: demo.shipping.Clerk
    may: [demo.shipping.Register, demo.shipping.Pack, demo.shipping.Dispatch]
commands:
  - name: demo.shipping.Register
    input:
      - {name: region, type: demo.shipping.Region}
    outcomes:
      - name: registered
        creates: demo.shipping.Customer
        instance: customer_id
        emits: [demo.shipping.CustomerRegistered]
        payload:
          demo.shipping.CustomerRegistered: {customer_id: {generated: true}}
        sets:
          region: input.region
  - name: demo.shipping.Pack
    input:
      - {name: customer_id, type: demo.shipping.CustomerId}
    outcomes:
      - name: packed
        creates: demo.shipping.Shipment
        instance: shipment_id
        emits: [demo.shipping.ShipmentPacked]
        payload:
          demo.shipping.ShipmentPacked:
            shipment_id: {generated: true}
            region: {related: {via: input.customer_id, field: region}}
        sets:
          customer_id: input.customer_id
  - name: demo.shipping.Dispatch
    input:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
    outcomes:
      - name: dispatched
        updates: demo.shipping.Shipment
        instance: shipment_id
        emits: [demo.shipping.ShipmentDispatched]
        payload:
          demo.shipping.ShipmentDispatched:
            shipment_id: input.shipment_id
            region: {related: {via: customer_id, field: region}}
        sets:
          region: {related: {via: customer_id, field: region}}
views:
  - name: demo.shipping.Shipments
    source: demo.shipping.Shipment
    consistency: read_your_writes
    fields:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
      - {name: region, type: Optional<demo.shipping.Region>}
";

const RELATED_CASE: Case<'static> = Case {
    name: "related",
    version: "",
    target: include_str!("fixtures/typescript-related-target.mjs"),
    modes: &["correct", "latest-customer", "other-customer"],
    also_correct: &[],
};

#[test]
fn related_values_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/4",
        ..RELATED_CASE
    }
    .ordinary(&ir(SHIPPING));
}

#[test]
fn related_values_coverage_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/5",
        ..RELATED_CASE
    }
    .coverage(&ir(SHIPPING));
}

const SET_EFFECTS: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");

const SET_EFFECTS_CASE: Case<'static> = Case {
    name: "set-effects",
    version: "",
    target: include_str!("fixtures/typescript-set-effects-target.mjs"),
    modes: &[
        "correct",
        "end-ignores-filter",
        "end-skips-one",
        "end-miscounts",
        "end-ignores-from",
        "end-ignores-sets",
        "note-ignores-filter",
        "note-skips-one",
        "note-miscounts",
        "invite-holds-subject",
        "invite-skips-others",
        "invite-holds-every-team",
    ],
    also_correct: &[],
};

#[test]
fn set_effects_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/12",
        ..SET_EFFECTS_CASE
    }
    .ordinary(&ir(SET_EFFECTS));
}

#[test]
fn set_effects_coverage_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/13",
        ..SET_EFFECTS_CASE
    }
    .coverage(&ir(SET_EFFECTS));
}

// ---- suite/12: retained results, which every later major implies ---------------------------------

const RETAINED_REPLAY: &str = include_str!("fixtures/retained-replay.yaml");

const RETAINED_REPLAY_CASE: Case<'static> = Case {
    name: "retained-replay",
    version: "",
    target: include_str!("fixtures/typescript-retained-replay-target.mjs"),
    modes: &[
        "correct",
        "next-result",
        "new-stamp",
        "null-optional",
        "extra-field",
        "reordered-list",
        "error",
        "missing-response",
        "extra-event",
        "ambiguous-original-event",
        "mutated-subject",
    ],
    also_correct: &[],
};

#[test]
fn retained_results_suite_12_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/12",
        ..RETAINED_REPLAY_CASE
    }
    .ordinary(&ir(RETAINED_REPLAY));
}

#[test]
fn retained_results_coverage_suite_13_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/13",
        ..RETAINED_REPLAY_CASE
    }
    .coverage(&ir(RETAINED_REPLAY));
}

// ---- suite/14: string operators in an authored `satisfies` (beyond10x/ess#95) ---------------------

/// A suite a person wrote: every row of a view satisfies three string operators. Synthesis decides
/// a string guard itself, so only an authored expectation carries one into a suite.
fn text_match_suite(version: &str) -> String {
    json!({
        "provenance": {"suite_version": version, "system": "routing", "specification_version": "v1",
            "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"routing.calls/authored/rows": {
            "purpose": "Every row is a non-UK urgent corporate call",
            "steps": [
                {"step": "query_view", "view": "routing.calls.Rows"},
                {"step": "expect_view", "view": "routing.calls.Rows", "expectation": {
                    "expect": "satisfies",
                    "predicate": {"all": [
                        {"not": {"caller": {"starts_with": "+44"}}},
                        {"email": {"ends_with": "@corp.example"}},
                        {"subject": {"contains": "urgent"}},
                    ]},
                }},
            ],
            "source": []}},
    })
    .to_string()
}

const TEXT_MATCH_CASE: Case<'static> = Case {
    name: "text-match",
    version: "",
    target: include_str!("fixtures/typescript-text-match-target.mjs"),
    modes: &[
        "correct",
        "uk-caller",
        "personal-mail",
        "routine",
        "numeric-subject",
    ],
    also_correct: &[],
};

#[test]
fn string_operators_suite_14_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/14",
        ..TEXT_MATCH_CASE
    }
    .authored(&text_match_suite("ess-conformance/14"));
}

#[test]
fn string_operators_in_a_suite_26_envelope_run_in_typescript_with_the_rust_verdicts() {
    Case {
        version: "ess-conformance/26",
        ..TEXT_MATCH_CASE
    }
    .authored(&text_match_suite("ess-conformance/26"));
}
