//! A model guarded by `when_related:` (story:related-guard-behaviour, beyond10x/ess#319), emitted
//! to Rust or Go, built as its generated network entry point with no hand-written code, and driven
//! over a real socket through the suite the same model synthesizes.
//!
//! The entry point stores in the generated ephemeral stores, so what the suite observes is the
//! generated behaviour reading the related row through its storage port. A faulty control edits
//! one line of the emitted behaviour before the build: the suite must fail it.

use std::collections::BTreeMap;
use std::io::{BufRead as _, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::Target;

/// One YAML document, compiled.
pub fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// `text` with one network component owning its one domain and accepting every command, where it
/// declares no component of its own.
pub fn served(text: &str, component: &str) -> String {
    if text.contains("\ncomponents:") {
        return text.to_owned();
    }
    let ir = model(text);
    let domain = text
        .lines()
        .find_map(|line| line.strip_prefix("domain: "))
        .expect("one domain")
        .trim();
    let commands: Vec<String> = ir.commands().keys().map(ToString::to_string).collect();
    let events: Vec<String> = ir.events().keys().map(ToString::to_string).collect();
    format!(
        "{}\ncomponents:\n  - component: {component}\n    reached_by: network\n    owns: \
         {{domains: [{domain}]}}\n    accepts: {{commands: [{}]}}\n    publishes: {{events: [{}]}}\n",
        text.trim_end(),
        commands.join(", "),
        events.join(", ")
    )
}

/// Where one emitted tree is written.
fn root(target: Target, case: &str) -> PathBuf {
    let label = match target {
        Target::Rust => "rust",
        Target::Go => "go",
        _ => unreachable!("the related-guard cases emit Rust and Go"),
    };
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "related-guard-{label}-{case}-{}",
        std::process::id()
    ))
}

