//! Reachable CLI counterexamples for the F04 frozen first adversary pass.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{json, Value};

static CALL: AtomicUsize = AtomicUsize::new(0);

fn scratch(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/review-boundaries-7/adversary-pass-1");
    let path = root.join(format!("cli-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn ess(dir: &Path, args: &[&str], input: &Path, out: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command.args(args).arg(input).args(["--format", "json"]);
    if let Some(out) = out {
        command.arg("--out").arg(out);
    }
    let record = dir.join(format!("call-{}", CALL.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(record.with_extension("argv"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    std::fs::write(record.with_extension("stdout"), &output.stdout).unwrap();
    std::fs::write(record.with_extension("stderr"), &output.stderr).unwrap();
    std::fs::write(
        record.with_extension("status"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    output
}

fn original(dir: &Path) -> PathBuf {
    let path = dir.join("source.json");
    std::fs::write(
        &path,
        json!({
            "openapi": "3.1.0", "info": {"title": "adversary", "version": "v1"},
            "paths": {}, "components": {"schemas": {"A": {"type": "integer"}}}
        })
        .to_string(),
    )
    .unwrap();
    path
}

#[test]
fn cli_rejects_unknown_unit_fields_before_any_projection_output() {
    let dir = scratch("unit-field");
    let source = original(&dir);
    let stored = dir.join("import.json");
    let imported = ess(
        &dir,
        &["infra", "import", "openapi", "--path"],
        &source,
        Some(&stored),
    );
    assert!(imported.status.success());
    let mut wire: Value = serde_json::from_str(&std::fs::read_to_string(&stored).unwrap()).unwrap();
    wire["interface"]["types"]["A"]["unexpected_constraint"] = json!({"const": 7});
    std::fs::write(&stored, wire.to_string()).unwrap();
    let mut failures = Vec::new();
    for (spelling, args) in [
        ("flat", &["project", "openapi", "--ir"][..]),
        ("area", &["generate", "project", "openapi", "--ir"][..]),
    ] {
        for destination in ["existing", "absent", "stdout"] {
            let out = dir.join(format!("{spelling}-{destination}.yaml"));
            if destination == "existing" {
                std::fs::write(&out, b"retain existing output\n").unwrap();
            }
            let result = ess(
                &dir,
                args,
                &stored,
                (destination != "stdout").then_some(out.as_path()),
            );
            let preserved = match destination {
                "existing" => std::fs::read(&out).unwrap() == b"retain existing output\n",
                "absent" => !out.exists(),
                "stdout" => result.stdout.is_empty(),
                _ => unreachable!(),
            };
            println!("{spelling} {destination}: exit={:?}; stdout_bytes={}; destination_preserved={preserved}", result.status.code(), result.stdout.len());
            if result.status.code() != Some(1) || !preserved {
                failures.push(format!("{spelling} {destination}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "unknown wire fields bypassed checked CLI admission and wrote output: {failures:?}"
    );
}

#[test]
fn cli_rejects_duplicate_keys_and_tampered_accounting_before_output() {
    let dir = scratch("tamper-controls");
    let source = original(&dir);
    let stored = dir.join("import.json");
    let imported = ess(
        &dir,
        &["import", "openapi", "--path"],
        &source,
        Some(&stored),
    );
    assert!(imported.status.success());
    let original = std::fs::read_to_string(&stored).unwrap();
    let mut tampered: Value = serde_json::from_str(&original).unwrap();
    tampered["source"]["sha256"] = json!("0".repeat(64));
    let duplicate = original.replacen(
        "\"kind\": \"integer\"",
        "\"kind\": \"integer\", \"kind\": \"integer\"",
        1,
    );
    assert_ne!(duplicate, original);
    for (name, bytes) in [("digest", tampered.to_string()), ("duplicate", duplicate)] {
        let input = dir.join(format!("{name}.json"));
        std::fs::write(&input, bytes).unwrap();
        let out = dir.join(format!("{name}.yaml"));
        std::fs::write(&out, b"retain existing output\n").unwrap();
        let result = ess(
            &dir,
            &["generate", "project", "openapi", "--ir"],
            &input,
            Some(&out),
        );
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert_eq!(std::fs::read(&out).unwrap(), b"retain existing output\n");
    }
}
