//! A model that uses `Json`, synthesized for the command-line target (beyond10x/ess#224).
//!
//! The clap target types exactly one thing about a field: the flag it becomes. A `Json` flag —
//! bare, optional, repeated, or through a newtype — takes one argument holding a JSON document,
//! validated at parse time by the same dependency-free reader the Rust target's types crate carries,
//! and hands the handler that crate's `json::Value`. A model that uses `Json` gets the Rust
//! target's `json` module byte for byte, so a handler prints a `Json` response with the same
//! `json::push_value` that keeps member order and number spelling; a model without `Json` gets
//! neither, and keeps its previous bytes.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Synthesis, Target};

/// `Json` at every position a model can name it, and at every flag position the clap target types:
/// bare (`body`), through a newtype (`wrapped`), repeated (`items`) and optional (`note`). A struct,
/// union or map input stays one free-text flag, as it is for every other member type.
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
    naming: {wire: store}
    input:
      - {name: id, type: String}
      - {name: body, type: Json}
      - {name: wrapped, type: demo.docs.Body}
      - {name: envelope, type: demo.docs.Envelope}
      - {name: either, type: demo.docs.Either}
      - {name: items, type: List<Json>}
      - {name: labels, type: \"Map<String, Json>\"}
      - {name: note, type: Optional<Json>}
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
    naming: {wire: note}
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
    naming: {wire: docs}
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
    reached_by: command_line
    cli:
      binary: docs
      commands: [demo.docs.Store, demo.docs.Note]
      views: [demo.docs.Docs]
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

/// One document of each of the six JSON kinds, compact and spelled the way `json::push_value`
/// writes it, so parsing then writing must reproduce it byte for byte. The object keeps a member
/// order that is not sorted and a number spelling a float would change.
const KINDS: [&str; 6] = [
    r#"{"z":1,"a":[true,null],"n":-12.50e3,"s":"say \"hi\""}"#,
    r#"[{},[],0,"",false,null]"#,
    "1.0E+2",
    r#""text""#,
    "true",
    "null",
];

/// The throwaway crate that drives the generated grammar from outside the generated tree: it
/// includes the emitted `tree` and `json` modules by path, so the bytes under test are exactly the
/// bytes synthesis emitted.
const HARNESS: &str = r#"#![allow(dead_code)]

#[path = "../../generated/crates/demo-cli/src/json.rs"]
mod json;
#[path = "../../generated/crates/demo-cli/src/tree.rs"]
mod tree;

fn written(value: &json::Value) -> String {
    let mut out = String::new();
    json::push_value(&mut out, value);
    out
}

fn store(extra: &[&str]) -> Result<::clap::ArgMatches, ::clap::Error> {
    let mut arguments = vec![
        "docs", "store", "--id", "d1", "--envelope", "{}", "--either", "{}", "--labels", "{}",
    ];
    arguments.extend_from_slice(extra);
    tree::command()
        .try_get_matches_from(arguments)
        .map(|matches| matches.subcommand_matches("store").unwrap().clone())
}

fn main() {
    let kinds: Vec<String> = std::env::args().skip(1).collect();
    assert_eq!(kinds.len(), 6, "six kinds");
    for kind in &kinds {
        let matches = store(&[
            "--body", kind, "--wrapped", kind, "--items", kind, "--items", kind, "--note", kind,
        ])
        .unwrap_or_else(|error| panic!("`{kind}` parses: {error}"));
        for flag in ["body", "wrapped", "note"] {
            let value = matches
                .get_one::<json::Value>(flag)
                .unwrap_or_else(|| panic!("--{flag} carries a json::Value"));
            assert_eq!(&written(value), kind, "--{flag} keeps `{kind}` unchanged");
        }
        let items: Vec<String> = matches
            .get_many::<json::Value>("items")
            .expect("--items repeats")
            .map(written)
            .collect();
        assert_eq!(items, [kind.clone(), kind.clone()], "--items keeps each occurrence");
        let noted = tree::command()
            .try_get_matches_from(["docs", "note", "--id", "d1", "--body", kind])
            .unwrap_or_else(|error| panic!("`note --body {kind}` parses: {error}"));
        let body = noted
            .subcommand_matches("note")
            .unwrap()
            .get_one::<json::Value>("body")
            .expect("--body carries a json::Value");
        assert_eq!(&written(body), kind);
    }
    let without_note = store(&["--body", "null", "--wrapped", "null"]).expect("--note is optional");
    assert!(without_note.get_one::<json::Value>("note").is_none());
    for malformed in ["{", "[1,]", "tru", "01x", "\"open", "{\"a\" 1}", ""] {
        let error = store(&["--body", malformed, "--wrapped", "null"])
            .expect_err("a malformed document is refused at parse time");
        assert_eq!(
            error.kind(),
            ::clap::error::ErrorKind::ValueValidation,
            "`{malformed}`: {error}"
        );
    }
    println!("json flags: six kinds unchanged");
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

fn clap(source: &str) -> Synthesis {
    synthesize_for(&ir(source), Target::Clap)
        .unwrap_or_else(|failure| panic!("the clap target represents Json: {failure}"))
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

/// The `.arg(` block of one flag in the emitted tree.
fn flag<'a>(tree: &'a str, name: &str) -> &'a str {
    tree.split(".arg(")
        .find(|block| block.contains(&format!("::clap::Arg::new({name:?})")))
        .unwrap_or_else(|| panic!("--{name} is emitted:\n{tree}"))
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("json-clap-{label}"))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&destination, &artifact.contents).unwrap();
    }
}

