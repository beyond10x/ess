//! Edge-owned generated output: explicit enrollment, complete-owner publication and recovery.
//! Controlled parents and cooperating directory-lock participants are required. Publication is
//! sequential across directories; successful native synchronization is not a hardware guarantee.
mod admission;
mod filesystem;
mod state;

use anyhow::{bail, ensure, Context, Result};
use filesystem::{Access, Locks, Mount, Observer};
use state::{
    Blob, Change, Checkpoint, Family, FileData, Identity, Image, Ledger, NativePath,
    OwnedDirectory, OwnedFile, Owner, OwnerKey, Payload, Profile, Purpose, Transaction,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
};

/// One fixed owner's complete requested file inventory, relative to its enclosing anchor.
pub(crate) struct Publication {
    owner: OwnerKey,
    files: BTreeMap<PathBuf, Vec<u8>>,
}
impl Publication {
    pub(crate) fn tree<'a>(
        family: &str,
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self> {
        let files: Vec<_> = files.into_iter().collect();
        ess_gen::artifact::validate_paths(files.iter().map(|(p, _)| *p))
            .map_err(anyhow::Error::msg)?;
        Self::new(
            OwnerKey::tree(Family::parse(family)?)?,
            files
                .into_iter()
                .map(|(p, b)| (PathBuf::from(p), b.as_bytes().to_vec())),
        )
    }
    pub(crate) fn named(family: &str, name: &OsStr, contents: &str) -> Result<Self> {
        Self::new(
            OwnerKey::file(Family::parse(family)?, name)?,
            [(PathBuf::from(name), contents.as_bytes().to_vec())],
        )
    }
    pub(crate) fn compose<'a>(files: impl IntoIterator<Item = (PathBuf, &'a str)>) -> Result<Self> {
        Self::new(
            OwnerKey::tree(Family::Compose)?,
            files.into_iter().map(|(p, b)| (p, b.as_bytes().to_vec())),
        )
    }
    fn new(owner: OwnerKey, files: impl IntoIterator<Item = (PathBuf, Vec<u8>)>) -> Result<Self> {
        let mut paths = BTreeMap::new();
        let mut admitted = BTreeMap::new();
        for (path, bytes) in files {
            NativePath::from_relative(&path)?.output()?;
            state::insert_path(&mut paths, &path, true)?;
            ensure!(
                admitted.insert(path, bytes).is_none(),
                "duplicate output path"
            );
        }
        Ok(Self {
            owner,
            files: admitted,
        })
    }
}

/// Keep this guard alive while a caller compares its already-rendered bytes.
pub(crate) struct CheckGuard {
    _locks: Locks,
}
pub(crate) fn check(anchor: &Path) -> Result<CheckGuard> {
    let root = filesystem::absolute(anchor)?;
    let locks = Locks::acquire(&[(root.clone(), Access::Shared)])?;
    if let Ok(fd) = locks.existing(&root) {
        let mount = Mount::of(&fd)?;
        filesystem::discovery(&fd, &mount, true)?;
        if let Some((_, payload)) = read_state(&fd, &root, &mount)? {
            ensure_idle(&payload, &root)?;
        }
    }
    Ok(CheckGuard { _locks: locks })
}
pub(crate) fn publish(anchor: &Path, publications: Vec<Publication>) -> Result<()> {
    publish_observed(anchor, publications, &mut |_| Ok(()))
}
pub(crate) fn recover(anchor: &Path) -> Result<()> {
    recover_observed(anchor, &mut |_| Ok(()))
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    /// Enroll only existing bytes matching a settled generated reference.
    Adopt {
        #[arg(long)]
        ownership_root: PathBuf,
        #[arg(long = "from")]
        reference: PathBuf,
        /// Fixed generator family, such as projection:site or typescript-file.
        #[arg(long)]
        owner: String,
        /// One native filename; required only for a standalone file family.
        #[arg(long)]
        file: Option<std::ffi::OsString>,
    },
    /// Settle the recorded operation without loading specification inputs.
    Recover {
        #[arg(long)]
        ownership_root: PathBuf,
    },
}
pub(crate) fn run(command: Command) -> Result<std::process::ExitCode> {
    match command {
        Command::Adopt {
            ownership_root,
            reference,
            owner,
            file,
        } => adopt(&ownership_root, &reference, &owner, file.as_deref())?,
        Command::Recover { ownership_root } => recover(&ownership_root)?,
    }
    Ok(std::process::ExitCode::SUCCESS)
}
pub(crate) fn adopt(
    anchor: &Path,
    reference: &Path,
    family: &str,
    file: Option<&OsStr>,
) -> Result<()> {
    adopt_observed(anchor, reference, family, file, &mut |_| Ok(()))
}
// Keep joint-lock acquisition, read-only admission and the sole metadata publication in order.
#[allow(clippy::too_many_lines)]
fn adopt_observed(
    anchor: &Path,
    reference: &Path,
    family: &str,
    file: Option<&OsStr>,
    observer: Observer<'_>,
) -> Result<()> {
    adopt_observers(anchor, reference, family, file, observer, &mut |_| Ok(()))
}

