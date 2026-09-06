//! `OpenAPI` accounting survives the actual file writer, reader and projection boundaries.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static CALL: AtomicUsize = AtomicUsize::new(0);

const SOURCE: &str =
    include_str!("../../../generate/ess-openapi/tests/fixtures/supported.openapi.yaml");
const LEGACY: &str =
    include_str!("../../../generate/ess-openapi/tests/fixtures/supported.interface-v1.json");
const PROJECTED: &str =
    include_str!("../../../generate/ess-openapi/tests/fixtures/supported.projected.yaml");

fn fixture(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let path = root.join(format!(
        "target/review-boundaries-7/cli-{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn ess(arguments: &[&str], input: &Path, out: Option<&Path>, format: &str) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(arguments)
        .arg(input)
        .args(["--format", format]);
    if let Some(out) = out {
        command.arg("--out").arg(out);
    }
    let output = command.output().unwrap();
    let record = input
        .parent()
        .unwrap()
        .join(format!("call-{}", CALL.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(record.with_extension("argv"), format!("{command:?}\n")).unwrap();
    std::fs::write(record.with_extension("stdout"), &output.stdout).unwrap();
    std::fs::write(record.with_extension("stderr"), &output.stderr).unwrap();
    std::fs::write(
        record.with_extension("status"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    output
}

#[test]
fn both_import_spellings_write_the_accounted_envelope_for_every_presentation() {
    let dir = fixture("write");
    let input = dir.join("source.yaml");
    let source = SOURCE.replace("type: number", "type: number\n          enum: [1, 2]");
    std::fs::write(&input, &source).unwrap();
    let mut bytes = Vec::new();
    for format in ["text", "json", "yaml"] {
        let out = dir.join(format!("import-{format}.json"));
        let mut streams = Vec::new();
        for arguments in [
            &["import", "openapi", "--path"][..],
            &["infra", "import", "openapi", "--path"][..],
        ] {
            let output = ess(arguments, &input, Some(&out), format);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let stored = std::fs::read_to_string(&out).unwrap();
            let value: serde_json::Value = serde_json::from_str(&stored).unwrap();
            assert_eq!(value["format"], "ess-openapi-import/1");
            assert_eq!(value["source"]["text"], source);
            let checked = ess_openapi::read_import(&stored).unwrap();
            assert_eq!(checked.accounting().coverage_gaps.len(), 1);
            if format != "text" {
                let terminal: serde_json::Value = serde_yaml::from_slice(&output.stdout).unwrap();
                assert_eq!(terminal["coverage_gaps"].as_array().unwrap().len(), 1);
            }
            bytes.push(stored);
            streams.push((output.stdout, output.stderr));
        }
        assert_eq!(streams[0], streams[1]);
    }
    assert!(bytes.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn legacy_projection_refuses_before_destination_mutation() {
    let dir = fixture("legacy");
    let input = dir.join("legacy.json");
    std::fs::write(&input, LEGACY).unwrap();
    refused_projection(&input, &dir, "accounting unavailable");
}

fn refused_projection(input: &Path, dir: &Path, reason: &str) {
    for format in ["text", "json", "yaml"] {
        for destination in ["existing", "absent", "stdout"] {
            let out = dir.join(format!("{format}-{destination}.yaml"));
            if destination == "existing" {
                std::fs::write(&out, "untouched\n").unwrap();
            }
            for arguments in [
                &["project", "openapi", "--ir"][..],
                &["generate", "project", "openapi", "--ir"][..],
            ] {
                let output = ess(
                    arguments,
                    input,
                    (destination != "stdout").then_some(out.as_path()),
                    format,
                );
                assert_eq!(output.status.code(), Some(1));
                assert!(
                    output.stdout.is_empty(),
                    "refused projection emitted artifact bytes"
                );
                assert!(
                    String::from_utf8_lossy(&output.stderr).contains(reason),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                if destination == "existing" {
                    assert_eq!(std::fs::read(&out).unwrap(), b"untouched\n");
                } else {
                    assert!(!out.exists());
                }
            }
        }
    }
}

#[test]
fn partial_projection_reports_the_durable_gap_and_preserves_destinations() {
    let dir = fixture("partial");
    let input = dir.join("import.json");
    let report =
        ess_openapi::import(&SOURCE.replace("type: number", "type: number\n          const: 1"))
            .unwrap();
    std::fs::write(&input, report.to_canonical_json()).unwrap();
    refused_projection(&input, &dir, "unpreserved semantics");
}

#[test]
fn unresolved_projection_reports_the_exact_reference_and_preserves_destinations() {
    let dir = fixture("unresolved");
    let input = dir.join("import.json");
    let report = ess_openapi::import(&SOURCE.replace(
        "#/components/schemas/Invoice",
        "#/components/schemas/Missing",
    ))
    .unwrap();
    std::fs::write(&input, report.to_canonical_json()).unwrap();
    refused_projection(&input, &dir, "unresolved local interface type `Missing`");
}

#[test]
fn complete_projection_preserves_supported_yaml_through_both_spellings() {
    let dir = fixture("complete");
    let input = dir.join("import.json");
    std::fs::write(
        &input,
        ess_openapi::import(SOURCE).unwrap().to_canonical_json(),
    )
    .unwrap();
    for arguments in [
        &["project", "openapi", "--ir"][..],
        &["generate", "project", "openapi", "--ir"][..],
    ] {
        let output = ess(arguments, &input, None, "yaml");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, PROJECTED.as_bytes());
        let out = dir.join("projection.yaml");
        let output = ess(arguments, &input, Some(&out), "json");
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(std::fs::read(&out).unwrap(), PROJECTED.as_bytes());
    }
}

#[test]
fn a_refused_import_keeps_existing_output_and_creates_no_new_file() {
    let dir = fixture("refused");
    let input = dir.join("source.yaml");
    std::fs::write(&input, SOURCE.replace("type: number", "type: array")).unwrap();
    for format in ["text", "json", "yaml"] {
        for existing in [false, true] {
            let out = dir.join(format!("output-{format}-{existing}.json"));
            if existing {
                std::fs::write(&out, "untouched\n").unwrap();
            }
            for arguments in [
                &["import", "openapi", "--path"][..],
                &["infra", "import", "openapi", "--path"][..],
            ] {
                let output = ess(arguments, &input, Some(&out), format);
                assert_eq!(output.status.code(), Some(1));
                assert!(String::from_utf8_lossy(&output.stdout).contains("/items"));
                if existing {
                    assert_eq!(std::fs::read(&out).unwrap(), b"untouched\n");
                } else {
                    assert!(!out.exists());
                }
            }
        }
    }
}
