//! Adversary pass 1 on `story:served-view-params`: the Rust and the Go servers read a served
//! view's parameters from the query string, and must answer every query alike, as the contract
//! they both publish at `/openapi.json` says.
//!
//! One view declares a parameter of every scalar kind a query carries, two of them under a wire
//! name that is not the declared name. Each harness implements the owed query and answers one row
//! whose `task_id` spells exactly what the port was handed. The servers are built, started on an
//! ephemeral port, asked over a socket and killed by their PID.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

use ess_compiler::EssIr;
use ess_gen::http::{self, Served};
use ess_synth::{synthesize_laid_out, OutputLayout, Synthesis, Target};

/// The model around one view, `desk.work.Probe`, whose `params:` and `filter:` are spliced in.
fn model(params: &str, filter: &str) -> String {
    format!(
        "format: ess/18
system: desk
version: v1
domain: desk.work
types:
  - {{name: desk.work.Level, kind: enum, variants: [Low, High]}}
  - {{name: desk.work.Code, kind: newtype, of: String}}
entities:
  - name: desk.work.Task
    identity: {{name: task_id, type: String}}
    fields:
      - {{name: owner, type: String}}
      - {{name: hours, type: Integer}}
      - {{name: cost, type: Decimal}}
      - {{name: reference, type: Uuid}}
      - {{name: due, type: Timestamp}}
      - {{name: urgent, type: Boolean}}
      - {{name: level, type: desk.work.Level}}
      - {{name: code, type: desk.work.Code}}
      - {{name: note, type: String}}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {{name: finish, from: [Open], to: Done}}
events:
  - name: desk.work.TaskFinished
    fields:
      - {{name: task_id, type: String}}
errors:
  - name: desk.work.Late
    fields:
      - {{name: reason, type: String}}
commands:
  - name: desk.work.FinishTask
    input:
      - {{name: task_id, type: String}}
    outcomes:
      - name: finished
        moves: desk.work.Task.finish
        instance: task_id
        emits: [desk.work.TaskFinished]
        payload:
          desk.work.TaskFinished: {{task_id: input.task_id}}
      - name: late
        external: the task is past due
        error: desk.work.Late
views:
  - name: desk.work.Probe
    source: desk.work.Task
    consistency: eventual
    params:
{params}    filter: [{filter}]
    fields:
      - {{name: task_id, type: String}}
      - {{name: owner, type: String}}
components:
  - component: desk-service
    owns: {{domains: [desk.work]}}
    accepts: {{commands: [desk.work.FinishTask]}}
    publishes: {{events: [desk.work.TaskFinished]}}
    reached_by: network
"
    )
}

/// The probe: every scalar kind, `owner` carried as `who` and `hours` as `hrs`.
fn probe_model() -> String {
    model(
        "      - {name: owner, type: String, wire: who}
      - {name: hours, type: Integer, wire: hrs}
      - {name: cost, type: Optional<Decimal>}
      - {name: reference, type: Optional<Uuid>}
      - {name: due, type: Optional<Timestamp>}
      - {name: urgent, type: Optional<Boolean>}
      - {name: level, type: Optional<desk.work.Level>}
      - {name: code, type: Optional<desk.work.Code>}
      - {name: note, type: Optional<String>}
",
        "owner == param.owner, hours >= param.hours, cost == param.cost, reference == \
         param.reference, due >= param.due, urgent == param.urgent, level == param.level, code \
         == param.code, note == param.note",
    )
}

const RUST_HARNESS: &str = r#"use desk_types::obligation::UnmetObligation;
use desk_types::primitives::{Decimal, Timestamp, Uuid};
use desk_types::work::{self, Code, Level};

struct Desk;

impl work::obligations::FinishTaskBehavior for Desk {
    fn finish_task(
        &mut self,
        _input: work::FinishTask,
    ) -> Result<work::FinishTaskOutcome, UnmetObligation> {
        Err(UnmetObligation { capability: "command behaviour", source: "desk.work.FinishTask" })
    }
}

