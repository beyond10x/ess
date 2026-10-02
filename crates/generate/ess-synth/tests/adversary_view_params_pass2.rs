//! Adversary pass 2 on `story:served-view-params`, correction 1: how the Rust server's single
//! accept loop survives callers that hang up or stall, a head the Go server reads and the Rust
//! one refuses, and the command-body values and refusals the correction says the two servers now
//! answer alike.
//!
//! One small model, synthesized for both serving targets, built, started on an ephemeral port,
//! asked over a raw socket and killed by its PID.

use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

use ess_compiler::EssIr;
use ess_gen::http::{self, Served};
use ess_synth::{synthesize_laid_out, OutputLayout, Synthesis, Target};

const MODEL: &str = r#"format: ess/18
system: desk
version: v1
domain: desk.work
entities:
  - name: desk.work.Task
    identity: {name: task_id, type: String}
    fields:
      - {name: owner, type: String}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Open], to: Done}
events:
  - name: desk.work.TaskFinished
    fields:
      - {name: task_id, type: String}
errors:
  - name: desk.work.Late
    fields:
      - {name: reason, type: String}
commands:
  - name: desk.work.FinishTask
    input:
      - {name: task_id, type: String}
      - {name: blob, type: Bytes}
      - {name: counts, type: "Map<Integer, String>"}
      - {name: flags, type: "Map<Boolean, String>"}
    outcomes:
      - name: finished
        moves: desk.work.Task.finish
        instance: task_id
        emits: [desk.work.TaskFinished]
        payload:
          desk.work.TaskFinished: {task_id: input.task_id}
      - name: late
        external: the task is past due
        error: desk.work.Late
views:
  - name: desk.work.Probe
    source: desk.work.Task
    consistency: eventual
    params:
      - {name: owner, type: String}
    filter: [owner == param.owner]
    fields:
      - {name: task_id, type: String}
      - {name: owner, type: String}
components:
  - component: desk-service
    owns: {domains: [desk.work]}
    accepts: {commands: [desk.work.FinishTask]}
    publishes: {events: [desk.work.TaskFinished]}
    reached_by: network
"#;

const RUST_HARNESS: &str = r#"use desk_types::obligation::UnmetObligation;
use desk_types::work;

struct Desk;

impl work::obligations::FinishTaskBehavior for Desk {
    fn finish_task(
        &mut self,
        _input: work::FinishTask,
    ) -> Result<work::FinishTaskOutcome, UnmetObligation> {
        Err(UnmetObligation { capability: "command behaviour", source: "desk.work.FinishTask" })
    }
}

impl work::obligations::ProbeQuery for Desk {
    fn probe(&self, owner: String) -> Result<Vec<work::Probe>, UnmetObligation> {
        Ok(vec![work::Probe { task_id: owner.clone(), owner }])
    }
}

fn main() {
    let mut system = desk_system::System::new(desk_service::DeskService::new(Desk));
    desk_server::desk_service::serve(&mut system, "127.0.0.1:0").expect("the surface serves");
}
"#;

const GO_HARNESS: &str = r#"package main

import (
	"example.invalid/desk/components/deskservice"
	"example.invalid/desk/server"
	"example.invalid/desk/system"
	"example.invalid/desk/types/obligation"
	"example.invalid/desk/types/work"
)

type desk struct{}

func (desk) FinishTask(work.FinishTask) (work.FinishTaskOutcome, *obligation.UnmetObligation) {
	return nil, &obligation.UnmetObligation{Capability: "command behaviour", Source: "desk.work.FinishTask"}
}

func (desk) Probe(owner string) ([]work.Probe, *obligation.UnmetObligation) {
	return []work.Probe{{TaskId: owner, Owner: owner}}, nil
}

func main() {
	if err := server.ServeDeskService(system.NewSystem(deskservice.New(desk{})), "127.0.0.1:0"); err != nil {
		panic(err)
	}
}
"#;

fn compiled(model: &str) -> EssIr {
    ess_ui_check::compile_sources(
        &[("model.yaml".to_owned(), model.to_owned())],
        Path::new("model.yaml"),
    )
    .unwrap_or_else(|error| panic!("the model compiles: {error}"))
}

fn command_path() -> String {
    let ir = compiled(MODEL);
    let component = ir.components().values().next().expect("one component");
    http::routes(&ir, component)
        .into_iter()
        .find(|route| matches!(route.serves, Served::Command(_)))
        .expect("the command is served")
        .path
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary-view-params-pass2-{label}"))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn reported(what: &str, output: &Output) -> String {
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!("{what}\n{text}");
    text
}

fn cargo() -> Command {
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env("CARGO_INCREMENTAL", "0");
    command
}

