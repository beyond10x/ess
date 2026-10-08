//! Adversary pass 1 for creating an output root that locks only what it creates
//! (beyond10x/ess#485): ownership safety around a root a run creates, not liveness.
//!
//! A separate target so that no earlier test file changes; it compiles the same engine.
#[allow(dead_code)]
#[path = "../src/output_ownership/mod.rs"]
mod ownership;

use rustix::fs::{flock, FlockOperation};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

const FILES: &[(&str, &str)] = &[("index.html", "root page"), ("assets/style.css", "nested")];

fn serial() -> std::sync::MutexGuard<'static, ()> {
    static TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
    TEST.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "ess-root-lock-adv1-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fn writable(path: &Path) {
            let Ok(meta) = fs::symlink_metadata(path) else {
                return;
            };
            if meta.is_dir() {
                let _ = fs::set_permissions(
                    path,
                    fs::Permissions::from_mode(meta.permissions().mode() | 0o700),
                );
                for entry in fs::read_dir(path).into_iter().flatten().flatten() {
                    writable(&entry.path());
                }
            }
        }
        writable(&self.0);
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("examples/billing").is_dir())
        .unwrap()
        .to_path_buf()
}

/// Every entry below `root`: file bytes, an empty value for a directory, and a symlink's target.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(path).unwrap() {
            let p = e.unwrap().path();
            let meta = fs::symlink_metadata(&p).unwrap();
            let key = p.strip_prefix(root).unwrap().to_path_buf();
            if meta.file_type().is_symlink() {
                let target = fs::read_link(&p).unwrap();
                out.insert(key, target.into_os_string().into_encoded_bytes());
            } else if meta.is_dir() {
                out.insert(key, Vec::new());
                visit(root, &p, out);
            } else {
                out.insert(key, fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}

fn names(directory: &Path) -> Vec<OsString> {
    let mut names: Vec<_> = fs::read_dir(directory)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    names.sort();
    names
}

/// Entries carrying the reserved initialization prefix: staged state or an admission namespace.
fn leftovers(root: &Path) -> Vec<PathBuf> {
    snapshot(root)
        .into_keys()
        .filter(|path| {
            path.components().any(|c| {
                c.as_os_str()
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .starts_with(".ess-output-init")
            })
        })
        .collect()
}

/// The generated files under a root, without its private `.ess-output` record.
fn visible(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    snapshot(root)
        .into_iter()
        .filter(|(path, _)| !path.starts_with(".ess-output"))
        .collect()
}

/// What [`publish`] leaves visible in a root.
fn published() -> BTreeMap<PathBuf, Vec<u8>> {
    BTreeMap::from([
        (PathBuf::from("assets"), Vec::new()),
        (PathBuf::from("assets/style.css"), b"nested".to_vec()),
        (PathBuf::from("index.html"), b"root page".to_vec()),
    ])
}

fn publish(root: &Path) -> anyhow::Result<()> {
    ownership::probe::publish(root, FILES, &mut |_| Ok(()))
}

fn busy_at(root: &Path) -> String {
    format!("output ownership busy at {}", root.display())
}

fn try_lock(path: &Path, operation: FlockOperation) -> bool {
    let fd = fs::File::open(path).unwrap();
    flock(&fd, operation).is_ok()
}

fn generate(out: &Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["generate", "--path"])
        .arg(workspace().join("examples/billing"))
        .args(["--kind", "site", "--out"])
        .arg(out)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

/// While a run publishes a root it created, a run creating a root nested beneath it is refused
/// busy at that root and creates nothing. The window chosen is the one before the creator's
/// `.ess-output` exists, where the enrolled-ancestor check cannot see it yet: only the exclusive
/// lock on the new root keeps the nested run out. The unit's own probe of that lock asserts that
/// an exclusive request fails, which a shared lock satisfies as well.
#[test]
fn a_root_nested_in_a_root_being_created_is_refused_busy_before_its_state_exists() {
    let _serial = serial();
    let f = Fixture::new();
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let root = shared.join("n");
    let nested = root.join("m");
    let mut seen = None;
    ownership::probe::publish(&root, FILES, &mut |event| {
        if event == "before:mkdir:initial-state-directory" && seen.is_none() {
            let shared_lock = try_lock(&root, FlockOperation::NonBlockingLockShared);
            let inner = publish(&nested);
            seen = Some((inner, nested.exists(), shared_lock));
        }
        Ok(())
    })
    .unwrap();
    let (inner, created, shared_lock) = seen.expect("the creator reached its initial state");
    let refusal = inner.expect_err("a root was published inside a root being created");
    assert!(
        format!("{refusal:#}").contains(&busy_at(&root)),
        "{refusal:#}"
    );
    assert!(
        !created,
        "the nested run created its root inside the new root"
    );
    assert!(
        !shared_lock,
        "a shared lock on the new root was granted while it was publishing"
    );
    assert_eq!(visible(&root), published());
    assert!(ownership::check(&root).is_ok());
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
}

/// A run that creates a root nested in a directory another run has just created as its root,
/// before that run locked it, publishes; the creator then refuses busy at its root and writes
/// nothing beside the nested root.
#[test]
fn a_root_nested_in_a_new_root_before_its_lock_leaves_the_creator_refused() {
    let _serial = serial();
    let f = Fixture::new();
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let root = shared.join("n");
    let nested = root.join("m");
    let mut inner = None;
    let outer = ownership::probe::publish(&root, FILES, &mut |event| {
        if event == "after:create-anchor-directory" && inner.is_none() {
            inner = Some(publish(&nested));
        }
        Ok(())
    });
    let inner = inner.expect("the creator reached root creation");
    assert!(inner.is_ok(), "the nested run was refused: {inner:?}");
    let refusal = outer.expect_err("the creator published over a nested root");
    assert!(
        format!("{refusal:#}").contains(&busy_at(&root)),
        "{refusal:#}"
    );
    assert_eq!(names(&root), vec![OsString::from("m")]);
    assert_eq!(visible(&nested), published());
    assert!(ownership::check(&nested).is_ok());
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
}

/// The implementor's named gap: two runs creating sibling roots under one parent that does not
/// exist yet. Whichever order they take, the parent is never anybody's root, a refused run
/// refuses busy at its own root before creating anything, and a run that proceeds publishes
/// completely.
#[test]
fn sibling_roots_under_one_missing_parent_refuse_only_before_writing() {
    let _serial = serial();
    for (order, event, inner_first) in [
        ("inner first", "before:create-anchor-directory", true),
        ("outer first", "after:create-anchor-directory", false),
    ] {
        let f = Fixture::new();
        let shared = f.0.join("shared");
        fs::create_dir(&shared).unwrap();
        let common = shared.join("common");
        let (a, b) = (common.join("a"), common.join("b"));
        let mut inner = None;
        let outer = ownership::probe::publish(&a, FILES, &mut |seen| {
            if seen == event && inner.is_none() {
                inner = Some(publish(&b));
            }
            Ok(())
        });
        let inner = inner.unwrap_or_else(|| panic!("{order}: {event} never fired"));
        assert!(inner.is_ok(), "{order}: {inner:?}");
        if inner_first {
            let refusal = outer.expect_err("both published after racing for the parent");
            assert!(
                format!("{refusal:#}").contains(&busy_at(&a)),
                "{order}: {refusal:#}"
            );
            assert!(!a.exists(), "{order}: the refused run created its root");
            assert_eq!(names(&common), vec![OsString::from("b")], "{order}");
        } else {
            assert!(outer.is_ok(), "{order}: {outer:?}");
            assert_eq!(visible(&a), published(), "{order}");
            assert!(ownership::check(&a).is_ok(), "{order}");
        }
        assert_eq!(visible(&b), published(), "{order}");
        assert!(ownership::check(&b).is_ok(), "{order}");
        assert!(!common.join(".ess-output").exists(), "{order}");
        assert_eq!(leftovers(&shared), Vec::<PathBuf>::new(), "{order}");
    }
}

/// The same gap as processes: N concurrent `generate` runs, each creating `<shared>/common/x<i>`
/// while `common` does not exist. Every run either publishes its root completely or refuses busy
/// naming its own root and leaves that root absent; nothing else appears.
#[test]
fn concurrent_runs_under_one_missing_parent_publish_completely_or_leave_their_root_absent() {
    const RUNS: usize = 8;
    let _serial = serial();
    let f = Fixture::new();
    let reference = f.0.join("reference/out");
    let output = generate(&reference).wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let expected = visible(&reference);
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let common = shared.join("common");
    let roots: Vec<_> = (0..RUNS).map(|i| common.join(format!("x{i}"))).collect();
    let children: Vec<_> = roots.iter().map(|root| generate(root)).collect();
    let outputs: Vec<Output> = children
        .into_iter()
        .map(|child| child.wait_with_output().unwrap())
        .collect();
    let mut kept = Vec::new();
    for (root, output) in roots.iter().zip(&outputs) {
        if output.status.success() {
            assert_eq!(visible(root), expected, "{} is complete", root.display());
            assert!(ownership::check(root).is_ok(), "{}", root.display());
            kept.push(root.file_name().unwrap().to_owned());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains(&busy_at(root)), "{stderr}");
            assert!(!root.exists(), "a refused run left {}", root.display());
        }
    }
    assert!(!kept.is_empty(), "no run published");
    kept.sort();
    assert_eq!(names(&shared), vec![OsString::from("common")]);
    assert_eq!(names(&common), kept);
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
    println!("{} of {RUNS} runs refused busy", RUNS - kept.len());
}

