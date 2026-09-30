//! Adversary pass 2 on `story:generated-server-publishes-and-reads-headers`: the corrected pump
//! against a binding that declares `delivery: at_most_once`.
//!
//! `ess_domain::binding::Delivery::AtMostOnce` is "one attempt, and no redelivery", and the system
//! emitter's module doc says "the pump itself never repeats one". `count-on-open` declares
//! `at_most_once` and `drop` and reacts to `Opened` before `note-on-open` does. `note-on-open`
//! escalates through an owed escalation that is unmet once, while the notebook is full, and met
//! after. The corrected pump keeps its cursor on the `Opened` whose delivery `note-on-open` stopped,
//! so the next pump delivers that `Opened` again to every binding, `count-on-open` included.

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
  - id: count-on-open
    when:
      event: ops.core.Opened
    invoke:
      command: ops.core.Tally
    mapping:
      id: event.id
    delivery: at_most_once
    on_failure: drop
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
    unmet: bool,
    busy: bool,
    tallies: usize,
}

#[derive(Clone, Default)]
struct Desk(Rc<RefCell<State>>);

impl Context for Desk {
    fn external(&mut self, command: &'static str, outcome: &'static str) -> bool {
        let mut state = self.0.borrow_mut();
        match (command, outcome) {
            ("ops.core.Note", "busy") => state.busy,
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

fn serve(system: &mut System, desk: &Desk, routes: &[(String, String, String)], label: &str, name: &str, body: &str) {
    let (method, path) = path(routes, name);
    let request = http::Request {
        method: method.to_owned(),
        path: path.to_owned(),
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

    desk.0.borrow_mut().unmet = true;
    desk.0.borrow_mut().busy = true;
    serve(s, &desk, &routes, "stopped", "ops.core.Open", r#"{"id":"e","count":1}"#);
    desk.0.borrow_mut().unmet = false;
    desk.0.borrow_mut().busy = false;
    serve(s, &desk, &routes, "ping", "ops.core.Ping", "{}");
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
            .join(format!("adversary-served-pass2-amo-{}", std::process::id()));
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
                "[package]\nname = \"adversary-served-pass2-amo-harness\"\nversion = \"0.0.0\"\n\
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

/// One `Opened` was published, and `count-on-open` declares at most one attempt per occurrence, so
/// `Tally` ran exactly once — the attempt it made before `note-on-open` stopped the delivery.
#[test]
fn an_at_most_once_binding_is_not_delivered_again_when_a_binding_beside_it_stopped() {
    let stopped = served("stopped");
    assert_eq!(stopped.status, 501, "{}", stopped.body);
    assert_eq!(
        stopped.tallies, 1,
        "`count-on-open` ran once before the stop"
    );
    let ping = served("ping");
    assert_eq!(ping.status, 202, "{}", ping.body);
    assert_eq!(
        ping.tallies, 1,
        "`count-on-open` declares at_most_once and ran {} times for one `Opened` (log {}, {} \
         `Pinged` kept)",
        ping.tallies, ping.log, ping.pings
    );
}