fn spelled(value: Option<&str>) -> String {
    value.map_or_else(|| "<none>".to_owned(), |value| format!("[{value}]"))
}

impl work::obligations::ProbeQuery for Desk {
    #[allow(clippy::too_many_arguments)]
    fn probe(
        &self,
        owner: String,
        hours: i64,
        cost: Option<Decimal>,
        reference: Option<Uuid>,
        due: Option<Timestamp>,
        urgent: Option<bool>,
        level: Option<Level>,
        code: Option<Code>,
        note: Option<String>,
    ) -> Result<Vec<work::Probe>, UnmetObligation> {
        let urgent = urgent.map(|urgent| urgent.to_string());
        let level = level.map(|level| match level {
            Level::Low => "Low",
            Level::High => "High",
        });
        Ok(vec![work::Probe {
            task_id: format!(
                "owner=[{owner}];hours=[{hours}];cost={};reference={};due={};urgent={};level={};code={};note={}",
                spelled(cost.as_ref().map(|it| it.0.as_str())),
                spelled(reference.as_ref().map(|it| it.0.as_str())),
                spelled(due.as_ref().map(|it| it.0.as_str())),
                spelled(urgent.as_deref()),
                spelled(level),
                spelled(code.as_ref().map(|it| it.0.as_str())),
                spelled(note.as_deref()),
            ),
            owner,
        }])
    }
}

fn main() {
    let mut system = desk_system::System::new(desk_service::DeskService::new(Desk));
    desk_server::desk_service::serve(&mut system, "127.0.0.1:0").expect("the surface serves");
}
"#;

const GO_HARNESS: &str = r#"package main

import (
	"fmt"

	"example.invalid/desk/components/deskservice"
	"example.invalid/desk/server"
	"example.invalid/desk/system"
	"example.invalid/desk/types/obligation"
	"example.invalid/desk/types/primitives"
	"example.invalid/desk/types/work"
)

type desk struct{}

func (desk) FinishTask(work.FinishTask) (work.FinishTaskOutcome, *obligation.UnmetObligation) {
	return nil, &obligation.UnmetObligation{Capability: "command behaviour", Source: "desk.work.FinishTask"}
}

func spelled(present bool, value string) string {
	if !present {
		return "<none>"
	}
	return "[" + value + "]"
}

func (desk) Probe(owner string, hours int64, cost *primitives.Decimal, reference *primitives.Uuid, due *primitives.Timestamp, urgent *bool, level *work.Level, code *work.Code, note *string) ([]work.Probe, *obligation.UnmetObligation) {
	value := func(present bool, read func() string) string {
		if !present {
			return spelled(false, "")
		}
		return spelled(true, read())
	}
	spelledLevel := spelled(false, "")
	if level != nil {
		switch (*level).(type) {
		case work.LevelLow:
			spelledLevel = spelled(true, "Low")
		case work.LevelHigh:
			spelledLevel = spelled(true, "High")
		}
	}
	taskID := fmt.Sprintf("owner=[%s];hours=[%d];cost=%s;reference=%s;due=%s;urgent=%s;level=%s;code=%s;note=%s",
		owner, hours,
		value(cost != nil, func() string { return cost.Value() }),
		value(reference != nil, func() string { return reference.Value() }),
		value(due != nil, func() string { return due.Value() }),
		value(urgent != nil, func() string { return fmt.Sprint(*urgent) }),
		spelledLevel,
		value(code != nil, func() string { return code.Value() }),
		value(note != nil, func() string { return *note }),
	)
	return []work.Probe{{TaskId: taskID, Owner: owner}}, nil
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

fn view_path(ir: &EssIr) -> String {
    let component = ir.components().values().next().expect("one component");
    http::routes(ir, component)
        .into_iter()
        .find(|route| matches!(route.serves, Served::View(_)))
        .expect("the view is served")
        .path
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary-view-params-pass1-{label}"))
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
        let synthesis = synthesize_laid_out(
            &compiled(&probe_model()),
            Target::Rust,
            OutputLayout::Workspace,
        )
        .expect("the probe synthesizes for Rust");
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
        let synthesis = synthesize_laid_out(
            &compiled(&probe_model()),
            Target::Go,
            OutputLayout::Workspace,
        )
        .expect("the probe synthesizes for Go");
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

/// `GET target` over a socket: the status and the body, parsed where it is JSON.
fn get(server: &Running, target: &[u8]) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).expect("the server accepts");
    let mut request = b"GET ".to_vec();
    request.extend_from_slice(target);
    request.extend_from_slice(b" HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    let _ = stream.write_all(&request);
    let mut answer = Vec::new();
    let _ = stream.read_to_end(&mut answer);
    let answer = String::from_utf8_lossy(&answer).into_owned();
    let Some((head, body)) = answer.split_once("\r\n\r\n") else {
        return (0, answer);
    };
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|status| status.parse().ok())
        .unwrap_or(0);
    let body = serde_json::from_str::<serde_json::Value>(body)
        .map_or_else(|_| format!("(not JSON) {body}"), |value| value.to_string());
    (status, body)
}

