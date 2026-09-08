//! Safe native descriptors, mount identity and cooperative directory locking.
use super::state::{self, FileData, Identity, Image};
use anyhow::{bail, ensure, Context, Result};
use rustix::{
    fd::AsFd,
    fs::{self, AtFlags, FileType, FlockOperation, Mode, OFlags, RenameFlags},
    io::Errno,
};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs::File,
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
};

const DIRECTORY: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);
pub(super) type Observer<'a> = &'a mut dyn FnMut(&str) -> Result<()>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Mount {
    device: u64,
    #[cfg(target_os = "linux")]
    mount_id: u64,
    #[cfg(target_os = "macos")]
    mounted_on: Vec<u8>,
    #[cfg(target_os = "macos")]
    mounted_from: Vec<u8>,
}
pub(super) fn identity(fd: &impl AsFd) -> Result<Identity> {
    let stat = fs::fstat(fd)?;
    Ok(Identity {
        device: stat.st_dev as u64,
        inode: stat.st_ino as u64,
    })
}
impl Mount {
    pub(super) fn of(fd: &impl AsFd) -> Result<Self> {
        let device = identity(fd)?.device;
        #[cfg(target_os = "linux")]
        {
            let stat = fs::statx(
                fd,
                "",
                AtFlags::EMPTY_PATH | AtFlags::SYMLINK_NOFOLLOW,
                fs::StatxFlags::MNT_ID,
            )?;
            ensure!(
                stat.stx_mask & fs::StatxFlags::MNT_ID.bits() != 0,
                "filesystem does not expose Linux mount identity"
            );
            Ok(Self {
                device,
                mount_id: stat.stx_mnt_id,
            })
        }
        #[cfg(target_os = "macos")]
        {
            let stat = fs::fstatfs(fd)?;
            // Darwin's mountpoint and mounted-from fields describe this descriptor's mount,
            // including distinct mounts of one device; st_dev alone cannot establish that.
            let native = |bytes: &[std::ffi::c_char]| {
                bytes
                    .iter()
                    .take_while(|b| **b != 0)
                    .map(|b| *b as u8)
                    .collect::<Vec<_>>()
            };
            let mounted_on = native(&stat.f_mntonname);
            let mounted_from = native(&stat.f_mntfromname);
            ensure!(
                !mounted_on.is_empty() && !mounted_from.is_empty(),
                "filesystem does not expose Darwin mount identity"
            );
            Ok(Self {
                device,
                mounted_on,
                mounted_from,
            })
        }
    }
    pub(super) fn check(&self, fd: &impl AsFd) -> Result<()> {
        ensure!(
            &Self::of(fd)? == self,
            "cross-mount output transaction or changed mount identity"
        );
        Ok(())
    }
}

