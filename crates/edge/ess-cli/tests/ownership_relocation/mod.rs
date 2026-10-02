//! A committed output root regenerates in another checkout (beyond10x/ess#306).
//!
//! Each case generates into one directory, copies the whole tree, `.ess-output` included, to a
//! second directory, and generates there: a settled (Idle) state is bound to its new location,
//! while a state holding an interrupted transaction keeps refusing and names where it was recorded.
use super::{ownership, serial, snapshot, workspace, Fixture};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const OLD: &[(&str, &str)] = &[("same", "original"), ("retired/old", "withdrawn")];
const NEW: &[(&str, &str)] = &[("same", "replacement"), ("new/leaf", "introduced")];

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

fn synthesize(out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["generate", "synthesize", "--path"])
        .arg(workspace().join("examples/gatepass"))
        .args(["--target", "rust", "--layout", "crate", "--out"])
        .arg(out)
        .output()
        .unwrap()
}

fn payload(root: &Path) -> serde_json::Value {
    serde_json::from_slice::<serde_json::Value>(
        &fs::read(root.join(".ess-output/state.json")).unwrap(),
    )
    .unwrap()["payload"]
        .clone()
}

/// The absolute root the published checkpoint is bound to.
fn recorded_root(root: &Path) -> PathBuf {
    let mut path = PathBuf::from("/");
    for part in payload(root)["root"]["components"].as_array().unwrap() {
        let hex = part.as_str().unwrap();
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        path.push(String::from_utf8(bytes).unwrap());
    }
    path
}

