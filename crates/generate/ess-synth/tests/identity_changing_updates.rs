//! An `updates:` whose `sets:` writes the identity re-keys the record (ess/23, beyond10x/ess#429,
//! `docs/design/identity-changing-updates.md`) in the code targets.
//!
//! Go, Web and Clap have no move of a record to a new identity, so each refuses the model by name
//! (`MissingRepresentation` at the identity write). The generated Rust behaviour looks the new
//! identity up through the entity's storage port for the declared collision refusal, then removes
//! the row under the old identity and inserts it under the new one. The test builds the generated
//! server beside `tests/fixtures/identity-changing-updates-harness/` (a storage port only, no
//! behaviour), runs the suite the same specification synthesizes against it, and runs it again
//! against mutants of the generated behaviour: one that ignores the write, one that keeps the old
//! row, and one that skips the collision lookup. Each fails a scenario.

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
use ess_synth::{synthesize_for, Target, TargetFailureCode};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/identity-changing-updates.yaml");

const RENAMED: &str = "demo.vault.RenameSecret/outcome/renamed";
const TAKEN: &str = "demo.vault.RenameSecret/outcome/taken";
const WRITE: &str = "commands.demo.vault.RenameSecret.outcomes.renamed.sets.name";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap();
    let spec = Specification::assemble([(Source::new("vault.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

// ---- Go, Web and Clap refuse the re-key by name --------------------------------------------------

#[test]
fn rename_targets_refuse_by_name() {
    let ir = ir();
    for target in [Target::Go, Target::Web, Target::Clap] {
        let Err(failure) = synthesize_for(&ir, target) else {
            panic!("{target:?} emits a re-key it cannot represent");
        };
        assert!(
            failure.causes().iter().any(|cause| {
                cause.code() == TargetFailureCode::MissingRepresentation
                    && cause.sources() == [WRITE]
                    && cause.detail().contains("identity")
            }),
            "{target:?}: {}",
            failure.to_canonical_json()
        );
    }
}

#[test]
fn the_rust_target_generates_the_rename_rather_than_owing_it() {
    let synthesis = synthesize_for(&ir(), Target::Rust).expect("the Rust target emits the model");
    let behaviour = synthesis
        .artifacts
        .iter()
        .find(|(path, _)| path.ends_with("behaviour.rs"))
        .map(|(_, artifact)| artifact.contents.clone())
        .expect("a generated behaviour");
    assert!(
        behaviour.contains("`demo.vault.RenameSecret`, generated: every outcome is one the specification fully determines."),
        "{behaviour}"
    );
    assert!(
        !behaviour.contains("RenameSecretBehavior> crate::"),
        "the command is not forwarded to the ports as an obligation: {behaviour}"
    );
    // The row is inserted under the new identity before the old one is removed, so a port that
    // fails between the two leaves the record held twice rather than not at all.
    let rename = behaviour
        .find("RenameSecretBehavior for Generated<P>")
        .expect("the rename impl");
    let body = &behaviour[rename..];
    let put = body.find("SecretStorage::put(&mut self.ports, next);");
    let delete = body.find("SecretStorage::delete(&mut self.ports, &input.name);");
    assert!(
        matches!((put, delete), (Some(put), Some(delete)) if put < delete),
        "put before delete in the rename: {body}"
    );
}

// ---- building the generated server ---------------------------------------------------------------

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

/// The synthesized model, written beside the harness, with `build` producing a harness binary from
/// the generated behaviour as `mutate` leaves it.
struct Workspace {
    root: PathBuf,
    harness: PathBuf,
    behaviour: PathBuf,
    original: String,
}

impl Workspace {
    fn new(label: &str) -> Self {
        let synthesis = synthesize_for(&ir(), Target::Rust).expect("the model synthesizes");
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("identity-429-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let tree = root.join("demo");
        for (relative, artifact) in &synthesis.artifacts {
            let destination = tree.join(relative);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::write(&destination, &artifact.contents).unwrap();
        }
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).unwrap();
        let dependencies = ["demo-server", "demo-system", "vault-service", "demo-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../demo/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"rename-harness\"\nversion = \"0.0.0\"\nedition = \
                 \"2021\"\npublish = false\n\n[workspace]\n\n[[bin]]\nname = \"harness\"\npath = \
                 \"src/main.rs\"\n\n[dependencies]\n{dependencies}"
            ),
        )
        .unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/identity-changing-updates-harness/main.rs"),
            harness.join("src/main.rs"),
        )
        .unwrap();
        let behaviour = tree.join("crates/demo-types/src/behaviour.rs");
        let original = std::fs::read_to_string(&behaviour).unwrap();
        Self {
            root,
            harness,
            behaviour,
            original,
        }
    }

    /// The harness built over the generated behaviour with `mutate` applied, kept as `name`.
    fn build(&self, name: &str, mutate: impl FnOnce(&str) -> String) -> PathBuf {
        let source = mutate(&self.original);
        std::fs::write(&self.behaviour, &source).unwrap();
        let target = self.root.join("target");
        assert!(
            cargo(&self.harness, &target).status.success(),
            "the harness builds over `{name}`"
        );
        let kept = self.root.join("kept");
        std::fs::create_dir_all(&kept).unwrap();
        std::fs::copy(target.join("debug/harness"), kept.join(name)).unwrap();
        kept.join(name)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

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

/// `source` with every line containing `needle` removed; at least one must.
fn without_lines(source: &str, needle: &str) -> String {
    assert!(
        source.contains(needle),
        "`{needle}` is in the generated behaviour"
    );
    source
        .lines()
        .filter(|line| !line.contains(needle))
        .fold(String::new(), |mut out, line| {
            let _ = writeln!(out, "{line}");
            out
        })
}

#[test]
fn generated_rust_executes_the_rename_and_every_mutant_fails_a_scenario() {
    let workspace = Workspace::new("rust");
    let healthy = workspace.build("healthy", str::to_owned);
    let ran = run(&healthy);
    assert_eq!(failed(&ran), Vec::<&str>::new(), "{ran:#?}");
    for id in [RENAMED, TAKEN] {
        assert!(ran.iter().any(|(ran, _)| ran == id), "{id} ran: {ran:#?}");
    }

    // Ignores the write: neither the identity is written nor the old row removed.
    let ignored = workspace.build("ignores-the-write", |source| {
        // The only write is the identity's, so the snapshot is no longer changed.
        without_lines(
            &without_lines(source, "next.data.name = "),
            "SecretStorage::delete(",
        )
        .replacen("let mut next = held;", "let next = held;", 1)
    });
    assert!(failed(&run(&ignored)).contains(&RENAMED));

    // Keeps the old row: the row is inserted under the new identity and the old one stays.
    let kept = workspace.build("keeps-the-old-row", |source| {
        without_lines(source, "SecretStorage::delete(")
    });
    assert!(failed(&run(&kept)).contains(&RENAMED));

    // Skips the collision lookup: a rename over another secret replaces it.
    let unguarded = workspace.build("skips-the-collision", |source| {
        let lookup = "&& SecretStorage::get(&self.ports, &input.new_name).is_some()";
        assert!(source.contains(lookup), "the collision lookup is generated");
        source.replacen(lookup, "&& false", 1)
    });
    assert_ne!(
        failed(&run(&unguarded)),
        Vec::<&str>::new(),
        "a rename onto a carried identity is answered"
    );
    assert!(failed(&run(&unguarded)).contains(&TAKEN));
}

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
            "identity-changing-updates",
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
