//! Adversary cases for unit variants through the generated Rust and Go targets (ess/22,
//! beyond10x/ess#418, `docs/design/union-unit-variants.md`).
//!
//! The implementor's cases use one union, `Status = Open | Complete`, tagged `kind`. These add:
//!
//! - `Choice = Alpha | Zulu`, a union of only unit variants (the design page's alternative to an
//!   enum): no type-switch binding is read, no payload codec is emitted;
//! - `Tagged = Alpha(String) | Zulu`, tagged `value`, so the content key is `content` and a unit
//!   variant is `{"value": "Zulu"}`;
//! - `Status` inside a `List<…>` and an `Optional<…>`.
//!
//! Each target is built with warnings denied, and its codecs and served command are run.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Synthesis, Target};

const MODEL: &str = "format: ess/22
system: demo
version: v1
domain: demo.work
types:
  - name: demo.work.Completion
    kind: struct
    fields:
      - {name: outcome, type: String}
  - name: demo.work.Status
    kind: union
    tag: kind
    variants:
      Open:
      Complete: demo.work.Completion
  - name: demo.work.Choice
    kind: union
    tag: kind
    variants:
      Alpha:
      Zulu:
  - name: demo.work.Tagged
    kind: union
    tag: value
    variants:
      Alpha: String
      Zulu:
events:
  - name: demo.work.Reported
    fields:
      - {name: statuses, type: List<demo.work.Status>}
      - {name: maybe, type: Optional<demo.work.Status>}
      - {name: choice, type: demo.work.Choice}
      - {name: tagged, type: demo.work.Tagged}
actors:
  - name: demo.work.Reporter
    may: [demo.work.Report]
commands:
  - name: demo.work.Report
    input:
      - {name: statuses, type: List<demo.work.Status>}
      - {name: maybe, type: Optional<demo.work.Status>}
      - {name: choice, type: demo.work.Choice}
      - {name: tagged, type: demo.work.Tagged}
    outcomes:
      - name: reported
        emits: [demo.work.Reported]
        payload:
          demo.work.Reported:
            statuses: input.statuses
            maybe: input.maybe
            choice: input.choice
            tagged: input.tagged
components:
  - component: work-service
    owns:
      domains: [demo.work]
    accepts:
      commands: [demo.work.Report]
    publishes:
      events: [demo.work.Reported]
    reached_by: network
";

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("work.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("work.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "union-unit-variants-adversary-{label}-{}",
        std::process::id()
    ))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&destination, &artifact.contents).unwrap();
    }
}

fn run(mut command: Command, label: &str) -> Output {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{label} runs: {error}"));
    eprintln!(
        "{label}: {command:?}\n{}{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        output.status
    );
    output
}

fn cargo(directory: &Path, target: &Path, arguments: &[&str]) -> Output {
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .args(arguments)
        .current_dir(directory)
        .env("CARGO_TARGET_DIR", target)
        .env("RUSTFLAGS", "-D warnings")
        .env("CARGO_INCREMENTAL", "0")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER");
    run(command, "cargo")
}

fn go(directory: &Path, arguments: &[&str]) -> Output {
    let mut command = Command::new("go");
    command
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .env("GOTOOLCHAIN", "local");
    if std::env::var_os("GOCACHE").is_none() {
        command.env("GOCACHE", scratch("gocache"));
    }
    run(command, "go")
}

const RUST_HARNESS: &str = r##"
#![cfg(test)]
use demo_server::{json, wire, work_service as surface};
use demo_types::work::{Choice, Tagged};

fn caller() -> demo_types::actor::Caller {
    demo_types::actor::Caller { actor: demo_types::actor::Actor::ALL[0] }
}

fn system() -> demo_system::System<demo_types::behaviour::Generated<()>> {
    demo_system::System::new(work_service::WorkService::new(demo_types::behaviour::Generated::new(())))
}

