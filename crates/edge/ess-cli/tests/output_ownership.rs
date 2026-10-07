//! Native generated-output ownership and recovery, without Go or browser dependencies.
#[allow(dead_code)]
#[path = "../src/output_ownership/mod.rs"]
mod ownership;
mod ownership_admission;
mod ownership_protocol;
mod ownership_relocation;
mod ownership_relocation_adversary;
mod ownership_root_lock;
mod ownership_routes;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
// A subprocess spawn may briefly inherit another thread's open flock descriptors before
// CLOEXEC closes them. Serialize this target's fixtures and child lifetimes so a recovery
// assertion cannot race that unrelated inheritance. Explicit contention still happens
// inside its dedicated test, using independently opened native directory descriptors.
fn serial() -> std::sync::MutexGuard<'static, ()> {
    static TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
    TEST.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
impl Drop for Fixture {
    /// Some cases leave read-only directories behind; restore owner access, then remove the tree.
    fn drop(&mut self) {
        fn writable(path: &Path) {
            use std::os::unix::fs::PermissionsExt;
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
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "ess-ownership-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(
            root.join("page.md"),
            "# Authored guide\n\nPreserve this source.\n",
        )
        .unwrap();
        Self(root)
    }
    fn site(&self, include: bool) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_ess"));
        c.args(["generate", "--path"])
            .arg(workspace().join("examples/billing"))
            .args(["--kind", "site", "--out"])
            .arg(self.0.join("out"));
        if include {
            c.arg("--include")
                .arg(format!("guide={}", self.0.join("page.md").display()));
        }
        c.output().unwrap()
    }
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("examples/billing").is_dir())
        .unwrap()
        .to_path_buf()
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(path).unwrap() {
            let e = e.unwrap();
            let p = e.path();
            assert!(!e.file_type().unwrap().is_symlink());
            if p.is_dir() {
                out.insert(p.strip_prefix(root).unwrap().to_path_buf(), Vec::new());
                visit(root, &p, out);
            } else {
                out.insert(
                    p.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

#[test]
fn ordinary_generation_refuses_existing_unowned_destination_before_any_write() {
    let _serial = serial();
    let f = Fixture::new();
    fs::create_dir(f.0.join("out")).unwrap();
    fs::write(
        f.0.join("out/index.html"),
        b"authored, despite the generated-looking name",
    )
    .unwrap();
    let before = snapshot(&f.0);
    let output = f.site(false);
    assert!(
        !output.status.success(),
        "unowned destination was replaced: {output:?}"
    );
    assert_eq!(snapshot(&f.0), before);
}

#[test]
fn selected_site_regeneration_retires_withdrawn_copy_and_keeps_authored_neighbors() {
    let _serial = serial();
    let f = Fixture::new();
    let first = f.site(true);
    assert!(first.status.success(), "{first:?}");
    assert!(f.0.join("out/guide.html").is_file());
    fs::write(f.0.join("out/skin.js"), "authored skin").unwrap();
    let source = fs::read(f.0.join("page.md")).unwrap();
    let next = f.site(false);
    assert!(next.status.success(), "{next:?}");
    assert!(
        !f.0.join("out/guide.html").exists(),
        "withdrawn owned copy remained stale"
    );
    assert_eq!(fs::read(f.0.join("out/skin.js")).unwrap(), b"authored skin");
    assert_eq!(fs::read(f.0.join("page.md")).unwrap(), source);
}

#[test]
fn identical_generation_keeps_complete_enrollment_bytes_and_repairs_owned_edits() {
    let _serial = serial();
    let f = Fixture::new();
    let first = f.site(false);
    assert!(first.status.success(), "{first:?}");
    assert!(
        f.0.join("out/.ess-output/state.json").is_file(),
        "generation did not enroll its output"
    );
    let before = snapshot(&f.0);
    let again = f.site(false);
    assert!(again.status.success(), "{again:?}");
    assert_eq!(snapshot(&f.0), before);
    let expected = fs::read(f.0.join("out/index.html")).unwrap();
    fs::write(f.0.join("out/index.html"), "edited owned output").unwrap();
    fs::remove_file(f.0.join("out/assets/style.css")).unwrap();
    let repair = f.site(false);
    assert!(repair.status.success(), "{repair:?}");
    assert_eq!(fs::read(f.0.join("out/index.html")).unwrap(), expected);
    assert!(f.0.join("out/assets/style.css").is_file());
}

/// Set one extended attribute on an existing output, as a person or another program would.
fn set_label(path: &Path, name: &str, value: &[u8]) -> rustix::io::Result<()> {
    let file = fs::File::open(path).unwrap();
    rustix::fs::fsetxattr(&file, name, value, rustix::fs::XattrFlags::empty())
}

#[test]
fn foreign_attribute_refusal_names_output_path_and_ess_version() {
    let _serial = serial();
    let f = Fixture::new();
    let first = f.site(false);
    assert!(first.status.success(), "{first:?}");
    match set_label(&f.0.join("out/index.html"), "user.ess_test", b"1") {
        Ok(()) => {}
        Err(rustix::io::Errno::NOTSUP) => {
            eprintln!("not run: this filesystem carries no user attributes");
            return;
        }
        Err(error) => panic!("setting user.ess_test for the test: {error}"),
    }
    let before = snapshot(&f.0);
    let again = f.site(false);
    assert!(
        !again.status.success(),
        "a foreign attribute refuses regeneration: {again:?}"
    );
    let stderr = String::from_utf8_lossy(&again.stderr);
    for expected in [
        "user.ess_test".to_owned(),
        "index.html".to_owned(),
        format!("ess {}", env!("CARGO_PKG_VERSION")),
    ] {
        assert!(
            stderr.contains(&expected),
            "the refusal names {expected}: {stderr}"
        );
    }
    assert_eq!(snapshot(&f.0), before, "nothing was written");
}

/// macOS attaches `com.apple.provenance` to files some processes write. Regenerating committed
/// output over such a file succeeds and leaves it unchanged. A refused `setxattr` fails the case:
/// the label must be on the file for the case to decide anything.
#[cfg(target_os = "macos")]
#[test]
fn macos_regenerates_over_os_provenance_label() {
    let _serial = serial();
    let f = Fixture::new();
    let first = f.site(false);
    assert!(first.status.success(), "{first:?}");
    let target = f.0.join("out/index.html");
    set_label(
        &target,
        "com.apple.provenance",
        b"\x01\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00",
    )
    .expect("setting com.apple.provenance on a generated output");
    let before = snapshot(&f.0);
    let again = f.site(false);
    assert!(
        again.status.success(),
        "regeneration over an OS provenance label succeeds: {again:?}"
    );
    assert_eq!(snapshot(&f.0), before, "the output is unchanged");
}

#[test]
fn requires_pin_answers_producing_release() {
    let page = fs::read_to_string(workspace().join("website/docs/guides/generate-artifacts.md"))
        .expect("generate-artifacts.md");
    let start = page
        .find("\n## Repeated generation and recovery\n")
        .expect("the section `## Repeated generation and recovery`");
    let rest = &page[start + 1..];
    let section = &rest[..rest[3..].find("\n## ").map_or(rest.len(), |end| end + 3)];
    for phrase in [
        "`requires: ess X.Y.Z`",
        "producing release",
        "specify/layout-and-validation.md",
    ] {
        assert!(
            section.contains(phrase),
            "`## Repeated generation and recovery` is missing {phrase}"
        );
    }
}

/// An output of the wrong type is refused naming it once, in the `ess <version> refused output`
/// context, and not a second time in the cause.
#[test]
fn an_incompatible_output_type_is_named_once() {
    let _serial = serial();
    let f = Fixture::new();
    let first = f.site(false);
    assert!(first.status.success(), "{first:?}");
    let target = f.0.join("out/index.html");
    fs::remove_file(&target).unwrap();
    std::os::unix::fs::symlink(f.0.join("page.md"), &target).unwrap();
    let again = f.site(false);
    assert!(
        !again.status.success(),
        "a symlinked output is refused: {again:?}"
    );
    let stderr = String::from_utf8_lossy(&again.stderr);
    let expected = format!(
        "ess {} refused output index.html: output path has an incompatible file type or symlink",
        env!("CARGO_PKG_VERSION")
    );
    assert!(stderr.contains(&expected), "{expected}: {stderr}");
    assert_eq!(
        stderr.matches("index.html").count(),
        1,
        "the refusal names the output once: {stderr}"
    );
}
