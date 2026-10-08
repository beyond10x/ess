//! Adversary pass 2 for the rebinding of a settled output state (beyond10x/ess#306).
//!
//! A separate target so that no earlier test file changes; it compiles the same engine.
#[allow(dead_code)]
#[path = "../src/output_ownership/mod.rs"]
mod ownership;

use ess_cli::TemporaryDirectory;
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const OLD: &[(&str, &str)] = &[("same", "original"), ("retired/old", "withdrawn")];
const NEW: &[(&str, &str)] = &[("same", "replacement"), ("new/leaf", "introduced")];

fn serial() -> std::sync::MutexGuard<'static, ()> {
    static TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
    TEST.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct Fixture(TemporaryDirectory);
impl Fixture {
    fn new() -> Self {
        let root = TemporaryDirectory::create("ess-reloc-p2").unwrap();
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
        .find(|p| p.join("examples/gatepass").is_dir())
        .unwrap()
        .to_path_buf()
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, u32)> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, (Vec<u8>, u32)>) {
        for e in fs::read_dir(path).unwrap() {
            let p = e.unwrap().path();
            let meta = fs::symlink_metadata(&p).unwrap();
            let key = p.strip_prefix(root).unwrap().to_path_buf();
            if meta.file_type().is_symlink() {
                let target = fs::read_link(&p).unwrap();
                out.insert(key, (target.into_os_string().into_encoded_bytes(), 0));
            } else if meta.is_dir() {
                out.insert(key, (Vec::new(), meta.permissions().mode()));
                visit(root, &p, out);
            } else {
                out.insert(key, (fs::read(&p).unwrap(), meta.permissions().mode()));
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}

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

fn ess(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn synthesize(spec: &Path, out: &Path) -> Output {
    ess(&[
        "generate".as_ref(),
        "synthesize".as_ref(),
        "--path".as_ref(),
        spec.as_os_str(),
        "--target".as_ref(),
        "rust".as_ref(),
        "--layout".as_ref(),
        "crate".as_ref(),
        "--out".as_ref(),
        out.as_os_str(),
    ])
}

/// A hermetic git: no user or system configuration, hooks or signing.
fn git(umask: &str, cwd: &Path, args: &[&str]) -> Output {
    let output = Command::new("sh")
        .arg("-c")
        .arg("umask \"$0\" && exec git -c core.hooksPath=/dev/null -c commit.gpgsign=false -c user.name=adversary -c user.email=adversary@example.invalid \"$@\"")
        .arg(umask)
        .args(args)
        .current_dir(cwd)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
    output
}

fn payload(root: &Path) -> serde_json::Value {
    serde_json::from_slice::<serde_json::Value>(
        &fs::read(root.join(".ess-output/state.json")).unwrap(),
    )
    .unwrap()["payload"]
        .clone()
}

/// A settled `ess-output-state/3` checkpoint records neither the root nor its directory identity
/// (beyond10x/ess#484), so no checkout commits another one's location.
fn unbound(root: &Path) -> bool {
    let payload = payload(root);
    payload["format"] == "ess-output-state/3"
        && payload.get("root").is_none()
        && payload.get("directory").is_none()
}

/// Commit a generated crate (with `.ess-output`) into a repository and return (spec, repo).
fn committed_repository(f: &Fixture) -> (PathBuf, PathBuf) {
    let spec = f.0.join("spec");
    copy_tree(&workspace().join("examples/gatepass"), &spec);
    let repo = f.0.join("origin");
    fs::create_dir(&repo).unwrap();
    git("022", &repo, &["init", "-q"]);
    let generated = synthesize(&spec, &repo.join("crate"));
    assert!(generated.status.success(), "{generated:?}");
    fs::write(
        repo.join("AUTHORED.md"),
        "authored beside generated output\n",
    )
    .unwrap();
    git("022", &repo, &["add", "-A"]);
    git("022", &repo, &["commit", "-q", "-m", "generated output"]);
    (spec, repo)
}

fn porcelain(repo: &Path) -> String {
    String::from_utf8(git("022", repo, &["status", "--porcelain"]).stdout).unwrap()
}

/// The design document says mode is not compared "since checkouts apply different umasks", and
/// that a regeneration that changes nothing leaves the committed `state.json` byte-identical "so a
/// regenerate-and-diff check stays clean". A clone made under the common group-writable umask
/// 002 is exactly such a checkout.
#[test]
fn a_clone_under_another_umask_regenerates_unchanged_with_a_clean_diff() {
    let _serial = serial();
    let f = Fixture::new();
    let (spec, repo) = committed_repository(&f);
    let clone = f.0.join("clone-002");
    git(
        "002",
        &f.0,
        &[
            "clone",
            "-q",
            repo.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let regenerated = synthesize(&spec, &clone.join("crate"));
    assert!(
        regenerated.status.success(),
        "{}",
        String::from_utf8_lossy(&regenerated.stderr)
    );
    let status = porcelain(&clone);
    assert_eq!(
        status,
        "",
        "an unchanged regeneration in a umask-002 clone dirtied the checkout; git diff:\n{}",
        String::from_utf8_lossy(&git("022", &clone, &["diff", "--stat"]).stdout)
    );
}

/// The CI flow, in the default umask: clone, regenerate, diff clean; then a specification change
/// regenerates, and `state.json` changes with the outputs and records no root.
#[test]
fn a_clone_regenerates_clean_and_a_specification_change_records_no_root() {
    let _serial = serial();
    let f = Fixture::new();
    let (spec, repo) = committed_repository(&f);
    let clone = f.0.join("clone-022");
    git(
        "022",
        &f.0,
        &[
            "clone",
            "-q",
            repo.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let crate_dir = clone.join("crate");
    let regenerated = synthesize(&spec, &crate_dir);
    assert!(regenerated.status.success(), "{regenerated:?}");
    assert_eq!(
        porcelain(&clone),
        "",
        "unchanged regeneration dirtied a clone"
    );
    git("022", &clone, &["diff", "--exit-code"]);
    assert!(unbound(&crate_dir), "{}", payload(&crate_dir));

    let domain = spec.join("domains/visit.yaml");
    let text = fs::read_to_string(&domain).unwrap();
    assert!(text.contains("variants: [North, South, Annex]"));
    fs::write(
        &domain,
        text.replace(
            "variants: [North, South, Annex]",
            "variants: [North, South, Annex, West]",
        ),
    )
    .unwrap();
    let changed = synthesize(&spec, &crate_dir);
    assert!(changed.status.success(), "{changed:?}");
    let status = porcelain(&clone);
    assert!(
        status.contains(" M crate/.ess-output/state.json"),
        "{status}"
    );
    assert!(
        status
            .lines()
            .any(|l| l.starts_with(" M crate/") && !l.contains(".ess-output")),
        "a specification change regenerated no output: {status}"
    );
    assert!(unbound(&crate_dir), "{}", payload(&crate_dir));
    assert_eq!(payload(&crate_dir)["checkpoint"]["phase"], "Idle");
    assert_eq!(
        fs::read(clone.join("AUTHORED.md")).unwrap(),
        b"authored beside generated output\n"
    );
}

/// Follow the adoption route an Idle mismatch refusal prints, exactly as printed, in a clone whose
/// owned file was hand-edited and committed. It must end with a root that regenerates and with
/// the authored file intact.
#[test]
fn the_adoption_route_an_idle_refusal_prints_ends_in_a_working_root() {
    let _serial = serial();
    let f = Fixture::new();
    let (spec, repo) = committed_repository(&f);
    fs::write(
        repo.join("crate/src/lib.rs"),
        "// hand-edited and committed\n",
    )
    .unwrap();
    git(
        "022",
        &repo,
        &["commit", "-q", "-am", "edit generated output"],
    );
    let clone = f.0.join("clone");
    git(
        "022",
        &f.0,
        &[
            "clone",
            "-q",
            repo.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let root = clone.join("crate");
    let refused = synthesize(&spec, &root);
    assert!(!refused.status.success(), "{refused:?}");
    let stderr = String::from_utf8_lossy(&refused.stderr).into_owned();
    let remove = format!("Remove {}", root.join(".ess-output").display());
    assert!(stderr.contains(&remove), "{stderr}");
    let printed = format!(
        "ess generate output adopt --ownership-root {} --from <fresh reference> --owner <family>",
        root.display()
    );
    assert!(stderr.contains(&printed), "{stderr}");

    // As printed: move the differing file aside, remove the state, generate a fresh
    // reference, adopt. (Coordinator, correction 2: the printed route starts with the move.)
    assert!(stderr.contains("src/lib.rs"), "{stderr}");
    fs::rename(root.join("src/lib.rs"), f.0.join("lib.rs.aside")).unwrap();
    fs::remove_dir_all(root.join(".ess-output")).unwrap();
    let reference = f.0.join("reference");
    let fresh = synthesize(&spec, &reference);
    assert!(fresh.status.success(), "{fresh:?}");
    let adopted = ess(&[
        "generate".as_ref(),
        "output".as_ref(),
        "adopt".as_ref(),
        "--ownership-root".as_ref(),
        root.as_os_str(),
        "--from".as_ref(),
        reference.as_os_str(),
        "--owner".as_ref(),
        "synthesis".as_ref(),
    ]);
    assert!(
        adopted.status.success(),
        "the adoption the refusal names refused: {}",
        String::from_utf8_lossy(&adopted.stderr)
    );
    let working = synthesize(&spec, &root);
    assert!(
        working.status.success(),
        "after the printed route the root still does not regenerate: {}",
        String::from_utf8_lossy(&working.stderr)
    );
    assert_eq!(
        fs::read(clone.join("AUTHORED.md")).unwrap(),
        b"authored beside generated output\n"
    );
}

/// The same-path copy refusal (correction 1) also prints "remove .ess-output, then adopt". Follow
/// it and confirm the root works afterwards and keeps authored files.
#[test]
fn the_adoption_route_a_same_path_copy_refusal_prints_ends_in_a_working_root() {
    let _serial = serial();
    let f = Fixture::new();
    let a = f.0.join("a");
    let copy = f.0.join("a.copy");
    ownership::probe::publish(&a, OLD, &mut |_| Ok(())).unwrap();
    fs::write(a.join("AUTHORED.md"), "authored").unwrap();
    let cut = ownership::probe::publish(&a, NEW, &mut |event| {
        if event == "after:rename:output-installation" {
            anyhow::bail!("cut at {event}")
        }
        Ok(())
    });
    assert!(cut.is_err());
    copy_tree(&a, &copy);
    fs::remove_dir_all(&a).unwrap();
    fs::rename(&copy, &a).unwrap();
    let refusal = format!(
        "{:#}",
        ownership::probe::publish(&a, NEW, &mut |_| Ok(())).unwrap_err()
    );
    assert!(
        refusal.contains(&format!("remove {}", a.join(".ess-output").display())),
        "{refusal}"
    );

    // The printed route starts by moving the listed files aside (coordinator, correction 2).
    assert!(refusal.contains("same"), "{refusal}");
    fs::rename(a.join("same"), f.0.join("same.aside")).unwrap();
    fs::remove_dir_all(a.join(".ess-output")).unwrap();
    let reference = f.0.join("reference");
    ownership::probe::publish(&reference, NEW, &mut |_| Ok(())).unwrap();
    ownership::probe::adopt(&a, &reference, &mut |_| Ok(()))
        .expect("the adoption the refusal names");
    ownership::probe::publish(&a, NEW, &mut |_| Ok(()))
        .expect("the root regenerates after the printed route");
    assert_eq!(fs::read(a.join("same")).unwrap(), b"replacement");
    assert_eq!(fs::read(a.join("new/leaf")).unwrap(), b"introduced");
    assert_eq!(fs::read(a.join("AUTHORED.md")).unwrap(), b"authored");
}

/// Each foreign root holds something at an owned path that is not the recorded bytes; generation
/// must refuse and leave the root exactly as it was.
#[test]
fn a_copied_ledger_refuses_every_non_matching_owned_path_without_writing() {
    type Shape = fn(&Path, &Path);
    let _serial = serial();
    let shapes: &[(&str, Shape)] = &[
        ("same length, other bytes", |b, _| {
            assert_eq!("original".len(), "ORIGINAL".len());
            fs::write(b.join("same"), "ORIGINAL").unwrap();
        }),
        ("retired path, same length, other bytes", |b, _| {
            fs::write(b.join("retired/old"), "WITHDRAWN").unwrap();
        }),
        ("owned path is a directory", |b, _| {
            fs::remove_file(b.join("same")).unwrap();
            fs::create_dir(b.join("same")).unwrap();
            fs::write(b.join("same/inside"), "authored").unwrap();
        }),
        ("owned path is a symlink to matching bytes", |b, outside| {
            fs::write(outside, "original").unwrap();
            fs::remove_file(b.join("same")).unwrap();
            std::os::unix::fs::symlink(outside, b.join("same")).unwrap();
        }),
        (
            "owned parent is a symlink to a matching tree",
            |b, outside| {
                fs::create_dir(outside).unwrap();
                fs::write(outside.join("old"), "withdrawn").unwrap();
                fs::remove_dir_all(b.join("retired")).unwrap();
                std::os::unix::fs::symlink(outside, b.join("retired")).unwrap();
            },
        ),
    ];
    for (name, shape) in shapes {
        let f = Fixture::new();
        let a = f.0.join("a");
        let b = f.0.join("b");
        let outside = f.0.join("outside");
        ownership::probe::publish(&a, OLD, &mut |_| Ok(())).unwrap();
        copy_tree(&a, &b);
        shape(&b, &outside);
        // The whole fixture: the root and anything outside it a symlink points at.
        let before = snapshot(&f.0);
        let result = ownership::probe::publish(&b, NEW, &mut |_| Ok(()));
        assert!(
            result.is_err(),
            "{name}: a non-matching owned path was admitted"
        );
        assert_eq!(snapshot(&f.0), before, "{name}: refusal wrote");
        // The read-only check reads an owned file under a symlinked parent as absent and admits;
        // it writes nothing, so only the publishing refusal above is asserted for that shape.
        if !name.starts_with("owned parent is a symlink") {
            assert!(ownership::check(&b).is_err(), "{name}: check admitted");
        }
    }
}

/// An owned file's parent is an authored regular file in the new root: the reader treats the owned
/// file as absent and admits; retiring it must not delete the authored file.
#[test]
fn an_authored_file_where_an_owned_directory_was_survives_retirement() {
    let _serial = serial();
    let f = Fixture::new();
    let a = f.0.join("a");
    let b = f.0.join("b");
    ownership::probe::publish(&a, OLD, &mut |_| Ok(())).unwrap();
    copy_tree(&a, &b);
    fs::remove_dir_all(b.join("retired")).unwrap();
    fs::write(b.join("retired"), "authored where a directory was").unwrap();
    let _ = ownership::probe::publish(&b, NEW, &mut |_| Ok(()));
    assert_eq!(
        fs::read(b.join("retired")).unwrap(),
        b"authored where a directory was"
    );
}

/// Several operations in one process, across three roots, each leave a settled record that names
/// no root, neither their own nor the one the copy came from.
#[test]
fn operations_in_one_process_record_no_root() {
    let _serial = serial();
    let f = Fixture::new();
    let a = f.0.join("a");
    let b = f.0.join("b");
    let c = f.0.join("c");
    ownership::probe::publish(&a, OLD, &mut |_| Ok(())).unwrap();
    copy_tree(&a, &b);
    ownership::probe::publish(&b, OLD, &mut |_| Ok(())).unwrap();
    drop(ownership::check(&b).unwrap());
    ownership::recover(&b).unwrap();
    ownership::probe::publish(&a, OLD, &mut |_| Ok(())).unwrap();
    assert!(unbound(&b), "{}", payload(&b));
    ownership::probe::publish(&b, NEW, &mut |_| Ok(())).unwrap();
    assert!(unbound(&b), "{}", payload(&b));
    assert!(unbound(&a), "{}", payload(&a));
    copy_tree(&b, &c);
    ownership::probe::publish(&c, OLD, &mut |_| Ok(())).unwrap();
    assert!(unbound(&c), "{}", payload(&c));
    assert!(unbound(&b), "{}", payload(&b));
    assert!(!c.join("new/leaf").exists());
    assert_eq!(fs::read(c.join("same")).unwrap(), b"original");
    // Back in the original root, the first state still governs.
    ownership::probe::publish(&a, NEW, &mut |_| Ok(())).unwrap();
    assert!(!a.join("retired/old").exists());
    assert!(unbound(&a), "{}", payload(&a));
}
