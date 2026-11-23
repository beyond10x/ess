//! Replacing a file in one step. The bytes go to a new file beside the destination. That file
//! is synchronised and then renamed onto the destination, so at every instant the destination
//! holds either its earlier bytes or the complete new ones.
//!
//! The rename applies only where the destination is absent or a regular file. A destination
//! that is something else (`/dev/stdout`, `/dev/null`, a FIFO, a socket) is written in place,
//! as `fs::write` did: a regular file renamed over it would not write to it.

use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Writes `bytes` to `path` in one step (see the module documentation).
pub(crate) fn replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_with(path, |file| file.write_all(bytes))
}

/// [`replace`], with the bytes written into the temporary file by `write`. This is the seam
/// the tests use to fail a write part-way and to look at the temporary file before its first
/// byte.
///
/// A destination that is a symbolic link is followed: the file it names is replaced in that
/// file's own directory, and the link stays. A file that is replaced keeps its permissions:
/// the temporary file is created with them, before a byte is written, so it is never readable
/// by more than the earlier file was. A new file gets the permissions of a plain create.
/// On any failure before the rename, the temporary file is removed and the destination is left
/// as it was. A destination that exists and is neither a regular file nor a directory is
/// written in place, with no temporary file and no synchronisation.
pub(crate) fn replace_with(
    path: &Path,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> io::Result<()> {
    // Decided on the path as given: `/dev/stdout` is a link whose target does not exist as a
    // path, so it must be opened, never canonicalized.
    if let Ok(metadata) = fs::metadata(path) {
        if !metadata.is_file() && !metadata.is_dir() {
            return write_in_place(path, write);
        }
    }
    let destination = destination(path)?;
    let earlier = match fs::metadata(&destination) {
        Ok(metadata) if metadata.is_file() => Some(metadata.permissions()),
        _ => None,
    };
    let name = destination.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "the destination names no file")
    })?;
    let parent = match destination.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let (temporary, mut file) = create_temporary(parent, name, earlier.as_ref())?;
    let written = match &earlier {
        Some(permissions) => file.set_permissions(permissions.clone()),
        None => Ok(()),
    }
    .and_then(|()| write(&mut file))
    .and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = written.and_then(|()| fs::rename(&temporary, &destination)) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    sync_directory(parent)
}

/// Writes an existing destination that is not a regular file the way `fs::write` does.
fn write_in_place(
    destination: &Path,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(destination)?;
    write(&mut file)
}

/// `path`, or the file it names when it is a symbolic link.
fn destination(path: &Path) -> io::Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => fs::canonicalize(path),
        _ => Ok(path.to_path_buf()),
    }
}

