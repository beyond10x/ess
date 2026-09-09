//! Final attack: execute admitted authored declarations at the process boundary.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Context, DynamicError, DynamicPhase, DynamicValidator, Handler,
    HandlerReply, Invocation, OsSources, OutputMode, ProcessOutput, ProtectedSource, Sources,
    MAX_PROTECTED_BYTES,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};
use std::ffi::OsString;

fn binding(fields: &str, arguments: &str) -> CompiledBinding {
    let model = format!(
        "format: ess/1\nsystem: attack\nversion: v1\ntypes:\n  - name: attack.Input\n    kind: struct\n    fields:\n{fields}"
    );
    let declaration = format!(
        "format: ess-cli/1\nbinary: attack\nabout: Final attack\nglobals: {{config: settings, state: storage, output: render}}\ncallables:\n  execute:\n    target: {{kind: local, owner: attack.cli, action: execute}}\n    input: attack.Input\n    result: String\n    errors: {{failed: String}}\ncommands:\n  - path: [group, execute]\n    aliases: [[run], [alternate, invoke]]\n    callable: execute\n    about: Execute\n    arguments:\n{arguments}"
    );
    resolve(&model, &declaration)
}

fn resolve(model: &str, declaration: &str) -> CompiledBinding {
    let specification = Specification::assemble(vec![(
        Source::new("adversarial-r2.yaml"),
        RawSpecFile::parse(model).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap();
    compile(&ir, &Binding::from_yaml(declaration).unwrap()).unwrap()
}

#[derive(Default)]
struct Recorder {
    calls: Vec<(Context, Value)>,
    reply: Option<HandlerReply>,
}

impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.calls
            .push((invocation.context.clone(), invocation.input.clone()));
        self.reply
            .take()
            .unwrap_or_else(|| HandlerReply::Success(json!("safe")))
    }
}

struct TextSource {
    value: String,
    calls: usize,
}

impl Sources for TextSource {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        self.calls += 1;
        Ok(self.value.clone())
    }
}

fn execute(
    compiled: &CompiledBinding,
    args: &[&str],
    sources: &mut dyn Sources,
    handler: &mut Recorder,
) -> ProcessOutput {
    runtime::run(
        compiled.plan(),
        args.iter().map(OsString::from).collect(),
        sources,
        handler,
        None,
    )
}

#[test]
fn exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers() {
    // The owning design accepts JSON integer tokens in i64 range and excludes only
    // decimal/exponent spellings. JSON's integer token -0 denotes the integer zero.
    // All inputs here travel from authored ESS/binding through ordinary argv.
    let mut observations = Vec::new();
    for (shape, text, expected) in [
        ("Integer", "-0", json!(0)),
        ("List<Integer>", "[-0]", json!([0])),
        ("Map<String, Integer>", "{\"zero\":-0}", json!({"zero":0})),
    ] {
        let compiled = binding(
            &format!("      - {{name: value, type: '{shape}'}}\n"),
            "      - {field: value, source: {kind: option, long: value}}\n",
        );
        let mut handler = Recorder::default();
        let mut sources = TextSource {
            value: String::new(),
            calls: 0,
        };
        let output = execute(
            &compiled,
            &["attack", "run", "--value", text, "--render=json"],
            &mut sources,
            &mut handler,
        );
        observations.push((shape, output, handler.calls, expected));
        assert_eq!(sources.calls, 0);
    }
    for (shape, output, calls, expected) in &observations {
        println!("{shape}: output={output:?}, dispatches={}", calls.len());
        if output.exit_code == 0 {
            assert_eq!(calls[0].1["value"], *expected);
        }
    }
    assert!(
        observations
            .iter()
            .all(|(_, output, calls, _)| output.exit_code == 0 && calls.len() == 1),
        "the documented exact integer token -0 must be admitted at each typed argv position"
    );
}

#[test]
fn optional_protected_omission_and_utf8_byte_limit_hold_at_the_handler_seam() {
    let compiled = binding(
        "      - {name: secret, type: 'Optional<String>'}\n",
        "      - {field: secret, source: {kind: protected, file: secret-file, stdin: secret-stdin, hidden_tty: secret-prompt}}\n",
    );
    let mut handler = Recorder::default();
    let mut sources = TextSource {
        value: "é".repeat(MAX_PROTECTED_BYTES / 2),
        calls: 0,
    };
    assert_eq!(
        execute(&compiled, &["attack", "run"], &mut sources, &mut handler).exit_code,
        0
    );
    assert_eq!(sources.calls, 0);
    assert_eq!(handler.calls[0].1, json!({}));
    let args = ["attack", "run", "--secret-stdin", "--render=json"];
    let accepted = execute(&compiled, &args, &mut sources, &mut handler);
    assert_eq!(accepted.exit_code, 0);
    assert_eq!(
        handler.calls[1].1["secret"].as_str().unwrap().len(),
        MAX_PROTECTED_BYTES
    );
    sources.value.push('é');
    let refused = execute(&compiled, &args, &mut sources, &mut handler);
    assert_eq!(refused.exit_code, 2);
    assert_eq!(handler.calls.len(), 2);
    assert_eq!(sources.calls, 2);
    assert_eq!(
        serde_json::from_str::<Value>(&refused.stderr).unwrap(),
        json!({"ok":false,"error":{"code":"cli_source","data":{}}})
    );
    assert!(accepted.stderr.is_empty());
    assert!(!accepted.stdout.contains('é'));
    assert!(refused.stdout.is_empty());
}