/// Query strings a browser, the terminal app or a hand-written client sends.
fn probes() -> Vec<(&'static str, Vec<u8>)> {
    let long = "a".repeat(2 * 1024 * 1024);
    let mut probes: Vec<(&'static str, Vec<u8>)> = [
        ("both required", "who=ada&hrs=3"),
        ("percent space and plus space", "who=ada%20l+ovelace&hrs=3"),
        ("UTF-8 percent-encoded", "who=%C3%A9l%C3%A8ve&hrs=3"),
        ("NUL", "who=a%00b&hrs=3"),
        ("percent-encoded key", "%77ho=ada&hrs=3"),
        ("key without =", "who&hrs=3"),
        ("empty required string", "who=&hrs=3"),
        ("empty pairs", "&&who=ada&&hrs=3&"),
        ("semicolon is not a separator", "who=ada;hrs=3"),
        ("upper-case key", "WHO=ada&hrs=3"),
        ("declared name, not wire name", "who=ada&hours=3"),
        ("repeated required", "who=ada&who=bob&hrs=3"),
        ("repeated through two spellings", "who=ada&%77ho=bob&hrs=3"),
        ("repeated optional", "who=ada&hrs=3&urgent=true&urgent=true"),
        ("malformed escape", "who=%ZZ&hrs=3"),
        ("truncated escape", "who=%4&hrs=3"),
        ("lone percent", "who=%&hrs=3"),
        ("invalid UTF-8", "who=%FF&hrs=3"),
        ("malformed undeclared key", "%ZZ=1&who=ada&hrs=3"),
        ("malformed undeclared value", "x=%ZZ&who=ada&hrs=3"),
        ("empty integer", "who=ada&hrs="),
        ("integer max", "who=ada&hrs=9223372036854775807"),
        ("integer min", "who=ada&hrs=-9223372036854775808"),
        ("integer over max", "who=ada&hrs=9223372036854775808"),
        (
            "integer far over max",
            "who=ada&hrs=99999999999999999999999",
        ),
        ("integer under min", "who=ada&hrs=-9223372036854775809"),
        ("integer leading zeros", "who=ada&hrs=007"),
        ("integer minus zero", "who=ada&hrs=-0"),
        ("integer plus sign", "who=ada&hrs=%2B5"),
        ("integer lone minus", "who=ada&hrs=-"),
        ("integer exponent", "who=ada&hrs=1e3"),
        ("integer fraction", "who=ada&hrs=1.0"),
        ("integer padded", "who=ada&hrs=+3"),
        ("decimal plain", "who=ada&hrs=1&cost=12.50"),
        ("decimal exponent", "who=ada&hrs=1&cost=1e3"),
        ("decimal junk", "who=ada&hrs=1&cost=abc"),
        (
            "uuid upper",
            "who=ada&hrs=1&reference=0E8F1C2A-0000-4000-8000-000000000001",
        ),
        ("uuid junk", "who=ada&hrs=1&reference=nope"),
        (
            "timestamp offset unencoded",
            "who=ada&hrs=1&due=2026-01-01T00:00:00+01:00",
        ),
        (
            "timestamp offset encoded",
            "who=ada&hrs=1&due=2026-01-01T00:00:00%2B01:00",
        ),
        ("boolean false", "who=ada&hrs=1&urgent=false"),
        ("boolean upper", "who=ada&hrs=1&urgent=TRUE"),
        ("boolean one", "who=ada&hrs=1&urgent=1"),
        ("boolean empty", "who=ada&hrs=1&urgent="),
        ("boolean null", "who=ada&hrs=1&urgent=null"),
        ("enum exact", "who=ada&hrs=1&level=High"),
        ("enum lower", "who=ada&hrs=1&level=high"),
        ("enum empty", "who=ada&hrs=1&level="),
        ("newtype", "who=ada&hrs=1&code=X-1"),
        ("newtype empty", "who=ada&hrs=1&code="),
        ("optional string empty", "who=ada&hrs=1&note="),
        ("optional string absent", "who=ada&hrs=1"),
        ("fragment", "who=ada&hrs=3#frag"),
        ("query of only ?", ""),
    ]
    .into_iter()
    .map(|(label, query)| (label, query.as_bytes().to_vec()))
    .collect();
    probes.push(("raw UTF-8 bytes", "who=élève&hrs=3".as_bytes().to_vec()));
    probes.push((
        "two-megabyte value",
        format!("who={long}&hrs=3").into_bytes(),
    ));
    probes
}

