//! Distinct canonical commands whose codec names flatten to one identifier are served, each by its
//! own codecs (beyond10x/ess#415, `docs/design/served-codec-names.md`).
//!
//! Two models. `ALIASED` is the issue's own shape: `renewal.input.AB`, `renewal.input.AB2` and
//! `renewal.input.A_B` in one bounded context, the code aliases keeping their payload types apart.
//! `CROSS` reaches the same flattening from two bounded contexts (`renewal.lease.AB` and
//! `renewal.lease_a.B`) without any alias, which is the shape the Go target can also serve: Go
//! refuses `ALIASED`'s component port for a reason unrelated to codecs (its seam method names do
//! not follow `naming.code`), and a refused port serves nothing. `CROSS`'s contexts are not
//! called `input`, because a Go port handler's `input` parameter shadows a package of that name
//! on this base — another defect, also unrelated to codecs.
//!
//! Every served command answers with its own outcome and its own event, so a request routed to
//! another command's codecs or handler shows up as the wrong outcome, not as a passing build.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::http::{self, Served};
use ess_synth::{
    synthesize_for, synthesize_laid_out, OutputLayout, Synthesis, SynthesisFailure, Target,
    TargetFailureCode,
};

/// Three commands of one bounded context, two of which flatten to `renewal_input_a_b`.
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

/// The same flattening from two bounded contexts, with no alias anywhere: the system document and
/// one document per context, separated by [`DOCUMENT`].
const CROSS: &str = "format: ess/20
system: renewal
version: v1
domains: [renewal.lease, renewal.lease_a]
components:
  - component: renewal-service
    owns: {domains: [renewal.lease, renewal.lease_a]}
    accepts: {commands: [renewal.lease.AB, renewal.lease.AB2, renewal.lease_a.B]}
    publishes: {events: [renewal.lease.FirstSeen, renewal.lease.ThirdSeen, renewal.lease_a.SecondSeen]}
    reached_by: network
---
domain: renewal.lease
commands:
  - name: renewal.lease.AB
    input: [{name: value, type: Integer}]
    outcomes:
      - name: first
        emits: [renewal.lease.FirstSeen]
        payload:
          renewal.lease.FirstSeen: {value: input.value}
  - name: renewal.lease.AB2
    input: [{name: value, type: Integer}]
    outcomes:
      - name: third
        emits: [renewal.lease.ThirdSeen]
        payload:
          renewal.lease.ThirdSeen: {value: input.value}
events:
  - name: renewal.lease.FirstSeen
    fields: [{name: value, type: Integer}]
  - name: renewal.lease.ThirdSeen
    fields: [{name: value, type: Integer}]
---
domain: renewal.lease_a
commands:
  - name: renewal.lease_a.B
    input: [{name: value, type: Integer}]
    outcomes:
      - name: second
        emits: [renewal.lease_a.SecondSeen]
        payload:
          renewal.lease_a.SecondSeen: {value: input.value}
events:
  - name: renewal.lease_a.SecondSeen
    fields: [{name: value, type: Integer}]
";

/// What separates the documents of one fixture.
const DOCUMENT: &str = "---\n";

/// The commands of `CROSS`'s `renewal.lease` document, in their order, as one block to swap.
const CROSS_FIRST_TWO: &str = "  - name: renewal.lease.AB
    input: [{name: value, type: Integer}]
    outcomes:
      - name: first
        emits: [renewal.lease.FirstSeen]
        payload:
          renewal.lease.FirstSeen: {value: input.value}
  - name: renewal.lease.AB2
    input: [{name: value, type: Integer}]
    outcomes:
      - name: third
        emits: [renewal.lease.ThirdSeen]
        payload:
          renewal.lease.ThirdSeen: {value: input.value}
";

/// Per command of either model: its outcome, the event that outcome publishes, the Rust codec
/// stem and the Go codec stem the allocation rule gives it.
const ANSWERS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "renewal.input.AB",
        "first",
        "renewal.input.FirstSeen",
        "renewal_input_a_b",
        "RenewalInputAB",
    ),
    (
        "renewal.input.AB2",
        "third",
        "renewal.input.ThirdSeen",
        "renewal_input_a_b2",
        "RenewalInputAB2",
    ),
    (
        "renewal.input.A_B",
        "second",
        "renewal.input.SecondSeen",
        "renewal_input_a_b_2",
        "RenewalInputAB_2",
    ),
    (
        "renewal.lease.AB",
        "first",
        "renewal.lease.FirstSeen",
        "renewal_lease_a_b",
        "RenewalLeaseAB",
    ),
    (
        "renewal.lease.AB2",
        "third",
        "renewal.lease.ThirdSeen",
        "renewal_lease_a_b2",
        "RenewalLeaseAB2",
    ),
    (
        "renewal.lease_a.B",
        "second",
        "renewal.lease_a.SecondSeen",
        "renewal_lease_a_b_2",
        "RenewalLeaseAB_2",
    ),
];

