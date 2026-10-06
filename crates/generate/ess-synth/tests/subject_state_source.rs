//! The held lifecycle state as a value source, `{subject: state}` (ess/23, beyond10x/ess#458), in
//! the generated Rust and Go behaviours.
//!
//! Each fills `StateConflict.current` from the held row's state, `DocPublished.from` from the state
//! before the move, and `previous` the same way. The test builds the generated Rust server beside
//! `tests/fixtures/subject-state-source-harness/` and the generated Go server beside
//! `tests/fixtures/subject-state-source-go-harness/` (a storage port each, no behaviour), runs the
//! suite the same specification synthesizes against each, and again against a mutant that answers
//! the requested state in `current`, which fails a refusal scenario.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::report::Status;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_synth::{synthesize_for, Target};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/subject-state-source.yaml");

/// The refusal a target answering the requested state in `current` fails: `archive` asked of a
/// draft, where the held state and the requested one differ.
const SWAPPED: &str = "demo.docs.Doc/state/Draft/refuses/demo.docs.ArchiveDoc";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap();
    let spec = Specification::assemble([(Source::new("docs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

/// The synthesized tree for `target`, written under a fresh scratch directory.
fn tree(target: Target, label: &str) -> PathBuf {
    let synthesis = synthesize_for(&ir(), target).expect("the model synthesizes");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("subject-state-458-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = root.join("demo").join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&destination, &artifact.contents).unwrap();
    }
    root
}

/// `source` with `from` replaced by `to`, which must occur.
fn replaced(source: &str, from: &str, to: &str) -> String {
    assert!(
        source.contains(from),
        "`{from}` is in the generated behaviour:\n{source}"
    );
    source.replace(from, to)
}

// ---- Rust ----------------------------------------------------------------------------------------

fn cargo(directory: &Path, target: &Path) -> Output {
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .args(["build", "--offline", "--quiet", "--target-dir"])
        .arg(target)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-D warnings");
    let output = command.output().expect("cargo runs");
    eprintln!(
        "cargo build in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// The Rust harness over the generated behaviour as `mutate` leaves it, kept as `name`.
fn rust_build(root: &Path, name: &str, mutate: impl FnOnce(&str) -> String) -> PathBuf {
    let harness = root.join("harness");
    if !harness.join("Cargo.toml").exists() {
        std::fs::create_dir_all(harness.join("src")).unwrap();
        let dependencies = ["demo-server", "demo-system", "docs-service", "demo-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../demo/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"state-harness\"\nversion = \"0.0.0\"\nedition = \
                 \"2021\"\npublish = false\n\n[workspace]\n\n[[bin]]\nname = \"harness\"\npath = \
                 \"src/main.rs\"\n\n[dependencies]\n{dependencies}"
            ),
        )
        .unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/subject-state-source-harness/main.rs"),
            harness.join("src/main.rs"),
        )
        .unwrap();
    }
    let behaviour = root.join("demo/crates/demo-types/src/behaviour.rs");
    let original = root.join("behaviour.rs.original");
    if !original.exists() {
        std::fs::copy(&behaviour, &original).unwrap();
    }
    let source = mutate(&std::fs::read_to_string(&original).unwrap());
    std::fs::write(&behaviour, source).unwrap();
    let target = root.join("target");
    assert!(
        cargo(&harness, &target).status.success(),
        "the Rust harness builds over `{name}`"
    );
    let kept = root.join("kept");
    std::fs::create_dir_all(&kept).unwrap();
    std::fs::copy(target.join("debug/harness"), kept.join(name)).unwrap();
    kept.join(name)
}

// ---- Go ------------------------------------------------------------------------------------------

fn go(directory: &Path, arguments: &[&str]) -> Output {
    let output = Command::new("go")
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("go runs");
    eprintln!(
        "go {arguments:?} in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// The Go harness over the generated behaviour as `mutate` leaves it, kept as `name`.
fn go_build(root: &Path, name: &str, mutate: impl FnOnce(&str) -> String) -> PathBuf {
    let harness = root.join("harness");
    if !harness.join("go.mod").exists() {
        std::fs::create_dir_all(&harness).unwrap();
        std::fs::write(
            harness.join("go.mod"),
            "module stateharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\nreplace \
             example.invalid/demo => ../demo\n",
        )
        .unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/subject-state-source-go-harness/main.go"),
            harness.join("main.go"),
        )
        .unwrap();
    }
    let behaviour = root.join("demo/types/behaviour/behaviour.go");
    let original = root.join("behaviour.go.original");
    if !original.exists() {
        std::fs::copy(&behaviour, &original).unwrap();
    }
    let source = mutate(&std::fs::read_to_string(&original).unwrap());
    std::fs::write(&behaviour, source).unwrap();
    let kept = root.join("kept");
    std::fs::create_dir_all(&kept).unwrap();
    let binary = kept.join(name);
    assert!(
        go(
            &harness,
            &["build", "-o", binary.to_str().expect("a UTF-8 path"), "."]
        )
        .status
        .success(),
        "the Go harness builds over `{name}`"
    );
    binary
}

// ---- the runs ------------------------------------------------------------------------------------

/// Every scenario of the suite the model synthesizes, run through one harness binary.
fn run(binary: &Path) -> Vec<(String, Status)> {
    let ir = ir();
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).expect("admits");
    let target = Served::start(binary, &ir);
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    for scenario in &report.scenarios {
        if scenario.status != Status::Passed {
            eprintln!(
                "{} via {}: {scenario:#?}",
                scenario.scenario,
                binary.display()
            );
        }
    }
    report
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn failed(ran: &[(String, Status)]) -> Vec<&str> {
    ran.iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

#[test]
fn generated_rust_and_go_fill_current_state() {
    // Rust.
    let root = tree(Target::Rust, "rust");
    let healthy = rust_build(&root, "healthy", str::to_owned);
    let ran = run(&healthy);
    assert_eq!(failed(&ran), Vec::<&str>::new(), "{ran:#?}");
    assert!(ran.iter().any(|(id, _)| id == SWAPPED), "{ran:#?}");
    let swapped = rust_build(&root, "swapped", |source| {
        replaced(source, RUST_CURRENT, RUST_REQUESTED_IN_CURRENT)
    });
    assert!(failed(&run(&swapped)).contains(&SWAPPED));
    let _ = std::fs::remove_dir_all(&root);

    // Go, where this machine has a Go toolchain.
    if Command::new("go").arg("version").output().is_err() {
        eprintln!("no Go toolchain on this machine; the Go held-state read is unchecked here");
        return;
    }
    let root = tree(Target::Go, "go");
    assert!(
        go(&root.join("demo"), &["vet", "./..."]).status.success(),
        "the generated Go is vet clean"
    );
    let healthy = go_build(&root, "healthy", str::to_owned);
    let ran = run(&healthy);
    assert_eq!(failed(&ran), Vec::<&str>::new(), "{ran:#?}");
    let swapped = go_build(&root, "swapped", |source| {
        replaced(source, GO_CURRENT, GO_REQUESTED_IN_CURRENT)
    });
    assert!(failed(&run(&swapped)).contains(&SWAPPED));
    let _ = std::fs::remove_dir_all(&root);
}

/// How the generated Rust fills `current` on a `wrong_state:` refusal, and the mutant that answers
/// the state `archive` would have entered instead.
const RUST_CURRENT: &str = "current: held_state";
const RUST_REQUESTED_IN_CURRENT: &str = "current: crate::docs::DocState::Archived";

/// The same in the generated Go.
const GO_CURRENT: &str = "Current: heldState";
const GO_REQUESTED_IN_CURRENT: &str = "Current: docs.DocStateArchived{}";

// ---- the adapter: the suite's requests, over the harness's line protocol -------------------------

struct Served {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ess_conformance::ObservedEvent>>,
    sequence: RefCell<u64>,
}

impl Served {
    fn start(binary: &Path, ir: &EssIr) -> Self {
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
        let mut child = Command::new(binary)
            .args(routes)
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

    fn ask(&self, request: &serde_json::Value) -> serde_json::Value {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"))
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

fn json_of(node: &Node) -> serde_json::Value {
    match node {
        Node::Null => serde_json::Value::Null,
        Node::Bool(value) => serde_json::Value::Bool(*value),
        Node::Number(number) => {
            serde_json::from_str(&number.to_string()).expect("a number's spelling is JSON")
        }
        Node::Text(text) => serde_json::Value::String(text.clone()),
        Node::Seq(items) => serde_json::Value::Array(items.iter().map(json_of).collect()),
        Node::Map(members) => serde_json::Value::Object(
            members
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        ),
    }
}

fn node_of(value: &serde_json::Value) -> Node {
    serde_json::from_value(value.clone()).expect("the wire writes JSON a node reads")
}

fn failure(observation: &str, answer: &serde_json::Value) -> ess_conformance::TargetError {
    ess_conformance::TargetError::unavailable(observation, answer.to_string())
}

impl ess_conformance::ConformanceTarget for Served {
    fn identity(
        &self,
    ) -> Result<ess_conformance::ImplementationIdentity, ess_conformance::TargetError> {
        Ok(ess_conformance::ImplementationIdentity::new(
            "subject-state-source",
            "1",
        ))
    }

    fn begin_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        let answer = self.ask(&serde_json::json!({"op": "reset"}));
        if answer["ok"] != serde_json::json!(true) {
            return Err(failure("opening a scenario", &answer));
        }
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
        Ok(())
    }

    fn end_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        self.published.borrow_mut().clear();
        Ok(())
    }

    fn execute_command(
        &self,
        request: ess_conformance::SemanticCommandRequest,
    ) -> Result<ess_conformance::SemanticCommandResult, ess_conformance::TargetError> {
        use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
        let observation = format!("invoking `{}`", request.command);
        let body = serde_json::Value::Object(
            request
                .input
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        );
        let answer = self.ask(&serde_json::json!({
            "op": "command",
            "command": request.command.to_string(),
            "actor": request.actor.as_ref().map(ToString::to_string),
            "body": body.to_string(),
        }));
        let status = answer["status"]
            .as_u64()
            .ok_or_else(|| failure(&observation, &answer))?;
        let served = &answer["answer"];
        if status == 501 {
            return Ok(ess_conformance::SemanticCommandResult::undeclared());
        }
        let outcome = served["outcome"]
            .as_str()
            .ok_or_else(|| failure(&observation, &answer))?;
        let outcome = OutcomeRef::new(
            CommandRef::new(request.command.name().clone()),
            outcome
                .parse()
                .map_err(|_| failure(&observation, &answer))?,
        );
        let error = match served.get("error") {
            Some(error) => {
                let name: ErrorRef = error
                    .as_str()
                    .and_then(|name| name.parse().ok())
                    .ok_or_else(|| failure(&observation, &answer))?;
                let mut declared = ess_conformance::DeclaredErrorValue::new(name);
                if let Some(serde_json::Value::Object(payload)) = served.get("payload") {
                    for (field, value) in payload {
                        if !value.is_null() {
                            declared = declared.with(field.clone(), node_of(value));
                        }
                    }
                }
                Some(declared)
            }
            None => None,
        };
        let mut direct_events = Vec::new();
        for published in served["published"].as_array().into_iter().flatten() {
            let reference: EventRef = published["event"]
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(|| failure(&observation, &answer))?;
            let mut event = ess_conformance::ObservedEvent::new(reference);
            if let Some(serde_json::Value::Object(payload)) = published.get("payload") {
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
            ess_primitives::consistency::ConsistencyToken::new(format!("seq:{}", self.tick()))
                .map_err(|why| {
                    ess_conformance::TargetError::unavailable(observation.clone(), why.to_string())
                })?,
        );
        Ok(ess_conformance::SemanticCommandResult {
            outcome: Some(outcome),
            error,
            consistency,
            direct_events,
            response: None,
        })
    }

    fn query_view(
        &self,
        request: ess_conformance::SemanticViewRequest,
    ) -> Result<ess_conformance::SemanticViewResult, ess_conformance::TargetError> {
        let observation = format!("reading `{}`", request.view);
        let answer = self.ask(&serde_json::json!({"op": "view", "view": request.view.to_string()}));
        let Some(rows) = answer["answer"]["rows"].as_array() else {
            return Err(failure(&observation, &answer));
        };
        Ok(ess_conformance::SemanticViewResult::of(rows.iter().map(
            |row| {
                row.as_object()
                    .into_iter()
                    .flatten()
                    .map(|(field, value)| (field.clone(), node_of(value)))
                    .collect::<ess_conformance::target::ViewRow>()
            },
        )))
    }

    fn observe_events(
        &self,
        request: ess_conformance::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::ObservedEvent>, ess_conformance::TargetError> {
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
        _: ess_conformance::ExternalOutcomeControl,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "forcing an outcome",
            "none is external",
        ))
    }

    fn redeliver_event(
        &self,
        _: ess_conformance::RedeliveryRequest,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "redelivering",
            "the model declares no binding",
        ))
    }
}
