//! Correction pass 1 on `story:generated-server-publishes-and-reads-headers`, for the Go target
//! and the retry path the Rust harness of `adversary_served_pass1` does not reach.
//!
//! One model with an escalating binding, a dropping binding, a retrying binding and a command with
//! no input. The Go emission must be gofmt-clean and pass `go vet` and `go build`, and its source
//! must carry the same rules the Rust server is driven through at run time: the served dispatch
//! holds one lock, a command without input reads an empty body as `{}`, the pump passes an event
//! only after every reacting binding has had its attempt, and a binding whose attempt stops keeps
//! the event held for itself alone. The Rust emission of the same model must build with its retry
//! path.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
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
  - id: tally-on-stamp
    when:
      event: ops.core.Stamped
    invoke:
      command: ops.core.Tally
    mapping:
      id: event.id
    delivery: at_least_once
    on_failure: retry
";

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

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("served-correction-{label}-{}", std::process::id()))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn artifact<'a>(synthesis: &'a Synthesis, suffix: &str) -> &'a str {
    synthesis
        .artifacts
        .iter()
        .find(|(path, _)| path.ends_with(suffix))
        .map_or_else(
            || panic!("an artifact ending `{suffix}`"),
            |(_, artifact)| artifact.contents.as_str(),
        )
}

/// A Go tool inside a generated module, offline.
fn tool(program: &str, directory: &Path, arguments: &[&str]) -> Output {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .env("GOCACHE", scratch("gocache"))
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap_or_else(|error| panic!("`{program}` runs: {error}"));
    eprintln!(
        "{program} {arguments:?} in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn the_go_server_locks_reads_an_empty_body_and_never_passes_an_undelivered_event() {
    let synthesis = synthesize_laid_out(&ir(), Target::Go, OutputLayout::Workspace)
        .expect("the model synthesizes for Go");

    let surface = artifact(&synthesis, "server/opsservice.go");
    assert!(
        surface.contains("\tserving.Lock()\n\tdefer serving.Unlock()\n"),
        "the served dispatch holds the one lock:\n{surface}"
    );
    let ping = surface
        .split("\nfunc serveOpsCorePing(")
        .nth(1)
        .and_then(|rest| rest.split("\n}\n").next())
        .expect("the Ping handler");
    assert!(
        ping.contains("var value any = map[string]any{}")
            && ping.contains("if len(bytes.TrimSpace(body)) != 0 {"),
        "a command without input reads an empty body as `{{}}`:\n{ping}"
    );
    let open = surface
        .split("\nfunc serveOpsCoreOpen(")
        .nth(1)
        .and_then(|rest| rest.split("\n}\n").next())
        .expect("the Open handler");
    assert!(
        !open.contains("TrimSpace"),
        "a command whose contract requires a body still requires one:\n{open}"
    );

    let system = artifact(&synthesis, "system/system.go");
    // Correction 2 replaced correction 1's stop-and-rewind pump with delivery tracked per binding;
    // these pin the same two rules on the shape that now carries them.
    assert!(
        system.contains(
            "\t\tevent := s.published[s.cursor]\n\t\tif unmet := s.deliver(event); unmet != nil \
             && failed == nil {\n\t\t\tfailed = unmet\n\t\t}\n\t\ts.cursor++\n"
        ),
        "the cursor passes an event only after every reacting binding had its attempt:\n{system}"
    );
    assert!(
        system.contains(
            "\tunmet := s.deliverNoteOnOpen(event)\n\tif unmet != nil {\n\t\ts.heldNoteOnOpen = \
             append(s.heldNoteOnOpen, event)\n\t}\n"
        ) && system.contains(
            "\tfor _, event := range heldNoteOnOpen {\n\t\ts.attemptNoteOnOpen(event)\n\t}\n"
        ),
        "a delivery that stops keeps its event held for that binding alone, and a held-back \
         attempt that stops again is held again:\n{system}"
    );

    let directory = scratch("go");
    write(&synthesis, &directory);
    let unformatted = tool("gofmt", &directory, &["-l", "."]);
    assert!(unformatted.status.success(), "gofmt runs");
    assert_eq!(
        String::from_utf8_lossy(&unformatted.stdout),
        "",
        "the generated Go is gofmt-clean"
    );
    assert!(
        tool("go", &directory, &["vet", "./..."]).status.success(),
        "`go vet` accepts the generated module"
    );
    assert!(
        tool("go", &directory, &["build", "./..."]).status.success(),
        "`go build` accepts the generated module"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn the_rust_pump_passes_an_event_only_after_delivery_and_keeps_held_events_and_builds() {
    let synthesis = synthesize_laid_out(&ir(), Target::Rust, OutputLayout::Workspace)
        .expect("the model synthesizes for Rust");
    let system = artifact(&synthesis, "ops-system/src/lib.rs");
    // Correction 2 replaced correction 1's stop-and-rewind pump with delivery tracked per binding;
    // these pin the same two rules on the shape that now carries them.
    assert!(
        system.contains(
            "            let event = self.published[self.cursor].clone();\n            if let \
             Err(failure) = self.deliver(&event) {\n                failed = \
             failed.or(Some(failure));\n            }\n            self.cursor += 1;\n"
        ),
        "the cursor passes an event only after every reacting binding had its attempt:\n{system}"
    );
    assert!(
        system.contains(
            "        match self.deliver_note_on_open(event) {\n            Ok(()) => Ok(()),\n            \
             Err(failure) => {\n                self.held_note_on_open.push(event.clone());\n"
        ) && system.contains(
            "        for event in held_note_on_open {\n            let _ = \
             self.attempt_note_on_open(&event);\n        }\n"
        ),
        "a delivery that stops keeps its event held for that binding alone, and a held-back \
         attempt that stops again is held again:\n{system}"
    );

    let directory = scratch("rust");
    write(&synthesis, &directory);
    let built = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args([
            "check",
            "--offline",
            "--quiet",
            "--workspace",
            "--target-dir",
        ])
        .arg(directory.join("target"))
        .current_dir(&directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo runs");
    let _ = std::fs::remove_dir_all(&directory);
    assert!(
        built.status.success(),
        "the generated workspace builds without a warning:\n{}",
        String::from_utf8_lossy(&built.stderr)
    );
}