/// A symlink raced in place of a parent the run just created, or in place of that parent after
/// it was locked and revalidated, is never followed: the directory it points at stays untouched.
#[test]
fn a_symlink_raced_in_place_of_a_created_parent_is_never_followed() {
    let _serial = serial();
    for (case, event, nth) in [
        ("after creating it", "after:create-anchor-directory", 1),
        (
            "before creating the root",
            "before:create-anchor-directory",
            2,
        ),
    ] {
        let f = Fixture::new();
        let shared = f.0.join("shared");
        fs::create_dir(&shared).unwrap();
        let elsewhere = f.0.join("elsewhere");
        fs::create_dir(&elsewhere).unwrap();
        let parent = shared.join("s");
        let root = parent.join("out");
        let mut fired = 0;
        let mut swapped = false;
        let outer = ownership::probe::publish(&root, FILES, &mut |seen| {
            if seen == event {
                fired += 1;
                if fired == nth {
                    fs::rename(&parent, shared.join("s.moved"))?;
                    std::os::unix::fs::symlink(&elsewhere, &parent)?;
                    swapped = true;
                }
            }
            Ok(())
        });
        assert!(swapped, "{case}: the race point never fired");
        assert!(
            outer.is_err(),
            "{case}: a run published through a raced symlink"
        );
        assert_eq!(snapshot(&elsewhere), BTreeMap::new(), "{case}");
    }
}