#[test]
fn a_union_of_only_unit_variants_round_trips_as_its_tag_alone() {
    for (value, text) in [(Choice::Alpha, r#"{"kind":"Alpha"}"#), (Choice::Zulu, r#"{"kind":"Zulu"}"#)] {
        let mut encoded = String::new();
        wire::encode_demo_work_choice(&value, &mut encoded);
        assert_eq!(encoded, text);
        assert_eq!(wire::decode_demo_work_choice(&json::parse(text).unwrap(), "choice").unwrap(), value);
    }
    for malformed in [r#"{"kind":"Alpha","value":null}"#, r#"{"kind":"Zulu","value":{}}"#] {
        assert!(wire::decode_demo_work_choice(&json::parse(malformed).unwrap(), "choice").is_err(), "{malformed}");
    }
}

#[test]
fn a_union_tagged_value_writes_its_payload_under_content_and_its_unit_variant_as_the_tag() {
    for (value, text) in [
        (Tagged::Alpha("x".into()), r#"{"value":"Alpha","content":"x"}"#),
        (Tagged::Zulu, r#"{"value":"Zulu"}"#),
    ] {
        let mut encoded = String::new();
        wire::encode_demo_work_tagged(&value, &mut encoded);
        assert_eq!(encoded, text);
        assert_eq!(wire::decode_demo_work_tagged(&json::parse(text).unwrap(), "tagged").unwrap(), value);
    }
    let refused = wire::decode_demo_work_tagged(&json::parse(r#"{"value":"Zulu","content":"x"}"#).unwrap(), "tagged")
        .expect_err("a unit variant carries no content member");
    assert_eq!(refused.at, "tagged.content", "{refused:?}");
    assert!(wire::decode_demo_work_tagged(&json::parse(r#"{"value":"Alpha"}"#).unwrap(), "tagged").is_err());
}

#[test]
fn the_served_command_reports_unit_variants_in_a_list_an_optional_and_both_new_unions() {
    let mut system = system();
    for input in [
        r#"{"statuses":[{"kind":"Open"},{"kind":"Complete","value":{"outcome":"shipped"}}],"maybe":{"kind":"Open"},"choice":{"kind":"Zulu"},"tagged":{"value":"Zulu"}}"#,
        r#"{"statuses":[],"choice":{"kind":"Alpha"},"tagged":{"value":"Alpha","content":"x"}}"#,
    ] {
        let handled = surface::handle(&mut system, Some(&caller()), "demo.work.Report", json::parse(input).unwrap()).unwrap();
        let outcome = format!(r#"{{"outcome":"reported","published":[{{"event":"demo.work.Reported","payload":{input}}}]}}"#);
        assert_eq!(handled, json::parse(&outcome).unwrap(), "{input}");
    }
    for malformed in [
        r#"{"statuses":[{"kind":"Open","value":{"outcome":"shipped"}}],"choice":{"kind":"Zulu"},"tagged":{"value":"Zulu"}}"#,
        r#"{"statuses":[],"maybe":{"kind":"Open","value":null},"choice":{"kind":"Zulu"},"tagged":{"value":"Zulu"}}"#,
        r#"{"statuses":[],"choice":{"kind":"Zulu","value":{}},"tagged":{"value":"Zulu"}}"#,
        r#"{"statuses":[],"choice":{"kind":"Zulu"},"tagged":{"value":"Zulu","content":"x"}}"#,
    ] {
        assert!(
            surface::handle(&mut system, Some(&caller()), "demo.work.Report", json::parse(malformed).unwrap()).is_err(),
            "{malformed} is refused as input"
        );
    }
}
"##;

const RUST_DEPENDENCIES: &str = "demo-types = {path = '../generated/crates/demo-types'}
demo-server = {path = '../generated/crates/demo-server'}
demo-system = {path = '../generated/crates/demo-system'}
work-service = {path = '../generated/crates/work-service'}
";

#[test]
fn the_rust_target_builds_and_runs_the_wider_unions() {
    let synthesis = synthesize_for(&ir(), Target::Rust).expect("the model synthesizes");
    let root = scratch("rust");
    write(&synthesis, &root.join("generated"));
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        format!(
            "[workspace]\n[package]\nname = 'codec-adversary'\nversion = '0.0.0'\nedition = \
             '2021'\n[dependencies]\n{RUST_DEPENDENCIES}"
        ),
    )
    .unwrap();
    std::fs::write(harness.join("src/lib.rs"), RUST_HARNESS).unwrap();
    let target = root.join("target");
    let tested = cargo(&harness, &target, &["test", "--offline"]);
    let _ = std::fs::remove_dir_all(&root);
    assert!(tested.status.success(), "the Rust adversary harness failed");
    assert!(
        String::from_utf8_lossy(&tested.stdout).contains("test result: ok. 3 passed"),
        "all three Rust cases ran"
    );
}

const GO_HARNESS: &str = r#"package server

import (
	"encoding/json"
	"reflect"
	"testing"

	"example.invalid/demo/components/workservice"
	"example.invalid/demo/system"
	"example.invalid/demo/types/behaviour"
	"example.invalid/demo/types/work"
)

func parsedValue(t *testing.T, text string) any {
	t.Helper()
	var value any
	if err := json.Unmarshal([]byte(text), &value); err != nil {
		t.Fatalf("fixture JSON: %v", err)
	}
	return value
}

func TestAdversaryUnitVariants(t *testing.T) {
	choices := []struct {
		value work.Choice
		text  string
	}{
		{work.ChoiceAlpha{}, `{"kind":"Alpha"}`},
		{work.ChoiceZulu{}, `{"kind":"Zulu"}`},
	}
	for _, each := range choices {
		encoded, err := json.Marshal(encodeDemoWorkChoice(each.value))
		if err != nil || string(encoded) != each.text {
			t.Fatalf("%T encodes as %s, %v; want %s", each.value, encoded, err, each.text)
		}
		back, err := decodeDemoWorkChoice(parsedValue(t, each.text), "choice")
		if err != nil || back != each.value {
			t.Fatalf("%s decodes as %#v, %v", each.text, back, err)
		}
	}
	tagged := []struct {
		value work.Tagged
		text  string
	}{
		{work.TaggedAlpha{Value: "x"}, `{"content":"x","value":"Alpha"}`},
		{work.TaggedZulu{}, `{"value":"Zulu"}`},
	}
	for _, each := range tagged {
		encoded, err := json.Marshal(encodeDemoWorkTagged(each.value))
		if err != nil || string(encoded) != each.text {
			t.Fatalf("%T encodes as %s, %v; want %s", each.value, encoded, err, each.text)
		}
		back, err := decodeDemoWorkTagged(parsedValue(t, each.text), "tagged")
		if err != nil || back != each.value {
			t.Fatalf("%s decodes as %#v, %v", each.text, back, err)
		}
	}
	for _, malformed := range []string{`{"value":"Zulu","content":"x"}`, `{"value":"Zulu","content":null}`, `{"value":"Alpha"}`} {
		if back, err := decodeDemoWorkTagged(parsedValue(t, malformed), "tagged"); err == nil {
			t.Fatalf("%s decodes as %#v; it must be refused", malformed, back)
		}
	}
	running := system.NewSystem(workservice.New(behaviour.New(NewMemoryPorts())))
	for _, input := range []string{
		`{"statuses":[{"kind":"Open"},{"kind":"Complete","value":{"outcome":"shipped"}}],"maybe":{"kind":"Open"},"choice":{"kind":"Zulu"},"tagged":{"value":"Zulu"}}`,
		`{"statuses":[],"choice":{"kind":"Alpha"},"tagged":{"value":"Alpha","content":"x"}}`,
	} {
		answer := serveDemoWorkReport(running, []byte(input))
		want := `{"outcome":"reported","published":[{"event":"demo.work.Reported","payload":` + input + `}]}`
		if answer.status != 202 || !reflect.DeepEqual(parsedValue(t, answer.body), parsedValue(t, want)) {
			t.Fatalf("served %d %s\nwant %s", answer.status, answer.body, want)
		}
	}
	for _, malformed := range []string{
		`{"statuses":[{"kind":"Open","value":{"outcome":"shipped"}}],"choice":{"kind":"Zulu"},"tagged":{"value":"Zulu"}}`,
		`{"statuses":[],"maybe":{"kind":"Open","value":null},"choice":{"kind":"Zulu"},"tagged":{"value":"Zulu"}}`,
		`{"statuses":[],"choice":{"kind":"Zulu","value":{}},"tagged":{"value":"Zulu"}}`,
		`{"statuses":[],"choice":{"kind":"Zulu"},"tagged":{"value":"Zulu","content":"x"}}`,
	} {
		answer := serveDemoWorkReport(running, []byte(malformed))
		if answer.status != 400 {
			t.Fatalf("%s is answered %d %s; it must be refused as input", malformed, answer.status, answer.body)
		}
	}
	t.Log("go adversary: wider unions round-trip, malformed shapes refused")
}
"#;

#[test]
fn the_go_target_vets_and_runs_the_wider_unions() {
    let synthesis = synthesize_for(&ir(), Target::Go).expect("Go represents the model");
    let directory = scratch("go");
    write(&synthesis, &directory);
    std::fs::write(directory.join("server/zz_adversary_test.go"), GO_HARNESS).unwrap();
    let vetted = go(&directory, &["vet", "./..."]);
    let tested = go(
        &directory,
        &[
            "test",
            "-count=1",
            "-run",
            "TestAdversaryUnitVariants",
            "-v",
            "./server/",
        ],
    );
    let _ = std::fs::remove_dir_all(&directory);
    assert!(vetted.status.success(), "go vet");
    assert!(tested.status.success(), "the Go adversary harness failed");
    assert!(String::from_utf8_lossy(&tested.stdout)
        .contains("go adversary: wider unions round-trip, malformed shapes refused"));
}

/// A model whose one command sends `field` of type `type_ref`, with `types` (YAML list items,
/// already indented) declared.
fn single(types: &str, type_ref: &str) -> String {
    format!(
        "format: ess/22
system: demo
version: v1
domain: demo.work
types:
{types}events:
  - name: demo.work.Reported
    fields:
      - {{name: field, type: {type_ref}}}
actors:
  - name: demo.work.Reporter
    may: [demo.work.Report]
commands:
  - name: demo.work.Report
    input:
      - {{name: field, type: {type_ref}}}
    outcomes:
      - name: reported
        emits: [demo.work.Reported]
        payload:
          demo.work.Reported:
            field: input.field
components:
  - component: work-service
    owns:
      domains: [demo.work]
    accepts:
      commands: [demo.work.Report]
    publishes:
      events: [demo.work.Reported]
    reached_by: network
"
    )
}

const CHOICE: &str = "  - name: demo.work.Choice
    kind: union
    tag: kind
    variants:
      Alpha:
      Zulu:
";

const TAGGED: &str = "  - name: demo.work.Tagged
    kind: union
    tag: value
    variants:
      Alpha: String
      Zulu:
";

const STATUS: &str = "  - name: demo.work.Completion
    kind: struct
    fields:
      - {name: outcome, type: String}
  - name: demo.work.Status
    kind: union
    tag: kind
    variants:
      Open:
      Complete: demo.work.Completion
";

/// Synthesizes `text` for Go into the build directory, vets it, and on failure prints the lines
/// the compiler names.
fn go_vets(label: &str, text: &str) {
    let spec = Specification::assemble([(
        Source::new("work.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("work.yaml", text);
    let ir = compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let synthesis = synthesize_for(&ir, Target::Go).expect("Go represents the model");
    let directory = scratch(&format!("go-vet-{label}"));
    write(&synthesis, &directory);
    let vetted = go(&directory, &["vet", "./..."]);
    let stderr = String::from_utf8_lossy(&vetted.stderr).to_string();
    for line in stderr
        .lines()
        .filter(|line| line.starts_with("server/wire.go:"))
    {
        let number: usize = line
            .split(':')
            .nth(1)
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);
        let wire = std::fs::read_to_string(directory.join("server/wire.go")).unwrap_or_default();
        let excerpt: Vec<_> = wire
            .lines()
            .enumerate()
            .skip(number.saturating_sub(12))
            .take(16)
            .map(|(n, l)| format!("{:5} {l}", n + 1))
            .collect();
        eprintln!("{label}: {line}\n{}", excerpt.join("\n"));
    }
    let _ = std::fs::remove_dir_all(&directory);
    assert!(vetted.status.success(), "{label}: go vet\n{stderr}");
}

#[test]
fn the_go_target_vets_a_union_of_only_unit_variants() {
    go_vets("choice", &single(CHOICE, "demo.work.Choice"));
}

#[test]
fn the_go_target_vets_a_union_tagged_value_with_a_unit_variant() {
    go_vets("tagged", &single(TAGGED, "demo.work.Tagged"));
}

#[test]
fn the_go_target_vets_a_list_of_a_union_with_a_unit_variant() {
    go_vets("list", &single(STATUS, "List<demo.work.Status>"));
}

#[test]
fn the_go_target_vets_an_optional_union_with_a_unit_variant() {
    go_vets("optional", &single(STATUS, "Optional<demo.work.Status>"));
}
