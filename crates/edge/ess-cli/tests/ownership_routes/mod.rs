//! Real CLI writer witnesses. Runtime libraries are emitted, not executed by these cases.
use super::{snapshot, workspace, Fixture};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
fn cli(f: &Fixture, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(&f.0)
        .args(args)
        .output()
        .unwrap()
}
fn good(f: &Fixture, args: &[&str]) -> Output {
    let out = cli(f, args);
    assert!(out.status.success(), "{args:?}: {out:?}");
    out
}
fn model(f: &Fixture) {
    fs::create_dir(f.0.join("model")).unwrap();
    fs::write(
        f.0.join("model/system.yaml"),
        include_str!("../../../../generate/ess-gen/tests/fixtures/model-types.yaml"),
    )
    .unwrap();
}
fn ledger(root: &Path) -> BTreeMap<String, BTreeSet<PathBuf>> {
    let value: Value =
        serde_json::from_slice(&fs::read(root.join(".ess-output/state.json")).unwrap()).unwrap();
    assert_eq!(value["payload"]["checkpoint"]["phase"], "Idle");
    value["payload"]["checkpoint"]["ledger"]["owners"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            let paths = o["files"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| {
                    use std::os::unix::ffi::OsStringExt;
                    let p: PathBuf = f["path"]["components"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|c| {
                            let c = c.as_str().unwrap();
                            std::ffi::OsString::from_vec(
                                c.as_bytes()
                                    .chunks_exact(2)
                                    .map(|b| {
                                        u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16)
                                            .unwrap()
                                    })
                                    .collect(),
                            )
                        })
                        .collect();
                    assert!(root.join(&p).is_file());
                    p
                })
                .collect();
            (o["key"]["family"].as_str().unwrap().to_owned(), paths)
        })
        .collect()
}
fn one(root: &Path, family: &str) -> BTreeSet<PathBuf> {
    let ledger = ledger(root);
    assert_eq!(
        ledger.keys().map(String::as_str).collect::<Vec<_>>(),
        [family]
    );
    ledger[family].clone()
}
fn changed(root: &Path, old: &BTreeSet<PathBuf>, new: &BTreeSet<PathBuf>) {
    for p in old.difference(new) {
        assert!(!root.join(p).exists(), "stale {}", p.display());
    }
}

#[test]
fn all_projection_owners_are_separate_and_selected_site_uses_artifact_paths() {
    let _serial = super::serial();
    let f = Fixture::new();
    let path = workspace().join("examples/billing");
    good(
        &f,
        &["generate", "--path", path.to_str().unwrap(), "--out", "out"],
    );
    let before = ledger(&f.0.join("out"));
    assert_eq!(
        before.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "projection:asyncapi",
            "projection:docs",
            "projection:openapi",
            "projection:schema",
            "projection:site"
        ]
    );
    let old_site = before["projection:site"].clone();
    assert!(old_site.iter().all(|p| p.starts_with("site")));
    let other_bytes = before
        .iter()
        .filter(|(k, _)| k.as_str() != "projection:site")
        .flat_map(|(_, p)| p)
        .map(|p| (p.clone(), fs::read(f.0.join("out").join(p)).unwrap()))
        .collect::<BTreeMap<_, _>>();
    good(
        &f,
        &[
            "generate",
            "--path",
            path.to_str().unwrap(),
            "--kind",
            "site",
            "--out",
            "out",
        ],
    );
    let after = ledger(&f.0.join("out"));
    assert!(after["projection:site"].contains(Path::new("index.html")));
    changed(&f.0.join("out"), &old_site, &after["projection:site"]);
    for (p, b) in other_bytes {
        assert_eq!(fs::read(f.0.join("out").join(p)).unwrap(), b);
    }
    good(
        &f,
        &[
            "generate",
            "--path",
            path.to_str().unwrap(),
            "--kind",
            "docs-ir",
            "--out",
            "out",
        ],
    );
    assert_eq!(ledger(&f.0.join("out")).len(), 6);
}