#[allow(clippy::too_many_lines)]
fn adopt_observers(
    anchor: &Path,
    reference: &Path,
    family: &str,
    file: Option<&OsStr>,
    mut observer: Observer<'_>,
    admission_observer: Observer<'_>,
) -> Result<()> {
    let family = Family::parse(family)?;
    let owner = match file {
        Some(name) => {
            ensure!(
                family.standalone(),
                "--file is only supported for standalone owner families"
            );
            OwnerKey::file(family, name)?
        }
        None => OwnerKey::tree(family)?,
    };
    let path = filesystem::absolute(anchor)?;
    let reference = filesystem::absolute(reference)?;
    ensure!(
        !path.starts_with(&reference) && !reference.starts_with(&path),
        "adoption roots must be disjoint and non-aliasing"
    );
    let mut locks = Locks::acquire(&[
        (path.clone(), Access::Exclusive),
        (reference.clone(), Access::Shared),
    ])?;
    let reference_fd = locks.existing(&reference)?;
    let reference_mount = Mount::of(&reference_fd)?;
    filesystem::discovery(&reference_fd, &reference_mount, true)?;
    let (_, reference_state) = read_state(&reference_fd, &reference, &reference_mount)?
        .context("reference has no settled generated ownership state")?;
    let reference_ledger = ensure_idle(&reference_state, &reference)?;
    for (relative, (_, data)) in reference_ledger.files() {
        filesystem::aliases(&reference_fd, &relative.output()?, &reference_mount)?;
        ensure!(
            filesystem::image(&reference_fd, &relative.output()?, &reference_mount)?.0
                == Image::File { data },
            "reference owned bytes or metadata differ from its settled ledger"
        );
    }
    let expected = reference_ledger
        .owners
        .iter()
        .find(|o| o.key == owner)
        .context("reference does not enroll the selected owner")?;
    let existing = locks.existing(&path).ok();
    let mount = Mount::of(&locks.nearest(&path)?)?;
    if let Some(root) = &existing {
        filesystem::discovery(root, &mount, true)?;
        ensure!(
            filesystem::identity(root)? != filesystem::identity(&reference_fd)?,
            "adoption root aliases reference"
        );
    }
    let admitted = existing
        .as_ref()
        .map(|fd| read_state(fd, &path, &mount))
        .transpose()?
        .flatten();
    let before = admitted
        .as_ref()
        .map(|(_, p)| ensure_idle(p, &path).cloned())
        .transpose()?
        .unwrap_or_default();
    let prior_files = before.files();
    let mut files = Vec::new();
    for entry in &expected.files {
        let Some(root) = &existing else {
            continue;
        };
        filesystem::aliases(root, &entry.path.output()?, &mount)?;
        let (image, _) = filesystem::image(root, &entry.path.output()?, &mount)?;
        if matches!(image, Image::Absent) {
            continue;
        }
        let Image::File { data } = image else {
            bail!("adoption target is not an ordinary file")
        };
        ensure!(
            data.length == entry.data.length && data.digest == entry.data.digest,
            "adoption target differs from reference: {}",
            entry.path.output()?.display()
        );
        ensure!(
            prior_files
                .get(&entry.path)
                .is_none_or(|(key, _)| key == &owner),
            "adoption target belongs to another owner"
        );
        files.push(OwnedFile {
            path: entry.path.clone(),
            data,
        });
    }
    let selected = Owner {
        key: owner.clone(),
        files,
    };
    if let Some(previous) = before.owners.iter().find(|o| o.key == owner) {
        ensure!(
            previous == &selected,
            "adoption cannot change or shrink an already enrolled owner"
        );
        return Ok(());
    }
    let mut after = before.clone();
    after.owners.push(selected);
    after.owners.sort_by(|a, b| a.key.cmp(&b.key));
    after.validate()?;
    admission::paths(
        &locks,
        &path,
        before
            .files()
            .keys()
            .chain(after.files().keys())
            .chain(expected.files.iter().map(|f| &f.path))
            .map(NativePath::output)
            .collect::<Result<Vec<_>>>()?,
        admission_observer,
    )?;
    let root = locks.create_root(&path, &mut observer)?;
    let (directory, mut payload) = match admitted {
        Some(value) => value,
        None => initialize(&root, &path, &mount, &mut observer)?,
    };
    let transaction = Transaction {
        id: state::new_uuid()?,
        anchor_id: payload.anchor_id.clone(),
        purpose: Purpose::Adopt,
        selected: vec![owner],
        before,
        after,
        changes: Vec::new(),
        directory_identity: None,
    };
    let directory_identity = filesystem::identity(&directory)?;
    run_plan(
        &mut Session {
            locks,
            root,
            mount,
            directory,
            directory_identity,
            observer,
        },
        &mut payload,
        Plan {
            transaction,
            old: BTreeMap::new(),
            new: BTreeMap::new(),
        },
    )
}

pub(crate) fn named(path: &Path, family: &str, contents: &str) -> Result<()> {
    let name = filename(path)?;
    publish(
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
        vec![Publication::named(family, name, contents)?],
    )
}

/// Check the original spelling before `Path::file_name` discards directory syntax.
pub(crate) fn filename(path: &Path) -> Result<&OsStr> {
    let last = path
        .as_os_str()
        .as_encoded_bytes()
        .rsplit(|byte| std::path::is_separator(char::from(*byte)))
        .next()
        .unwrap_or_default();
    ensure!(
        !last.is_empty() && last != b"." && last != b"..",
        "output must name a file, not a directory: {}",
        path.display()
    );
    path.file_name()
        .context("output requires one native filename")
}

