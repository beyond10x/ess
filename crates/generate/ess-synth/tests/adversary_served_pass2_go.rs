//! Adversary pass 2 on `story:generated-server-publishes-and-reads-headers`, the Go half of
//! `adversary_served_pass2`: the same model, served by the generated Go surface, with the system's
//! obligations left to the generated `system.Unimplemented` stub — the value a Go shell passes until
//! it has realized the escalation.
//!
//! One `Opened` whose `note-on-open` delivery needs that escalation (the notebook is full) is
//! undeliverable while the stub stands. A `Ping` served after it published only `Pinged`, which no
//! binding reacts to, and a declared refusal publishes nothing; the contract declares 202 and 422
//! for them.
//!
//! The Go harness is an in-package test of the generated `server` package, so it drives
//! `dispatchOpsService` with `httptest` requests and opens no socket; `go test -timeout` bounds it.

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
";

/// Every line is `step<TAB>label<TAB>status<TAB>log<TAB>tallies<TAB>body`.
const HARNESS: &str = r#"package server

import (
	"fmt"
	"net/http/httptest"
	"strings"
	"testing"

	"example.invalid/ops/components/opsservice"
	"example.invalid/ops/system"
	"example.invalid/ops/types/core"
	"example.invalid/ops/types/obligation"
)

type desk struct{ tallies int }

func (d *desk) Open(input core.Open) (core.OpenOutcome, *obligation.UnmetObligation) {
	if input.Count > 0 {
		return core.OpenOutcomeOpened{Opened: core.Opened{Id: input.Id}, Stamped: core.Stamped{Id: input.Id}}, nil
	}
	return core.OpenOutcomeRefused{}, nil
}

// The notebook stays full.
func (d *desk) Note(input core.Note) (core.NoteOutcome, *obligation.UnmetObligation) {
	return core.NoteOutcomeBusy{}, nil
}

func (d *desk) Tally(input core.Tally) (core.TallyOutcome, *obligation.UnmetObligation) {
	d.tallies++
	return core.TallyOutcomeTallied{Tallied: core.Tallied{Id: input.Id}}, nil
}

func (d *desk) Ping(input core.Ping) (core.PingOutcome, *obligation.UnmetObligation) {
	return core.PingOutcomePinged{}, nil
}

func TestAdversaryPass2(t *testing.T) {
	d := &desk{}
	s := system.NewSystem(opsservice.New(d), system.Unimplemented{})
	step := func(label, path, body string) {
		request := httptest.NewRequest("POST", path, strings.NewReader(body))
		answer := dispatchOpsService(s, request)
		fmt.Printf("step\t%s\t%d\t%d\t%d\t%s\n", label, answer.status, len(s.Published()), d.tallies, strings.ReplaceAll(answer.body, "\n", " "))
	}
	step("stuck", "/core/commands/Open", `{"id":"e","count":1}`)
	for round := 1; round <= 5; round++ {
		step(fmt.Sprintf("ping-%d", round), "/core/commands/Ping", "{}")
	}
	step("declined", "/core/commands/Open", `{"id":"g","count":0}`)
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

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-served-pass2-go-{label}-{}",
        std::process::id()
    ))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn go(directory: &Path, arguments: &[&str]) -> Output {
    let output = Command::new("go")
        .args(arguments)
        .current_dir(directory)
        .env("GOCACHE", scratch("gocache"))
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap_or_else(|error| panic!("`go` runs: {error}"));
    eprintln!(
        "go {arguments:?} in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn lines() -> &'static Vec<Vec<String>> {
    static LINES: std::sync::OnceLock<Vec<Vec<String>>> = std::sync::OnceLock::new();
    LINES.get_or_init(|| {
        let synthesis = synthesize_laid_out(&ir(), Target::Go, OutputLayout::Workspace)
            .expect("the model synthesizes for Go");
        let directory = scratch("module");
        write(&synthesis, &directory);
        std::fs::write(directory.join("server/adversary_pass2_test.go"), HARNESS).expect("write");
        let ran = go(
            &directory,
            &[
                "test",
                "-count=1",
                "-timeout",
                "60s",
                "-v",
                "-run",
                "TestAdversaryPass2",
                "./server/",
            ],
        );
        let _ = std::fs::remove_dir_all(&directory);
        let _ = std::fs::remove_dir_all(scratch("gocache"));
        assert!(ran.status.success(), "the Go harness builds and runs");
        String::from_utf8_lossy(&ran.stdout)
            .lines()
            .filter(|line| line.starts_with("step\t"))
            .map(|line| line.split('\t').skip(1).map(str::to_owned).collect())
            .collect()
    })
}

fn served(label: &str) -> (u16, usize, usize, String) {
    let line = lines()
        .iter()
        .find(|line| line[0] == label)
        .unwrap_or_else(|| panic!("the Go harness printed `{label}`"));
    (
        line[1].parse().expect("a status"),
        line[2].parse().expect("log"),
        line[3].parse().expect("tallies"),
        line[4].clone(),
    )
}

#[test]
fn the_go_server_answers_a_command_whose_events_no_binding_reacts_to_after_an_undeliverable_event()
{
    let (stuck, _, _, body) = served("stuck");
    assert_eq!(stuck, 501, "{body}");
    let (status, _, _, body) = served("ping-1");
    assert_eq!(
        status, 202,
        "`Ping` published only `Pinged`, which no binding reacts to, and was answered: {body}"
    );
}

#[test]
fn the_go_server_answers_a_declared_refusal_as_declared_after_an_undeliverable_event() {
    let (status, _, _, body) = served("declined");
    assert_eq!(
        status, 422,
        "`Open` with count 0 is the declared refusal and was answered: {body}"
    );
}

/// `tally-on-open` reacts to `Opened` and never fails; at least once means it receives the stuck
/// `Opened` while the server goes on serving, whatever `note-on-open` still owes.
#[test]
fn the_go_server_delivers_the_stuck_event_to_the_binding_beside_the_unmet_one() {
    let (_, log, tallies, _) = served("declined");
    assert!(
        tallies >= 1,
        "after six more served commands `tally-on-open` ran {tallies} times for the stuck \
         `Opened`; the log holds {log} events"
    );
}