#[test]
fn synthesis_and_model_type_target_switches_retire_the_fixed_family_set() {
    let _serial = super::serial();
    let f = Fixture::new();
    model(&f);
    let path = workspace().join("examples/billing");
    let mut old = BTreeSet::new();
    for target in ["rust", "go", "web", "clap"] {
        let output = cli(
            &f,
            &[
                "synthesize",
                "--path",
                path.to_str().unwrap(),
                "--target",
                target,
                "--out",
                "synthesis",
            ],
        );
        assert!(matches!(output.status.code(), Some(0 | 1)), "{output:?}");
        let new = one(&f.0.join("synthesis"), "synthesis");
        assert!(!new.is_empty());
        changed(&f.0.join("synthesis"), &old, &new);
        old = new;
    }
    old.clear();
    for target in ["typescript", "rust", "go", "typescript"] {
        let mut args = vec![
            "generate",
            "types",
            "--path",
            "model",
            "--root",
            "sample.data.Record",
            "--target",
            target,
            "--out",
            "types",
        ];
        if target != "typescript" {
            args.extend(["--package", "record_types"]);
        }
        if target == "go" {
            args.extend(["--module", "example.invalid/recordtypes"]);
        }
        good(&f, &args);
        let new = one(&f.0.join("types"), "model-types");
        changed(&f.0.join("types"), &old, &new);
        old = new;
    }
}

#[test]
fn every_composition_companion_combination_replaces_one_explicit_anchor_owner() {
    let _serial = super::serial();
    let f = Fixture::new();
    let source = workspace().join("crates/specify/ess-composition/tests/fixtures");
    let plan = source.join("compositions/workbench.yaml");
    let service = format!("todo={}", source.join("two-components").display());
    let usage = format!("usage={}", source.join("two-components").display());
    let base = vec![
        "compose",
        "--path",
        plan.to_str().unwrap(),
        "--service",
        &service,
        "--service",
        &usage,
    ];
    // Named compose companions retain their existing-parent admission contract.
    fs::create_dir_all(f.0.join("anchor/reports")).unwrap();
    fs::create_dir_all(f.0.join("anchor/plans")).unwrap();
    let before = snapshot(&f.0);
    good(&f, &base);
    assert_eq!(snapshot(&f.0), before);
    let mut missing = base.clone();
    missing.extend(["--out", "anchor/missing.json"]);
    let output = cli(&f, &missing);
    assert!(!output.status.success());
    assert!(!f.0.join("anchor/.ess-output").exists());
    assert!(!f.0.join("anchor/missing.json").exists());
    let mut old = BTreeSet::new();
    for mask in [7, 1, 2, 3, 4, 5, 6, 7] {
        let mut args = base.clone();
        args.extend(["--ownership-root", "anchor"]);
        if mask & 1 != 0 {
            args.extend(["--out", "anchor/reports/composition.json"]);
        }
        if mask & 2 != 0 {
            args.extend(["--client-plan-out", "anchor/plans/client.json"]);
        }
        if mask & 4 != 0 {
            args.extend(["--client-rust-out", "anchor/client"]);
        }
        good(&f, &args);
        let new = one(&f.0.join("anchor"), "compose");
        assert_eq!(
            new.len(),
            usize::from(mask & 1 != 0)
                + usize::from(mask & 2 != 0)
                + 3 * usize::from(mask & 4 != 0)
        );
        changed(&f.0.join("anchor"), &old, &new);
        old = new;
        fs::write(f.0.join("anchor/authored"), "keep").unwrap();
    }
    let before = snapshot(&f.0);
    good(&f, &base);
    assert_eq!(snapshot(&f.0), before);
    assert_eq!(fs::read(f.0.join("anchor/authored")).unwrap(), b"keep");
}