fn answers(executable: &Path) -> Vec<(u16, String)> {
    let server = start(executable);
    let path = view_path(&compiled(&probe_model()));
    probes()
        .into_iter()
        .map(|(_, query)| {
            let mut target = format!("{path}?").into_bytes();
            target.extend_from_slice(&query);
            get(&server, &target)
        })
        .collect()
}

fn shortened(text: &str) -> String {
    if text.len() > 200 {
        format!("{}… ({} bytes)", &text[..200], text.len())
    } else {
        text.to_owned()
    }
}

/// The two servers answer every probe with the same status and the same body.
#[test]
fn go_and_rust_answer_every_query_string_alike() {
    let rust = answers(rust_server());
    let go = answers(go_server());
    let mut differing = String::new();
    for (((label, _), rust), go) in probes().iter().zip(&rust).zip(&go) {
        if rust != go {
            let _ = writeln!(
                differing,
                "{label}:\n  rust {} {}\n  go   {} {}",
                rust.0,
                shortened(&rust.1),
                go.0,
                shortened(&go.1)
            );
        }
    }
    assert!(
        differing.is_empty(),
        "the servers disagree on {} probe(s):\n{differing}",
        differing.matches("\n  rust").count()
    );
}

/// What `/openapi.json` declares for the probe view's query parameters, `(name, required)`.
fn published(server: &Running, path: &str) -> BTreeSet<(String, bool)> {
    let (status, body) = get(server, b"/openapi.json");
    assert_eq!(status, 200, "{body}");
    let contract: serde_json::Value = serde_json::from_str(&body).expect("JSON");
    contract["paths"][path]["get"]["parameters"]
        .as_array()
        .unwrap_or_else(|| panic!("the view's operation lists parameters: {body}"))
        .iter()
        .filter(|parameter| parameter["in"] == "query")
        .map(|parameter| {
            (
                parameter["name"].as_str().expect("a name").to_owned(),
                parameter["required"].as_bool().unwrap_or(false),
            )
        })
        .collect()
}

