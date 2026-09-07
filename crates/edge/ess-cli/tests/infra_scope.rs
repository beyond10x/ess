//! Namespace coverage survives the public command surface and file writer.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn both_import_spellings_preserve_scope_through_graph_diff_and_projection_refusal() {
    let scratch =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("infra-scope-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let source = root().join("crates/infra/infra-compiler/tests/fixtures/namespace-topology.json");
    for (index, prefix) in [
        vec!["infra", "import", "kubernetes"],
        vec!["import", "kubernetes"],
    ]
    .into_iter()
    .enumerate()
    {
        let ir = scratch.join(format!("ir-{index}.json"));
        let output = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(&prefix)
            .arg("--path")
            .arg(&source)
            .arg("--out")
            .arg(&ir)
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(!report["coverage_gaps"].as_array().unwrap().is_empty());
        let document: Value = serde_json::from_slice(&std::fs::read(&ir).unwrap()).unwrap();
        assert_eq!(document["format"], "infra-ir/2");
        assert_eq!(document["model"]["coverage"]["namespace"], "app");
        let graph = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(["infra", "infra", "graph", "--path"])
            .arg(&ir)
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert!(graph.status.success());
        assert_eq!(
            serde_json::from_slice::<Value>(&graph.stdout).unwrap()["format"],
            "infra-graph/2"
        );
        let diff = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(["infra", "infra", "diff", "--from"])
            .arg(&ir)
            .arg("--to")
            .arg(&ir)
            .output()
            .unwrap();
        assert!(diff.status.success());
        assert!(String::from_utf8(diff.stdout)
            .unwrap()
            .contains("omitted content remains unobserved"));
        let destination = scratch.join(format!("projection-{index}"));
        let projection = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(["generate", "project", "kubernetes", "--ir"])
            .arg(&ir)
            .arg("--spec")
            .arg(root().join("examples/k3d-dev-cluster/expected.yaml"))
            .arg("--out")
            .arg(&destination)
            .output()
            .unwrap();
        assert!(!projection.status.success());
        assert!(!destination.exists());
        let missing_authority = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(&prefix)
            .args(["--namespace", "app", "--path"])
            .arg(&source)
            .output()
            .unwrap();
        assert!(!missing_authority.status.success());
    }
}
