//! Checked refusal must also precede mutation when input and output name the same file.

use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

#[test]
fn same_path_projection_refusal_preserves_the_unadmitted_input() {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/review-boundaries-7/adversary-pass-2")
        .join(format!("same-path-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let source = json!({
        "openapi": "3.1.0", "info": {"title": "pass-two", "version": "v1"},
        "paths": {}, "components": {"schemas": {"A": {
            "type": "array", "items": {"type": "number"}
        }}}
    })
    .to_string();
    let report = ess_openapi::import(&source).unwrap();
    let mut wire: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    wire["interface"]["types"]["A"]["items"]["extra~/"] = json!(null);
    for (encoding, bytes) in [
        ("json", wire.to_string()),
        ("yaml", serde_yaml::to_string(&wire).unwrap()),
    ] {
        for (spelling, arguments) in [
            ("flat", &["project", "openapi"][..]),
            ("area", &["generate", "project", "openapi"][..]),
        ] {
            for format in ["json", "yaml", "text"] {
                let record = scratch.join(format!("{encoding}-{spelling}-{format}"));
                let input = record.with_extension("input");
                std::fs::write(&input, &bytes).unwrap();
                let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
                command
                    .args(arguments)
                    .arg("--ir")
                    .arg(&input)
                    .arg("--out")
                    .arg(&input)
                    .args(["--format", format]);
                std::fs::write(record.with_extension("argv"), format!("{command:?}\n")).unwrap();
                let output = command.output().unwrap();
                std::fs::write(record.with_extension("stdout"), &output.stdout).unwrap();
                std::fs::write(record.with_extension("stderr"), &output.stderr).unwrap();
                std::fs::write(
                    record.with_extension("status"),
                    format!("{:?}\n", output.status.code()),
                )
                .unwrap();
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                assert!(String::from_utf8_lossy(&output.stderr)
                    .contains("/interface/types/A/items/extra~0~1"));
                assert_eq!(std::fs::read_to_string(&input).unwrap(), bytes);
            }
        }
    }
}
