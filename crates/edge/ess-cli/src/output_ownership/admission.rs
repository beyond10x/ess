//! Exercise exact names before enrollment, in private namespaces inherited from each parent.
//! A process cut can leave only a reserved, unowned admission orphan. Its name never grants
//! cleanup authority: successful cleanup uses this invocation's descriptors and identities.
use super::{
    filesystem::{self, Locks, Mount, Observer},
    state::{self, Identity},
};
use anyhow::{ensure, Context, Result};
use rustix::{
    fs::{self, AtFlags, FileType, Mode, OFlags},
    io::Errno,
};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

const PREFIX: &str = ".ess-output-init-names-";
pub(super) fn orphan(name: &OsStr) -> bool {
    name.to_str()
        .and_then(|s| s.strip_prefix(PREFIX))
        .is_some_and(|id| {
            id.len() == 36
                && id.bytes().enumerate().all(|(at, byte)| {
                    if [8, 13, 18, 23].contains(&at) {
                        byte == b'-'
                    } else {
                        byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
                    }
                })
        })
}

#[derive(Default)]
struct Node {
    children: BTreeMap<OsString, Node>,
}

pub(super) fn paths(
    locks: &Locks,
    anchor: &Path,
    paths: impl IntoIterator<Item = PathBuf>,
    mut observer: Observer<'_>,
) -> Result<()> {
    let (parent_path, parent) = locks.nearest_binding(anchor)?;
    let prefix = anchor.strip_prefix(parent_path)?;
    let mut tree = Node::default();
    for path in
        std::iter::once(prefix.to_path_buf()).chain(paths.into_iter().map(|p| prefix.join(p)))
    {
        let mut node = &mut tree;
        for component in path.components() {
            node = node
                .children
                .entry(component.as_os_str().to_owned())
                .or_default();
        }
    }
    if tree.children.is_empty() {
        return Ok(());
    }
    locks.revalidate()?;
    let mount = Mount::of(&parent)?;
    inspect(&parent, &tree, &mount, &mut observer)?;
    locks.revalidate()
}

struct Entry {
    parent: File,
    name: OsString,
    id: Identity,
    directory: bool,
}
struct Namespace {
    parent: File,
    name: OsString,
    root: File,
    id: Identity,
    entries: Vec<Entry>,
}
impl Namespace {
    fn create(parent: &File, mount: &Mount, observer: &mut Observer<'_>) -> Result<Self> {
        let name = OsString::from(format!("{PREFIX}{}", state::new_uuid()?));
        let root = filesystem::mkdir(parent, &name, 0o700, mount, observer, "admission-namespace")?;
        let id = filesystem::identity(&root)?;
        Ok(Self {
            parent: parent.try_clone()?,
            name,
            root,
            id,
            entries: Vec::new(),
        })
    }
    fn entry(
        &mut self,
        parent: &File,
        name: &OsStr,
        directory: bool,
        mount: &Mount,
        observer: &mut Observer<'_>,
    ) -> Result<File> {
        observer("before:create:admission-name")?;
        let mut fd = if directory {
            fs::mkdirat(parent, name, Mode::from_raw_mode(0o700))?;
            filesystem::open_directory(parent, name, mount)?
        } else {
            File::from(fs::openat(
                parent,
                name,
                OFlags::RDWR | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o600),
            )?)
        };
        mount.check(&fd)?;
        self.entries.push(Entry {
            parent: parent.try_clone()?,
            name: name.to_owned(),
            id: filesystem::identity(&fd)?,
            directory,
        });
        observer("after:create:admission-name")?;
        ensure!(
            filesystem::names(parent)?.iter().any(|entry| entry == name),
            "native filesystem does not preserve exact output name bytes"
        );
        if directory {
            inherited(parent, &fd)?;
        } else {
            observer("before:write:admission-name")?;
            fd.write_all(b"native output name admission\n")?;
            observer("after:write:admission-name")?;
        }
        filesystem::sync(&fd, observer, "admission-name")?;
        Ok(fd)
    }
    fn cleanup(&self, mount: &Mount, observer: &mut Observer<'_>) -> Result<()> {
        // Every member is known from a successful exclusive create in this invocation.
        // An unexplained member prevents removal of its containing directory naturally.
        for entry in self.entries.iter().rev() {
            same(&entry.parent, &entry.name, &entry.id, entry.directory)?;
            filesystem::remove(
                &entry.parent,
                &entry.name,
                entry.directory,
                mount,
                observer,
                "admission-name",
            )?;
        }
        same(&self.parent, &self.name, &self.id, true)?;
        filesystem::remove(
            &self.parent,
            &self.name,
            true,
            mount,
            observer,
            "admission-namespace",
        )
    }
}

