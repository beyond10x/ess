//! Adversary, pass 1, for `story:generated-decoders-check-struct-invariants`.
//!
//! The unit's own cases read one flat struct at one path. These drive the generated Rust and Go
//! decoders over the shapes a served command really carries a struct in: nested inside another
//! constrained struct, as an `Optional`, as a list element, beside an `Optional` member the
//! invariant reads, with a quantifier (a Go prelude), with two broken invariants (the first
//! declared wins) and with a member whose own newtype constraint refuses it first. Both decoders
//! must name the same path and the same words, apart from the list position, which the two targets
//! already spell differently for every list element (`.1` and `[1]`).
//!
//! Two more cases: the acceptance's refusal (a struct invariant neither target can check fails the
//! target, naming the struct and the invariant), and a model whose Go layout holds a package named
//! `out`, the name the generated Go struct decoder hard-codes for the value it builds.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Synthesis, Target};

const MODEL: &str = "format: ess/22
system: probe
version: v1
domain: probe.range
types:
  - name: probe.range.Level
    kind: newtype
    of: Integer
    invariants: [value >= 0]
  - name: probe.range.Window
    kind: struct
    fields:
      - {name: low, type: Integer}
      - {name: high, type: Integer}
      - {name: level, type: probe.range.Level}
    invariants:
      - low <= high
      - level <= 10
  - name: probe.range.Pair
    kind: struct
    fields:
      - {name: inner, type: probe.range.Window}
      - {name: cap, type: Optional<Integer>}
      - {name: tags, type: List<String>}
    invariants:
      - inner.high <= cap
      - forall: {in: tags, as: tag, that: tag.count > 0}
events:
  - name: probe.range.WindowSet
    fields: [{name: note, type: String}]
commands:
  - name: probe.range.SetWindow
    input:
      - {name: pair, type: probe.range.Pair}
      - {name: windows, type: List<probe.range.Window>}
      - {name: maybe, type: Optional<probe.range.Window>}
      - {name: note, type: String}
    outcomes:
      - name: set
        emits: [probe.range.WindowSet]
        payload:
          probe.range.WindowSet: {note: input.note}
components:
  - component: ranger
    owns: {domains: [probe.range]}
    accepts: {commands: [probe.range.SetWindow]}
    publishes: {events: [probe.range.WindowSet]}
    reached_by: network
";

const OK: &str = r#"{"low":0,"high":1,"level":1}"#;