#[test]
fn document_inline_and_acquired_text_use_the_same_utf8_byte_bound() {
    let compiled = binding(
        "      - {name: payload, type: String}\n",
        "      - {field: payload, source: {kind: document, inline: document, file: document-file, stdin: document-stdin}}\n",
    );
    for via_stdin in [false, true] {
        for (extra, expected_exit) in [(0, 0), (1, 2)] {
            let text = "é".repeat(MAX_PROTECTED_BYTES / 2 + extra);
            let mut sources = TextSource {
                value: text.clone(),
                calls: 0,
            };
            let mut handler = Recorder::default();
            let args = if via_stdin {
                vec!["attack", "run", "--document-stdin", "--render=json"]
            } else {
                vec!["attack", "run", "--document", &text, "--render=json"]
            };
            let output = execute(&compiled, &args, &mut sources, &mut handler);
            assert_eq!(output.exit_code, expected_exit);
            assert_eq!(sources.calls, usize::from(via_stdin));
            assert_eq!(handler.calls.len(), usize::from(expected_exit == 0));
            assert!(!output.stdout.contains('é'));
            assert!(!output.stderr.contains('é'));
        }
    }
}

#[cfg(unix)]
#[test]
fn native_regular_document_symlinks_and_protected_parent_symlinks_are_admitted() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let directory = tempfile::tempdir().unwrap();
    let actual = directory.path().join("actual");
    std::fs::create_dir(&actual).unwrap();
    let file = actual.join("value");
    std::fs::write(&file, "source-canary").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    let parent_alias = directory.path().join("parent-alias");
    symlink(&actual, &parent_alias).unwrap();
    assert_eq!(
        OsSources.acquire(ProtectedSource::File(parent_alias.join("value"))),
        Ok("source-canary".to_owned())
    );
    let final_alias = directory.path().join("document-alias");
    symlink(&file, &final_alias).unwrap();
    assert_eq!(
        OsSources.acquire(ProtectedSource::DocumentFile(final_alias.clone())),
        Ok("source-canary".to_owned())
    );
    assert_eq!(
        OsSources.acquire(ProtectedSource::File(final_alias)),
        Err(AcquireError::Unavailable)
    );
    for bytes in [vec![b'x'; MAX_PROTECTED_BYTES + 1], vec![0xff]] {
        std::fs::write(&file, bytes).unwrap();
        for source in [
            ProtectedSource::File(file.clone()),
            ProtectedSource::DocumentFile(file.clone()),
        ] {
            assert_eq!(OsSources.acquire(source), Err(AcquireError::Unavailable));
        }
    }
}

#[test]
fn aliases_preserve_custom_globals_and_double_dash_stops_output_selection() {
    let compiled = binding(
        "      - {name: value, type: String}\n",
        "      - {field: value, source: {kind: positional, index: 1}}\n",
    );
    for path in [
        vec!["group", "execute"],
        vec!["run"],
        vec!["alternate", "invoke"],
    ] {
        let mut args = vec!["attack", "--settings", "config.yaml"];
        args.extend(path);
        args.extend([
            "--storage",
            "state",
            "--render=json",
            "--",
            "--render=human",
        ]);
        let mut handler = Recorder::default();
        let output = execute(&compiled, &args, &mut OsSources, &mut handler);
        assert_eq!(output.exit_code, 0);
        assert_eq!(handler.calls.len(), 1);
        assert_eq!(handler.calls[0].1, json!({"value":"--render=human"}));
        assert_eq!(handler.calls[0].0.output, OutputMode::Json);
        assert_eq!(handler.calls[0].0.config, Some("config.yaml".into()));
        assert_eq!(handler.calls[0].0.state_dir, Some("state".into()));
    }
    let mut handler = Recorder::default();
    let output = execute(
        &compiled,
        &["attack", "run", "--", "--render=json", "unwanted-canary"],
        &mut OsSources,
        &mut handler,
    );
    assert_eq!(output.exit_code, 2);
    assert_eq!(output.stderr, "cli_parse\n");
    assert!(handler.calls.is_empty());
}

