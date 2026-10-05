//! Adversary pass 1 on `story:generated-server-publishes-and-reads-headers`: the generated Rust
//! server driven through every branch kind, a delivery that fails mid-pump, and real sockets for the
//! header list.
//!
//! One served component. `ops.core.Open` has an accepting branch that emits two events, a refusal
//! the input decides, and an external refusal; `ops.core.Ping` takes no input. Two bindings react to
//! `Opened`: `note-on-open` (escalates when `Note` refuses, through an owed escalation the harness
//! can leave unmet) and `tally-on-open` (drops a refusal). The harness prints every served answer
//! with what the system kept afterwards, and how often each binding's command ran.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::http::{self, Served};
use ess_synth::{synthesize_laid_out, OutputLayout, Synthesis, Target};

const MODEL: &str = "format: ess/15
system: ops
version: v1
domain: ops.core
events:
  - name: ops.core.Opened
    fields:
      - {name: id, type: String}
  - name: ops.core.Stamped
    fields:
      - {name: id, type: String}
  - name: ops.core.Noted
    fields:
      - {name: id, type: String}
  - name: ops.core.NoteDropped
    fields:
      - {name: id, type: String}
  - name: ops.core.Tallied
    fields:
      - {name: id, type: String}
  - name: ops.core.Pinged
    fields: []
commands:
  - name: ops.core.Open
    input:
      - {name: id, type: String}
      - {name: count, type: Integer}
    outcomes:
      - name: opened
        when: count > 0
        emits: [ops.core.Opened, ops.core.Stamped]
        payload:
          ops.core.Opened: {id: input.id}
          ops.core.Stamped: {id: input.id}
      - name: closed
        external: the desk is closed
        error: ops.core.Closed
      - name: refused
        error: ops.core.Bad
  - name: ops.core.Note
    input:
      - {name: id, type: String}
    outcomes:
      - name: noted
        emits: [ops.core.Noted]
        payload:
          ops.core.Noted: {id: input.id}
      - name: busy
        external: the notebook is full
        error: ops.core.Busy
  - name: ops.core.Tally
    input:
      - {name: id, type: String}
    outcomes:
      - name: tallied
        emits: [ops.core.Tallied]
        payload:
          ops.core.Tallied: {id: input.id}
      - name: jammed
        external: the counter is jammed
        error: ops.core.Jammed
  - name: ops.core.Ping
    outcomes:
      - name: pinged
        emits: [ops.core.Pinged]
errors:
  - name: ops.core.Closed
  - name: ops.core.Bad
  - name: ops.core.Busy
  - name: ops.core.Jammed
components:
  - component: ops-service
    owns: {domains: [ops.core]}
    accepts: {commands: [ops.core.Open, ops.core.Note, ops.core.Tally, ops.core.Ping]}
    publishes: {events: [ops.core.Opened, ops.core.Stamped, ops.core.Noted, ops.core.NoteDropped, ops.core.Tallied, ops.core.Pinged]}
    reached_by: network
bindings:
  - id: note-on-open
    when:
      event: ops.core.Opened
    invoke:
      command: ops.core.Note
    mapping:
      id: event.id
    delivery: at_least_once
    on_failure:
      escalate:
        emits: ops.core.NoteDropped
  - id: tally-on-open
    when:
      event: ops.core.Opened
    invoke:
      command: ops.core.Tally
    mapping:
      id: event.id
    delivery: at_least_once
    on_failure: drop
";

/// Every line is `label<TAB>status<TAB>body<TAB>kept<TAB>notes<TAB>tallies`, or `header<TAB>…`.
const HARNESS: &str = r##"use std::cell::RefCell;
use std::io::Write as _;
use std::rc::Rc;

use ops_server::ops_service as surface;
use ops_server::http;
use ops_types::behaviour::{Context, ExternalCommand, Generated};
use ops_types::core;
use ops_types::obligation::UnmetObligation;

#[derive(Default)]
struct State {
    closed: bool,
    busy: bool,
    unmet: bool,
    notes: usize,
    tallies: usize,
}

#[derive(Clone, Default)]
struct Desk(Rc<RefCell<State>>);

impl Context for Desk {
    fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool {
        let command = command.name();
        let mut state = self.0.borrow_mut();
        match (command, outcome) {
            ("ops.core.Open", "closed") => state.closed,
            ("ops.core.Note", "busy") => {
                state.notes += 1;
                state.busy
            }
            ("ops.core.Tally", "jammed") => {
                state.tallies += 1;
                false
            }
            _ => false,
        }
    }
}

