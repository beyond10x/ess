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
