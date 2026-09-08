//! Caller-class controls for output ownership correction 1.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn fixture() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "e19-correction-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    root
}
fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("examples/billing").is_dir())
        .unwrap()
        .to_owned()
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn walk(root: &Path, p: &Path, out: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for e in fs::read_dir(p).unwrap() {
            let p = e.unwrap().path();
            let m = fs::symlink_metadata(&p).unwrap();
            out.insert(
                p.strip_prefix(root).unwrap().to_owned(),
                m.is_file().then(|| fs::read(&p).unwrap()),
            );
            if m.is_dir() {
                walk(root, &p, out);
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}
fn cli(root: &Path, args: &[&str]) -> Output {
    let o = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    println!(
        "{args:?}: {:?}\n{}",
        o.status,
        String::from_utf8_lossy(&o.stderr)
    );
    o
}
fn composition(root: &Path, outputs: &[&str]) -> Output {
    let source = workspace().join("crates/specify/ess-composition/tests/fixtures");
    let plan = source.join("compositions/workbench.yaml");
    let todo = format!("todo={}", source.join("two-components").display());
    let usage = format!("usage={}", source.join("two-components").display());
    let mut args = vec![
        "compose",
        "--path",
        plan.to_str().unwrap(),
        "--service",
        &todo,
        "--service",
        &usage,
        "--ownership-root",
        "anchor",
    ];
    args.extend_from_slice(outputs);
    cli(root, &args)
}

#[test]
fn all_standalone_families_and_companions_refuse_directory_spelling_without_writes() {
    let root = fixture();
    fs::create_dir(root.join("out")).unwrap();
    fs::create_dir(root.join("anchor")).unwrap();
    fs::write(root.join("out/authored"), "keep").unwrap();
    fs::write(
        root.join("record.schema.json"),
        r#"{"$id":"urn:record","type":"string"}"#,
    )
    .unwrap();
    fs::write(
        root.join("source.openapi.yaml"),
        include_str!("../../../generate/ess-openapi/tests/fixtures/supported.openapi.yaml"),
    )
    .unwrap();
    assert!(cli(
        &root,
        &[
            "import",
            "openapi",
            "--path",
            "source.openapi.yaml",
            "--out",
            "import.json"
        ]
    )
    .status
    .success());
    let realization = workspace().join("examples/realizations/billing-local.yaml");
    let billing = workspace().join("examples/billing");
    for spelling in [
        "out/result/",
        "out/result//",
        "out/result/.",
        "out/result/..",
    ] {
        for mut args in [
            vec![
                "schema",
                "typescript",
                "--schemas",
                "record.schema.json",
                "urn:record",
                "--root",
                "Record",
            ],
            vec![
                "realization",
                "generate",
                "--path",
                realization.to_str().unwrap(),
                "--spec",
                billing.to_str().unwrap(),
            ],
            vec!["project", "openapi", "--ir", "import.json"],
        ] {
            args.extend(["--out", spelling]);
            let before = snapshot(&root);
            let result = cli(&root, &args);
            assert!(!result.status.success(), "{args:?}");
            assert_eq!(snapshot(&root), before);
        }
        for flag in ["--out", "--client-plan-out"] {
            let before = snapshot(&root);
            assert!(!composition(&root, &[flag, spelling]).status.success());
            assert_eq!(snapshot(&root), before);
        }
    }
}

#[test]
fn compose_transitions_both_companions_and_their_owned_parent_in_both_directions() {
    for flag in ["--out", "--client-plan-out"] {
        let root = fixture();
        fs::create_dir(root.join("anchor")).unwrap();
        fs::write(root.join("anchor/authored"), "keep").unwrap();
        assert!(composition(&root, &[flag, "anchor/client"])
            .status
            .success());
        assert!(composition(&root, &["--client-rust-out", "anchor/client"])
            .status
            .success());
        assert!(root.join("anchor/client/src/lib.rs").is_file());
        assert!(composition(&root, &[flag, "anchor/client"])
            .status
            .success());
        assert!(root.join("anchor/client").is_file());
        assert!(composition(&root, &[flag, "anchor/client/companion.json"])
            .status
            .success());
        assert!(root.join("anchor/client/companion.json").is_file());
        assert!(composition(&root, &[flag, "anchor/client"])
            .status
            .success());
        assert_eq!(fs::read(root.join("anchor/authored")).unwrap(), b"keep");
    }
}

#[test]
fn compose_transition_refuses_authored_children_unowned_parents_and_aliases() {
    let root = fixture();
    fs::create_dir(root.join("anchor")).unwrap();
    assert!(composition(&root, &["--client-rust-out", "anchor/client"])
        .status
        .success());
    fs::write(root.join("anchor/client/authored"), "keep").unwrap();
    let before = snapshot(&root);
    assert!(!composition(&root, &["--out", "anchor/client"])
        .status
        .success());
    assert_eq!(snapshot(&root), before);
    fs::write(root.join("anchor/unowned"), "keep").unwrap();
    let before = snapshot(&root);
    assert!(
        !composition(&root, &["--client-rust-out", "anchor/unowned"])
            .status
            .success()
    );
    assert_eq!(snapshot(&root), before);
    assert!(
        !composition(&root, &["--out", "anchor/missing/companion.json"])
            .status
            .success()
    );
    assert_eq!(snapshot(&root), before);
    assert!(!composition(
        &root,
        &["--out", "anchor/A", "--client-plan-out", "anchor/a"]
    )
    .status
    .success());
    assert_eq!(snapshot(&root), before);
}

#[test]
fn compose_does_not_discard_a_nondirectory_before_parent_resolution() {
    let root = fixture();
    fs::create_dir(root.join("anchor")).unwrap();
    fs::write(root.join("anchor/file"), "authored").unwrap();
    let before = snapshot(&root);
    assert!(
        !composition(&root, &["--client-rust-out", "anchor/file/../client"])
            .status
            .success()
    );
    assert_eq!(snapshot(&root), before);
}

#[test]
fn adoption_file_selector_preserves_exact_native_component_spelling() {
    let root = fixture();
    fs::write(
        root.join("record.schema.json"),
        r#"{"$id":"urn:record","type":"string"}"#,
    )
    .unwrap();
    assert!(cli(
        &root,
        &[
            "schema",
            "typescript",
            "--schemas",
            "record.schema.json",
            "urn:record",
            "--root",
            "Record",
            "--out",
            "reference/record.ts"
        ]
    )
    .status
    .success());
    fs::create_dir(root.join("target")).unwrap();
    fs::copy(
        root.join("reference/record.ts"),
        root.join("target/record.ts"),
    )
    .unwrap();
    for name in [
        "record.ts/",
        "record.ts//",
        "record.ts/.",
        "./record.ts",
        "part/../record.ts",
    ] {
        let before = snapshot(&root);
        assert!(!cli(
            &root,
            &[
                "output",
                "adopt",
                "--ownership-root",
                "target",
                "--from",
                "reference",
                "--owner",
                "typescript-file",
                "--file",
                name
            ]
        )
        .status
        .success());
        assert_eq!(snapshot(&root), before);
    }
    assert!(cli(
        &root,
        &[
            "output",
            "adopt",
            "--ownership-root",
            "target",
            "--from",
            "reference",
            "--owner",
            "typescript-file",
            "--file",
            "record.ts"
        ]
    )
    .status
    .success());
}