impl ops_system::obligations::NoteOnOpenEscalation for Desk {
    fn note_on_open_escalation(&self, failed: &core::Note) -> Result<core::NoteDropped, UnmetObligation> {
        if self.0.borrow().unmet {
            return Err(UnmetObligation { capability: "binding-escalation", source: "note-on-open" });
        }
        Ok(core::NoteDropped { id: failed.id.clone() })
    }
}

type System = ops_system::System<Generated<Desk>, Desk>;

fn path<'a>(routes: &'a [(String, String, String)], name: &str) -> (&'a str, &'a str) {
    let (_, method, path) = routes
        .iter()
        .find(|(declared, _, _)| declared == name)
        .unwrap_or_else(|| panic!("`{name}` is a route"));
    (method, path)
}

fn kept(system: &mut System) -> usize {
    system.published().len() + system.invocations().len() + system.ops_service.drain_outbox().len()
}

fn serve(system: &mut System, desk: &Desk, routes: &[(String, String, String)], label: &str, name: &str, body: &str) {
    let (method, path) = path(routes, name);
    let request = http::Request {
        method: method.to_owned(),
        path: path.to_owned(),
        query: String::new(),
        headers: Vec::new(),
        body: body.as_bytes().to_vec(),
    };
    let answered = surface::dispatch(system, &request);
    let kept = kept(system);
    let state = desk.0.borrow();
    println!("{label}\t{}\t{}\t{kept}\t{}\t{}", answered.status, answered.body, state.notes, state.tallies);
}

/// Reads one raw request off a real socket with the generated reader.
fn read_raw(raw: Vec<u8>) -> Result<http::Request, http::Response> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("address");
    let client = std::thread::spawn(move || {
        let mut stream = std::net::TcpStream::connect(address).expect("connect");
        let _ = stream.write_all(&raw);
        let _ = stream.flush();
        std::thread::sleep(std::time::Duration::from_millis(200));
    });
    let (stream, _) = listener.accept().expect("accept");
    let mut reader = std::io::BufReader::new(stream);
    let read = http::read(&mut reader);
    client.join().expect("client");
    read
}

fn headers(label: &str, raw: Vec<u8>) {
    match read_raw(raw) {
        Ok(request) => {
            let list: Vec<String> = request.headers.iter().map(|(name, value)| format!("{name}={value}")).collect();
            println!("header\t{label}\tok\t{}\t{}", request.headers.len(), list.join("|"));
        }
        Err(refusal) => println!("header\t{label}\trefused\t{}", refusal.status),
    }
}

