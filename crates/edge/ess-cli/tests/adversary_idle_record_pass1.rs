//! Adversary pass 1 on beyond10x/ess#484: a settled `.ess-output` names no machine path.
//!
//! Each case leaves a root settled by this release and asserts that nothing under its
//! `.ess-output` carries the root's absolute path or the binding fields. Production code is
//! unchanged; the engine is compiled in, as `output_ownership_adversary.rs` does.
#[allow(dead_code)]
#[path = "../src/output_ownership/mod.rs"]
mod ownership;

use ess_cli::TemporaryDirectory;
use serde_json::{json, Value};
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Component, Path, PathBuf},
};

const OLD: &[(&str, &str)] = &[("same", "original"), ("retired/old", "withdrawn")];
const NEW: &[(&str, &str)] = &[("same", "replacement"), ("new/leaf", "introduced")];

fn serial() -> std::sync::MutexGuard<'static, ()> {
    static TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
    TEST.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn fixture() -> TemporaryDirectory {
    TemporaryDirectory::create("ess-idle-adversary").unwrap()
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    for byte in bytes {
        write!(&mut text, "{byte:02x}").unwrap();
    }
    text
}

fn state_path(root: &Path) -> PathBuf {
    root.join(".ess-output/state.json")
}

fn payload(root: &Path) -> Value {
    let value: Value = serde_json::from_slice(&fs::read(state_path(root)).unwrap()).unwrap();
    value["payload"].clone()
}

/// Rewrite the payload and its checksum canonically, as another release would have left it.
fn rewrite(root: &Path, edit: impl FnOnce(&mut Value)) {
    use sha2::{Digest as _, Sha256};
    let mut value: Value = serde_json::from_slice(&fs::read(state_path(root)).unwrap()).unwrap();
    edit(&mut value["payload"]);
    let mut bytes = serde_json::to_vec(&value["payload"]).unwrap();
    bytes.push(b'\n');
    value["checksum"] = hex(&Sha256::digest(&bytes)).into();
    let mut out = serde_json::to_vec(&value).unwrap();
    out.push(b'\n');
    fs::write(state_path(root), out).unwrap();
}

/// The binding a `/2` writer records for `root`.
fn binding(root: &Path) -> (Value, Value) {
    let components: Vec<String> = root
        .components()
        .filter_map(|part| match part {
            Component::Normal(name) => Some(hex(name.as_bytes())),
            _ => None,
        })
        .collect();
    let meta = fs::metadata(root).unwrap();
    (
        json!({"encoding": "UnixBytes1", "components": components}),
        json!({"device": meta.dev(), "inode": meta.ino()}),
    )
}

/// Every file under `root/.ess-output` whose bytes name a component of `root`, plain or
/// hex-encoded (the `UnixBytes1` encoding), or carry a `root`/`directory` binding key.
fn naming_the_root(root: &Path) -> Vec<(PathBuf, String)> {
    fn visit(path: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(&entry.path(), out);
            } else {
                out.push(entry.path());
            }
        }
    }
    let mut files = Vec::new();
    visit(&root.join(".ess-output"), &mut files);
    let mut found = Vec::new();
    for file in files {
        let text = String::from_utf8_lossy(&fs::read(&file).unwrap()).into_owned();
        for part in root.components() {
            let Component::Normal(name) = part else {
                continue;
            };
            // Long enough that a digest or UUID cannot contain it by chance.
            if name.len() >= 8 && text.contains(&hex(name.as_bytes())) {
                found.push((
                    file.clone(),
                    format!("{} hex-encoded", name.to_string_lossy()),
                ));
            }
        }
        for key in ["\"root\":", "\"directory\":"] {
            if text.contains(key) {
                found.push((file.clone(), key.to_owned()));
            }
        }
    }
    found
}

