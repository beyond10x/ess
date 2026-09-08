//! Additional native ownership contract witnesses; all production code remains unchanged.
#[allow(dead_code)]
#[path = "../src/output_ownership/mod.rs"]
mod ownership;

use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn fixture() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "e19-adversary-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    println!("retained fixture: {}", root.display());
    root
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("examples/billing").is_dir())
        .unwrap()
        .to_path_buf()
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, (u32, Option<Vec<u8>>)> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, (u32, Option<Vec<u8>>)>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            assert!(!metadata.file_type().is_symlink());
            out.insert(
                path.strip_prefix(root).unwrap().to_path_buf(),
                (
                    metadata.permissions().mode() & 0o7777,
                    metadata.is_file().then(|| fs::read(&path).unwrap()),
                ),
            );
            if metadata.is_dir() {
                visit(root, &path, out);
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

fn visible(root: &Path) -> BTreeMap<PathBuf, (u32, Option<Vec<u8>>)> {
    snapshot(root)
        .into_iter()
        .filter(|(p, _)| !p.starts_with(".ess-output"))
        .collect()
}

fn command(root: &Path, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    println!(
        "CLI {args:?}: status={}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
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
    command(root, &args)
}

#[test]
fn standalone_generation_refuses_directory_spelling_before_enrollment() {
    let root = fixture();
    fs::create_dir(root.join("out")).unwrap();
    fs::write(
        root.join("record.schema.json"),
        r#"{"$id":"urn:record","type":"string"}"#,
    )
    .unwrap();
    fs::write(root.join("out/authored"), "preserve authored neighbor").unwrap();
    let before = snapshot(&root);
    let output = command(
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
            "out/record.ts/",
        ],
    );
    assert!(
        !output.status.success(),
        "a directory-spelled destination was silently published as a file"
    );
    assert_eq!(
        snapshot(&root),
        before,
        "refusal must precede output enrollment"
    );
}

#[test]
fn composition_replaces_its_owned_companion_with_a_client_directory() {
    let root = fixture();
    fs::create_dir(root.join("anchor")).unwrap();
    fs::write(root.join("anchor/authored"), "keep").unwrap();
    let first = composition(&root, &["--out", "anchor/client"]);
    assert!(first.status.success());
    assert!(root.join("anchor/client").is_file());
    let next = composition(&root, &["--client-rust-out", "anchor/client"]);
    assert!(
        next.status.success(),
        "the selected compose owner's file must be replaceable by its complete client tree"
    );
    for name in ["Cargo.toml", "src/lib.rs", "ess-client-plan.json"] {
        assert!(root.join("anchor/client").join(name).is_file(), "{name}");
    }
    assert_eq!(fs::read(root.join("anchor/authored")).unwrap(), b"keep");
}

#[test]
fn unicode_companions_follow_actual_native_alias_behavior_before_any_write() {
    let root = fixture();
    fs::create_dir(root.join("lookup")).unwrap();
    fs::write(root.join("lookup/É.json"), "native lookup witness").unwrap();
    let aliases = root.join("lookup/é.json").try_exists().unwrap();
    println!("native É.json/é.json alias lookup: {aliases}");
    fs::create_dir(root.join("anchor")).unwrap();
    let before = snapshot(&root.join("anchor"));
    let output = composition(
        &root,
        &[
            "--out",
            "anchor/É.json",
            "--client-plan-out",
            "anchor/é.json",
        ],
    );
    if aliases {
        assert!(!output.status.success(), "aliased outputs must refuse");
        assert_eq!(
            snapshot(&root.join("anchor")),
            before,
            "native output aliases must refuse before publishing state or a companion"
        );
    } else {
        assert!(output.status.success());
        assert_ne!(
            fs::read(root.join("anchor/É.json")).unwrap(),
            fs::read(root.join("anchor/é.json")).unwrap()
        );
        let settled = snapshot(&root.join("anchor"));
        ownership::recover(&root.join("anchor")).unwrap();
        assert_eq!(snapshot(&root.join("anchor")), settled);
    }
}

#[test]
fn rollback_preserves_an_unselected_owner_and_actual_readonly_file_modes() {
    let root = fixture();
    let anchor = root.join("anchor");
    ownership::probe::publish(&anchor, &[("shared/a", "old")], &mut |_| Ok(())).unwrap();
    ownership::publish(
        &anchor,
        vec![ownership::Publication::tree("model-types", [("shared/b", "other")]).unwrap()],
    )
    .unwrap();
    fs::write(anchor.join("shared/a"), "edited actual preimage").unwrap();
    fs::set_permissions(anchor.join("shared/a"), fs::Permissions::from_mode(0o400)).unwrap();
    fs::write(anchor.join("shared/authored"), "preserve").unwrap();
    let before = visible(&anchor);
    let result = ownership::probe::publish(
        &anchor,
        &[("shared/a", "replacement"), ("new/leaf", "new")],
        &mut |event| {
            if event == "after:rename:output-installation" {
                anyhow::bail!("injected process-equivalent interruption after first install");
            }
            Ok(())
        },
    );
    assert!(result.is_err());
    println!("first installation interruption: {result:?}");
    ownership::recover(&anchor).unwrap();
    assert_eq!(visible(&anchor), before);
    let settled = snapshot(&anchor);
    ownership::recover(&anchor).unwrap();
    assert_eq!(snapshot(&anchor), settled);
    ownership::probe::publish(&anchor, &[], &mut |_| Ok(())).unwrap();
    assert!(!anchor.join("shared/a").exists());
    assert_eq!(fs::read(anchor.join("shared/b")).unwrap(), b"other");
    assert_eq!(
        fs::read(anchor.join("shared/authored")).unwrap(),
        b"preserve"
    );
}
