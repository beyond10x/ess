//! A test scratch directory under `TMPDIR` that is removed when its guard drops.
//!
//! Shared by this crate's unit tests and, through `#[path]`, by its integration tests. A fixture
//! removed by a statement at the end of a test is retained by every test that panics before that
//! statement, and on a shared workstation those directories accumulate by the thousand. Dropping
//! the guard runs on unwind too, so a red test leaves nothing behind either. A fixture a test made
//! read-only gets its write bit back before removal; without it the removal fails half-way.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

/// A fresh, empty directory `<TMPDIR>/<prefix>-<pid>-<sequence>`, removed on drop.
pub(crate) struct Scratch(PathBuf);

impl Scratch {
    /// Creates the directory. `prefix` names the test that owns it, so a leak names its source.
    pub(crate) fn new(prefix: &str) -> Self {
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{sequence}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|error| panic!("creating {}: {error}", path.display()));
        Self(path)
    }

    /// The directory's path.
    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        restore_write_bits(&self.0);
        // Never panic here: a drop that panics while a failing test unwinds aborts the process
        // and hides the test's own message.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Gives every directory below `path` its owner write and search bits back, without following
/// symbolic links.
fn restore_write_bits(path: &Path) {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return;
    };
    if !metadata.is_dir() {
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if mode & 0o700 != 0o700 {
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode | 0o700));
        }
    }
    #[cfg(not(unix))]
    {
        let mut permissions = metadata.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(path, permissions);
        }
    }
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            restore_write_bits(&entry.path());
        }
    }
}
