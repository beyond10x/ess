//! Format labels and optional recursive values through every generated codec (beyond10x/ess#400).
//!
//! A format enum whose wire labels are not identifiers (`demo.explanation/2`) is authored with the
//! existing explicit form, `{name: ExplanationV2, wire: demo.explanation/2}`: the name is the
//! emitted identifier and the label is the serialized spelling. A struct that holds an optional
//! child of its own type is a self-recursive value. Each test below synthesizes fresh code,
//! compiles it with the target's own toolchain and runs it, so what is asserted is what the
//! generated codecs actually do:
//!
//! - the Rust server's named, command, event, outcome and system-event codecs, and its served
//!   entry, in both the workspace and the single-crate layouts;
//! - the shared Web bridge's codecs and its served command, executed as WebAssembly;
//! - the Go server's codecs and its served command.
//!
//! The controls hold the rest of the decision (`docs/design/optional-recursive-rust.md`): a label
//! used as an identifier is refused, source-addressed, by every code target rather than emitted
//! as source that does not compile, and a mutual optional cycle stays a Rust `recursive-layout`
//! refusal while Go, whose optional is a pointer, compiles and round-trips it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, synthesize_laid_out, OutputLayout, Synthesis, Target};

/// Labelled formats and a self-recursive value at every position a served component's codecs
/// reach: a named type, a command input, an event payload and the outcome that publishes it.
const CODECS: &str = r"format: ess/19
system: demo
version: v1
domain: demo.kernel
types:
  - name: demo.kernel.ExplanationFormat
    kind: enum
    variants:
      - {name: ExplanationV2, wire: demo.explanation/2}
      - {name: TransactionDocumentV1, wire: demo.transaction-document/1}
      - {name: TransactionDocumentV2, wire: demo.transaction-document/2}
      - plain
  - name: demo.kernel.ValueSpec
    kind: struct
    fields:
      - {name: format, type: demo.kernel.ExplanationFormat}
      - {name: kind, type: String}
      - {name: element, type: Optional<demo.kernel.ValueSpec>}
      - {name: alternatives, type: 'List<demo.kernel.ValueSpec>'}
events:
  - name: demo.kernel.Explained
    fields:
      - {name: format, type: demo.kernel.ExplanationFormat}
      - {name: spec, type: demo.kernel.ValueSpec}
      - {name: nested, type: Optional<demo.kernel.ValueSpec>}
commands:
  - name: demo.kernel.Explain
    input:
      - {name: format, type: demo.kernel.ExplanationFormat}
      - {name: spec, type: demo.kernel.ValueSpec}
      - {name: nested, type: Optional<demo.kernel.ValueSpec>}
    outcomes:
      - name: explained
        emits: [demo.kernel.Explained]
        payload:
          demo.kernel.Explained:
            format: input.format
            spec: input.spec
            nested: input.nested
components:
  - component: kernel
    owns:
      domains: [demo.kernel]
    accepts:
      commands: [demo.kernel.Explain]
    publishes:
      events: [demo.kernel.Explained]
    reached_by: network
";

/// Two structs that hold each other optionally: a cycle through two declarations.
const MUTUAL: &str = r"format: ess/19
system: demo
version: v1
domain: demo.kernel
types:
  - name: demo.kernel.Left
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: right, type: Optional<demo.kernel.Right>}
  - name: demo.kernel.Right
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: left, type: Optional<demo.kernel.Left>}
";

/// One recursive value and one leaf, in the member order the Rust and Web encoders write
/// (declaration order); the Go harness compares against `encoding/json`'s sorted rendering.
const DEEP: &str = r#"{"format":"demo.explanation/2","kind":"list","element":{"format":"demo.transaction-document/1","kind":"map","element":{"format":"plain","kind":"text","alternatives":[]},"alternatives":[{"format":"demo.transaction-document/2","kind":"pair","element":{"format":"plain","kind":"left","alternatives":[]},"alternatives":[]}]},"alternatives":[]}"#;
const NESTED: &str = r#"{"format":"demo.transaction-document/2","kind":"leaf","alternatives":[]}"#;

