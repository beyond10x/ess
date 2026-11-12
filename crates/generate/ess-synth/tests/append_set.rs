//! An append to a stored list, `sets: {tags: {append: input.tag}}` (`ess/31`), in the generated
//! Rust and Go behaviours: the commands stay generated, the row they write holds a copy of the
//! previous list (an absent optional one read as empty) with the input element last, and the Go
//! module builds. The suite synthesized over a `List<Optional<String>>` sends a present element and
//! leaves one out; it passes against the generated Rust server and the generated Go server.
//!
//! The Rust harness is not built here: a test that runs cargo compiles a crate graph inside its
//! lane (`crates/edge/ess-xtask/tests/no_test_builds.rs`). `task append-set-harness` generates the
//! Rust server and builds the harness once, before the test binaries run, and the Rust case runs
//! the binary `ESS_APPEND_SET_HARNESS` names. Without it that case fails, naming the task.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError, ViewRow,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, DeclaredErrorValue, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{consistency::ConsistencyToken, node::Node};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const JOBS: &str = r"
format: ess/31
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: tags, type: List<String>}
      - {name: notes, type: Optional<List<String>>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
events:
  - name: demo.jobs.Changed
    fields: [{name: job_id, type: demo.jobs.JobId}]
errors:
  - name: demo.jobs.NoSuchJob
commands:
  - name: demo.jobs.AddJob
    input: [{name: tags, type: List<String>}]
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets: {tags: input.tags}
        emits: [demo.jobs.Changed]
        payload: {demo.jobs.Changed: {job_id: {generated: true}}}
  - name: demo.jobs.AddTag
    input: [{name: job_id, type: demo.jobs.JobId}, {name: tag, type: String}]
    outcomes:
      - name: tagged
        updates: demo.jobs.Job
        instance: job_id
        sets: {tags: {append: input.tag}}
        emits: [demo.jobs.Changed]
        payload: {demo.jobs.Changed: {job_id: input.job_id}}
      - {name: unknown, unknown_instance: true, error: demo.jobs.NoSuchJob}
  - name: demo.jobs.AddNote
    input: [{name: job_id, type: demo.jobs.JobId}, {name: note, type: String}]
    outcomes:
      - name: noted
        updates: demo.jobs.Job
        instance: job_id
        sets: {notes: {append: input.note}}
        emits: [demo.jobs.Changed]
        payload: {demo.jobs.Changed: {job_id: input.job_id}}
      - {name: unknown, unknown_instance: true, error: demo.jobs.NoSuchJob}
views:
  - name: demo.jobs.Jobs
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: tags, type: List<String>}
      - {name: notes, type: Optional<List<String>>}
components:
  - component: jobs-service
    owns: {domains: [demo.jobs]}
    accepts: {commands: [demo.jobs.AddJob, demo.jobs.AddTag, demo.jobs.AddNote]}
    publishes: {events: [demo.jobs.Changed]}
    reached_by: network
";

fn model() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("jobs.yaml"),
        RawSpecFile::parse(JOBS).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// Every artifact of `target`'s synthesis, by path, with both appending behaviours generated.
fn artifacts(target: Target) -> BTreeMap<String, String> {
    let synthesis = synthesize_for(&model(), target).expect("a realizable target");
    for command in ["demo.jobs.AddTag", "demo.jobs.AddNote"] {
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, command),
            Some(&SynthesisDisposition::Generated),
            "{target:?}: the behaviour of {command} is generated"
        );
    }
    synthesis
        .artifacts
        .into_iter()
        .map(|(path, artifact)| (path, artifact.contents))
        .collect()
}

fn artifact<'a>(artifacts: &'a BTreeMap<String, String>, path: &str) -> &'a str {
    artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    })
}

