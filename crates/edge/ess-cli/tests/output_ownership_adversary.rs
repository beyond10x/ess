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

fn pass2_fixture() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "e19-adversary-pass2-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    println!("retained pass2 fixture: {}", root.display());
    root
}

#[test]
fn interrupted_nested_admission_stays_opaque_through_cli_retirement() {
    let root = pass2_fixture();
    let anchor = root.join("anchor");
    fs::create_dir(&anchor).unwrap();
    assert!(composition(&root, &["--client-rust-out", "anchor/client"])
        .status
        .success());
    let before = snapshot(&anchor);
    let mut reached = false;
    let result = ownership::probe::publish_admission(
        &anchor,
        &[("client/new-synthesis-file", "never published")],
        &mut |event| {
            if event == "after:mkdir:admission-namespace" {
                reached = true;
                anyhow::bail!("interrupt after creating the nested admission namespace");
            }
            Ok(())
        },
    );
    println!("nested admission interruption: {result:?}");
    assert!(reached && result.is_err());
    let orphans: Vec<_> = fs::read_dir(anchor.join("client"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".ess-output-init-names-")
        })
        .collect();
    assert_eq!(orphans.len(), 1);
    let orphan = &orphans[0];
    let relative = orphan.strip_prefix(&anchor).unwrap();
    assert_eq!(
        snapshot(&anchor)
            .into_iter()
            .filter(|(path, _)| !path.starts_with(relative))
            .collect::<BTreeMap<_, _>>(),
        before,
        "admission interruption changed a pre-existing entry"
    );
    // Opaque bytes can resemble another enrollment; their spelling grants no authority.
    fs::create_dir(orphan.join(".ess-output")).unwrap();
    fs::write(
        orphan.join(".ess-output/state.json"),
        b"opaque invalid state\n",
    )
    .unwrap();
    let orphan_before = snapshot(orphan);
    assert!(composition(&root, &["--client-rust-out", "anchor/client"])
        .status
        .success());
    assert!(composition(&root, &["--out", "anchor/final.json"])
        .status
        .success());
    for retired in ["Cargo.toml", "src", "ess-client-plan.json"] {
        assert!(!anchor.join("client").join(retired).exists());
    }
    assert_eq!(snapshot(orphan), orphan_before);
    assert!(!anchor.join("client/new-synthesis-file").exists());
    let settled = snapshot(&anchor);
    for _ in 0..2 {
        assert!(
            command(&root, &["output", "recover", "--ownership-root", "anchor"])
                .status
                .success()
        );
        assert_eq!(snapshot(&anchor), settled);
    }
    assert!(!composition(&root, &["--out", "anchor/client"])
        .status
        .success());
    assert_eq!(snapshot(&anchor), settled);
}

#[test]
fn native_alias_refusal_preserves_the_owned_file_before_a_shape_transition() {
    for (first, second) in [("É.json", "é.json"), ("é.json", "e\u{301}.json")] {
        let root = pass2_fixture();
        fs::create_dir(root.join("lookup")).unwrap();
        fs::write(root.join("lookup").join(first), "native witness").unwrap();
        let aliases = root.join("lookup").join(second).try_exists().unwrap();
        println!("native transition pair {first:?}/{second:?}: aliases={aliases}");
        fs::create_dir(root.join("anchor")).unwrap();
        fs::write(root.join("anchor/authored"), "preserve neighbor").unwrap();
        assert!(composition(&root, &["--out", "anchor/client"])
            .status
            .success());
        fs::write(root.join("anchor/client"), "edited owned preimage").unwrap();
        fs::set_permissions(
            root.join("anchor/client"),
            fs::Permissions::from_mode(0o400),
        )
        .unwrap();
        let before = snapshot(&root.join("anchor"));
        let first = format!("anchor/{first}");
        let second = format!("anchor/{second}");
        let result = composition(
            &root,
            &[
                "--client-rust-out",
                "anchor/client",
                "--out",
                &first,
                "--client-plan-out",
                &second,
            ],
        );
        if aliases {
            assert!(!result.status.success());
            assert_eq!(snapshot(&root.join("anchor")), before);
        } else {
            assert!(result.status.success());
            assert!(root.join("anchor/client/src/lib.rs").is_file());
            assert_ne!(
                fs::read(root.join(first)).unwrap(),
                fs::read(root.join(second)).unwrap()
            );
            assert_eq!(
                fs::read(root.join("anchor/authored")).unwrap(),
                b"preserve neighbor"
            );
        }
        let settled = snapshot(&root.join("anchor"));
        for _ in 0..2 {
            assert!(
                command(&root, &["output", "recover", "--ownership-root", "anchor"])
                    .status
                    .success()
            );
            assert_eq!(snapshot(&root.join("anchor")), settled);
        }
    }
}

#[test]
fn exact_native_file_adoption_in_an_enrolled_readonly_root_needs_no_probe_write() {
    let root = pass2_fixture();
    fs::create_dir(root.join("reference")).unwrap();
    fs::create_dir(root.join("anchor")).unwrap();
    fs::write(
        root.join("record.schema.json"),
        r#"{"$id":"urn:record","type":"string"}"#,
    )
    .unwrap();
    let name = format!("{}.ts", "é".repeat(126));
    assert_eq!(name.len(), 255);
    for output in [format!("reference/{name}"), "anchor/other.ts".to_owned()] {
        assert!(command(
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
                &output,
            ],
        )
        .status
        .success());
    }
    fs::copy(
        root.join("reference").join(&name),
        root.join("anchor").join(&name),
    )
    .unwrap();
    fs::set_permissions(
        root.join("anchor").join(&name),
        fs::Permissions::from_mode(0o400),
    )
    .unwrap();
    fs::write(root.join("anchor/authored"), "untouched").unwrap();
    fs::set_permissions(root.join("anchor"), fs::Permissions::from_mode(0o555)).unwrap();
    let before = visible(&root.join("anchor"));
    let reference = snapshot(&root.join("reference"));
    let result = command(
        &root,
        &[
            "output",
            "adopt",
            "--ownership-root",
            "anchor",
            "--from",
            "reference",
            "--owner",
            "typescript-file",
            "--file",
            &name,
        ],
    );
    assert!(result.status.success());
    assert_eq!(visible(&root.join("anchor")), before);
    assert_eq!(snapshot(&root.join("reference")), reference);
    assert_eq!(
        fs::metadata(root.join("anchor"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o555
    );
    let settled = snapshot(&root.join("anchor"));
    assert!(command(
        &root,
        &[
            "output",
            "adopt",
            "--ownership-root",
            "anchor",
            "--from",
            "reference",
            "--owner",
            "typescript-file",
            "--file",
            &name,
        ],
    )
    .status
    .success());
    assert_eq!(snapshot(&root.join("anchor")), settled);
    for _ in 0..2 {
        assert!(
            command(&root, &["output", "recover", "--ownership-root", "anchor"])
                .status
                .success()
        );
        assert_eq!(snapshot(&root.join("anchor")), settled);
    }
}