/// For each server: the published query parameters are exactly the keys it reads, and a
/// parameter is published as required exactly where the server refuses its absence.
#[test]
fn each_server_reads_exactly_the_query_parameters_it_publishes() {
    let ir = compiled(&probe_model());
    let path = view_path(&ir);
    let full: [(&str, &str); 9] = [
        ("who", "ada"),
        ("hrs", "3"),
        ("cost", "1.5"),
        ("reference", "0e8f1c2a-0000-4000-8000-000000000001"),
        ("due", "2026-01-01T00:00:00Z"),
        ("urgent", "true"),
        ("level", "Low"),
        ("code", "c"),
        ("note", "n"),
    ];
    for (label, executable) in [("rust", rust_server()), ("go", go_server())] {
        let server = start(executable);
        let contract = published(&server, &path);
        let mut behaved = BTreeSet::new();
        for (left_out, _) in full {
            let query: Vec<String> = full
                .iter()
                .filter(|(key, _)| *key != left_out)
                .map(|(key, value)| format!("{key}={value}"))
                .collect();
            let (status, body) = get(&server, format!("{path}?{}", query.join("&")).as_bytes());
            assert!(status == 200 || status == 400, "{label}: {status} {body}");
            behaved.insert((left_out.to_owned(), status == 400));
        }
        assert_eq!(
            contract, behaved,
            "{label}: `/openapi.json` (left) against what the server refuses to go without (right)"
        );
    }
}

/// A value the published parameter schema refuses — `cost` against its decimal `pattern`,
/// `reference` against its UUID `pattern` — is refused by each server, not handed to the port.
#[test]
fn a_query_value_its_published_schema_refuses_is_refused() {
    let path = view_path(&compiled(&probe_model()));
    let mut accepted = Vec::new();
    for (label, executable) in [("rust", rust_server()), ("go", go_server())] {
        let server = start(executable);
        let (status, body) = get(&server, b"/openapi.json");
        assert_eq!(status, 200, "{body}");
        let contract: serde_json::Value = serde_json::from_str(&body).expect("JSON");
        let parameters = contract["paths"][path.as_str()]["get"]["parameters"].to_string();
        for (key, value) in [("cost", "abc"), ("reference", "nope")] {
            assert!(
                parameters.contains("pattern"),
                "{label}: the contract constrains `{key}`: {parameters}"
            );
            let (status, body) = get(
                &server,
                format!("{path}?who=ada&hrs=1&{key}={value}").as_bytes(),
            );
            if status != 400 {
                accepted.push(format!("{label} `{key}={value}`: {status} {body}"));
            }
        }
    }
    assert!(
        accepted.is_empty(),
        "values outside the published schema reach the port:\n{}",
        accepted.join("\n")
    );
}

/// Two parameters carried under one wire name are one query key answering two questions: the
/// model or both serving targets refuse it.
#[test]
fn two_params_under_one_wire_name_are_refused() {
    let shared = model(
        "      - {name: owner, type: String, wire: who}
      - {name: note, type: String, wire: who}
",
        "owner == param.owner, note == param.note",
    );
    let Ok(ir) = ess_ui_check::compile_sources(
        &[("model.yaml".to_owned(), shared)],
        Path::new("model.yaml"),
    ) else {
        return;
    };
    for target in [Target::Rust, Target::Go] {
        assert!(
            synthesize_laid_out(&ir, target, OutputLayout::Workspace).is_err(),
            "{target:?} serves two parameters under the one query key `who`"
        );
    }
}

/// The web target of a model whose view declares parameters builds against the Rust target's
/// tree: the bridge's `Unrealized` stub and its projection both meet the new query signature.
#[test]
fn the_web_target_of_a_param_view_builds() {
    let ir = compiled(&probe_model());
    let root = scratch("web");
    let _ = std::fs::remove_dir_all(&root);
    let rust = synthesize_laid_out(&ir, Target::Rust, OutputLayout::Workspace).expect("Rust");
    write(&rust, &root.join("rust/desk"));
    let web = match synthesize_laid_out(&ir, Target::Web, OutputLayout::Workspace) {
        Ok(web) => web,
        Err(error) => panic!("the web target refuses the probe: {error:?}"),
    };
    write(&web, &root.join("web/desk"));
    let built = cargo()
        .args([
            "check",
            "--offline",
            "--quiet",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
        ])
        .arg(root.join("target"))
        .current_dir(root.join("web/desk"))
        .output()
        .expect("cargo runs");
    let text = reported("cargo check (the web target)", &built);
    let _ = std::fs::remove_dir_all(&root);
    assert!(built.status.success(), "the web target builds:\n{text}");
}