/// Cargo inside a scratch workspace, with every warning an error: generated code that warns is an
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
fn issue_224_clap_parses_a_json_flag_as_a_document_at_every_flag_position() {
    let synthesis = clap(EVERY_POSITION);
    let json = artifact(&synthesis, "crates/demo-cli/src/json.rs");
    assert!(json.contains("pub enum Value {"), "{json}");
    assert!(json.contains("pub fn parse("), "{json}");
    assert!(json.contains("pub fn push_value("), "{json}");
    let main = artifact(&synthesis, "crates/demo-cli/src/main.rs");
    assert!(main.contains("mod json;"), "{main}");
    let manifest = artifact(&synthesis, "crates/demo-cli/Cargo.toml");
    assert!(
        manifest.ends_with("[dependencies]\nclap = \"4\"\nclap_complete = \"4\"\n"),
        "no dependency beyond the two the target already has:\n{manifest}"
    );

    let tree = artifact(&synthesis, "crates/demo-cli/src/tree.rs");
    for name in ["body", "wrapped", "items", "note"] {
        let block = flag(tree, name);
        assert!(block.contains("crate::json::parse"), "--{name}:{block}");
    }
    for name in ["id", "envelope", "either", "labels"] {
        let block = flag(tree, name);
        assert!(!block.contains("crate::json"), "--{name}:{block}");
    }
    assert!(flag(tree, "items").contains("ArgAction::Append"));
    assert!(!flag(tree, "note").contains(".required(true)"));
}

#[test]
fn issue_224_the_generated_cli_builds_and_parses_each_of_the_six_kinds() {
    let synthesis = clap(EVERY_POSITION);
    let root = scratch("six-kinds");
    let _ = std::fs::remove_dir_all(&root);
    write(&synthesis, &root.join("generated"));
    std::fs::create_dir_all(root.join("harness/src")).unwrap();
    std::fs::write(
        root.join("harness/Cargo.toml"),
        "[package]\nname = \"json-clap-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
         [dependencies]\nclap = \"4\"\nclap_complete = \"4\"\n",
    )
    .unwrap();
    std::fs::write(root.join("harness/src/main.rs"), HARNESS).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"generated/crates/demo-cli\", \"harness\"]\n",
    )
    .unwrap();
    assert!(cargo(&root, &["generate-lockfile", "--offline"])
        .status
        .success());
    let target = scratch("target");
    let target = target.to_str().unwrap();
    let build = cargo(
        &root,
        &[
            "build",
            "--locked",
            "--offline",
            "--workspace",
            "--target-dir",
            target,
        ],
    );
    assert!(
        build.status.success(),
        "the generated CLI does not build with -D warnings"
    );

    let run = Command::new(Path::new(target).join("debug/json-clap-harness"))
        .args(KINDS)
        .output()
        .expect("the harness runs");
    assert!(
        run.status.success(),
        "the six JSON kinds do not survive the generated grammar:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "json flags: six kinds unchanged\n"
    );

    // The emitted binary itself: every kind is accepted, reaching the handler that owes it, and a
    // malformed document is a usage error naming the reader's refusal.
    let binary = Path::new(target).join("debug/docs");
    for kind in KINDS {
        let accepted = Command::new(&binary)
            .args(["note", "--id", "d1", "--body", kind])
            .output()
            .expect("the generated binary runs");
        let stderr = String::from_utf8_lossy(&accepted.stderr);
        assert_eq!(accepted.status.code(), Some(1), "`{kind}`: {stderr}");
        assert!(
            stderr.contains("`demo.docs.Note` is an obligation nothing has implemented"),
            "`{kind}`: {stderr}"
        );
    }
    let refused = Command::new(&binary)
        .args(["note", "--id", "d1", "--body", "{\"a\":}"])
        .output()
        .expect("the generated binary runs");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert_eq!(refused.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("at byte 5: expected a value"), "{stderr}");
}

#[test]
fn issue_224_a_model_without_json_emits_no_json_module_in_the_cli() {
    let synthesis = clap(
        &EVERY_POSITION
            .replace("Json", "String")
            .replace("raw: String", "raw: Integer"),
    );
    assert!(!synthesis
        .artifacts
        .contains_key("crates/demo-cli/src/json.rs"));
    let main = artifact(&synthesis, "crates/demo-cli/src/main.rs");
    assert!(!main.contains("mod json;"), "{main}");
    let tree = artifact(&synthesis, "crates/demo-cli/src/tree.rs");
    assert!(!tree.contains("crate::json"), "{tree}");
}