#[cfg(unix)]
#[test]
fn non_utf8_argv_is_refused_in_selected_json_mode_without_dispatch() {
    use std::os::unix::ffi::OsStringExt;
    let compiled = binding(
        "      - {name: value, type: String}\n",
        "      - {field: value, source: {kind: option, long: value}}\n",
    );
    let mut handler = Recorder::default();
    let output = runtime::run(
        compiled.plan(),
        vec![
            "attack".into(),
            "run".into(),
            "--value".into(),
            OsString::from_vec(vec![0xff]),
            "--render=json".into(),
        ],
        &mut OsSources,
        &mut handler,
        None,
    );
    assert_eq!(output.exit_code, 2);
    assert_eq!(
        serde_json::from_str::<Value>(&output.stderr).unwrap(),
        json!({"ok":false,"error":{"code":"cli_parse","data":{}}})
    );
    assert!(output.stdout.is_empty());
    assert!(handler.calls.is_empty());
}

struct RejectReply {
    error: DynamicError,
    phases: Vec<&'static str>,
}
impl DynamicValidator for RejectReply {
    fn validate(
        &mut self,
        _: &Invocation<'_>,
        phase: DynamicPhase,
        _: &Value,
    ) -> Result<(), DynamicError> {
        match phase {
            DynamicPhase::Input => {
                self.phases.push("input");
                Ok(())
            }
            DynamicPhase::Result => {
                self.phases.push("result");
                Err(self.error)
            }
            DynamicPhase::Error(code) => {
                assert_eq!(code, "failed");
                self.phases.push("error");
                Err(self.error)
            }
        }
    }
}
#[test]
fn dynamic_result_and_both_error_replies_preserve_finite_failure_policy() {
    let model = "format: ess/1\nsystem: attack\nversion: v1\ntypes:\n  - name: attack.Input\n    kind: struct\n    fields:\n      - {name: operation, type: String}\n      - {name: schema, type: String}\n      - {name: payload, type: String}\n";
    let declaration = "format: ess-cli/1\nbinary: attack\nabout: Dynamic attack\nglobals: {config: settings, state: storage, output: render}\ncallables:\n  execute:\n    target: {kind: dynamic, owner: attack.cli, operation_field: operation, schema_field: schema, payload_field: payload}\n    input: attack.Input\n    result: String\n    errors: {failed: String}\ncommands:\n  - path: [run]\n    callable: execute\n    about: Run\n    arguments:\n      - {field: operation, source: {kind: option, long: operation}}\n      - {field: schema, source: {kind: option, long: schema}}\n      - {field: payload, source: {kind: document, inline: document, file: document-file, stdin: document-stdin}}\n";
    let compiled = resolve(model, declaration);
    for reply_kind in ["result", "error", "usage"] {
        for (error, expected_exit, expected_code) in [
            (
                DynamicError::InvalidValue,
                1,
                if reply_kind == "result" {
                    "cli_dynamic_result"
                } else {
                    "cli_dynamic_error"
                },
            ),
            (
                DynamicError::Unavailable,
                1,
                "cli_dynamic_validator_unavailable",
            ),
            (DynamicError::Interrupted, 130, "cli_interrupted"),
        ] {
            let mut handler = Recorder {
                reply: Some(match reply_kind {
                    "result" => HandlerReply::Success(json!("native-canary")),
                    "error" => HandlerReply::Error {
                        code: "failed".to_owned(),
                        data: json!("native-canary"),
                    },
                    _ => HandlerReply::UsageError {
                        code: "failed".to_owned(),
                        data: json!("native-canary"),
                    },
                }),
                ..Recorder::default()
            };
            let mut validator = RejectReply {
                error,
                phases: Vec::new(),
            };
            let output = runtime::run(
                compiled.plan(),
                [
                    "attack",
                    "run",
                    "--operation",
                    "op",
                    "--schema",
                    "op/v1",
                    "--document",
                    "{}",
                    "--render=json",
                ]
                .into_iter()
                .map(OsString::from)
                .collect(),
                &mut OsSources,
                &mut handler,
                Some(&mut validator),
            );
            assert_eq!(output.exit_code, expected_exit);
            assert_eq!(handler.calls.len(), 1);
            assert_eq!(
                validator.phases,
                [
                    "input",
                    if reply_kind == "result" {
                        "result"
                    } else {
                        "error"
                    }
                ]
            );
            assert!(output.stdout.is_empty());
            assert_eq!(
                serde_json::from_str::<Value>(&output.stderr).unwrap(),
                json!({"ok":false,"error":{"code":expected_code,"data":{}}})
            );
        }
    }
}
