//! A settled `.ess-output` record carries no absolute path, device or inode (beyond10x/ess#484).
//!
//! `ess-output-state/3` binds a checkpoint to its root only while a transaction is in flight,
//! because only an in-flight transaction is recovered where it was recorded. A settled (Idle)
//! record is what a repository commits, so it names nothing about the machine that wrote it.
//! `/1` and `/2` records stay readable: a write-mode run rewrites one as `/3`, and `--check`
//! admits it with one warning.
use super::{ownership, serial, snapshot, workspace, Fixture};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Component, Path, PathBuf},
    process::{Command, Output},
};

const THIS: &str = env!("CARGO_PKG_VERSION");
const OLD: &[(&str, &str)] = &[("same", "original"), ("retired/old", "withdrawn")];
const NEW: &[(&str, &str)] = &[("same", "replacement"), ("new/leaf", "introduced")];

/// Every key a settled `/3` record may carry, at any depth.
const IDLE_KEYS: &[&str] = &[
    "anchor_id",
    "checkpoint",
    "checksum",
    "components",
    "data",
    "digest",
    "directories",
    "encoding",
    "family",
    "files",
    "format",
    "key",
    "ledger",
    "length",
    "location",
    "mode",
    "owners",
    "path",
    "payload",
    "phase",
    "producer",
    "profile",
    "sequence",
];

/// The keys a number may sit under in a settled record: sizes, modes and the checkpoint counter.
const NUMERIC_KEYS: &[&str] = &["length", "mode", "sequence"];

/// `ess generate --kind openapi` of the billing example into `out`, writing or with `--check`.
fn generate(out: &Path, check: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["generate", "--path"])
        .arg(workspace().join("examples/billing"))
        .args(["--kind", "openapi", "--out"])
        .arg(out);
    if check {
        command.arg("--check");
    }
    command.output().unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn state_path(root: &Path) -> PathBuf {
    root.join(".ess-output/state.json")
}

fn state(root: &Path) -> Value {
    serde_json::from_slice(&fs::read(state_path(root)).unwrap()).unwrap()
}

/// The files under `root`, without the ownership record.
fn outputs(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    snapshot(root)
        .into_iter()
        .filter(|(path, _)| !path.starts_with(".ess-output"))
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    for byte in bytes {
        write!(&mut text, "{byte:02x}").unwrap();
    }
    text
}

/// Rewrite the payload and its checksum canonically, as another release would have left it.
fn rewrite(root: &Path, edit: impl FnOnce(&mut Value)) {
    use sha2::{Digest as _, Sha256};
    let mut value = state(root);
    edit(&mut value["payload"]);
    let mut payload = serde_json::to_vec(&value["payload"]).unwrap();
    payload.push(b'\n');
    value["checksum"] = hex(&Sha256::digest(&payload)).into();
    let mut bytes = serde_json::to_vec(&value).unwrap();
    bytes.push(b'\n');
    fs::write(state_path(root), bytes).unwrap();
}

/// The binding a `/1` or `/2` writer recorded for `root`: its absolute path as `UnixBytes1`
/// components, and its directory's device and inode.
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

/// Leave the record at `root` as a release writing `format` left it: bound to `bound`, with
/// `producer`, which `/1` does not have.
fn as_legacy(root: &Path, format: &str, bound: &Path, producer: Option<&str>) {
    let (native, identity) = binding(bound);
    rewrite(root, |payload| {
        payload["format"] = format.into();
        payload["root"] = native;
        payload["directory"] = identity;
        let object = payload.as_object_mut().unwrap();
        if let Some(producer) = producer {
            object.insert("producer".to_owned(), producer.into());
        } else {
            object.remove("producer");
        }
    });
}

/// Copy a tree with its modes, as a clone, a second worktree or a CI checkout would carry it.
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        assert!(!kind.is_symlink(), "{}", entry.path().display());
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
    fs::set_permissions(to, fs::metadata(from).unwrap().permissions()).unwrap();
}

/// Every object key in `value`, at any depth, and the keys a number sits under.
fn keys(value: &Value, all: &mut BTreeSet<String>, numeric: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                all.insert(key.clone());
                if inner.is_number() {
                    numeric.insert(key.clone());
                }
                keys(inner, all, numeric);
            }
        }
        Value::Array(items) => {
            for inner in items {
                keys(inner, all, numeric);
            }
        }
        _ => {}
    }
}

