//! Unsupported codecs must fail before creating or replacing any output artifact.

use std::{fs, path::Path, process::Command};

#[test]
fn binary64_sparse_model_refuses_every_unsupported_publication_route() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/binary64-publication")
        .join(std::process::id().to_string());
    fs::create_dir_all(root.join("spec")).unwrap();
    fs::write(root.join("spec/system.yaml"), "format: ess/2\nsystem: sample\nversion: v1\ndomains: [sample.data]\ndomain: sample.data\ntypes:\n  - {name: sample.data.Ratio, kind: newtype, of: Binary64}\n").unwrap();
    let operations: &[(&[&str], bool)] = &[
        (&["synthesize", "--target", "rust"], true),
        (&["synthesize", "--target", "go"], true),
        (&["synthesize", "--target", "web"], true),
        (&["synthesize", "--target", "clap"], true),
        (
            &["verify", "conform", "synthesize", "--target", "ir"],
            false,
        ),
        (&["verify", "conform", "synthesize", "--target", "go"], true),
        (&["verify", "conform", "author"], false),
        (&["verify", "conform", "web"], true),
        (&["verify", "conform", "run", "--target", "billing"], false),
    ];
    for (index, (arguments, directory)) in operations.iter().enumerate() {
        for existing in [false, true] {
            let out = root.join(format!("out-{index}-{existing}"));
            if existing {
                if *directory {
                    fs::create_dir_all(&out).unwrap();
                    fs::write(out.join("sentinel"), "owned bytes\n").unwrap();
                } else {
                    fs::write(&out, "owned bytes\n").unwrap();
                }
            }
            let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
            command
                .args(*arguments)
                .arg("--path")
                .arg(root.join("spec"))
                .args(["--format", "json"])
                .arg(if arguments.contains(&"run") {
                    "--report-out"
                } else {
                    "--out"
                })
                .arg(&out);
            let result = command.output().unwrap();
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            fs::write(
                root.join(format!("case-{index}-{existing}.log")),
                format!("{command:?}\n{}\n{combined}", result.status),
            )
            .unwrap();
            assert!(!result.status.success(), "{arguments:?}: {combined}");
            assert!(
                combined.contains("Binary64") && combined.contains("sample.data.Ratio"),
                "located finite refusal: {combined}"
            );
            if arguments[0] == "synthesize" {
                let failure: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
                let expected = if arguments[2] == "rust" || arguments[2] == "web" {
                    "ess-target-failure/1"
                } else {
                    "ess-target-failure/2"
                };
                assert_eq!(failure["format"], expected);
            }
            if existing {
                let file = if *directory {
                    out.join("sentinel")
                } else {
                    out.clone()
                };
                assert_eq!(fs::read(file).unwrap(), b"owned bytes\n");
                if *directory {
                    assert_eq!(fs::read_dir(&out).unwrap().count(), 1);
                }
            } else {
                assert!(!out.exists(), "refusal created {}", out.display());
            }
        }
    }
    eprintln!("18 actual CLI refusal executions preserved absent/existing destinations");
}
