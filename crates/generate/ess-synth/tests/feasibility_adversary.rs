//! Independent generated-compiler attacks on actual target allocation scopes.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize, synthesize_for, Synthesis, SynthesisPlan, Target};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

fn fixture(body: &str, wiring: &str) -> EssIr {
    let documents = [
        (
            "system.yaml",
            "format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\n".to_owned(),
        ),
        ("core.yaml", format!("domain: demo.core\n{body}")),
        ("wiring.yaml", wiring.to_owned()),
    ];
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    let mut labels = Vec::new();
    for (name, text) in documents {
        let raw = RawSpecFile::parse(&text).expect("attack fixture syntax is valid");
        sources.insert(name.to_owned(), text);
        labels.push(name.to_owned());
        parsed.push((Source::new(name), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("attack fixture must be compiler-admitted ESS: {errors}"));
    compile_locating(&specification, &sources, &labels).unwrap_or_else(|errors| {
        panic!("attack fixture resolves before target admission: {errors}")
    })
}

fn scratch(label: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/review-boundaries-5/adversary-compiler")
        .join(format!(
            "{label}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn emit(directory: &Path, synthesis: &Synthesis) {
    for (path, artifact) in &synthesis.artifacts {
        assert_eq!(path, &artifact.path);
        let destination = directory.join(path);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, &artifact.contents).unwrap();
    }
}

fn cargo(directory: &Path, lane: &str, args: &[&str]) -> Output {
    let mut command = Command::new(std::env::var_os("CARGO").expect("Cargo executable"));
    command
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .args(args);
    std::fs::write(
        directory.join(format!("{lane}.command")),
        format!("{command:?}\n"),
    )
    .unwrap();
    let output = command.output().expect("the compiler witness must execute");
    std::fs::write(directory.join(format!("{lane}.stdout")), &output.stdout).unwrap();
    std::fs::write(directory.join(format!("{lane}.stderr")), &output.stderr).unwrap();
    std::fs::write(
        directory.join(format!("{lane}.exit")),
        output.status.to_string(),
    )
    .unwrap();
    eprintln!(
        "{command:?}\n{}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        output.status
    );
    output
}

fn compile_emitted(directory: &Path, synthesis: &Synthesis, wasm: bool) {
    emit(directory, synthesis);
    let lock = cargo(directory, "lock", &["generate-lockfile", "--offline"]);
    assert!(lock.status.success(), "compiler witness lock setup failed");
    let mut args = vec![
        "check",
        "--locked",
        "--offline",
        "--workspace",
        "--all-targets",
        "--target-dir",
        "target",
    ];
    if wasm {
        args.extend(["--target", "wasm32-unknown-unknown"]);
    }
    let output = cargo(directory, "check", &args);
    for (path, artifact) in &synthesis.artifacts {
        assert_eq!(
            std::fs::read(directory.join(path)).unwrap(),
            artifact.contents.as_bytes(),
            "compiler must preserve emitted artifact {path}"
        );
    }
    assert!(
        output.status.success(),
        "admitted generated workspace failed its actual compiler: {}",
        directory.display()
    );
}

fn event_model(component: &str, event: &str, network: bool) -> EssIr {
    fixture(
        &format!(
            "events:\n  - name: demo.core.{event}\ncommands:\n  - name: demo.core.Fire\n    outcomes:\n      - name: done\n        emits: [demo.core.{event}]\n"
        ),
        &format!(
            "components:\n  - component: {component}\n    owns:\n      domains: [demo.core]\n    accepts:\n      commands: [demo.core.Fire]\n    publishes:\n      events: [demo.core.{event}]\n{}",
            if network {
                "    reached_by: network\n"
            } else {
                ""
            }
        ),
    )
}

fn checked_failure(ir: &EssIr, target: Target, failure: &ess_synth::TargetFailure) {
    assert_eq!(failure.target(), target.name());
    assert_eq!(failure.plan(), &SynthesisPlan::of(ir));
    assert!(!failure.causes().is_empty());
    assert!(failure
        .causes()
        .iter()
        .all(|cause| !cause.sources().is_empty() && !cause.detail().is_empty()));
    let repeated = synthesize_for(ir, target)
        .err()
        .expect("deterministic refusal");
    assert_eq!(failure.to_canonical_json(), repeated.to_canonical_json());
}

fn web_dependency(component: &str) {
    let ir = event_model(component, "Fired", false);
    let directory = scratch(component);
    let rust =
        synthesize(&ir).expect("the component name is valid in its pure Rust dependency scopes");
    compile_emitted(&directory.join("generated/rust/demo"), &rust, false);
    match synthesize_for(&ir, Target::Web) {
        Err(failure) => {
            assert_ne!(
                component, "core",
                "the measured compilable control must remain admitted"
            );
            checked_failure(&ir, Target::Web, &failure);
        }
        Ok(web) => compile_emitted(&directory.join("generated/web/demo"), &web, true),
    }
}

#[test]
fn web_component_dependency_cannot_be_hidden_by_the_json_module() {
    web_dependency("json");
}

#[test]
fn web_component_dependency_cannot_capture_core_used_by_json_errors() {
    web_dependency("core");
}

#[test]
fn outcome_event_binding_cannot_capture_the_codec_output_buffer() {
    let ir = event_model("worker", "Out", true);
    match synthesize(&ir) {
        Err(failure) => checked_failure(&ir, Target::Rust, &failure),
        Ok(synthesis) => compile_emitted(&scratch("outcome-out"), &synthesis, false),
    }
}

#[test]
fn helper_like_event_names_remain_legal_without_the_codec_scope() {
    let ir = event_model("worker", "Out", false);
    let synthesis =
        synthesize(&ir).expect("Out is a valid semantic event and port binding without codecs");
    assert!(synthesis.target.is_none());
    compile_emitted(&scratch("outcome-out-pure-rust"), &synthesis, false);
}

#[test]
fn multiple_causes_keep_a_complete_unchanged_plan_and_canonical_order() {
    let ir = fixture(
        "types:\n  - name: demo.core.FooBar\n    kind: newtype\n    of: String\n  - name: demo.core.Foo_Bar\n    kind: newtype\n    of: String\n  - name: demo.core.Loop\n    kind: struct\n    fields:\n      - { name: next, type: Optional<demo.core.Loop> }\n",
        "components: []\n",
    );
    let error = synthesize(&ir)
        .err()
        .expect("two independent fatal source classes");
    checked_failure(&ir, Target::Rust, &error);
    assert!(error
        .causes()
        .iter()
        .any(|cause| cause.code() == ess_synth::TargetFailureCode::RecursiveLayout));
    assert!(error
        .causes()
        .iter()
        .any(|cause| cause.code() == ess_synth::TargetFailureCode::SymbolCollision));
    assert!(error.causes().windows(2).all(|pair| pair[0] < pair[1]));
    for cause in error.causes() {
        assert!(cause.sources().windows(2).all(|pair| pair[0] < pair[1]));
    }
    let json = error.to_canonical_json();
    assert!(json.ends_with('\n') && !json.ends_with("\n\n"));
    let object: BTreeMap<String, serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert_eq!(
        object.keys().map(String::as_str).collect::<Vec<_>>(),
        ["causes", "format", "plan", "target"]
    );
    let web = synthesize_for(&ir, Target::Web)
        .err()
        .expect("same prerequisite failures");
    assert_eq!(web.causes(), error.causes());
    assert_eq!(web.plan(), error.plan());
}

fn binding_model(names: &[&str]) -> EssIr {
    let body = "events:\n  - name: demo.core.Fired\n  - name: demo.core.Handled\ncommands:\n  - name: demo.core.Handle\n    outcomes:\n      - name: done\n        emits: [demo.core.Handled]\n";
    let mut wiring = "components:\n  - component: worker\n    owns:\n      domains: [demo.core]\n    accepts:\n      commands: [demo.core.Handle]\n    publishes:\n      events: [demo.core.Fired, demo.core.Handled]\nbindings:\n".to_owned();
    for name in names {
        write!(wiring, "  - id: {name}\n    when:\n      event: demo.core.Fired\n    invoke:\n      command: demo.core.Handle\n    mapping: {{}}\n    delivery: at_least_once\n    on_failure: drop\n").unwrap();
    }
    let directory = scratch("pass-2-binding-source");
    std::fs::write(
        directory.join("system.yaml"),
        "format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("core.yaml"),
        format!("domain: demo.core\n{body}"),
    )
    .unwrap();
    std::fs::write(directory.join("wiring.yaml"), &wiring).unwrap();
    eprintln!("source witness: {}", directory.display());
    fixture(body, &wiring)
}

#[test]
fn binding_function_cannot_be_hidden_by_the_delivery_event_parameter() {
    let ir = binding_model(&["event"]);
    match synthesize(&ir) {
        Err(failure) => checked_failure(&ir, Target::Rust, &failure),
        Ok(synthesis) => compile_emitted(&scratch("pass-2-binding-event"), &synthesis, false),
    }
}

#[test]
fn a_later_binding_function_cannot_be_hidden_by_a_prior_input_local() {
    let ir = binding_model(&["alpha", "input"]);
    match synthesize(&ir) {
        Err(failure) => checked_failure(&ir, Target::Rust, &failure),
        Ok(synthesis) => compile_emitted(&scratch("pass-2-binding-input-later"), &synthesis, false),
    }
}

#[test]
fn a_first_binding_named_input_remains_compilable_on_both_targets() {
    let ir = binding_model(&["input"]);
    let directory = scratch("pass-2-binding-input-first");
    let rust = synthesize(&ir).expect("a let binding does not capture its own initializer");
    compile_emitted(&directory.join("generated/rust/demo"), &rust, false);
    let web = synthesize_for(&ir, Target::Web).expect("Web preserves the same legal binding scope");
    compile_emitted(&directory.join("generated/web/demo"), &web, true);
}

#[test]
fn error_encoder_names_in_separate_outcome_arms_remain_compilable() {
    let ir = fixture("events:\n  - name: demo.core.EncodeErrorDemoCoreBroken\nerrors:\n  - name: demo.core.Broken\ncommands:\n  - name: demo.core.Fire\n    outcomes:\n      - name: done\n        emits: [demo.core.EncodeErrorDemoCoreBroken]\n      - name: refused\n        external: the external operation is refused\n        error: demo.core.Broken\n", "components:\n  - component: worker\n    owns:\n      domains: [demo.core]\n    accepts:\n      commands: [demo.core.Fire]\n    publishes:\n      events: [demo.core.EncodeErrorDemoCoreBroken]\n    reached_by: network\n");
    compile_emitted(
        &scratch("pass-2-error-encoder-separate-arms"),
        &synthesize(&ir).expect("an encoder used by a different match arm is not captured"),
        false,
    );
}

fn primitive_map_compiles(key: &str) {
    let body = format!("types:\n  - name: demo.core.Record\n    kind: struct\n    fields:\n      - name: lookup\n        type: Map<{key}, String>\n");
    let ir = fixture(&body, "components: []\n");
    let directory = scratch(&format!("pass-2-map-{key}"));
    let source = directory.join("spec");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(
        source.join("system.yaml"),
        "format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\n",
    )
    .unwrap();
    std::fs::write(
        source.join("core.yaml"),
        format!("domain: demo.core\n{body}"),
    )
    .unwrap();
    compile_emitted(
        &directory.join("generated/rust/demo"),
        &synthesize(&ir).expect("primitive-key maps compile as Rust types"),
        false,
    );
    let synthesis =
        synthesize_for(&ir, Target::Web).expect("primitive-key maps retain their supported codec");
    compile_emitted(&directory.join("generated/web/demo"), &synthesis, true);
}

#[test]
fn boolean_map_decoder_path_is_borrowed_in_the_supported_codec() {
    primitive_map_compiles("Boolean");
}

#[test]
fn bytes_map_decoder_path_is_borrowed_in_the_supported_codec() {
    primitive_map_compiles("Bytes");
}

#[test]
fn a_string_map_key_keeps_its_compilable_codec() {
    let ir = fixture("types:\n  - name: demo.core.Record\n    kind: struct\n    fields:\n      - name: lookup\n        type: Map<String, String>\n", "components: []\n");
    let directory = scratch("pass-2-map-string");
    compile_emitted(
        &directory.join("generated/rust/demo"),
        &synthesize(&ir).unwrap(),
        false,
    );
    let synthesis = synthesize_for(&ir, Target::Web)
        .expect("String keys do not call a fallible parser with a path");
    compile_emitted(&directory.join("generated/web/demo"), &synthesis, true);
}

#[test]
fn an_unaccepted_outcome_binding_has_no_web_codec_local_scope() {
    let ir = fixture("events:\n  - name: demo.core.Out\ncommands:\n  - name: demo.core.Fire\n    outcomes:\n      - name: done\n        emits: [demo.core.Out]\n", "components:\n  - component: worker\n    owns:\n      domains: [demo.core]\n    publishes:\n      events: [demo.core.Out]\n");
    let directory = scratch("pass-2-unaccepted-out");
    compile_emitted(
        &directory.join("generated/rust/demo"),
        &synthesize(&ir).unwrap(),
        false,
    );
    let web =
        synthesize_for(&ir, Target::Web).expect("unaccepted commands retain their partial report");
    assert!(!web.target.as_ref().unwrap().refusals.is_empty());
    compile_emitted(&directory.join("generated/web/demo"), &web, true);
}

#[test]
fn optional_recursion_behind_nested_collections_keeps_its_size_break() {
    let ir = fixture("types:\n  - name: demo.core.Node\n    kind: struct\n    fields:\n      - name: children\n        type: Optional<Map<Integer, List<Optional<demo.core.Node>>>>\n", "components: []\n");
    let directory = scratch("pass-2-map-recursion");
    compile_emitted(
        &directory.join("generated/rust/demo"),
        &synthesize(&ir).expect("map and list break the size cycle"),
        false,
    );
    compile_emitted(
        &directory.join("generated/web/demo"),
        &synthesize_for(&ir, Target::Web).expect("nested recursive codecs must compile"),
        true,
    );
}
