//! A file-shaped conformance `--out` creates its missing parent directories, as the directory
//! targets and `ess generate --out` do (beyond10x/ess#78). The existing-ancestor checks every
//! output root gets still apply.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ess-conform-parent-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn billing() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|path| path.join("examples/billing").is_dir())
        .unwrap()
        .join("examples/billing")
}

fn ess(args: &[&str], out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .arg("--path")
        .arg(billing())
        .arg("--out")
        .arg(out)
        .output()
        .unwrap()
}

fn assert_written(output: &Output, out: &Path) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = fs::read_to_string(out).expect("the suite was written");
    assert!(text.starts_with('{'), "{text}");
}

#[test]
fn synthesize_ir_creates_the_missing_parent_of_its_out_file() {
    let scratch = Scratch::new();
    let out = scratch.0.join("missing/deeper/suite.json");
    let output = ess(&["verify", "conform", "synthesize", "--target", "ir"], &out);
    assert_written(&output, &out);
}

#[test]
fn synthesize_ir_at_suite_format_5_creates_the_missing_parent_of_its_out_file() {
    let scratch = Scratch::new();
    let out = scratch.0.join("missing/suite.json");
    let output = ess(
        &["verify", "conform", "synthesize", "--suite-format", "5"],
        &out,
    );
    assert_written(&output, &out);
}

#[test]
fn author_creates_the_missing_parent_of_its_out_file() {
    let scratch = Scratch::new();
    let out = scratch.0.join("missing/authored.json");
    let output = ess(&["verify", "conform", "author"], &out);
    assert_written(&output, &out);
}

#[cfg(unix)]
#[test]
fn a_symlinked_ancestor_is_still_refused_and_nothing_is_created_through_it() {
    let scratch = Scratch::new();
    let elsewhere = scratch.0.join("elsewhere");
    fs::create_dir(&elsewhere).unwrap();
    std::os::unix::fs::symlink(&elsewhere, scratch.0.join("link")).unwrap();
    let out = scratch.0.join("link/missing/suite.json");
    let output = ess(&["verify", "conform", "synthesize", "--target", "ir"], &out);
    assert!(
        !output.status.success(),
        "a symlinked ancestor was followed"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("symlink"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!elsewhere.join("missing").exists());
}
