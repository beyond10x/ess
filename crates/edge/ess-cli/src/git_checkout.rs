//! Whether a path lies inside a Git checkout, decided from the filesystem without running Git.
//!
//! Compiled into `ess` and into `ess-xtask` (`#[path]`), so both "outside Git checkouts" guards
//! apply one rule.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The entries of a `.git` directory that Git reads to open it as a repository. A directory
/// holding none of them — a harness creates `.git/info/exclude` alone in a directory that is no
/// repository — is not one, and Git refuses to open it.
const REPOSITORY_ENTRIES: [&str; 4] = ["HEAD", "objects", "refs", "commondir"];

/// The nearest of `path` and its ancestors whose `.git` marks a checkout.
///
/// A `.git` file (a linked worktree's `gitdir:` pointer) and a `.git` symlink mark one, and so
/// does a `.git` directory holding any entry in [`REPOSITORY_ENTRIES`]. A marker that cannot be
/// read is an error, never an absence.
pub(crate) fn enclosing_checkout(path: &Path) -> io::Result<Option<PathBuf>> {
    for ancestor in path.ancestors() {
        let marker = ancestor.join(".git");
        let metadata = match fs::symlink_metadata(&marker) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if !metadata.is_dir() || opens_as_repository(&marker)? {
            return Ok(Some(ancestor.to_path_buf()));
        }
    }
    Ok(None)
}

fn opens_as_repository(marker: &Path) -> io::Result<bool> {
    for entry in REPOSITORY_ENTRIES {
        match fs::symlink_metadata(marker.join(entry)) {
            Ok(_) => return Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::enclosing_checkout;
    use ess_cli::TemporaryDirectory;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    fn root(case: &str) -> TemporaryDirectory {
        let root = TemporaryDirectory::create(&format!(
            "ess-git-checkout-{case}-{}",
            env!("CARGO_CRATE_NAME")
        ))
        .unwrap();
        fs::create_dir(root.join("below")).unwrap();
        root
    }

    #[test]
    fn a_git_directory_git_cannot_open_is_no_checkout() {
        let exclude_only = root("exclude-only");
        fs::create_dir_all(exclude_only.join(".git/info")).unwrap();
        fs::write(
            exclude_only.join(".git/info/exclude"),
            "# harness runtime\n",
        )
        .unwrap();
        let empty = root("empty");
        fs::create_dir(empty.join(".git")).unwrap();
        for root in [exclude_only, empty] {
            let found = enclosing_checkout(&root.join("below")).unwrap();
            fs::remove_dir_all(&root).unwrap();
            assert_eq!(found, None, "{}", root.display());
        }
    }

    #[test]
    fn each_entry_git_opens_a_repository_by_marks_a_checkout() {
        for entry in super::REPOSITORY_ENTRIES {
            let root = root(&format!("entry-{entry}"));
            fs::create_dir(root.join(".git")).unwrap();
            fs::write(root.join(".git").join(entry), "").unwrap();
            let found = enclosing_checkout(&root.join("below")).unwrap();
            fs::remove_dir_all(&root).unwrap();
            assert_eq!(found.as_deref(), Some(root.path()), "{entry}");
        }
    }

    #[test]
    fn a_gitfile_and_a_git_symlink_mark_a_checkout() {
        let gitfile = root("gitfile");
        fs::write(gitfile.join(".git"), "gitdir: /unavailable/linked-tree\n").unwrap();
        let symlink = root("symlink");
        std::os::unix::fs::symlink(symlink.join("below"), symlink.join(".git")).unwrap();
        for root in [gitfile, symlink] {
            let found = enclosing_checkout(&root.join("below")).unwrap();
            fs::remove_dir_all(&root).unwrap();
            assert_eq!(found.as_deref(), Some(root.path()));
        }
    }

    #[test]
    fn an_unreadable_git_directory_is_an_error_not_an_absence() {
        let root = root("unreadable");
        let marker = root.join(".git");
        fs::create_dir(&marker).unwrap();
        fs::set_permissions(&marker, fs::Permissions::from_mode(0o000)).unwrap();
        // A privileged runner reads through the mode bits, so there is nothing to observe.
        let privileged = fs::read_dir(&marker).is_ok();
        let found = enclosing_checkout(&root.join("below"));
        fs::set_permissions(&marker, fs::Permissions::from_mode(0o755)).unwrap();
        fs::remove_dir_all(&root).unwrap();
        if !privileged {
            assert!(found.is_err(), "{found:?}");
        }
    }
}