/// Every artifact of `ir`'s `target` synthesis under a fresh directory, and that synthesis.
pub fn emit(ir: &EssIr, target: Target, case: &str) -> (PathBuf, ess_synth::Synthesis) {
    let synthesis = ess_synth::synthesize_for(ir, target).expect("the model synthesizes");
    let root = root(target, case);
    let _ = std::fs::remove_dir_all(&root);
    for artifact in synthesis.artifacts.values() {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
    (root, synthesis)
}

/// The emitted behaviour source of a tree.
pub fn behaviour_path(root: &Path, target: Target, system: &str) -> PathBuf {
    match target {
        Target::Rust => root.join(format!("crates/{system}-types/src/behaviour.rs")),
        Target::Go => root.join("types/behaviour/behaviour.go"),
        _ => unreachable!("the related-guard cases emit Rust and Go"),
    }
}

/// Replaces `from` with `to` in a file, asserting `from` is there: the faulty control's one edit.
pub fn mutate(path: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(path).expect("readable");
    assert!(
        text.contains(from),
        "the faulty control edits `{from}`, which {} carries:\n{text}",
        path.display()
    );
    std::fs::write(path, text.replacen(from, to, 1)).expect("write");
}

/// Builds the emitted entry point. Go is formatted, vetted and — with `race` — built with the race
/// detector, as the repository's Go server lane builds its examples.
pub fn build(root: &Path, target: Target, system: &str, component: &str, race: bool) -> PathBuf {
    let name = format!("{component}-server");
    match target {
        Target::Go => {
            let formatted = Command::new("gofmt")
                .args(["-l", "."])
                .current_dir(root)
                .output()
                .expect("gofmt runs");
            assert!(
                formatted.status.success() && formatted.stdout.is_empty(),
                "the generated Go is gofmt-clean:\n{}",
                String::from_utf8_lossy(&formatted.stdout)
            );
            let go = |arguments: &[&str], cgo: bool| {
                let output = Command::new("go")
                    .args(arguments)
                    .current_dir(root)
                    .env("GOWORK", "off")
                    .env("GOPROXY", "off")
                    .env("GOFLAGS", "-mod=mod")
                    .env("CGO_ENABLED", if cgo { "1" } else { "0" })
                    .output()
                    .expect("go runs");
                assert!(
                    output.status.success(),
                    "go {arguments:?} in {}:\n{}{}",
                    root.display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            };
            go(&["vet", "./..."], false);
            go(&["build", "./..."], false);
            let binary = root.join(&name);
            let output = binary.to_str().expect("a UTF-8 path").to_owned();
            let entry = format!("./cmd/{name}");
            if race {
                go(&["build", "-race", "-o", &output, &entry], true);
            } else {
                go(&["build", "-o", &output, &entry], false);
            }
            binary
        }
        Target::Rust => {
            static BUILDS: std::sync::Mutex<()> = std::sync::Mutex::new(());
            let _build = BUILDS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let target_dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("related-guard-rust-build-{}", std::process::id()));
            clear_generated_packages(root, &target_dir);
            let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies itself"))
                .args(["build", "--offline", "--bin", &name, "--target-dir"])
                .arg(&target_dir)
                .current_dir(root)
                .env_remove("CARGO_TARGET_DIR")
                .env_remove("CARGO_ENCODED_RUSTFLAGS")
                .env_remove("RUSTC_WRAPPER")
                .env("RUSTFLAGS", "-D warnings")
                .output()
                .expect("cargo runs");
            assert!(
                output.status.success(),
                "the generated {system} workspace builds with -D warnings:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let binary = root.join(&name);
            std::fs::copy(target_dir.join(format!("debug/{name}")), &binary).expect("copy");
            binary
        }
        _ => unreachable!("the related-guard cases emit Rust and Go"),
    }
}

/// Independently emitted workspaces reuse package names; drop their artifacts so Cargo never
/// mistakes another model's binary for this one, keeping the third-party dependencies.
fn clear_generated_packages(root: &Path, target_dir: &Path) {
    let cargo = std::env::var_os("CARGO").expect("Cargo supplies itself");
    let metadata = Command::new(&cargo)
        .args([
            "metadata",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(root)
        .output()
        .expect("cargo metadata runs");
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&metadata.stdout).expect("metadata JSON");
    let mut clean = Command::new(&cargo);
    clean
        .args(["clean", "--offline", "--target-dir"])
        .arg(target_dir)
        .current_dir(root);
    for package in metadata["packages"].as_array().expect("packages") {
        clean
            .arg("--package")
            .arg(package["name"].as_str().expect("a name"));
    }
    let cleaned = clean.output().expect("cargo clean runs");
    assert!(
        cleaned.status.success(),
        "{}",
        String::from_utf8_lossy(&cleaned.stderr)
    );
}

/// What each scenario of the suite `ir` synthesizes reported against the entry point `binary`.
pub fn run(ir: &EssIr, binary: &Path) -> BTreeMap<String, Status> {
    run_with_fixtures(ir, binary, None)
}

/// [`run`], with `fixtures` supplied as the independently provisioned values of every scenario
/// that resolves fixtures before it starts; `None` leaves the adapter without a provider.
pub fn run_with_fixtures(
    ir: &EssIr,
    binary: &Path,
    fixtures: Option<BTreeMap<String, ess_primitives::node::Node>>,
) -> BTreeMap<String, Status> {
    let suite = ess_conformance::synthesize(ir).suite;
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).expect("admitted");
    let mut adapter = HttpTarget::new(binary, ir);
    adapter.fixtures = fixtures;
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(&admitted, &adapter)
        .into_report();
    assert_eq!(report.scenarios.len(), suite.scenarios.len());
    report
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// Every scenario passed, at least one per named outcome among them.
pub fn assert_passes(label: &str, statuses: &BTreeMap<String, Status>, outcomes: &[&str]) {
    let failed: Vec<&String> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    assert!(
        failed.is_empty(),
        "{label}: {} of {} scenarios failed: {failed:#?}",
        failed.len(),
        statuses.len()
    );
    for outcome in outcomes {
        assert!(
            statuses.keys().any(|id| id.contains(outcome)),
            "{label}: a scenario witnesses `{outcome}`: {statuses:#?}"
        );
    }
    eprintln!("{label}: {} scenarios passed", statuses.len());
}

/// The faulty control failed at least one scenario.
pub fn assert_fails(label: &str, statuses: &BTreeMap<String, Status>) {
    let failed = statuses
        .values()
        .filter(|status| **status != Status::Passed)
        .count();
    assert!(
        failed > 0,
        "{label}: the faulty target passed every scenario: {statuses:#?}"
    );
    eprintln!(
        "{label}: the faulty target failed {failed} of {} scenarios",
        statuses.len()
    );
}

/// `true` where this machine has a Go toolchain.
pub fn go_available() -> bool {
    Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
}

struct Server {
    child: std::process::Child,
    address: String,
}

impl Server {
    fn start(binary: &Path) -> Self {
        let mut child = Command::new(binary)
            .args(["--listen", "127.0.0.1:0", "--callers", "actor-header"])
            .env("GORACE", "halt_on_error=1 exitcode=66")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("the entry point starts");
        let stdout = child.stdout.take().expect("piped");
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                    if value["event"] == "system.ready" {
                        let _ = sender.send(
                            value["runtime"]["address"]
                                .as_str()
                                .expect("an address")
                                .to_owned(),
                        );
                    }
                }
            }
        });
        let Ok(address) = receiver.recv_timeout(Duration::from_secs(20)) else {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the entry point did not become ready");
        };
        Self { child, address }
    }

    fn json(
        &self,
        method: &str,
        path: &str,
        actor: Option<&str>,
        body: &serde_json::Value,
    ) -> (u16, serde_json::Value) {
        let body = body.to_string();
        let mut stream = std::net::TcpStream::connect(&self.address).expect("connects");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("timeout");
        let actor = actor.map_or_else(String::new, |name| {
            format!("Authorization: Actor {name}\r\n")
        });
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\
             Content-Length: {}\r\n{actor}Connection: close\r\n\r\n{body}",
            body.len()
        )
        .expect("writes");
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).expect("reads");
        let boundary = bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("a head");
        let head = std::str::from_utf8(&bytes[..boundary]).expect("UTF-8 head");
        let status = head
            .split_whitespace()
            .nth(1)
            .and_then(|code| code.parse().ok())
            .expect("a status");
        let answer = serde_json::from_slice(&bytes[boundary + 4..]).unwrap_or_else(|error| {
            panic!(
                "{error}: {}",
                String::from_utf8_lossy(&bytes[boundary + 4..])
            )
        });
        (status, answer)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let status = self.child.wait();
        // A race the detector found exits with 66 before it is killed.
        if !std::thread::panicking() {
            assert!(
                !status.is_ok_and(|status| status.code() == Some(66)),
                "the race detector reported a data race"
            );
        }
    }
}

