//! A served surface enforces the read grants an actor's `may:` states (beyond10x/ess#286).
//!
//! From source `ess/22` a view an actor's `may:` names is read-granted: the generated Rust and Go
//! servers answer it only to a caller authenticated as an actor naming it, and the standard
//! refusal — `403` `{"refused": "not granted", "actor": …}`, the body a command answers an
//! ungranted actor — to anyone else, before the view is read. A view no actor names stays open.
//! Both generated executables are built and run over HTTP, authenticated with their
//! `--callers actor-header` demonstration mode, and the synthesized `…/grant/read/…` scenarios
//! run against them: each generated server passes them, and the same server with its read check
//! taken out fails the denied one.

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

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("parses");
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

/// The synthesized workspace, written under this test process's scratch directory.
fn emitted(ir: &EssIr, target: Target, case: &str) -> PathBuf {
    let synthesis = ess_synth::synthesize_laid_out(ir, target, ess_synth::OutputLayout::Workspace)
        .expect("the model synthesizes");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "view-grants-{}-{case}-{}",
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

const SERVER: &str = "desk-service-server";

/// Builds the generated executable at `root` and copies it to `kept`.
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
            .env("RUSTFLAGS", "-D warnings")
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

/// The generated Go tree, unmutated, is `gofmt`-clean and passes `go vet`, as `served_entry` holds
/// every generated Go tree.
fn go_is_formatted_and_vetted(root: &Path) {
    let formatted = Command::new("gofmt")
        .args(["-l", "."])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        formatted.status.success() && formatted.stdout.is_empty(),
        "generated Go needs formatting:\n{}",
        String::from_utf8_lossy(&formatted.stdout)
    );
    let vetted = Command::new("go")
        .args(["vet", "./..."])
        .current_dir(root)
        .env("GOWORK", "off")
        .env("GOPROXY", "off")
        .env("GOFLAGS", "-mod=mod")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap();
    assert!(
        vetted.status.success(),
        "{}",
        String::from_utf8_lossy(&vetted.stderr)
    );
}

/// The generated server, and the same server with its read check admitting every caller.
struct Built {
    served: PathBuf,
    mutant: PathBuf,
}

/// The read check, as each target emits it, and the same check admitting every caller.
fn mutation(target: Target) -> (&'static str, &'static str, &'static str) {
    match target {
        Target::Rust => (
            "crates/desk-server/src/desk_service.rs",
            "    match caller {\n        Some(caller) if caller.may_read(view) => Ok(()),\n",
            "    let _ = (caller, view);\n    if true {\n        return Ok(());\n    }\n    match \
             caller {\n        Some(caller) if caller.may_read(view) => Ok(()),\n",
        ),
        Target::Go => (
            "server/server.go",
            "func admitRead(caller *Caller, view string) *response {\n",
            "func admitRead(caller *Caller, view string) *response {\n\tif true {\n\t\treturn \
             nil\n\t}\n",
        ),
        _ => unreachable!(),
    }
}

fn built(target: Target) -> Built {
    let root = emitted(&model(MODEL), target, "served");
    let kept = root.with_extension("kept");
    std::fs::create_dir_all(&kept).unwrap();
    if target == Target::Go {
        go_is_formatted_and_vetted(&root);
    }
    let served = kept.join("served");
    build(&root, target, &served);
    let (file, check, admitting) = mutation(target);
    let surface = root.join(file);
    let source = std::fs::read_to_string(&surface)
        .unwrap_or_else(|error| panic!("{}: {error}", surface.display()));
    assert_eq!(
        source.matches(check).count(),
        1,
        "one read check:\n{source}"
    );
    std::fs::write(&surface, source.replace(check, admitting)).unwrap();
    let mutant = kept.join("mutant");
    build(&root, target, &mutant);
    Built { served, mutant }
}

/// One running generated server, authenticating `Authorization: Actor <name>`.
struct Server {
    child: std::process::Child,
    address: String,
}

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

    fn request(
        &self,
        method: &str,
        path: &str,
        actor: Option<&str>,
        body: &str,
    ) -> (u16, serde_json::Value) {
        let mut stream = std::net::TcpStream::connect(&self.address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let actor = actor.map_or_else(String::new, |name| {
            format!("Authorization: Actor {name}\r\n")
        });
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: \
             application/json\r\nContent-Length: {}\r\n{actor}Connection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        let boundary = bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap();
        let head = std::str::from_utf8(&bytes[..boundary]).unwrap();
        let status = head.split_whitespace().nth(1).unwrap().parse().unwrap();
        let body = &bytes[boundary + 4..];
        let value = if body.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(body)
                .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(body)))
        };
        (status, value)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Every route, by the qualified name it serves: `(name, method, path)`.
fn routes(ir: &EssIr) -> Vec<(String, String, String)> {
    let component = ir.components().values().next().expect("one component");
    ess_gen::http::routes(ir, component)
        .into_iter()
        .map(|route| {
            let name = match route.serves {
                ess_gen::http::Served::Command(handle) => ir.command(handle).name.to_string(),
                ess_gen::http::Served::View(handle) => ir.view(handle).name.to_string(),
            };
            (name, route.method.as_str().to_owned(), route.path)
        })
        .collect()
}

