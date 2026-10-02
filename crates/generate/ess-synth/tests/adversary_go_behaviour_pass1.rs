//! Adversary pass 1 on `story:go-generated-behaviour`: the Go target generates every behaviour,
//! query and invariant check the plan marks generated, at parity with the Rust target.
//!
//! Each case below holds the Go emission to something the Rust emission, the committed Rust
//! example or the unit's own documentation already does:
//!
//! - the committed gatepass servers, Rust and Go, answer the same view after the same commands;
//! - a domain whose package name is one of the generated behaviour package's private helpers still
//!   yields a module that builds, as it does in Rust, where the helpers live in their own module;
//! - an entity field named `broken_invariant` still yields a data type that builds, as Rust's does;
//! - two owed seams whose Go method names coincide are not dropped from `behaviour.Generated`
//!   without `TARGET.md` saying so.

use std::io::{BufRead as _, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, CapabilityKind, Synthesis, Target};
use serde_json::{json, Value};

// ---- shared ----------------------------------------------------------------------------------

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn scratch(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-go-behaviour-{name}-{}",
        std::process::id()
    ))
}

/// Compiles specification documents given as `(label, text)`.
fn fixture(documents: &[(&str, &str)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (label, text) in documents {
        let raw = RawSpecFile::parse(text)
            .unwrap_or_else(|error| panic!("the fixture `{label}` is well formed: {error}"));
        sources.insert((*label).to_owned(), (*text).to_owned());
        labels.push((*label).to_owned());
        parsed.push((Source::new((*label).to_owned()), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn write_tree(directory: &Path, synthesis: &Synthesis) {
    let _ = std::fs::remove_dir_all(directory);
    for artifact in synthesis.artifacts.values() {
        let path = directory.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
}

/// Runs a Go tool in `directory` with nothing fetched from a network.
fn go_tool(directory: &Path, arguments: &[&str]) -> Output {
    Command::new("go")
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .env("CGO_ENABLED", "0")
        .output()
        .expect("the Go tool runs")
}

fn go_available() -> bool {
    Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Builds every package of a synthesized Go tree; `Err` carries the compiler's words.
fn go_builds(label: &str, synthesis: &Synthesis) -> Result<(), String> {
    let tree = scratch(label);
    write_tree(&tree, synthesis);
    let built = go_tool(&tree, &["build", "./..."]);
    let _ = std::fs::remove_dir_all(&tree);
    if built.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{}{}",
            String::from_utf8_lossy(&built.stdout),
            String::from_utf8_lossy(&built.stderr)
        ))
    }
}

// ---- the committed gatepass servers ----------------------------------------------------------

fn rust_server() -> PathBuf {
    let target = scratch("rust-target");
    let built = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args([
            "build",
            "--offline",
            "--quiet",
            "--bin",
            "gatepass-server",
            "--manifest-path",
        ])
        .arg(root().join("examples/gatepass-realization/Cargo.toml"))
        .arg("--target-dir")
        .arg(&target)
        .env_remove("CARGO_TARGET_DIR")
        .env("CARGO_INCREMENTAL", "0")
        .env_remove("RUSTC_WRAPPER")
        .output()
        .expect("cargo runs");
    assert!(
        built.status.success(),
        "the Rust gatepass server builds: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    target.join("debug/gatepass-server")
}

fn go_server() -> PathBuf {
    let binary = scratch("go-server");
    let built = Command::new("go")
        .arg("build")
        .arg("-o")
        .arg(&binary)
        .arg("./cmd/gatepass-server")
        .current_dir(root().join("examples/gatepass-go-realization"))
        .env("GOPROXY", "off")
        .env("CGO_ENABLED", "0")
        .output()
        .expect("go build runs");
    assert!(
        built.status.success(),
        "the Go gatepass server builds: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    binary
}

struct Served(Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn serve(binary: &Path) -> (Served, u16) {
    let mut child = Command::new(binary)
        .env("PORT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the server starts");
    let stdout = child.stdout.take().expect("piped");
    let mut lines = std::io::BufReader::new(stdout).lines();
    let first = lines.next().expect("a startup record").expect("text");
    let record: Value = serde_json::from_str(&first).expect("the record is JSON");
    let port = record["runtime"]["port"]
        .as_u64()
        .and_then(|port| u16::try_from(port).ok())
        .expect("the record names the port");
    std::thread::spawn(move || lines.for_each(drop));
    (Served(child), port)
}

fn request(port: u16, method: &str, path: &str, body: &str) -> (u16, Value) {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("the server accepts");
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n\
         Authorization: Actor gatepass.visit.Receptionist\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )
    .expect("the request writes");
    let mut answer = String::new();
    stream
        .read_to_string(&mut answer)
        .expect("the answer reads");
    let (head, payload) = answer.split_once("\r\n\r\n").expect("a head and a body");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|status| status.parse().ok())
        .expect("a status");
    let payload =
        serde_json::from_str(payload).unwrap_or_else(|_| Value::String(payload.to_owned()));
    (status, payload)
}

/// Registers one visit, admits it with a badge, and reads both views — every answer with the
/// visit's identity replaced by `{V1}`.
fn register_admit_and_read(port: u16) -> Vec<(&'static str, u16, Value)> {
    let register = json!({
        "visitor": "Ada",
        "building": "North",
        "host": {"kind": "employee", "value": "E1"},
        "expected_minutes": 30,
        "expected_stay": "PT30M",
        "deposit": {"amount": "10.50", "currency": "EUR"},
        "escorts": ["Bo"],
        "notes": {"k": "v"},
        "on_watchlist": false,
    })
    .to_string();
    let (status, registered) = request(port, "POST", "/visits/commands/register-visit", &register);
    assert_eq!(status, 202, "the registration is accepted: {registered}");
    let id = registered["published"][0]["payload"]["visit_id"]
        .as_str()
        .expect("the created identity is in the payload")
        .to_owned();
    let badge = r#"{"serial":"s-1","printed_at":"2026-09-30T08:00:00Z","signature":"AAEC"}"#;
    let admit = format!(r#"{{"visit_id":"{id}","badge":{badge}}}"#);
    let mut answers = Vec::new();
    for (label, method, path, body) in [
        (
            "admit",
            "POST",
            "/visits/commands/admit-visitor",
            admit.as_str(),
        ),
        ("by-id", "GET", "/visits/views/by-id", ""),
        ("expected", "GET", "/visits/views/expected", ""),
    ] {
        let (status, answer) = request(port, method, path, body);
        let text = answer.to_string().replace(&id, "{V1}");
        answers.push((label, status, serde_json::from_str(&text).expect("JSON")));
    }
    answers
}

/// The committed examples are the demonstration that the two trees answer the same requests the
/// same way (`examples/gatepass-go-realization/visit.go`). Before this unit the Go realization
/// hand-wrote `AdmitVisitor` as the Rust realization still does, storing the admitted badge; now
/// the Go server runs the generated behaviour, which stores no badge, and `VisitById` reads one.
#[test]
fn adversary_the_rust_and_go_gatepass_servers_answer_one_view_identically_after_an_admission() {
    if !go_available() {
        eprintln!("no Go toolchain on this machine; the two gatepass servers are not compared");
        return;
    }
    let rust = rust_server();
    let go = go_server();
    let rust_answers = {
        let (_served, port) = serve(&rust);
        register_admit_and_read(port)
    };
    let go_answers = {
        let (_served, port) = serve(&go);
        register_admit_and_read(port)
    };
    let _ = std::fs::remove_file(&go);
    let _ = std::fs::remove_dir_all(scratch("rust-target"));
    let mut problems = Vec::new();
    for ((label, rust_status, rust_answer), (_, go_status, go_answer)) in
        rust_answers.iter().zip(&go_answers)
    {
        if (rust_status, rust_answer) != (go_status, go_answer) {
            problems.push(format!(
                "`{label}`: Rust {rust_status} {rust_answer}\n            but Go {go_status} \
                 {go_answer}"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "the two gatepass servers answer differently:\n{}",
        problems.join("\n")
    );
}

// ---- name collisions -------------------------------------------------------------------------

/// One bounded context named `domain`, with a generated move, an invariant, and a generated filtered
/// query; `extra_field` is one more stored field, or nothing.
fn probe_model(domain: &str, extra_field: &str) -> String {
    let d = format!("probe.{domain}");
    format!(
        "format: ess/18
system: probe
version: v1
domain: {d}
types:
  - {{name: {d}.TaskId, kind: newtype, of: Uuid}}
errors:
  - name: {d}.AlreadyDone
    summary: The task is already done.
events:
  - name: {d}.TaskDone
    fields:
      - {{name: task_id, type: {d}.TaskId}}
entities:
  - name: {d}.Task
    identity: {{name: task_id, type: {d}.TaskId}}
    fields:
      - {{name: owner, type: String}}
      - {{name: hours, type: Integer}}{extra_field}
    invariants:
      - hours >= 0
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {{name: finish, from: [Open], to: Done}}
commands:
  - name: {d}.FinishTask
    input:
      - {{name: task_id, type: {d}.TaskId}}
    outcomes:
      - name: finished
        moves: {d}.Task.finish
        instance: task_id
        emits: [{d}.TaskDone]
        payload:
          {d}.TaskDone:
            task_id: input.task_id
      - {{name: already-done, wrong_state: true, error: {d}.AlreadyDone}}
views:
  - name: {d}.OpenTasks
    source: {d}.Task
    consistency: read_your_writes
    filter: state == Open
    fields:
      - {{name: task_id, type: {d}.TaskId}}
      - {{name: owner, type: String}}
components:
  - component: probe-service
    summary: Holds every task.
    owns: {{domains: [{d}]}}
    accepts: {{commands: [{d}.FinishTask]}}
    publishes: {{events: [{d}.TaskDone]}}
"
    )
}

/// Synthesizes the probe for both targets, asserting what the plan generates.
fn both_targets(model: &str) -> (Synthesis, Synthesis) {
    let ir = fixture(&[("model.yaml", model)]);
    let rust = synthesize_for(&ir, Target::Rust).expect("the probe synthesizes to Rust");
    let go = synthesize_for(&ir, Target::Go).expect("the probe synthesizes to Go");
    let command = ir
        .commands()
        .values()
        .next()
        .expect("one command")
        .name
        .to_string();
    assert!(
        rust.plan
            .is_generated(CapabilityKind::CommandBehavior, &command),
        "the probe's command is generated"
    );
    assert!(
        go.artifacts.contains_key("types/behaviour/behaviour.go"),
        "the Go tree carries the generated behaviours"
    );
    (rust, go)
}

/// `behaviour.go` imports every domain package it spells and declares its private helpers at
/// package level, so a domain whose package name is a helper's (`equal`, `some`, `verity`) is a
/// file that declares one name twice. Rust reaches every domain through `crate::` paths and keeps
/// its helpers private to the `behaviour` module, so the same model is one Rust emits. The
/// generator moves locals out of the way of package names (`Locals::new`, `fresh`), never helpers.
#[test]
fn adversary_a_domain_named_like_a_behaviour_helper_still_builds_in_go() {
    if !go_available() {
        eprintln!("no Go toolchain on this machine; the helper collision is unchecked");
        return;
    }
    let (_, control) = both_targets(&probe_model("work", ""));
    go_builds("control", &control).unwrap_or_else(|log| {
        panic!("the probe itself builds under a neutral domain name:\n{log}")
    });
    let mut broken = Vec::new();
    for domain in ["equal", "some", "verity"] {
        let (_, go) = both_targets(&probe_model(domain, ""));
        if let Err(log) = go_builds(domain, &go) {
            broken.push(format!("domain `probe.{domain}`:\n{log}"));
        }
    }
    assert!(
        broken.is_empty(),
        "the Go module does not build where the Rust workspace is emitted:\n{}",
        broken.join("\n")
    );
}

/// `BrokenInvariant` is a new method on every entity data type that declares an invariant, and Go
/// gives a struct one namespace for fields and methods. A stored field named `broken_invariant`
/// exports to the same identifier; Rust keeps fields and methods apart, so its data type carries
/// both.
#[test]
fn adversary_a_field_named_broken_invariant_still_builds_in_go() {
    if !go_available() {
        eprintln!("no Go toolchain on this machine; the method collision is unchecked");
        return;
    }
    let (rust, go) = both_targets(&probe_model(
        "work",
        "\n      - {name: broken_invariant, type: Boolean}",
    ));
    let rust_domain = &rust.artifacts["crates/probe-types/src/work.rs"].contents;
    assert!(
        rust_domain.contains("pub broken_invariant: bool")
            && rust_domain.contains("pub fn broken_invariant(&self)"),
        "the Rust data type carries the field and the check side by side"
    );
    go_builds("broken-invariant", &go)
        .unwrap_or_else(|log| panic!("the Go module does not build:\n{log}"));
}

/// Two bounded contexts, each served by its own component, each with an owed `Place` and a
/// generated `Ping<Domain>`.
fn owed_twins() -> EssIr {
    let context = |domain: &str, ping: &str| {
        format!(
            "domain: twice.{domain}
errors:
  - name: twice.{domain}.Rejected
    summary: The note is refused.
    fields:
      - {{name: reason, type: String}}
events:
  - name: twice.{domain}.Placed
    fields:
      - {{name: note, type: String}}
commands:
  - name: twice.{domain}.Place
    input:
      - {{name: note, type: String}}
    outcomes:
      - name: rejected
        when: note == \"no\"
        error: twice.{domain}.Rejected
      - name: done
        emits: [twice.{domain}.Placed]
        payload:
          twice.{domain}.Placed:
            note: input.note
  - name: twice.{domain}.{ping}
    input:
      - {{name: note, type: String}}
    outcomes:
      - name: done
        emits: [twice.{domain}.Placed]
        payload:
          twice.{domain}.Placed:
            note: input.note
"
        )
    };
    let component = |domain: &str, ping: &str| {
        format!(
            "  - component: {domain}-desk
    owns:
      domains: [twice.{domain}]
    accepts:
      commands: [twice.{domain}.Place, twice.{domain}.{ping}]
    publishes:
      events: [twice.{domain}.Placed]
"
        )
    };
    fixture(&[
        (
            "system.yaml",
            "format: ess/1\nsystem: twice\nversion: v1\ndomains:\n  - twice.alpha\n  - \
             twice.beta\n",
        ),
        ("alpha.yaml", &context("alpha", "PingAlpha")),
        ("beta.yaml", &context("beta", "PingBeta")),
        (
            "wiring.yaml",
            &format!(
                "components:\n{}{}",
                component("alpha", "PingAlpha"),
                component("beta", "PingBeta")
            ),
        ),
    ])
}

/// The unit's own words (`website/docs/guides/synthesize.md`, the `behaviour` package doc): the
/// `*Generated` is "a complete bundle for every component port", and where two seams' Go method
/// names coincide "`TARGET.md` names the weakening". `Seams::of` drops both owed `Place` seams
/// from `Generated` — neither a method nor forwarded — and `one_method_set` is pushed only when a
/// *generated* seam collides, so nothing names the hole: `alpha_desk.New(behaviour.New(…))` does
/// not compile, and neither document says why.
#[test]
fn adversary_two_owed_seams_dropped_from_generated_are_named_in_target_md() {
    let ir = owed_twins();
    let go = synthesize_for(&ir, Target::Go).expect("the fixture synthesizes to Go");
    for (kind, source, generated) in [
        (CapabilityKind::CommandBehavior, "twice.alpha.Place", false),
        (CapabilityKind::CommandBehavior, "twice.beta.Place", false),
        (
            CapabilityKind::CommandBehavior,
            "twice.alpha.PingAlpha",
            true,
        ),
        (CapabilityKind::CommandBehavior, "twice.beta.PingBeta", true),
    ] {
        assert_eq!(
            go.plan.is_generated(kind, source),
            generated,
            "`{source}` is {} by the plan",
            if generated { "generated" } else { "owed" }
        );
    }
    let behaviour = &go.artifacts["types/behaviour/behaviour.go"].contents;
    let target = &go.artifacts["TARGET.md"].contents;
    let carried = behaviour.contains(") Place(input ");
    let named = target.contains("`Place`") || target.contains("twice.alpha.Place");
    assert!(
        carried || named,
        "both components accept a seam `Generated` does not carry, and TARGET.md is silent:\n\
         --- behaviour.go ---\n{behaviour}\n--- TARGET.md ---\n{target}"
    );
}