/// The Go `artifacts`, written under a fresh directory and built offline; the log, and whether it
/// built. Go only: a test here spawns no cargo (`ess-xtask/tests/no_test_builds.rs`).
fn go_build(artifacts: BTreeMap<String, String>) -> (String, bool) {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("append-set-go-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, contents) in artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    let output = Command::new("go")
        .args(["build", "-buildvcs=false", "./..."])
        .current_dir(&directory)
        .env("GOWORK", "off")
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    (log, output.status.success())
}

#[test]
fn generated_rust_appends_to_the_previous_list() {
    let rust = artifacts(Target::Rust);
    let behaviour = artifact(&rust, "crates/demo-types/src/behaviour.rs");
    assert!(
        behaviour.contains(".clone(); items.push(") && behaviour.contains("items }"),
        "the Rust row holds the previous tags with the element last:\n{behaviour}"
    );
    assert!(
        behaviour.contains(".clone().unwrap_or_default(); items.push(")
            && behaviour.contains("Some(items) }"),
        "an absent optional list reads as empty:\n{behaviour}"
    );
}

#[test]
fn generated_go_appends_to_a_fresh_slice_and_builds() {
    let go = artifacts(Target::Go);
    let behaviour = artifact(&go, "types/behaviour/behaviour.go");
    assert!(
        behaviour.contains("append(append([]string{}, "),
        "the Go row holds a fresh slice with the element last:\n{behaviour}"
    );
    assert!(
        behaviour.contains("items := []string{}; if "),
        "an absent optional list reads as empty:\n{behaviour}"
    );
    let (log, built) = go_build(go);
    assert!(built, "{log}");
}

// ---- a present element, through the generated servers ------------------------------------------

/// `LABELLED` of `ess-conformance`'s `append_set` with a component serving it, read by
/// `task append-set-harness` too, which generates the Rust server the harness is built against
/// from the same file. It is named `.yaml.in` so the walkers that read every repository model do
/// not read it.
const LABELLED: &str = include_str!("fixtures/append-set/model.yaml.in");

/// The variable naming the Rust harness `task append-set-harness` built.
const HARNESS: &str = "ESS_APPEND_SET_HARNESS";

const LABELLED_ID: &str = "demo.jobs.AddLabel/outcome/labelled";

fn labelled() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("labelled.yaml"),
        RawSpecFile::parse(LABELLED).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// Every route of the model's component, as `name method path` triples.
fn routes(ir: &EssIr) -> Vec<String> {
    let mut routes = Vec::new();
    for component in ir.components().values() {
        for route in ess_gen::http::routes(ir, component) {
            let name = match route.serves {
                ess_gen::http::Served::Command(handle) => ir.command(handle).name.to_string(),
                ess_gen::http::Served::View(handle) => ir.view(handle).name.to_string(),
            };
            routes.extend([name, route.method.as_str().to_owned(), route.path]);
        }
    }
    routes
}

/// The harness `task append-set-harness` built against the generated Rust server, from the path
/// [`HARNESS`] names. A file that is not executable is made so: an uploaded CI artifact loses its
/// mode bits.
fn rust_harness() -> PathBuf {
    let Some(path) = std::env::var_os(HARNESS).map(PathBuf::from) else {
        panic!(
            "{HARNESS} is not set: run `task append-set-harness` and set it to the binary that \
             task names, or run the tests through `task test`, which does both"
        );
    };
    let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
        panic!(
            "{HARNESS}={} names no harness ({error}): run `task append-set-harness`",
            path.display()
        )
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            permissions.set_mode(permissions.mode() | 0o755);
            std::fs::set_permissions(&path, permissions).expect("the harness is made executable");
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    path
}

/// The harness built against the generated Go server, under a fresh directory, and the directory.
fn go_harness() -> (PathBuf, PathBuf) {
    let ir = labelled();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("append-set-served-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let tree = root.join("go-demo");
    for artifact in synthesize_for(&ir, Target::Go)
        .expect("the model synthesizes to Go")
        .artifacts
        .values()
    {
        let path = tree.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).unwrap();
        std::fs::write(&path, &artifact.contents).unwrap();
    }
    let harness = root.join("go-harness");
    std::fs::create_dir_all(&harness).unwrap();
    std::fs::write(
        harness.join("go.mod"),
        "module appendsetharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\nreplace \
         example.invalid/demo => ../go-demo\n",
    )
    .unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/append-set-go-harness/main.go"),
        harness.join("main.go"),
    )
    .unwrap();
    let binary = root.join("harness-go");
    let output = Command::new("go")
        .args(["build", "-buildvcs=false", "-o"])
        .arg(&binary)
        .arg(".")
        .current_dir(&harness)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "the Go harness builds:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (binary, root)
}

/// One running harness, and the events one scenario published.
struct Served {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ObservedEvent>>,
    sequence: RefCell<u64>,
}

impl Served {
    fn start(binary: &Path, ir: &EssIr) -> Self {
        let mut child = Command::new(binary)
            .args(routes(ir))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the harness starts");
        let stdin = child.stdin.take().expect("piped");
        let stdout = BufReader::new(child.stdout.take().expect("piped"));
        Self {
            child: RefCell::new(child),
            stdin: RefCell::new(stdin),
            stdout: RefCell::new(stdout),
            published: RefCell::default(),
            sequence: RefCell::new(0),
        }
    }