fn path_of(ir: &EssIr, name: &str) -> String {
    routes(ir)
        .into_iter()
        .find(|(served, _, _)| served == name)
        .unwrap_or_else(|| panic!("`{name}` is served"))
        .2
}

fn answers_reads_as_granted(binary: &Path) {
    let ir = model(MODEL);
    let server = Server::start(binary);
    let board = path_of(&ir, "desk.tickets.Board");
    let titles = path_of(&ir, "desk.tickets.Titles");
    let opened = server.request(
        "POST",
        &path_of(&ir, "desk.tickets.OpenTicket"),
        Some("desk.tickets.Watcher"),
        r#"{"id":"t-1","title":"printer"}"#,
    );
    assert_eq!(opened.0, 202, "{opened:?}");
    let (status, body) = server.request("GET", &board, Some("desk.tickets.Clerk"), "");
    assert_eq!(status, 200, "the Clerk's grant names the Board: {body}");
    assert_eq!(
        body["rows"],
        serde_json::json!([{"id": "t-1", "title": "printer"}])
    );
    assert_eq!(
        server.request("GET", &board, Some("desk.tickets.Watcher"), ""),
        (
            403,
            serde_json::json!({"refused": "not granted", "actor": "desk.tickets.Watcher"})
        ),
        "the Watcher's grant does not name the Board"
    );
    assert_eq!(
        server.request("GET", &board, None, ""),
        (
            403,
            serde_json::json!({"refused": "not granted", "actor": null})
        ),
        "no caller is no actor the grant names"
    );
    for actor in [
        Some("desk.tickets.Watcher"),
        Some("desk.tickets.Clerk"),
        None,
    ] {
        let (status, body) = server.request("GET", &titles, actor, "");
        assert_eq!(status, 200, "no grant names Titles, so it is open: {body}");
        assert_eq!(body["rows"], serde_json::json!([{"title": "printer"}]));
    }
}

/// The synthesized read-grant scenarios, over the generated server's HTTP surface: the actor a
/// `ReadAs` names is the caller the request is authenticated as.
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
                        serde_json::Value::Bool(value) => Node::Bool(*value),
                        serde_json::Value::Null => Node::Null,
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
        let reader = reader.map(ToString::to_string);
        let (status, body) = self
            .running
            .borrow()
            .as_ref()
            .expect("a scenario is open")
            .request("GET", &path, reader.as_deref(), "");
        if status == 403 && body["refused"] == "not granted" {
            return Err(TargetError::not_granted(body["actor"].as_str()));
        }
        assert_eq!(status, 200, "{body}");
        Ok(SemanticViewResult::of(rows(&body)))
    }
}

impl ConformanceTarget for Served {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("view-grants-served", "1"))
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

/// The verdict of every read-grant scenario against `binary`.
fn run_read_grants(binary: &Path) -> Vec<(String, Status)> {
    let ir = model(MODEL);
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

fn the_server_enforces_read_grants_and_its_mutant_fails_the_denied_read(target: Target) {
    let built = built(target);
    answers_reads_as_granted(&built.served);
    let admitted = "desk.tickets.Board/grant/read/admitted/desk.tickets.Clerk".to_owned();
    let denied = "desk.tickets.Board/grant/read/denied".to_owned();
    assert_eq!(
        run_read_grants(&built.served),
        [
            (admitted.clone(), Status::Passed),
            (denied.clone(), Status::Passed)
        ],
        "{target:?}"
    );
    assert_eq!(
        run_read_grants(&built.mutant),
        [(admitted, Status::Passed), (denied, Status::Failed)],
        "{target:?}: a server that checks no read grant fails the denied read"
    );
}

#[test]
fn the_rust_server_enforces_read_grants_and_its_mutant_fails_the_denied_read() {
    the_server_enforces_read_grants_and_its_mutant_fails_the_denied_read(Target::Rust);
}

#[test]
fn the_go_server_enforces_read_grants_and_its_mutant_fails_the_denied_read() {
    the_server_enforces_read_grants_and_its_mutant_fails_the_denied_read(Target::Go);
}

#[test]
fn a_model_that_names_no_view_keeps_its_generated_bytes() {
    let granted = model(MODEL);
    let open = model(&MODEL.replace("      - desk.tickets.Board\n", ""));
    for target in [Target::Rust, Target::Go] {
        let synthesis = ess_synth::synthesize_for(&open, target).expect("synthesizes");
        for (path, artifact) in &synthesis.artifacts {
            for read in [
                "may_read",
                "admit_read",
                "MayRead",
                "admitRead",
                "readGrants",
            ] {
                assert!(
                    !artifact.contents.contains(read),
                    "{target:?} `{path}` mentions `{read}` for a model naming no view"
                );
            }
        }
        let synthesis = ess_synth::synthesize_for(&granted, target).expect("synthesizes");
        assert!(
            synthesis
                .artifacts
                .values()
                .any(|artifact| artifact.contents.contains(match target {
                    Target::Rust => "admit_read",
                    _ => "admitRead",
                })),
            "{target:?}"
        );
    }
}
