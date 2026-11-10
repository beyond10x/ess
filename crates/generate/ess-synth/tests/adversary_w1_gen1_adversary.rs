//! Adversary pass on unit GEN1 of wave W1:
//! `story:the-generated-rust-server-refuses-what-its-openapi-refuses` and
//! `story:a-json-command-body-refuses-an-unknown-member`.
//!
//! 1. The new `json::timestamp_at` is compared with the served document's `format: date-time`,
//!    as the `jsonschema` validator the unit's own agreement test uses asserts it. Whether it
//!    admits only instants the server can order is backlog
//!    `story:an-admitted-timestamp-is-an-instant-the-server-can-order`.
//! 2. The generated Go server without a binding is sent a JSON command body with a member no
//!    input field names; its routes read the default `request`, whose `unknown` is `refuse`.
//!
//! The reader under test is the committed generated copy of the emitter's `json.rs`, which the
//! projection check holds to the emitter's bytes.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_laid_out, OutputLayout, Synthesis, Target};
use serde_json::Value;

// `rustfmt::skip`, so `cargo fmt -p ess-synth` never rewrites the byte-pinned generated file.
#[allow(dead_code, clippy::all, clippy::pedantic)]
#[rustfmt::skip]
#[path = "../../../../generated/rust/gatepass/crates/gatepass-server/src/json.rs"]
mod generated_json;

/// Whether the generated decoder admits `text` as a `Timestamp`.
fn decoder_admits(text: &str) -> bool {
    generated_json::timestamp_at(
        &generated_json::Value::Text(text.to_owned()),
        "body.at",
        "an RFC 3339 instant",
    )
    .is_ok()
}

