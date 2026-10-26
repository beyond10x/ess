//! Adversary pass 1 on derived variant identifiers and a variant's `code:` (beyond10x/ess#662).
//!
//! The unit's own tests reach a dotted or coded label through the codecs only. These cases reach
//! the same labels through every other site that names a variant (a guard, a stored-row guard, a
//! payload and `sets:` literal, an entity invariant, a view filter), through an authored `code:`
//! that is not an identifier the target can use, and through collisions the derivation creates.
//! The every-site case checks a generated Rust workspace with cargo, so it lives in
//! `optional_recursive_codecs.rs`, a file the `no_test_builds` allowlist already holds.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Synthesis, Target};

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
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary-g-labels-{label}"))
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

fn go_vet(directory: &Path) -> Output {
    let mut command = Command::new("go");
    command
        .args(["vet", "./..."])
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local");
    if std::env::var_os("GOCACHE").is_none() {
        command.env("GOCACHE", scratch("gocache"));
    }
    run(command, "go")
}

fn causes(ir: &EssIr, target: Target) -> Vec<(String, Vec<String>, String)> {
    let failure = synthesize_for(ir, target)
        .err()
        .unwrap_or_else(|| panic!("{} must refuse rather than emit", target.name()));
    let json: serde_json::Value = serde_json::from_str(&failure.to_canonical_json()).unwrap();
    json["causes"]
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
        .collect()
}

/// `CODED` with the authored identifier replaced.
fn coded(code: &str) -> String {
    format!(
        "format: ess/19
system: demo
version: v1
domain: demo.kernel
types:
  - name: demo.kernel.CallEvent
    kind: enum
    variants:
      - {{name: call.started, code: {code}}}
      - call.ended
events:
  - name: demo.kernel.Recorded
    fields:
      - {{name: event, type: demo.kernel.CallEvent}}
commands:
  - name: demo.kernel.Record
    input:
      - {{name: event, type: demo.kernel.CallEvent}}
    outcomes:
      - name: recorded
        emits: [demo.kernel.Recorded]
        payload:
          demo.kernel.Recorded: {{event: input.event}}
components:
  - component: kernel
    owns: {{domains: [demo.kernel]}}
    accepts: {{commands: [demo.kernel.Record]}}
    publishes: {{events: [demo.kernel.Recorded]}}
    reached_by: network
"
    )
}

/// Red: Go emits a variant `code:` verbatim as a package-level type. A lower-case code is an
/// unexported type the server package cannot name, and a keyword is no declaration at all; the
/// target must refuse it (as it refuses a code that is not spelled as an identifier) or emit Go
/// that vets.
#[test]
fn adv_g_go_refuses_or_compiles_a_variant_code_it_cannot_export() {
    let mut broken = Vec::new();
    for code in ["begun", "type"] {
        let ir = ir(&coded(code));
        match synthesize_for(&ir, Target::Go) {
            Err(failure) => {
                let text = failure.to_string();
                if !text.contains("call.started") {
                    broken.push(format!(
                        "`code: {code}`: the refusal names no variant: {text}"
                    ));
                }
            }
            Ok(go) => {
                let directory = scratch(&format!("go-code-{code}"));
                write(&go, &directory);
                if !go_vet(&directory).status.success() {
                    broken.push(format!(
                        "`code: {code}`: Go emitted a module that does not vet; see {}",
                        directory.display()
                    ));
                }
            }
        }
    }
    assert!(broken.is_empty(), "{}", broken.join("\n"));
}

/// Red: a refusal of an identifier the author wrote under `code:` tells the author to write
/// `code:`. It should say the written code is not an identifier.
#[test]
fn adv_g_an_invalid_authored_code_is_not_answered_with_write_a_code() {
    let mut circular = Vec::new();
    for (target, code) in [
        (Target::Rust, "Self"),
        (Target::Web, "Self"),
        (Target::Go, "'1x'"),
    ] {
        let ir = ir(&coded(code));
        let found = causes(&ir, target);
        let (kind, _, detail) = found
            .iter()
            .find(|(kind, _, _)| kind == "invalid-identifier")
            .unwrap_or_else(|| panic!("{}: {found:?}", target.name()));
        assert_eq!(kind, "invalid-identifier");
        if detail.contains("give the variant `naming: { code: … }`") {
            circular.push(format!(
                "{} `code: {code}`: the hint asks for the `code:` already written: {detail}",
                target.name()
            ));
        }
    }
    assert!(circular.is_empty(), "{}", circular.join("\n"));
}

/// Two union labels that derive one identifier.
const UNION_COLLIDING: &str = r"format: ess/19
system: demo
version: v1
domain: demo.kernel
types:
  - name: demo.kernel.Shape
    kind: union
    tag: kind
    variants:
      a.b: String
      a_b: Integer
";

