use cli_contract::*;
use serde_json::{json, Value};
use std::ffi::OsString;

#[derive(Default)]
struct Recorder {
    calls: Vec<(String, Context, Value)>,
    reply: Option<HandlerReply>,
}
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.calls.push((
            invocation.callable.to_owned(),
            invocation.context.clone(),
            invocation.input.clone(),
        ));
        self.reply
            .take()
            .unwrap_or_else(|| HandlerReply::Success(json!({"profile":"safe"})))
    }
}

#[derive(Default)]
struct SourceRecorder {
    calls: Vec<ProtectedSource>,
    error: Option<AcquireError>,
}
impl Sources for SourceRecorder {
    fn acquire(&mut self, source: ProtectedSource) -> Result<String, AcquireError> {
        self.calls.push(source);
        if let Some(error) = self.error {
            Err(error)
        } else {
            Ok("secret-canary".to_owned())
        }
    }
}

fn execute(args: &[&str], sources: &mut dyn Sources, handler: &mut dyn Handler) -> ProcessOutput {
    run(
        args.iter().map(OsString::from).collect(),
        sources,
        handler,
        None,
    )
}

fn safe(output: &ProcessOutput) {
    assert!(
        !output.stdout.contains("secret-canary"),
        "{}",
        output.stdout
    );
    assert!(
        !output.stderr.contains("secret-canary"),
        "{}",
        output.stderr
    );
}

#[test]
fn aliases_dispatch_once_and_globals_never_enter_payload() {
    let mut sources = SourceRecorder::default();
    let mut handler = Recorder::default();
    let output = execute(
        &[
            "demo",
            "credential",
            "set",
            "--profile",
            "p",
            "--secret-stdin",
            "--config",
            "settings.yaml",
            "--state-dir",
            "state",
            "--output",
            "json",
        ],
        &mut sources,
        &mut handler,
    );
    assert_eq!(output.exit_code, 0);
    assert_eq!(
        serde_json::from_str::<Value>(&output.stdout).unwrap(),
        json!({"ok":true,"result":{"profile":"safe"}})
    );
    assert_eq!(handler.calls.len(), 1);
    assert_eq!(handler.calls[0].0, "store");
    assert_eq!(handler.calls[0].1.config, Some("settings.yaml".into()));
    assert_eq!(handler.calls[0].1.state_dir, Some("state".into()));
    assert_eq!(
        handler.calls[0].2,
        json!({"profile":"p","secret":"secret-canary"})
    );
    assert_eq!(sources.calls, vec![ProtectedSource::Stdin]);
    safe(&output);
}

#[test]
fn all_protected_channels_are_exclusive_and_never_accept_secret_argv() {
    for (tail, expected) in [
        (
            vec!["--secret-file", "credential.txt"],
            ProtectedSource::File("credential.txt".into()),
        ),
        (vec!["--secret-stdin"], ProtectedSource::Stdin),
        (vec!["--secret-prompt"], ProtectedSource::HiddenTty),
    ] {
        let mut args = vec!["demo", "credential", "store", "--profile", "p"];
        args.extend(tail);
        let mut sources = SourceRecorder::default();
        let mut handler = Recorder::default();
        assert_eq!(execute(&args, &mut sources, &mut handler).exit_code, 0);
        assert_eq!(sources.calls, vec![expected]);
    }
    for tail in [
        vec![],
        vec!["--secret-file", "secret-canary", "--secret-stdin"],
        vec!["--secret-prompt", "--secret-stdin"],
        vec!["--secret-stdin=secret-canary"],
        vec!["--secret", "secret-canary"],
    ] {
        let mut args = vec![
            "demo",
            "--output=json",
            "credential",
            "store",
            "--profile",
            "p",
        ];
        args.extend(tail);
        let mut sources = SourceRecorder::default();
        let mut handler = Recorder::default();
        let output = execute(&args, &mut sources, &mut handler);
        assert_eq!(output.exit_code, 2);
        assert_eq!(
            serde_json::from_str::<Value>(&output.stderr).unwrap()["ok"],
            false
        );
        assert!(sources.calls.is_empty());
        assert!(handler.calls.is_empty());
        safe(&output);
    }
}