/// A fixed corpus over every component boundary of a `date-time`: no randomness, so a failure
/// reproduces on any machine.
fn corpus() -> Vec<String> {
    let years = ["0000", "1900", "2000", "2016", "2024", "2026", "9999"];
    let months = ["00", "01", "02", "04", "12", "13"];
    let days = ["00", "28", "29", "30", "31", "32"];
    let hours = ["00", "18", "23", "24"];
    let minutes = ["00", "29", "59", "60"];
    let seconds = ["59", "60", "61"];
    let fractions = ["", ".", ".5", ".123456789", ".1234567890"];
    let offsets = [
        "Z", "z", "+00:00", "-00:00", "+05:30", "-05:30", "+23:59", "+24:00", "-00:60", "+0530",
    ];
    let mut out = Vec::new();
    for year in years {
        for month in months {
            for day in days {
                for hour in hours {
                    for minute in minutes {
                        for second in seconds {
                            for fraction in fractions {
                                for offset in offsets {
                                    out.push(format!(
                                        "{year}-{month}-{day}T{hour}:{minute}:{second}{fraction}{offset}"
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// The decoder admits exactly what the served `format: date-time` admits, over the corpus, as the
/// validator the unit's own agreement test uses asserts the format.
#[test]
fn the_timestamp_decoder_admits_exactly_what_the_served_date_time_format_admits() {
    let schema = serde_json::json!({"type": "string", "format": "date-time"});
    let validator = schema_contract::uniqueness::options()
        .should_validate_formats(true)
        .build(&schema)
        .expect("the schema compiles");
    let mut failures = Vec::new();
    let mut admitted = 0;
    for text in corpus() {
        let document = validator.is_valid(&Value::String(text.clone()));
        if document {
            admitted += 1;
        }
        if document != decoder_admits(&text) {
            failures.push(format!("`{text}`: the document admits={document}"));
        }
    }
    assert!(
        admitted > 1_000,
        "the corpus holds admitted texts: {admitted}"
    );
    assert!(
        failures.is_empty(),
        "{} disagreement(s), e.g. {:?}",
        failures.len(),
        &failures[..failures.len().min(6)]
    );
}

const GO_MODEL: &str = "format: ess/15
system: ops
version: v1
domain: ops.core
events:
  - name: ops.core.Opened
    fields:
      - {name: id, type: String}
commands:
  - name: ops.core.Open
    input:
      - {name: id, type: String}
      - {name: count, type: Integer}
    outcomes:
      - name: opened
        emits: [ops.core.Opened]
        payload:
          ops.core.Opened: {id: input.id}
components:
  - component: ops-service
    owns: {domains: [ops.core]}
    accepts: {commands: [ops.core.Open]}
    publishes: {events: [ops.core.Opened]}
    reached_by: network
";

/// Every line is `step<TAB>label<TAB>status<TAB>runs<TAB>body`, `runs` counting the port calls so
/// far. `@PATH@` is the command's route, read from the served document.
const GO_HARNESS: &str = r#"package server

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

type desk struct{ ran int }

func (d *desk) Open(input core.Open) (core.OpenOutcome, *obligation.UnmetObligation) {
	d.ran++
	return core.OpenOutcomeOpened{Opened: core.Opened{Id: input.Id}}, nil
}

func TestAdversaryW1Gen1(t *testing.T) {
	d := &desk{}
	s := system.NewSystem(opsservice.New(d))
	step := func(label, body string) {
		request := httptest.NewRequest("POST", "@PATH@", strings.NewReader(body))
		request.Header.Set("Content-Type", "application/json")
		answer := dispatchOpsService(s, request)
		fmt.Printf("step\t%s\t%d\t%d\t%s\n", label, answer.status, d.ran, strings.ReplaceAll(answer.body, "\n", " "))
	}
	step("declared", `{"id":"a","count":1}`)
	step("unknown", `{"id":"b","count":1,"colour":"red"}`)
}
"#;

fn go_ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("ops.yaml"),
        RawSpecFile::parse(GO_MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("ops.yaml", GO_MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-w1-gen1-adversary-{label}-{}",
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

/// A JSON command body with a member no input field names is refused with `400` by the generated
/// Go server serving the model without a binding, whose routes read the default `request`
/// (`unknown: refuse`), and the command does not run. Its served document closes the input, and
/// the generated Rust server for the same model now refuses the body in `decode_command_*`.
#[test]
fn the_go_server_without_a_binding_refuses_a_member_no_input_field_names() {
    let synthesis = synthesize_laid_out(&go_ir(), Target::Go, OutputLayout::Workspace)
        .expect("the model synthesizes for Go");
    let (_, openapi) = synthesis
        .artifacts
        .iter()
        .find(|(relative, _)| relative.ends_with(".openapi.json"))
        .expect("the Go server serves a document");
    let document: Value = serde_json::from_str(&openapi.contents).expect("JSON");
    let (route, item) = document["paths"]
        .as_object()
        .expect("paths")
        .iter()
        .find(|(_, item)| item.get("post").is_some())
        .expect("the command's route");
    let reference = item["post"]["requestBody"]["content"]["application/json"]["schema"]["$ref"]
        .as_str()
        .expect("the body is a reference");
    let input = reference.rsplit('/').next().expect("a name");
    assert_eq!(
        document["components"]["schemas"][input]["additionalProperties"],
        serde_json::json!(false),
        "the served document closes the command's input"
    );
    let directory = scratch("module");
    write(&synthesis, &directory);
    std::fs::write(
        directory.join("server/adversary_w1_gen1_test.go"),
        GO_HARNESS.replace("@PATH@", route),
    )
    .expect("write");
    let ran = go(
        &directory,
        &[
            "test",
            "-count=1",
            "-timeout",
            "120s",
            "-v",
            "-run",
            "TestAdversaryW1Gen1",
            "./server/",
        ],
    );
    let _ = std::fs::remove_dir_all(&directory);
    let _ = std::fs::remove_dir_all(scratch("gocache"));
    assert!(ran.status.success(), "the Go harness builds and runs");
    let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
    let answer = |label: &str| -> (u16, usize, String) {
        let line: Vec<&str> = stdout
            .lines()
            .find(|line| line.starts_with(&format!("step\t{label}\t")))
            .unwrap_or_else(|| panic!("the harness answered `{label}`:\n{stdout}"))
            .split('\t')
            .collect();
        (
            line[2].parse().expect("a status"),
            line[3].parse().expect("a count"),
            line[4].to_owned(),
        )
    };
    let (declared, runs, body) = answer("declared");
    assert!(
        (200..300).contains(&declared) && runs == 1,
        "the declared body runs: {declared} {body}"
    );
    let (unknown, runs, body) = answer("unknown");
    assert_eq!(
        (unknown, runs),
        (400, 1),
        "story:a-json-command-body-refuses-an-unknown-member: `colour` is a member no input \
         field of `ops.core.Open` names, on a route whose `unknown` is the default `refuse`; \
         the Go server answered {unknown} after {runs} port call(s) in all: {body}"
    );
}