/// Synthesizes `model` for `target` and builds what it emits: `Ok(())` where the model or its
/// synthesis is refused, or the emitted code builds, the build's output where it does not.
fn builds(model: &str, target: Target, label: &str) -> Result<(), String> {
    let ir = match ess_ui_check::compile_sources(
        &[("model.yaml".to_owned(), model.to_owned())],
        Path::new("model.yaml"),
    ) {
        Ok(ir) => ir,
        Err(error) => {
            eprintln!("{label}: the model is refused: {error}");
            return Ok(());
        }
    };
    let synthesis = match synthesize_laid_out(&ir, target, OutputLayout::Workspace) {
        Ok(synthesis) => synthesis,
        Err(error) => {
            eprintln!("{label}: synthesis refuses: {error:?}");
            return Ok(());
        }
    };
    let root = scratch(label);
    write(&synthesis, &root);
    let built = match target {
        Target::Rust => cargo()
            .args(["check", "--offline", "--quiet", "--target-dir"])
            .arg(root.join("target"))
            .current_dir(&root)
            .output()
            .expect("cargo runs"),
        Target::Go => go()
            .args(["build", "./..."])
            .current_dir(&root)
            .output()
            .expect("go runs"),
        _ => unreachable!("only the serving targets"),
    };
    let text = reported(label, &built);
    let _ = std::fs::remove_dir_all(&root);
    if built.status.success() {
        Ok(())
    } else {
        Err(text)
    }
}

fn colliding_model() -> String {
    model(
        "      - {name: minHours, type: Integer}
      - {name: min_hours, type: Integer}
",
        "hours >= param.minHours, hours >= param.min_hours",
    )
}

/// Two parameters whose names spell one identifier — `minHours` and `min_hours` — are refused
/// at synthesis or emitted as code that builds; the Rust target.
#[test]
fn params_that_normalise_to_one_identifier_are_refused_or_build_rust() {
    if let Err(output) = builds(&colliding_model(), Target::Rust, "collide-rust") {
        panic!("the Rust target emits code that does not build:\n{output}");
    }
}

/// The same, for the Go target.
#[test]
fn params_that_normalise_to_one_identifier_are_refused_or_build_go() {
    if let Err(output) = builds(&colliding_model(), Target::Go, "collide-go") {
        panic!("the Go target emits code that does not build:\n{output}");
    }
}

/// A parameter named as a Go predeclared identifier the emitted method bodies use: `nil`.
fn predeclared_model() -> String {
    model(
        "      - {name: nil, type: String}
",
        "owner == param.nil",
    )
}

#[test]
fn a_param_named_nil_is_refused_or_builds_go() {
    if let Err(output) = builds(&predeclared_model(), Target::Go, "nil-go") {
        panic!("the Go target emits code that does not build:\n{output}");
    }
}

/// Rust and Go keywords as parameter names: `type`, `ref`, `func`, `range`, `match`, `self`.
fn keyword_model() -> String {
    model(
        "      - {name: type, type: String}
      - {name: ref, type: String}
      - {name: func, type: String}
      - {name: range, type: String}
      - {name: match, type: String}
      - {name: self, type: String}
",
        "owner == param.type, owner == param.ref, owner == param.func, owner == param.range, \
         owner == param.match, owner == param.self",
    )
}

#[test]
fn keyword_params_are_refused_or_build_rust() {
    if let Err(output) = builds(&keyword_model(), Target::Rust, "keyword-rust") {
        panic!("the Rust target emits code that does not build:\n{output}");
    }
}

#[test]
fn keyword_params_are_refused_or_build_go() {
    if let Err(output) = builds(&keyword_model(), Target::Go, "keyword-go") {
        panic!("the Go target emits code that does not build:\n{output}");
    }
}