fn answer_of(command: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    let (_, outcome, event, rust, go) = ANSWERS
        .iter()
        .find(|(name, ..)| *name == command)
        .unwrap_or_else(|| panic!("`{command}` is a fixture command"));
    (outcome, event, rust, go)
}

/// Every document of a fixture, assembled and resolved as one specification.
fn compile_model(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (position, document) in text.split(DOCUMENT).enumerate() {
        let label = format!("renewal-{position}.yaml");
        let raw = RawSpecFile::parse(document)
            .unwrap_or_else(|error| panic!("`{label}` is well formed: {error}"));
        sources.insert(label.clone(), document.to_owned());
        labels.push(label.clone());
        parsed.push((Source::new(label), raw));
    }
    let spec =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile_locating(&spec, &sources, &labels).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn embedded(text: &str) -> String {
    let stripped = text.replace("    reached_by: network\n", "");
    assert_ne!(stripped, text, "the fixture is served");
    stripped
}

/// `ALIASED` with its three command blocks, and its `accepts` list, reversed.
fn reversed_aliased() -> String {
    let start = ALIASED
        .find("  - name: renewal.input.AB\n")
        .expect("first command");
    let end = ALIASED.find("events:\n").expect("events");
    let body = &ALIASED[start..end];
    let mut blocks: Vec<&str> = Vec::new();
    let mut rest = body;
    while let Some(next) = rest[1..].find("  - name: renewal.input.") {
        blocks.push(&rest[..=next]);
        rest = &rest[next + 1..];
    }
    blocks.push(rest);
    assert_eq!(blocks.len(), 3, "three command blocks");
    blocks.reverse();
    let reordered = format!(
        "{}{}{}",
        &ALIASED[..start],
        blocks.concat(),
        &ALIASED[end..]
    )
    .replace(
        "accepts: {commands: [renewal.input.AB, renewal.input.AB2, renewal.input.A_B]}",
        "accepts: {commands: [renewal.input.A_B, renewal.input.AB2, renewal.input.AB]}",
    );
    assert_ne!(reordered, ALIASED);
    reordered
}

/// `CROSS` with its two documents, the first one's commands, and `accepts`, reversed.
fn reversed_cross() -> String {
    let documents: Vec<&str> = CROSS.split(DOCUMENT).collect();
    let [system, input, input_a] = documents[..] else {
        panic!("three documents");
    };
    let system = system
        .replace(
            "accepts: {commands: [renewal.lease.AB, renewal.lease.AB2, renewal.lease_a.B]}",
            "accepts: {commands: [renewal.lease_a.B, renewal.lease.AB2, renewal.lease.AB]}",
        )
        .replace(
            "domains: [renewal.lease, renewal.lease_a]\n",
            "domains: [renewal.lease_a, renewal.lease]\n",
        );
    let (ab, ab2) = CROSS_FIRST_TWO.split_at(
        CROSS_FIRST_TWO
            .find("  - name: renewal.lease.AB2\n")
            .expect("second command"),
    );
    let swapped = input.replace(CROSS_FIRST_TWO, &format!("{ab2}{ab}"));
    assert_ne!(swapped, input);
    format!("{system}{DOCUMENT}{input_a}{DOCUMENT}{swapped}")
}

fn commands(ir: &EssIr) -> Vec<String> {
    ir.commands()
        .values()
        .map(|command| command.name.to_string())
        .collect()
}

fn artifact<'a>(synthesis: &'a Synthesis, suffix: &str) -> &'a str {
    let found: Vec<_> = synthesis
        .artifacts
        .iter()
        .filter(|(path, _)| path.ends_with(suffix))
        .collect();
    assert_eq!(found.len(), 1, "exactly one `{suffix}` artifact");
    &found[0].1.contents
}

fn everything(synthesis: &Synthesis) -> String {
    synthesis
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect()
}

fn occurrences(text: &str, needle: &str) -> usize {
    text.matches(needle).count()
}

