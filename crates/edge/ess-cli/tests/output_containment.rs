//! Output refusal is decided before any file in the destination set changes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ess-output-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("out")).unwrap();
        fs::write(path.join("page.md"), "# Authored page\n\nKept verbatim.\n").unwrap();
        fs::write(path.join("out/index.html"), "inside sentinel").unwrap();
        fs::write(path.join("escaped.html"), "outside sentinel").unwrap();
        Self(path)
    }

    fn site(&self, ids: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command.args(["generate", "--path"]);
        command.arg(workspace_root().join("examples/billing"));
        command.args(["--kind", "site", "--out"]);
        command.arg(self.0.join("out"));
        for id in ids {
            command
                .arg("--include")
                .arg(format!("{id}={}", self.0.join("page.md").display()));
        }
        command.output().unwrap()
    }

    fn assert_refused_without_writes(&self, output: &Output) {
        assert_eq!(
            fs::read_to_string(self.0.join("escaped.html")).unwrap(),
            "outside sentinel"
        );
        assert_eq!(
            fs::read_to_string(self.0.join("out/index.html")).unwrap(),
            "inside sentinel"
        );
        assert!(
            !output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|path| path.join("examples/billing").is_dir())
        .unwrap()
        .to_path_buf()
}

#[test]
fn an_escaping_include_is_refused_before_any_output_changes() {
    let fixture = Fixture::new();
    let output = fixture.site(&["../escaped"]);
    fixture.assert_refused_without_writes(&output);
}

#[test]
fn include_aliases_and_duplicate_generated_pages_are_refused_before_writing() {
    for ids in [
        vec!["index"],
        vec!["plan/board", "plan/board"],
        vec!["plan/board", "PLAN/BOARD"],
        vec!["plan.html/board", "plan"],
        vec!["domains/billing-invoice"],
    ] {
        let fixture = Fixture::new();
        let output = fixture.site(&ids);
        fixture.assert_refused_without_writes(&output);
    }
}

#[test]
fn noncanonical_and_platform_paths_are_refused_before_writing() {
    for id in [
        "",
        "./plan",
        "plan//board",
        "plan/../escaped",
        "C:/escaped",
        "C:escaped",
        "\\\\server\\share",
        "plan\\board",
        "NUL",
        "COM1/page",
        "plan./board",
        "plan /board",
    ] {
        let fixture = Fixture::new();
        let output = fixture.site(&[id]);
        fixture.assert_refused_without_writes(&output);
    }
    let fixture = Fixture::new();
    let absolute = fixture.0.join("escaped");
    let output = fixture.site(&[absolute.to_str().unwrap()]);
    fixture.assert_refused_without_writes(&output);
}

#[test]
fn a_valid_nested_include_keeps_the_existing_site_layout_and_bytes() {
    let fixture = Fixture::new();
    let output = fixture.site(&["plan/board"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(fixture.0.join("out/plan/board.html").is_file());
    assert!(fixture.0.join("out/assets/style.css").is_file());
    assert!(!fixture.0.join("out/site").exists());
    let actual = fs::read(fixture.0.join("out/index.html")).unwrap();
    let second = fixture.site(&["plan/board"]);
    assert!(second.status.success());
    assert_eq!(fs::read(fixture.0.join("out/index.html")).unwrap(), actual);
    assert_eq!(
        fs::read_to_string(fixture.0.join("escaped.html")).unwrap(),
        "outside sentinel"
    );
}

#[cfg(unix)]
#[test]
fn symlink_roots_parents_and_destinations_are_refused_before_writing() {
    use std::os::unix::fs::symlink;
    for position in ["root", "parent", "destination", "dangling"] {
        let fixture = Fixture::new();
        let outside = fixture.0.join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("board.html"), "outside page").unwrap();
        match position {
            "root" => {
                fs::rename(fixture.0.join("out"), outside.join("root")).unwrap();
                symlink(outside.join("root"), fixture.0.join("out")).unwrap();
            }
            "parent" => symlink(&outside, fixture.0.join("out/plan")).unwrap(),
            "destination" => {
                fs::create_dir(fixture.0.join("out/plan")).unwrap();
                symlink(
                    outside.join("board.html"),
                    fixture.0.join("out/plan/board.html"),
                )
                .unwrap();
            }
            "dangling" => symlink(outside.join("absent"), fixture.0.join("out/plan")).unwrap(),
            _ => unreachable!(),
        }
        let output = fixture.site(&["plan/board"]);
        fixture.assert_refused_without_writes(&output);
        assert_eq!(
            fs::read_to_string(outside.join("board.html")).unwrap(),
            "outside page"
        );
    }
}

#[cfg(unix)]
#[test]
fn a_hardlinked_destination_is_refused_before_other_files_change() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("out/plan")).unwrap();
    fs::hard_link(
        fixture.0.join("escaped.html"),
        fixture.0.join("out/plan/board.html"),
    )
    .unwrap();
    let output = fixture.site(&["plan/board"]);
    fixture.assert_refused_without_writes(&output);
}
