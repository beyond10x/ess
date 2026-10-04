//! Security review of beyond10x/ess#286: the generated Rust and Go servers on every way a request
//! can reach a read-granted view, and what the synthesized read-grant scenarios measure of them.
//!
//! The model is `docs/design/view-grants.example.yaml`: `desk.tickets.Board` is read-granted to
//! `desk.tickets.Clerk`; `desk.tickets.Watcher` may not read it; `desk.tickets.Titles` is open.
//! Each generated executable is built and run with its `--callers actor-header` mode, which
//! authenticates `Authorization: Actor <name>`.

use std::io::{BufRead as _, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::ActorRef;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError, ViewRow,
};
use ess_conformance::{AdmittedSuite, Runner, ScenarioId};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_synth::Target;

const MODEL: &str = include_str!("../../../../docs/design/view-grants.example.yaml");
const SERVER: &str = "desk-service-server";
const CLERK: &str = "desk.tickets.Clerk";
const WATCHER: &str = "desk.tickets.Watcher";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("parses");
    let spec = Specification::assemble([(Source::new("view-grants.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn label(target: Target) -> &'static str {
    match target {
        Target::Rust => "rust",
        Target::Go => "go",
        _ => unreachable!(),
    }
}

fn emitted(ir: &EssIr, target: Target, case: &str) -> PathBuf {
    let synthesis = ess_synth::synthesize_laid_out(ir, target, ess_synth::OutputLayout::Workspace)
        .expect("the model synthesizes");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "view-grant-security-{}-{case}-{}",
        label(target),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    for (relative, artifact) in synthesis.artifacts {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    root
}

fn build(root: &Path, target: Target, kept: &Path) {
    let output = match target {
        Target::Rust => Command::new(std::env::var_os("CARGO").unwrap())
            .args([
                "build",
                "--offline",
                "--quiet",
                "--bin",
                SERVER,
                "--target-dir",
            ])
            .arg(root.join("target"))
            .current_dir(root)
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("RUSTC_WRAPPER")
            .output()
            .unwrap(),
        Target::Go => Command::new("go")
            .args(["build", "-o"])
            .arg(kept)
            .arg(format!("./cmd/{SERVER}"))
            .current_dir(root)
            .env("GOWORK", "off")
            .env("GOPROXY", "off")
            .env("GOFLAGS", "-mod=mod")
            .env("GOTOOLCHAIN", "local")
            .output()
            .unwrap(),
        _ => unreachable!(),
    };
    assert!(
        output.status.success(),
        "{}: {}",
        root.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    if target == Target::Rust {
        std::fs::copy(root.join(format!("target/debug/{SERVER}")), kept).unwrap();
    }
}

/// The generated server as emitted, and the same server with its read check's no-actor arm
/// admitting the read ([`admits_no_actor`]). Built once per target and shared by every test.
struct Built {
    served: PathBuf,
    mutant: PathBuf,
}

fn build_both(target: Target) -> Built {
    let root = emitted(&model(), target, "served");
    let kept = root.with_extension("kept");
    std::fs::create_dir_all(&kept).unwrap();
    let served = kept.join("served");
    build(&root, target, &served);
    let (file, from, to) = admits_no_actor(target);
    let surface = root.join(file);
    let source = std::fs::read_to_string(&surface)
        .unwrap_or_else(|error| panic!("{}: {error}", surface.display()));
    assert_eq!(source.matches(from).count(), 1, "one site:\n{source}");
    std::fs::write(&surface, source.replace(from, to)).unwrap();
    let mutant = kept.join("mutant");
    build(&root, target, &mutant);
    Built { served, mutant }
}

fn built(target: Target) -> &'static Built {
    static RUST: std::sync::OnceLock<Built> = std::sync::OnceLock::new();
    static GO: std::sync::OnceLock<Built> = std::sync::OnceLock::new();
    match target {
        Target::Rust => RUST.get_or_init(|| build_both(Target::Rust)),
        Target::Go => GO.get_or_init(|| build_both(Target::Go)),
        _ => unreachable!(),
    }
}

struct Server {
    child: std::process::Child,
    address: String,
}

/// A response's status and body bytes.
type Answer = (u16, String);

impl Server {
    fn start(binary: &Path) -> Self {
        let mut child = Command::new(binary)
            .args(["--listen", "127.0.0.1:0", "--callers", "actor-header"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                    if value["event"] == "system.ready" {
                        let _ =
                            sender.send(value["runtime"]["address"].as_str().unwrap().to_owned());
                    }
                }
            }
        });
        let address = receiver
            .recv_timeout(Duration::from_secs(10))
            .unwrap_or_else(|error| {
                let _ = child.kill();
                panic!("the server did not become ready: {error}")
            });
        Self { child, address }
    }

    /// One request, `headers` written verbatim as header lines.
    fn send(&self, method: &str, path: &str, headers: &[&str], body: &str) -> Answer {
        let mut stream = std::net::TcpStream::connect(&self.address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut head = String::new();
        for header in headers {
            head.push_str(header);
            head.push_str("\r\n");
        }
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: \
             application/json\r\nContent-Length: {}\r\n{head}Connection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut bytes = Vec::new();
        let _ = stream.read_to_end(&mut bytes);
        let boundary = bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap_or_else(|| panic!("no head: {}", String::from_utf8_lossy(&bytes)));
        let status_line = std::str::from_utf8(&bytes[..boundary]).unwrap();
        let status = status_line
            .split_whitespace()
            .nth(1)
            .unwrap()
            .parse()
            .unwrap();
        (
            status,
            String::from_utf8_lossy(&bytes[boundary + 4..]).into_owned(),
        )
    }

    fn get_as(&self, path: &str, headers: &[&str]) -> Answer {
        self.send("GET", path, headers, "")
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn path_of(ir: &EssIr, name: &str) -> String {
    let component = ir.components().values().next().expect("one component");
    ess_gen::http::routes(ir, component)
        .into_iter()
        .find(|route| match route.serves {
            ess_gen::http::Served::Command(handle) => ir.command(handle).name.to_string() == name,
            ess_gen::http::Served::View(handle) => ir.view(handle).name.to_string() == name,
        })
        .unwrap_or_else(|| panic!("`{name}` is served"))
        .path
}

fn actor(name: &str) -> String {
    format!("Authorization: Actor {name}")
}

/// A server with one ticket opened, and the Board's path.
fn seeded(binary: &Path) -> (Server, String) {
    let ir = model();
    let server = Server::start(binary);
    let opened = server.send(
        "POST",
        &path_of(&ir, "desk.tickets.OpenTicket"),
        &[&actor(WATCHER)],
        r#"{"id":"t-1","title":"printer"}"#,
    );
    assert_eq!(opened.0, 202, "{opened:?}");
    (server, path_of(&ir, "desk.tickets.Board"))
}

/// Invariant 2: a request carrying two actor credentials is not granted the Board, in either
/// order. The demonstration authentication reads the first `Authorization` header only.
fn two_actor_headers_are_not_granted(binary: &Path, target: Target) {
    let (server, board) = seeded(binary);
    let clerk = actor(CLERK);
    let watcher = actor(WATCHER);
    let mut granted = Vec::new();
    for (order, headers) in [
        ("Clerk, Watcher", [clerk.as_str(), watcher.as_str()]),
        ("Watcher, Clerk", [watcher.as_str(), clerk.as_str()]),
    ] {
        let answer = server.get_as(&board, &headers);
        if answer.0 != 403 {
            granted.push(format!("{order}: {answer:?}"));
        }
    }
    assert_eq!(
        granted,
        Vec::<String>::new(),
        "{target:?}: a request carrying two `Authorization: Actor` headers was answered the \
         read-granted Board"
    );
}

/// Invariant 2: malformed, unknown and absent credentials are refused the Board with the standard
/// refusal naming no actor; the documented forms are admitted.
fn malformed_and_unknown_actors_are_refused(binary: &Path, target: Target) {
    let (server, board) = seeded(binary);
    let refused = (
        403,
        serde_json::json!({"refused": "not granted", "actor": null}),
    );
    for headers in [
        vec![],
        vec!["Authorization: Actor"],
        vec!["Authorization: Actor "],
        vec!["Authorization: actor desk.tickets.Clerk"],
        vec!["Authorization: Actor  desk.tickets.Clerk"],
        vec!["Authorization: Bearer desk.tickets.Clerk"],
        vec!["Authorization: Actor desk.tickets.Clerk,desk.tickets.Watcher"],
        vec!["Authorization: Actor desk.tickets.Nobody"],
        vec!["Authorization: Actor desk.tickets.clerk"],
        vec!["Authorization: Actor desk.tickets.Board"],
        vec!["X-Actor: desk.tickets.Clerk"],
    ] {
        let (status, body) = server.get_as(&board, &headers);
        let body: serde_json::Value = serde_json::from_str(&body)
            .unwrap_or_else(|error| panic!("{target:?} {headers:?}: {error}: {body}"));
        assert_eq!((status, body), refused, "{target:?} {headers:?}");
    }
    for headers in [
        vec!["Authorization: Actor desk.tickets.Clerk"],
        vec!["Authorization: Actor Clerk"],
        vec!["authorization: Actor desk.tickets.Clerk "],
    ] {
        let (status, body) = server.get_as(&board, &headers);
        assert_eq!(status, 200, "{target:?} {headers:?}: {body}");
    }
}

/// Invariant 1: every method other than the route's own, and a query string, tells a refused
/// caller nothing a granted one is not told, and never a row.
fn other_methods_and_query_strings_leak_nothing(binary: &Path, target: Target) {
    let (server, board) = seeded(binary);
    let callers: [Vec<String>; 3] = [vec![actor(CLERK)], vec![actor(WATCHER)], vec![]];
    for method in ["HEAD", "OPTIONS", "POST", "PUT", "DELETE", "PATCH"] {
        let answers: Vec<Answer> = callers
            .iter()
            .map(|headers| {
                let headers: Vec<&str> = headers.iter().map(String::as_str).collect();
                server.send(method, &board, &headers, "")
            })
            .collect();
        assert!(
            answers.iter().all(|answer| answer == &answers[0]),
            "{target:?} {method}: {answers:?}"
        );
        assert!(
            !answers[0].1.contains("printer"),
            "{target:?} {method}: {answers:?}"
        );
    }
    for suffix in ["?", "?x=1", "?param.id=t-1"] {
        let path = format!("{board}{suffix}");
        let (status, body) = server.get_as(&path, &[&actor(WATCHER)]);
        assert_eq!(status, 403, "{target:?} {path}: {body}");
        assert!(!body.contains("printer"), "{target:?} {path}: {body}");
        let (status, body) = server.get_as(&path, &[]);
        assert_eq!(status, 403, "{target:?} {path}: {body}");
    }
}

#[test]
fn the_rust_server_refuses_two_actor_headers() {
    two_actor_headers_are_not_granted(&built(Target::Rust).served, Target::Rust);
}

#[test]
fn the_go_server_refuses_two_actor_headers() {
    two_actor_headers_are_not_granted(&built(Target::Go).served, Target::Go);
}

#[test]
fn the_rust_server_refuses_malformed_unknown_and_absent_actors_and_leaks_nothing() {
    malformed_and_unknown_actors_are_refused(&built(Target::Rust).served, Target::Rust);
    other_methods_and_query_strings_leak_nothing(&built(Target::Rust).served, Target::Rust);
}

#[test]
fn the_go_server_refuses_malformed_unknown_and_absent_actors_and_leaks_nothing() {
    malformed_and_unknown_actors_are_refused(&built(Target::Go).served, Target::Go);
    other_methods_and_query_strings_leak_nothing(&built(Target::Go).served, Target::Go);
}

// ---- what the synthesized scenarios measure ---------------------------------------------------

/// The synthesized read-grant scenarios, over a running server's HTTP surface.
struct Served {
    binary: PathBuf,
    ir: EssIr,
    running: std::cell::RefCell<Option<Server>>,
}

fn rows(body: &serde_json::Value) -> Vec<ViewRow> {
    body["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|row| {
            row.as_object()
                .expect("a row is an object")
                .iter()
                .map(|(name, value)| {
                    let node = match value {
                        serde_json::Value::String(text) => Node::Text(text.clone()),
                        other => panic!("the fixture projects text only: {other}"),
                    };
                    (name.clone(), node)
                })
                .collect()
        })
        .collect()
}

impl Served {
    fn read(
        &self,
        request: &SemanticViewRequest,
        reader: Option<&ActorRef>,
    ) -> Result<SemanticViewResult, TargetError> {
        let path = path_of(&self.ir, &request.view.to_string());
        let header = reader.map(|reader| actor(&reader.to_string()));
        let headers: Vec<&str> = header.iter().map(String::as_str).collect();
        let (status, body) = self
            .running
            .borrow()
            .as_ref()
            .expect("a scenario is open")
            .get_as(&path, &headers);
        let body: serde_json::Value = serde_json::from_str(&body).unwrap();
        if status == 403 && body["refused"] == "not granted" {
            return Err(TargetError::not_granted(body["actor"].as_str()));
        }
        assert_eq!(status, 200, "{body}");
        Ok(SemanticViewResult::of(rows(&body)))
    }
}

impl ConformanceTarget for Served {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("view-grant-security", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.running.borrow_mut() = Some(Server::start(&self.binary));
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.running.borrow_mut() = None;
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unsupported(
            format!("invoking `{}`", request.command),
            "only reads are sent",
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.read(&request, None)
    }
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        self.read(&request, Some(reader))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "forcing an outcome",
            "none is external",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivering", "not needed"))
    }
}

/// Every read-grant scenario's verdict against `binary`.
fn run_read_grants(binary: &Path) -> Vec<(String, Status)> {
    let ir = model();
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite.scenarios.retain(|id, _| {
        matches!(
            id,
            ScenarioId::ReadGrant { .. } | ScenarioId::ReadGrantAdmitted { .. }
        )
    });
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let target = Served {
        binary: binary.to_owned(),
        ir,
        running: std::cell::RefCell::new(None),
    };
    Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// The read check's no-actor arm, as each target emits it, and the same arm admitting the read.
fn admits_no_actor(target: Target) -> (&'static str, &'static str, &'static str) {
    match target {
        Target::Rust => (
            "crates/desk-server/src/desk_service.rs",
            "        Some(caller) if caller.may_read(view) => Ok(()),\n        Some(caller) => \
             Err(Some(caller.actor.name())),\n        None => Err(None),\n",
            "        Some(caller) if caller.may_read(view) => Ok(()),\n        Some(caller) => \
             Err(Some(caller.actor.name())),\n        None => Ok(()),\n",
        ),
        Target::Go => (
            "server/server.go",
            "func AdmitRead(caller *Caller, view string) (bool, Actor) {\n\tif caller == nil \
             {\n\t\treturn false, \"\"\n",
            "func AdmitRead(caller *Caller, view string) (bool, Actor) {\n\tif caller == nil \
             {\n\t\treturn true, \"\"\n",
        ),
        _ => unreachable!(),
    }
}

/// Invariant 2 as the suite measures it: a server whose read check admits a request
/// authenticated as no actor serves the Board to anyone without a credential, and some
/// synthesized read-grant scenario has to fail it.
fn the_no_actor_mutant_fails_some_read_grant_scenario(target: Target) {
    let Built { served, mutant } = built(target);
    {
        let (server, board) = seeded(served);
        assert_eq!(server.get_as(&board, &[]).0, 403, "{target:?}: control");
        let (server, board) = seeded(mutant);
        assert_eq!(
            server.get_as(&board, &[]).0,
            200,
            "{target:?}: the mutation took: the mutant serves the Board to no actor"
        );
    }
    let verdicts = run_read_grants(mutant);
    assert!(
        verdicts.iter().any(|(_, status)| *status != Status::Passed),
        "{target:?}: a server serving the read-granted Board to a request with no actor passes \
         every synthesized read-grant scenario: {verdicts:?}"
    );
}

#[test]
fn the_rust_no_actor_mutant_fails_some_read_grant_scenario() {
    the_no_actor_mutant_fails_some_read_grant_scenario(Target::Rust);
}

#[test]
fn the_go_no_actor_mutant_fails_some_read_grant_scenario() {
    the_no_actor_mutant_fails_some_read_grant_scenario(Target::Go);
}
