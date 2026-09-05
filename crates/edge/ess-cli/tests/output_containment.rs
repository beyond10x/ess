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

fn composition_command() -> Command {
    let fixtures = workspace_root().join("crates/specify/ess-composition/tests/fixtures");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command.args(["compose", "--path"]);
    command.arg(fixtures.join("compositions/workbench.yaml"));
    for key in ["todo", "usage"] {
        command.arg("--service").arg(format!(
            "{key}={}",
            fixtures.join("two-components").display()
        ));
    }
    command
}

fn output_snapshot(root: &Path) -> std::collections::BTreeMap<PathBuf, (String, Vec<u8>)> {
    fn visit(
        root: &Path,
        path: &Path,
        into: &mut std::collections::BTreeMap<PathBuf, (String, Vec<u8>)>,
    ) {
        let metadata = fs::symlink_metadata(path).unwrap();
        let (kind, bytes) = if metadata.file_type().is_symlink() {
            (
                "symlink",
                fs::read_link(path)
                    .unwrap()
                    .as_os_str()
                    .as_encoded_bytes()
                    .to_vec(),
            )
        } else if metadata.is_dir() {
            ("directory", Vec::new())
        } else {
            ("file", fs::read(path).unwrap())
        };
        into.insert(
            path.strip_prefix(root).unwrap().to_path_buf(),
            (kind.to_owned(), bytes),
        );
        if metadata.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), into);
            }
        }
    }
    let mut snapshot = std::collections::BTreeMap::new();
    visit(root, root, &mut snapshot);
    snapshot
}

