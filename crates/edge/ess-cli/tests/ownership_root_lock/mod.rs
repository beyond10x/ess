//! Creating a new output root locks only what the run creates (beyond10x/ess#485). Every existing
//! ancestor, the nearest one included, is locked shared; the root is locked exclusive once this run
//! has created it. Runs creating different roots under one directory therefore proceed together,
//! two runs creating the same root still exclude each other, and an enrolled or reserved ancestor
//! still refuses.
use super::{ownership, serial, snapshot, workspace, Fixture};
use rustix::fs::{flock, FlockOperation};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
};

const FILES: &[(&str, &str)] = &[("index.html", "root page"), ("assets/style.css", "nested")];

fn publish(root: &Path) -> anyhow::Result<()> {
    ownership::probe::publish(root, FILES, &mut |_| Ok(()))
}

fn busy_at(root: &Path) -> String {
    format!("output ownership busy at {}", root.display())
}

/// Every entry below `root` whose name carries the reserved initialization prefix: a staged state
/// directory or an admission namespace that a run left behind.
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

fn reference_site(f: &Fixture) -> BTreeMap<PathBuf, Vec<u8>> {
    let out = f.0.join("reference/out");
    let output = generate(&out).wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    visible(&out)
}

/// The issue's reproducer: N concurrent `generate` runs, each creating a different new root
/// `<shared>/x<i>/out` under one existing directory, as runs sharing one `$TMPDIR` do. On ess
/// 0.55.0 all but one of them refused with `output ownership busy at <shared>`.
#[test]
fn concurrent_generate_runs_creating_distinct_roots_under_one_directory_all_publish() {
    const RUNS: usize = 16;
    let _serial = serial();
    let f = Fixture::new();
    let expected = reference_site(&f);
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let roots: Vec<_> = (0..RUNS)
        .map(|i| shared.join(format!("x{i}")).join("out"))
        .collect();
    let children: Vec<_> = roots.iter().map(|root| generate(root)).collect();
    let outputs: Vec<Output> = children
        .into_iter()
        .map(|child| child.wait_with_output().unwrap())
        .collect();
    let refused: Vec<_> = roots
        .iter()
        .zip(&outputs)
        .filter(|(_, output)| !output.status.success())
        .map(|(root, output)| {
            format!(
                "{}: {}",
                root.display(),
                String::from_utf8_lossy(&output.stderr)
            )
        })
        .collect();
    assert!(
        refused.is_empty(),
        "{} of {RUNS} runs creating distinct roots refused:\n{}",
        refused.len(),
        refused.join("\n")
    );
    for root in &roots {
        assert_eq!(visible(root), expected, "{} is complete", root.display());
        assert!(
            ownership::check(root).is_ok(),
            "{} is enrolled and settled",
            root.display()
        );
    }
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
    let mut names: Vec<_> = fs::read_dir(&shared)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    names.sort();
    let mut created: Vec<_> = (0..RUNS).map(|i| OsString::from(format!("x{i}"))).collect();
    created.sort();
    assert_eq!(names, created, "the shared directory holds only the roots");
}

/// Two `generate` runs per new root, all started together. Each root ends complete and settled
/// with no staged leftovers; a run that does not publish refuses as busy naming its own root, not
/// the directory the roots share.
#[test]
fn concurrent_generate_runs_creating_the_same_root_leave_one_complete_publication() {
    const ROOTS: usize = 6;
    let _serial = serial();
    let f = Fixture::new();
    let expected = reference_site(&f);
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let roots: Vec<_> = (0..ROOTS)
        .map(|i| shared.join(format!("same{i}")).join("out"))
        .collect();
    let children: Vec<_> = roots
        .iter()
        .flat_map(|root| [generate(root), generate(root)])
        .collect();
    let outputs: Vec<Output> = children
        .into_iter()
        .map(|child| child.wait_with_output().unwrap())
        .collect();
    for (root, pair) in roots.iter().zip(outputs.chunks(2)) {
        assert!(
            pair.iter().any(|output| output.status.success()),
            "neither run published {}: {pair:?}",
            root.display()
        );
        for output in pair.iter().filter(|output| !output.status.success()) {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains(&busy_at(root)),
                "a run losing {} refuses as busy naming it: {stderr}",
                root.display()
            );
        }
        assert_eq!(visible(root), expected, "{} is complete", root.display());
        assert!(
            ownership::check(root).is_ok(),
            "{} is enrolled and settled",
            root.display()
        );
    }
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
    let refused = outputs.iter().filter(|o| !o.status.success()).count();
    println!(
        "{refused} of {} runs refused as busy; the rest published or found the root published",
        outputs.len()
    );
}

