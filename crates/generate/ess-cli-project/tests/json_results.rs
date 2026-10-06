//! beyond10x/ess#468: a CLI result field typed `Json` is written as JSON, not as JSON text, and the
//! result check refuses a reply outside the declared shape around it as `cli_result`.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProcessOutput, ProtectedSource, Sources,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};
use std::ffi::OsString;

const MODEL: &str =
    include_str!("../../../specify/ess-cli-contract/tests/fixtures/json-model.yaml");
const BINDING: &str =
    include_str!("../../../specify/ess-cli-contract/tests/fixtures/json-cli.yaml");

fn compiled() -> CompiledBinding {
    let specification = Specification::assemble(vec![(
        Source::new("model.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap();
    compile(&ir, &Binding::from_yaml(BINDING).unwrap()).unwrap()
}

/// Answers its one call with the reply it holds.
struct Reply(Option<Value>);
impl Handler for Reply {
    fn call(&mut self, _: &Invocation<'_>) -> HandlerReply {
        HandlerReply::Success(self.0.take().expect("one call"))
    }
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        unreachable!("describe reads no protected or document source")
    }
}

fn describe(output: &str, reply: &Value) -> ProcessOutput {
    runtime::run(
        compiled().plan(),
        [
            "json-demo",
            "describe",
            "--profile",
            "p",
            "--output",
            output,
        ]
        .map(OsString::from)
        .to_vec(),
        &mut NoSources,
        &mut Reply(Some(reply.clone())),
        None,
    )
}

#[test]
fn a_json_result_field_is_written_as_json_not_as_json_text() {
    let document = json!({"schema": {"type": "object", "required": ["id"]}, "rows": [1, null]});
    for payload in [
        document.clone(),
        json!([1, "two"]),
        json!("text"),
        json!(null),
    ] {
        let reply = json!({
            "payload": payload,
            "maybe": payload,
            "items": [payload, 2],
            "keyed": {"k": payload},
            "nested": {"document": payload},
        });
        let output = describe("json", &reply);
        assert_eq!(output.exit_code, 0, "{payload}: {output:?}");
        assert_eq!(output.stderr, "");
        assert_eq!(
            output.stdout,
            format!("{}\n", json!({"ok": true, "result": reply}))
        );
        let human = describe("human", &reply);
        assert_eq!(human.exit_code, 0, "{payload}: {human:?}");
        assert_eq!(human.stdout, format!("{reply}\n"));
    }
    let output = describe(
        "json",
        &json!({"payload": document, "items": [], "keyed": {}}),
    );
    let written: Value = serde_json::from_str(&output.stdout).unwrap();
    assert_eq!(written["result"]["payload"], document);
    assert!(!output.stdout.contains("\\\""), "{}", output.stdout);
}

#[test]
fn a_reply_outside_the_shape_around_json_is_cli_result() {
    for reply in [
        json!({"items": [], "keyed": {}}),
        json!({"payload": {}, "keyed": {}}),
        json!({"payload": {}, "items": {}, "keyed": {}}),
        json!({"payload": {}, "items": [], "keyed": []}),
        json!({"payload": {}, "items": [], "keyed": {}, "nested": {}}),
        json!({"payload": {}, "items": [], "keyed": {}, "extra": true}),
        json!("{\"payload\": {}, \"items\": [], \"keyed\": {}}"),
    ] {
        let output = describe("json", &reply);
        assert_eq!(output.exit_code, 1, "{reply}");
        assert_eq!(output.stdout, "", "{reply}");
        assert_eq!(
            serde_json::from_str::<Value>(&output.stderr).unwrap(),
            json!({"ok": false, "error": {"code": "cli_result", "data": {}}}),
            "{reply}"
        );
        let human = describe("human", &reply);
        assert_eq!(human.exit_code, 1, "{reply}");
        assert_eq!(human.stdout, "", "{reply}");
        assert_eq!(human.stderr, "cli_result\n", "{reply}");
    }
}

/// The Cargo target the generated packages of this crate's tests build into
/// (`projection.rs`, `workspace.rs`): registry dependencies compile once.
fn nested_target() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("ess-cli-project-target")
}

/// The generated package's own runtime and `wire.rs`, built offline on its own, carry the shape.
#[test]
fn the_generated_package_writes_json_results_and_refuses_the_rest() {
    let artifacts = ess_cli_project::project(&compiled());
    assert!(artifacts["binding.json"].contains("\"kind\": \"json\""));
    let temporary = tempfile::Builder::new()
        .prefix("ess-cli-json-")
        .tempdir()
        .unwrap();
    let directory = temporary.path();
    for (path, contents) in &artifacts {
        let path = directory.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    std::fs::create_dir_all(directory.join("tests")).unwrap();
    std::fs::write(
        directory.join("tests/process.rs"),
        include_str!("fixtures/json_process.rs"),
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO"))
        .args(["test", "--offline", "--manifest-path"])
        .arg(directory.join("Cargo.toml"))
        .env("CARGO_BUILD_JOBS", "2")
        .env("CARGO_TARGET_DIR", nested_target())
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_PROFILE_DEV_DEBUG", "0")
        .env("CARGO_PROFILE_TEST_DEBUG", "0")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    println!("{log}");
    assert!(
        output.status.success(),
        "generated fixture exited {}:\n{log}",
        output.status
    );
    assert!(
        log.contains("3 passed; 0 failed"),
        "fixture test count must select the authored cases"
    );
}
