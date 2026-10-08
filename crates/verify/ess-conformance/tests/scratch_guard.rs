//! The scratch guard every test of this crate holds for a directory under `std::env::temp_dir()`.

mod support_scratch;

use support_scratch::Scratch;

fn name(label: &str) -> String {
    format!("ess-scratch-guard-{label}-{}", std::process::id())
}

#[test]
fn a_scratch_directory_is_removed_when_its_guard_drops() {
    let scratch = Scratch::new(name("drop"));
    let path = scratch.to_path_buf();
    std::fs::create_dir_all(scratch.join("nested/deeper")).unwrap();
    std::fs::write(scratch.join("nested/deeper/file.txt"), "contents").unwrap();
    assert!(path.is_dir());
    drop(scratch);
    assert!(!path.exists(), "{} survived its guard", path.display());
}

#[test]
fn a_scratch_directory_is_removed_when_the_test_holding_it_panics() {
    let path = std::env::temp_dir().join(name("panic"));
    let held = path.clone();
    let unwound = std::panic::catch_unwind(move || {
        let scratch = Scratch::adopt(held);
        std::fs::create_dir_all(scratch.join("inside")).unwrap();
        panic!("the test fails while it holds the directory");
    });
    assert!(unwound.is_err());
    assert!(!path.exists(), "{} survived a panic", path.display());
}

#[cfg(unix)]
#[test]
fn a_read_only_fixture_gets_its_write_bit_back_and_is_removed() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new(name("read-only"));
    let path = scratch.to_path_buf();
    let fixture = scratch.join("fixture");
    std::fs::create_dir_all(fixture.join("inner")).unwrap();
    std::fs::write(fixture.join("inner/file.txt"), "contents").unwrap();
    for directory in [fixture.join("inner"), fixture.clone()] {
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o500)).unwrap();
    }
    assert!(std::fs::remove_dir_all(&fixture).is_err());
    drop(scratch);
    assert!(!path.exists(), "{} survived read-only", path.display());
}

#[test]
fn an_adopted_path_that_was_never_created_drops_quietly() {
    let path = std::env::temp_dir().join(name("never"));
    drop(Scratch::adopt(path.clone()));
    assert!(!path.exists());
}