/// Red: a union label declares no naming, so the collision hint that names `code:` cannot be
/// followed for it.
#[test]
fn adv_g_a_union_label_collision_does_not_offer_code() {
    let ir = ir(UNION_COLLIDING);
    for target in [Target::Rust, Target::Web] {
        let found = causes(&ir, target);
        assert_eq!(found.len(), 1, "{}: {found:?}", target.name());
        let (kind, sources, detail) = &found[0];
        assert_eq!(kind, "symbol-collision");
        assert_eq!(sources, &["demo.kernel.Shape.a.b", "demo.kernel.Shape.a_b"]);
        assert!(
            !detail.contains("naming: { code"),
            "{}: a union label cannot carry `code:`: {detail}",
            target.name()
        );
    }
}

/// A derived identifier beside a declared variant of that spelling.
const DERIVED_BESIDE_DECLARED: &str = r"format: ess/19
system: demo
version: v1
domain: demo.kernel
types:
  - name: demo.kernel.CallEvent
    kind: enum
    variants:
      - call.started
      - CallStarted
";

/// Regression guard: Rust and Web refuse the pair by name, naming both sources; Go suffixes one.
#[test]
fn adv_g_a_derived_identifier_beside_a_declared_one_is_a_rust_collision() {
    let ir = ir(DERIVED_BESIDE_DECLARED);
    for target in [Target::Rust, Target::Web] {
        let found = causes(&ir, target);
        assert_eq!(found.len(), 1, "{}: {found:?}", target.name());
        let (kind, sources, detail) = &found[0];
        assert_eq!(kind, "symbol-collision", "{}", target.name());
        assert_eq!(
            sources,
            &[
                "demo.kernel.CallEvent.CallStarted",
                "demo.kernel.CallEvent.call.started"
            ]
        );
        assert!(detail.contains("`CallStarted`"), "{detail}");
    }
    let go = synthesize_for(&ir, Target::Go).expect("Go suffixes the second identifier");
    let text: String = go.artifacts.values().map(|a| a.contents.as_str()).collect();
    assert!(text.contains("type CallEventCallStarted "));
    assert!(text.contains("type CallEventCallStarted_ "));
}

/// A coded variant whose code is a derived identifier of its sibling.
const CODE_BESIDE_DERIVED: &str = r"format: ess/19
system: demo
version: v1
domain: demo.kernel
types:
  - name: demo.kernel.CallEvent
    kind: enum
    variants:
      - {name: call.started, code: CallEnded}
      - call.ended
";

/// Regression guard: an authored code that lands on a sibling's derived identifier is a Rust and
/// Web collision too, naming both variants.
#[test]
fn adv_g_a_code_colliding_with_a_derived_sibling_is_a_rust_collision() {
    let ir = ir(CODE_BESIDE_DERIVED);
    for target in [Target::Rust, Target::Web] {
        let found = causes(&ir, target);
        assert_eq!(found.len(), 1, "{}: {found:?}", target.name());
        let (kind, sources, _) = &found[0];
        assert_eq!(kind, "symbol-collision", "{}", target.name());
        assert_eq!(
            sources,
            &[
                "demo.kernel.CallEvent.call.ended",
                "demo.kernel.CallEvent.call.started"
            ]
        );
    }
}

/// `served_codec_names.rs`'s `ALIASED`: three commands whose codec names flatten alike, kept apart
/// by declaration `naming: {code: …}`. No enum or union, so no derived variant identifier.
const ALIASED: &str = "format: ess/20
system: renewal
version: v1
domain: renewal.input
commands:
  - name: renewal.input.AB
    naming: {code: First}
    input: [{name: value, type: Integer}]
    outcomes:
      - name: first
        emits: [renewal.input.FirstSeen]
        payload:
          renewal.input.FirstSeen: {value: input.value}
  - name: renewal.input.AB2
    naming: {code: Third}
    input: [{name: value, type: Integer}]
    outcomes:
      - name: third
        emits: [renewal.input.ThirdSeen]
        payload:
          renewal.input.ThirdSeen: {value: input.value}
  - name: renewal.input.A_B
    naming: {code: Second}
    input: [{name: value, type: Integer}]
    outcomes:
      - name: second
        emits: [renewal.input.SecondSeen]
        payload:
          renewal.input.SecondSeen: {value: input.value}
events:
  - name: renewal.input.FirstSeen
    fields: [{name: value, type: Integer}]
  - name: renewal.input.SecondSeen
    fields: [{name: value, type: Integer}]
  - name: renewal.input.ThirdSeen
    fields: [{name: value, type: Integer}]
components:
  - component: renewal-service
    owns: {domains: [renewal.input]}
    accepts: {commands: [renewal.input.AB, renewal.input.AB2, renewal.input.A_B]}
    publishes: {events: [renewal.input.FirstSeen, renewal.input.SecondSeen, renewal.input.ThirdSeen]}
    reached_by: network
";

/// Red, pre-existing: the Go emitter answers `ALIASED` with a panic (its plan bijection assert)
/// rather than a refusal. The model has no variant, so this unit's change cannot reach it.
#[test]
fn adv_g_go_refuses_rather_than_panics_on_aliased_commands() {
    let ir = ir(ALIASED);
    let outcome = std::panic::catch_unwind(|| synthesize_for(&ir, Target::Go).map(|_| ()));
    assert!(
        outcome.is_ok(),
        "the Go emitter panicked instead of refusing or synthesizing"
    );
}