    /// One request line out, one `{status, body}` line back.
    fn ask(&self, request: &Value) -> (u64, Value) {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        let answer: Value =
            serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"));
        (
            answer["status"].as_u64().unwrap_or_default(),
            answer["body"].clone(),
        )
    }

    fn tick(&self) -> u64 {
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        *sequence
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.borrow_mut().kill();
        let _ = self.child.borrow_mut().wait();
    }
}

fn failure(observation: &str, answer: &Value) -> TargetError {
    TargetError::unavailable(observation, answer.to_string())
}

fn node_of(value: &Value) -> Node {
    serde_json::from_value(value.clone()).expect("the wire writes JSON a node reads")
}

impl ConformanceTarget for Served {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "append-set",
            env!("CARGO_PKG_VERSION"),
        ))
    }

    fn begin_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.ask(&json!({"op": "reset"}));
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let observation = format!("invoking `{}`", request.command);
        let body = serde_json::to_value(&request.input).expect("a suite input is JSON");
        let (_, answer) = self.ask(&json!({
            "op": "command",
            "command": request.command.to_string(),
            "body": body.to_string(),
        }));
        let Some(outcome) = answer["outcome"].as_str() else {
            return Err(failure(&observation, &answer));
        };
        let outcome = OutcomeRef::new(
            CommandRef::new(request.command.name().clone()),
            outcome
                .parse()
                .map_err(|_| failure(&observation, &answer))?,
        );
        let error = match answer.get("error") {
            Some(error) => {
                let name: ErrorRef = error
                    .as_str()
                    .and_then(|name| name.parse().ok())
                    .ok_or_else(|| failure(&observation, &answer))?;
                Some(DeclaredErrorValue::new(name))
            }
            None => None,
        };
        let mut direct_events = Vec::new();
        for published in answer["published"].as_array().into_iter().flatten() {
            let reference: EventRef = published["event"]
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(|| failure(&observation, &answer))?;
            let mut event = ObservedEvent::new(reference);
            if let Some(Value::Object(payload)) = published.get("payload") {
                for (field, value) in payload {
                    event = event.with(field.clone(), node_of(value));
                }
            }
            let event = event
                .in_activity(request.correlation.clone())
                .at(self.tick());
            self.published.borrow_mut().push(event.clone());
            direct_events.push(event);
        }
        let consistency = Some(
            ConsistencyToken::new(format!("seq:{}", self.tick()))
                .map_err(|why| TargetError::unavailable(observation.clone(), why.to_string()))?,
        );
        Ok(SemanticCommandResult {
            outcome: Some(outcome),
            error,
            consistency,
            direct_events,
            response: None,
        })
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let observation = format!("reading `{}`", request.view);
        let (status, answer) = self.ask(&json!({
            "op": "view",
            "view": request.view.to_string(),
            "query": "",
        }));
        let Some(rows) = answer["rows"].as_array().filter(|_| status == 200) else {
            return Err(failure(&observation, &answer));
        };
        Ok(SemanticViewResult::of(rows.iter().map(|row| {
            row.as_object()
                .into_iter()
                .flatten()
                .map(|(field, value)| (field.clone(), node_of(value)))
                .collect::<ViewRow>()
        })))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("forcing `{}`", request.force),
            "the model declares no external branch",
        ))
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("delivering `{}` again", request.event),
            "the model declares no binding",
        ))
    }

    fn end_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.published.borrow_mut().clear();
        Ok(())
    }
}

/// The suite `LABELLED` synthesizes, which sends `AddLabel` a present element and leaves one out.
fn labelled_suite(ir: &EssIr) -> ConformanceSuite {
    let mut suite = ess_conformance::synthesize(ir).suite;
    suite.select_fresh_format_for(ir);
    suite
}