fn go() -> Command {
    let mut command = Command::new("go");
    command
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local");
    command
}

fn rust_server() -> &'static Path {
    static BUILT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        let synthesis =
            synthesize_laid_out(&compiled(MODEL), Target::Rust, OutputLayout::Workspace)
                .expect("the model synthesizes for Rust");
        let root = scratch("rust");
        let _ = std::fs::remove_dir_all(&root);
        write(&synthesis, &root.join("desk"));
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).expect("mkdir");
        let dependencies = ["desk-server", "desk-system", "desk-service", "desk-types"]
            .iter()
            .fold(String::new(), |mut lines, name| {
                let _ = writeln!(lines, "{name} = {{ path = \"../desk/crates/{name}\" }}");
                lines
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"probe-harness\"\nversion = \"0.0.0\"\nedition = \
                 \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/main.rs"), RUST_HARNESS).expect("write");
        let target = root.join("target");
        let built = cargo()
            .args(["build", "--offline", "--quiet", "--target-dir"])
            .arg(&target)
            .current_dir(&harness)
            .output()
            .expect("cargo runs");
        reported("cargo build (the Rust probe)", &built);
        assert!(built.status.success(), "the Rust probe builds");
        let binary = scratch("rust-server");
        std::fs::copy(target.join("debug/probe-harness"), &binary).expect("copy");
        let _ = std::fs::remove_dir_all(&root);
        binary
    })
}

fn go_server() -> &'static Path {
    static BUILT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        let synthesis = synthesize_laid_out(&compiled(MODEL), Target::Go, OutputLayout::Workspace)
            .expect("the model synthesizes for Go");
        let root = scratch("go");
        let _ = std::fs::remove_dir_all(&root);
        let module = root.join("desk");
        write(&synthesis, &module);
        std::fs::create_dir_all(module.join("cmd/harness")).expect("mkdir");
        std::fs::write(module.join("cmd/harness/main.go"), GO_HARNESS).expect("write");
        let binary = scratch("go-server");
        let built = go()
            .args(["build", "-o"])
            .arg(&binary)
            .arg("./cmd/harness")
            .current_dir(&module)
            .output()
            .unwrap_or_else(|error| panic!("`go` runs: {error}"));
        reported("go build (the Go probe)", &built);
        assert!(built.status.success(), "the Go probe builds");
        let _ = std::fs::remove_dir_all(&root);
        binary
    })
}

struct Running {
    child: Child,
    port: u16,
}

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start(executable: &Path) -> Running {
    let mut child = Command::new(executable)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|error| panic!("{} starts: {error}", executable.display()));
    let stdout = child.stdout.take().expect("piped");
    let mut lines = BufReader::new(stdout).lines();
    let first = lines
        .next()
        .expect("the server writes its startup record")
        .expect("UTF-8");
    let record: serde_json::Value = serde_json::from_str(&first).expect("a JSON startup line");
    let port =
        u16::try_from(record["runtime"]["port"].as_u64().expect("the bound port")).expect("a port");
    std::thread::spawn(move || for _ in lines {});
    Running { child, port }
}

/// Sends `request` verbatim and reads to the end of the connection: every status line the
/// server wrote, and the body of the first answer.
fn exchange(server: &Running, request: &[u8]) -> (Vec<u16>, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).expect("the server accepts");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("a timeout");
    let _ = stream.write_all(request);
    let mut answer = Vec::new();
    let _ = stream.read_to_end(&mut answer);
    let answer = String::from_utf8_lossy(&answer).into_owned();
    let statuses = answer
        .match_indices("HTTP/1.1 ")
        .filter_map(|(index, _)| answer[index + 9..].get(..3)?.parse().ok())
        .collect();
    let body = answer
        .split_once("\r\n\r\n")
        .map_or_else(String::new, |(_, body)| body.to_owned());
    (statuses, body)
}

/// A caller that sends a whole request and hangs up before reading the answer — a browser
/// navigating away, a client giving up — leaves the server answering everybody else.
#[test]
fn a_caller_that_hangs_up_early_does_not_stop_the_rust_server() {
    let mut rust = start(rust_server());
    for _ in 0..20 {
        let Ok(mut stream) = TcpStream::connect(("127.0.0.1", rust.port)) else {
            break;
        };
        let _ = stream.write_all(b"GET /openapi.json HTTP/1.1\r\nHost: x\r\n\r\n");
        drop(stream);
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(200));
    let exited = rust.child.try_wait().expect("the child can be polled");
    assert!(
        exited.is_none(),
        "the Rust server exited ({exited:?}) after callers hung up before reading their answers"
    );
}