/// Every string value in `value`, at any depth.
fn strings<'a>(value: &'a Value, out: &mut Vec<&'a str>) {
    match value {
        Value::String(text) => out.push(text),
        Value::Object(map) => {
            for inner in map.values() {
                strings(inner, out);
            }
        }
        Value::Array(items) => {
            for inner in items {
                strings(inner, out);
            }
        }
        _ => {}
    }
}

/// The issue's reproducer: a write-mode generation leaves a settled `/3` record that names no
/// component of the output's absolute path, plain or hex-encoded, and no device or inode.
#[test]
fn a_written_idle_record_names_no_absolute_path_device_or_inode() {
    let _serial = serial();
    let f = Fixture::new();
    let out = f.0.join("generated-484");
    let written = generate(&out, false);
    assert!(written.status.success(), "{}", stderr(&written));
    let text = fs::read_to_string(state_path(&out)).unwrap();
    let record: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(record["payload"]["checkpoint"]["phase"], "Idle");

    let (mut all, mut numeric) = (BTreeSet::new(), BTreeSet::new());
    keys(&record, &mut all, &mut numeric);
    let foreign: Vec<_> = all
        .iter()
        .filter(|key| !IDLE_KEYS.contains(&key.as_str()))
        .collect();
    assert!(
        foreign.is_empty(),
        "a settled record carries {foreign:?}: {text}"
    );
    let counted: Vec<_> = numeric
        .iter()
        .filter(|key| !NUMERIC_KEYS.contains(&key.as_str()))
        .collect();
    assert!(
        counted.is_empty(),
        "a settled record carries numbers under {counted:?}: {text}"
    );

    let mut values = Vec::new();
    strings(&record, &mut values);
    for part in out.components() {
        let Component::Normal(name) = part else {
            continue;
        };
        let plain = name.to_str().unwrap();
        let encoded = hex(name.as_bytes());
        assert!(
            !values
                .iter()
                .any(|value| *value == plain || *value == encoded),
            "the record names the path component {plain:?}: {text}"
        );
        // Long enough that a digest cannot contain it by chance.
        if name.len() >= 5 {
            assert!(!text.contains(&encoded), "{plain:?} hex-encoded: {text}");
        }
        if name.len() >= 8 {
            assert!(!text.contains(plain), "{plain:?}: {text}");
        }
    }
    assert_eq!(record["payload"]["format"], "ess-output-state/3");
    assert_eq!(record["payload"]["producer"], format!("ess {THIS}"));
}

/// Leave `root` holding an interrupted transaction in `phase`.
fn interrupt(root: &Path, phase: &str) {
    ownership::probe::publish(root, OLD, &mut |_| Ok(())).unwrap();
    let cut = match phase {
        "Staging" => "after:sync:complete-staging",
        "Prepared" | "Restored" => "after:rename:output-installation",
        "Committed" => "before:remove:transaction-cleanup",
        other => unreachable!("{other}"),
    };
    let published = ownership::probe::publish(root, NEW, &mut |event| {
        if event == cut {
            anyhow::bail!("cut at {event}")
        }
        Ok(())
    });
    assert!(published.is_err());
    if phase == "Restored" {
        let recovered = ownership::probe::recover(root, &mut |event| {
            if event == "before:remove:transaction-cleanup" {
                anyhow::bail!("cut at {event}")
            }
            Ok(())
        });
        assert!(recovered.is_err());
    }
    assert_eq!(state(root)["payload"]["checkpoint"]["phase"], phase);
}