pub(super) fn absolute(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            Component::RootDir => result.push("/"),
            Component::Normal(name) => result.push(name),
            Component::Prefix(_) => bail!("unsupported output path prefix"),
        }
        match std::fs::symlink_metadata(&result) {
            Ok(m) => ensure!(
                m.is_dir() && !m.file_type().is_symlink(),
                "output path has an incompatible file type or symlink: {}",
                result.display()
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).with_context(|| format!("inspecting {}", result.display())),
        }
    }
    ensure!(result.is_absolute(), "output root is not absolute");
    Ok(result)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Access {
    Shared,
    Exclusive,
}
pub(super) struct Locks {
    directories: BTreeMap<PathBuf, File>,
    requested: Vec<(PathBuf, Access)>,
}
impl Locks {
    pub(super) fn acquire(requested: &[(PathBuf, Access)]) -> Result<Self> {
        let mut needed = BTreeMap::new();
        for (root, access) in requested {
            ensure!(
                !root.components().any(|c| state::reserved(c.as_os_str())),
                "ownership root intersects reserved state namespace"
            );
            let mut existing = root.as_path();
            while !existing.try_exists()? {
                existing = existing.parent().context("no existing output ancestor")?;
            }
            for path in existing.ancestors() {
                let lock = if path == existing && *access == Access::Exclusive {
                    Access::Exclusive
                } else {
                    Access::Shared
                };
                needed
                    .entry(path.to_path_buf())
                    .and_modify(|prior| {
                        if lock == Access::Exclusive {
                            *prior = lock;
                        }
                    })
                    .or_insert(lock);
            }
        }
        let mut paths: Vec<_> = needed.into_iter().collect();
        paths.sort_by(|a, b| {
            a.0.components()
                .count()
                .cmp(&b.0.components().count())
                .then(a.0.cmp(&b.0))
        });
        let mut result = Self {
            directories: BTreeMap::new(),
            requested: requested.to_vec(),
        };
        for (path, access) in paths {
            let fd = if let Some(parent) = path.parent() {
                File::from(fs::openat(
                    result
                        .directories
                        .get(parent)
                        .context("missing locked ancestor")?,
                    path.file_name().context("directory name")?,
                    DIRECTORY,
                    Mode::empty(),
                )?)
            } else {
                File::from(fs::open("/", DIRECTORY, Mode::empty())?)
            };
            fs::flock(
                &fd,
                match access {
                    Access::Shared => FlockOperation::NonBlockingLockShared,
                    Access::Exclusive => FlockOperation::NonBlockingLockExclusive,
                },
            )
            .with_context(|| format!("output ownership busy at {}", path.display()))?;
            result.directories.insert(path, fd);
        }
        result.revalidate()?;
        for (root, _) in requested {
            for ancestor in root.ancestors().skip(1) {
                if let Some(fd) = result.directories.get(ancestor) {
                    for name in names(fd)? {
                        if name.eq_ignore_ascii_case(OsStr::new(state::RESERVED)) {
                            bail!("output root is beneath enrolled or reserved ancestor {}; use its existing ownership root",ancestor.display());
                        }
                    }
                }
            }
        }
        Ok(result)
    }
    pub(super) fn revalidate(&self) -> Result<()> {
        for (path, fd) in &self.directories {
            let fresh = if let Some(parent) = path.parent() {
                fs::openat(
                    self.directories
                        .get(parent)
                        .context("missing locked ancestor")?,
                    path.file_name().context("directory name")?,
                    DIRECTORY,
                    Mode::empty(),
                )?
            } else {
                fs::open("/", DIRECTORY, Mode::empty())?
            };
            ensure!(
                identity(fd)? == identity(&fresh)?,
                "locked output directory binding changed: {}",
                path.display()
            );
        }
        Ok(())
    }
    pub(super) fn existing(&self, root: &Path) -> Result<File> {
        Ok(self
            .directories
            .get(root)
            .context("output ownership root does not exist")?
            .try_clone()?)
    }
    pub(super) fn nearest(&self, root: &Path) -> Result<File> {
        Ok(root
            .ancestors()
            .find_map(|p| self.directories.get(p))
            .context("no locked ancestor")?
            .try_clone()?)
    }
    pub(super) fn create_root(&mut self, root: &Path, observer: &mut Observer<'_>) -> Result<File> {
        ensure!(
            self.requested
                .iter()
                .any(|(p, a)| p == root && *a == Access::Exclusive),
            "root creation lacks exclusive admission"
        );
        let mut missing = Vec::new();
        let mut cursor = root;
        while !self.directories.contains_key(cursor) {
            missing.push(cursor.to_path_buf());
            cursor = cursor.parent().context("missing root ancestor")?;
        }
        let mount = Mount::of(
            self.directories
                .get(cursor)
                .context("missing root ancestor")?,
        )?;
        for path in missing.into_iter().rev() {
            self.revalidate()?;
            let parent = self
                .directories
                .get(path.parent().context("root parent")?)
                .context("locked root parent")?;
            observer("before:create-anchor-directory")?;
            fs::mkdirat(
                parent,
                path.file_name().context("root name")?,
                Mode::from_raw_mode(0o755),
            )?;
            observer("after:create-anchor-directory")?;
            sync(parent, observer, "anchor-parent")?;
            let fd = File::from(fs::openat(
                parent,
                path.file_name().context("root name")?,
                DIRECTORY,
                Mode::empty(),
            )?);
            mount.check(&fd)?;
            fs::flock(&fd, FlockOperation::NonBlockingLockExclusive)?;
            self.directories.insert(path, fd);
        }
        self.existing(root)
    }
}

