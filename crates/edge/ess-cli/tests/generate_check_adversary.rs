//! Adversary cases for `ess generate --check` (W1-4, pass 1).
//!
//! The property under attack: `--check` exiting 0 means the documented regeneration (the same
//! command without `--check`) leaves the committed tree as it is. And: a check writes nothing, and
//! refuses a pending operation wherever the root was copied to.
#[allow(dead_code)]
#[path = "../src/output_ownership/mod.rs"]
mod ownership;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn serial() -> std::sync::MutexGuard<'static, ()> {
    static TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
    TEST.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("examples/billing").is_dir())
        .expect("the workspace holds examples/billing")
        .to_path_buf()
}

fn ess(args: &[&str], out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["generate", "--path"])
        .arg(workspace().join("examples/billing"))
        .args(args)
        .arg("--out")
        .arg(out)
        .output()
        .expect("ess runs")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Every file under `root`, `.ess-output` included, with its bytes.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).expect("readable directory") {
            let path = entry.expect("directory entry").path();
            if fs::symlink_metadata(&path).expect("metadata").is_dir() {
                visit(root, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(root).expect("under root").to_path_buf(),
                    fs::read(&path).expect("readable file"),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

fn changed(
    before: &BTreeMap<PathBuf, Vec<u8>>,
    after: &BTreeMap<PathBuf, Vec<u8>>,
) -> Vec<PathBuf> {
    before
        .keys()
        .chain(after.keys())
        .filter(|p| before.get(*p) != after.get(*p))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The generated files are current but the committed `.ess-output` record is the one from the
/// previous generation, as when a change commits `site/` and not the dot directory beside it.
/// `--check` passes, and the documented reconcile step (the same command without `--check`)
/// then rewrites `.ess-output/state.json`: CI is green on a tree regeneration changes.
///
/// Since beyond10x/ess#484 an owned file whose bytes differ from the record refuses regeneration
/// in every root, so `--check` names those files with the re-enroll route, and regeneration
/// refuses them without writing.
#[test]
fn check_passing_means_regeneration_leaves_the_tree_unchanged_with_a_stale_record() {
    let _serial = serial();
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("site");
    let guide = dir.path().join("guide.md");
    let include = format!("guide={}", guide.display());
    let args = ["--kind", "site", "--include", &include];

    fs::write(&guide, "# A guide\n\nFirst wording.\n").expect("guide");
    let first = ess(&args, &out);
    assert!(first.status.success(), "first generation: {first:?}");
    let stale_record = fs::read(out.join(".ess-output/state.json")).expect("state.json");

    fs::write(&guide, "# A guide\n\nSecond wording.\n").expect("guide");
    let second = ess(&args, &out);
    assert!(second.status.success(), "second generation: {second:?}");
    fs::write(out.join(".ess-output/state.json"), &stale_record).expect("restore stale record");

    let check = ess(&[&args[..], &["--check"]].concat(), &out);
    let before = snapshot(&out);
    let regenerate = ess(&args, &out);
    let after = snapshot(&out);
    let rewritten = changed(&before, &after);

    let named = stderr(&check);
    let guide_line = named
        .lines()
        .find(|line| line.contains("guide.html"))
        .unwrap_or_else(|| panic!("--check names guide.html: {named}"));
    assert!(
        guide_line.contains("ess generate output adopt"),
        "the drift line names the re-enroll route: {guide_line}"
    );
    let refused = stderr(&regenerate);
    assert!(
        !regenerate.status.success(),
        "regeneration replaced bytes the record does not hold: {refused}"
    );
    for named in [
        "guide.html".to_owned(),
        format!(
            "ess generate output adopt --ownership-root {}",
            out.display()
        ),
    ] {
        assert!(
            refused.contains(&named),
            "the refusal names {named}: {refused}"
        );
    }
    assert!(rewritten.is_empty(), "the refusal wrote {rewritten:?}");
    assert!(
        check.status.code() == Some(1) || rewritten.is_empty(),
        "--check exited {:?} (stderr: {}), and then the same command without --check rewrote \
         {rewritten:?}",
        check.status.code(),
        stderr(&check)
    );
}

/// The projections are committed and current, `.ess-output` is not (a `git add generated/*` glob
/// skips the dot directory). `--check` passes, and the documented reconcile step refuses every
/// file as an unowned destination: CI is green on a tree that cannot be regenerated in place.
#[test]
fn check_passing_means_regeneration_succeeds_without_the_record() {
    let _serial = serial();
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("openapi");
    let first = ess(&["--kind", "openapi"], &out);
    assert!(first.status.success(), "generation: {first:?}");
    fs::remove_dir_all(out.join(".ess-output")).expect("drop the record");

    let check = ess(&["--kind", "openapi", "--check"], &out);
    let regenerate = ess(&["--kind", "openapi"], &out);

    assert!(
        check.status.code() == Some(1) || regenerate.status.success(),
        "--check exited {:?} (stderr: {}), and then the same command without --check refused: {}",
        check.status.code(),
        stderr(&check),
        stderr(&regenerate)
    );
}

/// `--check` against an `--out` that does not exist names every file and creates nothing.
#[test]
fn check_against_an_absent_root_creates_nothing() {
    let _serial = serial();
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("absent/deeper");
    let check = ess(&["--kind", "openapi", "--check"], &out);
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    assert!(
        stderr(&check).contains("is missing; regenerate it with `ess generate`"),
        "{}",
        stderr(&check)
    );
    assert!(
        !dir.path().join("absent").exists(),
        "--check created the output root"
    );
}

/// A foreign attribute on the ownership record itself is refused naming `state.json`, the path
/// relative to `.ess-output`, while the guide says a refusal names the file relative to the
/// output root.
#[test]
fn a_refusal_of_the_ownership_record_names_its_path_relative_to_the_output_root() {
    let _serial = serial();
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("openapi");
    let first = ess(&["--kind", "openapi"], &out);
    assert!(first.status.success(), "generation: {first:?}");
    let state = fs::File::open(out.join(".ess-output/state.json")).expect("state.json");
    match rustix::fs::fsetxattr(
        &state,
        "user.ess_test",
        b"1",
        rustix::fs::XattrFlags::empty(),
    ) {
        Ok(()) => {}
        Err(rustix::io::Errno::NOTSUP) => {
            eprintln!("not run: this filesystem carries no user attributes");
            return;
        }
        Err(error) => panic!("setting user.ess_test: {error}"),
    }
    for args in [
        &["--kind", "openapi"][..],
        &["--kind", "openapi", "--check"],
    ] {
        let refused = ess(args, &out);
        assert!(!refused.status.success(), "{args:?}: {refused:?}");
        let err = stderr(&refused);
        assert!(err.contains("user.ess_test"), "{args:?}: {err}");
        assert!(
            err.contains("refused output .ess-output/state.json"),
            "{args:?}: the refusal names the record relative to the output root: {err}"
        );
    }
}

const OLD: &[(&str, &str)] = &[("same", "original"), ("retired/old", "withdrawn")];
const NEW: &[(&str, &str)] = &[("same", "replacement"), ("new/leaf", "introduced")];

fn phase(root: &Path) -> String {
    serde_json::from_slice::<serde_json::Value>(
        &fs::read(root.join(".ess-output/state.json")).expect("state.json"),
    )
    .expect("state.json parses")["payload"]["checkpoint"]["phase"]
        .as_str()
        .expect("phase")
        .to_owned()
}

/// Leave `root` holding a publication interrupted after staging.
fn interrupt(root: &Path) {
    ownership::probe::publish(root, OLD, &mut |_| Ok(())).expect("first publication");
    let cut = ownership::probe::publish(root, NEW, &mut |event| {
        if event == "after:sync:complete-staging" {
            anyhow::bail!("cut at {event}")
        }
        Ok(())
    });
    assert!(cut.is_err());
    assert_eq!(phase(root), "Staging");
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir(to).expect("mkdir");
    for entry in fs::read_dir(from).expect("read_dir") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy");
        }
    }
    fs::set_permissions(to, fs::metadata(from).expect("meta").permissions()).expect("chmod");
}

fn new_publication() -> Vec<ownership::Publication> {
    vec![ownership::Publication::tree("synthesis", NEW.iter().copied()).expect("publication")]
}

/// A check of a root holding an interrupted operation refuses, writes nothing and points at
/// recovery, as `check` does.
#[test]
fn drift_refuses_an_interrupted_operation_in_place() {
    let _serial = serial();
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("a");
    interrupt(&root);
    let before = snapshot(&root);
    let error = ownership::drift(&root, &new_publication())
        .err()
        .expect("a pending operation refuses the comparison");
    assert!(
        format!("{error:#}").contains("ess generate output recover"),
        "{error:#}"
    );
    assert_eq!(snapshot(&root), before, "the refusal wrote nothing");
}

/// A copy of a root holding an interrupted operation refuses the comparison naming the root
/// the operation was recorded at, the branch the relocation bypass keeps for non-idle states.
#[test]
fn drift_on_a_copied_interrupted_root_names_the_recorded_root() {
    let _serial = serial();
    let dir = tempfile::tempdir().expect("tempdir");
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    interrupt(&a);
    copy_tree(&a, &b);
    let before = snapshot(&b);
    let error = ownership::drift(&b, &new_publication())
        .err()
        .expect("a copied pending operation refuses the comparison");
    assert!(
        format!("{error:#}").contains(&format!("was recorded at {}", a.display())),
        "the refusal names the recorded root {}: {error:#}",
        a.display()
    );
    assert_eq!(snapshot(&b), before, "the refusal wrote nothing");
    ownership::recover(&a).expect("recovery where it was recorded");
}