#[test]
fn composition_companions_form_one_output_set_even_without_a_generated_tree() {
    let mut violations = Vec::new();
    for rust in [false, true] {
        for pair in [
            ["same.json", "same.json"],
            ["same.json", "SAME.JSON"],
            ["same.json", "working/../same.json"],
            ["new-parent", "new-parent/child.json"],
            ["new-parent/child.json", "new-parent"],
            ["same.json", "missing/child.json"],
        ] {
            let fixture = Fixture::new();
            fs::create_dir(fixture.0.join("working")).unwrap();
            fs::write(fixture.0.join("same.json"), "companion sentinel").unwrap();
            let before = output_snapshot(&fixture.0);
            let mut command = composition_command();
            command
                .current_dir(&fixture.0)
                .arg("--out")
                .arg(pair[0])
                .arg("--client-plan-out")
                .arg(pair[1]);
            if rust {
                command.args(["--client-rust-out", "out"]);
            }
            let output = command.output().unwrap();
            if output.status.success() || output_snapshot(&fixture.0) != before {
                violations.push(format!(
                    "{pair:?}, rust={rust}: exit {:?}, output changed",
                    output.status.code()
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn composition_preflight_includes_companion_generated_aliases_and_both_companions() {
    let mut violations = Vec::new();
    for flag in ["--out", "--client-plan-out"] {
        for collision in [
            "out/cargo.TOML",
            "out/working/../Cargo.toml",
            "out/src/lib.rs",
            "out/src/lib.rs/child",
            "out/SRC/other.json",
            "out",
        ] {
            let fixture = Fixture::new();
            fs::create_dir(fixture.0.join("out/working")).unwrap();
            fs::create_dir(fixture.0.join(if collision == "out/SRC/other.json" {
                "out/SRC"
            } else {
                "out/src"
            }))
            .unwrap();
            fs::write(fixture.0.join("out/Cargo.toml"), "manifest sentinel").unwrap();
            fs::write(fixture.0.join("other.json"), "other companion sentinel").unwrap();
            let before = output_snapshot(&fixture.0);
            let other = if flag == "--out" {
                "--client-plan-out"
            } else {
                "--out"
            };
            let output = composition_command()
                .current_dir(&fixture.0)
                .arg(flag)
                .arg(collision)
                .args([other, "other.json", "--client-rust-out", "out"])
                .output()
                .unwrap();
            if output.status.success() || output_snapshot(&fixture.0) != before {
                violations.push(format!(
                    "{flag} {collision}: exit {:?}, output changed",
                    output.status.code()
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn composition_keeps_disjoint_caller_selected_filenames_and_parent_roots() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("working")).unwrap();
    fs::create_dir(fixture.0.join("reports ü")).unwrap();
    let mut command = composition_command();
    command.current_dir(fixture.0.join("working")).args([
        "--out",
        "../reports ü/composition +copy.json",
        "--client-plan-out",
        "../reports ü/résumé report.json",
        "--client-rust-out",
        "../out",
    ]);
    let first = command.output().unwrap();
    assert!(first.status.success(), "{first:?}");
    let before = output_snapshot(&fixture.0);
    let second = command.output().unwrap();
    assert!(second.status.success(), "{second:?}");
    assert_eq!(before, output_snapshot(&fixture.0));
    let composition: serde_json::Value = serde_json::from_slice(
        &fs::read(fixture.0.join("reports ü/composition +copy.json")).unwrap(),
    )
    .unwrap();
    let plan: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.0.join("reports ü/résumé report.json")).unwrap())
            .unwrap();
    assert_ne!(composition, plan);
    assert!(fixture.0.join("out/src/lib.rs").is_file());
}

#[test]
fn composition_does_not_reinterpret_directory_spelling_as_a_named_output_file() {
    let mut violations = Vec::new();
    for flag in ["--out", "--client-plan-out"] {
        for directory in ["new-file/", "existing/.", ".", "existing/.."] {
            let fixture = Fixture::new();
            fs::create_dir(fixture.0.join("existing")).unwrap();
            fs::write(fixture.0.join("other.json"), "other companion sentinel").unwrap();
            let before = output_snapshot(&fixture.0);
            let other = if flag == "--out" {
                "--client-plan-out"
            } else {
                "--out"
            };
            let output = composition_command()
                .current_dir(&fixture.0)
                .args([
                    flag,
                    directory,
                    other,
                    "other.json",
                    "--client-rust-out",
                    "out",
                ])
                .output()
                .unwrap();
            if output.status.success() || output_snapshot(&fixture.0) != before {
                violations.push(format!(
                    "{flag} {directory}: exit {:?}, output changed",
                    output.status.code()
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[cfg(unix)]
#[test]
fn composition_refuses_companion_links_before_any_other_output_changes() {
    for flag in ["--out", "--client-plan-out"] {
        for hard in [false, true] {
            let fixture = Fixture::new();
            fs::write(fixture.0.join("other.json"), "other companion sentinel").unwrap();
            if hard {
                fs::hard_link(
                    fixture.0.join("escaped.html"),
                    fixture.0.join("linked.json"),
                )
                .unwrap();
            } else {
                std::os::unix::fs::symlink(
                    fixture.0.join("escaped.html"),
                    fixture.0.join("linked.json"),
                )
                .unwrap();
            }
            let before = output_snapshot(&fixture.0);
            let other = if flag == "--out" {
                "--client-plan-out"
            } else {
                "--out"
            };
            let output = composition_command()
                .current_dir(&fixture.0)
                .args([
                    flag,
                    "linked.json",
                    other,
                    "other.json",
                    "--client-rust-out",
                    "out",
                ])
                .output()
                .unwrap();
            assert!(!output.status.success(), "{flag}, hard={hard}: {output:?}");
            assert_eq!(before, output_snapshot(&fixture.0));
        }
    }
}

#[test]
fn composition_companion_outputs_cannot_collide_with_the_generated_client_tree() {
    let control = Fixture::new();
    let valid = composition_command()
        .arg("--out")
        .arg(control.0.join("composition.json"))
        .arg("--client-plan-out")
        .arg(control.0.join("client-plan.json"))
        .arg("--client-rust-out")
        .arg(control.0.join("out"))
        .output()
        .unwrap();
    assert!(valid.status.success(), "valid fixture: {valid:?}");
    assert!(control.0.join("out/Cargo.toml").is_file());
    assert!(control.0.join("out/src/lib.rs").is_file());

    let mut violations = Vec::new();
    for flag in ["--out", "--client-plan-out"] {
        for collision in ["Cargo.toml", "src"] {
            let fixture = Fixture::new();
            fs::write(fixture.0.join("out/Cargo.toml"), "client manifest sentinel").unwrap();
            let output = composition_command()
                .arg(flag)
                .arg(fixture.0.join("out").join(collision))
                .arg("--client-rust-out")
                .arg(fixture.0.join("out"))
                .output()
                .unwrap();
            let manifest_preserved =
                fs::read(fixture.0.join("out/Cargo.toml")).unwrap() == b"client manifest sentinel";
            let source_parent_absent = !fixture.0.join("out/src").exists();
            if output.status.success() || !manifest_preserved || !source_parent_absent {
                violations.push(format!(
                    "{flag} out/{collision}: exit {:?}, client manifest preserved \
                     {manifest_preserved}, source parent absent {source_parent_absent}; stderr: {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "companion/generated destination collisions must refuse before writes:\n{}",
        violations.join("\n")
    );
}

fn site_at(fixture: &Fixture, root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(&fixture.0)
        .args(["generate", "--path"])
        .arg(workspace_root().join("examples/billing"))
        .args(["--kind", "site", "--out"])
        .arg(root)
        .output()
        .unwrap()
}

#[test]
fn requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files() {
    let fixture = Fixture::new();
    let ordinary = fixture.site(&[]);
    assert!(ordinary.status.success());
    let bytes = fs::read(fixture.0.join("out/index.html")).unwrap();
    fs::create_dir(fixture.0.join("working")).unwrap();
    let parent = site_at(&fixture, Path::new("working/../out"));
    assert!(parent.status.success(), "parent root: {parent:?}");
    let missing = site_at(&fixture, Path::new("absent/../out"));
    assert!(
        missing.status.success(),
        "normalized missing root: {missing:?}"
    );
    assert!(!fixture.0.join("absent").exists());
    assert_eq!(fs::read(fixture.0.join("out/index.html")).unwrap(), bytes);

    fs::write(fixture.0.join("not-a-directory"), "root obstacle").unwrap();
    let invalid = site_at(&fixture, Path::new("not-a-directory/../out"));
    assert!(!invalid.status.success());
    assert_eq!(fs::read(fixture.0.join("out/index.html")).unwrap(), bytes);
    assert_eq!(
        fs::read(fixture.0.join("not-a-directory")).unwrap(),
        b"root obstacle"
    );
}

#[test]
fn late_site_asset_aliases_refuse_before_even_creating_output_directories() {
    for ids in [
        vec!["Assets/extra"],
        vec!["assets/mermaid.min.js/child"],
        vec!["nested/first", "Nested/second"],
    ] {
        let fixture = Fixture::new();
        let output = fixture.site(&ids);
        fixture.assert_refused_without_writes(&output);
        assert_eq!(fs::read_dir(fixture.0.join("out")).unwrap().count(), 1);
    }
}