struct HttpTarget {
    binary: PathBuf,
    server: std::cell::RefCell<Server>,
    routes: BTreeMap<String, (String, String)>,
    events: std::cell::RefCell<Vec<ess_conformance::ObservedEvent>>,
    sequence: std::cell::Cell<u64>,
    fixtures: Option<BTreeMap<String, ess_primitives::node::Node>>,
}

impl HttpTarget {
    fn new(binary: &Path, ir: &EssIr) -> Self {
        let routes = ir
            .components()
            .values()
            .flat_map(|component| ess_gen::http::routes(ir, component))
            .map(|route| {
                let name = match route.serves {
                    ess_gen::http::Served::Command(command) => command.name().to_string(),
                    ess_gen::http::Served::View(view) => view.name().to_string(),
                };
                (name, (route.method.as_str().to_owned(), route.path))
            })
            .collect();
        Self {
            binary: binary.to_owned(),
            server: std::cell::RefCell::new(Server::start(binary)),
            routes,
            events: std::cell::RefCell::default(),
            sequence: std::cell::Cell::new(0),
            fixtures: None,
        }
    }

    fn tick(&self) -> u64 {
        let value = self.sequence.get() + 1;
        self.sequence.set(value);
        value
    }
}

fn json_of(node: &ess_primitives::node::Node) -> serde_json::Value {
    use ess_primitives::node::Node;
    match node {
        Node::Null => serde_json::Value::Null,
        Node::Bool(value) => serde_json::Value::Bool(*value),
        Node::Number(value) => serde_json::from_str(&value.to_string()).expect("a JSON number"),
        Node::Text(value) => serde_json::Value::String(value.clone()),
        Node::Seq(values) => serde_json::Value::Array(values.iter().map(json_of).collect()),
        Node::Map(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), json_of(value)))
                .collect(),
        ),
    }
}

impl ess_conformance::ConformanceTarget for HttpTarget {
    fn fixture_values(
        &self,
        _: &ess_conformance::ScenarioContext,
        _: &ess_conformance::fixtures::Contract,
    ) -> Result<BTreeMap<String, ess_primitives::node::Node>, ess_conformance::TargetError> {
        // Without values the adapter answers as a target with no provider does.
        self.fixtures.clone().ok_or_else(|| {
            ess_conformance::TargetError::unsupported(
                "fixture values",
                "no pre-execution fixture provider",
            )
        })
    }

    fn identity(
        &self,
    ) -> Result<ess_conformance::ImplementationIdentity, ess_conformance::TargetError> {
        Ok(ess_conformance::ImplementationIdentity::new(
            "generated-related-guard-entry",
            "1",
        ))
    }

