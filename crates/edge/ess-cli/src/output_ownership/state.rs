//! Closed, canonical private output-state data. A checksum detects inconsistency, not forgery.
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Component, Path, PathBuf},
};

pub(super) const FORMAT: &str = "ess-output-state/1";
pub(super) const RESERVED: &str = ".ess-output";
pub(super) const INIT_PREFIX: &str = ".ess-output-init-";

pub(super) fn digest(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
pub(super) fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    bytes
        .iter()
        .flat_map(|b| {
            [
                char::from(DIGITS[usize::from(b >> 4)]),
                char::from(DIGITS[usize::from(b & 15)]),
            ]
        })
        .collect()
}
fn unhex(value: &str) -> Result<Vec<u8>> {
    ensure!(
        value.len() % 2 == 0
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "invalid canonical native hex"
    );
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| Ok(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?))
        .collect()
}
pub(super) fn reserved(name: &OsStr) -> bool {
    let bytes = name.as_bytes().to_ascii_lowercase();
    bytes == RESERVED.as_bytes() || bytes.starts_with(INIT_PREFIX.as_bytes())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativePath {
    encoding: Encoding,
    components: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Encoding {
    UnixBytes1,
}
impl NativePath {
    pub(super) fn from_relative(path: &Path) -> Result<Self> {
        let mut components = Vec::new();
        for part in path.components() {
            match part {
                Component::Normal(name) => components.push(hex(name.as_bytes())),
                _ => bail!("non-relative output path: {}", path.display()),
            }
        }
        let result = Self {
            encoding: Encoding::UnixBytes1,
            components,
        };
        result.validate()?;
        Ok(result)
    }
    pub(super) fn root() -> Self {
        Self {
            encoding: Encoding::UnixBytes1,
            components: Vec::new(),
        }
    }
    pub(super) fn absolute(path: &Path) -> Result<Self> {
        ensure!(path.is_absolute(), "anchor is not absolute");
        Self::from_relative(path.strip_prefix("/")?)
    }
    pub(super) fn path(&self) -> Result<PathBuf> {
        self.validate()?;
        Ok(self
            .components
            .iter()
            .map(|part| unhex(part).map(OsString::from_vec))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect())
    }
    pub(super) fn validate(&self) -> Result<()> {
        for part in &self.components {
            let bytes = unhex(part)?;
            ensure!(
                !bytes.is_empty()
                    && bytes != b"."
                    && bytes != b".."
                    && !bytes.contains(&0)
                    && !bytes.contains(&b'/'),
                "unsafe native path component"
            );
        }
        Ok(())
    }
    pub(super) fn output(&self) -> Result<PathBuf> {
        let path = self.path()?;
        ensure!(
            !path.as_os_str().is_empty() && !path.components().any(|c| reserved(c.as_os_str())),
            "output path intersects reserved ownership state"
        );
        Ok(path)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(super) enum Family {
    #[serde(rename = "projection:docs")]
    Docs,
    #[serde(rename = "projection:site")]
    Site,
    #[serde(rename = "projection:schema")]
    Schema,
    #[serde(rename = "projection:openapi")]
    Openapi,
    #[serde(rename = "projection:asyncapi")]
    Asyncapi,
    #[serde(rename = "projection:docs-ir")]
    DocsIr,
    #[serde(rename = "synthesis")]
    Synthesis,
    #[serde(rename = "compose")]
    Compose,
    #[serde(rename = "conformance-go")]
    ConformanceGo,
    #[serde(rename = "conformance-browser")]
    ConformanceBrowser,
    #[serde(rename = "buildkit")]
    Buildkit,
    #[serde(rename = "helm")]
    Helm,
    #[serde(rename = "kubernetes")]
    Kubernetes,
    #[serde(rename = "openapi-file")]
    OpenapiFile,
    #[serde(rename = "types-bundle")]
    TypesBundle,
    #[serde(rename = "model-types")]
    ModelTypes,
    #[serde(rename = "cli-binding")]
    CliBinding,
    #[serde(rename = "normalization")]
    Normalization,
    #[serde(rename = "typescript-file")]
    TypescriptFile,
    #[serde(rename = "realization-markdown")]
    RealizationMarkdown,
}
impl Family {
    pub(super) fn parse(value: &str) -> Result<Self> {
        serde_json::from_value(serde_json::Value::String(value.to_owned()))
            .context("unknown fixed output owner family")
    }
    pub(super) fn standalone(self) -> bool {
        matches!(
            self,
            Self::OpenapiFile | Self::TypescriptFile | Self::RealizationMarkdown
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnerKey {
    pub(super) family: Family,
    pub(super) location: NativePath,
}
impl OwnerKey {
    pub(super) fn tree(family: Family) -> Result<Self> {
        ensure!(!family.standalone(), "standalone owner requires a filename");
        Ok(Self {
            family,
            location: NativePath::root(),
        })
    }
    pub(super) fn file(family: Family, name: &OsStr) -> Result<Self> {
        ensure!(
            super::filename(Path::new(name))? == name,
            "standalone owner requires one exact native filename"
        );
        let result = Self {
            family,
            location: NativePath::from_relative(Path::new(name))?,
        };
        result.validate()?;
        Ok(result)
    }
    pub(super) fn validate(&self) -> Result<()> {
        self.location.validate()?;
        ensure!(
            if self.family.standalone() {
                self.location.components.len() == 1
            } else {
                self.location.components.is_empty()
            },
            "owner family/location mismatch"
        );
        if self.family.standalone() {
            self.location.output()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FileData {
    pub(super) length: u64,
    pub(super) digest: String,
    pub(super) mode: u32,
}
impl FileData {
    pub(super) fn of(bytes: &[u8], mode: u32) -> Self {
        Self {
            length: bytes.len() as u64,
            digest: digest(bytes),
            mode,
        }
    }
    pub(super) fn validate(&self) -> Result<()> {
        ensure!(
            self.digest.len() == 64 && unhex(&self.digest)?.len() == 32,
            "invalid SHA256"
        );
        mode(self.mode)
    }
}
pub(super) fn mode(value: u32) -> Result<()> {
    ensure!(
        value & !0o777 == 0,
        "unsupported privilege or nonordinary mode bits"
    );
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(super) enum Image {
    Absent,
    File { data: FileData },
    Directory { mode: u32 },
}
impl Image {
    pub(super) fn validate(&self) -> Result<()> {
        match self {
            Self::Absent => Ok(()),
            Self::File { data } => data.validate(),
            Self::Directory { mode: value } => mode(*value),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnedFile {
    pub(super) path: NativePath,
    pub(super) data: FileData,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Owner {
    pub(super) key: OwnerKey,
    pub(super) files: Vec<OwnedFile>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnedDirectory {
    pub(super) path: NativePath,
    pub(super) mode: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Ledger {
    pub(super) owners: Vec<Owner>,
    pub(super) directories: Vec<OwnedDirectory>,
}
fn strictly_sorted<T: Ord>(values: impl IntoIterator<Item = T>) -> Result<()> {
    let mut previous = None;
    for v in values {
        ensure!(
            previous.as_ref().is_none_or(|p| p < &v),
            "duplicate or unsorted inventory"
        );
        previous = Some(v);
    }
    Ok(())
}
impl Ledger {
    pub(super) fn validate(&self) -> Result<()> {
        strictly_sorted(self.owners.iter().map(|o| &o.key))?;
        strictly_sorted(self.directories.iter().map(|d| &d.path))?;
        let mut files = BTreeSet::new();
        for owner in &self.owners {
            owner.key.validate()?;
            strictly_sorted(owner.files.iter().map(|f| &f.path))?;
            for f in &owner.files {
                f.path.output()?;
                f.data.validate()?;
                ensure!(files.insert(f.path.clone()), "cross-owner output collision");
                ensure!(
                    !owner.key.family.standalone() || f.path == owner.key.location,
                    "standalone owner/file mismatch"
                );
            }
        }
        let mut paths = BTreeMap::new();
        for f in &files {
            insert_path(&mut paths, &f.output()?, true)?;
        }
        for d in &self.directories {
            mode(d.mode)?;
            let p = d.path.output()?;
            ensure!(!files.contains(&d.path), "owned file/directory collision");
            insert_path(&mut paths, &p, false)?;
        }
        Ok(())
    }
    pub(super) fn files(&self) -> BTreeMap<NativePath, (OwnerKey, FileData)> {
        self.owners
            .iter()
            .flat_map(|o| {
                o.files
                    .iter()
                    .map(|f| (f.path.clone(), (o.key.clone(), f.data.clone())))
            })
            .collect()
    }
}
pub(super) fn insert_path(
    into: &mut BTreeMap<PathBuf, (PathBuf, bool)>,
    path: &Path,
    file: bool,
) -> Result<()> {
    let mut original = PathBuf::new();
    let mut folded = PathBuf::new();
    let mut parts = path.components().peekable();
    while let Some(c) = parts.next() {
        original.push(c.as_os_str());
        folded.push(c.as_os_str().to_ascii_lowercase());
        let leaf = file && parts.peek().is_none();
        if let Some((prior, prior_file)) = into.get(&folded) {
            ensure!(
                prior == &original && !leaf && !prior_file,
                "colliding command output paths: {} and {}",
                prior.display(),
                path.display()
            );
        } else {
            into.insert(folded.clone(), (original.clone(), leaf));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    pub(super) device: u64,
    pub(super) inode: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Blob {
    pub(super) name: String,
    pub(super) data: FileData,
    pub(super) identity: Option<Identity>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Change {
    pub(super) path: NativePath,
    pub(super) before: Image,
    pub(super) after: Image,
    pub(super) backup: Option<Blob>,
    pub(super) stage: Option<Blob>,
    pub(super) restore: Option<Blob>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Transaction {
    pub(super) id: String,
    pub(super) anchor_id: String,
    pub(super) purpose: Purpose,
    pub(super) selected: Vec<OwnerKey>,
    pub(super) before: Ledger,
    pub(super) after: Ledger,
    pub(super) changes: Vec<Change>,
    pub(super) directory_identity: Option<Identity>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Purpose {
    Publish,
    Adopt,
}
impl Transaction {
    // Read the complete ledger/path/blob authority checks together before admitting a phase.
    #[allow(clippy::too_many_lines)]
    pub(super) fn validate(&self, anchor: &str, prepared: bool) -> Result<()> {
        uuid(&self.id)?;
        ensure!(
            self.anchor_id == anchor,
            "transaction anchor identity mismatch"
        );
        self.before.validate()?;
        self.after.validate()?;
        strictly_sorted(&self.selected)?;
        ensure!(!self.selected.is_empty(), "empty transaction selection");
        strictly_sorted(self.changes.iter().map(|c| &c.path))?;
        for owner in &self.selected {
            owner.validate()?;
            ensure!(
                self.after.owners.iter().any(|o| &o.key == owner),
                "transaction omits a selected owner's settled inventory"
            );
        }
        let unchanged = |l: &Ledger| {
            l.owners
                .iter()
                .filter(|o| !self.selected.contains(&o.key))
                .cloned()
                .collect::<Vec<_>>()
        };
        ensure!(
            unchanged(&self.before) == unchanged(&self.after),
            "transaction changed an unselected owner"
        );
        let before = self.before.files();
        let after = self.after.files();
        let expected: BTreeSet<_> = before
            .iter()
            .chain(&after)
            .filter(|(_, v)| self.selected.contains(&v.0))
            .map(|(p, _)| p.clone())
            .chain(self.before.directories.iter().map(|d| d.path.clone()))
            .chain(self.after.directories.iter().map(|d| d.path.clone()))
            .collect();
        if self.purpose == Purpose::Publish {
            let required: BTreeSet<_> = before
                .iter()
                .chain(&after)
                .filter(|(_, v)| self.selected.contains(&v.0))
                .map(|(p, _)| p)
                .chain(
                    self.before
                        .directories
                        .iter()
                        .filter(|d| !self.after.directories.contains(d))
                        .map(|d| &d.path),
                )
                .chain(
                    self.after
                        .directories
                        .iter()
                        .filter(|d| !self.before.directories.contains(d))
                        .map(|d| &d.path),
                )
                .collect();
            ensure!(
                required
                    .iter()
                    .all(|p| self.changes.iter().any(|c| &c.path == *p)),
                "incomplete transaction path inventory"
            );
        }
        let mut names = BTreeSet::new();
        for (index, c) in self.changes.iter().enumerate() {
            c.path.output()?;
            c.before.validate()?;
            c.after.validate()?;
            ensure!(
                expected.contains(&c.path),
                "transaction path has no ledger authority"
            );
            let after_image = if let Some((_, data)) = after.get(&c.path) {
                Image::File { data: data.clone() }
            } else if let Some(d) = self.after.directories.iter().find(|d| d.path == c.path) {
                Image::Directory { mode: d.mode }
            } else {
                Image::Absent
            };
            ensure!(
                c.after == after_image,
                "transaction image contradicts its after ledger"
            );
            match &c.before {
                Image::File { .. } => ensure!(
                    before
                        .get(&c.path)
                        .is_some_and(|(o, _)| self.selected.contains(o)),
                    "file preimage lacks selected-owner authority"
                ),
                Image::Directory { .. } => ensure!(
                    self.before.directories.iter().any(|d| d.path == c.path),
                    "directory preimage lacks creation authority"
                ),
                Image::Absent => {}
            }
            for (prefix, blob, image) in [
                ("backup", &c.backup, &c.before),
                ("stage", &c.stage, &c.after),
                ("restore", &c.restore, &c.before),
            ] {
                match (blob, image) {
                    (Some(b), Image::File { data }) => {
                        ensure!(
                            b.name == format!("{prefix}-{index:08}") && names.insert(&b.name),
                            "invalid or duplicate transaction member"
                        );
                        ensure!(&b.data == data, "blob/image mismatch");
                        b.data.validate()?;
                        ensure!(
                            !prepared || prefix == "restore" || b.identity.is_some(),
                            "prepared blob lacks identity"
                        );
                    }
                    (None, Image::File { .. }) => bail!("file snapshot lacks its recorded blob"),
                    (None, _) => {}
                    _ => bail!("blob without file snapshot"),
                }
            }
            if let Image::File { data } = &c.after {
                ensure!(
                    after.get(&c.path).is_some_and(|(_, d)| d == data),
                    "installed file differs from after ledger"
                );
            }
        }
        if prepared {
            ensure!(
                self.directory_identity.is_some(),
                "prepared transaction directory lacks identity"
            );
        }
        if self.purpose == Purpose::Adopt {
            ensure!(
                self.changes.is_empty() && self.before.directories == self.after.directories,
                "adoption attempted output mutation"
            );
            for owner in self
                .before
                .owners
                .iter()
                .filter(|o| self.selected.contains(&o.key))
            {
                ensure!(
                    self.after.owners.contains(owner),
                    "adoption checkpoint changed or shrank an already enrolled owner"
                );
            }
        }
        Ok(())
    }
    pub(super) fn member_names(&self) -> BTreeSet<String> {
        self.changes
            .iter()
            .flat_map(|c| [&c.backup, &c.stage, &c.restore])
            .flatten()
            .map(|b| b.name.clone())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", deny_unknown_fields)]
pub(super) enum Checkpoint {
    Idle { ledger: Ledger },
    Staging { transaction: Transaction },
    Prepared { transaction: Transaction },
    Committed { transaction: Transaction },
    Restored { transaction: Transaction },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Payload {
    pub(super) format: String,
    pub(super) profile: Profile,
    pub(super) anchor_id: String,
    pub(super) root: NativePath,
    pub(super) directory: Identity,
    pub(super) sequence: u64,
    pub(super) checkpoint: Checkpoint,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Profile {
    LinuxLocal1,
    MacosLocal1,
}
impl Profile {
    pub(super) fn current() -> Self {
        #[cfg(target_os = "linux")]
        {
            Self::LinuxLocal1
        }
        #[cfg(target_os = "macos")]
        {
            Self::MacosLocal1
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    checksum: String,
    payload: Payload,
}
fn canonical(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(&serde_json::to_value(value)?)?;
    bytes.push(b'\n');
    Ok(bytes)
}
pub(super) fn encode(payload: &Payload) -> Result<Vec<u8>> {
    validate(payload)?;
    canonical(&Envelope {
        checksum: digest(&canonical(payload)?),
        payload: payload.clone(),
    })
}
pub(super) fn decode(bytes: &[u8]) -> Result<Payload> {
    let envelope: Envelope = serde_json::from_slice(bytes).context("invalid output state")?;
    ensure!(
        canonical(&envelope)? == bytes,
        "output state is not canonical (including duplicate keys)"
    );
    ensure!(
        envelope.checksum == digest(&canonical(&envelope.payload)?),
        "output state checksum mismatch"
    );
    validate(&envelope.payload)?;
    Ok(envelope.payload)
}
fn validate(payload: &Payload) -> Result<()> {
    ensure!(payload.format == FORMAT, "unsupported output-state version");
    ensure!(
        payload.profile == Profile::current(),
        "output-state platform profile mismatch"
    );
    payload.root.validate()?;
    uuid(&payload.anchor_id)?;
    match &payload.checkpoint {
        Checkpoint::Idle { ledger } => ledger.validate(),
        Checkpoint::Staging { transaction } => transaction.validate(&payload.anchor_id, false),
        Checkpoint::Prepared { transaction }
        | Checkpoint::Committed { transaction }
        | Checkpoint::Restored { transaction } => transaction.validate(&payload.anchor_id, true),
    }
}
fn uuid(value: &str) -> Result<()> {
    ensure!(
        value.len() == 36 && [8, 13, 18, 23].iter().all(|i| value.as_bytes()[*i] == b'-'),
        "invalid UUID"
    );
    ensure!(
        value
            .bytes()
            .enumerate()
            .all(|(i, b)| if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }),
        "noncanonical UUID"
    );
    Ok(())
}
pub(super) fn new_uuid() -> Result<String> {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes)
        .map_err(|e| anyhow::anyhow!("obtaining anchor/transaction entropy: {e}"))?;
    bytes[6] = (bytes[6] & 15) | 0x40;
    bytes[8] = (bytes[8] & 63) | 0x80;
    let s = hex(&bytes);
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &s[..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..]
    ))
}
