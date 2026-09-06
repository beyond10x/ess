//! Native Binary64 publication must preserve every destination when a later path refuses.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn snapshot(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    if path.exists() {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                files.extend(snapshot(&path));
            } else {
                files.insert(path.clone(), fs::read(path).unwrap());
            }
        }
    }
    files
}

#[test]
fn binary64_publication_never_replaces_sources_or_partially_updates_a_library() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "binary64-structural-adversary-cli-{}",
        std::process::id()
    ));
    let model = root.join("model");
    fs::create_dir_all(&model).unwrap();
    fs::write(model.join("system.yaml"), "format: ess/2\nsystem: probe\nversion: v1\ndomains: [probe.float]\ndomain: probe.float\ntypes:\n  - {name: probe.float.Number, kind: newtype, of: Binary64}\n").unwrap();
    for (target, extension, manifest) in [("rust", "rs", "Cargo.toml"), ("go", "go", "go.mod")] {
        let invoke = |destination: &Path| {
            let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
            command
                .args(["generate", "types", "--path"])
                .arg(&model)
                .args([
                    "--all-types",
                    "--target",
                    target,
                    "--package",
                    "adversary_types",
                    "--out",
                ])
                .arg(destination)
                .env_remove("ESS_GO_COMPILER")
                .env("PATH", "/nonexistent");
            if target == "go" {
                command.args(["--module", "example.invalid/adversary-types"]);
            }
            let output = command.output().unwrap();
            println!(
                "{command:?}: {}\nstdout:\n{}\nstderr:\n{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            output
        };
        let destination = root.join(target);
        fs::create_dir_all(&destination).unwrap();
        for file in [
            format!("types.{extension}"),
            manifest.to_owned(),
            "source.schema.json".to_owned(),
        ] {
            fs::write(destination.join(file), "existing library sentinel\n").unwrap();
        }
        fs::create_dir(destination.join("types-report.json")).unwrap();
        fs::write(
            destination.join("types-report.json/keep"),
            "nested sentinel",
        )
        .unwrap();
        let before = snapshot(&destination);
        let output = invoke(&destination);
        assert!(!output.status.success());
        assert_eq!(
            snapshot(&destination),
            before,
            "{target}: partial publication"
        );

        let before = snapshot(&model);
        let nested = model.join(format!("{target}-inside"));
        let output = invoke(&nested);
        assert!(!output.status.success());
        assert_eq!(snapshot(&model), before);
        assert!(
            !nested.exists(),
            "created a destination inside source input"
        );

        #[cfg(unix)]
        {
            let linked = root.join(format!("{target}-linked"));
            fs::create_dir_all(&linked).unwrap();
            std::os::unix::fs::symlink(
                model.join("system.yaml"),
                linked.join(format!("types.{extension}")),
            )
            .unwrap();
            let output = invoke(&linked);
            assert!(!output.status.success());
            assert_eq!(snapshot(&model), before);
            assert!(!linked.join(manifest).exists());
            assert!(!linked.join("types-report.json").exists());
        }

        let success = root.join(format!("{target}-fresh"));
        assert!(invoke(&success).status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(success.join("types-report.json")).unwrap()).unwrap();
        assert_eq!(report["format"], "ess-types-report/3");
        assert_eq!(report["roots"], serde_json::json!(["probe.float.Number"]));
        assert!(
            fs::read_to_string(success.join(format!("types.{extension}")))
                .unwrap()
                .contains("EssBinary64")
        );
    }
}
