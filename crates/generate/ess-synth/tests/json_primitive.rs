//! A model that uses `Json` (beyond10x/ess#138).
//!
//! The Rust target represents it as the generated types crate's own dependency-free `json::Value`
//! (beyond10x/ess#224), at every position the target types: a newtype, a struct member (bare, in a
//! list, in a map, optional), a union variant, an entity field, a command input and response, an
//! event payload, an error payload and a view row. Go, web and the command-line target represent it
//! too (`json_go.rs`, `json_web.rs`, `json_clap.rs`), so no code target refuses a model for using it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Synthesis, Target};

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - {name: demo.msgs.Body, kind: newtype, of: Json}
events:
  - name: demo.msgs.Sent
    fields:
      - {name: body, type: demo.msgs.Body}
      - {name: headers, type: \"Map<String, Json>\"}
commands:
  - name: demo.msgs.Send
    input:
      - {name: body, type: demo.msgs.Body}
    outcomes:
      - name: sent
        emits: [demo.msgs.Sent]
        payload:
          demo.msgs.Sent: {body: input.body, headers: {generated: true}}
components:
  - component: msgs-service
    owns: {domains: [demo.msgs]}
    accepts: {commands: [demo.msgs.Send]}
    publishes: {events: [demo.msgs.Sent]}
    reached_by: network
";

/// `Json` at every position the Rust target types, served over the network so the server crate's
/// wire codecs are emitted too.
const EVERY_POSITION: &str = "format: ess/15
system: demo
version: v1
domain: demo.docs
types:
  - {name: demo.docs.Body, kind: newtype, of: Json}
  - name: demo.docs.Envelope
    kind: struct
    fields:
      - {name: body, type: Json}
      - {name: items, type: List<Json>}
      - {name: labels, type: \"Map<String, Json>\"}
      - {name: note, type: Optional<Json>}
  - name: demo.docs.Either
    kind: union
    tag: kind
    variants:
      raw: Json
      text: String
entities:
  - name: demo.docs.Doc
    identity: {name: id, type: String}
    fields:
      - {name: body, type: Json}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
events:
  - name: demo.docs.Stored
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
      - {name: wrapped, type: demo.docs.Body}
  - name: demo.docs.Noted
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
commands:
  - name: demo.docs.Store
    input:
      - {name: id, type: String}
      - {name: body, type: Json}
      - {name: wrapped, type: demo.docs.Body}
      - {name: envelope, type: demo.docs.Envelope}
      - {name: either, type: demo.docs.Either}
      - {name: items, type: List<Json>}
      - {name: labels, type: \"Map<String, Json>\"}
    response:
      - {name: echo, type: Json}
    outcomes:
      - name: stored
        emits: [demo.docs.Stored]
        payload:
          demo.docs.Stored: {id: input.id, body: input.body, wrapped: input.wrapped}
      - name: refused
        external: the store refuses the document
        error: demo.docs.Refused
  - name: demo.docs.Note
    input:
      - {name: id, type: String}
      - {name: body, type: Json}
    outcomes:
      - name: noted
        emits: [demo.docs.Noted]
        payload:
          demo.docs.Noted: {id: input.id, body: input.body}
errors:
  - name: demo.docs.Refused
    fields:
      - {name: detail, type: Json}
views:
  - name: demo.docs.Docs
    source: demo.docs.Doc
    consistency: eventual
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
components:
  - component: docs-service
    owns: {domains: [demo.docs]}
    accepts: {commands: [demo.docs.Store, demo.docs.Note]}
    publishes: {events: [demo.docs.Stored, demo.docs.Noted]}
    reached_by: network
bindings:
  - id: note-on-stored
    when: {event: demo.docs.Stored}
    invoke: {command: demo.docs.Note}
    mapping:
      id: event.id
      body: event.body
    delivery: at_least_once
    on_failure: retry
";

/// One command input carrying all six JSON kinds, at the top level of a `Json` field, inside a
/// newtype, inside a struct member, a union variant, a list and a map. Compact, and spelled the way
/// the generated writer spells it, so decoding then encoding must reproduce it byte for byte. A
/// `Map<String, Json>` is a `BTreeMap`, so its keys are in order here; a `Json` object's members
/// are not, and keep the order they arrived in.
const INPUT: &str = r#"{"id":"d1","body":{"object":{"k":"v"},"array":[1,"two",[]],"number":-12.50e3,"string":"say \"hi\"","bool":true,"null":null},"wrapped":[{},[],0,"",false,null],"envelope":{"body":null,"items":[{"a":1},[2],3.0,"four",true,null],"labels":{"a":[],"b":false,"n":1E2,"o":{},"s":"x","z":null},"note":{"deep":[{"deeper":null}]}},"either":{"kind":"raw","value":{"x":[true]}},"items":[null,{"m":"n"}],"labels":{"only":[1,{"two":2}]}}"#;

/// One view row whose `Json` column is an object holding the other five kinds.
const ROW_BODY: &str = r#"{"array":[1,2],"number":0.5,"string":"s","bool":false,"null":null}"#;

/// The throwaway crate that exercises the generated wire module from outside the generated tree,
/// so the generated bytes stay exactly what synthesis emitted.
const HARNESS: &str = r#"use demo_server::{json, wire};

fn main() {
    let input = std::env::args().nth(1).expect("the command input");
    let row_body = std::env::args().nth(2).expect("the view row's Json column");

    let document = json::parse(&input).expect("the input parses");
    let decoded = wire::decode_command_demo_docs_store(&document, "$").expect("the input decodes");
    let body: &demo_types::json::Value = &decoded.body;
    assert_eq!(body, document.member("body").unwrap());
    let mut encoded = String::new();
    wire::encode_command_demo_docs_store(&decoded, &mut encoded);
    assert_eq!(encoded, input, "a command input round-trips byte for byte");
    assert_eq!(json::parse(&encoded).unwrap(), document);

    let column: demo_types::json::Value = json::parse(&row_body).expect("the column parses");
    let row = demo_types::docs::Docs {
        id: "d1".to_owned(),
        body: column.clone(),
    };
    let mut written = String::new();
    wire::encode_view_demo_docs_docs(&row, &mut written);
    let expected = format!("{{\"id\":\"d1\",\"body\":{row_body}}}");
    assert_eq!(written, expected, "a view row carries its Json column unchanged");
    assert_eq!(json::parse(&written).unwrap().member("body"), Some(&column));
    println!("json round trip: command input and view row unchanged");
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

fn rust(source: &str) -> Synthesis {
    synthesize_for(&ir(source), Target::Rust)
        .unwrap_or_else(|failure| panic!("the Rust target represents Json: {failure}"))
}

fn artifact<'a>(synthesis: &'a Synthesis, path: &str) -> &'a str {
    &synthesis
        .artifacts
        .get(path)
        .unwrap_or_else(|| {
            panic!(
                "`{path}` was not emitted; emitted: {:?}",
                synthesis.artifacts.keys().collect::<Vec<_>>()
            )
        })
        .contents
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("json-primitive-{label}"))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&destination, &artifact.contents).unwrap();
    }
}

