//! Correction pass 2 on `story:generated-server-publishes-and-reads-headers`: the web bridge runs
//! the Rust pump after every command, so an event that keeps failing must not stop it answering.
//!
//! The model is adversary pass 2's: `note-on-open` escalates through an owed escalation the
//! realization answers unmet every time, while the notebook it invokes stays full, so one `Opened`
//! is undeliverable to it for as long as the realization is unfinished. `tally-on-open` reacts to
//! the same `Opened` and never fails. `Ping` publishes `Pinged`, which no binding reacts to, and
//! `Open` with `count: 0` is a declared refusal that publishes nothing.
//!
//! The host links the generated web crate natively and drives it through `serve`, the function
//! the page's requests reach. The command whose event stuck is refused as unmet; every command
//! after it is answered with its own outcome, and `tally-on-open` ran exactly once for the stuck
//! `Opened` — it had its attempt beside the failing binding and is never delivered it again.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Synthesis, Target};

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

/// The realized module: the generated ports over declared behaviour, with the escalation still
/// owed, and one extra export reporting how often `Tally` ran.
const HOST: &str = r#"use std::sync::atomic::{AtomicU32, Ordering};

use ops_types::behaviour::{Context, ExternalCommand, Generated};
use ops_types::core;
use ops_types::obligation::UnmetObligation;

static TALLIES: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Default)]
struct Desk;

