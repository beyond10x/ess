use super::{ownership, snapshot, Fixture};
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const OLD: &[(&str, &str)] = &[
    ("same", "original"),
    ("retired/old", "withdrawn"),
    ("shape", "was a file"),
];
const NEW: &[(&str, &str)] = &[
    ("same", "replacement"),
    ("new/leaf", "introduced"),
    ("shape/child", "now a directory"),
];
fn publish(root: &Path, files: &[(&str, &str)]) {
    ownership::probe::publish(root, files, &mut |_| Ok(())).unwrap();
}
fn prepare() -> (Fixture, PathBuf) {
    let f = Fixture::new();
    let root = f.0.join("transaction");
    publish(&root, OLD);
    fs::write(root.join("same"), "actual edited preimage").unwrap();
    fs::write(root.join("authored"), "preserve authored neighbor").unwrap();
    (f, root)
}
fn visible(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    snapshot(root)
        .into_iter()
        .filter(|(p, _)| {
            !p.components()
                .next()
                .unwrap()
                .as_os_str()
                .to_string_lossy()
                .starts_with(".ess-output")
        })
        .collect()
}
fn phase(root: &Path) -> String {
    serde_json::from_slice::<serde_json::Value>(
        &fs::read(root.join(".ess-output/state.json")).unwrap(),
    )
    .unwrap()["payload"]["checkpoint"]["phase"]
        .as_str()
        .unwrap()
        .to_owned()
}
fn child(root: &Path, action: &str, cut: usize) -> std::process::Output {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "ownership_protocol::native_process_driver",
            "--nocapture",
        ])
        .env("ESS_OWNERSHIP_TEST_ROOT", root)
        .env("ESS_OWNERSHIP_TEST_ACTION", action)
        .env("ESS_OWNERSHIP_TEST_CUT", cut.to_string())
        .output()
        .unwrap()
}
#[test]
fn native_process_driver() {
    let _serial = super::serial();
    if let Some(root) = std::env::var_os("ESS_OWNERSHIP_TEST_ROOT") {
        let root = PathBuf::from(root);
        let cut = std::env::var("ESS_OWNERSHIP_TEST_CUT")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let mut count = 0;
        let mut observer = |_: &str| -> Result<()> {
            let current = count;
            count += 1;
            if current == cut {
                std::process::exit(93)
            }
            Ok(())
        };
        match std::env::var("ESS_OWNERSHIP_TEST_ACTION").unwrap().as_str() {
            "admit-publish" => {
                ownership::probe::publish_admission(&root, NEW, &mut observer).unwrap();
            }
            "admit-adopt" => ownership::probe::adopt_admission(
                &root,
                &root.parent().unwrap().join("reference"),
                &mut observer,
            )
            .unwrap(),
            "recover" => ownership::probe::recover(&root, &mut observer).unwrap(),
            "adopt" => ownership::probe::adopt(
                &root,
                &root.parent().unwrap().join("reference"),
                &mut observer,
            )
            .unwrap(),
            _ => ownership::probe::publish(&root, NEW, &mut observer).unwrap(),
        }
    } else {
        let (_f, root) = prepare();
        publish(&root, NEW);
        assert_eq!(fs::read(root.join("same")).unwrap(), b"replacement");
    }
}
#[test]
fn every_publication_syscall_boundary_refuses_then_recovers_the_actual_preimage_or_committed_set() {
    let _serial = super::serial();
    let (_control, root) = prepare();
    let old = visible(&root);
    let mut trace = Vec::new();
    ownership::probe::publish(&root, NEW, &mut |event| {
        trace.push(event.to_owned());
        Ok(())
    })
    .unwrap();
    let new = visible(&root);
    assert!(trace.iter().any(|e| e == "after:partial-write:checkpoint"));
    assert!(trace
        .iter()
        .any(|e| e == "after:rename:output-installation"));
    for (cut, event) in trace.iter().enumerate() {
        let (_f, root) = prepare();
        let mut seen = 0;
        let result = ownership::probe::publish(&root, NEW, &mut |_| {
            let current = seen;
            seen += 1;
            if current == cut {
                Err(std::io::Error::from_raw_os_error(5).into())
            } else {
                Ok(())
            }
        });
        assert!(result.is_err(), "fault {cut} {event} did not refuse");
        let committed =
            phase(&root) == "Committed" || phase(&root) == "Idle" && visible(&root) == new;
        if phase(&root) != "Idle" {
            assert!(ownership::check(&root).is_err());
            assert!(ownership::probe::publish(&root, NEW, &mut |_| Ok(())).is_err());
        }
        ownership::recover(&root).unwrap_or_else(|e| panic!("recovery after {cut} {event}: {e:#}"));
        assert_eq!(
            visible(&root),
            if committed { new.clone() } else { old.clone() },
            "fault {cut} {event}"
        );
        let settled = snapshot(&root);
        ownership::recover(&root).unwrap();
        assert_eq!(snapshot(&root), settled);
    }
    println!("{} actual injected publication boundaries", trace.len());
}
#[test]
fn every_publication_process_cut_recovers_without_current_inputs() {
    let _serial = super::serial();
    let (_control, root) = prepare();
    let old = visible(&root);
    let mut count = 0;
    ownership::probe::publish(&root, NEW, &mut |_| {
        count += 1;
        Ok(())
    })
    .unwrap();
    let new = visible(&root);
    for cut in 0..count {
        let (_f, root) = prepare();
        let output = child(&root, "publish", cut);
        assert_eq!(output.status.code(), Some(93), "cut {cut}: {output:?}");
        let committed =
            phase(&root) == "Committed" || phase(&root) == "Idle" && visible(&root) == new;
        ownership::recover(&root)
            .unwrap_or_else(|e| panic!("recovery after process cut {cut}: {e:#}"));
        assert_eq!(
            visible(&root),
            if committed { new.clone() } else { old.clone() },
            "process cut {cut}"
        );
        assert_eq!(phase(&root), "Idle");
    }
    println!("{count} actual publication process cuts");
}
fn interrupt_prepared(root: &Path) {
    let result = ownership::probe::publish(root, NEW, &mut |event| {
        if event == "after:rename:output-installation" {
            anyhow::bail!("cut after installing one changed output")
        }
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(phase(root), "Prepared");
}
#[test]
fn every_recovery_process_cut_and_io_failure_keeps_restoration_repeatable() {
    let _serial = super::serial();
    let (_control, root) = prepare();
    let old = visible(&root);
    interrupt_prepared(&root);
    let mut trace = Vec::new();
    ownership::probe::recover(&root, &mut |event| {
        trace.push(event.to_owned());
        Ok(())
    })
    .unwrap();
    assert_eq!(visible(&root), old);
    assert!(trace
        .iter()
        .any(|e| e == "after:partial-write:restore-stage"));
    for (cut, event) in trace.iter().enumerate() {
        for process in [false, true] {
            let (_f, root) = prepare();
            interrupt_prepared(&root);
            if process {
                let output = child(&root, "recover", cut);
                assert_eq!(
                    output.status.code(),
                    Some(93),
                    "recovery cut {cut} {event}: {output:?}"
                );
            } else {
                let mut seen = 0;
                assert!(ownership::probe::recover(&root, &mut |_| {
                    let current = seen;
                    seen += 1;
                    if current == cut {
                        Err(std::io::Error::from_raw_os_error(5).into())
                    } else {
                        Ok(())
                    }
                })
                .is_err());
            }
            ownership::recover(&root).unwrap_or_else(|e| {
                panic!("repeated recovery {cut} {event} process={process}: {e:#}")
            });
            assert_eq!(visible(&root), old);
            let settled = snapshot(&root);
            ownership::recover(&root).unwrap();
            assert_eq!(snapshot(&root), settled);
        }
    }
    println!(
        "{} actual recovery process cuts and {} injected recovery boundaries",
        trace.len(),
        trace.len()
    );
}

fn fresh(adoption: bool) -> (Fixture, PathBuf) {
    let f = Fixture::new();
    let root = f.0.join("fresh");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("authored"), "never generated").unwrap();
    if adoption {
        publish(&f.0.join("reference"), NEW);
        for (path, bytes) in NEW {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
    }
    (f, root)
}
#[test]
fn initialization_and_metadata_only_adoption_survive_each_process_cut_and_io_failure() {
    let _serial = super::serial();
    for adoption in [false, true] {
        let (_control, root) = fresh(adoption);
        let old = visible(&root);
        let mut trace = Vec::new();
        let mut observer = |event: &str| {
            trace.push(event.to_owned());
            Ok(())
        };
        if adoption {
            ownership::probe::adopt(
                &root,
                &root.parent().unwrap().join("reference"),
                &mut observer,
            )
            .unwrap();
        } else {
            ownership::probe::publish(&root, NEW, &mut observer).unwrap();
        }
        let new = visible(&root);
        if adoption {
            assert_eq!(old, new);
        }
        assert!(trace
            .iter()
            .any(|e| e == "after:rename:initial-state-publication"));
        for (cut, event) in trace.iter().enumerate() {
            for process in [false, true] {
                let (_f, root) = fresh(adoption);
                if process {
                    let output = child(&root, if adoption { "adopt" } else { "publish" }, cut);
                    assert_eq!(
                        output.status.code(),
                        Some(93),
                        "initial cut {cut} {event}: {output:?}"
                    );
                } else {
                    let mut seen = 0;
                    let mut observer = |_: &str| {
                        let current = seen;
                        seen += 1;
                        if current == cut {
                            Err(std::io::Error::from_raw_os_error(5).into())
                        } else {
                            Ok(())
                        }
                    };
                    let result = if adoption {
                        ownership::probe::adopt(
                            &root,
                            &root.parent().unwrap().join("reference"),
                            &mut observer,
                        )
                    } else {
                        ownership::probe::publish(&root, NEW, &mut observer)
                    };
                    assert!(result.is_err());
                }
                let committed = root.join(".ess-output/state.json").exists()
                    && (phase(&root) == "Committed"
                        || phase(&root) == "Idle" && visible(&root) == new);
                let orphans = fs::read_dir(&root)
                    .unwrap()
                    .map(|e| e.unwrap().path())
                    .filter(|p| {
                        p.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .starts_with(".ess-output-init-")
                    })
                    .map(|p| {
                        let bytes = snapshot(&p);
                        (p, bytes)
                    })
                    .collect::<Vec<_>>();
                ownership::recover(&root).unwrap_or_else(|e| {
                    panic!("initialization/adoption={adoption} cut {cut} {event}: {e:#}")
                });
                assert_eq!(
                    visible(&root),
                    if committed { new.clone() } else { old.clone() }
                );
                for (path, bytes) in orphans {
                    assert_eq!(
                        snapshot(&path),
                        bytes,
                        "unpublished initialization orphan changed"
                    );
                }
            }
        }
        println!(
            "adoption={adoption}: {} native process cuts and IO boundaries",
            trace.len()
        );
    }
}

#[test]
fn adoption_is_exact_metadata_only_and_never_shrinks_an_enrolled_owner() {
    let _serial = super::serial();
    let f = Fixture::new();
    let reference = f.0.join("reference");
    publish(&reference, OLD);
    let target = f.0.join("legacy");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("same"), "original").unwrap();
    fs::write(target.join("extra.html"), "false generated provenance").unwrap();
    let before = visible(&target);
    ownership::adopt(&target, &reference, "synthesis", None).unwrap();
    assert_eq!(visible(&target), before);
    let settled = snapshot(&target);
    ownership::adopt(&target, &reference, "synthesis", None).unwrap();
    assert_eq!(snapshot(&target), settled);
    publish(&reference, &[]);
    assert!(ownership::adopt(&target, &reference, "synthesis", None).is_err());
    assert_eq!(snapshot(&target), settled);
    publish(&target, &[]);
    assert!(!target.join("same").exists());
    assert_eq!(
        fs::read(target.join("extra.html")).unwrap(),
        b"false generated provenance"
    );
    assert!(target.join(".ess-output/state.json").exists());
    let before = snapshot(&target);
    assert!(ownership::adopt(&target, &target, "synthesis", None).is_err());
    assert!(ownership::adopt(&target, &target.join("nested"), "synthesis", None).is_err());
    assert_eq!(snapshot(&target), before);
}

#[test]
fn different_owners_never_claim_or_retire_each_others_files() {
    let _serial = super::serial();
    let f = Fixture::new();
    let root = f.0.join("owners");
    publish(&root, &[("first", "one")]);
    ownership::publish(
        &root,
        vec![ownership::Publication::tree("model-types", [("second", "two")]).unwrap()],
    )
    .unwrap();
    let before = snapshot(&root);
    assert!(ownership::publish(
        &root,
        vec![ownership::Publication::tree("model-types", [("first", "steal")]).unwrap()]
    )
    .is_err());
    assert_eq!(snapshot(&root), before);
    publish(&root, &[]);
    assert!(!root.join("first").exists());
    assert_eq!(fs::read(root.join("second")).unwrap(), b"two");
}

#[test]
fn recorded_directories_change_shape_but_authored_children_and_adopted_directories_survive() {
    let _serial = super::serial();
    let f = Fixture::new();
    let root = f.0.join("shapes");
    publish(&root, &[("node/leaf", "owned")]);
    fs::write(root.join("node/authored"), "unowned child").unwrap();
    let before = snapshot(&root);
    assert!(ownership::probe::publish(&root, &[("node", "file")], &mut |_| Ok(())).is_err());
    assert_eq!(snapshot(&root), before);
    publish(&root, &[]);
    assert_eq!(
        fs::read(root.join("node/authored")).unwrap(),
        b"unowned child"
    );
    assert!(!root.join("node/leaf").exists());
    let another = f.0.join("another");
    publish(&another, &[("node/leaf", "owned")]);
    publish(&another, &[("node", "file")]);
    assert_eq!(fs::read(another.join("node")).unwrap(), b"file");
    publish(&another, &[("node/leaf", "again")]);
    assert_eq!(fs::read(another.join("node/leaf")).unwrap(), b"again");
}

#[test]
fn native_filenames_are_lossless_while_alias_links_and_reserved_paths_refuse() {
    use std::os::unix::{
        ffi::OsStringExt,
        fs::{symlink, MetadataExt},
    };
    let _serial = super::serial();
    let f = Fixture::new();
    let root = f.0.join("native");
    fs::create_dir(&root).unwrap();
    let name = std::ffi::OsString::from_vec(b"native-\xff\\copy.json".to_vec());
    let probe = f.0.join("native-capability");
    fs::create_dir(&probe).unwrap();
    assert_eq!(
        fs::metadata(&probe).unwrap().dev(),
        fs::metadata(&root).unwrap().dev()
    );
    match fs::write(probe.join(&name), b"independent native filename probe") {
        Ok(()) => {
            assert_eq!(
                fs::read(probe.join(&name)).unwrap(),
                b"independent native filename probe"
            );
            println!("native filesystem admitted the exact invalid-UTF8 filename");
            ownership::named(&root.join(&name), "typescript-file", "opaque name").unwrap();
            assert_eq!(fs::read(root.join(&name)).unwrap(), b"opaque name");
            let state = fs::read_to_string(root.join(".ess-output/state.json")).unwrap();
            assert!(state.contains("UnixBytes1"));
            assert!(state.contains("6e61746976652dff5c636f70792e6a736f6e"));
        }
        Err(native) => {
            let eilseq = rustix::io::Errno::ILSEQ.raw_os_error();
            assert_eq!(native.raw_os_error(), Some(eilseq), "{native:?}");
            assert!(snapshot(&probe).is_empty());
            let before = snapshot(&root);
            let refusal =
                ownership::named(&root.join(&name), "typescript-file", "opaque name").unwrap_err();
            assert!(
                refusal.chain().any(|cause| {
                    cause.downcast_ref::<rustix::io::Errno>() == Some(&rustix::io::Errno::ILSEQ)
                        || cause
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(|error| error.raw_os_error() == Some(eilseq))
                }),
                "native={native:?}; ownership={refusal:#}"
            );
            assert_eq!(snapshot(&root), before);
            assert!(!root.join(".ess-output").exists());
            println!("native filename refusal preserved the complete target: {refusal:#}");
        }
    }
    let unicode = "native-λ\\copy.json";
    ownership::named(
        &root.join(unicode),
        "typescript-file",
        "Unicode and backslash",
    )
    .unwrap();
    assert_eq!(
        fs::read(root.join(unicode)).unwrap(),
        b"Unicode and backslash"
    );
    let state = fs::read_to_string(root.join(".ess-output/state.json")).unwrap();
    assert!(state.contains("UnixBytes1"));
    assert!(state.contains("6e61746976652dcebb5c636f70792e6a736f6e"));
    for path in [
        ".ess-output/file",
        ".ESS-OUTPUT/file",
        ".ess-output-init-madeup/file",
    ] {
        let before = snapshot(&root);
        assert!(ownership::probe::publish(&root, &[(path, "bad")], &mut |_| Ok(())).is_err());
        assert_eq!(snapshot(&root), before);
    }
    fs::write(root.join("CASE"), "authored").unwrap();
    assert!(ownership::probe::publish(&root, &[("case", "bad")], &mut |_| Ok(())).is_err());
    let outside = f.0.join("outside");
    fs::write(&outside, "outside").unwrap();
    symlink(&outside, root.join("link")).unwrap();
    fs::hard_link(&outside, root.join("hard")).unwrap();
    for name in ["link", "hard"] {
        assert!(ownership::probe::publish(&root, &[(name, "bad")], &mut |_| Ok(())).is_err());
    }
    assert_eq!(fs::read(outside).unwrap(), b"outside");
}

#[test]
fn directory_locks_serialize_same_nested_and_missing_roots_and_allow_readers() {
    use rustix::fs::{flock, FlockOperation};
    let _serial = super::serial();
    let f = Fixture::new();
    let root = f.0.join("locks");
    fs::create_dir(&root).unwrap();
    let fd = fs::File::open(&root).unwrap();
    flock(&fd, FlockOperation::NonBlockingLockExclusive).unwrap();
    assert!(ownership::probe::publish(&root, &[("a", "b")], &mut |_| Ok(())).is_err());
    assert!(ownership::probe::publish(
        &root.join("missing/nested"),
        &[("a", "b")],
        &mut |_| Ok(())
    )
    .is_err());
    assert!(ownership::check(&root).is_err());
    assert!(!root.join("missing").exists());
    flock(&fd, FlockOperation::Unlock).unwrap();
    let reader = ownership::check(&root).unwrap();
    let another = ownership::check(&root).unwrap();
    assert!(ownership::probe::publish(&root, &[("a", "b")], &mut |_| Ok(())).is_err());
    drop(another);
    drop(reader);
    publish(&root, &[("a", "b")]);
    let before = snapshot(&root);
    assert!(
        ownership::probe::publish(&root.join("nested"), &[("a", "b")], &mut |_| Ok(())).is_err()
    );
    assert_eq!(snapshot(&root), before);
    assert!(ownership::probe::publish(&f.0, &[("outer", "b")], &mut |_| Ok(())).is_err());
}

#[test]
fn actual_native_mount_identity_distinguishes_a_foreign_mount_before_mutation() {
    let _serial = super::serial();
    let f = Fixture::new();
    let before = snapshot(&f.0);
    ownership::probe::check_mount(&f.0, &f.0).unwrap();
    assert!(ownership::probe::check_mount(&f.0, Path::new("/dev")).is_err());
    assert_eq!(snapshot(&f.0), before);
}

fn canonical_state(value: &mut serde_json::Value) -> Vec<u8> {
    use sha2::{Digest as _, Sha256};
    use std::fmt::Write as _;
    let mut payload = serde_json::to_vec(&value["payload"]).unwrap();
    payload.push(b'\n');
    let mut checksum = String::new();
    for byte in Sha256::digest(&payload) {
        write!(&mut checksum, "{byte:02x}").unwrap();
    }
    value["checksum"] = checksum.into();
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}
#[test]
fn canonical_reader_refuses_bad_versions_fields_paths_owners_root_and_checksums_without_cleanup() {
    let _serial = super::serial();
    for case in [
        "version",
        "extra",
        "profile",
        "root",
        "owner",
        "path",
        "mode",
        "checksum",
        "whitespace",
        "duplicate",
    ] {
        let (_f, root) = prepare();
        let state_path = root.join(".ess-output/state.json");
        let original = fs::read(&state_path).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
        match case {
            "version" => value["payload"]["format"] = "ess-output-state/99".into(),
            "extra" => value["payload"]["unrecognized"] = true.into(),
            "profile" => value["payload"]["profile"] = "FutureProfile".into(),
            "root" => value["payload"]["root"]["components"] = serde_json::json!(["657363617065"]),
            "owner" => {
                value["payload"]["checkpoint"]["ledger"]["owners"][0]["key"]["family"] =
                    "invented".into();
            }
            "path" => {
                value["payload"]["checkpoint"]["ledger"]["owners"][0]["files"][0]["path"]
                    ["components"] = serde_json::json!(["2e2e", "657363617065"]);
            }
            "mode" => {
                value["payload"]["checkpoint"]["ledger"]["owners"][0]["files"][0]["data"]["mode"] =
                    4095.into();
            }
            _ => {}
        }
        let mut bytes = canonical_state(&mut value);
        if case == "checksum" {
            bytes[14] = if bytes[14] == b'a' { b'b' } else { b'a' };
        }
        if case == "whitespace" {
            bytes.push(b'\n');
        }
        if case == "duplicate" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replacen(
                    "{\"checksum\":",
                    &format!("{{\"checksum\":{},\"checksum\":", value["checksum"]),
                    1,
                )
                .into_bytes();
        }
        fs::write(&state_path, bytes).unwrap();
        let before = snapshot(&root);
        assert!(ownership::recover(&root).is_err(), "{case}");
        assert!(ownership::check(&root).is_err(), "{case}");
        assert!(
            ownership::probe::publish(&root, NEW, &mut |_| Ok(())).is_err(),
            "{case}"
        );
        assert_eq!(snapshot(&root), before, "{case}");
    }
}
#[test]
fn prepared_state_requires_complete_inventory_exact_preimages_and_no_unexplained_members() {
    let _serial = super::serial();
    for case in [
        "omitted-change",
        "missing-backup",
        "changed-backup",
        "unknown-member",
        "unexpected-output",
    ] {
        let (_f, root) = prepare();
        interrupt_prepared(&root);
        let state_path = root.join(".ess-output/state.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
        let tx = &value["payload"]["checkpoint"]["transaction"];
        let txdir = root
            .join(".ess-output")
            .join(format!("transaction-{}", tx["id"].as_str().unwrap()));
        let backup = tx["changes"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|c| c["backup"]["name"].as_str())
            .unwrap()
            .to_owned();
        match case {
            "omitted-change" => {
                value["payload"]["checkpoint"]["transaction"]["changes"] = serde_json::json!([]);
                fs::write(&state_path, canonical_state(&mut value)).unwrap();
            }
            "missing-backup" => fs::remove_file(txdir.join(backup)).unwrap(),
            "changed-backup" => fs::write(txdir.join(backup), "not preimage").unwrap(),
            "unknown-member" => fs::write(txdir.join("unrecorded"), "preserve evidence").unwrap(),
            "unexpected-output" => fs::write(root.join("same"), "a third set of bytes").unwrap(),
            _ => unreachable!(),
        }
        let before = snapshot(&root);
        assert!(ownership::recover(&root).is_err(), "{case}");
        assert_eq!(snapshot(&root), before, "{case}");
    }
}

#[test]
fn check_and_pending_invalid_new_inputs_never_initialize_or_recover() {
    let _serial = super::serial();
    let f = Fixture::new();
    let missing = f.0.join("not-created/deeper");
    let before = snapshot(&f.0);
    drop(ownership::check(&missing).unwrap());
    assert_eq!(snapshot(&f.0), before);
    let root = f.0.join("pending");
    publish(&root, OLD);
    interrupt_prepared(&root);
    let before = snapshot(&root);
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["generate", "--path"])
        .arg(f.0.join("missing-model"))
        .args(["--out"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(snapshot(&root), before);
    let recovery = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["output", "recover", "--ownership-root"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(recovery.status.success(), "{recovery:?}");
    assert_eq!(phase(&root), "Idle");
}

#[test]
fn a_checksummed_adoption_checkpoint_cannot_drop_an_existing_owners_retirement_authority() {
    let _serial = super::serial();
    let (_f, root) = fresh(true);
    let reference = root.parent().unwrap().join("reference");
    let result = ownership::probe::adopt(&root, &reference, &mut |event| {
        if event == "before:sync:complete-new-output" {
            anyhow::bail!("hold metadata decision")
        }
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(phase(&root), "Prepared");
    let path = root.join(".ess-output/state.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["payload"]["checkpoint"]["phase"] = "Committed".into();
    let tx = &mut value["payload"]["checkpoint"]["transaction"];
    tx["before"] = tx["after"].clone();
    tx["after"]["owners"][0]["files"] = serde_json::json!([]);
    fs::write(path, canonical_state(&mut value)).unwrap();
    let before = snapshot(&root);
    assert!(
        ownership::recover(&root).is_err(),
        "adoption checkpoint silently dropped an enrolled inventory"
    );
    assert_eq!(snapshot(&root), before);
}

#[test]
fn losing_the_locked_anchor_binding_stops_before_installing_output() {
    let _serial = super::serial();
    let (f, root) = prepare();
    let old = visible(&root);
    let moved = f.0.join("moved-anchor");
    let result = ownership::probe::publish(&root, NEW, &mut |event| {
        if event == "after:sync:complete-staging" {
            fs::rename(&root, &moved)?;
        }
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(visible(&moved), old);
    fs::rename(&moved, &root).unwrap();
    ownership::recover(&root).unwrap();
    assert_eq!(visible(&root), old);
}

#[test]
fn adoption_refuses_mismatched_targets_and_dirty_pending_or_corrupt_references() {
    let _serial = super::serial();
    for case in [
        "target-mismatch",
        "dirty-reference",
        "pending-reference",
        "corrupt-reference",
    ] {
        let (f, root) = fresh(true);
        let reference = f.0.join("reference");
        match case {
            "target-mismatch" => fs::write(root.join("same"), "target edit").unwrap(),
            "dirty-reference" => fs::write(reference.join("same"), "reference edit").unwrap(),
            "pending-reference" => {
                let result = ownership::probe::publish(&reference, OLD, &mut |event| {
                    if event == "before:mkdir:transaction-directory" {
                        anyhow::bail!("retained pending reference")
                    }
                    Ok(())
                });
                assert!(result.is_err());
            }
            "corrupt-reference" => {
                fs::write(reference.join(".ess-output/state.json"), "not a checkpoint").unwrap();
            }
            _ => unreachable!(),
        }
        let before = snapshot(&f.0);
        assert!(
            ownership::adopt(&root, &reference, "synthesis", None).is_err(),
            "{case}"
        );
        assert_eq!(snapshot(&f.0), before, "{case}");
    }
}

#[test]
fn missing_anchor_creation_cuts_preserve_authored_parents_and_never_publish_partial_output() {
    let _serial = super::serial();
    let control = Fixture::new();
    let root = control.0.join("missing/deeper");
    let mut trace = Vec::new();
    ownership::probe::publish(&root, NEW, &mut |event| {
        trace.push(event.to_owned());
        Ok(())
    })
    .unwrap();
    let cuts = trace
        .iter()
        .enumerate()
        .filter(|(_, event)| event.contains("anchor-"))
        .collect::<Vec<_>>();
    assert!(!cuts.is_empty());
    for (cut, event) in &cuts {
        for process in [false, true] {
            let f = Fixture::new();
            let root = f.0.join("missing/deeper");
            let authored = fs::read(f.0.join("page.md")).unwrap();
            if process {
                let output = child(&root, "publish", *cut);
                assert_eq!(output.status.code(), Some(93), "{event}: {output:?}");
            } else {
                let mut seen = 0;
                assert!(ownership::probe::publish(&root, NEW, &mut |_| {
                    let current = seen;
                    seen += 1;
                    if current == *cut {
                        Err(std::io::Error::from_raw_os_error(5).into())
                    } else {
                        Ok(())
                    }
                })
                .is_err());
            }
            assert_eq!(fs::read(f.0.join("page.md")).unwrap(), authored);
            assert!(!root.join("same").exists());
            assert!(!root.join(".ess-output").exists());
            if root.exists() {
                ownership::recover(&root).unwrap();
                assert!(visible(&root).is_empty());
            } else {
                assert!(ownership::recover(&root).is_err());
            }
            publish(&root, NEW);
            assert_eq!(fs::read(root.join("same")).unwrap(), b"replacement");
        }
    }
    println!(
        "{} missing-anchor process cuts and IO boundaries",
        cuts.len()
    );
}