/// An in-flight transaction keeps its root and directory binding, refuses at any other root
/// naming the recorded one, and settles where it was recorded into a record that has neither.
#[test]
fn a_pending_record_keeps_its_binding_and_refuses_at_another_root() {
    let _serial = serial();
    for phase in ["Staging", "Prepared", "Committed", "Restored"] {
        let f = Fixture::new();
        let a = f.0.join("a");
        let b = f.0.join("b");
        interrupt(&a, phase);
        let pending = state(&a)["payload"].clone();
        assert_eq!(pending["format"], "ess-output-state/3", "{phase}");
        let (native, identity) = binding(&a);
        assert_eq!(pending["root"], native, "{phase}: {pending}");
        assert_eq!(pending["directory"], identity, "{phase}: {pending}");

        copy_tree(&a, &b);
        let before = snapshot(&b);
        let refused = format!(
            "{:#}",
            ownership::probe::publish(&b, NEW, &mut |_| Ok(())).unwrap_err()
        );
        assert!(
            refused.contains(&a.display().to_string()),
            "{phase}: the refusal names no recorded root: {refused}"
        );
        assert_eq!(snapshot(&b), before, "{phase}: a refusal wrote");

        ownership::recover(&a).unwrap();
        let settled = state(&a)["payload"].clone();
        assert_eq!(settled["checkpoint"]["phase"], "Idle", "{phase}");
        assert_eq!(settled["format"], "ess-output-state/3", "{phase}");
        assert!(
            settled.get("root").is_none() && settled.get("directory").is_none(),
            "{phase}: the settled record kept its binding: {settled}"
        );
    }
}

/// A `/1` or `/2` settled record is admitted where it was written and in a copy of its root, and
/// a write-mode run rewrites it as `/3` although no owned file changes. A recorded producer is
/// kept, since nothing was published; `/1` recorded none, so the run records itself. The `/3`
/// record it leaves is not rewritten again.
#[test]
fn a_legacy_idle_record_is_admitted_and_a_write_run_rewrites_it_as_version_three() {
    let _serial = serial();
    let this = format!("ess {THIS}");
    for (format, producer) in [
        ("ess-output-state/2", Some("ess 0.0.1")),
        ("ess-output-state/1", None),
    ] {
        for copied in [false, true] {
            let case = format!("{format}, copied: {copied}");
            let f = Fixture::new();
            let first = f.0.join("first");
            let written = generate(&first, false);
            assert!(written.status.success(), "{case}: {}", stderr(&written));
            as_legacy(&first, format, &first, producer);
            let root = if copied {
                let second = f.0.join("second");
                copy_tree(&first, &second);
                second
            } else {
                first
            };
            let legacy = state(&root)["payload"].clone();
            let files = outputs(&root);

            let rewritten = generate(&root, false);
            assert!(rewritten.status.success(), "{case}: {}", stderr(&rewritten));
            let payload = state(&root)["payload"].clone();
            assert_eq!(payload["format"], "ess-output-state/3", "{case}: {payload}");
            assert!(
                payload.get("root").is_none() && payload.get("directory").is_none(),
                "{case}: the rewritten record kept its binding: {payload}"
            );
            assert_eq!(payload["anchor_id"], legacy["anchor_id"], "{case}");
            assert_eq!(payload["checkpoint"], legacy["checkpoint"], "{case}");
            assert_eq!(payload["producer"], producer.unwrap_or(&this), "{case}");
            assert_eq!(outputs(&root), files, "{case}: an owned file changed");

            let settled = fs::read(state_path(&root)).unwrap();
            let again = generate(&root, false);
            assert!(again.status.success(), "{case}: {}", stderr(&again));
            assert_eq!(
                fs::read(state_path(&root)).unwrap(),
                settled,
                "{case}: an unchanged run rewrote a /3 record"
            );
        }
    }
}

/// `--check` admits a `/2` settled record, where it was written or in a copy, without reporting
/// drift, and prints one warning naming the machine-specific fields and the run that removes
/// them. A `/3` record draws no warning. Neither run writes.
#[test]
fn check_admits_a_version_two_idle_record_without_drift_and_warns_once() {
    let _serial = serial();
    let this = format!("ess {THIS}");
    for copied in [false, true] {
        let f = Fixture::new();
        let first = f.0.join("first");
        let written = generate(&first, false);
        assert!(written.status.success(), "{}", stderr(&written));
        let current = generate(&first, true);
        assert_eq!(current.status.code(), Some(0), "{}", stderr(&current));
        assert!(
            !stderr(&current).contains("ess-output-state"),
            "a /3 record drew a warning: {}",
            stderr(&current)
        );

        as_legacy(&first, "ess-output-state/2", &first, Some(this.as_str()));
        let root = if copied {
            let second = f.0.join("second");
            copy_tree(&first, &second);
            second
        } else {
            first
        };
        let before = snapshot(&root);
        let checked = generate(&root, true);
        let err = stderr(&checked);
        assert_eq!(
            checked.status.code(),
            Some(0),
            "copied: {copied}: a /2 record is reported as drift: {err}"
        );
        assert!(
            !err.contains("regenerate it with"),
            "copied: {copied}: a drift line: {err}"
        );
        let warnings: Vec<&str> = err
            .lines()
            .filter(|line| line.contains("warning:"))
            .collect();
        assert_eq!(warnings.len(), 1, "copied: {copied}: one warning: {err}");
        for named in [
            "state.json",
            "ess-output-state/2",
            "`root`",
            "`directory`",
            "without `--check`",
        ] {
            assert!(
                warnings[0].contains(named),
                "copied: {copied}: the warning does not name {named}: {err}"
            );
        }
        assert_eq!(snapshot(&root), before, "copied: {copied}: --check wrote");
    }
}