#[test]
fn typed_values_optional_omission_and_unit_payload_are_enforced() {
    let mut handler = Recorder::default();
    let mut sources = SourceRecorder::default();
    let args = [
        "demo",
        "typed",
        "7",
        "--enabled",
        "false",
        "--mode",
        "Safe",
        "--items",
        "[1,2]",
    ];
    assert_eq!(execute(&args, &mut sources, &mut handler).exit_code, 0);
    assert_eq!(
        handler.calls[0].2,
        json!({"count":7,"enabled":false,"mode":"Safe","items":[1,2]})
    );
    for (index, invalid) in [(2, "1.5"), (4, "yes"), (6, "unsafe"), (8, "[\"bad\"]")] {
        let mut invalid_args = args;
        invalid_args[index] = invalid;
        assert_eq!(
            execute(&invalid_args, &mut sources, &mut handler).exit_code,
            2
        );
    }
    assert_eq!(handler.calls.len(), 1);
    assert_eq!(
        execute(&["demo", "show"], &mut sources, &mut handler).exit_code,
        0
    );
    assert_eq!(handler.calls[1].2, json!({}));
}

#[test]
fn parser_refusals_follow_json_selection_without_echoing_values() {
    for args in [
        vec!["demo", "--output=json", "secret-canary"],
        vec!["demo", "show", "--secret-canary", "--output", "json"],
        vec!["demo", "--output=json", "show", "--config"],
        vec![
            "demo",
            "--output=json",
            "show",
            "--config=secret-canary",
            "--config=twice",
        ],
    ] {
        let output = execute(
            &args,
            &mut SourceRecorder::default(),
            &mut Recorder::default(),
        );
        assert_eq!(output.exit_code, 2);
        assert_eq!(
            serde_json::from_str::<Value>(&output.stderr).unwrap()["error"]["code"],
            "cli_parse"
        );
        safe(&output);
    }
}