pub(super) fn names(fd: &impl AsFd) -> Result<Vec<OsString>> {
    let mut result = Vec::new();
    for entry in fs::Dir::read_from(fd)? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_bytes() != b"." && name.to_bytes() != b".." {
            result.push(OsStr::from_bytes(name.to_bytes()).to_owned());
        }
    }
    result.sort();
    Ok(result)
}
pub(super) fn open_directory(fd: &impl AsFd, name: &OsStr, mount: &Mount) -> Result<File> {
    let file = File::from(
        fs::openat(fd, name, DIRECTORY, Mode::empty()).with_context(|| {
            format!(
                "output path has an incompatible file type or symlink: {}",
                name.to_string_lossy()
            )
        })?,
    );
    mount.check(&file)?;
    Ok(file)
}
pub(super) fn parent(
    root: &File,
    relative: &Path,
    mount: &Mount,
) -> Result<Option<(File, OsString)>> {
    let mut fd = root.try_clone()?;
    let mut components = relative.components().peekable();
    while let Some(c) = components.next() {
        let Component::Normal(name) = c else {
            bail!("unsafe descriptor-relative output path")
        };
        if components.peek().is_none() {
            return Ok(Some((fd, name.to_owned())));
        }
        match fs::openat(&fd, name, DIRECTORY, Mode::empty()) {
            Ok(next) => {
                fd = File::from(next);
                mount.check(&fd)?;
            }
            Err(Errno::NOENT | Errno::NOTDIR) => return Ok(None),
            Err(e) => {
                return Err(e).context("output path has an incompatible file type or symlink")
            }
        }
    }
    bail!("output path names its anchor")
}
pub(super) fn image(
    root: &File,
    relative: &Path,
    mount: &Mount,
) -> Result<(Image, Option<Vec<u8>>)> {
    let Some((parent, name)) = parent(root, relative, mount)? else {
        return Ok((Image::Absent, None));
    };
    let stat = match fs::statat(&parent, &name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(s) => s,
        Err(Errno::NOENT) => return Ok((Image::Absent, None)),
        Err(e) => return Err(e).context("inspecting output"),
    };
    let mode = stat.st_mode as u32 & 0o7777;
    state::mode(mode)?;
    match FileType::from_raw_mode(stat.st_mode) {
        FileType::Directory => {
            let fd = open_directory(&parent, &name, mount)?;
            ordinary_metadata(&fd, false)?;
            Ok((Image::Directory { mode }, None))
        }
        FileType::RegularFile => {
            ensure!(
                stat.st_nlink == 1,
                "output destination has multiple hard links"
            );
            let mut file = File::from(fs::openat(
                &parent,
                &name,
                OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
                Mode::empty(),
            )?);
            mount.check(&file)?;
            ordinary_metadata(&file, true)?;
            ensure!(
                identity(&file)?
                    == Identity {
                        device: stat.st_dev as u64,
                        inode: stat.st_ino as u64
                    },
                "output changed while opening snapshot"
            );
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            let after = fs::fstat(&file)?;
            ensure!(
                after.st_size == stat.st_size
                    && after.st_mtime == stat.st_mtime
                    && after.st_ctime == stat.st_ctime,
                "output changed while reading snapshot"
            );
            Ok((
                Image::File {
                    data: FileData::of(&bytes, mode),
                },
                Some(bytes),
            ))
        }
        _ => bail!(
            "output path has an incompatible file type or symlink: {}",
            relative.display()
        ),
    }
}
fn ordinary_metadata(fd: &impl AsFd, file: bool) -> Result<()> {
    let stat = fs::fstat(fd)?;
    ensure!(
        stat.st_uid == rustix::process::geteuid().as_raw()
            && stat.st_gid == rustix::process::getegid().as_raw(),
        "output has ownership metadata outside the ordinary snapshot contract"
    );
    if file {
        ensure!(
            stat.st_nlink == 1,
            "output destination has multiple hard links"
        );
    }
    let mut names = [0u8; 1];
    match fs::flistxattr(fd, &mut names[..]) {
        Ok(0) | Err(Errno::NOTSUP) => {}
        Ok(_) | Err(Errno::RANGE) => {
            bail!("output has extended metadata outside the ordinary snapshot contract")
        }
        Err(e) => return Err(e).context("checking output extended metadata"),
    }
    #[cfg(target_os = "macos")]
    ensure!(
        stat.st_flags == 0,
        "output has Darwin flags outside the ordinary snapshot contract"
    );
    Ok(())
}
pub(super) fn sync(fd: &impl AsFd, observer: &mut Observer<'_>, label: &str) -> Result<()> {
    observer(&format!("before:sync:{label}"))?;
    fs::fsync(fd).with_context(|| format!("synchronizing output {label}"))?;
    observer(&format!("after:sync:{label}"))?;
    Ok(())
}
pub(super) fn write_new(
    parent: &File,
    name: &OsStr,
    bytes: &[u8],
    mode: u32,
    mount: &Mount,
    observer: &mut Observer<'_>,
    label: &str,
) -> Result<Identity> {
    mount.check(parent)?;
    observer(&format!("before:create:{label}"))?;
    let mut file = File::from(fs::openat(
        parent,
        name,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::from_raw_mode(0o600),
    )?);
    observer(&format!("after:create:{label}"))?;
    mount.check(&file)?;
    // Split the write so partial staging is a reachable and precisely injected boundary.
    let middle = bytes.len() / 2;
    observer(&format!("before:write:{label}"))?;
    file.write_all(&bytes[..middle])?;
    observer(&format!("after:partial-write:{label}"))?;
    file.write_all(&bytes[middle..])?;
    observer(&format!("after:write:{label}"))?;
    observer(&format!("before:mode:{label}"))?;
    fs::fchmod(&file, Mode::from_raw_mode(mode))?;
    observer(&format!("after:mode:{label}"))?;
    sync(&file, observer, label)?;
    let id = identity(&file)?;
    let (actual, _) = image(parent, Path::new(name), mount)?;
    ensure!(
        actual
            == Image::File {
                data: FileData::of(bytes, mode)
            },
        "written output readback failed"
    );
    Ok(id)
}
// Both descriptor/name pairs, no-replacement policy and measured fault boundary are explicit.
#[allow(clippy::too_many_arguments)]
pub(super) fn rename(
    from: &File,
    name: &OsStr,
    to: &File,
    destination: &OsStr,
    no_replace: bool,
    mount: &Mount,
    observer: &mut Observer<'_>,
    label: &str,
) -> Result<()> {
    mount.check(from)?;
    mount.check(to)?;
    observer(&format!("before:rename:{label}"))?;
    fs::renameat_with(
        from,
        name,
        to,
        destination,
        if no_replace {
            RenameFlags::NOREPLACE
        } else {
            RenameFlags::empty()
        },
    )?;
    observer(&format!("after:rename:{label}"))?;
    sync(from, observer, "rename-source")?;
    sync(to, observer, "rename-destination")
}
pub(super) fn remove(
    parent: &File,
    name: &OsStr,
    directory: bool,
    mount: &Mount,
    observer: &mut Observer<'_>,
    label: &str,
) -> Result<()> {
    mount.check(parent)?;
    observer(&format!("before:remove:{label}"))?;
    fs::unlinkat(
        parent,
        name,
        if directory {
            AtFlags::REMOVEDIR
        } else {
            AtFlags::empty()
        },
    )?;
    observer(&format!("after:remove:{label}"))?;
    sync(parent, observer, "removed-entry-parent")
}
pub(super) fn mkdir(
    parent: &File,
    name: &OsStr,
    mode: u32,
    mount: &Mount,
    observer: &mut Observer<'_>,
    label: &str,
) -> Result<File> {
    mount.check(parent)?;
    observer(&format!("before:mkdir:{label}"))?;
    fs::mkdirat(parent, name, Mode::from_raw_mode(mode))?;
    observer(&format!("after:mkdir:{label}"))?;
    let fd = open_directory(parent, name, mount)?;
    fs::fchmod(&fd, Mode::from_raw_mode(mode))?;
    sync(&fd, observer, "created-directory")?;
    sync(parent, observer, "created-directory-parent")?;
    Ok(fd)
}
pub(super) fn discovery(root: &File, mount: &Mount, at_anchor: bool) -> Result<()> {
    for name in names(root)? {
        if state::reserved(&name) {
            if at_anchor
                && (name == OsStr::new(state::RESERVED)
                    || name.as_bytes().starts_with(state::INIT_PREFIX.as_bytes()))
            {
                continue;
            }
            bail!(
                "descendant or aliased reserved output enrollment: {}",
                name.to_string_lossy()
            );
        }
        let stat = fs::statat(root, &name, AtFlags::SYMLINK_NOFOLLOW)?;
        if FileType::from_raw_mode(stat.st_mode) == FileType::Directory {
            let child = open_directory(root, &name, mount)?;
            discovery(&child, mount, false)?;
        }
    }
    Ok(())
}
pub(super) fn aliases(root: &File, path: &Path, mount: &Mount) -> Result<()> {
    let mut fd = root.try_clone()?;
    let mut parts = path.components().peekable();
    while let Some(part) = parts.next() {
        let name = part.as_os_str();
        for entry in names(&fd)? {
            ensure!(
                entry == name || !entry.eq_ignore_ascii_case(name),
                "output path aliases an existing entry: {}",
                path.display()
            );
        }
        if parts.peek().is_some() {
            match fs::openat(&fd, name, DIRECTORY, Mode::empty()) {
                Ok(next) => {
                    fd = File::from(next);
                    mount.check(&fd)?;
                }
                Err(Errno::NOENT | Errno::NOTDIR) => break,
                Err(e) => {
                    return Err(e).context("output path has an incompatible file type or symlink")
                }
            }
        }
    }
    Ok(())
}