/// A shared lock on the parent, as `check` or a run creating a sibling holds, does not refuse
/// creating a root beneath it. While a run is publishing, its parent and the intermediate it
/// created stay open to shared locks; only the new root itself is held exclusively. An exclusive
/// lock on the parent still refuses, naming the parent, before anything is created.
#[test]
fn creating_a_root_locks_its_ancestors_shared_and_only_the_root_exclusive() {
    let _serial = serial();
    let f = Fixture::new();
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();

    let reader = fs::File::open(&shared).unwrap();
    flock(&reader, FlockOperation::NonBlockingLockShared).unwrap();
    let first = shared.join("x/out");
    let result = publish(&first);
    assert!(
        result.is_ok(),
        "a shared lock on the parent refused creating a root beneath it: {result:?}"
    );
    flock(&reader, FlockOperation::Unlock).unwrap();

    let root = shared.join("y/out");
    let mut seen = Vec::new();
    ownership::probe::publish(&root, FILES, &mut |event| {
        if event == "before:mkdir:transaction-directory" && seen.is_empty() {
            seen.push((
                "shared lock on the parent",
                try_lock(&shared, FlockOperation::NonBlockingLockShared),
            ));
            seen.push((
                "shared lock on the created intermediate",
                try_lock(&shared.join("y"), FlockOperation::NonBlockingLockShared),
            ));
            seen.push((
                "exclusive lock on the new root",
                !try_lock(&root, FlockOperation::NonBlockingLockExclusive),
            ));
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(seen.len(), 3, "the publication reached its transaction");
    for (what, held_as_expected) in seen {
        assert!(held_as_expected, "{what} during publication");
    }

    let blocker = fs::File::open(&shared).unwrap();
    flock(&blocker, FlockOperation::NonBlockingLockExclusive).unwrap();
    let before = snapshot(&f.0);
    let refusal = publish(&shared.join("z/out")).unwrap_err();
    assert!(
        format!("{refusal:#}").contains(&busy_at(&shared)),
        "{refusal:#}"
    );
    assert_eq!(snapshot(&f.0), before, "a refused run created nothing");
}

/// A second run publishes a different new root while the first is between admission and creating
/// its own; both publish.
#[test]
fn a_run_creating_another_root_mid_creation_is_not_refused() {
    let _serial = serial();
    let f = Fixture::new();
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let (a, b) = (shared.join("a/out"), shared.join("b/out"));
    let mut inner = None;
    ownership::probe::publish(&a, FILES, &mut |event| {
        if event == "before:create-anchor-directory" && inner.is_none() {
            inner = Some(publish(&b));
        }
        Ok(())
    })
    .unwrap();
    let inner = inner.expect("the first run reached root creation");
    assert!(inner.is_ok(), "the second root was refused: {inner:?}");
    for root in [&a, &b] {
        assert_eq!(fs::read(root.join("index.html")).unwrap(), b"root page");
        assert!(ownership::check(root).is_ok());
    }
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
}

/// Where the first run is when a second run publishes the same new root, and how the second gets
/// there.
#[derive(Clone, Copy, Debug)]
enum Race {
    /// During the first run's name admission, before it creates anything.
    Admission,
    /// After admission, before the first run creates the root's missing parent.
    BeforeCreation,
    /// After the first run created the root and before it locked it.
    BeforeLock,
}

/// Two runs creating the same new root exclude each other: the run that publishes first leaves a
/// complete, settled root, and the other refuses as busy naming that root, writing nothing into it.
#[test]
fn a_second_run_creating_the_same_root_first_leaves_the_other_refused_busy() {
    let _serial = serial();
    for race in [Race::Admission, Race::BeforeCreation, Race::BeforeLock] {
        let f = Fixture::new();
        let shared = f.0.join("shared");
        fs::create_dir(&shared).unwrap();
        // `BeforeLock` creates no parent first, so its first creation event is the root's own.
        let root = match race {
            Race::BeforeLock => shared.join("same"),
            Race::Admission | Race::BeforeCreation => shared.join("pair/out"),
        };
        let event = match race {
            Race::Admission => "before:mkdir:admission-namespace",
            Race::BeforeCreation => "before:create-anchor-directory",
            Race::BeforeLock => "after:create-anchor-directory",
        };
        let mut inner = None;
        let mut hook = |seen: &str| -> anyhow::Result<()> {
            if seen == event && inner.is_none() {
                let result = publish(&root);
                inner = Some((result.is_ok().then(|| snapshot(&root)), result));
            }
            Ok(())
        };
        let outer = match race {
            Race::Admission => ownership::probe::publish_admission(&root, FILES, &mut hook),
            Race::BeforeCreation | Race::BeforeLock => {
                ownership::probe::publish(&root, FILES, &mut hook)
            }
        };
        let (published, inner) = inner.unwrap_or_else(|| panic!("{race:?}: {event} never fired"));
        assert!(
            inner.is_ok(),
            "{race:?}: the run that reached the root first was refused: {inner:?}"
        );
        let refusal = outer.expect_err("both runs published one new root");
        assert!(
            format!("{refusal:#}").contains(&busy_at(&root)),
            "{race:?}: {refusal:#}"
        );
        assert_eq!(
            Some(snapshot(&root)),
            published,
            "{race:?}: the refused run wrote into the published root"
        );
        assert!(ownership::check(&root).is_ok(), "{race:?}");
        assert_eq!(leftovers(&shared), Vec::<PathBuf>::new(), "{race:?}");
    }
}

/// Name admission never enters a root another run created after this run locked its parent: the
/// directory is that run's, not this one's. Here the second run publishes fewer files, so entering
/// its root would make this run probe a missing name there.
#[test]
fn admission_never_enters_a_root_another_run_created_after_locking() {
    let _serial = serial();
    let f = Fixture::new();
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let root = shared.join("pair/out");
    let more: Vec<_> = FILES
        .iter()
        .copied()
        .chain([("extra.html", "only the first run")])
        .collect();
    let mut published = None;
    let mut entered = Vec::new();
    let refusal = ownership::probe::publish_admission(&root, &more, &mut |event| {
        if published.is_none() && event == "before:mkdir:admission-namespace" {
            publish(&root)?;
            published = Some(snapshot(&root));
        } else if published.is_some() && !leftovers(&root).is_empty() {
            entered.push(event.to_owned());
        }
        Ok(())
    })
    .unwrap_err();
    assert!(published.is_some(), "the second run published");
    assert_eq!(
        entered,
        Vec::<String>::new(),
        "admission wrote inside another run's root"
    );
    assert!(
        format!("{refusal:#}").contains(&busy_at(&root)),
        "{refusal:#}"
    );
    assert_eq!(Some(snapshot(&root)), published);
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
}

/// A run that created the root but finds it locked by another run refuses as busy naming the root
/// and leaves the directory, empty, to the run holding it.
#[test]
fn a_created_root_bound_by_another_run_before_it_is_locked_is_left_to_that_run() {
    let _serial = serial();
    let f = Fixture::new();
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();
    let root = shared.join("held");
    let mut holder = None;
    let refusal = ownership::probe::publish(&root, FILES, &mut |event| {
        if event == "after:create-anchor-directory" && holder.is_none() {
            let fd = fs::File::open(&root)?;
            flock(&fd, FlockOperation::NonBlockingLockExclusive)?;
            holder = Some(fd);
        }
        Ok(())
    })
    .unwrap_err();
    assert!(holder.is_some(), "the run reached root creation");
    assert!(
        format!("{refusal:#}").contains(&busy_at(&root)),
        "{refusal:#}"
    );
    assert!(
        root.is_dir(),
        "the refused run removed a root another run holds"
    );
    assert_eq!(
        snapshot(&root),
        BTreeMap::new(),
        "the refused run wrote into a root another run holds"
    );
    drop(holder);
    publish(&root).unwrap();
}

/// An enrolled or reserved ancestor still refuses a new root beneath it: one present when the run
/// starts, one another run enrolls before this run creates that directory, and one another run
/// enrolls in the directory this run just created, before this run locks it.
#[test]
fn an_enrolled_or_reserved_ancestor_still_refuses_a_new_root_beneath_it() {
    let _serial = serial();
    let f = Fixture::new();
    let shared = f.0.join("shared");
    fs::create_dir(&shared).unwrap();

    let enrolled = shared.join("enrolled");
    publish(&enrolled).unwrap();
    fs::create_dir_all(shared.join("reserved/.ESS-OUTPUT")).unwrap();
    for (root, refusal) in [
        (
            enrolled.join("x/out"),
            "output root is beneath enrolled or reserved ancestor",
        ),
        (
            shared.join("reserved/x/out"),
            "output root is beneath enrolled or reserved ancestor",
        ),
        (
            shared.join(".ess-output-init-x/out"),
            "ownership root intersects reserved state namespace",
        ),
    ] {
        let before = snapshot(&f.0);
        let error = publish(&root).unwrap_err();
        assert!(
            format!("{error:#}").contains(refusal),
            "{}: {error:#}",
            root.display()
        );
        assert_eq!(snapshot(&f.0), before, "{}", root.display());
    }

    for (name, event) in [
        ("before-creation", "before:create-anchor-directory"),
        ("before-lock", "after:create-anchor-directory"),
    ] {
        let ancestor = shared.join(name);
        let root = ancestor.join("out");
        let mut inner = None;
        let outer = ownership::probe::publish(&root, FILES, &mut |seen| {
            if seen == event && inner.is_none() {
                let result = publish(&ancestor);
                inner = Some((result.is_ok().then(|| snapshot(&ancestor)), result));
            }
            Ok(())
        });
        let (published, inner) = inner.unwrap_or_else(|| panic!("{name}: {event} never fired"));
        assert!(
            inner.is_ok(),
            "{name}: the run enrolling the ancestor was refused: {inner:?}"
        );
        let refusal = outer.expect_err("a root was created beneath a newly enrolled ancestor");
        let message = format!("{refusal:#}");
        assert!(
            message.contains(&busy_at(&root))
                || message.contains("output root is beneath enrolled or reserved ancestor"),
            "{name}: {message}"
        );
        assert!(
            !root.exists(),
            "{name}: a root was created inside an enrolled one"
        );
        assert_eq!(Some(snapshot(&ancestor)), published, "{name}");
        assert!(ownership::check(&ancestor).is_ok(), "{name}");
    }
    assert_eq!(leftovers(&shared), Vec::<PathBuf>::new());
}