/// A run cut after the Staging checkpoint is written to `state.next` and before it replaces
/// `state.json` leaves the settled `/3` record in force and the pending checkpoint, binding and
/// all, in `state.next`. The next run changes nothing, so it returns before any checkpoint and
/// never clears the slot; `--check` does not look at it. The root is settled and `.ess-output`,
/// which the guide says to commit, still names the absolute path, device and inode.
#[test]
fn a_settled_root_keeps_no_binding_in_a_stale_checkpoint_slot() {
    let _serial = serial();
    let f = fixture();
    let root = f.join("checkpoint-slot-root");
    ownership::probe::publish(&root, OLD, &mut |_| Ok(())).unwrap();
    let cut = ownership::probe::publish(&root, NEW, &mut |event| {
        if event == "before:rename:checkpoint" {
            anyhow::bail!("cut at {event}")
        }
        Ok(())
    });
    assert!(cut.is_err());
    assert_eq!(payload(&root)["checkpoint"]["phase"], "Idle");

    // The same command again, writing and checking: both succeed and neither reports anything.
    ownership::probe::publish(&root, OLD, &mut |_| Ok(())).unwrap();
    let publication = ownership::Publication::tree("synthesis", OLD.iter().copied()).unwrap();
    let drift = ownership::drift(&root, &[publication]).unwrap();
    println!(
        "--check drift entries: {:?}",
        drift.iter().map(|d| &d.path).collect::<Vec<_>>()
    );

    let found = naming_the_root(&root);
    let _ = fs::remove_dir_all(&f);
    assert!(
        found.is_empty(),
        "a settled .ess-output still names this machine's root: {found:?}"
    );
}

/// A transaction left pending by a `/2` writer (0.55.0) is recovered by this release where it was
/// recorded. Recovery is a write-mode command and ends in a settled checkpoint, which this release
/// writes; that checkpoint must name no machine path (ess#484), as one written by generation does.
#[test]
fn recovering_a_version_two_transaction_settles_a_record_without_the_binding() {
    let _serial = serial();
    let f = fixture();
    let root = f.join("recovered-root");
    ownership::probe::publish(&root, OLD, &mut |_| Ok(())).unwrap();
    let cut = ownership::probe::publish(&root, NEW, &mut |event| {
        if event == "after:rename:output-installation" {
            anyhow::bail!("cut at {event}")
        }
        Ok(())
    });
    assert!(cut.is_err());
    // As 0.55.0 leaves a pending checkpoint: the same shape, labelled `/2`.
    rewrite(&root, |payload| {
        payload["format"] = "ess-output-state/2".into();
        payload["producer"] = "ess 0.55.0".into();
    });
    assert_eq!(payload(&root)["checkpoint"]["phase"], "Prepared");

    ownership::recover(&root).unwrap();
    let settled = payload(&root);
    let found = naming_the_root(&root);
    let _ = fs::remove_dir_all(&f);
    assert_eq!(settled["checkpoint"]["phase"], "Idle");
    assert!(
        settled.get("root").is_none() && settled.get("directory").is_none(),
        "recovery settled a {} record carrying the binding: {found:?}",
        settled["format"]
    );
}

/// Adoption into a root whose settled record a `/2` writer left is a write-mode command: it runs
/// a metadata transaction and writes a new settled checkpoint, bound in memory to this root. That
/// checkpoint must name no machine path (ess#484), and here it names this machine's root even when
/// the record it replaced was copied from another one.
#[test]
fn adopting_into_a_version_two_root_settles_a_record_without_the_binding() {
    let _serial = serial();
    let f = fixture();
    let reference = f.join("adoption-reference");
    let target = f.join("adoption-target");
    ownership::probe::publish(&reference, OLD, &mut |_| Ok(())).unwrap();
    ownership::publish(
        &target,
        vec![ownership::Publication::tree("model-types", [("other/x", "kept")]).unwrap()],
    )
    .unwrap();
    let (native, identity) = binding(&target);
    rewrite(&target, |payload| {
        payload["format"] = "ess-output-state/2".into();
        payload["producer"] = "ess 0.55.0".into();
        payload["root"] = native;
        payload["directory"] = identity;
    });
    fs::write(target.join("same"), "original").unwrap();
    fs::create_dir(target.join("retired")).unwrap();
    fs::write(target.join("retired/old"), "withdrawn").unwrap();

    ownership::probe::adopt(&target, &reference, &mut |_| Ok(())).unwrap();
    let settled = payload(&target);
    let found = naming_the_root(&target);
    let _ = fs::remove_dir_all(&f);
    assert_eq!(settled["checkpoint"]["phase"], "Idle");
    assert_eq!(
        settled["checkpoint"]["ledger"]["owners"]
            .as_array()
            .unwrap()
            .len(),
        2,
        "adoption enrolled the owner"
    );
    assert!(
        settled.get("root").is_none() && settled.get("directory").is_none(),
        "adoption settled a {} record carrying the binding: {found:?}",
        settled["format"]
    );
}