#[test]
fn conformance_legacy_and_coverage_writers_share_owners_and_preserve_authored_skin() {
    let _serial = super::serial();
    let f = Fixture::new();
    let path = workspace().join("examples/billing");
    fs::write(
        f.0.join("scenario.yaml"),
        include_str!("../fixtures/coverage-producers/inputs/authored/single/a.yaml"),
    )
    .unwrap();
    for suite in ["4", "5", "4"] {
        let go = cli(
            &f,
            &[
                "conform",
                "synthesize",
                "--path",
                path.to_str().unwrap(),
                "--scenarios",
                "scenario.yaml",
                "--target",
                "go",
                "--suite-format",
                suite,
                "--out",
                "go",
            ],
        );
        assert!(matches!(go.status.code(), Some(0 | 1)), "{go:?}");
        assert!(!one(&f.0.join("go"), "conformance-go").is_empty());
        let web = cli(
            &f,
            &[
                "conform",
                "web",
                "--path",
                path.to_str().unwrap(),
                "--scenarios",
                "scenario.yaml",
                "--suite-format",
                suite,
                "--out",
                "browser",
            ],
        );
        assert!(matches!(web.status.code(), Some(0 | 1)), "{web:?}");
        assert!(!one(&f.0.join("browser"), "conformance-browser").is_empty());
        fs::write(f.0.join("browser/skin.js"), "authored skin").unwrap();
    }
    assert_eq!(
        fs::read(f.0.join("browser/skin.js")).unwrap(),
        b"authored skin"
    );
}

