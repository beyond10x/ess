use super::{ownership, serial, snapshot, Fixture};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

fn permission_snapshot(root: &Path) -> BTreeMap<PathBuf, (u32, Vec<u8>)> {
    std::iter::once((PathBuf::new(), Vec::new()))
        .chain(snapshot(root))
        .map(|(path, bytes)| {
            let mode = fs::symlink_metadata(root.join(&path))
                .unwrap()
                .permissions()
                .mode()
                & 0o7777;
            (path, (mode, bytes))
        })
        .collect()
}

fn permission_cli(args: &[&str]) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap();
    println!(
        "CLI {args:?}: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn read_only_unselected_owner_subtree_needs_no_write_permission_even_with_a_missing_file() {
    let _serial = serial();
    let model = super::workspace().join("examples/billing");
    for present in [true, false] {
        let f = Fixture::new();
        let root = f.0.join("target");
        assert!(permission_cli(&[
            "generate",
            "--path",
            model.to_str().unwrap(),
            "--out",
            root.to_str().unwrap(),
        ])
        .status
        .success());
        let untouched = root.join("schema");
        let file = untouched.join("types/billing.invoice.AccountId.schema.json");
        if !present {
            fs::remove_file(&file).unwrap();
        }
        fs::set_permissions(&untouched, fs::Permissions::from_mode(0o555)).unwrap();
        fs::set_permissions(untouched.join("types"), fs::Permissions::from_mode(0o555)).unwrap();
        let before = permission_snapshot(&untouched);
        let result = permission_cli(&[
            "generate",
            "--path",
            model.to_str().unwrap(),
            "--kind",
            "site",
            "--out",
            root.to_str().unwrap(),
        ]);
        assert_eq!(permission_snapshot(&untouched), before);
        assert_eq!(file.exists(), present);
        assert!(result.status.success(), "present={present}: {result:?}");
        assert!(root.join("index.html").is_file());
    }
}

#[test]
fn read_only_matching_adoption_subtree_needs_no_write_permission_even_with_a_missing_file() {
    let _serial = serial();
    let model = super::workspace().join("examples/billing");
    for present in [true, false] {
        let f = Fixture::new();
        let reference = f.0.join("reference");
        assert!(permission_cli(&[
            "generate",
            "--path",
            model.to_str().unwrap(),
            "--kind",
            "site",
            "--out",
            reference.to_str().unwrap(),
        ])
        .status
        .success());
        let root = f.0.join("target");
        fs::create_dir(&root).unwrap();
        for path in snapshot(&reference)
            .keys()
            .filter(|p| !p.starts_with(".ess-output"))
        {
            let source = reference.join(path);
            if source.is_dir() {
                fs::create_dir_all(root.join(path)).unwrap();
            } else {
                fs::copy(source, root.join(path)).unwrap();
            }
        }
        let file = root.join("assets/style.css");
        if !present {
            fs::remove_file(&file).unwrap();
        }
        fs::set_permissions(root.join("assets"), fs::Permissions::from_mode(0o555)).unwrap();
        let before = permission_snapshot(&root);
        let result = permission_cli(&[
            "output",
            "adopt",
            "--ownership-root",
            root.to_str().unwrap(),
            "--from",
            reference.to_str().unwrap(),
            "--owner",
            "projection:site",
        ]);
        let visible = permission_snapshot(&root)
            .into_iter()
            .filter(|(p, _)| !p.starts_with(".ess-output"))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(visible, before);
        assert_eq!(file.exists(), present);
        assert!(result.status.success(), "present={present}: {result:?}");
        assert!(root.join(".ess-output/state.json").is_file());
    }
}

fn refused(
    root: &Path,
    observer: &mut dyn FnMut(&str) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let path = format!("missing/deeper/{}", "x".repeat(256));
    ownership::probe::publish_admission(
        root,
        &[
            ("aa", "probe written before native refusal"),
            (&path, "bad"),
        ],
        observer,
    )
}

