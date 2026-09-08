//! Native generated-output ownership and recovery, without Go or browser dependencies.
#[allow(dead_code)]
#[path = "../src/output_ownership/mod.rs"]
mod ownership;
mod ownership_protocol;
mod ownership_routes;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
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