/// The Rust codecs and handlers of every command, declared once each, under the allocated stem,
/// and the declaration each one documents is the command it was allocated for.
fn assert_rust_names(label: &str, synthesis: &Synthesis, commands: &[String]) {
    let wire = artifact(synthesis, "wire.rs");
    let all = everything(synthesis);
    let mut stems = Vec::new();
    for command in commands {
        let (_, _, stem, _) = answer_of(command);
        stems.push(stem);
        for documented in [
            format!(
                "/// Reads the input of `{command}` from JSON.\n///\n/// # Errors\n///\n/// \
                 [`json::DecodeError`] naming the path and what the declaration says belongs \
                 there.\npub fn decode_command_{stem}("
            ),
            format!(
                "/// Writes the outcome of `{command}` as JSON: the branch taken, what it \
                 published, and the declared\n/// refusal it carries where it carries one.\npub \
                 fn encode_outcome_{stem}("
            ),
            format!("/// Writes the input of `{command}` as JSON.\npub fn encode_command_{stem}("),
        ] {
            assert_eq!(occurrences(wire, &documented), 1, "{label}: {documented}");
        }
        for function in [
            format!("pub fn decode_command_{stem}("),
            format!("pub fn encode_command_{stem}("),
            format!("pub fn encode_outcome_{stem}("),
        ] {
            assert_eq!(occurrences(wire, &function), 1, "{label}: `{function}`");
        }
        // A handler is generic over the system's parameters where it has any: `fn serve_x<…>(`.
        for function in [format!("fn serve_{stem}"), format!("fn run_{stem}")] {
            let declared = occurrences(&all, &format!("{function}("))
                + occurrences(&all, &format!("{function}<"));
            assert_eq!(declared, 1, "{label}: `{function}`");
        }
    }
    stems.sort_unstable();
    stems.dedup();
    assert_eq!(stems.len(), commands.len(), "{label}: one stem per command");
}

#[test]
fn rust_served_codecs_of_flattening_commands_are_allocated_by_rule() {
    for (model, text) in [("aliased", ALIASED), ("cross", CROSS)] {
        let ir = compile_model(text);
        for layout in [OutputLayout::Workspace, OutputLayout::Crate] {
            let label = format!("{model}/{}", layout.name());
            let synthesis = synthesize_laid_out(&ir, Target::Rust, layout)
                .unwrap_or_else(|error| panic!("{label} synthesizes: {error:?}"));
            assert_rust_names(&label, &synthesis, &commands(&ir));
        }
    }
}

#[test]
fn web_codecs_of_flattening_commands_are_allocated_by_rule() {
    let ir = compile_model(ALIASED);
    let synthesis =
        synthesize_for(&ir, Target::Web).unwrap_or_else(|error| panic!("Web: {error:?}"));
    let all = everything(&synthesis);
    for command in commands(&ir) {
        let (_, _, stem, _) = answer_of(&command);
        let routed = format!(
            "{command:?} => {{\n                let input = \
             wire::decode_command_{stem}(input, \"input\")?;"
        );
        assert_eq!(occurrences(&all, &routed), 1, "Web: {command}");
        assert_eq!(
            occurrences(&all, &format!("pub fn decode_command_{stem}(")),
            1,
            "Web: {command}"
        );
    }
}

#[test]
fn go_served_codecs_of_flattening_commands_are_allocated_by_rule() {
    let ir = compile_model(CROSS);
    let synthesis = synthesize_for(&ir, Target::Go).unwrap_or_else(|error| panic!("Go: {error:?}"));
    let wire = artifact(&synthesis, "server/wire.go");
    let all = everything(&synthesis);
    let mut stems = Vec::new();
    for command in commands(&ir) {
        let (_, _, _, stem) = answer_of(&command);
        stems.push(stem);
        assert_eq!(
            occurrences(
                wire,
                &format!(
                    "// decodeCommand{stem} reads the input of `{command}` from JSON.\nfunc \
                     decodeCommand{stem}("
                )
            ),
            1,
            "Go: {command}"
        );
        for function in [
            format!("func decodeCommand{stem}("),
            format!("func serve{stem}("),
            format!("func answer{stem}("),
        ] {
            assert_eq!(occurrences(&all, &function), 1, "Go: `{function}`");
        }
    }
    stems.sort_unstable();
    stems.dedup();
    assert_eq!(stems.len(), 3, "Go: one stem per command");
}