#[test]
fn every_refusal_cleanup_process_and_io_cut_retains_only_exact_admission_orphans() {
    let _serial = serial();
    let control = Fixture::new();
    let root = control.0.join("target");
    fs::create_dir(&root).unwrap();
    let before = snapshot(&control.0);
    let mut trace = Vec::new();
    refused(&root, &mut |event| {
        trace.push(event.to_owned());
        Ok(())
    })
    .unwrap_err();
    assert_eq!(snapshot(&control.0), before);
    let cuts = trace
        .iter()
        .enumerate()
        .filter(|(_, event)| event.contains("remove:") || event.contains("removed-entry-parent"))
        .collect::<Vec<_>>();
    assert!(!cuts.is_empty());
    for (cut, event) in &cuts {
        for process in [false, true] {
            let f = Fixture::new();
            let root = f.0.join("target");
            fs::create_dir(&root).unwrap();
            let before = snapshot(&f.0);
            if process {
                let output = Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "ownership_admission::native_refusal_process_driver",
                        "--nocapture",
                    ])
                    .env("ESS_OWNERSHIP_ADMISSION_ROOT", &root)
                    .env("ESS_OWNERSHIP_ADMISSION_CUT", cut.to_string())
                    .output()
                    .unwrap();
                assert_eq!(output.status.code(), Some(93), "{cut} {event}: {output:?}");
            } else {
                let mut seen = 0;
                let result = refused(&root, &mut |_| {
                    let current = seen;
                    seen += 1;
                    if current == *cut {
                        Err(std::io::Error::from_raw_os_error(5).into())
                    } else {
                        Ok(())
                    }
                });
                assert!(result.is_err());
            }
            let after = snapshot(&f.0);
            preserved(&before, &after);
            for _ in 0..2 {
                ownership::recover(&root).unwrap();
                assert_eq!(snapshot(&f.0), after);
            }
        }
    }
    println!(
        "{} refusal cleanup process cuts and IO boundaries",
        cuts.len()
    );
}

#[test]
fn native_refusal_process_driver() {
    let _serial = serial();
    if let Some(root) = std::env::var_os("ESS_OWNERSHIP_ADMISSION_ROOT") {
        let cut = std::env::var("ESS_OWNERSHIP_ADMISSION_CUT")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let mut seen = 0;
        let result = refused(Path::new(&root), &mut |_| {
            let current = seen;
            seen += 1;
            if current == cut {
                std::process::exit(93);
            }
            Ok(())
        });
        panic!("requested process cut was never reached: {result:?}");
    } else {
        let f = Fixture::new();
        let root = f.0.join("target");
        fs::create_dir(&root).unwrap();
        let before = snapshot(&f.0);
        assert!(refused(&root, &mut |_| Ok(())).is_err());
        assert_eq!(snapshot(&f.0), before);
    }
}