/// The Rust values `DEEP` and `NESTED` spell, the command input carrying both, and its outcome.
const RUST_VALUES: &str = r##"
fn leaf(kind: &str) -> ValueSpec {
    ValueSpec { format: ExplanationFormat::Plain, kind: kind.into(), element: None, alternatives: Vec::new() }
}

fn deep() -> ValueSpec {
    ValueSpec {
        format: ExplanationFormat::ExplanationV2,
        kind: "list".into(),
        element: Some(Box::new(ValueSpec {
            format: ExplanationFormat::TransactionDocumentV1,
            kind: "map".into(),
            element: Some(Box::new(leaf("text"))),
            alternatives: vec![Box::new(ValueSpec {
                format: ExplanationFormat::TransactionDocumentV2,
                kind: "pair".into(),
                element: Some(Box::new(leaf("left"))),
                alternatives: Vec::new(),
            })],
        })),
        alternatives: Vec::new(),
    }
}

fn nested() -> ValueSpec {
    ValueSpec { format: ExplanationFormat::TransactionDocumentV2, kind: "leaf".into(), element: None, alternatives: Vec::new() }
}

const DEEP: &str = r#"@DEEP@"#;
const NESTED: &str = r#"@NESTED@"#;

fn input() -> String {
    format!(r#"{{"format":"demo.transaction-document/1","spec":{DEEP},"nested":{NESTED}}}"#)
}

fn outcome() -> String {
    format!(r#"{{"outcome":"explained","published":[{{"event":"demo.kernel.Explained","payload":{}}}]}}"#, input())
}

const LABELS: [(ExplanationFormat, &str); 4] = [
    (ExplanationFormat::ExplanationV2, "demo.explanation/2"),
    (ExplanationFormat::TransactionDocumentV1, "demo.transaction-document/1"),
    (ExplanationFormat::TransactionDocumentV2, "demo.transaction-document/2"),
    (ExplanationFormat::Plain, "plain"),
];
"##;

/// The native Rust harness: every server codec, its served entry, and the system-event envelope.
const RUST_HARNESS: &str = r##"
#![cfg(test)]
use demo_server::{entry, http, json, kernel as surface, wire};
use demo_types::kernel::{Explain, ExplainOutcome, Explained, ExplanationFormat, ValueSpec};

@VALUES@

#[test]
fn labels_are_exact_in_both_directions() {
    for (variant, label) in LABELS {
        let mut encoded = String::new();
        wire::encode_demo_kernel_explanation_format(&variant, &mut encoded);
        assert_eq!(encoded, format!("\"{label}\""));
        let decoded = wire::decode_demo_kernel_explanation_format(&json::parse(&encoded).unwrap(), "").unwrap();
        assert_eq!(decoded, variant);
    }
    for spelling in ["\"ExplanationV2\"", "\"demo.explanation/3\"", "\"Plain\""] {
        let refused = wire::decode_demo_kernel_explanation_format(&json::parse(spelling).unwrap(), "format")
            .expect_err("only the declared wire label is the value");
        assert_eq!(refused.at, "format");
        assert!(refused.expected.contains("`demo.explanation/2`"), "{refused:?}");
    }
}

#[test]
fn recursive_values_round_trip_through_the_named_codec() {
    let mut encoded = String::new();
    wire::encode_demo_kernel_value_spec(&deep(), &mut encoded);
    assert_eq!(encoded, DEEP);
    assert_eq!(wire::decode_demo_kernel_value_spec(&json::parse(DEEP).unwrap(), "").unwrap(), deep());
    let null = json::parse(r#"{"format":"plain","kind":"text","element":null,"alternatives":[]}"#).unwrap();
    assert_eq!(wire::decode_demo_kernel_value_spec(&null, "").unwrap(), leaf("text"));
    let wrong = DEEP.replacen("\"demo.transaction-document/1\"", "\"TransactionDocumentV1\"", 1);
    let refused = wire::decode_demo_kernel_value_spec(&json::parse(&wrong).unwrap(), "").unwrap_err();
    assert_eq!(refused.at, "element.format");
}

#[test]
fn command_event_and_outcome_codecs_carry_labels_and_recursion() {
    let explain = Explain { format: ExplanationFormat::TransactionDocumentV1, spec: Box::new(deep()), nested: Some(Box::new(nested())) };
    let decoded = wire::decode_command_demo_kernel_explain(&json::parse(&input()).unwrap(), "").unwrap();
    assert_eq!(decoded, explain);
    let mut encoded = String::new();
    wire::encode_command_demo_kernel_explain(&explain, &mut encoded);
    assert_eq!(encoded, input());
    let explained = Explained { format: explain.format.clone(), spec: explain.spec.clone(), nested: explain.nested.clone() };
    encoded.clear();
    wire::encode_event_demo_kernel_explained(&explained, &mut encoded);
    assert_eq!(encoded, input());
    encoded.clear();
    wire::encode_outcome_demo_kernel_explain(&ExplainOutcome::Explained { explained: explained.clone() }, &mut encoded);
    assert_eq!(encoded, outcome());
    let logged = wire::encode_system_event(&demo_system::SystemEvent::Explained(explained));
    assert_eq!(logged, format!(r#"{{"event":"demo.kernel.Explained","payload":{}}}"#, input()));
}

#[test]
fn the_served_component_answers_the_exact_outcome() {
    let mut system = demo_system::System::new(kernel::Kernel::new(demo_types::behaviour::Generated::new(())));
    let handled = surface::handle(&mut system, "demo.kernel.Explain", json::parse(&input()).unwrap()).unwrap();
    assert_eq!(handled, json::parse(&outcome()).unwrap());
    let request = http::Request {
        method: "POST".to_owned(),
        path: "/kernel/commands/Explain".to_owned(),
        query: String::new(),
        headers: Vec::new(),
        body: input().into_bytes(),
    };
    let answered = surface::dispatch(&mut system, &request);
    assert_eq!(answered.status, 202, "{}", answered.body);
    assert_eq!(answered.body, outcome());
    let wrong = input().replacen("\"plain\",\"kind\":\"left\"", "\"Plain\",\"kind\":\"left\"", 1);
    match surface::handle(&mut system, "demo.kernel.Explain", json::parse(&wrong).unwrap()) {
        Err(entry::Refused::Input(detail)) => assert!(detail.contains("body.spec.element.alternatives.0.element.format"), "{detail}"),
        other => panic!("a nested label that is not declared is refused as input: {other:?}"),
    }
}
"##;

/// The WebAssembly harness over the shared Web bridge: its codecs and its served command.
const WEB_HARNESS: &str = r##"
use demo_web::{json, wire};
use demo_types::kernel::{Explain, ExplainOutcome, Explained, ExplanationFormat, ValueSpec};

@VALUES@

#[no_mangle]
pub extern "C" fn codec_proof() -> u32 {
    for (variant, label) in LABELS {
        let mut encoded = String::new();
        wire::encode_demo_kernel_explanation_format(&variant, &mut encoded);
        assert_eq!(encoded, format!("\"{label}\""));
        assert_eq!(wire::decode_demo_kernel_explanation_format(&json::parse(&encoded).unwrap(), "").unwrap(), variant);
    }
    assert!(wire::decode_demo_kernel_explanation_format(&json::parse("\"ExplanationV2\"").unwrap(), "").is_err());
    let mut encoded = String::new();
    wire::encode_demo_kernel_value_spec(&deep(), &mut encoded);
    assert_eq!(encoded, DEEP);
    assert_eq!(wire::decode_demo_kernel_value_spec(&json::parse(DEEP).unwrap(), "").unwrap(), deep());
    let explain = wire::decode_command_demo_kernel_explain(&json::parse(&input()).unwrap(), "").unwrap();
    assert_eq!(explain, Explain { format: ExplanationFormat::TransactionDocumentV1, spec: Box::new(deep()), nested: Some(Box::new(nested())) });
    let explained = Explained { format: explain.format, spec: explain.spec, nested: explain.nested };
    encoded.clear();
    wire::encode_outcome_demo_kernel_explain(&ExplainOutcome::Explained { explained }, &mut encoded);
    assert_eq!(encoded, outcome());
    demo_web::install(Box::new(demo_system::System::new(kernel::Kernel::new(demo_types::behaviour::Generated::new(())))));
    let answer = demo_web::serve(&format!(r#"{{"request":"command","command":"demo.kernel.Explain","input":{}}}"#, input()));
    let answer = json::parse(&answer).unwrap();
    assert_eq!(answer.member("ok"), Some(&json::Value::Bool(true)));
    assert_eq!(answer.member("outcome"), Some(&json::parse(&outcome()).unwrap()));
    73
}
"##;

/// The Go harness, inside the generated server package so it reaches the unexported codecs.
const GO_HARNESS: &str = r#"package server

import (
	"encoding/json"
	"testing"

	"example.invalid/demo/components/kernelcomponent"
	"example.invalid/demo/system"
	"example.invalid/demo/types/behaviour"
	"example.invalid/demo/types/kernel"
)

func parsed(t *testing.T, text string) any {
	t.Helper()
	var value any
	if err := json.Unmarshal([]byte(text), &value); err != nil {
		t.Fatalf("fixture JSON: %v", err)
	}
	return value
}

func sorted(t *testing.T, text string) string {
	t.Helper()
	encoded, err := json.Marshal(parsed(t, text))
	if err != nil {
		t.Fatalf("fixture JSON: %v", err)
	}
	return string(encoded)
}

func TestRecursiveCodecs(t *testing.T) {
	labels := []struct {
		variant kernel.ExplanationFormat
		label   string
	}{
		{kernel.ExplanationFormatExplanationV2{}, "demo.explanation/2"},
		{kernel.ExplanationFormatTransactionDocumentV1{}, "demo.transaction-document/1"},
		{kernel.ExplanationFormatTransactionDocumentV2{}, "demo.transaction-document/2"},
		{kernel.ExplanationFormatPlain{}, "plain"},
	}
	for _, each := range labels {
		if got := encodeDemoKernelExplanationFormat(each.variant); got != each.label {
			t.Fatalf("%T encodes as %v, want %q", each.variant, got, each.label)
		}
		back, err := decodeDemoKernelExplanationFormat(each.label, "format")
		if err != nil || back != each.variant {
			t.Fatalf("%q decodes as %T, %v", each.label, back, err)
		}
	}
	if _, err := decodeDemoKernelExplanationFormat("ExplanationV2", "format"); err == nil {
		t.Fatal("the identifier is not the wire label")
	}
	spec, err := decodeDemoKernelValueSpec(parsed(t, `@DEEP@`), "body")
	if err != nil {
		t.Fatal(err)
	}
	if spec.Element == nil || spec.Element.Element == nil || spec.Element.Element.Kind != "text" ||
		spec.Element.Alternatives[0].Element.Kind != "left" || spec.Element.Element.Element != nil {
		t.Fatalf("the recursive value decoded as %+v", spec)
	}
	encoded, err := json.Marshal(encodeDemoKernelValueSpec(spec))
	if err != nil || string(encoded) != sorted(t, `@DEEP@`) {
		t.Fatalf("the recursive value encodes as %s, %v", encoded, err)
	}
	input := `{"format":"demo.transaction-document/1","spec":@DEEP@,"nested":@NESTED@}`
	running := system.NewSystem(kernelcomponent.New(behaviour.New(NewMemoryPorts())))
	answer := serveDemoKernelExplain(running, []byte(input))
	want := sorted(t, `{"outcome":"explained","published":[{"event":"demo.kernel.Explained","payload":`+input+`}]}`)
	if answer.status != 202 || answer.body != want {
		t.Fatalf("served %d %s\nwant %s", answer.status, answer.body, want)
	}
	t.Log("go codecs: labels exact, recursive value and served outcome round-trip")
}
"#;

/// The Go harness for the mutual cycle the Rust target refuses. Without a component the Go
/// module has no server package, so the harness is its own package beside the types.
const GO_MUTUAL_HARNESS: &str = r#"package mutual

import (
	"testing"

	"example.invalid/demo/types/kernel"
)

func TestMutualValues(t *testing.T) {
	value := kernel.Left{Label: "a", Right: &kernel.Right{Label: "b", Left: &kernel.Left{Label: "c"}}}
	if value.Right.Left.Label != "c" || value.Right.Left.Right != nil {
		t.Fatalf("the mutual value is %+v", value)
	}
	t.Log("go mutual values compile")
}
"#;

fn ir(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(source).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", source);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("optional-recursive-codecs-{label}"))
}

/// One build directory for every harness this file compiles, so the generated crates' shared
/// dependencies are built once.
fn harness_target() -> PathBuf {
    scratch("target")
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

fn cargo(directory: &Path, arguments: &[&str], deny_warnings: bool) -> Output {
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .args(arguments)
        .current_dir(directory)
        .env("CARGO_TARGET_DIR", harness_target())
        .env_remove("CARGO_ENCODED_RUSTFLAGS");
    if deny_warnings {
        command.env("RUSTFLAGS", "-D warnings");
    } else {
        command.env_remove("RUSTFLAGS");
    }
    run(command, "cargo")
}

/// A Go tool inside a generated module, offline: no module download, no toolchain switch.
fn go(directory: &Path, arguments: &[&str]) -> Output {
    let mut command = Command::new("go");
    command
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local");
    if std::env::var_os("GOCACHE").is_none() {
        command.env("GOCACHE", scratch("gocache"));
    }
    run(command, "go")
}

fn values() -> String {
    RUST_VALUES
        .replace("@DEEP@", DEEP)
        .replace("@NESTED@", NESTED)
}

fn harness_crate(directory: &Path, dependencies: &str, cdylib: bool, source: &str) {
    std::fs::create_dir_all(directory.join("src")).unwrap();
    let lib = if cdylib {
        "[lib]\ncrate-type = ['cdylib']\n"
    } else {
        ""
    };
    std::fs::write(
        directory.join("Cargo.toml"),
        format!("[workspace]\n[package]\nname = 'codec-proof'\nversion = '0.0.0'\nedition = '2021'\n{lib}[dependencies]\n{dependencies}"),
    )
    .unwrap();
    std::fs::write(directory.join("src/lib.rs"), source).unwrap();
    let lock = cargo(directory, &["generate-lockfile", "--offline"], false);
    assert!(lock.status.success(), "the harness resolves offline");
}

/// A refusal's envelope format and its causes as `(code, sources, detail)`.
fn refusal(ir: &EssIr, target: Target) -> (String, Vec<(String, Vec<String>, String)>) {
    let failure = synthesize_for(ir, target)
        .err()
        .unwrap_or_else(|| panic!("{} must refuse rather than emit", target.name()));
    let json: serde_json::Value = serde_json::from_str(&failure.to_canonical_json()).unwrap();
    let causes = json["causes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|cause| {
            (
                cause["code"].as_str().unwrap().to_owned(),
                cause["sources"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|source| source.as_str().unwrap().to_owned())
                    .collect(),
                cause["detail"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    (json["format"].as_str().unwrap().to_owned(), causes)
}

#[test]
fn rust_server_codecs_carry_labels_and_recursive_values_in_both_layouts() {
    let ir = ir(CODECS);
    for (label, layout) in [
        ("workspace", OutputLayout::Workspace),
        ("crate", OutputLayout::Crate),
    ] {
        let synthesis = synthesize_laid_out(&ir, Target::Rust, layout)
            .unwrap_or_else(|failure| panic!("labelled recursive model: {failure}"));
        assert_eq!(
            synthesis.artifacts,
            synthesize_laid_out(&ir, Target::Rust, layout)
                .unwrap()
                .artifacts,
            "synthesis is deterministic"
        );
        let root = scratch(&format!("rust-{label}"));
        write(&synthesis, &root.join("generated"));
        let harness = RUST_HARNESS.replace("@VALUES@", &values());
        let (dependencies, source) = if layout == OutputLayout::Workspace {
            ("demo-types = {path = '../generated/crates/demo-types'}\ndemo-server = {path = '../generated/crates/demo-server'}\ndemo-system = {path = '../generated/crates/demo-system'}\nkernel = {path = '../generated/crates/kernel'}\n", harness)
        } else {
            (
                "demo = {path = '../generated', features = ['server']}\n",
                harness
                    .replace("demo_server::", "demo::server::")
                    .replace("demo_types::", "demo::")
                    .replace("demo_system::", "demo::system::")
                    .replace("kernel::Kernel", "demo::ports::kernel::Kernel"),
            )
        };
        let directory = root.join("harness");
        harness_crate(&directory, dependencies, false, &source);
        let tested = cargo(&directory, &["test", "--offline", "--locked"], true);
        assert!(
            tested.status.success(),
            "the {label} codec harness failed; see {}",
            directory.display()
        );
        assert!(
            String::from_utf8_lossy(&tested.stdout).contains("test result: ok. 4 passed"),
            "all four {label} codec cases ran"
        );
    }
}

#[test]
fn web_bridge_codecs_carry_labels_and_recursive_values_under_wasm() {
    let ir = ir(CODECS);
    let root = scratch("web");
    write(
        &synthesize_for(&ir, Target::Rust).expect("the Web target's Rust prerequisite"),
        &root.join("generated/rust/demo"),
    );
    write(
        &synthesize_for(&ir, Target::Web).expect("the shared Web codec"),
        &root.join("generated/web/demo"),
    );
    let directory = root.join("harness");
    harness_crate(
        &directory,
        "demo-web = {path = '../generated/web/demo/crates/demo-web'}\ndemo-types = {path = '../generated/rust/demo/crates/demo-types'}\ndemo-system = {path = '../generated/rust/demo/crates/demo-system'}\nkernel = {path = '../generated/rust/demo/crates/kernel'}\n",
        true,
        &WEB_HARNESS.replace("@VALUES@", &values()),
    );
    let built = cargo(
        &directory,
        &[
            "build",
            "--offline",
            "--locked",
            "--target",
            "wasm32-unknown-unknown",
        ],
        false,
    );
    assert!(built.status.success(), "the WASM codec harness compiles");
    let module = harness_target().join("wasm32-unknown-unknown/debug/codec_proof.wasm");
    let mut node = Command::new("node");
    node.arg("-e")
        .arg("const fs=require('node:fs');WebAssembly.instantiate(fs.readFileSync(process.argv[1]),{}).then(({instance})=>{const answer=instance.exports.codec_proof();if(answer!==73)throw Error(String(answer));console.log('web codecs: labels exact, recursive value and served outcome round-trip');}).catch(e=>{console.error(e);process.exit(1);});")
        .arg(&module);
    let ran = run(node, "node");
    assert!(
        ran.status.success(),
        "the generated Web codecs must execute under WASM"
    );
    assert!(String::from_utf8_lossy(&ran.stdout)
        .contains("web codecs: labels exact, recursive value and served outcome round-trip"));
}

#[test]
fn go_server_codecs_carry_labels_and_recursive_values() {
    let synthesis = synthesize_for(&ir(CODECS), Target::Go)
        .unwrap_or_else(|failure| panic!("Go represents the labelled recursive model: {failure}"));
    let directory = scratch("go");
    write(&synthesis, &directory);
    std::fs::write(
        directory.join("server/zz_recursive_codecs_test.go"),
        GO_HARNESS
            .replace("@DEEP@", DEEP)
            .replace("@NESTED@", NESTED),
    )
    .unwrap();
    assert!(go(&directory, &["vet", "./..."]).status.success());
    let tested = go(
        &directory,
        &[
            "test",
            "-count=1",
            "-run",
            "TestRecursiveCodecs",
            "-v",
            "./server/",
        ],
    );
    assert!(tested.status.success(), "the Go codec harness failed");
    assert!(String::from_utf8_lossy(&tested.stdout)
        .contains("go codecs: labels exact, recursive value and served outcome round-trip"));
}

#[test]
fn mutual_optional_recursion_stays_a_rust_refusal_and_compiles_in_go() {
    let ir = ir(MUTUAL);
    for target in [Target::Rust, Target::Web] {
        let (format, causes) = refusal(&ir, target);
        assert_eq!(format, "ess-target-failure/1");
        assert_eq!(causes.len(), 1, "{causes:?}");
        let (code, sources, detail) = &causes[0];
        assert_eq!(code, "recursive-layout");
        assert_eq!(sources, &["demo.kernel.Left", "demo.kernel.Right"]);
        assert!(
            detail.contains("demo.kernel.Left.right -> demo.kernel.Right.left -> demo.kernel.Left"),
            "{detail}"
        );
    }
    let synthesis = synthesize_for(&ir, Target::Go).expect("a Go optional is a pointer");
    let directory = scratch("go-mutual");
    write(&synthesis, &directory);
    std::fs::create_dir_all(directory.join("mutual")).unwrap();
    std::fs::write(
        directory.join("mutual/zz_mutual_test.go"),
        GO_MUTUAL_HARNESS,
    )
    .unwrap();
    assert!(go(&directory, &["vet", "./..."]).status.success());
    let tested = go(&directory, &["test", "-count=1", "-v", "./mutual/"]);
    assert!(tested.status.success(), "the Go mutual harness failed");
    assert!(String::from_utf8_lossy(&tested.stdout).contains("go mutual values compile"));
}

/// Spellings that are wire text and not identifiers, reached as identifiers: a bare enum label, a
/// union label, and an explicit variant name.
const NOT_IDENTIFIERS: &str = r"format: ess/19
system: demo
version: v1
domain: demo.kernel
types:
  - name: demo.kernel.ExplanationFormat
    kind: enum
    variants:
      - demo.explanation/2
      - {name: 'Bad.Name', wire: demo.bad/1}
      - {name: Fine, wire: demo.fine/1}
  - name: demo.kernel.Shape
    kind: union
    tag: kind
    variants:
      demo.shape/1: String
";

#[test]
fn labels_used_as_identifiers_are_refused_by_every_code_target() {
    let ir = ir(NOT_IDENTIFIERS);
    let expected = [
        "demo.kernel.ExplanationFormat.Bad.Name",
        "demo.kernel.ExplanationFormat.demo.explanation/2",
        "demo.kernel.Shape.demo.shape/1",
    ];
    for (target, format) in [
        (Target::Rust, "ess-target-failure/1"),
        (Target::Web, "ess-target-failure/1"),
        (Target::Go, "ess-target-failure/2"),
    ] {
        let (actual, causes) = refusal(&ir, target);
        assert_eq!(actual, format, "{}", target.name());
        assert!(
            causes
                .iter()
                .all(|(code, _, _)| code == "invalid-identifier"),
            "{}: {causes:?}",
            target.name()
        );
        let mut sources: Vec<&str> = causes
            .iter()
            .flat_map(|(_, sources, _)| sources.iter().map(String::as_str))
            .collect();
        sources.sort_unstable();
        assert_eq!(sources, expected, "{}: {causes:?}", target.name());
        assert_eq!(
            synthesize_for(&ir, target)
                .err()
                .unwrap()
                .to_canonical_json(),
            synthesize_for(&ir, target)
                .err()
                .unwrap()
                .to_canonical_json(),
            "the refusal is deterministic"
        );
    }
    let (_, go) = refusal(&ir, Target::Go);
    let bare = go
        .iter()
        .find(|(_, sources, _)| sources[0].ends_with("demo.explanation/2"))
        .unwrap();
    assert!(
        bare.2.contains("invalid Go identifier") && bare.2.contains("wire"),
        "the Go refusal names the identifier and the explicit-name remedy: {}",
        bare.2
    );
}