fn same(parent: &File, name: &OsStr, expected: &Identity, directory: bool) -> Result<()> {
    let stat = fs::statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)?;
    ensure!(
        Identity {
            device: stat.st_dev as u64,
            inode: stat.st_ino as u64
        } == *expected
            && FileType::from_raw_mode(stat.st_mode)
                == if directory {
                    FileType::Directory
                } else {
                    FileType::RegularFile
                },
        "admission entry identity or type changed; preserving private evidence"
    );
    Ok(())
}

fn inspect(parent: &File, node: &Node, mount: &Mount, observer: &mut Observer<'_>) -> Result<()> {
    let mut namespace = Namespace::create(parent, mount, observer)?;
    let root = namespace.root.try_clone()?;
    let outcome = inherited(parent, &root)
        .and_then(|()| populate(&mut namespace, &root, Some(parent), node, mount, observer));
    let cleanup = namespace.cleanup(mount, observer);
    match (outcome, cleanup) {
        (Err(native), Err(cleanup)) => Err(native).context(format!(
            "admission cleanup also failed; reserved orphan preserved: {cleanup:#}"
        )),
        (Err(native), Ok(())) => Err(native),
        (Ok(()), cleanup) => cleanup,
    }
}

fn populate(
    namespace: &mut Namespace,
    probe: &File,
    actual: Option<&File>,
    node: &Node,
    mount: &Mount,
    observer: &mut Observer<'_>,
) -> Result<()> {
    for (name, child) in &node.children {
        if let Some(actual) = actual {
            filesystem::aliases(actual, Path::new(name), mount)?;
        }
        let fd = namespace
            .entry(probe, name, !child.children.is_empty(), mount, observer)
            .with_context(|| format!("native output name admission: {}", name.to_string_lossy()))?;
        if child.children.is_empty() {
            continue;
        }
        let existing = actual
            .map(
                |actual| match fs::statat(actual, name, AtFlags::SYMLINK_NOFOLLOW) {
                    Ok(stat) if FileType::from_raw_mode(stat.st_mode) == FileType::Directory => {
                        filesystem::open_directory(actual, name, mount).map(Some)
                    }
                    Ok(_) | Err(Errno::NOENT) => Ok(None),
                    Err(e) => Err(e.into()),
                },
            )
            .transpose()?
            .flatten();
        if let Some(existing) = existing {
            // An existing descendant may have a different casefold policy. Its own private
            // child, rather than this ancestor's emulation, supplies its native admission.
            inspect(&existing, child, mount, observer)?;
        } else {
            populate(namespace, &fd, None, child, mount, observer)?;
        }
    }
    Ok(())
}

fn inherited(parent: &File, child: &File) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        match (fs::ioctl_getflags(parent), fs::ioctl_getflags(child)) {
            // FS_CASEFOLD_FL from Linux's stable FS_IOC_GETFLAGS interface; rustix retains
            // unknown bits but does not yet name this flag in IFlags.
            (Ok(parent), Ok(child)) => ensure!(
                parent.bits() & 0x4000_0000 == child.bits() & 0x4000_0000,
                "native admission directory did not inherit parent casefold policy"
            ),
            (Err(a), Err(b)) if a == b && matches!(a, Errno::NOTTY | Errno::NOTSUP) => {
                // Optional flags are unavailable on this filesystem. Admission still uses
                // exact native operations in an immediate child, relying on ordinary new-
                // directory inheritance, not on an inference that names are case sensitive.
            }
            (Err(error), _) | (_, Err(error)) => {
                return Err(error).context("checking native admission directory inheritance")
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = (parent, child);
    }
    Ok(())
}
