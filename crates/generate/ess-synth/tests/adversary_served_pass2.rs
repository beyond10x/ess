//! Adversary pass 2 on `story:generated-server-publishes-and-reads-headers`: the corrected pump,
//! which keeps its cursor on an event whose delivery an unmet obligation stopped, driven through
//! the generated Rust server when that obligation stays unmet.
//!
//! The model is pass 1's: `note-on-open` escalates through an owed escalation the realization has
//! not supplied yet (it always answers `UnmetObligation`, which is what an unrealized obligation
//! does), and the notebook `Note` invokes is full. One `Opened` is therefore undeliverable for as
//! long as the realization is unfinished. `Ping` publishes `Pinged`, which no binding reacts to, and
//! `Open` with `count: 0` is a declared refusal that publishes nothing.
//!
//! The contract's `501` says "delivering what the command published failed" and "the undelivered
//! events stay for the next delivery". Neither is true of a `Ping` served after the stuck
//! `Opened`: nothing `Ping` published was undeliverable, yet the pump stops on the old event first.

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

/// Every line is `label<TAB>status<TAB>body<TAB>log<TAB>tallies<TAB>pings`.
const HARNESS: &str = r##"use std::cell::RefCell;
use std::rc::Rc;

use ops_server::ops_service as surface;
use ops_server::http;
use ops_types::behaviour::{Context, Generated};
use ops_types::core;
use ops_types::obligation::UnmetObligation;

#[derive(Default)]
struct State {
    tallies: usize,
}

#[derive(Clone, Default)]
struct Desk(Rc<RefCell<State>>);

impl Context for Desk {
    fn external(&mut self, command: &'static str, outcome: &'static str) -> bool {
        match (command, outcome) {
            // The notebook stays full.
            ("ops.core.Note", "busy") => true,
            ("ops.core.Tally", "jammed") => {
                self.0.borrow_mut().tallies += 1;
                false
            }
            _ => false,
        }
    }
}

// The owed escalation is not realized yet: it answers unmet, every time.
impl ops_system::obligations::NoteOnOpenEscalation for Desk {
    fn note_on_open_escalation(&self, _failed: &core::Note) -> Result<core::NoteDropped, UnmetObligation> {
        Err(UnmetObligation { capability: "binding-escalation", source: "note-on-open" })
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
    let log = system.published().len();
    let pings = system
        .published()
        .iter()
        .filter(|event| event.name() == "ops.core.Pinged")
        .count();
    let state = desk.0.borrow();
    println!(
        "{label}\t{}\t{}\t{log}\t{}\t{pings}",
        answered.status,
        answered.body.replace('\t', " "),
        state.tallies
    );
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

    serve(s, &desk, &routes, "stuck", "ops.core.Open", r#"{"id":"e","count":1}"#);
    for round in 1..=5 {
        serve(s, &desk, &routes, &format!("ping-{round}"), "ops.core.Ping", "{}");
    }
    serve(s, &desk, &routes, "declined", "ops.core.Open", r#"{"id":"g","count":0}"#);
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
            .join(format!("adversary-served-pass2-{}", std::process::id()));
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
                "[package]\nname = \"adversary-served-pass2-harness\"\nversion = \"0.0.0\"\n\
                 edition = \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
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

struct Answer {
    status: u16,
    body: String,
    log: usize,
    tallies: usize,
    pings: usize,
}

fn served(label: &str) -> Answer {
    let line = lines()
        .iter()
        .find(|line| line[0] == label)
        .unwrap_or_else(|| panic!("the harness printed `{label}`"));
    Answer {
        status: line[1].parse().expect("a status"),
        body: line[2].clone(),
        log: line[3].parse().expect("log"),
        tallies: line[4].parse().expect("tallies"),
        pings: line[5].parse().expect("pings"),
    }
}

/// The stuck `Opened` answers 501, which is the corrected behaviour and not in dispute. A `Ping`
/// served after it published only `Pinged`, which no binding reacts to, so nothing it published can
/// have failed to deliver: its answer is `pinged`, 202, as the contract declares.
#[test]
fn a_command_whose_events_no_binding_reacts_to_is_answered_after_an_undeliverable_event() {
    assert_eq!(served("stuck").status, 501, "{}", served("stuck").body);
    let ping = served("ping-1");
    assert_eq!(
        ping.status, 202,
        "`Ping` published only `Pinged`, which no binding reacts to, and was answered: {}",
        ping.body
    );
}

/// A declared refusal publishes nothing and delivers nothing, so its answer is the declared 422
/// whatever an earlier command left undelivered.
#[test]
fn a_declared_refusal_is_answered_as_declared_after_an_undeliverable_event() {
    let declined = served("declined");
    assert_eq!(
        declined.status, 422,
        "`Open` with count 0 is the declared refusal and was answered: {}",
        declined.body
    );
}

/// A long-running server keeps nothing from one request to the next (`settle`'s own doc). Five
/// `Ping`s after the stuck event leave no `Pinged` on the log, and the log does not grow by one
/// entry per request.
#[test]
fn the_log_does_not_grow_with_every_request_after_an_undeliverable_event() {
    let first = served("ping-1");
    let fifth = served("ping-5");
    assert_eq!(
        fifth.pings, 0,
        "after five served `Ping`s the log still holds {} `Pinged` (log length {} after the first, \
         {} after the fifth)",
        fifth.pings, first.log, fifth.log
    );
}

/// `tally-on-open` reacts to `Opened` and never fails; at least once means it receives the stuck
/// `Opened` while the server goes on serving, whatever `note-on-open` still owes.
#[test]
fn the_binding_beside_the_unmet_one_receives_the_stuck_event() {
    let declined = served("declined");
    assert!(
        declined.tallies >= 1,
        "after six more served commands `tally-on-open` ran {} times for the stuck `Opened`; the \
         log holds {} events",
        declined.tallies,
        declined.log
    );
}
