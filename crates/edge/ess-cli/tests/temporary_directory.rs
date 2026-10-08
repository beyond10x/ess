//! Test scratch under `TMPDIR` is removed when its guard drops: after a panic, and after a test
//! left its fixture read-only.
//!
//! One run of the 0.56.0 release gate left about 9 GiB of `ess-*` directories in a shared
//! `TMPDIR`; some of them were read-only fixtures nobody could remove without restoring the write
//! bit first. `ess_cli::TemporaryDirectory` is the one guard a test creates `TMPDIR` scratch with.

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    panic,
    path::{Path, PathBuf},
};

use ess_cli::TemporaryDirectory;

fn mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[test]
fn a_read_only_fixture_is_removed_when_its_guard_drops_during_a_panic() {
    let mut created = PathBuf::new();
    let outcome = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let scratch = TemporaryDirectory::create("ess-guard-read-only").unwrap();
        created = scratch.path().to_path_buf();
        let shared = scratch.path().join("anchor/shared");
        let sealed = scratch.path().join("anchor/sealed");
        fs::create_dir_all(&shared).unwrap();
        fs::create_dir_all(sealed.join("inner")).unwrap();
        fs::write(shared.join("a"), b"preserve").unwrap();
        fs::write(sealed.join("inner/b"), b"preserve").unwrap();
        mode(&shared.join("a"), 0o400);
        mode(&shared, 0o555);
        mode(&sealed.join("inner"), 0o000);
        mode(&sealed, 0o500);
        mode(&scratch.path().join("anchor"), 0o555);
        panic!("the test failed while its fixture was read-only");
    }));
    assert!(outcome.is_err());
    assert!(
        !created.as_os_str().is_empty() && fs::symlink_metadata(&created).is_err(),
        "{} survived its guard",
        created.display()
    );
}

#[test]
fn a_guard_removes_a_tree_whose_symlink_points_at_a_read_only_directory_outside_it() {
    let outside = TemporaryDirectory::create("ess-guard-outside").unwrap();
    let target = outside.path().join("kept");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("file"), b"outside").unwrap();
    mode(&target, 0o555);
    let created = {
        let scratch = TemporaryDirectory::create("ess-guard-link").unwrap();
        std::os::unix::fs::symlink(&target, scratch.path().join("link")).unwrap();
        mode(scratch.path(), 0o500);
        scratch.path().to_path_buf()
    };
    assert!(fs::symlink_metadata(&created).is_err());
    // Restoring write bits never follows a link out of the guarded tree.
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o555
    );
    assert_eq!(fs::read(target.join("file")).unwrap(), b"outside");
    mode(&target, 0o755);
}

/// Every `TMPDIR` directory this crate's tests create goes through the guard, so a new helper that
/// reaches for the process temporary directory directly fails here rather than in a full `TMPDIR`.
#[test]
fn no_test_creates_tmpdir_scratch_without_a_guard() {
    let needles = [concat!("temp", "_dir"), concat!("into", "_path()")];
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // The guard itself, and the std-only ones kept by files another crate compiles too and which
    // therefore cannot depend on `ess-cli`: `support/browser.rs` (`ess-conformance`) and
    // `src/git_checkout.rs` (`ess-xtask`, through `#[path]`).
    let guards = [
        (
            crate_root.join("src/git_checkout.rs"),
            "impl Drop for Scratch",
        ),
        (
            crate_root.join("src/lib.rs"),
            "impl Drop for TemporaryDirectory",
        ),
        (
            crate_root.join("tests/support/browser.rs"),
            "impl Drop for ProfileScratch",
        ),
    ];
    let mut found = Vec::new();
    let mut pending = vec![crate_root.join("src"), crate_root.join("tests")];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap();
            if let Some((_, drop)) = guards.iter().find(|(guard, _)| *guard == path) {
                assert!(text.contains(drop), "{} lost its guard", path.display());
                continue;
            }
            for (number, line) in text.lines().enumerate() {
                if needles.iter().any(|needle| line.contains(needle)) {
                    found.push(format!(
                        "{}:{}: {}",
                        path.strip_prefix(crate_root).unwrap().display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    found.sort();
    assert!(
        found.is_empty(),
        "TMPDIR scratch created without ess_cli::TemporaryDirectory:\n{}",
        found.join("\n")
    );
}
