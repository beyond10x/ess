//! Adversarial cases for a command whose input and response fields share a name: the response is
//! its own struct and its own field scope, `fields:{command}.response`.
//!
//! Each case either builds the generated code, serves a real answer through it, or reads the
//! refusal a collision now carries.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::http::{self, Served};
use ess_synth::{synthesize, synthesize_for, Synthesis, Target};
use serde_json::{json, Value};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

fn fixture(documents: &[(&str, &str)]) -> Result<EssIr, String> {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (label, text) in documents {
        let raw = RawSpecFile::parse(text).map_err(|error| format!("parse: {error}"))?;
        sources.insert((*label).to_owned(), (*text).to_owned());
        labels.push((*label).to_owned());
        parsed.push((Source::new((*label).to_owned()), raw));
    }
    let specification =
        Specification::assemble(parsed).map_err(|errors| format!("assemble: {errors}"))?;
    compile_locating(&specification, &sources, &labels)
        .map_err(|errors| format!("compile: {errors}"))
}

fn details(failure: &ess_synth::TargetFailure) -> String {
    failure
        .causes()
        .iter()
        .map(|cause| cause.detail().to_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-u8-{name}-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("scratch");
    directory
}

fn write_emission(directory: &Path, synthesis: &Synthesis) {
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("artifact parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn report(what: &str, output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    eprintln!(
        "{what}: {}\n{stdout}{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
}

fn cargo(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(arguments)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo runs")
}

/// `cargo check` of a generated workspace, offline, every warning an error.
fn check(directory: &Path, target: Option<&str>) {
    let lock = cargo(directory, &["generate-lockfile", "--offline"]);
    report("lock", &lock);
    assert!(lock.status.success(), "the lockfile resolves offline");
    let mut arguments = vec![
        "check",
        "--locked",
        "--offline",
        "--workspace",
        "--all-targets",
        "--target-dir",
        "target",
    ];
    if let Some(target) = target {
        arguments.extend(["--target", target]);
    }
    let checked = cargo(directory, &arguments);
    report("check", &checked);
    assert!(
        checked.status.success(),
        "the generated workspace compiles: {}",
        directory.display()
    );
}

/// A served command whose input and response share every field name but one, and that one pair
/// differs only by case and underscores: both become `foo_bar` in Rust. Struct, List and Optional
/// responses of one declared struct, and a keyword field, ride along.
const SHARED: &str = "format: ess/22
system: demo
version: v1
domain: demo.core
types:
  - name: demo.core.Body
    kind: struct
    fields:
      - {name: text, type: String}
      - {name: fooBar, type: String}
events:
  - name: demo.core.Sent
    fields:
      - {name: id, type: String}
commands:
  - name: demo.core.Send
    input:
      - {name: id, type: String}
      - {name: fooBar, type: String}
      - {name: type, type: String}
      - {name: body, type: demo.core.Body}
      - {name: bodies, type: List<demo.core.Body>}
      - {name: note, type: Optional<demo.core.Body>}
    response:
      - {name: id, type: String}
      - {name: foo_bar, type: String}
      - {name: type, type: String}
      - {name: body, type: demo.core.Body}
      - {name: bodies, type: List<demo.core.Body>}
      - {name: note, type: Optional<demo.core.Body>}
    outcomes:
      - name: sent
        returns: true
        emits: [demo.core.Sent]
        payload:
          demo.core.Sent:
            id: {generated: true}
actors:
  - name: demo.core.Writer
    may: [demo.core.Send]
components:
  - component: messages
    owns: {domains: [demo.core]}
    accepts: {commands: [demo.core.Send]}
    publishes: {events: [demo.core.Sent]}
    reached_by: network
";

fn shared() -> EssIr {
    fixture(&[("demo.yaml", SHARED)]).unwrap_or_else(|error| panic!("the model is ESS: {error}"))
}

#[test]
fn shared_struct_list_optional_and_mangled_names_compile_for_rust() {
    let synthesis = synthesize(&shared()).unwrap_or_else(|failure| panic!("{}", details(&failure)));
    let directory = scratch("rust");
    write_emission(&directory, &synthesis);
    check(&directory, None);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn shared_struct_list_optional_and_mangled_names_compile_for_web() {
    // The web bridge depends on the Rust workspace beside it, as `generated/<target>/<system>`.
    let ir = shared();
    let directory = scratch("web");
    let rust = directory.join("generated/rust/demo");
    let web = directory.join("generated/web/demo");
    write_emission(
        &rust,
        &synthesize(&ir).unwrap_or_else(|failure| panic!("{}", details(&failure))),
    );
    write_emission(
        &web,
        &synthesize_for(&ir, Target::Web).unwrap_or_else(|failure| panic!("{}", details(&failure))),
    );
    check(&web, Some("wasm32-unknown-unknown"));
    let _ = std::fs::remove_dir_all(&directory);
}

/// Echoes every input field into the response field of the same name.
const HARNESS: &str = r##"use demo_server::http;
use demo_server::messages as surface;
use demo_types::actor::{Actor, Caller};
use demo_types::core;
use demo_types::core::obligations::SendBehavior;
use demo_types::obligation::UnmetObligation;

struct Echo;

impl SendBehavior for Echo {
    fn send(&mut self, input: core::Send) -> Result<core::SendOutcome, UnmetObligation> {
        Ok(core::SendOutcome::Sent {
            sent: core::Sent { id: input.id.clone() },
            response: core::SendResponse {
                id: format!("re-{}", input.id),
                foo_bar: format!("re-{}", input.foo_bar),
                r#type: format!("re-{}", input.r#type),
                body: input.body,
                bodies: input.bodies,
                note: input.note,
            },
        })
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("the route");
    let mut system = demo_system::System::new(messages::Messages::new(Echo));
    let caller = Caller { actor: Actor::ALL[0] };
    let request = http::Request {
        method: "POST".to_owned(),
        path,
        query: String::new(),
        headers: Vec::new(),
        body: br#"{"id":"a","fooBar":"b","type":"c","body":{"text":"t","fooBar":"f"},"bodies":[{"text":"u","fooBar":"g"}],"note":{"text":"v","fooBar":"h"}}"#.to_vec(),
    };
    let answered = surface::dispatch(&mut system, Some(&caller), &request);
    println!("answer\t{}\t{}", answered.status, answered.body.replace('\n', " "));
}
"##;

fn route(ir: &EssIr) -> String {
    let component = ir.components().values().next().expect("one component");
    http::routes(ir, component)
        .into_iter()
        .find(|route| matches!(route.serves, Served::Command(_)))
        .expect("the command's route")
        .path
}

fn validate(ir: &EssIr, status: u16, body: &Value) -> Result<(), String> {
    let component = ir.components().values().next().expect("one component");
    let document: Value =
        serde_json::from_str(&ess_gen::openapi::json(ir, component)).expect("JSON");
    let at = &document["paths"][route(ir)]["post"]["responses"][status.to_string()]["content"]
        ["application/json"]["schema"];
    if !at.is_object() {
        return Err(format!("the document declares no {status}"));
    }
    let mut schema = document.clone();
    for key in ["$ref", "oneOf"] {
        if let Some(value) = at.get(key) {
            schema[key] = value.clone();
        }
    }
    let validator = jsonschema::draft202012::new(&schema).expect("the schema compiles");
    let errors: Vec<String> = validator
        .iter_errors(body)
        .map(|error| format!("{error} at {}", error.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[test]
fn the_served_answer_keeps_input_and_response_members_apart() {
    let ir = shared();
    let synthesis = synthesize(&ir).unwrap_or_else(|failure| panic!("{}", details(&failure)));
    let root = scratch("served");
    write_emission(&root.join("demo"), &synthesis);
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    let dependencies = ["demo-server", "demo-system", "messages", "demo-types"]
        .iter()
        .fold(String::new(), |mut out, name| {
            let _ = writeln!(out, "{name} = {{ path = \"../demo/crates/{name}\" }}");
            out
        });
    std::fs::write(
        harness.join("Cargo.toml"),
        format!(
            "[package]\nname = \"adversary-u8-harness\"\nversion = \"0.0.0\"\nedition = \
             \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
        ),
    )
    .expect("write");
    std::fs::write(harness.join("src/main.rs"), HARNESS).expect("write");
    let target = root.join("target");
    let ran = cargo(
        &harness,
        &[
            "run",
            "--offline",
            "--quiet",
            "--target-dir",
            target.to_str().expect("UTF-8"),
            "--",
            &route(&ir),
        ],
    );
    let stdout = report("harness", &ran);
    assert!(ran.status.success(), "the harness builds and runs");
    let _ = std::fs::remove_dir_all(&root);
    let line = stdout
        .lines()
        .find_map(|line| line.strip_prefix("answer\t"))
        .unwrap_or_else(|| panic!("the harness answered:\n{stdout}"));
    let (status, body) = line.split_once('\t').expect("status and body");
    let status: u16 = status.parse().expect("a status");
    let body: Value = serde_json::from_str(body).expect("a JSON body");
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body["response"],
        json!({
            "id": "re-a",
            "foo_bar": "re-b",
            "type": "re-c",
            "body": {"text": "t", "fooBar": "f"},
            "bodies": [{"text": "u", "fooBar": "g"}],
            "note": {"text": "v", "fooBar": "h"},
        }),
        "{body}"
    );
    validate(&ir, status, &body).expect("the published document admits the served answer");
}

/// The response scope `fields:demo.core.Send.response` is a string; a declaration whose qualified
/// name spells the same string allocates its fields in the same scope.
#[test]
fn a_declaration_named_like_the_response_scope_is_no_collision() {
    let system = "format: ess/22\nsystem: demo\nversion: v1\ndomains:\n  - demo.core\n";
    let core = "domain: demo.core\ntypes:\n  - name: demo.core.Send.response\n    naming: {code: Envelope}\n    kind: struct\n    fields:\n      - {name: id, type: String}\nevents:\n  - name: demo.core.Sent\n    fields:\n      - {name: id, type: String}\ncommands:\n  - name: demo.core.Send\n    input:\n      - {name: text, type: String}\n    response:\n      - {name: id, type: String}\n    outcomes:\n      - name: sent\n        returns: true\n        emits: [demo.core.Sent]\n        payload:\n          demo.core.Sent:\n            id: {generated: true}\n";
    let wiring = "components:\n  - component: messages\n    owns: {domains: [demo.core]}\n    accepts: {commands: [demo.core.Send]}\n    publishes: {events: [demo.core.Sent]}\n    reached_by: network\n";
    let ir = match fixture(&[
        ("system.yaml", system),
        ("core.yaml", core),
        ("wiring.yaml", wiring),
    ]) {
        Ok(ir) => ir,
        // The specification refuses the model: nobody reaches the shared scope.
        Err(error) => {
            eprintln!("refused before synthesis: {error}");
            return;
        }
    };
    if let Err(failure) = synthesize(&ir) {
        let detail = details(&failure);
        assert!(
            !detail.contains("in `fields:demo.core.Send.response`"),
            "two structs share one field scope by spelling: {detail}"
        );
    }
}

/// A type that takes the generated `{Command}Response` name collides with the response struct.
/// That collision is not field against field, so it keeps the `code:` remedy.
#[test]
fn a_mixed_collision_with_the_response_struct_keeps_the_code_remedy() {
    let model = SHARED.replace(
        "types:\n",
        "types:\n  - name: demo.core.SendResponse\n    kind: struct\n    fields:\n      - {name: other, type: String}\n",
    );
    let ir = fixture(&[("demo.yaml", &model)]).expect("the model is ESS");
    let detail = details(
        &synthesize(&ir)
            .err()
            .expect("`SendResponse` is allocated twice"),
    );
    assert!(
        detail.contains("`SendResponse` is allocated 2 times in `domain:"),
        "{detail}"
    );
    assert!(detail.contains("naming: { code: … }"), "{detail}");
    assert!(!detail.contains("naming: { wire: … }"), "{detail}");
}

/// Two response fields of one wire name write one JSON member twice; Rust refuses the same for
/// input fields (`wire_fields`).
#[test]
fn two_response_fields_of_one_wire_name_do_not_reach_the_codec() {
    let model = SHARED.replace(
        "    response:\n      - {name: id, type: String}\n      - {name: foo_bar, type: String}\n      - {name: type, type: String}\n      - {name: body, type: demo.core.Body}\n      - {name: bodies, type: List<demo.core.Body>}\n      - {name: note, type: Optional<demo.core.Body>}\n",
        "    response:\n      - {name: reply, type: String}\n      - {name: other, type: String, naming: {wire: reply}}\n",
    );
    assert_ne!(model, SHARED, "the response block is replaced");
    let ir = match fixture(&[("demo.yaml", &model)]) {
        Ok(ir) => ir,
        Err(error) => {
            assert!(error.contains("reply"), "{error}");
            return;
        }
    };
    match synthesize(&ir) {
        Err(failure) => assert!(details(&failure).contains("reply"), "{}", details(&failure)),
        Ok(synthesis) => {
            let wire = &synthesis.artifacts["crates/demo-server/src/wire.rs"].contents;
            let encoder = &wire[wire
                .find("pub fn encode_response_")
                .expect("the response encoder")..];
            let encoder = &encoder[..encoder.find("\n}\n").expect("it closes")];
            assert_eq!(
                encoder.matches("json::member(out, \"reply\");").count(),
                1,
                "the response encoder writes `reply` twice: {encoder}"
            );
        }
    }
}