fn phase(root: &Path) -> String {
    payload(root)["checkpoint"]["phase"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn without_state(tree: &std::collections::BTreeMap<PathBuf, Vec<u8>>) -> Vec<(&PathBuf, &Vec<u8>)> {
    tree.iter()
        .filter(|(p, _)| !p.starts_with(".ess-output"))
        .collect()
}

#[test]
fn a_committed_settled_output_root_regenerates_in_a_second_checkout() {
    let _serial = serial();
    let f = Fixture::new();
    let first = f.0.join("checkout-a");
    let second = f.0.join("checkout-b");
    fs::create_dir(&first).unwrap();
    let a = first.join("crate");
    let generated = synthesize(&a);
    assert!(generated.status.success(), "{generated:?}");
    fs::write(a.join("AUTHORED.md"), "authored beside generated output").unwrap();
    let original = snapshot(&a);
    copy_tree(&first, &second);
    let b = second.join("crate");
    assert_eq!(
        recorded_root(&b),
        a,
        "the copy carries the first checkout's binding"
    );
    let copied = snapshot(&b);

    // Unchanged output writes nothing, so a committed state stays byte-identical.
    let unchanged = synthesize(&b);
    assert!(
        unchanged.status.success(),
        "a settled copied root refused: {}",
        String::from_utf8_lossy(&unchanged.stderr)
    );
    assert_eq!(
        snapshot(&b),
        copied,
        "an unchanged regeneration in the second checkout wrote"
    );

    // A change that writes anyway records the new binding with the carried owners.
    let lib = fs::read(b.join("src/lib.rs")).unwrap();
    fs::remove_file(b.join("src/lib.rs")).unwrap();
    let recreated = synthesize(&b);
    assert!(recreated.status.success(), "{recreated:?}");
    assert_eq!(fs::read(b.join("src/lib.rs")).unwrap(), lib);
    assert_eq!(
        recorded_root(&b),
        b,
        "a writing run did not record its root"
    );
    assert_eq!(phase(&b), "Idle");
    assert_eq!(
        payload(&b)["checkpoint"]["ledger"],
        payload(&a)["checkpoint"]["ledger"],
        "rebinding changed the recorded owners"
    );
    let settled = snapshot(&b);
    assert_eq!(without_state(&settled), without_state(&original));
    let again = synthesize(&b);
    assert!(again.status.success(), "{again:?}");
    assert_eq!(snapshot(&b), settled, "a repeated run wrote");

    // Bound here, the owner repairs its own edited file and keeps the authored one.
    fs::write(b.join("src/lib.rs"), "edited owned output").unwrap();
    let repaired = synthesize(&b);
    assert!(repaired.status.success(), "{repaired:?}");
    assert_eq!(fs::read(b.join("src/lib.rs")).unwrap(), lib);
    assert_eq!(
        fs::read(b.join("AUTHORED.md")).unwrap(),
        b"authored beside generated output"
    );

    // The first checkout is untouched, and still regenerates as a no-op.
    assert_eq!(snapshot(&a), original);
    let back = synthesize(&a);
    assert!(back.status.success(), "{back:?}");
    assert_eq!(snapshot(&a), original);
}

#[test]
fn a_copied_checkout_whose_owned_file_differs_refuses_naming_it_and_adoption() {
    let _serial = serial();
    let f = Fixture::new();
    let a = f.0.join("a");
    let b = f.0.join("b");
    let generated = synthesize(&a);
    assert!(generated.status.success(), "{generated:?}");
    copy_tree(&a, &b);
    fs::write(b.join("src/lib.rs"), "edited before the first run here").unwrap();
    let before = snapshot(&b);
    let refused = synthesize(&b);
    assert!(!refused.status.success(), "{refused:?}");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    for expected in [
        "src/lib.rs",
        "ess generate output adopt",
        &a.display().to_string(),
    ] {
        assert!(stderr.contains(expected), "missing {expected}: {stderr}");
    }
    assert_eq!(snapshot(&b), before, "refusal wrote");
}

#[test]
fn a_settled_copy_is_bound_in_memory_and_recorded_by_the_next_write() {
    let _serial = serial();
    let f = Fixture::new();
    let a = f.0.join("a");
    let b = f.0.join("b");
    ownership::probe::publish(&a, OLD, &mut |_| Ok(())).unwrap();
    copy_tree(&a, &b);
    let copied = snapshot(&b);

    // Check, recovery and an unchanged publication admit the copy and write nothing.
    drop(ownership::check(&b).unwrap());
    ownership::recover(&b).unwrap();
    ownership::probe::publish(&b, OLD, &mut |_| Ok(())).unwrap();
    assert_eq!(snapshot(&b), copied);

    // A write cut before its first checkpoint is published leaves the copied state in force.
    let cut = ownership::probe::publish(&b, NEW, &mut |event| {
        if event == "before:rename:checkpoint" {
            anyhow::bail!("cut before publishing the first checkpoint")
        }
        Ok(())
    });
    assert!(cut.is_err());
    assert_eq!(
        snapshot(&b)
            .into_iter()
            .filter(|(p, _)| p.as_path() != Path::new(".ess-output/state.next"))
            .collect::<std::collections::BTreeMap<_, _>>(),
        copied
    );
    assert_eq!(recorded_root(&b), a);

    // The completed write runs one ordinary transaction from the copied ledger, bound here.
    ownership::probe::publish(&b, NEW, &mut |_| Ok(())).unwrap();
    assert!(
        !b.join("retired/old").exists(),
        "copied ledger lost retirement authority"
    );
    assert_eq!(fs::read(b.join("same")).unwrap(), b"replacement");
    assert_eq!(recorded_root(&b), b);
    assert_eq!(phase(&b), "Idle");
}

/// Leave `root` holding an interrupted transaction in the named phase.
fn interrupt(root: &Path, phase_name: &str) {
    ownership::probe::publish(root, OLD, &mut |_| Ok(())).unwrap();
    let publish_cut = match phase_name {
        "Staging" => "after:sync:complete-staging",
        "Prepared" | "Restored" => "after:rename:output-installation",
        "Committed" => "before:remove:transaction-cleanup",
        other => unreachable!("{other}"),
    };
    let result = ownership::probe::publish(root, NEW, &mut |event| {
        if event == publish_cut {
            anyhow::bail!("cut at {event}")
        }
        Ok(())
    });
    assert!(result.is_err());
    if phase_name == "Restored" {
        let result = ownership::probe::recover(root, &mut |event| {
            if event == "before:remove:transaction-cleanup" {
                anyhow::bail!("cut at {event}")
            }
            Ok(())
        });
        assert!(result.is_err());
    }
    assert_eq!(phase(root), phase_name);
}

#[test]
fn a_copied_root_with_an_interrupted_transaction_still_refuses_naming_its_recorded_root() {
    let _serial = serial();
    for phase_name in ["Staging", "Prepared", "Committed", "Restored"] {
        let f = Fixture::new();
        let a = f.0.join("a");
        let b = f.0.join("b");
        interrupt(&a, phase_name);
        copy_tree(&a, &b);
        let before = snapshot(&b);
        let recorded = a.display().to_string();

        let refused = synthesize(&b);
        assert!(!refused.status.success(), "{phase_name}: {refused:?}");
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert!(
            stderr.contains(&recorded),
            "{phase_name}: refusal does not name the recorded root: {stderr}"
        );
        for error in [
            ownership::probe::publish(&b, NEW, &mut |_| Ok(())).unwrap_err(),
            ownership::recover(&b).unwrap_err(),
            ownership::check(&b).err().unwrap(),
        ] {
            assert!(
                format!("{error:#}").contains(&recorded),
                "{phase_name}: {error:#}"
            );
        }
        assert_eq!(
            snapshot(&b),
            before,
            "{phase_name}: refusal deleted evidence"
        );
        assert_eq!(recorded_root(&b), a);

        // Where the transaction was recorded, recovery still settles it.
        ownership::recover(&a).unwrap();
        assert_eq!(phase(&a), "Idle", "{phase_name}");
    }
}