/// Cargo inside a generated tree, with every warning an error: generated code that warns is an
/// emitter defect a consumer's own `-D warnings` build would inherit.
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

#[test]
fn issue_224_every_code_target_synthesizes_a_json_model() {
    for source in [MODEL, EVERY_POSITION] {
        let ir = ir(source);
        for target in [Target::Rust, Target::Go, Target::Web, Target::Clap] {
            if let Err(failure) = synthesize_for(&ir, target) {
                panic!("{target:?} refused a Json model: {failure}");
            }
        }
    }
}

#[test]
fn issue_224_rust_names_json_value_at_every_position() {
    let synthesis = rust(EVERY_POSITION);
    let lib = artifact(&synthesis, "crates/demo-types/src/lib.rs");
    assert!(lib.contains("pub mod json;"), "{lib}");
    let json = artifact(&synthesis, "crates/demo-types/src/json.rs");
    assert!(json.contains("pub enum Value {"), "{json}");
    assert!(json.contains("pub fn push_value("), "{json}");
    let manifest = artifact(&synthesis, "crates/demo-types/Cargo.toml");
    assert!(
        manifest.contains("[dependencies]\n\n[features]\n"),
        "semantic types keep an empty unconditional dependency table: {manifest}"
    );
    assert!(
        manifest.contains("memory = [\"dep:uuid\", \"dep:time\"]")
            && manifest
                .lines()
                .filter(|line| line.starts_with("uuid =") || line.starts_with("time ="))
                .all(|line| line.contains("optional = true")),
        "the native demo runtime is explicitly optional: {manifest}"
    );
    assert!(
        !manifest.contains("serde"),
        "Json remains the generated value type: {manifest}"
    );

    let module = artifact(&synthesis, "crates/demo-types/src/docs.rs");
    for position in [
        "pub struct Body(pub crate::json::Value);",
        "pub body: crate::json::Value,",
        "pub items: Vec<crate::json::Value>,",
        "pub labels: std::collections::BTreeMap<String, crate::json::Value>,",
        "pub note: Option<crate::json::Value>,",
        "Raw(crate::json::Value)",
        "pub echo: crate::json::Value,",
        "pub detail: crate::json::Value,",
    ] {
        assert!(module.contains(position), "`{position}` missing:\n{module}");
    }
    // The entity's data, the command input, the event payload and the view row each carry a
    // `body: Json` — one occurrence per record, so count them rather than find one.
    let bodies = module.matches("pub body: crate::json::Value,").count();
    assert!(
        bodies >= 5,
        "Envelope, DocData, Store, Stored and Docs each carry `body`; found {bodies}:\n{module}"
    );

    let server = artifact(&synthesis, "crates/demo-server/src/json.rs");
    assert!(server.contains("pub use demo_types::json::*;"), "{server}");
    assert!(!server.contains("pub enum Value"), "{server}");
    let wire = artifact(&synthesis, "crates/demo-server/src/wire.rs");
    assert!(wire.contains("json::push_value(out, "), "{wire}");
    // A binding copies a `Json` event field into a `Json` command input in the system crate.
    assert!(
        synthesis
            .artifacts
            .iter()
            .any(|(path, artifact)| path.starts_with("crates/demo-system/")
                && artifact.contents.contains("note-on-stored")),
        "the binding carrying a Json field is emitted"
    );

    let directory = scratch("every-position");
    write(&synthesis, &directory);
    assert!(cargo(&directory, &["generate-lockfile", "--offline"])
        .status
        .success());
    let target = scratch("target");
    let target = target.to_str().unwrap();
    let check = cargo(
        &directory,
        &[
            "check",
            "--locked",
            "--offline",
            "--workspace",
            "--all-targets",
            "--target-dir",
            target,
        ],
    );
    assert!(
        check.status.success(),
        "the generated Rust workspace does not compile"
    );
}