fn ensure_idle<'a>(payload: &'a Payload, root: &Path) -> Result<&'a Ledger> {
    match &payload.checkpoint {
        Checkpoint::Idle { ledger } => Ok(ledger),
        _ => bail!(
            "pending generated output at {}; run `ess output recover --ownership-root {}`",
            root.display(),
            root.display()
        ),
    }
}
fn transaction(payload: &Payload) -> Result<&Transaction> {
    match &payload.checkpoint {
        Checkpoint::Idle { .. } => bail!("no transaction"),
        Checkpoint::Staging { transaction }
        | Checkpoint::Prepared { transaction }
        | Checkpoint::Committed { transaction }
        | Checkpoint::Restored { transaction } => Ok(transaction),
    }
}
fn tx_name(tx: &Transaction) -> String {
    format!("transaction-{}", tx.id)
}

fn read_state(root: &File, path: &Path, mount: &Mount) -> Result<Option<(File, Payload)>> {
    let members = filesystem::names(root)?;
    if !members.iter().any(|n| n == OsStr::new(state::RESERVED)) {
        return Ok(None);
    }
    let directory = filesystem::open_directory(root, OsStr::new(state::RESERVED), mount)
        .context("reserved output state is not an admitted directory")?;
    let (image, bytes) = filesystem::image(&directory, Path::new("state.json"), mount)?;
    ensure!(
        matches!(image, Image::File { .. }),
        "missing published output-state checkpoint"
    );
    let payload = state::decode(&bytes.context("missing output-state bytes")?)?;
    ensure!(
        payload.root == NativePath::absolute(path)?
            && payload.directory == filesystem::identity(root)?,
        "output-state root binding or directory identity mismatch"
    );
    let tx = match &payload.checkpoint {
        Checkpoint::Idle { .. } => None,
        _ => Some(transaction(&payload)?),
    };
    for name in filesystem::names(&directory)? {
        if name == OsStr::new("state.json") {
            continue;
        }
        if name == OsStr::new("state.next") {
            let (image, _) = filesystem::image(&directory, Path::new(&name), mount)?;
            ensure!(
                matches!(image, Image::File { .. }),
                "unpublished checkpoint slot has an unsafe type"
            );
            continue;
        }
        let tx = tx.context("unexplained entry in settled output state")?;
        ensure!(
            name == OsStr::new(&tx_name(tx)),
            "unexplained output-state transaction member"
        );
        let fd = filesystem::open_directory(&directory, &name, mount)?;
        if let Some(expected) = &tx.directory_identity {
            ensure!(
                filesystem::identity(&fd)? == *expected,
                "transaction directory identity changed"
            );
        }
        let admitted = tx.member_names();
        for member in filesystem::names(&fd)? {
            ensure!(
                member.to_str().is_some_and(|m| admitted.contains(m)),
                "unexplained internal transaction entry"
            );
            let (image, _) = filesystem::image(&fd, Path::new(&member), mount)?;
            ensure!(
                matches!(image, Image::File { .. }),
                "unsafe internal transaction member type"
            );
        }
    }
    Ok(Some((directory, payload)))
}

struct Session<'a> {
    locks: Locks,
    root: File,
    mount: Mount,
    directory: File,
    directory_identity: Identity,
    observer: Observer<'a>,
}
impl Session<'_> {
    fn validate(&self) -> Result<()> {
        self.locks.revalidate()?;
        self.mount.check(&self.root)?;
        let actual =
            filesystem::open_directory(&self.root, OsStr::new(state::RESERVED), &self.mount)?;
        ensure!(
            filesystem::identity(&actual)? == self.directory_identity,
            "published output-state directory changed"
        );
        Ok(())
    }
    fn checkpoint(&mut self, payload: &mut Payload, checkpoint: Checkpoint) -> Result<()> {
        self.validate()?;
        let next = Payload {
            sequence: payload
                .sequence
                .checked_add(1)
                .context("output-state sequence exhausted")?,
            checkpoint,
            ..payload.clone()
        };
        let bytes = state::encode(&next)?;
        if filesystem::names(&self.directory)?
            .iter()
            .any(|n| n == OsStr::new("state.next"))
        {
            let (image, _) =
                filesystem::image(&self.directory, Path::new("state.next"), &self.mount)?;
            ensure!(
                matches!(image, Image::File { .. }),
                "unsafe unpublished checkpoint slot"
            );
            filesystem::remove(
                &self.directory,
                OsStr::new("state.next"),
                false,
                &self.mount,
                &mut self.observer,
                "old-state-next",
            )?;
        }
        filesystem::write_new(
            &self.directory,
            OsStr::new("state.next"),
            &bytes,
            0o600,
            &self.mount,
            &mut self.observer,
            "checkpoint",
        )?;
        self.validate()?;
        filesystem::rename(
            &self.directory,
            OsStr::new("state.next"),
            &self.directory,
            OsStr::new("state.json"),
            false,
            &self.mount,
            &mut self.observer,
            "checkpoint",
        )?;
        // A failure after rename is ambiguous. Callers retain the visible checkpoint, never
        // overwrite it with a guessed rollback based on this in-memory payload.
        *payload = next;
        Ok(())
    }
    fn tx_directory(&self, tx: &Transaction, required: bool) -> Result<Option<File>> {
        self.validate()?;
        if !filesystem::names(&self.directory)?
            .iter()
            .any(|n| n == OsStr::new(&tx_name(tx)))
        {
            ensure!(!required, "missing prepared transaction directory");
            return Ok(None);
        }
        let fd =
            filesystem::open_directory(&self.directory, OsStr::new(&tx_name(tx)), &self.mount)?;
        if let Some(expected) = &tx.directory_identity {
            ensure!(
                filesystem::identity(&fd)? == *expected,
                "transaction directory identity mismatch"
            );
        }
        Ok(Some(fd))
    }
}