fn many(count: usize) -> Vec<u8> {
    let mut raw = String::from("POST /x HTTP/1.1\r\nContent-Length: 0\r\n");
    for index in 1..count {
        raw.push_str(&format!("X-H{index}: {index}\r\n"));
    }
    raw.push_str("\r\n");
    raw.into_bytes()
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let routes: Vec<(String, String, String)> = arguments
        .chunks(3)
        .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
        .collect();
    let desk = Desk::default();
    let mut system: System = ops_system::System::new(
        ops_service::OpsService::new(Generated::new(desk.clone())),
        desk.clone(),
    );
    let s = &mut system;

    serve(s, &desk, &routes, "opened", "ops.core.Open", r#"{"id":"a","count":1}"#);
    serve(s, &desk, &routes, "refused", "ops.core.Open", r#"{"id":"b","count":0}"#);
    desk.0.borrow_mut().closed = true;
    serve(s, &desk, &routes, "closed", "ops.core.Open", r#"{"id":"c","count":0}"#);
    desk.0.borrow_mut().closed = false;
    desk.0.borrow_mut().busy = true;
    serve(s, &desk, &routes, "busy", "ops.core.Note", r#"{"id":"d"}"#);
    serve(s, &desk, &routes, "ping-empty-object", "ops.core.Ping", "{}");
    serve(s, &desk, &routes, "ping-no-body", "ops.core.Ping", "");

    // The escalation is owed and unmet, and the notebook is busy: the pump fails on `note-on-open`
    // before `tally-on-open` has run for this `Opened`.
    desk.0.borrow_mut().unmet = true;
    serve(s, &desk, &routes, "unmet", "ops.core.Open", r#"{"id":"e","count":1}"#);
    desk.0.borrow_mut().unmet = false;
    desk.0.borrow_mut().busy = false;
    serve(s, &desk, &routes, "after-unmet", "ops.core.Open", r#"{"id":"f","count":1}"#);

    let request = http::Request {
        method: "GET".to_owned(),
        path: "/openapi.json".to_owned(),
        query: String::new(),
        headers: Vec::new(),
        body: Vec::new(),
    };
    let contract = surface::dispatch(s, &request);
    println!("contract\t{}", contract.body.replace('\n', " "));

    headers(
        "mixed",
        b"POST /x HTTP/1.1\r\nX-Trace: one\r\nAuthorization:   Bearer z  \r\nx-trace: two\r\nX-Name: J\xc3\xbcrgen\r\nContent-Length: 0\r\n\r\n".to_vec(),
    );
    headers("hundred", many(100));
    headers("hundred-and-one", many(101));
}
"##;

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn route_table(ir: &EssIr) -> Vec<String> {
    let component = ir.components().values().next().expect("one component");
    let mut table = Vec::new();
    for route in http::routes(ir, component) {
        let name = match route.serves {
            Served::Command(handle) => ir.command(handle).name.to_string(),
            Served::View(handle) => ir.view(handle).name.to_string(),
        };
        table.extend([name, route.method.as_str().to_owned(), route.path]);
    }
    table
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn cargo(directory: &Path, arguments: &[String]) -> Output {
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(arguments)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    eprintln!(
        "cargo {arguments:?} in {}\n{}",
        directory.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// The harness's output, once per test binary.
fn lines() -> &'static Vec<Vec<String>> {
    static LINES: std::sync::OnceLock<Vec<Vec<String>>> = std::sync::OnceLock::new();
    LINES.get_or_init(|| {
        let ir = ir();
        let synthesis = synthesize_laid_out(&ir, Target::Rust, OutputLayout::Workspace)
            .expect("the model synthesizes");
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("adversary-served-pass1-{}", std::process::id()));
        let generated = root.join("ops");
        write(&synthesis, &generated);
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).expect("mkdir");
        let dependencies = ["ops-server", "ops-system", "ops-service", "ops-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../ops/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"adversary-served-harness\"\nversion = \"0.0.0\"\nedition = \
                 \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/main.rs"), HARNESS).expect("write");
        let target = root.join("target");
        let mut arguments: Vec<String> = [
            "run",
            "--offline",
            "--quiet",
            "--target-dir",
            target.to_str().expect("UTF-8"),
            "--",
        ]
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect();
        arguments.extend(route_table(&ir));
        let ran = cargo(&harness, &arguments);
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        let _ = std::fs::remove_dir_all(&root);
        assert!(ran.status.success(), "the harness builds and runs");
        eprintln!("{stdout}");
        stdout
            .lines()
            .map(|line| line.split('\t').map(str::to_owned).collect())
            .collect()
    })
}

fn served(label: &str) -> (u16, serde_json::Value, usize, usize, usize) {
    let line = lines()
        .iter()
        .find(|line| line[0] == label)
        .unwrap_or_else(|| panic!("the harness printed `{label}`"));
    (
        line[1].parse().expect("a status"),
        serde_json::from_str(&line[2]).unwrap_or(serde_json::Value::String(line[2].clone())),
        line[3].parse().expect("kept"),
        line[4].parse().expect("notes"),
        line[5].parse().expect("tallies"),
    )
}

fn header(label: &str) -> Vec<String> {
    lines()
        .iter()
        .find(|line| line[0] == "header" && line[1] == label)
        .unwrap_or_else(|| panic!("the harness printed header `{label}`"))[2..]
        .to_vec()
}

fn contract() -> serde_json::Value {
    let line = lines()
        .iter()
        .find(|line| line[0] == "contract")
        .expect("the contract");
    serde_json::from_str(&line[1]).expect("JSON")
}

/// Validates one answer against the response schema the served contract declares for its path and
/// status; `Err` names why.
fn validate(command: &str, status: u16, answer: &serde_json::Value) -> Result<(), String> {
    let contract = contract();
    let ir = ir();
    let table = route_table(&ir);
    let path = table
        .chunks(3)
        .find(|row| row[0] == command)
        .map(|row| row[2].clone())
        .expect("a route");
    let schema_at = &contract["paths"][&path]["post"]["responses"][status.to_string()]["content"]
        ["application/json"]["schema"];
    if !schema_at.is_object() {
        return Err(format!("the contract declares no {status} for `{command}`"));
    }
    let mut schema = contract.clone();
    for key in ["$ref", "oneOf"] {
        if let Some(value) = schema_at.get(key) {
            schema[key] = value.clone();
        }
    }
    let validator = jsonschema::draft202012::new(&schema).expect("compiles");
    let errors: Vec<String> = validator
        .iter_errors(answer)
        .map(|error| format!("{error} at {}", error.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[test]
fn a_two_event_branch_is_answered_in_emit_order_and_validates() {
    let (status, answer, kept, _, _) = served("opened");
    assert_eq!(status, 202);
    assert_eq!(
        answer["published"],
        serde_json::json!([
            {"event": "ops.core.Opened", "payload": {"id": "a"}},
            {"event": "ops.core.Stamped", "payload": {"id": "a"}},
        ])
    );
    assert_eq!(kept, 0);
    validate("ops.core.Open", status, &answer).expect("the served contract admits it");
}

#[test]
fn every_refusal_branch_answer_validates_against_the_served_contract() {
    for (label, command, status) in [
        ("refused", "ops.core.Open", 422),
        ("closed", "ops.core.Open", 502),
        ("busy", "ops.core.Note", 502),
    ] {
        let (answered, answer, kept, _, _) = served(label);
        assert_eq!(answered, status, "`{label}` answers {status}: {answer}");
        assert_eq!(answer["published"], serde_json::json!([]), "`{label}`");
        assert_eq!(kept, 0, "`{label}` keeps nothing");
        validate(command, status, &answer)
            .unwrap_or_else(|error| panic!("`{label}` {answer}: {error}"));
    }
}

#[test]
fn a_command_without_input_is_answered_and_validates_with_or_without_a_body() {
    for label in ["ping-empty-object", "ping-no-body"] {
        let (status, answer, kept, _, _) = served(label);
        assert_eq!(status, 202, "`{label}`: {answer}");
        assert_eq!(answer["published"][0]["event"], "ops.core.Pinged");
        assert_eq!(kept, 0);
        validate("ops.core.Ping", status, &answer)
            .unwrap_or_else(|error| panic!("`{label}` {answer}: {error}"));
    }
}

/// A delivery that fails mid-pump answers 501, and the next served command answers only its own
/// events and leaves nothing behind.
#[test]
fn after_a_failed_delivery_the_next_answer_is_its_own_and_nothing_accumulates() {
    let (status, _, _, _, _) = served("unmet");
    assert_eq!(status, 501);
    let (status, answer, kept, _, _) = served("after-unmet");
    assert_eq!(status, 202);
    assert_eq!(
        answer["published"],
        serde_json::json!([
            {"event": "ops.core.Opened", "payload": {"id": "f"}},
            {"event": "ops.core.Stamped", "payload": {"id": "f"}},
        ])
    );
    assert_eq!(
        kept, 0,
        "the log, the record and the outbox are empty again"
    );
}

/// `at_least_once`: every `Opened` the server accepted reaches `tally-on-open` at least once, even
/// the one whose pump failed on the other binding first. Three `Opened` were published (`a`, `e`,
/// `f`), so `Tally` ran three times.
#[test]
fn a_failed_pump_does_not_lose_the_other_bindings_delivery() {
    let (_, _, _, _, tallies) = served("after-unmet");
    assert_eq!(
        tallies, 3,
        "`tally-on-open` ran for every published `Opened`, including the one whose pump failed \
         on `note-on-open`"
    );
}

#[test]
fn headers_keep_arrival_order_duplicates_lower_cased_names_and_trimmed_utf8_values() {
    let mixed = header("mixed");
    assert_eq!(mixed[0], "ok");
    assert_eq!(
        mixed[2],
        "x-trace=one|authorization=Bearer z|x-trace=two|x-name=Jürgen|content-length=0"
    );
}

#[test]
fn a_hundred_headers_are_kept_and_the_hundred_and_first_is_431() {
    assert_eq!(header("hundred")[..2], ["ok".to_owned(), "100".to_owned()]);
    assert_eq!(
        header("hundred-and-one")[..2],
        ["refused".to_owned(), "431".to_owned()]
    );
}