/// A new, empty `.<name>.<pid>-<n>.tmp` in `parent`. It is created with `create_new`, so it
/// never replaces another file, and a name that is taken is retried with the next `n`. Where
/// the platform has permission bits, the create call carries those of `earlier`; the process
/// umask can only narrow them, and the caller sets them exactly before the first byte.
fn create_temporary(
    parent: &Path,
    name: &OsStr,
    earlier: Option<&fs::Permissions>,
) -> io::Result<(PathBuf, File)> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let mut taken = None;
    for _ in 0..16 {
        let mut file_name = OsString::from(".");
        file_name.push(name);
        file_name.push(format!(
            ".{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let temporary = parent.join(file_name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        if let Some(permissions) = earlier {
            use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};
            options.mode(permissions.mode());
        }
        #[cfg(not(unix))]
        let _ = earlier;
        match options.open(&temporary) {
            Ok(file) => return Ok((temporary, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => taken = Some(error),
            Err(error) => return Err(error),
        }
    }
    Err(taken.unwrap_or_else(|| io::Error::from(io::ErrorKind::AlreadyExists)))
}

/// Makes the rename durable: the directory entry is synchronised like the file was.
#[cfg(unix)]
fn sync_directory(parent: &Path) -> io::Result<()> {
    File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn sync_directory(_: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ess_cli::TemporaryDirectory;

    const EARLIER: &[u8] = b"earlier report\n";

    fn entries(directory: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn directory() -> TemporaryDirectory {
        TemporaryDirectory::create("ess-replace-file").unwrap()
    }

    #[test]
    fn a_write_that_fails_after_the_first_byte_leaves_the_earlier_file() {
        let directory = directory();
        let path = directory.join("report.json");
        fs::write(&path, EARLIER).unwrap();
        let error = replace_with(&path, |file| {
            file.write_all(b"{")?;
            Err(io::Error::other("injected failure"))
        })
        .unwrap_err();
        assert_eq!(error.to_string(), "injected failure");
        assert_eq!(fs::read(&path).unwrap(), EARLIER);
        assert_eq!(entries(&directory), ["report.json"]);
    }

    #[test]
    fn the_destination_keeps_its_earlier_bytes_until_the_new_ones_are_complete() {
        let directory = directory();
        let path = directory.join("report.json");
        fs::write(&path, EARLIER).unwrap();
        replace_with(&path, |file| {
            file.write_all(b"{\"new\": true}\n")?;
            assert_eq!(fs::read(&path).unwrap(), EARLIER);
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"{\"new\": true}\n");
        assert_eq!(entries(&directory), ["report.json"]);
    }

    #[test]
    fn an_absent_destination_appears_only_complete() {
        let directory = directory();
        let path = directory.join("report.json");
        replace_with(&path, |file| {
            file.write_all(b"complete\n")?;
            assert!(!path.exists());
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"complete\n");
        assert_eq!(entries(&directory), ["report.json"]);
    }

    #[test]
    fn a_failed_write_without_an_earlier_file_leaves_nothing() {
        let directory = directory();
        let path = directory.join("report.json");
        let result = replace_with(&path, |file| {
            file.write_all(b"{")?;
            Err(io::Error::other("injected failure"))
        });
        assert!(result.is_err());
        assert_eq!(entries(&directory), Vec::<String>::new());
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_destination_replaces_the_file_it_names() {
        let directory = directory();
        fs::write(directory.join("real.json"), EARLIER).unwrap();
        let link = directory.join("report.json");
        std::os::unix::fs::symlink("real.json", &link).unwrap();
        replace(&link, b"new\n").unwrap();
        assert!(fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("real.json"));
        assert_eq!(fs::read(directory.join("real.json")).unwrap(), b"new\n");
        assert_eq!(entries(&directory), ["real.json", "report.json"]);
    }

    #[test]
    fn a_directory_destination_is_refused_and_leaves_no_temporary() {
        let directory = directory();
        let path = directory.join("report.json");
        fs::create_dir(&path).unwrap();
        fs::write(path.join("keep.txt"), b"keep\n").unwrap();
        assert!(replace(&path, b"x").is_err());
        assert!(path.is_dir());
        assert_eq!(fs::read(path.join("keep.txt")).unwrap(), b"keep\n");
        assert_eq!(entries(&directory), ["report.json"]);
    }

    #[cfg(unix)]
    #[test]
    fn the_replaced_file_keeps_its_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let directory = directory();
        let path = directory.join("report.json");
        fs::write(&path, EARLIER).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        replace(&path, b"new\n").unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_temporary_file_has_the_earlier_mode_before_its_first_byte() {
        use std::os::unix::fs::PermissionsExt;
        let directory = directory();
        let path = directory.join("report.json");
        fs::write(&path, EARLIER).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let mut seen = None;
        replace_with(&path, |file| {
            let metadata = file.metadata()?;
            seen = Some((metadata.len(), metadata.permissions().mode() & 0o777));
            file.write_all(b"new\n")
        })
        .unwrap();
        assert_eq!(seen, Some((0, 0o600)));
        assert_eq!(fs::read(&path).unwrap(), b"new\n");
    }

    #[cfg(unix)]
    #[test]
    fn a_new_temporary_file_is_never_wider_than_a_plain_create() {
        use std::os::unix::fs::PermissionsExt;
        let directory = directory();
        let plain = directory.join("plain");
        File::create(&plain).unwrap();
        let expected = fs::metadata(&plain).unwrap().permissions().mode() & 0o777;
        let path = directory.join("report.json");
        let mut seen = None;
        replace_with(&path, |file| {
            seen = Some(file.metadata()?.permissions().mode() & 0o777);
            Ok(())
        })
        .unwrap();
        assert_eq!(seen, Some(expected));
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_write_leaves_the_earlier_private_report_and_no_temporary() {
        use std::os::unix::fs::PermissionsExt;
        let directory = directory();
        let path = directory.join("report.json");
        fs::write(&path, EARLIER).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let result = replace_with(&path, |file| {
            file.write_all(b"{")?;
            Err(io::Error::other("injected failure"))
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), EARLIER);
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(entries(&directory), ["report.json"]);
    }

    #[cfg(unix)]
    #[test]
    fn a_device_destination_is_written_in_place_and_stays_a_device() {
        use std::os::unix::fs::FileTypeExt;
        replace(Path::new("/dev/null"), b"discarded\n").unwrap();
        assert!(fs::metadata("/dev/null")
            .unwrap()
            .file_type()
            .is_char_device());
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_to_a_device_is_written_through_and_stays_a_link() {
        let directory = directory();
        let link = directory.join("report.json");
        std::os::unix::fs::symlink("/dev/null", &link).unwrap();
        replace(&link, b"discarded\n").unwrap();
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("/dev/null"));
        assert_eq!(entries(&directory), ["report.json"]);
    }

    #[cfg(unix)]
    #[test]
    fn a_fifo_destination_is_written_in_place_and_stays_a_fifo() {
        use std::os::unix::fs::FileTypeExt;
        use std::process::Command;
        let directory = directory();
        let path = directory.join("report.fifo");
        assert!(Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success());
        let reader = std::thread::spawn({
            let path = path.clone();
            move || fs::read(path)
        });
        replace(&path, b"through the pipe\n").unwrap();
        assert_eq!(reader.join().unwrap().unwrap(), b"through the pipe\n");
        assert!(fs::symlink_metadata(&path).unwrap().file_type().is_fifo());
        assert_eq!(entries(&directory), ["report.fifo"]);
    }
}