impl Context for Desk {
    fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool {
        let command = command.name();
        match (command, outcome) {
            // The notebook stays full.
            ("ops.core.Note", "busy") => true,
            ("ops.core.Tally", "jammed") => {
                TALLIES.fetch_add(1, Ordering::Relaxed);
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

/// Installs the realization.
#[no_mangle]
pub extern "C" fn ess_realize() {
    let system = ops_system::System::new(ops_service::OpsService::new(Generated::new(Desk)), Desk);
    ops_web::install(Box::new(system));
}

/// How many times `Tally` ran.
#[no_mangle]
pub extern "C" fn ess_tallies() -> u32 {
    TALLIES.load(Ordering::Relaxed)
}
"#;

/// Drives the module through the page's own glue. Every line is `label<TAB>tallies<TAB>answer`.
const DRIVER: &str = r#"import { readFileSync } from "node:fs";

const [glue, wasm] = process.argv.slice(2);
const { open } = await import(glue);
const system = await open(readFileSync(wasm));
if (!system.realized) throw new Error("the host installs its realization");
const requests = [
  ["stuck", { request: "command", command: "ops.core.Open", input: { id: "e", count: 1 } }],
  ["ping-1", { request: "command", command: "ops.core.Ping", input: {} }],
  ["ping-2", { request: "command", command: "ops.core.Ping", input: {} }],
  ["declined", { request: "command", command: "ops.core.Open", input: { id: "g", count: 0 } }],
];
for (const [label, request] of requests) {
  const answer = JSON.stringify(system.request(request));
  console.log(`${label}\t${system.exports.ess_tallies()}\t${answer}`);
}
"#;

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

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

/// The host's output, once per test binary.
fn lines() -> &'static Vec<Vec<String>> {
    static LINES: std::sync::OnceLock<Vec<Vec<String>>> = std::sync::OnceLock::new();
    LINES.get_or_init(|| {
        let ir = ir();
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("served-correction-pass2-bridge-{}", std::process::id()));
        // The web manifest names the Rust crates at `../../../../rust/<system>`, so the two trees
        // sit where the committed ones do.
        write(
            &synthesize_for(&ir, Target::Rust).expect("the model synthesizes for Rust"),
            &root.join("generated/rust/ops"),
        );
        write(
            &synthesize_for(&ir, Target::Web).expect("the model synthesizes for the web"),
            &root.join("generated/web/ops"),
        );
        let host = root.join("host");
        std::fs::create_dir_all(host.join("src")).expect("mkdir");
        std::fs::write(
            host.join("Cargo.toml"),
            "[package]\nname = \"bridge-host\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
             [lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\n\
             ops-web = { path = \"../generated/web/ops/crates/ops-web\" }\n\
             ops-types = { path = \"../generated/rust/ops/crates/ops-types\" }\n\
             ops-system = { path = \"../generated/rust/ops/crates/ops-system\" }\n\
             ops-service = { path = \"../generated/rust/ops/crates/ops-service\" }\n\n[workspace]\n",
        )
        .expect("write");
        std::fs::write(host.join("src/lib.rs"), HOST).expect("write");
        let target = root.join("target");
        let built: Output =
            Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
                .args([
                    "build",
                    "--offline",
                    "--quiet",
                    "--target",
                    "wasm32-unknown-unknown",
                    "--target-dir",
                ])
                .arg(&target)
                .current_dir(&host)
                .env_remove("CARGO_TARGET_DIR")
                .env_remove("CARGO_ENCODED_RUSTFLAGS")
                .env("CARGO_INCREMENTAL", "0")
                .env("RUSTFLAGS", "-D warnings")
                .output()
                .expect("cargo runs");
        eprintln!("{}", String::from_utf8_lossy(&built.stderr));
        if !built.status.success() {
            let _ = std::fs::remove_dir_all(&root);
            panic!("the realized host builds for wasm32 without a warning");
        }
        std::fs::write(root.join("driver.mjs"), DRIVER).expect("write");
        let ran = Command::new("node")
            .current_dir(&root)
            .arg("driver.mjs")
            .arg(format!(
                "file://{}",
                root.join("generated/web/ops/bridge.js").display()
            ))
            .arg(target.join("wasm32-unknown-unknown/debug/bridge_host.wasm"))
            .output()
            .expect("node runs");
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        eprintln!("{}\n{stdout}", String::from_utf8_lossy(&ran.stderr));
        let _ = std::fs::remove_dir_all(&root);
        assert!(ran.status.success(), "the driver runs the realized module");
        stdout
            .lines()
            .map(|line| line.split('\t').map(str::to_owned).collect())
            .collect()
    })
}

/// `(tallies, answer)` after the request labelled `label`.
fn served(label: &str) -> (usize, String) {
    let line = lines()
        .iter()
        .find(|line| line[0] == label)
        .unwrap_or_else(|| panic!("the host printed `{label}`"));
    (line[1].parse().expect("tallies"), line[2].clone())
}

#[test]
fn the_bridge_refuses_the_command_whose_event_stuck_as_unmet() {
    let (_, answer) = served("stuck");
    assert!(
        answer.starts_with(r#"{"ok":false"#) && answer.contains(r#""kind":"unmet-obligation""#),
        "delivering the stuck `Opened` failed, and that command's answer says so: {answer}"
    );
}

/// beyond10x/ess#260: the command whose event stuck took effect, and the bridge's refusal says so
/// as a member a page reads, as the served `501` does.
#[test]
fn the_bridge_marks_the_stuck_command_as_committed() {
    let (_, answer) = served("stuck");
    assert!(
        answer.contains(r#""kind":"unmet-obligation""#) && answer.contains(r#""committed":true"#),
        "the stuck `Open` took effect before its delivery failed: {answer}"
    );
}

#[test]
fn the_bridge_keeps_answering_every_command_after_a_stuck_event() {
    for label in ["ping-1", "ping-2"] {
        let (_, answer) = served(label);
        assert!(
            answer.starts_with(r#"{"ok":true"#) && answer.contains(r#""pinged""#),
            "`{label}` published only `Pinged`, which no binding reacts to: {answer}"
        );
    }
    let (_, answer) = served("declined");
    assert!(
        answer.starts_with(r#"{"ok":true"#) && answer.contains(r#""refused""#),
        "`Open` with count 0 is the declared refusal and comes back as its outcome: {answer}"
    );
}

#[test]
fn the_binding_beside_the_stuck_one_runs_once_and_is_never_delivered_it_again() {
    let (after_stuck, answer) = served("stuck");
    assert_eq!(
        after_stuck, 1,
        "`tally-on-open` had its attempt at the stuck `Opened` beside `note-on-open`: {answer}"
    );
    let (at_end, answer) = served("declined");
    assert_eq!(
        at_end, 1,
        "three pumps later `tally-on-open` has still run once for one `Opened`: {answer}"
    );
}