#[test]
fn bundle_and_normalization_targets_keep_independent_owners_and_checks_do_not_write() {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::normalize::Root;
    let _serial = super::serial();
    let f = Fixture::new();
    let schema = json!({"components":{"schemas":{"Input":{"type":"string"}}}}).to_string();
    let bundle = import(
        &schema,
        &["Input".to_owned()].into_iter().collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    fs::write(f.0.join("source.bundle.json"), bundle.to_json().unwrap()).unwrap();
    let recipe = json!({"format":"ess-normalization/1","branches":{"primary":[{"input":Root::pin(&bundle,"Input").unwrap(),"output":Root::pin(&bundle,"Input").unwrap(),"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}});
    fs::write(f.0.join("recipe.json"), recipe.to_string()).unwrap();
    for (operation, family, root) in [
        ("types-bundle", "types-bundle", "bundle-types"),
        ("normalize-generate", "normalization", "normalization"),
    ] {
        let mut old = BTreeSet::new();
        for target in ["typescript", "rust", "go", "typescript"] {
            let mut args = vec![
                "schema",
                operation,
                "--bundle",
                "source.bundle.json",
                "--target",
                target,
                "--out",
                root,
            ];
            if operation == "types-bundle" {
                args.extend(["--root", "Input"]);
            } else {
                args.extend(["--recipe", "recipe.json"]);
            }
            if target != "typescript" || operation == "normalize-generate" {
                args.extend(["--package", "library"]);
            }
            if target == "go" {
                args.extend(["--module", "example.invalid/library"]);
            }
            good(&f, &args);
            let new = one(&f.0.join(root), family);
            changed(&f.0.join(root), &old, &new);
            old = new;
            if operation == "normalize-generate" {
                args.push("--check");
                let before = snapshot(&f.0);
                good(&f, &args);
                assert_eq!(snapshot(&f.0), before);
            }
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn standalone_writers_and_output_management_use_fixed_native_file_owners() {
    let _serial = super::serial();
    let f = Fixture::new();
    let path = workspace().join("examples/billing");
    good(
        &f,
        &[
            "realization",
            "generate",
            "--path",
            workspace()
                .join("examples/realizations/billing-local.yaml")
                .to_str()
                .unwrap(),
            "--spec",
            path.to_str().unwrap(),
            "--out",
            "standalone/run.md",
        ],
    );
    assert_eq!(
        one(&f.0.join("standalone"), "realization-markdown"),
        [PathBuf::from("run.md")].into_iter().collect()
    );
    let before = snapshot(&f.0);
    good(
        &f,
        &[
            "realization",
            "generate",
            "--path",
            workspace()
                .join("examples/realizations/billing-local.yaml")
                .to_str()
                .unwrap(),
            "--spec",
            path.to_str().unwrap(),
            "--out",
            "standalone/run.md",
            "--check",
        ],
    );
    assert_eq!(snapshot(&f.0), before);
    fs::write(
        f.0.join("record.schema.json"),
        r#"{"$id":"urn:record","type":"string"}"#,
    )
    .unwrap();
    good(
        &f,
        &[
            "schema",
            "typescript",
            "--schemas",
            "record.schema.json",
            "urn:record",
            "--root",
            "Record",
            "--out",
            "ts/record.ts",
        ],
    );
    one(&f.0.join("ts"), "typescript-file");
    fs::create_dir(f.0.join("legacy")).unwrap();
    fs::copy(f.0.join("ts/record.ts"), f.0.join("legacy/record.ts")).unwrap();
    good(
        &f,
        &[
            "output",
            "adopt",
            "--ownership-root",
            "legacy",
            "--from",
            "ts",
            "--owner",
            "typescript-file",
            "--file",
            "record.ts",
        ],
    );
    let before = snapshot(&f.0);
    good(
        &f,
        &[
            "generate",
            "output",
            "adopt",
            "--ownership-root",
            "legacy",
            "--from",
            "ts",
            "--owner",
            "typescript-file",
            "--file",
            "record.ts",
        ],
    );
    assert_eq!(snapshot(&f.0), before);
    for args in [
        vec![
            "output",
            "adopt",
            "--ownership-root",
            "legacy",
            "--from",
            "ts",
            "--owner",
            "typescript-file",
        ],
        vec![
            "output",
            "adopt",
            "--ownership-root",
            "legacy",
            "--from",
            "ts",
            "--owner",
            "synthesis",
            "--file",
            "record.ts",
        ],
    ] {
        assert!(!cli(&f, &args).status.success());
        assert_eq!(snapshot(&f.0), before);
    }
    fs::write(
        f.0.join("source.openapi.yaml"),
        include_str!("../../../../generate/ess-openapi/tests/fixtures/supported.openapi.yaml"),
    )
    .unwrap();
    good(
        &f,
        &[
            "import",
            "openapi",
            "--path",
            "source.openapi.yaml",
            "--out",
            "import.json",
        ],
    );
    good(
        &f,
        &[
            "project",
            "openapi",
            "--ir",
            "import.json",
            "--out",
            "openapi/document.yaml",
        ],
    );
    one(&f.0.join("openapi"), "openapi-file");
    good(
        &f,
        &[
            "project",
            "openapi",
            "--path",
            path.to_str().unwrap(),
            "--out",
            "projection",
        ],
    );
    one(&f.0.join("projection"), "projection:openapi");
}

#[test]
#[allow(clippy::too_many_lines)]
fn native_projection_routes_enroll_and_build_execution_waits_for_publication() {
    use std::os::unix::fs::PermissionsExt;
    let _serial = super::serial();
    let f = Fixture::new();
    let build="format: ess-build/1\nbuild: ownership-fixture\nplatforms: [{os: linux, architecture: amd64}]\nnodes:\n  - {id: source, kind: source, path: ., destination: /src}\n  - {id: output, kind: artifact, from: source, path: /src/fixture.bin}\noutputs:\n  - {name: binary, release_unit: ownership-fixture, node: output, kind: binary}\n";
    fs::write(f.0.join("build.yaml"), build).unwrap();
    good(
        &f,
        &[
            "build",
            "compile",
            "--path",
            "build.yaml",
            "--out",
            "build.ir.json",
        ],
    );
    good(
        &f,
        &[
            "project",
            "buildkit",
            "--ir",
            "build.ir.json",
            "--out",
            "buildkit",
        ],
    );
    assert_eq!(
        one(&f.0.join("buildkit"), "buildkit"),
        ["Dockerfile.ess", "docker-bake.hcl", "ess-build-ir.json"]
            .into_iter()
            .map(PathBuf::from)
            .collect()
    );
    let digest = format!("sha256:{}", "0".repeat(64));
    let runtime = json!({"format":"ess-runtime-ir/1","runtime":"fixture","semantic_digest":digest,"realization_digest":digest,"build_digest":digest,"processes":{"server":{"name":"server","image":"app"}},"containers":{"server":{"name":"server","process":"server"}},"workloads":{"app":{"name":"app","components":["component"],"containers":["server"],"replicas":1}},"provided_endpoints":{}});
    let checked = ess_deployment::RuntimeIr::from_json(&runtime.to_string()).unwrap();
    fs::write(f.0.join("runtime.json"), checked.to_canonical_json()).unwrap();
    good(
        &f,
        &[
            "project",
            "helm",
            "--ir",
            "runtime.json",
            "--chart",
            "fixture",
            "--version",
            "1.0.0",
            "--out",
            "helm",
        ],
    );
    assert_eq!(one(&f.0.join("helm"), "helm").len(), 5);
    good(
        &f,
        &[
            "project",
            "kubernetes",
            "--spec",
            workspace()
                .join("examples/k3d-dev-cluster/expected.yaml")
                .to_str()
                .unwrap(),
            "--ir",
            workspace()
                .join("examples/k3d-dev-cluster/cluster.ir.json")
                .to_str()
                .unwrap(),
            "--out",
            "kubernetes",
        ],
    );
    assert!(!one(&f.0.join("kubernetes"), "kubernetes").is_empty());
    let tools = f.0.join("tools");
    fs::create_dir(&tools).unwrap();
    let compiled = Command::new("rustc")
        .arg("--edition=2021")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ownership_routes/fake_docker.rs"))
        .arg("-o")
        .arg(tools.join("docker"))
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .current_dir(&f.0)
        .env("PATH", &tools)
        .env("ESS_OWNERSHIP_DOCKER_WITNESS", f.0.join("called"))
        .args([
            "build",
            "execute",
            "--path",
            "build.yaml",
            "--workdir",
            ".",
            "--projection-out",
            "buildkit",
        ]);
    let first = command.output().unwrap();
    assert!(first.status.success(), "{first:?}");
    assert_eq!(fs::read(f.0.join("called")).unwrap(), b"executor reached");
    fs::remove_file(f.0.join("called")).unwrap();
    let before = snapshot(&f.0.join("buildkit"));
    fs::write(
        f.0.join("build.yaml"),
        build.replace("fixture.bin", "changed.bin"),
    )
    .unwrap();
    fs::set_permissions(
        f.0.join("buildkit/.ess-output"),
        fs::Permissions::from_mode(0o500),
    )
    .unwrap();
    let refused = command.output().unwrap();
    fs::set_permissions(
        f.0.join("buildkit/.ess-output"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    assert!(!refused.status.success(), "{refused:?}");
    assert!(!f.0.join("called").exists());
    assert_eq!(snapshot(&f.0.join("buildkit")), before);
    let repaired = command.output().unwrap();
    assert!(repaired.status.success(), "{repaired:?}");
    assert_eq!(fs::read(f.0.join("called")).unwrap(), b"executor reached");
    one(&f.0.join("buildkit"), "buildkit");
}

#[test]
fn a_published_incomplete_coverage_suite_keeps_semantic_exit_one() {
    let _serial = super::serial();
    let f = Fixture::new();
    fs::write(f.0.join("refused.yaml"), "not: a scenario\n").unwrap();
    let output = cli(
        &f,
        &[
            "conform",
            "synthesize",
            "--path",
            workspace().join("examples/billing").to_str().unwrap(),
            "--scenarios",
            "refused.yaml",
            "--suite-format",
            "5",
            "--target",
            "go",
            "--out",
            "partial",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let suite =
        ess_conformance::AdmittedSuite::from_json(std::str::from_utf8(&output.stdout).unwrap())
            .unwrap();
    assert!(!suite.coverage().unwrap().is_complete());
    assert!(!suite.coverage().unwrap().refused.is_empty());
    assert!(!one(&f.0.join("partial"), "conformance-go").is_empty());
    assert_eq!(
        fs::read(f.0.join("partial/essconform/suite.json")).unwrap(),
        output.stdout
    );
}