#[test]
fn unicode_parent_other_owner_and_adoption_names_follow_actual_native_distinctions() {
    let _serial = serial();
    for (first, second) in [("É", "é"), ("é", "e\u{301}")] {
        let f = Fixture::new();
        let native = f.0.join("native-pair");
        fs::create_dir(&native).unwrap();
        fs::write(native.join(first), "native witness").unwrap();
        let aliases = native.join(second).exists();
        println!("independent native pair {first:?}/{second:?}: aliases={aliases}");

        let parents = f.0.join("parents");
        fs::create_dir(&parents).unwrap();
        let before = snapshot(&parents);
        let a = format!("{first}/one");
        let b = format!("{second}/two");
        let result = native_publish(&parents, &[(&a, "one"), (&b, "two")]);
        if aliases {
            assert!(result.is_err());
            assert_eq!(snapshot(&parents), before);
        } else {
            result.unwrap();
            assert_eq!(fs::read(parents.join(&a)).unwrap(), b"one");
            assert_eq!(fs::read(parents.join(&b)).unwrap(), b"two");
        }

        for present in [true, false] {
            let root = f.0.join(format!("other-owner-{present}"));
            ownership::publish(
                &root,
                vec![ownership::Publication::named(
                    "typescript-file",
                    std::ffi::OsStr::new(first),
                    "other owner",
                )
                .unwrap()],
            )
            .unwrap();
            if !present {
                fs::remove_file(root.join(first)).unwrap();
            }
            let before = snapshot(&root);
            let result = native_publish(&root, &[(second, "new owner")]);
            if aliases {
                assert!(result.is_err());
                assert_eq!(snapshot(&root), before);
            } else {
                result.unwrap();
                assert_eq!(fs::read(root.join(second)).unwrap(), b"new owner");
                if present {
                    assert_eq!(fs::read(root.join(first)).unwrap(), b"other owner");
                } else {
                    assert!(!root.join(first).exists());
                }
            }
        }

        let reference = f.0.join("reference");
        native_publish(&reference, &[(first, "reference")]).unwrap();
        let legacy = f.0.join("legacy");
        fs::create_dir(&legacy).unwrap();
        fs::write(legacy.join(second), "reference").unwrap();
        let before = snapshot(&legacy);
        let result = ownership::adopt(&legacy, &reference, "compose", None);
        if aliases {
            assert!(result.is_err());
            assert_eq!(snapshot(&legacy), before);
        } else {
            result.unwrap();
            assert_eq!(fs::read(legacy.join(second)).unwrap(), b"reference");
            native_publish(&legacy, &[(first, "generated")]).unwrap();
            assert_eq!(fs::read(legacy.join(first)).unwrap(), b"generated");
            assert_eq!(fs::read(legacy.join(second)).unwrap(), b"reference");
        }
        fs::rename(reference.join(first), reference.join(second)).unwrap();
        let actual_spelling_changed = fs::read_dir(&reference)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .any(|name| name == std::ffi::OsStr::new(second));
        let target = f.0.join("changed-reference");
        fs::create_dir(&target).unwrap();
        fs::write(target.join(first), "reference").unwrap();
        let before = snapshot(&target);
        let result = ownership::adopt(&target, &reference, "compose", None);
        if actual_spelling_changed {
            assert!(
                result.is_err(),
                "renamed reference spelling was admitted under a stale ledger"
            );
            assert_eq!(snapshot(&target), before);
        } else {
            // Some native filesystems leave an equivalent-spelling rename unchanged.
            assert!(aliases);
            result.unwrap();
        }
        println!("reference rename {first:?}->{second:?}: actual spelling changed={actual_spelling_changed}");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn native_parent_and_probe_inode_flags_are_measured_on_assigned_filesystems() {
    let _serial = serial();
    let f = Fixture::new();
    let mut roots = vec![f.0.join("inheritance")];
    if let Some(tmpfs) = std::env::var_os("ESS_BROWSER_TMPDIR") {
        roots.push(
            PathBuf::from(tmpfs).join(format!("ownership-inheritance-{}", std::process::id())),
        );
    }
    for root in roots {
        fs::create_dir(&root).unwrap();
        let parent = fs::File::open(&root).unwrap();
        fs::create_dir(root.join("child")).unwrap();
        let child = fs::File::open(root.join("child")).unwrap();
        let a = rustix::fs::ioctl_getflags(&parent);
        let b = rustix::fs::ioctl_getflags(&child);
        println!(
            "actual native flags {}: parent={a:?}; child={b:?}",
            root.display()
        );
        match (a, b) {
            (Ok(a), Ok(b)) => assert_eq!(a.bits() & 0x4000_0000, b.bits() & 0x4000_0000),
            (Err(a), Err(b)) => {
                assert_eq!(a, b);
                assert!(matches!(
                    a,
                    rustix::io::Errno::NOTTY | rustix::io::Errno::NOTSUP
                ));
            }
            mismatch => panic!("different native inheritance query outcomes: {mismatch:?}"),
        }
        ownership::named(
            &root.join("child/native-λ\\name"),
            "typescript-file",
            "preserved",
        )
        .unwrap();
        assert_eq!(
            fs::read(root.join("child/native-λ\\name")).unwrap(),
            b"preserved"
        );
    }
}
fn native_publish(root: &Path, files: &[(&str, &str)]) -> anyhow::Result<()> {
    ownership::publish(
        root,
        vec![ownership::Publication::compose(
            files
                .iter()
                .map(|(name, bytes)| (PathBuf::from(name), *bytes)),
        )?],
    )
}
const NEXT: &[(&str, &str)] = &[
    ("same", "replacement"),
    ("new/leaf", "introduced"),
    ("shape/child", "now a directory"),
];

fn prepare(kind: &str) -> (Fixture, PathBuf) {
    let f = Fixture::new();
    let root = f.0.join("target");
    match kind {
        "nested" => {
            ownership::probe::publish(
                &root,
                &[("shape/old", "owned old"), ("same", "old")],
                &mut |_| Ok(()),
            )
            .unwrap();
            fs::write(root.join("shape/authored"), "nested authored sentinel").unwrap();
        }
        "adopt" => {
            let reference = f.0.join("reference");
            ownership::probe::publish(&reference, NEXT, &mut |_| Ok(())).unwrap();
            fs::create_dir_all(root.join("shape")).unwrap();
            fs::write(root.join("shape/child"), "now a directory").unwrap();
            fs::write(root.join("shape/authored"), "nested authored sentinel").unwrap();
        }
        "adopt-missing" => {
            let reference = f.0.join("reference");
            ownership::probe::publish(&reference, NEXT, &mut |_| Ok(())).unwrap();
        }
        "missing" => {}
        _ => panic!("unknown fixture"),
    }
    (f, root)
}

fn invoke(
    root: &Path,
    kind: &str,
    observer: &mut dyn FnMut(&str) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    if matches!(kind, "adopt" | "adopt-missing") {
        ownership::probe::adopt_admission(root, &root.parent().unwrap().join("reference"), observer)
    } else {
        ownership::probe::publish_admission(root, NEXT, observer)
    }
}

fn preserved(
    before: &BTreeMap<PathBuf, Vec<u8>>,
    after: &BTreeMap<PathBuf, Vec<u8>>,
) -> Vec<PathBuf> {
    let orphans = after
        .keys()
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            let Some(id) = name.strip_prefix(".ess-output-init-names-") else {
                return false;
            };
            !before.contains_key(*path)
                && id.len() == 36
                && id.bytes().enumerate().all(|(n, b)| {
                    if [8, 13, 18, 23].contains(&n) {
                        b == b'-'
                    } else {
                        b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
                    }
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    let without_exact_orphans = after
        .iter()
        .filter(|(path, _)| !orphans.iter().any(|orphan| path.starts_with(orphan)))
        .map(|(path, bytes)| (path.clone(), bytes.clone()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        &without_exact_orphans, before,
        "admission changed output/state/authored bytes outside its exact private orphans"
    );
    orphans
}

#[test]
fn existing_only_adoption_retains_its_files_and_absences_without_any_admission_mutation() {
    let _serial = serial();
    // This is the original adoption cut fixture. Its artifacts already exist or will
    // remain absent, so the permission correction deliberately removes its probe writes.
    let (f, root) = prepare("adopt");
    let before = snapshot(&f.0);
    let mut events = Vec::new();
    invoke(&root, "adopt", &mut |event| {
        events.push(event.to_owned());
        anyhow::bail!("existing-only adoption attempted admission mutation");
    })
    .unwrap();
    assert!(events.is_empty());
    let after = snapshot(&f.0);
    let visible = after
        .iter()
        .filter(|(path, _)| !path.starts_with("target/.ess-output"))
        .map(|(path, bytes)| (path.clone(), bytes.clone()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(visible, before);
    assert!(!root.join("same").exists());
    assert!(!root.join("new").exists());
    assert!(root.join(".ess-output/state.json").is_file());
    for _ in 0..2 {
        ownership::recover(&root).unwrap();
        assert_eq!(snapshot(&f.0), after);
    }
    println!("original existing-only adoption fixture: 0 admission mutations; all artifact bytes/absences retained");
}

#[test]
fn every_native_admission_process_and_io_boundary_preserves_outputs_state_and_exact_orphans() {
    let _serial = serial();
    for kind in ["nested", "missing", "adopt-missing"] {
        let (_control, root) = prepare(kind);
        let mut trace = Vec::new();
        invoke(&root, kind, &mut |event| {
            trace.push(event.to_owned());
            Ok(())
        })
        .unwrap();
        for required in [
            "after:create:admission-name",
            "before:remove:admission-name",
            "after:sync:removed-entry-parent",
        ] {
            assert!(
                trace.iter().any(|event| event == required),
                "{kind}: {required}"
            );
        }
        let wrote_probe_file = trace
            .iter()
            .any(|event| event == "after:write:admission-name");
        assert_eq!(wrote_probe_file, kind != "adopt-missing",
            "publication must exercise probe file writes; missing-anchor adoption creates only its directory");
        let mut retained_nested = false;
        for (cut, event) in trace.iter().enumerate() {
            for process in [false, true] {
                let (f, root) = prepare(kind);
                let before = snapshot(&f.0);
                if process {
                    let output = Command::new(std::env::current_exe().unwrap())
                        .args([
                            "--exact",
                            "ownership_protocol::native_process_driver",
                            "--nocapture",
                        ])
                        .env("ESS_OWNERSHIP_TEST_ROOT", &root)
                        .env(
                            "ESS_OWNERSHIP_TEST_ACTION",
                            if kind == "adopt-missing" {
                                "admit-adopt"
                            } else {
                                "admit-publish"
                            },
                        )
                        .env("ESS_OWNERSHIP_TEST_CUT", cut.to_string())
                        .output()
                        .unwrap();
                    assert_eq!(
                        output.status.code(),
                        Some(93),
                        "{kind}: {cut} {event}: {output:?}"
                    );
                } else {
                    let mut seen = 0;
                    let result = invoke(&root, kind, &mut |_| {
                        let current = seen;
                        seen += 1;
                        if current == cut {
                            Err(std::io::Error::from_raw_os_error(5).into())
                        } else {
                            Ok(())
                        }
                    });
                    assert!(result.is_err(), "{kind}: {cut} {event}");
                }
                let after = snapshot(&f.0);
                let orphans = preserved(&before, &after);
                retained_nested |= orphans.iter().any(|path| path.starts_with("target/shape"));
                for _ in 0..2 {
                    if root.exists() {
                        ownership::recover(&root).unwrap();
                    } else {
                        assert!(ownership::recover(&root).is_err());
                    }
                    assert_eq!(
                        snapshot(&f.0),
                        after,
                        "recovery changed admission orphan inventory: {kind} {cut} {event}"
                    );
                }
            }
        }
        if kind == "nested" {
            assert!(retained_nested, "no actual nested orphan witnessed");
        }
        println!(
            "{kind}: {} native admission process cuts and IO boundaries",
            trace.len()
        );
    }
}

#[test]
fn native_name_refusal_cleans_private_probes_and_preserves_cause_if_cleanup_fails() {
    let _serial = serial();
    let f = Fixture::new();
    let root = f.0.join("target");
    fs::create_dir(&root).unwrap();
    let name = "x".repeat(256);
    let native = fs::write(f.0.join(&name), b"independent name limit probe").unwrap_err();
    let path = format!("missing/deeper/{name}");
    let before = snapshot(&f.0);
    let refusal =
        ownership::probe::publish_admission(&root, &[(&path, "new")], &mut |_| Ok(())).unwrap_err();
    assert!(
        refusal.chain().any(|cause| cause
            .downcast_ref::<rustix::io::Errno>()
            .is_some_and(|error| Some(error.raw_os_error()) == native.raw_os_error())),
        "{refusal:#}"
    );
    assert_eq!(snapshot(&f.0), before);
    let refusal = ownership::probe::publish_admission(&root, &[(&path, "new")], &mut |event| {
        if event == "before:remove:admission-name" {
            anyhow::bail!("injected refusal cleanup failure");
        }
        Ok(())
    })
    .unwrap_err();
    assert!(
        refusal.chain().any(|cause| cause
            .downcast_ref::<rustix::io::Errno>()
            .is_some_and(|error| Some(error.raw_os_error()) == native.raw_os_error())),
        "{refusal:#}"
    );
    assert!(format!("{refusal:#}").contains("injected refusal cleanup failure"));
    let after = snapshot(&f.0);
    assert!(!preserved(&before, &after).is_empty());
    ownership::recover(&root).unwrap();
    assert_eq!(snapshot(&f.0), after);
}

#[test]
fn noncanonical_or_aliased_admission_orphan_names_still_refuse_without_cleanup() {
    let _serial = serial();
    for name in [
        ".ess-output-init-names-invalid",
        ".ESS-OUTPUT-INIT-NAMES-00000000-0000-0000-0000-000000000000",
        ".ess-output",
    ] {
        let f = Fixture::new();
        let root = f.0.join("target");
        fs::create_dir_all(root.join("nested").join(name)).unwrap();
        let before = snapshot(&f.0);
        assert!(ownership::recover(&root).is_err());
        assert_eq!(snapshot(&f.0), before);
    }
}