/// Generation, writing and with `--check`, refuses the record at `root` naming `expected`, and
/// leaves the root as it was.
fn refuses(root: &Path, case: &str, expected: &str) {
    let before = snapshot(root);
    for check in [false, true] {
        let output = generate(root, check);
        let err = stderr(&output);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{case}, check: {check}: {err}"
        );
        assert!(err.contains(expected), "{case}, check: {check}: {err}");
        assert_eq!(
            snapshot(root),
            before,
            "{case}, check: {check}: a refusal wrote"
        );
    }
}

/// The old-reader compatibility case (`AGENTS.md`, Determinism and formats). A release that reads
/// only `/2` refuses a `/3` record before writing (0.55.0: a settled one as missing `root`, a
/// pending one as an unsupported version); this reader holds each version to its own binding shape, so
/// a `/3` record relabelled `/2` is not read as a binding to nowhere, a settled `/3` record never
/// carries a binding, and a pending one never lacks it.
#[test]
fn each_output_state_version_is_held_to_its_own_binding_shape() {
    let _serial = serial();
    let this = format!("ess {THIS}");
    let f = Fixture::new();
    let idle = f.0.join("idle");
    let written = generate(&idle, false);
    assert!(written.status.success(), "{}", stderr(&written));
    let settled = fs::read(state_path(&idle)).unwrap();
    let unbind = |payload: &mut Value| {
        let object = payload.as_object_mut().unwrap();
        object.remove("root");
        object.remove("directory");
    };

    rewrite(&idle, |payload| {
        unbind(payload);
        payload["format"] = "ess-output-state/2".into();
    });
    refuses(
        &idle,
        "a /3 record relabelled /2",
        "ess-output-state/2 requires its `root` and `directory` binding",
    );
    fs::write(state_path(&idle), &settled).unwrap();
    rewrite(&idle, |payload| {
        unbind(payload);
        payload["format"] = "ess-output-state/1".into();
        payload.as_object_mut().unwrap().remove("producer");
    });
    refuses(
        &idle,
        "a /1 record without a binding",
        "ess-output-state/1 requires its `root` and `directory` binding",
    );
    fs::write(state_path(&idle), &settled).unwrap();
    as_legacy(&idle, "ess-output-state/3", &idle, Some(this.as_str()));
    refuses(
        &idle,
        "a settled /3 record with a binding",
        "records no `root` or `directory`",
    );
    fs::write(state_path(&idle), &settled).unwrap();
    rewrite(&idle, |payload| {
        payload["format"] = "ess-output-state/4".into();
    });
    refuses(&idle, "a later version", "unsupported output-state version");

    for kept in [None, Some("root"), Some("directory")] {
        let pending = f.0.join(format!("pending-{}", kept.unwrap_or("none")));
        interrupt(&pending, "Prepared");
        let (native, identity) = binding(&pending);
        rewrite(&pending, |payload| {
            unbind(payload);
            payload["format"] = "ess-output-state/3".into();
            match kept {
                Some("root") => payload["root"] = native,
                Some(_) => payload["directory"] = identity,
                None => {}
            }
        });
        let before = snapshot(&pending);
        let refused = format!("{:#}", ownership::recover(&pending).unwrap_err());
        assert!(
            refused.contains("pending")
                && refused.contains("requires its `root` and `directory` binding"),
            "a pending /3 record keeping {kept:?}: {refused}"
        );
        assert_eq!(snapshot(&pending), before, "{kept:?}: a refusal wrote");
    }
}
