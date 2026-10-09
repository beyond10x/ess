//! A directory under `std::env::temp_dir()` that is removed when its guard drops, also while a
//! failed test unwinds.
//!
//! A test that writes a directory into `$TMPDIR` holds a [`Scratch`] for as long as it uses the
//! directory. Dropping the guard gives every directory beneath it its owner permissions back, so a
//! fixture a test made read-only does not survive, and then removes the whole tree. Directories
//! under `CARGO_TARGET_TMPDIR` are not scratch in this sense and do not use it.
#![allow(dead_code)]

use std::ffi::OsStr;
use std::ops::Deref;
use std::path::{Path, PathBuf};

/// Owns one scratch directory and removes it on drop.
#[derive(Debug)]
pub struct Scratch(PathBuf);

impl Scratch {
    /// `std::env::temp_dir()/<name>`, emptied of anything an earlier run left and created afresh.
    pub fn new(name: impl AsRef<Path>) -> Self {
        let scratch = Self::adopt(std::env::temp_dir().join(name));
        let _ = std::fs::remove_dir_all(&scratch.0);
        std::fs::create_dir_all(&scratch.0).expect("the scratch directory is created");
        scratch
    }

    /// Takes over `path` as it is, created or not: whatever is there when the guard drops is
    /// removed.
    pub fn adopt(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// The directory.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Deref for Scratch {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for Scratch {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<OsStr> for Scratch {
    fn as_ref(&self) -> &OsStr {
        self.0.as_os_str()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        writable(&self.0);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Gives `path` and every directory beneath it back the permissions removal needs. Symbolic links
/// are not followed: only what lies inside the scratch directory is changed.
fn writable(path: &Path) {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return;
    };
    if metadata.file_type().is_symlink() {
        return;
    }
    let mut permissions = metadata.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.is_dir() && (permissions.mode() & 0o700) != 0o700 {
            permissions.set_mode(permissions.mode() | 0o700);
            let _ = std::fs::set_permissions(path, permissions);
        }
    }
    #[cfg(not(unix))]
    #[allow(clippy::permissions_set_readonly_false)]
    {
        if permissions.readonly() {
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(path, permissions);
        }
    }
    if !metadata.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        writable(&entry.path());
    }
}
