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
        assert_ne!(report["coverage_gaps"].as_array().unwrap().len(), 0);
        assert!(report["supported"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("route")));
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

#[test]
fn the_imported_ir_is_json_whatever_the_out_extension_and_the_help_says_so() {
    let scratch =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("infra-out-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let source = root().join("crates/infra/infra-compiler/tests/fixtures/namespace-topology.json");
    let out = scratch.join("cluster.ir.yaml");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["infra", "import", "kubernetes", "--path"])
        .arg(&source)
        .arg("--out")
        .arg(&out)
        .args(["--format", "yaml"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let document: Value = serde_json::from_slice(&std::fs::read(&out).unwrap())
        .expect("`--out` writes JSON even under a .yaml name");
    assert_eq!(document["format"], "infra-ir/2");

    let help = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["infra", "import", "kubernetes", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success(), "{help:?}");
    let help = String::from_utf8(help.stdout).unwrap();
    let out_line = help
        .lines()
        .skip_while(|line| !line.trim_start().starts_with("--out"))
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        out_line.contains("JSON"),
        "the --out help names JSON: {help}"
    );
}

#[test]
fn the_infrastructure_graph_renders_as_dot_and_graphviz_parses_it() {
    let source = root().join("crates/infra/infra-compiler/tests/fixtures/namespace-topology.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["infra", "infra", "graph", "--path"])
        .arg(&source)
        .args(["--format", "dot"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let dot = String::from_utf8(output.stdout).unwrap();
    assert!(dot.contains("digraph infrastructure {\n"), "{dot}");
    assert!(dot.contains("subgraph \"cluster_app\" {"), "{dot}");

    let scratch =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("infra-dot-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let file = scratch.join("graph.dot");
    std::fs::write(&file, &dot).unwrap();
    match Command::new("dot").arg("-Tsvg").arg(&file).output() {
        Ok(svg) => {
            assert!(
                svg.status.success(),
                "dot -Tsvg refused the output: {svg:?}"
            );
            assert!(String::from_utf8_lossy(&svg.stdout).contains("<svg"));
        }
        Err(error) => println!("skip: Graphviz `dot` is not runnable here ({error})"),
    }
}

#[test]
fn importing_a_legacy_ir_writes_no_secret_digest_it_carried() {
    // A persisted `infra-ir/1` still carries the scanner's unsalted `{sha256, length}` per Secret
    // key. Reading it is allowed; writing it again is not: anything this build writes records a
    // Secret key as present and nothing derived from its value.
    let scratch =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("infra-legacy-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let legacy =
        root().join("crates/infra/infra-compiler/tests/fixtures/legacy-k3d-dev-cluster.ir-1.json");
    let source: Value = serde_json::from_slice(&std::fs::read(&legacy).unwrap()).unwrap();
    let mut digests = Vec::new();
    for secret in source["model"]["secrets"].as_object().unwrap().values() {
        for value in secret["keys"].as_object().unwrap().values() {
            digests.push(value["sha256"].as_str().unwrap().to_owned());
        }
    }
    assert!(
        !digests.is_empty(),
        "the legacy fixture carries no digest to drop"
    );
    for (index, prefix) in [
        vec!["infra", "import", "kubernetes"],
        vec!["import", "kubernetes"],
    ]
    .into_iter()
    .enumerate()
    {
        let out = scratch.join(format!("ir-{index}.json"));
        let output = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(&prefix)
            .arg("--path")
            .arg(&legacy)
            .arg("--out")
            .arg(&out)
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let written = std::fs::read_to_string(&out).unwrap();
        for digest in &digests {
            assert!(
                !written.contains(digest.as_str()),
                "{prefix:?} wrote the legacy Secret digest {digest} back out"
            );
        }
        let document: Value = serde_json::from_str(&written).unwrap();
        assert_eq!(document["format"], "infra-ir/3");
        for secret in document["model"]["secrets"].as_object().unwrap().values() {
            for value in secret["keys"].as_object().unwrap().values() {
                assert_eq!(value, &serde_json::json!({"present": true}));
            }
        }
        let reread = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(["infra", "infra", "graph", "--path"])
            .arg(&out)
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert!(
            reread.status.success(),
            "the written IR reads back: {reread:?}"
        );
    }
}

#[test]
fn served_kinds_the_scan_did_not_read_are_coverage_gaps_of_the_import_report() {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("infra-unobserved-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let empty = serde_json::json!({ "items": [] });
    let mut kinds = serde_json::Map::new();
    for kind in [
        "namespaces",
        "nodes",
        "deployments",
        "statefulsets",
        "daemonsets",
        "pods",
        "services",
        "ingresses",
        "configmaps",
        "secrets",
        "serviceaccounts",
        "persistentvolumeclaims",
    ] {
        kinds.insert(kind.to_owned(), empty.clone());
    }
    let bundle = serde_json::json!({
        "format": "infra-observation/3", "context": "synthetic",
        "scanned_at": "2026-11-10T00:00:00Z", "scout_version": "0.92.0", "kinds": kinds,
        "unobserved_kinds": ["certificates.cert-manager.io", "networkpolicies.networking.k8s.io"],
    });
    let source = scratch.join("observation.json");
    std::fs::write(&source, bundle.to_string()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["infra", "import", "kubernetes", "--path"])
        .arg(&source)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["coverage_gaps"],
        serde_json::json!([
            "served but not read: certificates.cert-manager.io",
            "served but not read: networkpolicies.networking.k8s.io"
        ])
    );
}
