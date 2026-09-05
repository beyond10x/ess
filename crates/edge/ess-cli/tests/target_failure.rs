//! Whole-target failures must precede every output write, through both CLI spellings.

use std::path::Path;
use std::process::Command;

#[test]
fn fatal_synthesis_preserves_destinations_and_has_a_typed_envelope() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let fixture = root.join(format!(
        "target/review-boundaries-5/cli-failure-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(fixture.join("spec")).unwrap();
    std::fs::write(
        fixture.join("spec/system.yaml"),
        "format: ess/1\nsystem: demo\nversion: v1\ndomains:\n  - demo.lib\n",
    )
    .unwrap();
    std::fs::write(fixture.join("spec/lib.yaml"), "domain: demo.lib\n").unwrap();
    for target in ["rust", "web"] {
        for format in ["text", "json", "yaml"] {
            for destination in ["absent", "existing", "none"] {
                let out = fixture.join(format!("{target}-{format}-{destination}"));
                if destination == "existing" {
                    std::fs::create_dir_all(&out).unwrap();
                    std::fs::write(out.join("plan.json"), "untouched plan\n").unwrap();
                }
                let mut streams = Vec::new();
                for prefix in [&["generate", "synthesize"][..], &["synthesize"][..]] {
                    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
                    command
                        .args(prefix)
                        .arg("--path")
                        .arg(fixture.join("spec"))
                        .args(["--target", target, "--format", format]);
                    if destination != "none" {
                        command.arg("--out").arg(&out);
                    }
                    let output = command.output().unwrap();
                    assert_eq!(
                        output.status.code(),
                        Some(1),
                        "{command:?}: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    assert!(
                        output.stderr.is_empty(),
                        "structured target failure belongs on stdout"
                    );
                    if format == "text" {
                        let text = String::from_utf8_lossy(&output.stdout);
                        assert!(text.contains("demo.lib") && text.contains(target));
                        assert!(!text.contains("artifact(s)"));
                    } else {
                        let value: serde_json::Value = if format == "json" {
                            serde_json::from_slice(&output.stdout).unwrap()
                        } else {
                            serde_yaml::from_slice(&output.stdout).unwrap()
                        };
                        assert_eq!(value["format"], "ess-target-failure/1");
                        assert_eq!(value["target"], target);
                        assert_eq!(value["plan"]["capabilities"], serde_json::json!([]));
                        assert!(!value["causes"].as_array().unwrap().is_empty());
                    }
                    streams.push((output.stdout, output.stderr));
                }
                assert_eq!(streams[0], streams[1]);
                if destination == "existing" {
                    assert_eq!(
                        std::fs::read(out.join("plan.json")).unwrap(),
                        b"untouched plan\n"
                    );
                    assert_eq!(std::fs::read_dir(out).unwrap().count(), 1);
                } else {
                    assert!(!out.exists());
                }
            }
        }
    }
}