/// A run that finds its new root missing when it locks, and another run publishes that root
/// before this run reaches name admission, refuses busy at the root and leaves the publication
/// exactly as it was: it never plans against, admits into or writes over the other run's root.
#[test]
fn a_root_published_between_locking_and_admission_is_left_to_its_publisher() {
    let _serial = serial();
    let f = Fixture::new();
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let root = shared.join("pair/out");
    let mut published_by_inner = None;
    let outer = ownership::probe::publish(&root, FILES, &mut |event| {
        if event == "before:plan:preimages" && published_by_inner.is_none() {
            publish(&root)?;
            published_by_inner = Some(snapshot(&root));
        }
        Ok(())
    });
    assert!(published_by_inner.is_some(), "the inner run published");
    let refusal = outer.expect_err("both runs published one new root");
    assert!(
        format!("{refusal:#}").contains(&busy_at(&root)),
        "{refusal:#}"
    );
    assert_eq!(Some(snapshot(&root)), published_by_inner);
    assert!(ownership::check(&root).is_ok());
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
}

/// Adoption into a new root that another run publishes during this run's name admission refuses
/// busy at the root and leaves that publication unchanged.
#[test]
fn adoption_into_a_new_root_published_during_its_admission_refuses_busy() {
    let _serial = serial();
    let f = Fixture::new();
    let reference = f.0.join("reference");
    publish(&reference).unwrap();
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let root = shared.join("pair/out");
    let mut published_by_inner = None;
    let outer = ownership::probe::adopt_admission(&root, &reference, &mut |event| {
        if event == "before:mkdir:admission-namespace" && published_by_inner.is_none() {
            publish(&root)?;
            published_by_inner = Some(snapshot(&root));
        }
        Ok(())
    });
    assert!(published_by_inner.is_some(), "the inner run published");
    let refusal = outer.expect_err("adoption wrote into a root another run created");
    assert!(
        format!("{refusal:#}").contains(&busy_at(&root)),
        "{refusal:#}"
    );
    assert_eq!(Some(snapshot(&root)), published_by_inner);
    assert!(ownership::check(&root).is_ok());
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
}