/// A model with no flattening keeps every codec name it had: the plain flattened stem.
#[test]
fn a_model_without_flattening_keeps_its_plain_codec_names() {
    let text = ALIASED
        .replace("renewal.input.A_B", "renewal.input.Other")
        .replace("    naming: {code: Second}\n", "");
    let ir = compile_model(&text);
    let rust = synthesize_laid_out(&ir, Target::Rust, OutputLayout::Workspace).expect("Rust");
    let wire = artifact(&rust, "wire.rs");
    for stem in [
        "renewal_input_a_b",
        "renewal_input_a_b2",
        "renewal_input_other",
    ] {
        assert_eq!(
            occurrences(wire, &format!("pub fn decode_command_{stem}(")),
            1
        );
    }
    assert_eq!(occurrences(wire, "_2("), 0, "no suffix is allocated");
}

#[test]
fn declaration_order_does_not_change_generated_bytes() {
    let cases: [(&str, &str, String, &[Target]); 2] = [
        (
            "aliased",
            ALIASED,
            reversed_aliased(),
            &[Target::Rust, Target::Web],
        ),
        (
            "cross",
            CROSS,
            reversed_cross(),
            &[Target::Rust, Target::Go],
        ),
    ];
    for (model, text, reordered, targets) in cases {
        let ir = compile_model(text);
        let swapped = compile_model(&reordered);
        for target in targets {
            let layouts: &[OutputLayout] = if *target == Target::Rust {
                &[OutputLayout::Workspace, OutputLayout::Crate]
            } else {
                &[OutputLayout::Workspace]
            };
            for layout in layouts {
                let label = format!("{model}/{target:?}/{}", layout.name());
                let first = synthesize_laid_out(&ir, *target, *layout)
                    .unwrap_or_else(|error| panic!("{label}: {error:?}"));
                let second = synthesize_laid_out(&swapped, *target, *layout)
                    .unwrap_or_else(|error| panic!("{label} reordered: {error:?}"));
                assert_eq!(
                    first.artifacts.keys().collect::<Vec<_>>(),
                    second.artifacts.keys().collect::<Vec<_>>(),
                    "{label}"
                );
                for (path, artifact) in &first.artifacts {
                    assert_eq!(
                        artifact.contents, second.artifacts[path].contents,
                        "{label}: {path}"
                    );
                }
            }
        }
    }
}

/// Embedded/direct (no `reached_by: network`): no served surface, so no codec is allocated or
/// inventoried, and both targets admit the model exactly as before. Go refuses `ALIASED`'s
/// component port for its seam method names, which is that target's existing boundary.
#[test]
fn embedded_targets_keep_their_admission_boundary() {
    for (model, text) in [("aliased", ALIASED), ("cross", CROSS)] {
        let ir = compile_model(&embedded(text));
        for (target, layout) in [
            (Target::Rust, OutputLayout::Workspace),
            (Target::Rust, OutputLayout::Crate),
            (Target::Go, OutputLayout::Workspace),
        ] {
            let label = format!("{model}/{target:?}/{}", layout.name());
            let synthesis = synthesize_laid_out(&ir, target, layout)
                .unwrap_or_else(|error| panic!("{label}: {error:?}"));
            for path in synthesis.artifacts.keys() {
                assert!(
                    !path.contains("server") && !path.ends_with("wire.rs"),
                    "{label}: no served artifact, found `{path}`"
                );
            }
            for (path, artifact) in &synthesis.artifacts {
                for absent in [
                    "decode_command_",
                    "decodeCommand",
                    "fn serve_",
                    "func serve",
                ] {
                    assert!(
                        !artifact.contents.contains(absent),
                        "{label}: `{absent}` in `{path}`"
                    );
                }
            }
            if target == Target::Go {
                let notes = artifact(&synthesis, "TARGET.md");
                let refused = notes.contains(
                    "| component port | `renewal-service` | Go gives a type one method set",
                );
                assert_eq!(refused, model == "aliased", "{label}: {notes}");
            }
        }
    }
}