fn input(inner: &str, cap: &str, tags: &str, second: &str, maybe: Option<&str>) -> String {
    let maybe = maybe.map_or(String::new(), |window| format!(r#","maybe":{window}"#));
    format!(
        r#"{{"pair":{{"inner":{inner}{cap},"tags":{tags}}},"windows":[{OK},{second}]{maybe},"note":"x"}}"#
    )
}

/// Each input, and the verdict both decoders must give; `{at}` is the list position, spelled per
/// target; a trailing `*` matches any rest.
fn cases() -> Vec<(String, &'static str)> {
    let cap = r#","cap":5"#;
    let tags = r#"["a"]"#;
    vec![
        (input(OK, cap, tags, OK, None), "ok"),
        (
            input(OK, cap, tags, OK, Some(r#"{"low":2,"high":1,"level":1}"#)),
            "refused $.maybe | a value satisfying `low <= high`",
        ),
        (
            input(OK, cap, tags, r#"{"low":2,"high":1,"level":1}"#, None),
            "refused $.windows{at} | a value satisfying `low <= high`",
        ),
        (
            input(r#"{"low":2,"high":1,"level":1}"#, cap, tags, OK, None),
            "refused $.pair.inner | a value satisfying `low <= high`",
        ),
        (
            input(OK, r#","cap":0"#, tags, OK, None),
            "refused $.pair | a value satisfying `inner.high <= cap`",
        ),
        (input(OK, "", tags, OK, None), "ok"),
        (
            input(OK, cap, r#"["a",""]"#, OK, None),
            "refused $.pair | a value satisfying `*",
        ),
        (
            input(r#"{"low":2,"high":1,"level":-1}"#, cap, tags, OK, None),
            "refused $.pair.inner.level | a value satisfying `value >= 0`",
        ),
        (
            input(r#"{"low":0,"high":1,"level":11}"#, cap, tags, OK, None),
            "refused $.pair.inner | a value satisfying `level <= 10`",
        ),
        (
            input(r#"{"low":2,"high":1,"level":11}"#, cap, tags, OK, None),
            "refused $.pair.inner | a value satisfying `low <= high`",
        ),
    ]
}

fn check(target: &str, position: &str, lines: &[&str]) {
    let expected = cases();
    assert_eq!(lines.len(), expected.len(), "{target}: {lines:#?}");
    for ((input, verdict), line) in expected.iter().zip(lines) {
        let verdict = verdict.replace("{at}", position);
        let holds = match verdict.strip_suffix('*') {
            Some(prefix) => line.starts_with(prefix),
            None => *line == verdict,
        };
        assert!(
            holds,
            "{target}: {input}\n  expected {verdict}\n  got      {line}"
        );
    }
}

const RUST_HARNESS: &str = r#"use probe_server::{json, wire};

fn main() {
    for input in std::env::args().skip(1) {
        let document = json::parse(&input).expect("the input parses");
        match wire::decode_command_probe_range_set_window(&document, "$") {
            Ok(_) => println!("ok"),
            Err(refusal) => println!("refused {} | {}", refusal.at, refusal.expected),
        }
    }
}
"#;

const GO_HARNESS: &str = r#"package server

import (
	"errors"
	"fmt"
	"testing"
)

func TestAdversaryStructInvariants(t *testing.T) {
	for _, input := range []string{
{cases}	} {
		document, refused := readJSON([]byte(input))
		if refused != nil {
			t.Fatalf("%s does not parse: %s", input, refused.body)
		}
		_, err := decodeCommandProbeRangeSetWindow(document, "$")
		var refusal DecodeError
		switch {
		case err == nil:
			fmt.Println("case\tok")
		case errors.As(err, &refusal):
			fmt.Printf("case\trefused %s | %s\n", refusal.At, refusal.Expected)
		default:
			t.Fatalf("%s: %v", input, err)
		}
	}
}
"#;

fn has_go() -> bool {
    Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// The Rust decoder: nested, optional and listed structs, an optional member, a quantifier, two
/// broken invariants, and a member its own newtype refuses first.
#[test]
fn adv_rust_decoder_holds_struct_invariants_wherever_the_struct_sits() {
    let synthesis = synthesize_for(&ir(MODEL), Target::Rust).expect("the model synthesizes");
    let root = scratch("rust");
    let generated = root.join("generated");
    write(&synthesis, &generated);
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        "[package]\nname = \"adv-struct-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
         [dependencies]\nprobe-server = { path = \"../generated/crates/probe-server\" }\n\n\
         [workspace]\n",
    )
    .unwrap();
    std::fs::write(harness.join("src/main.rs"), RUST_HARNESS).unwrap();
    let mut arguments = vec![
        "run".to_owned(),
        "--offline".to_owned(),
        "--quiet".to_owned(),
        "--target-dir".to_owned(),
        root.join("target").to_str().unwrap().to_owned(),
        "--".to_owned(),
    ];
    arguments.extend(cases().into_iter().map(|(input, _)| input));
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let run = cargo(&harness, &arguments);
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&root);
    assert!(run.status.success(), "the harness builds and runs");
    let lines: Vec<&str> = stdout.lines().collect();
    check("rust", ".1", &lines);
}

/// The Go decoder, the same inputs and the same verdicts; its generated code gofmt- and vet-clean
/// with a quantifier's prelude inside the decoder.
#[test]
fn adv_go_decoder_holds_struct_invariants_wherever_the_struct_sits() {
    if !has_go() {
        eprintln!("no Go toolchain on this machine; the Go struct decoder is unchecked here");
        return;
    }
    let synthesis = synthesize_for(&ir(MODEL), Target::Go).expect("the model synthesizes");
    let root = scratch("go");
    write(&synthesis, &root);
    let literals = cases().iter().fold(String::new(), |mut out, (case, _)| {
        let _ = writeln!(out, "\t\t`{case}`,");
        out
    });
    std::fs::write(
        root.join("server/zz_adv_struct_invariants_test.go"),
        GO_HARNESS.replace("{cases}", &literals),
    )
    .unwrap();
    let unformatted = go_tool(&root, "gofmt", &["-l", "."]);
    let vet = go_tool(&root, "go", &["vet", "./..."]);
    let run = go_tool(
        &root,
        "go",
        &[
            "test",
            "-count=1",
            "-run",
            "TestAdversaryStructInvariants",
            "-v",
            "./server/",
        ],
    );
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        unformatted.status.success() && unformatted.stdout.is_empty(),
        "the generated Go is gofmt-clean: {}",
        String::from_utf8_lossy(&unformatted.stdout)
    );
    assert!(vet.status.success(), "the generated Go is vet clean");
    assert!(
        run.status.success(),
        "the harness builds and runs: {stdout}"
    );
    let lines: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("case\t"))
        .collect();
    check("go", "[1]", &lines);
}

/// Acceptance: a struct invariant the target cannot check fails synthesis for that target, naming
/// the struct and the invariant. `distinct` is refused by both lanes for an entity invariant.
#[test]
fn adv_a_struct_invariant_no_target_checks_refuses_both_targets_by_name() {
    let model = "format: ess/22
system: probe
version: v1
domain: probe.range
types:
  - name: probe.range.Bag
    kind: struct
    fields:
      - {name: tags, type: List<String>}
    invariants:
      - {distinct: {in: tags, as: tag}}
events:
  - name: probe.range.BagSet
    fields: [{name: note, type: String}]
commands:
  - name: probe.range.SetBag
    input:
      - {name: bag, type: probe.range.Bag}
      - {name: note, type: String}
    outcomes:
      - name: set
        emits: [probe.range.BagSet]
        payload:
          probe.range.BagSet: {note: input.note}
components:
  - component: ranger
    owns: {domains: [probe.range]}
    accepts: {commands: [probe.range.SetBag]}
    publishes: {events: [probe.range.BagSet]}
    reached_by: network
";
    for target in [Target::Rust, Target::Go] {
        let refused = synthesize_for(&ir(model), target)
            .err()
            .unwrap_or_else(|| panic!("{target:?}: a distinct struct invariant is refused"));
        let rendered = format!("{refused:?}");
        assert!(
            rendered.contains("types.probe.range.Bag.invariants.0")
                && rendered.contains("requires distinct list members"),
            "{target:?}: {rendered}"
        );
    }
}

/// A Go layout holding a package named `out` (here the domain `probe.out`) moves every check
/// local, the receiver included, off that name; the struct decoder still declares `var out`.
/// The same model without the invariant is the control: it compiles before and after the unit.
#[test]
fn adv_go_struct_check_compiles_beside_a_package_named_out() {
    if !has_go() {
        eprintln!("no Go toolchain on this machine; the Go struct decoder is unchecked here");
        return;
    }
    let model = "format: ess/22
system: probe
version: v1
domain: probe.out
types:
  - name: probe.out.Window
    kind: struct
    fields:
      - {name: low, type: Integer}
      - {name: high, type: Integer}
    invariants:
      - low >= 0
events:
  - name: probe.out.WindowSet
    fields: [{name: note, type: String}]
commands:
  - name: probe.out.SetWindow
    input:
      - {name: window, type: probe.out.Window}
      - {name: note, type: String}
    outcomes:
      - name: set
        emits: [probe.out.WindowSet]
        payload:
          probe.out.WindowSet: {note: input.note}
components:
  - component: ranger
    owns: {domains: [probe.out]}
    accepts: {commands: [probe.out.SetWindow]}
    publishes: {events: [probe.out.WindowSet]}
    reached_by: network
";
    let bare = model.replace("    invariants:\n      - low >= 0\n", "");
    let mut verdicts = Vec::new();
    for (label, text) in [("control", bare.as_str()), ("checked", model)] {
        let synthesis = synthesize_for(&ir(text), Target::Go).expect("the model synthesizes");
        let root = scratch(&format!("go-out-{label}"));
        write(&synthesis, &root);
        let vet = go_tool(&root, "go", &["vet", "./..."]);
        let _ = std::fs::remove_dir_all(&root);
        verdicts.push((
            label,
            vet.status.success(),
            String::from_utf8_lossy(&vet.stderr).into_owned(),
        ));
    }
    assert!(
        verdicts.iter().all(|(_, compiled, _)| *compiled),
        "the generated Go compiles beside a package named `out`: {verdicts:#?}"
    );
}

fn go_tool(directory: &Path, tool: &str, arguments: &[&str]) -> Output {
    let output = Command::new(tool)
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod -buildvcs=false")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("the Go tool runs");
    eprintln!(
        "{tool} {arguments:?} in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

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
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adv-struct-invariants-{label}"))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&destination, &artifact.contents).unwrap();
    }
}

fn cargo(directory: &Path, arguments: &[&str]) -> Output {
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(arguments)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo runs");
    eprintln!(
        "cargo {arguments:?} in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