/// The command body the probes vary: `blob`, `counts` and `flags` spliced in.
fn body(blob: &str, counts: &str, flags: &str) -> String {
    format!("{{\"task_id\":\"t\",\"blob\":{blob},\"counts\":{counts},\"flags\":{flags}}}")
}

fn post(server: &Running, path: &str, body: &str) -> (u16, String) {
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\nContent-Length: \
         {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let (statuses, body) = exchange(server, request.as_bytes());
    (statuses.first().copied().unwrap_or(0), body)
}

/// Each probe posted to both servers: the lines where they answer differently, comparing the
/// status alone or the status and the body.
fn differences(probes: &[(&str, String)], with_body: bool) -> String {
    let rust = start(rust_server());
    let go = start(go_server());
    let path = command_path();
    let mut differ = String::new();
    for (label, sent) in probes {
        let (rust_status, rust_body) = post(&rust, &path, sent);
        let (go_status, go_body) = post(&go, &path, sent);
        if rust_status != go_status || (with_body && rust_body != go_body) {
            let _ = writeln!(
                differ,
                "{label}: rust {rust_status} {rust_body}\n{:>width$}go   {go_status} {go_body}",
                "",
                width = label.len() + 2
            );
        }
    }
    differ
}

/// Correction 1 makes a value outside the published pattern a `400` in both servers, for
/// queries and command bodies — for `Decimal` and `Uuid`. `Bytes` has a published pattern too
/// (`^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$`), and a boolean map key
/// is `true` or `false`; one body must be admitted or refused by both.
#[test]
fn bytes_and_boolean_keys_are_admitted_alike() {
    let probes = [
        ("bytes unpadded", body("\"A\"", "{}", "{}")),
        ("bytes padding then data", body("\"AA=A\"", "{}", "{}")),
        ("bytes all padding", body("\"====\"", "{}", "{}")),
        ("bytes with a newline", body("\"AA\\n==\"", "{}", "{}")),
        ("boolean key spelled 1", body("\"\"", "{}", "{\"1\":\"a\"}")),
        (
            "boolean key spelled TRUE",
            body("\"\"", "{}", "{\"TRUE\":\"a\"}"),
        ),
    ];
    let differ = differences(&probes, false);
    assert!(
        differ.is_empty(),
        "one server admits what the other refuses:\n{differ}"
    );
}

/// Correction 1: "Go refusal texts use Rust's words for every primitive (commands too)". Bytes
/// and non-string map keys are primitives a command body carries.
#[test]
fn bytes_and_map_key_refusals_use_one_wording() {
    let probes = [
        ("bytes not base64", body("\"!!!!\"", "{}", "{}")),
        (
            "integer key not a number",
            body("\"\"", "{\"x\":\"a\"}", "{}"),
        ),
        (
            "boolean key not a boolean",
            body("\"\"", "{}", "{\"yes\":\"a\"}"),
        ),
    ];
    let differ = differences(&probes, true);
    assert!(
        differ.is_empty(),
        "the servers refuse in different words:\n{differ}"
    );
}

/// The servers answer a request carrying 101 headers alike (Go keeps no count).
#[test]
fn a_request_with_101_headers_is_answered_alike() {
    let rust = start(rust_server());
    let go = start(go_server());
    let mut request = b"GET /openapi.json HTTP/1.1\r\nHost: x\r\n".to_vec();
    for index in 0..100 {
        request.extend_from_slice(format!("X-H{index}: v\r\n").as_bytes());
    }
    request.extend_from_slice(b"\r\n");
    let (rust_statuses, rust_body) = exchange(&rust, &request);
    let (go_statuses, _) = exchange(&go, &request);
    assert_eq!(
        rust_statuses, go_statuses,
        "101 headers: rust answers {rust_statuses:?} ({rust_body}), go answers {go_statuses:?}"
    );
}

/// While one caller has sent half a request line and nothing more, another caller is answered.
#[test]
fn a_silent_caller_does_not_hold_the_rust_server() {
    let rust = start(rust_server());
    let mut silent = TcpStream::connect(("127.0.0.1", rust.port)).expect("the server accepts");
    silent.write_all(b"GET /openapi").expect("write");
    std::thread::sleep(Duration::from_millis(100));
    let mut other = TcpStream::connect(("127.0.0.1", rust.port)).expect("the server accepts");
    other
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("a timeout");
    other
        .write_all(b"GET /openapi.json HTTP/1.1\r\nHost: x\r\n\r\n")
        .expect("write");
    let mut first = [0_u8; 12];
    let read = other.read(&mut first);
    drop(silent);
    assert!(
        matches!(read, Ok(12)),
        "a second caller got no answer within 3 s while a first sent nothing: {read:?}"
    );
}