/// A flattening the per-family rule does not reach stays a typed refusal: a declared type's codecs
/// and a command's input codecs share the `encode_` and `decode_` prefixes, so a type whose
/// flattened name begins with `command_` lands on the same two functions.
#[test]
fn a_codec_collision_across_declaration_kinds_remains_a_wire_collision() {
    let text = "format: ess/20
system: command
version: v1
domains: [command.x, command.command.x]
components:
  - component: command-service
    owns: {domains: [command.x, command.command.x]}
    accepts: {commands: [command.x.Y]}
    publishes: {events: [command.x.Done]}
    reached_by: network
---
domain: command.x
commands:
  - name: command.x.Y
    input: [{name: value, type: Integer}]
    outcomes:
      - name: done
        emits: [command.x.Done]
        payload:
          command.x.Done: {value: input.value}
events:
  - name: command.x.Done
    fields: [{name: value, type: Integer}]
---
domain: command.command.x
types:
  - {name: command.command.x.Y, kind: newtype, of: Integer}
";
    let ir = compile_model(text);
    for layout in [OutputLayout::Workspace, OutputLayout::Crate] {
        let Err(SynthesisFailure::Target(failure)) = synthesize_laid_out(&ir, Target::Rust, layout)
        else {
            panic!("a cross-kind codec collision must stay refused");
        };
        let wire: Vec<_> = failure
            .causes()
            .iter()
            .filter(|cause| cause.code() == TargetFailureCode::WireCollision)
            .collect();
        assert_eq!(wire.len(), 2, "{:?}", failure.causes());
        for (cause, function) in wire
            .iter()
            .zip(["decode_command_command_x_y", "encode_command_command_x_y"])
        {
            assert_eq!(cause.sources(), ["command.command.x.Y", "command.x.Y"]);
            assert!(cause.detail().contains(function), "{}", cause.detail());
        }
    }
}

// ---- served calls --------------------------------------------------------------------------------

/// `name method path` per command route, in route order.
fn route_table(ir: &EssIr) -> Vec<(String, String, String)> {
    let component = ir.components().values().next().expect("one component");
    http::routes(ir, component)
        .into_iter()
        .filter_map(|route| match route.serves {
            Served::Command(handle) => Some((
                ir.command(handle).name.to_string(),
                route.method.as_str().to_owned(),
                route.path,
            )),
            Served::View(_) => None,
        })
        .collect()
}

/// What a harness prints for the route table: the command at position `n` is sent
/// `{"value": n + 1}` over the served dispatch and, where the target has a transport-free entry,
/// `{"value": 10 * (n + 1)}` through it.
fn expected(ir: &EssIr, handled: bool) -> Vec<String> {
    let mut lines = Vec::new();
    for (position, (name, _, _)) in route_table(ir).iter().enumerate() {
        let value = position + 1;
        let (outcome, event, _, _) = answer_of(name);
        lines.push(format!(
            "served {name} 202 {{\"outcome\":\"{outcome}\",\"published\":[{{\"event\":\
             \"{event}\",\"payload\":{{\"value\":{value}}}}}]}}"
        ));
        if handled {
            lines.push(format!("handled {name} {outcome} {}", value * 10));
        }
    }
    lines
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("served-codec-names-{label}-{}", std::process::id()))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn report(what: &str, output: &Output) -> Vec<String> {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    eprintln!(
        "{what}\n{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    stdout.lines().map(str::to_owned).collect()
}

const RUST_HARNESS: &str = r#"use renewal_server::renewal_service as surface;
use renewal_server::{http, json};
use renewal_types::behaviour::Generated;

fn first_value(answer: Option<&json::Value>) -> String {
    let Some(json::Value::Array(items)) = answer.and_then(|value| value.member("published")) else {
        return "none".to_owned();
    };
    match items
        .first()
        .and_then(|item| item.member("payload"))
        .and_then(|payload| payload.member("value"))
    {
        Some(json::Value::Number(number)) => number.clone(),
        _ => "none".to_owned(),
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut system =
        renewal_system::System::new(renewal_service::RenewalService::new(Generated::new(())));
    for (position, row) in arguments.chunks(3).enumerate() {
        let (name, method, path) = (&row[0], &row[1], &row[2]);
        let value = position + 1;
        let request = http::Request {
            method: method.clone(),
            path: path.clone(),
            query: String::new(),
            headers: Vec::new(),
            body: format!("{{\"value\":{value}}}").into_bytes(),
        };
        let answered = surface::dispatch(&mut system, &request);
        println!("served {name} {} {}", answered.status, answered.body);
        let handled = surface::handle(
            &mut system,
            name,
            json::parse(&format!("{{\"value\":{}}}", value * 10)).expect("JSON"),
        );
        let answer = handled.as_ref().ok();
        let outcome = match answer.and_then(|value| value.member("outcome")) {
            Some(json::Value::Text(text)) => text.clone(),
            _ => "none".to_owned(),
        };
        println!("handled {name} {outcome} {}", first_value(answer));
    }
}
"#;

fn run_rust(label: &str, text: &str, layout: OutputLayout) {
    let ir = compile_model(text);
    let synthesis = synthesize_laid_out(&ir, Target::Rust, layout)
        .unwrap_or_else(|error| panic!("{label} synthesizes: {error:?}"));
    let root = scratch(label);
    write(&synthesis, &root.join("renewal"));
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    let (dependencies, source) = match layout {
        OutputLayout::Crate => (
            "renewal = { path = \"../renewal\", features = [\"server\"] }\n".to_owned(),
            RUST_HARNESS
                .replace("renewal_server::", "renewal::server::")
                .replace("renewal_system::", "renewal::system::")
                .replace("renewal_service::", "renewal::ports::renewal_service::")
                .replace("renewal_types::", "renewal::"),
        ),
        OutputLayout::Workspace => (
            [
                "renewal-server",
                "renewal-system",
                "renewal-service",
                "renewal-types",
            ]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../renewal/crates/{name}\" }}");
                out
            }),
            RUST_HARNESS.to_owned(),
        ),
    };
    std::fs::write(
        harness.join("Cargo.toml"),
        format!(
            "[package]\nname = \"served-codec-names-harness\"\nversion = \"0.0.0\"\nedition = \
             \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
        ),
    )
    .expect("write");
    std::fs::write(harness.join("src/main.rs"), source).expect("write");
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
    for (name, method, path) in route_table(&ir) {
        arguments.extend([name, method, path]);
    }
    let ran = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(&arguments)
        .current_dir(&harness)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo runs");
    let lines = report(label, &ran);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        ran.status.success(),
        "{label}: the harness builds with -D warnings and runs"
    );
    assert_eq!(lines, expected(&ir, true), "{label}");
}