fn initialize(
    root: &File,
    path: &Path,
    mount: &Mount,
    observer: &mut Observer<'_>,
) -> Result<(File, Payload)> {
    let payload = Payload {
        format: state::FORMAT.to_owned(),
        profile: Profile::current(),
        anchor_id: state::new_uuid()?,
        root: NativePath::absolute(path)?,
        directory: filesystem::identity(root)?,
        sequence: 0,
        checkpoint: Checkpoint::Idle {
            ledger: Ledger::default(),
        },
    };
    let temporary = format!("{}{}", state::INIT_PREFIX, state::new_uuid()?);
    let dir = filesystem::mkdir(
        root,
        OsStr::new(&temporary),
        0o700,
        mount,
        observer,
        "initial-state-directory",
    )?;
    let bytes = state::encode(&payload)?;
    filesystem::write_new(
        &dir,
        OsStr::new("state.json"),
        &bytes,
        0o600,
        mount,
        observer,
        "initial-checkpoint",
    )?;
    filesystem::sync(&dir, observer, "initial-state-directory")?;
    filesystem::rename(
        root,
        OsStr::new(&temporary),
        root,
        OsStr::new(state::RESERVED),
        true,
        mount,
        observer,
        "initial-state-publication",
    )?;
    Ok((dir, payload))
}