/// Every scenario's status against the harness `binary`, which must hold the scenario that sends
/// `label: "label"`.
fn verdicts(ir: &EssIr, suite: &ConformanceSuite, binary: &Path) -> BTreeMap<String, Status> {
    let sends_present = format!(
        "{:?}",
        suite
            .scenarios
            .iter()
            .find(|(id, _)| id.to_string() == LABELLED_ID)
    );
    assert!(
        sends_present.contains(r#"Text("label")"#),
        "story:a-present-optional-element-is-appended: the suite sends `label`: {sends_present}"
    );
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let target = Served::start(binary, ir);
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &target)
        .into_report();
    assert_eq!(report.scenarios.len(), suite.len());
    report
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn assert_all_passed(server: &str, verdicts: &BTreeMap<String, Status>) {
    assert!(
        verdicts.contains_key(LABELLED_ID),
        "{server}: no labelled scenario in {verdicts:#?}"
    );
    let not_passed: Vec<String> = verdicts
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect();
    assert!(
        not_passed.is_empty(),
        "story:a-present-optional-element-is-appended: {server}: every scenario passes: \
         {not_passed:#?}"
    );
}

#[test]
fn the_generated_rust_server_passes_the_suite_that_sends_a_present_element() {
    let ir = labelled();
    let verdicts = verdicts(&ir, &labelled_suite(&ir), &rust_harness());
    assert_all_passed("rust", &verdicts);
}

#[test]
fn the_generated_go_server_passes_the_suite_that_sends_a_present_element() {
    let ir = labelled();
    let (binary, root) = go_harness();
    let verdicts = verdicts(&ir, &labelled_suite(&ir), &binary);
    let _ = std::fs::remove_dir_all(root);
    assert_all_passed("go", &verdicts);
}

// Adversary pass 1 of story:a-present-optional-element-is-appended: the present element of other
// element types, through the generated Go server.

/// `LABELLED` over `List<Optional<{element}>>`, `types` declared beside the identity, compiled.
fn labelled_of(element: &str, types: &str) -> EssIr {
    let text = LABELLED
        .replace("Optional<String>", &format!("Optional<{element}>"))
        .replace(
            "  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}\n",
            &format!("  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}\n{types}"),
        );
    let spec = Specification::assemble([(
        Source::new("labelled.yaml"),
        RawSpecFile::parse(&text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The suite of `ir` run against the generated Go server built from it: every scenario passes.
fn go_passes(ir: &EssIr, tag: &str) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("append-set-adv-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let tree = root.join("go-demo");
    for artifact in synthesize_for(ir, Target::Go)
        .expect("the model synthesizes to Go")
        .artifacts
        .values()
    {
        let path = tree.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).unwrap();
        std::fs::write(&path, &artifact.contents).unwrap();
    }
    let harness = root.join("go-harness");
    std::fs::create_dir_all(&harness).unwrap();
    std::fs::write(
        harness.join("go.mod"),
        "module appendsetharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\nreplace \
         example.invalid/demo => ../go-demo\n",
    )
    .unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/append-set-go-harness/main.go"),
        harness.join("main.go"),
    )
    .unwrap();
    let binary = root.join("harness-go");
    let output = Command::new("go")
        .args(["build", "-buildvcs=false", "-o"])
        .arg(&binary)
        .arg(".")
        .current_dir(&harness)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{tag}: the Go harness builds:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let suite = labelled_suite(ir);
    let labelled = format!(
        "{:#?}",
        suite
            .scenarios
            .iter()
            .find(|(id, _)| id.to_string() == LABELLED_ID)
    );
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let target = Served::start(&binary, ir);
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    drop(target);
    let _ = std::fs::remove_dir_all(&root);
    let not_passed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| format!("{}: {:?}", result.scenario, result))
        .collect();
    assert!(
        not_passed.is_empty(),
        "story:a-present-optional-element-is-appended: go, {tag}: every scenario passes: \
         {not_passed:#?}\nthe labelled scenario: {labelled}"
    );
}

#[test]
fn adv_go_appends_a_present_integer() {
    go_passes(&labelled_of("Integer", ""), "integer");
}

#[test]
fn adv_go_appends_a_present_boolean() {
    go_passes(&labelled_of("Boolean", ""), "boolean");
}

#[test]
fn adv_go_appends_a_present_timestamp() {
    go_passes(&labelled_of("Timestamp", ""), "timestamp");
}

#[test]
fn adv_go_appends_a_present_enum() {
    go_passes(
        &labelled_of(
            "demo.jobs.Colour",
            "  - {name: demo.jobs.Colour, kind: enum, variants: [Red, Green]}\n",
        ),
        "enum",
    );
}

#[test]
fn adv_go_appends_a_present_struct() {
    go_passes(
        &labelled_of(
            "demo.jobs.Tag",
            "  - name: demo.jobs.Tag\n    kind: struct\n    fields:\n      - {name: name, type: \
             String}\n      - {name: note, type: Optional<String>}\n",
        ),
        "struct",
    );
}