#[test]
fn rust_workspace_serves_each_flattening_command_through_its_own_handler() {
    run_rust("rust-aliased-workspace", ALIASED, OutputLayout::Workspace);
}

#[test]
fn rust_single_crate_serves_each_flattening_command_through_its_own_handler() {
    run_rust("rust-aliased-crate", ALIASED, OutputLayout::Crate);
}

#[test]
fn rust_workspace_serves_flattening_commands_of_two_contexts() {
    run_rust("rust-cross-workspace", CROSS, OutputLayout::Workspace);
}

const GO_HARNESS: &str = r#"package server

import (
	"fmt"
	"net/http/httptest"
	"os"
	"strings"
	"testing"

	"example.invalid/renewal/components/renewalservice"
	"example.invalid/renewal/system"
	"example.invalid/renewal/types/behaviour"
)

func TestServedCodecNames(t *testing.T) {
	s := system.NewSystem(renewalservice.New(behaviour.New(behaviour.Ports{})))
	for position, row := range strings.Split(os.Getenv("SERVED_ROUTES"), "\n") {
		fields := strings.Split(row, " ")
		name, method, path := fields[0], fields[1], fields[2]
		body := fmt.Sprintf(`{"value":%d}`, position+1)
		answer := dispatchRenewalService(s, httptest.NewRequest(method, path, strings.NewReader(body)))
		fmt.Printf("served %s %d %s\n", name, answer.status, strings.ReplaceAll(answer.body, "\n", ""))
	}
}
"#;

#[test]
fn go_serves_flattening_commands_of_two_contexts() {
    let ir = compile_model(CROSS);
    let synthesis = synthesize_for(&ir, Target::Go).expect("Go synthesizes");
    let directory = scratch("go-cross");
    write(&synthesis, &directory);
    std::fs::write(
        directory.join("server/served_codec_names_test.go"),
        GO_HARNESS,
    )
    .expect("write");
    let routes = route_table(&ir)
        .into_iter()
        .map(|(name, method, path)| format!("{name} {method} {path}"))
        .collect::<Vec<_>>()
        .join("\n");
    let ran = Command::new("go")
        .args([
            "test",
            "-count=1",
            "-timeout",
            "120s",
            "-v",
            "-run",
            "TestServedCodecNames",
            "./server/",
        ])
        .current_dir(&directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local")
        .env("SERVED_ROUTES", routes)
        .output()
        .unwrap_or_else(|error| panic!("`go` runs: {error}"));
    let lines: Vec<String> = report("go harness", &ran)
        .into_iter()
        .filter(|line| line.starts_with("served "))
        .collect();
    let _ = std::fs::remove_dir_all(&directory);
    assert!(ran.status.success(), "the Go harness builds and runs");
    assert_eq!(lines, expected(&ir, false));
}