struct Plan {
    transaction: Transaction,
    old: BTreeMap<NativePath, Vec<u8>>,
    new: BTreeMap<NativePath, Vec<u8>>,
}
// One read-only pass shares the actual preimage cache across files and directory transitions.
#[allow(clippy::too_many_lines)]
fn plan(
    root: Option<&File>,
    mount: &Mount,
    ledger: &Ledger,
    publications: Vec<Publication>,
) -> Result<Plan> {
    ensure!(
        !publications.is_empty(),
        "publication must explicitly select an owner"
    );
    let selected: BTreeSet<_> = publications.iter().map(|p| p.owner.clone()).collect();
    ensure!(
        selected.len() == publications.len(),
        "duplicate selected owner"
    );
    let old_files = ledger.files();
    let mut after = ledger.clone();
    after.owners.retain(|o| !selected.contains(&o.key));
    let mut actual: BTreeMap<NativePath, Image> = BTreeMap::new();
    let mut old = BTreeMap::new();
    let mut new = BTreeMap::new();
    let mut inspect = |p: &NativePath| -> Result<Image> {
        if let Some(image) = actual.get(p) {
            return Ok(image.clone());
        }
        let (image, bytes) = if let Some(root) = root {
            filesystem::aliases(root, &p.output()?, mount)?;
            filesystem::image(root, &p.output()?, mount)?
        } else {
            (Image::Absent, None)
        };
        if let Some(bytes) = bytes {
            old.insert(p.clone(), bytes);
        }
        actual.insert(p.clone(), image.clone());
        Ok(image)
    };
    for (path, (owner, _)) in &old_files {
        if selected.contains(owner) {
            ensure!(
                !matches!(inspect(path)?, Image::Directory { .. }),
                "owned file has an incompatible file type or symlink"
            );
        }
    }
    // Inspect the entire concrete destination set before deciding enrollment. A late type/link
    // conflict must remain visible even when an earlier ordinary file is still unowned.
    for publication in &publications {
        for path in publication.files.keys() {
            let key = NativePath::from_relative(path)?;
            ensure!(
                !matches!(inspect(&key)?, Image::Directory { .. })
                    || ledger.directories.iter().any(|d| d.path == key),
                "output destination has an incompatible file type or symlink: {}",
                path.display()
            );
            for parent in path
                .ancestors()
                .skip(1)
                .filter(|p| !p.as_os_str().is_empty())
            {
                let parent = NativePath::from_relative(parent)?;
                ensure!(
                    !matches!(inspect(&parent)?, Image::File { .. })
                        || old_files
                            .get(&parent)
                            .is_some_and(|(owner, _)| selected.contains(owner)),
                    "output parent has an incompatible file type or symlink"
                );
            }
        }
    }
    for publication in publications {
        let mut files = Vec::new();
        for (path, bytes) in publication.files {
            let path = NativePath::from_relative(&path)?;
            if let Some((owner, _)) = old_files.get(&path) {
                ensure!(
                    selected.contains(owner),
                    "output path belongs to another owner: {}",
                    path.output()?.display()
                );
            }
            let current = inspect(&path)?;
            if !old_files.contains_key(&path) {
                ensure!(matches!(current,Image::Absent) || (matches!(current,Image::Directory{..}) && ledger.directories.iter().any(|d|d.path==path)),"unowned output destination: {}; adopt exact generated reference bytes explicitly",path.output()?.display());
            }
            let mode = match current {
                Image::File { data } => data.mode,
                _ => 0o644,
            };
            files.push(OwnedFile {
                path: path.clone(),
                data: FileData::of(&bytes, mode),
            });
            ensure!(
                new.insert(path, bytes).is_none(),
                "cross-owner output collision"
            );
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        after.owners.push(Owner {
            key: publication.owner,
            files,
        });
    }
    after.owners.sort_by(|a, b| a.key.cmp(&b.key));
    let after_files = after.files();
    let mut affected: BTreeSet<_> = old_files
        .iter()
        .filter(|(_, v)| selected.contains(&v.0))
        .map(|(p, _)| p.clone())
        .chain(new.keys().cloned())
        .collect();
    let mut desired_directories = BTreeSet::new();
    for path in after_files.keys().filter(|p| new.contains_key(*p)) {
        for parent in path
            .output()?
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            let parent = NativePath::from_relative(parent)?;
            let image = inspect(&parent)?;
            ensure!(
                !matches!(image, Image::File { .. })
                    || old_files
                        .get(&parent)
                        .is_some_and(|(o, _)| selected.contains(o))
                        && !after_files.contains_key(&parent),
                "output parent has an incompatible file type or symlink"
            );
            if !matches!(image, Image::Directory { .. }) {
                if !after.directories.iter().any(|d| d.path == parent) {
                    after.directories.push(OwnedDirectory {
                        path: parent.clone(),
                        mode: 0o755,
                    });
                }
                affected.insert(parent.clone());
            }
            desired_directories.insert(parent);
        }
    }
    for directory in &ledger.directories {
        let path = directory.path.output()?;
        let selected_parent = old_files.iter().any(|(p, (o, _))| {
            selected.contains(o) && p.output().is_ok_and(|p| p.starts_with(&path))
        });
        if !selected_parent && !after_files.contains_key(&directory.path) {
            continue;
        }
        if after_files
            .keys()
            .any(|p| p != &directory.path && p.output().is_ok_and(|p| p.starts_with(&path)))
        {
            continue;
        }
        let must_remove = after_files.contains_key(&directory.path);
        let removable = if let Some(root) = root {
            subtree_owned(
                root,
                &path,
                mount,
                &old_files,
                &ledger.directories,
                &selected,
            )?
        } else {
            true
        };
        ensure!(
            !must_remove || removable,
            "authored descendant blocks owned directory/file transition: {}",
            path.display()
        );
        if removable && !desired_directories.contains(&directory.path) {
            after.directories.retain(|d| d.path != directory.path);
            affected.insert(directory.path.clone());
        }
    }
    after.directories.sort_by(|a, b| a.path.cmp(&b.path));
    after.validate()?;
    let mut changes = Vec::new();
    for path in affected {
        let before = inspect(&path)?;
        let after_image = if let Some((_, data)) = after_files.get(&path) {
            Image::File { data: data.clone() }
        } else if let Some(directory) = after.directories.iter().find(|d| d.path == path) {
            Image::Directory {
                mode: directory.mode,
            }
        } else {
            Image::Absent
        };
        let index = changes.len();
        let blob = |prefix: &str, image: &Image| match image {
            Image::File { data } => Some(Blob {
                name: format!("{prefix}-{index:08}"),
                data: data.clone(),
                identity: None,
            }),
            _ => None,
        };
        changes.push(Change {
            backup: blob("backup", &before),
            stage: blob("stage", &after_image),
            restore: blob("restore", &before),
            path,
            before,
            after: after_image,
        });
    }
    Ok(Plan {
        transaction: Transaction {
            id: state::new_uuid()?,
            anchor_id: String::new(),
            purpose: Purpose::Publish,
            selected: selected.into_iter().collect(),
            before: ledger.clone(),
            after,
            changes,
            directory_identity: None,
        },
        old,
        new,
    })
}
fn subtree_owned(
    root: &File,
    path: &Path,
    mount: &Mount,
    files: &BTreeMap<NativePath, (OwnerKey, FileData)>,
    directories: &[OwnedDirectory],
    selected: &BTreeSet<OwnerKey>,
) -> Result<bool> {
    let (current, _) = filesystem::image(root, path, mount)?;
    if matches!(current, Image::Absent) {
        return Ok(true);
    }
    let key = NativePath::from_relative(path)?;
    if matches!(current, Image::File { .. }) {
        return Ok(files.get(&key).is_some_and(|(o, _)| selected.contains(o)));
    }
    if !directories.iter().any(|d| d.path == key) {
        return Ok(false);
    }
    let Some((parent, name)) = filesystem::parent(root, path, mount)? else {
        return Ok(true);
    };
    let directory = filesystem::open_directory(&parent, &name, mount)?;
    for child in filesystem::names(&directory)? {
        if !subtree_owned(root, &path.join(child), mount, files, directories, selected)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn publish_observed(
    anchor: &Path,
    publications: Vec<Publication>,
    observer: Observer<'_>,
) -> Result<()> {
    publish_observers(anchor, publications, observer, &mut |_| Ok(()))
}

fn publish_observers(
    anchor: &Path,
    publications: Vec<Publication>,
    mut observer: Observer<'_>,
    admission_observer: Observer<'_>,
) -> Result<()> {
    let path = filesystem::absolute(anchor)?;
    let mut locks = Locks::acquire(&[(path.clone(), Access::Exclusive)])?;
    let existing = locks.existing(&path).ok();
    let mount = Mount::of(&locks.nearest(&path)?)?;
    if let Some(fd) = &existing {
        filesystem::discovery(fd, &mount, true)?;
    }
    let admitted = existing
        .as_ref()
        .map(|fd| read_state(fd, &path, &mount))
        .transpose()?
        .flatten();
    let ledger = if let Some((_, payload)) = &admitted {
        ensure_idle(payload, &path)?.clone()
    } else {
        Ledger::default()
    };
    let mut planned = plan(existing.as_ref(), &mount, &ledger, publications)?;
    if admitted.is_some()
        && planned.transaction.before == planned.transaction.after
        && planned
            .transaction
            .changes
            .iter()
            .all(|c| c.before == c.after)
    {
        return Ok(());
    }
    admission::paths(
        &locks,
        &path,
        planned
            .transaction
            .before
            .files()
            .keys()
            .chain(planned.transaction.after.files().keys())
            .chain(
                planned
                    .transaction
                    .before
                    .directories
                    .iter()
                    .map(|d| &d.path),
            )
            .chain(
                planned
                    .transaction
                    .after
                    .directories
                    .iter()
                    .map(|d| &d.path),
            )
            .map(NativePath::output)
            .collect::<Result<Vec<_>>>()?,
        admission_observer,
    )?;
    locks.revalidate()?;
    let root = locks.create_root(&path, &mut observer)?;
    let (directory, mut payload) = match admitted {
        Some(state) => state,
        None => initialize(&root, &path, &mount, &mut observer)?,
    };
    planned.transaction.anchor_id.clone_from(&payload.anchor_id);
    let directory_identity = filesystem::identity(&directory)?;
    let mut session = Session {
        locks,
        root,
        mount,
        directory,
        directory_identity,
        observer,
    };
    run_plan(&mut session, &mut payload, planned)
}
fn run_plan(session: &mut Session<'_>, payload: &mut Payload, mut planned: Plan) -> Result<()> {
    session.checkpoint(
        payload,
        Checkpoint::Staging {
            transaction: planned.transaction.clone(),
        },
    )?;
    let mut tx = planned.transaction;
    let txdir = filesystem::mkdir(
        &session.directory,
        OsStr::new(&tx_name(&tx)),
        0o700,
        &session.mount,
        &mut session.observer,
        "transaction-directory",
    )?;
    tx.directory_identity = Some(filesystem::identity(&txdir)?);
    for change in &mut tx.changes {
        for (blob, source) in [
            (&mut change.backup, &mut planned.old),
            (&mut change.stage, &mut planned.new),
        ] {
            if let Some(blob) = blob {
                let bytes = source
                    .get(&change.path)
                    .context("planned file bytes absent")?;
                ensure!(
                    FileData::of(bytes, blob.data.mode) == blob.data,
                    "planned bytes changed before staging"
                );
                blob.identity = Some(filesystem::write_new(
                    &txdir,
                    OsStr::new(&blob.name),
                    bytes,
                    blob.data.mode,
                    &session.mount,
                    &mut session.observer,
                    &blob.name,
                )?);
            }
        }
    }
    filesystem::sync(&txdir, &mut session.observer, "complete-transaction")?;
    filesystem::sync(
        &session.directory,
        &mut session.observer,
        "complete-staging",
    )?;
    session.checkpoint(
        payload,
        Checkpoint::Prepared {
            transaction: tx.clone(),
        },
    )?;
    verify_blobs(session, &tx, true)?;
    apply_forward(session, &tx, &txdir)?;
    verify_images(session, &tx, false, false)?;
    filesystem::sync(&session.root, &mut session.observer, "complete-new-output")?;
    session.checkpoint(
        payload,
        Checkpoint::Committed {
            transaction: tx.clone(),
        },
    )?;
    cleanup(session, payload, &tx, false)
}
fn verify_blob(
    session: &Session<'_>,
    directory: &File,
    blob: &Blob,
    required: bool,
) -> Result<Option<Vec<u8>>> {
    let (actual, bytes) = filesystem::image(directory, Path::new(&blob.name), &session.mount)?;
    if matches!(actual, Image::Absent) {
        ensure!(
            !required,
            "missing immutable transaction blob {}",
            blob.name
        );
        return Ok(None);
    }
    ensure!(
        actual
            == Image::File {
                data: blob.data.clone()
            },
        "transaction blob content or mode mismatch: {}",
        blob.name
    );
    if let Some(expected) = &blob.identity {
        let fd = rustix::fs::openat(
            directory,
            blob.name.as_str(),
            rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )?;
        ensure!(
            filesystem::identity(&fd)? == *expected,
            "transaction blob inode identity mismatch: {}",
            blob.name
        );
    }
    Ok(bytes)
}
fn verify_blobs(session: &Session<'_>, tx: &Transaction, require_stages: bool) -> Result<()> {
    let directory = session
        .tx_directory(tx, true)?
        .context("missing prepared directory")?;
    for c in &tx.changes {
        if let Some(blob) = &c.backup {
            verify_blob(session, &directory, blob, true)?;
        }
        if let Some(blob) = &c.stage {
            verify_blob(session, &directory, blob, require_stages)?;
        }
    }
    Ok(())
}
fn current(session: &Session<'_>, change: &Change) -> Result<Image> {
    session.validate()?;
    Ok(filesystem::image(&session.root, &change.path.output()?, &session.mount)?.0)
}
fn verify_images(
    session: &Session<'_>,
    tx: &Transaction,
    before: bool,
    transitions: bool,
) -> Result<()> {
    for change in &tx.changes {
        let actual = current(session, change)?;
        let expected = if before {
            &change.before
        } else {
            &change.after
        };
        ensure!(
            &actual == expected
                || transitions
                    && (actual == change.before
                        || actual == change.after
                        || matches!(actual, Image::Absent)),
            "unexpected output edit during pending operation at {}",
            change.path.output()?.display()
        );
        // A directory's own mode cannot describe an authored descendant. Refuse new children
        // before a recovery is allowed to remove a directory introduced by this transaction.
        if matches!(actual, Image::Directory { .. })
            && (matches!(change.before, Image::File { .. } | Image::Absent)
                || matches!(change.after, Image::File { .. } | Image::Absent))
        {
            let allowed: BTreeSet<_> = tx.changes.iter().map(|c| c.path.clone()).collect();
            ensure!(
                only_recorded_descendants(
                    &session.root,
                    &change.path.output()?,
                    &session.mount,
                    &allowed
                )?,
                "unexpected descendant during pending output recovery"
            );
        }
    }
    Ok(())
}
fn only_recorded_descendants(
    root: &File,
    path: &Path,
    mount: &Mount,
    allowed: &BTreeSet<NativePath>,
) -> Result<bool> {
    let Some((parent, name)) = filesystem::parent(root, path, mount)? else {
        return Ok(true);
    };
    let directory = filesystem::open_directory(&parent, &name, mount)?;
    for name in filesystem::names(&directory)? {
        let child = path.join(name);
        let key = NativePath::from_relative(&child)?;
        if !allowed.contains(&key) {
            return Ok(false);
        }
        if matches!(
            filesystem::image(root, &child, mount)?.0,
            Image::Directory { .. }
        ) && !only_recorded_descendants(root, &child, mount, allowed)?
        {
            return Ok(false);
        }
    }
    Ok(true)
}
fn ordered_changes(tx: &Transaction, deepest: bool) -> Vec<&Change> {
    let mut result: Vec<_> = tx.changes.iter().collect();
    result.sort_by(|a, b| {
        a.path
            .path()
            .expect("admitted path")
            .components()
            .count()
            .cmp(&b.path.path().expect("admitted path").components().count())
            .then(a.path.cmp(&b.path))
    });
    if deepest {
        result.reverse();
    }
    result
}
fn remove_output(session: &mut Session<'_>, change: &Change, expected: &Image) -> Result<()> {
    ensure!(
        current(session, change)? == *expected,
        "output precondition changed before retirement"
    );
    let (parent, name) = filesystem::parent(&session.root, &change.path.output()?, &session.mount)?
        .context("retirement parent missing")?;
    filesystem::remove(
        &parent,
        &name,
        matches!(expected, Image::Directory { .. }),
        &session.mount,
        &mut session.observer,
        "output-retirement",
    )
}
fn make_output_directory(session: &mut Session<'_>, change: &Change, mode: u32) -> Result<()> {
    ensure!(
        matches!(current(session, change)?, Image::Absent),
        "directory creation precondition changed"
    );
    let (parent, name) = filesystem::parent(&session.root, &change.path.output()?, &session.mount)?
        .context("new directory parent missing")?;
    filesystem::mkdir(
        &parent,
        &name,
        mode,
        &session.mount,
        &mut session.observer,
        "output-directory",
    )?;
    Ok(())
}
fn install(
    session: &mut Session<'_>,
    change: &Change,
    directory: &File,
    blob: &Blob,
    expected: &Image,
) -> Result<()> {
    verify_blob(session, directory, blob, true)?;
    ensure!(
        current(session, change)? == *expected,
        "output precondition changed before install"
    );
    let (parent, name) = filesystem::parent(&session.root, &change.path.output()?, &session.mount)?
        .context("installation parent missing")?;
    filesystem::rename(
        directory,
        OsStr::new(&blob.name),
        &parent,
        &name,
        matches!(expected, Image::Absent),
        &session.mount,
        &mut session.observer,
        "output-installation",
    )?;
    let fd = rustix::fs::openat(
        &parent,
        &name,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )?;
    filesystem::sync(&fd, &mut session.observer, "installed-output")
}
fn apply_forward(session: &mut Session<'_>, tx: &Transaction, txdir: &File) -> Result<()> {
    verify_images(session, tx, true, false)?;
    for c in ordered_changes(tx, true) {
        if c.before != c.after
            && !matches!(c.before, Image::Absent)
            && (matches!(c.after, Image::Absent)
                || std::mem::discriminant(&c.before) != std::mem::discriminant(&c.after))
        {
            remove_output(session, c, &c.before)?;
        }
    }
    for c in ordered_changes(tx, false) {
        if let Image::Directory { mode } = c.after {
            if !matches!(c.before, Image::Directory { .. }) {
                make_output_directory(session, c, mode)?;
            }
        }
    }
    for c in &tx.changes {
        if c.before != c.after {
            if let Some(stage) = &c.stage {
                let expected = if matches!(c.before, Image::File { .. }) {
                    c.before.clone()
                } else {
                    Image::Absent
                };
                install(session, c, txdir, stage, &expected)?;
            }
        }
    }
    Ok(())
}
fn cleanup(
    session: &mut Session<'_>,
    payload: &mut Payload,
    tx: &Transaction,
    before: bool,
) -> Result<()> {
    if let Some(directory) = session.tx_directory(tx, false)? {
        let allowed = tx.member_names();
        let names = filesystem::names(&directory)?;
        ensure!(
            names
                .iter()
                .all(|n| n.to_str().is_some_and(|n| allowed.contains(n))),
            "unexplained transaction cleanup member"
        );
        for name in names {
            session.validate()?;
            let (image, _) = filesystem::image(&directory, Path::new(&name), &session.mount)?;
            ensure!(
                matches!(image, Image::File { .. }),
                "unsafe cleanup member type"
            );
            filesystem::remove(
                &directory,
                &name,
                false,
                &session.mount,
                &mut session.observer,
                "transaction-cleanup",
            )?;
        }
        filesystem::sync(&directory, &mut session.observer, "empty-transaction")?;
        filesystem::remove(
            &session.directory,
            OsStr::new(&tx_name(tx)),
            true,
            &session.mount,
            &mut session.observer,
            "transaction-directory-cleanup",
        )?;
    }
    filesystem::sync(
        &session.directory,
        &mut session.observer,
        "transaction-cleanup-complete",
    )?;
    session.checkpoint(
        payload,
        Checkpoint::Idle {
            ledger: if before {
                tx.before.clone()
            } else {
                tx.after.clone()
            },
        },
    )
}
fn recover_observed(anchor: &Path, observer: Observer<'_>) -> Result<()> {
    let path = filesystem::absolute(anchor)?;
    let locks = Locks::acquire(&[(path.clone(), Access::Exclusive)])?;
    let root = locks.existing(&path)?;
    let mount = Mount::of(&root)?;
    filesystem::discovery(&root, &mount, true)?;
    let Some((directory, mut payload)) = read_state(&root, &path, &mount)? else {
        return Ok(());
    };
    let directory_identity = filesystem::identity(&directory)?;
    let mut session = Session {
        locks,
        root,
        mount,
        directory,
        directory_identity,
        observer,
    };
    let checkpoint = payload.checkpoint.clone();
    match checkpoint {
        Checkpoint::Idle { .. } => Ok(()),
        Checkpoint::Staging { transaction } => {
            verify_images(&session, &transaction, true, false)?;
            cleanup(&mut session, &mut payload, &transaction, true)
        }
        Checkpoint::Prepared { transaction } => restore(&mut session, &mut payload, transaction),
        Checkpoint::Committed { transaction } => {
            verify_images(&session, &transaction, false, false)?;
            filesystem::sync(
                &session.directory,
                &mut session.observer,
                "observed-committed-decision",
            )?;
            cleanup(&mut session, &mut payload, &transaction, false)
        }
        Checkpoint::Restored { transaction } => {
            verify_images(&session, &transaction, true, false)?;
            filesystem::sync(
                &session.directory,
                &mut session.observer,
                "observed-restored-decision",
            )?;
            cleanup(&mut session, &mut payload, &transaction, true)
        }
    }
}
fn restore(session: &mut Session<'_>, payload: &mut Payload, mut tx: Transaction) -> Result<()> {
    verify_blobs(session, &tx, false)?;
    verify_images(session, &tx, true, true)?;
    let txdir = session
        .tx_directory(&tx, true)?
        .context("prepared transaction missing")?;
    for c in ordered_changes(&tx, true) {
        let actual = current(session, c)?;
        if actual != c.before
            && !matches!(actual, Image::Absent)
            && (matches!(c.before, Image::Absent)
                || std::mem::discriminant(&actual) != std::mem::discriminant(&c.before))
        {
            remove_output(session, c, &actual)?;
        }
    }
    for c in ordered_changes(&tx, false) {
        if let Image::Directory { mode } = c.before {
            if matches!(current(session, c)?, Image::Absent) {
                make_output_directory(session, c, mode)?;
            }
        }
    }
    for index in 0..tx.changes.len() {
        let change = tx.changes[index].clone();
        let Some(backup) = &change.backup else {
            continue;
        };
        let actual = current(session, &change)?;
        if actual == change.before {
            continue;
        }
        let bytes =
            verify_blob(session, &txdir, backup, true)?.context("immutable preimage absent")?;
        let mut restore = change
            .restore
            .clone()
            .context("restore inventory missing")?;
        if filesystem::names(&txdir)?
            .iter()
            .any(|n| n == OsStr::new(&restore.name))
        {
            filesystem::remove(
                &txdir,
                OsStr::new(&restore.name),
                false,
                &session.mount,
                &mut session.observer,
                "partial-restore-stage",
            )?;
        }
        restore.identity = Some(filesystem::write_new(
            &txdir,
            OsStr::new(&restore.name),
            &bytes,
            restore.data.mode,
            &session.mount,
            &mut session.observer,
            "restore-stage",
        )?);
        tx.changes[index].restore = Some(restore.clone());
        session.checkpoint(
            payload,
            Checkpoint::Prepared {
                transaction: tx.clone(),
            },
        )?;
        install(session, &change, &txdir, &restore, &actual)?;
    }
    verify_images(session, &tx, true, false)?;
    filesystem::sync(
        &session.root,
        &mut session.observer,
        "complete-restored-output",
    )?;
    session.checkpoint(
        payload,
        Checkpoint::Restored {
            transaction: tx.clone(),
        },
    )?;
    cleanup(session, payload, &tx, true)
}

/// The native integration target compiles this exact engine. Hooks are absent from the CLI
/// surface and cannot be selected by a production environment variable.
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod probe {
    use super::*;
    pub(crate) fn publish_admission(
        anchor: &Path,
        files: &[(&str, &str)],
        observer: &mut dyn FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        publish_observers(
            anchor,
            vec![Publication::tree("synthesis", files.iter().copied())?],
            &mut |_| Ok(()),
            observer,
        )
    }
    pub(crate) fn adopt_admission(
        anchor: &Path,
        reference: &Path,
        observer: &mut dyn FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        adopt_observers(
            anchor,
            reference,
            "synthesis",
            None,
            &mut |_| Ok(()),
            observer,
        )
    }
    pub(crate) fn publish(
        anchor: &Path,
        files: &[(&str, &str)],
        observer: &mut dyn FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        publish_observed(
            anchor,
            vec![Publication::tree("synthesis", files.iter().copied())?],
            observer,
        )
    }
    pub(crate) fn recover(
        anchor: &Path,
        observer: &mut dyn FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        recover_observed(anchor, observer)
    }
    pub(crate) fn adopt(
        anchor: &Path,
        reference: &Path,
        observer: &mut dyn FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        adopt_observed(anchor, reference, "synthesis", None, observer)
    }
    pub(crate) fn check_mount(anchor: &Path, other: &Path) -> Result<()> {
        Mount::of(&File::open(anchor)?)?.check(&File::open(other)?)
    }
}
