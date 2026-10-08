//! The `ess` edge, as a library.
//!
//! `main.rs` remains the shipped `ess` binary and this crate's only adopter-facing surface. The
//! library exists for one reason: the finite recovery contract in [`recovery`] has to be executed
//! by *real, separate processes* to be tested honestly — a second driver that outlives its parent,
//! a target that outlives the first driver — and a `[[bin]]`-only crate has no way to let a second
//! process run the same parser, admission and journal code that the shipped binary runs.
//!
//! What is exported is therefore the smallest set that a second process needs to run the
//! production algorithm. The shipped CLI gains no bypass flag, environment switch or permissive
//! fixture profile from any of it; the test-only entry point is
//! `crates/edge/ess-cli/tests/support/recovery_driver.rs`, and it injects its host, transport and
//! storage controls through the Rust seams in [`recovery`] rather than through the binary.

#[path = "oci_cache.rs"]
pub mod oci_cache;
pub mod recovery;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// A private mode-0700 directory that removes itself, also when a test panics while holding it.
///
/// A tree left with read-only directories in it (a test fixture that sealed one) gets owner access
/// back before removal; restoring it never follows a symbolic link out of the tree.
///
/// Interruption may leave residue behind — the cleanup is best effort and always was — and neither
/// a missing cleanup nor a surviving file establishes whether anything outside this directory
/// changed. It is transient scratch, not evidence.
#[derive(Debug)]
pub struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    /// Creates one private transient directory under the process TMPDIR.
    pub fn create(prefix: &str) -> Result<Self> {
        use std::time::{SystemTime, UNIX_EPOCH};
        // Two guards made in the same nanosecond by two threads still get two directories.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock precedes the Unix epoch")?
            .as_nanos();
        let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "{prefix}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        let mut directory = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            directory.mode(0o700);
        }
        directory
            .create(&path)
            .with_context(|| format!("creating temporary directory {}", path.display()))?;
        Ok(Self(path))
    }

    /// The directory's path.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl std::ops::Deref for TemporaryDirectory {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for TemporaryDirectory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for TemporaryDirectory {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_os_str()
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if fs::remove_dir_all(&self.0).is_err() {
            restore_owner_access(&self.0);
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

/// Gives the owner full access to every directory below `path`, without following links.
fn restore_owner_access(path: &Path) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
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
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode | 0o700));
        }
    }
    for entry in fs::read_dir(path).into_iter().flatten().flatten() {
        restore_owner_access(&entry.path());
    }
}