    fn begin_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        // A fresh process, so a fresh ephemeral store: no reset route, no injected store.
        *self.server.borrow_mut() = Server::start(&self.binary);
        self.events.borrow_mut().clear();
        self.sequence.set(0);
        Ok(())
    }

    fn end_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        Ok(())
    }

    fn execute_command(
        &self,
        request: ess_conformance::SemanticCommandRequest,
    ) -> Result<ess_conformance::SemanticCommandResult, ess_conformance::TargetError> {
        use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
        let (method, path) = &self.routes[&request.command.to_string()];
        let body = serde_json::Value::Object(
            request
                .input
                .iter()
                .map(|(key, value)| (key.clone(), json_of(value)))
                .collect(),
        );
        let actor = request.actor.as_ref().map(ToString::to_string);
        let (status, answer) = self
            .server
            .borrow()
            .json(method, path, actor.as_deref(), &body);
        if status == 403 && answer["refused"] == "not granted" {
            return Err(ess_conformance::TargetError::not_granted(
                answer["actor"].as_str(),
            ));
        }
        if status == 501 {
            return Ok(ess_conformance::SemanticCommandResult::undeclared());
        }
        let failure = || {
            ess_conformance::TargetError::unavailable("command HTTP response", answer.to_string())
        };
        let outcome = answer["outcome"]
            .as_str()
            .ok_or_else(failure)?
            .parse()
            .map_err(|_| failure())?;
        let outcome = OutcomeRef::new(CommandRef::new(request.command.name().clone()), outcome);
        let error = if let Some(error) = answer.get("error") {
            let reference: ErrorRef = error
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(failure)?;
            let mut declared = ess_conformance::DeclaredErrorValue::new(reference);
            for (key, value) in answer
                .get("payload")
                .and_then(serde_json::Value::as_object)
                .into_iter()
                .flatten()
            {
                if !value.is_null() {
                    declared = declared.with(
                        key.clone(),
                        serde_json::from_value(value.clone()).expect("a value"),
                    );
                }
            }
            Some(declared)
        } else {
            None
        };
        let mut direct_events = Vec::new();
        for published in answer["published"].as_array().into_iter().flatten() {
            let reference: EventRef = published["event"]
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(failure)?;
            let mut event = ess_conformance::ObservedEvent::new(reference);
            for (key, value) in published["payload"].as_object().into_iter().flatten() {
                event = event.with(
                    key.clone(),
                    serde_json::from_value(value.clone()).expect("a value"),
                );
            }
            let event = event
                .in_activity(request.correlation.clone())
                .at(self.tick());
            self.events.borrow_mut().push(event.clone());
            direct_events.push(event);
        }
        Ok(ess_conformance::SemanticCommandResult {
            outcome: Some(outcome),
            error,
            consistency: Some(
                ess_primitives::consistency::ConsistencyToken::new(format!("http:{}", self.tick()))
                    .expect("a token"),
            ),
            direct_events,
            response: None,
        })
    }

    fn query_view(
        &self,
        request: ess_conformance::SemanticViewRequest,
    ) -> Result<ess_conformance::SemanticViewResult, ess_conformance::TargetError> {
        let (method, path) = &self.routes[&request.view.to_string()];
        let (status, answer) =
            self.server
                .borrow()
                .json(method, path, None, &serde_json::Value::Null);
        let Some(rows) = answer["rows"].as_array().filter(|_| status == 200) else {
            return Err(ess_conformance::TargetError::unavailable(
                "view HTTP response",
                answer.to_string(),
            ));
        };
        Ok(ess_conformance::SemanticViewResult::of(rows.iter().map(
            |row| {
                row.as_object()
                    .expect("a row object")
                    .iter()
                    .map(|(key, value)| {
                        (
                            key.clone(),
                            serde_json::from_value(value.clone()).expect("a value"),
                        )
                    })
                    .collect::<ess_conformance::target::ViewRow>()
            },
        )))
    }

    fn observe_events(
        &self,
        request: ess_conformance::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::ObservedEvent>, ess_conformance::TargetError> {
        Ok(self
            .events
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
            "external control",
            "the related-guard fixtures declare no external outcome",
        ))
    }

    fn redeliver_event(
        &self,
        _: ess_conformance::RedeliveryRequest,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "redelivery",
            "the related-guard fixtures declare no binding",
        ))
    }
}