#[test]
fn help_and_completions_are_executable_and_do_not_dispatch() {
    for args in [
        vec!["demo", "--help"],
        vec!["demo", "credential", "store", "--help"],
        vec!["demo", "completions", "bash"],
    ] {
        let mut handler = Recorder::default();
        let output = execute(&args, &mut SourceRecorder::default(), &mut handler);
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("secret-file") || output.stdout.contains("credential"));
        assert!(handler.calls.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn handler_results_errors_and_interruptions_follow_process_contract() {
    for (reply, exit, code) in [
        (HandlerReply::Success(json!({"profile":3})), 1, "cli_result"),
        (
            HandlerReply::Error {
                code: "unknown".to_owned(),
                data: json!({"secret":"secret-canary"}),
            },
            1,
            "cli_error",
        ),
        (HandlerReply::Unavailable, 1, "cli_handler_unavailable"),
        (HandlerReply::Interrupted, 130, "cli_interrupted"),
    ] {
        let mut handler = Recorder {
            reply: Some(reply),
            ..Recorder::default()
        };
        let output = execute(
            &["demo", "show", "--output=json"],
            &mut SourceRecorder::default(),
            &mut handler,
        );
        assert_eq!(output.exit_code, exit);
        assert_eq!(
            serde_json::from_str::<Value>(&output.stderr).unwrap()["error"]["code"],
            code
        );
        safe(&output);
    }
    for (data, expected) in [
        (json!({"reason":"denied"}), "store_failed"),
        (json!({"reason":3}), "cli_error"),
    ] {
        let mut handler = Recorder {
            reply: Some(HandlerReply::Error {
                code: "store_failed".to_owned(),
                data,
            }),
            ..Recorder::default()
        };
        let output = execute(
            &[
                "demo",
                "credential",
                "store",
                "--profile",
                "p",
                "--secret-stdin",
                "--output=json",
            ],
            &mut SourceRecorder::default(),
            &mut handler,
        );
        assert_eq!(output.exit_code, 1);
        assert_eq!(
            serde_json::from_str::<Value>(&output.stderr).unwrap()["error"]["code"],
            expected
        );
    }
}

#[test]
fn acquisition_failure_and_interruption_are_safe_and_do_not_dispatch() {
    for (error, exit) in [
        (AcquireError::Unavailable, 2),
        (AcquireError::Interrupted, 130),
    ] {
        let mut sources = SourceRecorder {
            error: Some(error),
            ..SourceRecorder::default()
        };
        let mut handler = Recorder::default();
        let output = execute(
            &[
                "demo",
                "credential",
                "store",
                "--profile",
                "p",
                "--secret-stdin",
                "--output=json",
            ],
            &mut sources,
            &mut handler,
        );
        assert_eq!(output.exit_code, exit);
        assert!(handler.calls.is_empty());
        safe(&output);
    }
}

#[test]
fn typed_application_usage_errors_preserve_the_declared_error_contract() {
    for (code, data, exit, expected_code) in [
        (
            "store_failed",
            json!({"reason":"invalid configuration"}),
            2,
            "store_failed",
        ),
        ("unknown", json!({"reason":"secret-canary"}), 1, "cli_error"),
        ("store_failed", json!({"reason":3}), 1, "cli_error"),
    ] {
        let mut handler = Recorder {
            reply: Some(HandlerReply::UsageError {
                code: code.to_owned(),
                data,
            }),
            ..Recorder::default()
        };
        let output = execute(
            &[
                "demo",
                "credential",
                "store",
                "--profile",
                "p",
                "--secret-stdin",
                "--output=json",
            ],
            &mut SourceRecorder::default(),
            &mut handler,
        );
        assert_eq!(output.exit_code, exit);
        assert_eq!(
            serde_json::from_str::<Value>(&output.stderr).unwrap()["error"]["code"],
            expected_code
        );
        assert!(output.stdout.is_empty());
        safe(&output);
    }
}

#[test]
fn integer_input_preserves_signed_boundaries_and_refuses_unsigned_overflow() {
    for text in [
        "-1",
        "9223372036854775807",
        "-9223372036854775808",
        "9007199254740993",
    ] {
        let mut handler = Recorder::default();
        let output = execute(
            &[
                "demo",
                "typed",
                text,
                "--enabled",
                "true",
                "--mode",
                "Safe",
                "--items",
                "[]",
            ],
            &mut SourceRecorder::default(),
            &mut handler,
        );
        assert_eq!(output.exit_code, 0, "rejected {text}");
        assert_eq!(
            handler.calls[0].2["count"],
            serde_json::from_str::<Value>(text).unwrap()
        );
    }
    for text in [
        "9223372036854775808",
        "18446744073709551615",
        "-9223372036854775809",
    ] {
        let mut handler = Recorder::default();
        let output = execute(
            &[
                "demo",
                "typed",
                text,
                "--enabled",
                "true",
                "--mode",
                "Safe",
                "--items",
                "[]",
            ],
            &mut SourceRecorder::default(),
            &mut handler,
        );
        assert_eq!(output.exit_code, 2, "accepted {text}");
        assert!(handler.calls.is_empty());
    }
}

#[derive(Default)]
struct NativeValidator {
    phases: Vec<String>,
    reject_result: bool,
}
impl DynamicValidator for NativeValidator {
    fn validate(
        &mut self,
        invocation: &Invocation<'_>,
        phase: DynamicPhase,
        value: &Value,
    ) -> Result<(), DynamicError> {
        assert_eq!(invocation.input["operation"], "lookup");
        assert_eq!(invocation.input["schema"], "lookup/v1");
        let valid = match phase {
            DynamicPhase::Input => {
                self.phases.push("input".to_owned());
                value == &json!({"id":7})
            }
            DynamicPhase::Result => {
                self.phases.push("result".to_owned());
                !self.reject_result
            }
            DynamicPhase::Error(code) => {
                self.phases.push(code);
                true
            }
        };
        if valid {
            Ok(())
        } else {
            Err(DynamicError::InvalidValue)
        }
    }
}

#[test]
fn dynamic_refusals_distinguish_input_resolution_and_interruption() {
    struct RefusingValidator(DynamicError);
    impl DynamicValidator for RefusingValidator {
        fn validate(
            &mut self,
            _: &Invocation<'_>,
            _: DynamicPhase,
            _: &Value,
        ) -> Result<(), DynamicError> {
            Err(self.0)
        }
    }
    for (error, exit, code) in [
        (DynamicError::InvalidValue, 2, "cli_dynamic_input"),
        (
            DynamicError::Unavailable,
            1,
            "cli_dynamic_validator_unavailable",
        ),
        (DynamicError::Interrupted, 130, "cli_interrupted"),
    ] {
        let mut handler = Recorder::default();
        let output = run(
            [
                "demo",
                "invoke",
                "--operation",
                "lookup",
                "--schema",
                "lookup/v1",
                "--input-json",
                "{\"id\":\"wrong type\"}",
                "--output=json",
            ]
            .into_iter()
            .map(OsString::from)
            .collect(),
            &mut SourceRecorder::default(),
            &mut handler,
            Some(&mut RefusingValidator(error)),
        );
        assert_eq!(output.exit_code, exit);
        assert!(output.stdout.is_empty());
        assert_eq!(
            serde_json::from_str::<Value>(&output.stderr).unwrap()["error"]["code"],
            code
        );
        assert!(handler.calls.is_empty());
    }
}

#[test]
fn dynamic_input_result_and_errors_require_the_native_validator() {
    let args = [
        "demo",
        "invoke",
        "--operation",
        "lookup",
        "--schema",
        "lookup/v1",
        "--input-json",
        "{\"id\":7}",
        "--output=json",
    ];
    let mut handler = Recorder::default();
    let mut sources = SourceRecorder::default();
    let output = execute(&args, &mut sources, &mut handler);
    assert_eq!(output.exit_code, 1);
    assert!(handler.calls.is_empty());
    assert!(output.stderr.contains("cli_dynamic_validator_unavailable"));
    let mut validator = NativeValidator::default();
    let output = run(
        args.iter().map(OsString::from).collect(),
        &mut sources,
        &mut handler,
        Some(&mut validator),
    );
    assert_eq!(output.exit_code, 0);
    assert_eq!(validator.phases, ["input", "result"]);
    validator.reject_result = true;
    let output = run(
        args.iter().map(OsString::from).collect(),
        &mut sources,
        &mut handler,
        Some(&mut validator),
    );
    assert_eq!(output.exit_code, 1);
    assert!(output.stderr.contains("cli_dynamic_result"));
    handler.reply = Some(HandlerReply::Error {
        code: "store_failed".to_owned(),
        data: json!({"reason":"denied"}),
    });
    let output = run(
        args.iter().map(OsString::from).collect(),
        &mut sources,
        &mut handler,
        Some(&mut validator),
    );
    assert_eq!(output.exit_code, 1);
    assert_eq!(validator.phases.last().unwrap(), "store_failed");
}

#[cfg(unix)]
#[test]
fn native_file_acquisition_checks_permissions_symlinks_size_and_utf8() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("source-fixture");
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join("secret.txt");
    std::fs::write(&file, "secret-canary").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        OsSources
            .acquire(ProtectedSource::File(file.clone()))
            .unwrap(),
        "secret-canary"
    );
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(OsSources
        .acquire(ProtectedSource::File(file.clone()))
        .is_err());
    assert!(OsSources
        .acquire(ProtectedSource::DocumentFile(file.clone()))
        .is_ok());
    let link = directory.join("symlink.txt");
    if !link.exists() {
        symlink(&file, &link).unwrap();
    }
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(OsSources.acquire(ProtectedSource::File(link)).is_err());
    std::fs::write(&file, [0xff]).unwrap();
    assert!(OsSources
        .acquire(ProtectedSource::File(file.clone()))
        .is_err());
    std::fs::write(&file, vec![b'a'; MAX_PROTECTED_BYTES + 1]).unwrap();
    assert!(OsSources.acquire(ProtectedSource::File(file)).is_err());
}
