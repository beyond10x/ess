//! Offline process-boundary cases for malformed synthetic Secret responses.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use serde_json::{json, Value};

const SENTINEL: &str = "SYNTHETIC-MALFORMED-SECRET-SENTINEL";
const PREVIOUS: &[u8] = b"previous sanitized observation";

fn fixture_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("secret-boundary-{}", std::process::id()));
        let bin = root.join("bin");
        std::fs::create_dir_all(&bin).expect("fixture directory");
        let helper = bin.join(format!("kubectl{}", std::env::consts::EXE_SUFFIX));
        let compiled = Command::new("rustc")
            .args(["--edition=2021", "-Dwarnings"])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_command.rs"))
            .arg("-o")
            .arg(&helper)
            .output()
            .expect("compile Rust fixture");
        assert!(
            compiled.status.success(),
            "fixture compilation: {compiled:?}"
        );
        std::fs::copy(
            &helper,
            bin.join(format!("date{}", std::env::consts::EXE_SUFFIX)),
        )
        .expect("fixed clock fixture");
        root
    })
}

fn scan(response: &str, preserve_existing: bool) -> (Output, PathBuf) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = fixture_root();
    let destination = root.join(format!(
        "observation-{}.json",
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    if preserve_existing {
        std::fs::write(&destination, PREVIOUS).expect("existing sanitized observation");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_ess-kubernetes"))
        .args(["scan", "--context", "synthetic-context", "--out"])
        .arg(&destination)
        .env("PATH", root.join("bin"))
        .env("ESS_TEST_SECRET_RESPONSE", response)
        .output()
        .expect("scan against synthetic subprocesses");
    (output, destination)
}

fn malformed_responses() -> Vec<(String, Value)> {
    let shapes = [
        ("string", json!(SENTINEL)),
        ("number", json!(123)),
        ("boolean", json!(false)),
        ("null", Value::Null),
        ("array", json!([SENTINEL])),
        ("object", json!({"nested": [SENTINEL]})),
    ];
    let mut cases = vec![("missing-items".to_owned(), json!({"unexpected": SENTINEL}))];
    for (shape, value) in &shapes {
        cases.push((format!("list-{shape}"), value.clone()));
        if !value.is_array() {
            cases.push((format!("items-{shape}"), json!({"items": value})));
        }
        if !value.is_object() {
            cases.push((format!("item-{shape}"), json!({"items": [value]})));
            for field in ["data", "stringData", "metadata"] {
                cases.push((
                    format!("{field}-{shape}"),
                    json!({"items": [{field: value}]}),
                ));
            }
            cases.push((
                format!("annotations-{shape}"),
                json!({"items": [{"metadata": {"annotations": value}}]}),
            ));
        }
        if !value.is_string() {
            for field in ["data", "stringData"] {
                cases.push((
                    format!("{field}-entry-{shape}"),
                    json!({"items": [{field: {"synthetic-key": value}}]}),
                ));
            }
            for annotation in ["kept", "kubectl.kubernetes.io/last-applied-configuration"] {
                cases.push((
                    format!("annotation-entry-{annotation}-{shape}"),
                    json!({"items": [{"metadata": {"annotations": {annotation: value}}}]}),
                ));
            }
        }
    }
    cases.push((
        "valid-item-before-malformed-item".to_owned(),
        json!({"items": [{"data": {"token": SENTINEL}}, {"stringData": [SENTINEL]}]}),
    ));
    cases
}

#[test]
fn malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values() {
    let mut failures = Vec::new();
    for (label, response) in malformed_responses() {
        for existing in [false, true] {
            let (output, destination) = scan(&response.to_string(), existing);
            if output.status.success() {
                failures.push(format!(
                    "{label}, existing={existing}: accepted malformed response"
                ));
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stdout.contains(SENTINEL) || stderr.contains(SENTINEL) {
                failures.push(format!(
                    "{label}, existing={existing}: diagnostic leaked sentinel"
                ));
            }
            let contents = std::fs::read(destination).ok();
            if existing && contents.as_deref() != Some(PREVIOUS) {
                failures.push(format!("{label}: replaced prior observation"));
            } else if !existing && contents.is_some() {
                failures.push(format!("{label}: created an observation on refusal"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn valid_secret_observation_bytes_remain_compatible() {
    let response = json!({"items": [{
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {"name": "synthetic", "namespace": "default", "annotations": {
            "kept": "visible",
            "kubectl.kubernetes.io/last-applied-configuration": "SYNTHETIC-LAST-APPLIED"
        }},
        "data": {"password": "SYNTHETIC-BASE64-VALUE"},
        "stringData": {"token": "SYNTHETIC-STRING-VALUE", "unicode": "λ-synthetic"},
        "type": "Opaque"
    }]});
    let (output, destination) = scan(&response.to_string(), true);
    assert!(output.status.success(), "{output:?}");
    let bytes = std::fs::read(destination).expect("sanitized observation");
    // All envelope fields, kind ordering, whitespace, digest and UTF-8 byte lengths are frozen.
    let expected = include_str!("fixtures/valid-observation.json")
        .replace("@SCOUT_VERSION@", env!("CARGO_PKG_VERSION"));
    assert_eq!(bytes, expected.trim_end().as_bytes());
    let text = String::from_utf8(bytes).expect("UTF-8 observation");
    for value in [
        "SYNTHETIC-BASE64-VALUE",
        "SYNTHETIC-STRING-VALUE",
        "λ-synthetic",
        "SYNTHETIC-LAST-APPLIED",
    ] {
        assert!(
            !text.contains(value),
            "synthetic value survived sanitization"
        );
    }
}

#[test]
fn absent_optional_secret_fields_and_empty_maps_remain_allowed() {
    let response = json!({"items": [
        {},
        {"metadata": {}},
        {"metadata": {"annotations": {}}},
        {"data": {}, "stringData": {}},
        {"data": {"empty": ""}}
    ]});
    let (output, destination) = scan(&response.to_string(), false);
    assert!(output.status.success(), "{output:?}");
    let observation: Value =
        serde_json::from_slice(&std::fs::read(destination).expect("observation"))
            .expect("JSON observation");
    let items = observation["kinds"]["secrets"]["items"]
        .as_array()
        .expect("items");
    assert_eq!(
        &items[..4],
        &response["items"].as_array().expect("source items")[..4]
    );
    assert_eq!(
        items[4]["data"]["empty"],
        json!({
            "length": 0,
            "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        })
    );
}

#[test]
fn failed_secret_subprocess_diagnostics_do_not_echo_secret_values() {
    let root = fixture_root();
    let destination = root.join("failed-secret-observation.json");
    std::fs::write(&destination, PREVIOUS).expect("existing sanitized observation");
    let output = Command::new(env!("CARGO_BIN_EXE_ess-kubernetes"))
        .args(["scan", "--context", "synthetic-context", "--out"])
        .arg(&destination)
        .env("PATH", root.join("bin"))
        .env("ESS_TEST_SECRET_FAILURE", SENTINEL)
        .output()
        .expect("scan against synthetic failing subprocess");
    assert!(
        !output.status.success(),
        "subprocess failure must refuse scan"
    );
    assert_eq!(
        std::fs::read(destination).expect("preserved observation"),
        PREVIOUS
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains(SENTINEL)
            && !String::from_utf8_lossy(&output.stderr).contains(SENTINEL),
        "the failed Secret subprocess diagnostic leaked the synthetic sentinel"
    );
    assert!(
        output.stdout.is_empty(),
        "a failed scan must not print a response"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("kubectl get resources failed"));
    assert!(stderr.contains('1'), "exit status must remain visible");
}

#[test]
fn every_kubectl_caller_uses_value_free_failure_diagnostics() {
    const CONTEXT: &str = "SYNTHETIC-UNTRUSTED-CONTEXT";
    let root = fixture_root();
    let operations = ["contexts", "current-context"]
        .into_iter()
        .chain(ess_kubernetes::KINDS.iter().copied());
    for operation in operations {
        for invalid_utf8 in [false, true] {
            for explicit_context in [false, true] {
                let destination = root.join(format!(
                    "failure-{operation}-{invalid_utf8}-{explicit_context}.json"
                ));
                let call_log = destination.with_extension("calls");
                std::fs::write(&destination, PREVIOUS).expect("previous observation");
                let mut command = Command::new(env!("CARGO_BIN_EXE_ess-kubernetes"));
                if operation == "contexts" {
                    command.arg("contexts");
                } else {
                    command.args(["scan", "--out"]).arg(&destination);
                    if explicit_context && operation != "current-context" {
                        command.args(["--context", CONTEXT]);
                    }
                }
                command
                    .env("PATH", root.join("bin"))
                    .env("ESS_TEST_CONTEXT", CONTEXT)
                    .env("ESS_TEST_CALL_LOG", &call_log)
                    .env("ESS_TEST_SECRET_RESPONSE", "{\"items\":[]}")
                    .env("ESS_TEST_FAILURE_OPERATION", operation)
                    .env("ESS_TEST_FAILURE_DIAGNOSTIC", SENTINEL);
                if invalid_utf8 {
                    command.env("ESS_TEST_INVALID_UTF8_STDERR", "1");
                }
                let output = command.output().expect("synthetic subprocess failure");
                assert!(!output.status.success(), "{operation} must refuse");
                assert!(
                    output.stdout.is_empty(),
                    "failed response must be discarded"
                );
                assert_eq!(
                    std::fs::read(&destination).expect("prior observation"),
                    PREVIOUS
                );
                let stderr = String::from_utf8(output.stderr).expect("value-free UTF-8 diagnostic");
                assert!(
                    !stderr.contains(SENTINEL),
                    "{operation} leaked process output"
                );
                assert!(
                    !stderr.contains(CONTEXT),
                    "{operation} leaked context argument"
                );
                let label = match operation {
                    "contexts" => "list contexts",
                    "current-context" => "read current context",
                    _ => "get resources",
                };
                assert!(
                    stderr.contains(&format!("kubectl {label} failed")),
                    "{stderr}"
                );
                assert!(stderr.contains('1'), "exit status must remain visible");
                let calls = std::fs::read_to_string(call_log).expect("recorded operations");
                if ess_kubernetes::KINDS.contains(&operation) {
                    assert!(
                        calls.ends_with(&format!(
                            "{operation}:all_namespaces=true\n{operation}:all_namespaces=false\n"
                        )),
                        "both resource attempts must be preserved: {calls}"
                    );
                } else {
                    assert_eq!(calls, format!("{operation}:all_namespaces=false\n"));
                }
            }
        }
    }
}

#[test]
fn successful_resource_retry_preserves_context_order_and_observation_bytes() {
    let root = fixture_root();
    let (baseline, baseline_path) = scan("{\"items\":[]}", false);
    assert!(baseline.status.success());
    let baseline_bytes = std::fs::read(baseline_path).expect("baseline observation");
    for explicit_context in [false, true] {
        let destination = root.join(format!("successful-retry-{explicit_context}.json"));
        let call_log = destination.with_extension("calls");
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess-kubernetes"));
        command.args(["scan", "--out"]).arg(&destination);
        if explicit_context {
            command.args(["--context", "synthetic-context"]);
        }
        let output = command
            .env("PATH", root.join("bin"))
            .env("ESS_TEST_CALL_LOG", &call_log)
            .env("ESS_TEST_SECRET_RESPONSE", "{\"items\":[]}")
            .env("ESS_TEST_FAILURE_OPERATION", "all-resources")
            .env("ESS_TEST_FAILURE_DIAGNOSTIC", SENTINEL)
            .env("ESS_TEST_FAILURE_FIRST_ATTEMPT_ONLY", "1")
            .env("ESS_TEST_INVALID_UTF8_STDERR", "1")
            .output()
            .expect("successful retry");
        assert!(
            output.status.success(),
            "successful fallback must complete scan"
        );
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains(SENTINEL));
        assert_eq!(
            std::fs::read(destination).expect("retry observation"),
            baseline_bytes
        );
        let mut expected = if explicit_context {
            String::new()
        } else {
            "current-context:all_namespaces=false\n".to_owned()
        };
        for kind in ess_kubernetes::KINDS {
            use std::fmt::Write as _;
            write!(
                expected,
                "{kind}:all_namespaces=true\n{kind}:all_namespaces=false\n"
            )
            .expect("render expected invocation");
        }
        assert_eq!(
            std::fs::read_to_string(call_log).expect("recorded retry order"),
            expected
        );
    }
}

#[test]
fn successful_context_listing_preserves_output_bytes() {
    let output = Command::new(env!("CARGO_BIN_EXE_ess-kubernetes"))
        .arg("contexts")
        .env("PATH", fixture_root().join("bin"))
        .output()
        .expect("synthetic context listing");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"synthetic-context\nsecond-context\n");
    assert!(output.stderr.is_empty());
}

fn adversary_failure(operation: &str, mode: &str, preserve_existing: bool) -> (Output, PathBuf) {
    let root = fixture_root();
    let destination = root.join(format!("adversary-{mode}-{operation}.json"));
    if preserve_existing {
        std::fs::write(&destination, PREVIOUS).expect("previous sanitized observation");
    }
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess-kubernetes"));
    if operation == "contexts" {
        command.arg("contexts");
    } else {
        command.args(["scan", "--out"]).arg(&destination);
        if operation != "current-context" {
            command.args(["--context", "synthetic-context"]);
        }
    }
    let output = command
        .env("PATH", root.join("bin"))
        .env("ESS_TEST_SECRET_RESPONSE", "{\"items\":[]}")
        .env("ESS_TEST_FAILURE_OPERATION", operation)
        .env("ESS_TEST_FAILURE_DIAGNOSTIC", SENTINEL)
        .env("ESS_TEST_ADVERSARY_FAILURE_MODE", mode)
        .env("ESS_TEST_CALL_LOG", destination.with_extension("calls"))
        .output()
        .expect("run synthetic adversarial subprocess");
    (output, destination)
}

#[test]
#[cfg(target_os = "linux")]
fn signal_terminated_kubectl_discards_both_streams_before_refusing() {
    for operation in ["contexts", "current-context", "secrets"] {
        let (output, destination) = adversary_failure(operation, "signal", true);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("safe diagnostic UTF-8");
        assert!(!stderr.contains(SENTINEL), "signal path leaked: {stderr}");
        assert!(
            stderr.contains("signal: 15 (SIGTERM)"),
            "termination must remain actionable: {stderr}"
        );
        assert_eq!(
            std::fs::read(&destination).expect("preserved output"),
            PREVIOUS
        );
        let calls = std::fs::read_to_string(destination.with_extension("calls"))
            .expect("recorded signal invocations");
        if operation == "secrets" {
            assert!(calls.ends_with("secrets:all_namespaces=true\nsecrets:all_namespaces=false\n"));
        } else {
            assert_eq!(calls, format!("{operation}:all_namespaces=false\n"));
        }
    }
}

#[test]
fn retry_failure_reports_the_final_exit_status_without_child_values() {
    let (output, destination) = adversary_failure("secrets", "statuses", true);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("safe diagnostic UTF-8");
    assert!(!stderr.contains(SENTINEL));
    assert!(stderr.ends_with("error: kubectl get resources failed: exit status: 254\n"));
    assert!(!stderr.contains("exit status: 23"));
    assert_eq!(
        std::fs::read(&destination).expect("preserved output"),
        PREVIOUS
    );
    let calls = std::fs::read_to_string(destination.with_extension("calls"))
        .expect("recorded retry invocations");
    assert!(calls.ends_with("secrets:all_namespaces=true\nsecrets:all_namespaces=false\n"));
}

#[test]
fn failed_payloads_larger_than_pipe_capacity_are_not_partially_reported_or_written() {
    for operation in ["contexts", "secrets"] {
        let (output, destination) = adversary_failure(operation, "large", false);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("safe diagnostic UTF-8");
        assert!(!stderr.contains(SENTINEL), "large response leaked");
        assert!(stderr.len() < 4096, "diagnostic grew with child output");
        assert!(stderr.ends_with("failed: exit status: 73\n"));
        assert!(
            !destination.exists(),
            "failed response created an observation"
        );
    }
}

#[test]
fn invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output() {
    let responses = [
        format!("{{\"items\":[{{\"data\":{{\"token\":\"{SENTINEL}\"}}}}]"),
        format!("{{\"items\":[{{\"stringData\":{{\"token\":\"{SENTINEL}\\q\"}}}}]}}"),
        format!("{{\"items\":[{{\"data\":{{\"token\":\"{SENTINEL}\"}}}}]}} trailing"),
    ];
    for response in responses {
        let (output, destination) = scan(&response, true);
        assert!(!output.status.success(), "invalid JSON must refuse scan");
        assert_eq!(
            std::fs::read(destination).expect("preserved observation"),
            PREVIOUS
        );
        assert!(
            !String::from_utf8_lossy(&output.stdout).contains(SENTINEL)
                && !String::from_utf8_lossy(&output.stderr).contains(SENTINEL),
            "invalid JSON diagnostic leaked the synthetic sentinel"
        );
    }
}

#[test]
fn malformed_late_annotations_with_sentinel_keys_refuse_before_any_write() {
    let response = json!({"items": [
        {"data": {"first": "synthetic-first-secret"}},
        {"stringData": {"second": "synthetic-second-secret"}, "metadata": {
            "annotations": {
                "kubectl.kubernetes.io/last-applied-configuration": SENTINEL,
                SENTINEL: {"nested": [SENTINEL]}
            }
        }}
    ]});
    for existing in [false, true] {
        let (output, destination) = scan(&response.to_string(), existing);
        assert!(!output.status.success(), "malformed annotation must refuse");
        let contents = std::fs::read(destination).ok();
        assert_eq!(contents.as_deref(), existing.then_some(PREVIOUS));
        assert!(
            !String::from_utf8_lossy(&output.stdout).contains(SENTINEL)
                && !String::from_utf8_lossy(&output.stderr).contains(SENTINEL),
            "malformed annotation diagnostic leaked a synthetic key or value"
        );
    }
}