#[test]
fn issue_224_json_round_trips_through_a_command_input_and_a_view_row() {
    let synthesis = rust(EVERY_POSITION);
    let root = scratch("round-trip");
    let generated = root.join("generated");
    write(&synthesis, &generated);
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        "[package]\nname = \"json-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
         [dependencies]\ndemo-server = { path = \"../generated/crates/demo-server\" }\n\
         demo-types = { path = \"../generated/crates/demo-types\" }\n\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(harness.join("src/main.rs"), HARNESS).unwrap();
    let target = scratch("target");
    let run = cargo(
        &harness,
        &[
            "run",
            "--offline",
            "--quiet",
            "--target-dir",
            target.to_str().unwrap(),
            "--",
            INPUT,
            ROW_BODY,
        ],
    );
    assert!(
        run.status.success(),
        "the six JSON kinds do not round-trip through the generated wire module"
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "json round trip: command input and view row unchanged\n"
    );
}

#[test]
fn issue_224_a_model_without_json_emits_no_json_module_in_the_types_crate() {
    let synthesis = rust(
        &MODEL
            .replace("of: Json}", "of: String}")
            .replace("\"Map<String, Json>\"", "\"Map<String, String>\""),
    );
    assert!(!synthesis
        .artifacts
        .contains_key("crates/demo-types/src/json.rs"));
    let lib = artifact(&synthesis, "crates/demo-types/src/lib.rs");
    assert!(!lib.contains("pub mod json;"), "{lib}");
    let server = artifact(&synthesis, "crates/demo-server/src/json.rs");
    assert!(server.contains("pub enum Value {"), "{server}");
}
