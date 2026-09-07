//! Filesystem acquisition only: semantic readers retain document and format authority.
//!
//! An immediate `ess-inputs.yaml` opts an explicit directory into exact role selection.
//! The configuration is never an authored source or a persisted provenance document.
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use ess_conformance::coverage::SourceIdentity;
use serde::Deserialize;

const MANIFEST: &str = "ess-inputs.yaml";
const FORMAT: &str = "ess-inputs/1";

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Specification,
    Authored,
    Coverage,
}
impl Kind {
    fn role(self) -> &'static str {
        match self {
            Self::Specification => "specification",
            Self::Authored | Self::Coverage => "scenarios",
        }
    }
}

pub(crate) struct Input {
    pub(crate) identity: String,
    pub(crate) origin: PathBuf,
    pub(crate) text: String,
}

/// All required values are decoded strictly. In particular YAML scalars such as `true` and `42`
/// are not converted to strings by `serde_yaml`'s permissive String deserializer.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(deserialize_with = "string")]
    format: String,
    #[serde(deserialize_with = "strings")]
    specification: Vec<String>,
    #[serde(deserialize_with = "strings")]
    scenarios: Vec<String>,
}
fn string<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    match serde_yaml::Value::deserialize(d)? {
        serde_yaml::Value::String(s) => Ok(s),
        _ => Err(serde::de::Error::custom("expected a string")),
    }
}
fn strings<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    let serde_yaml::Value::Sequence(values) = serde_yaml::Value::deserialize(d)? else {
        return Err(serde::de::Error::custom("expected a list of strings"));
    };
    values
        .into_iter()
        .map(|v| match v {
            serde_yaml::Value::String(s) => Ok(s),
            _ => Err(serde::de::Error::custom("expected a string list entry")),
        })
        .collect()
}

/// Omission never probes a default directory or the specification's inactive role.
pub(crate) fn authored(path: Option<&Path>, coverage: bool) -> Result<Vec<Input>> {
    path.map_or_else(
        || Ok(Vec::new()),
        |path| {
            acquire(
                path,
                if coverage {
                    Kind::Coverage
                } else {
                    Kind::Authored
                },
            )
        },
    )
}

pub(crate) fn acquire(path: &Path, kind: Kind) -> Result<Vec<Input>> {
    // Keep the requested spelling until manifest-mode link policy has been applied.
    // Direct files do not consult parent or ancestor configuration.
    if path.is_dir() {
        let manifest = path.join(MANIFEST);
        match fs::symlink_metadata(&manifest) {
            Ok(metadata) => {
                let repair = || {
                    format!(
                    "refused {} input {} using {}: list real contained files in the {} list; \
                     rename a legacy source occupying {MANIFEST}, or adopt {FORMAT} and pass its containing directory",
                    kind.role(), path.display(), manifest.display(), kind.role()
                )
                };
                return (|| {
                    if !metadata.is_file() || metadata.file_type().is_symlink() {
                        bail!("{MANIFEST} must be a regular non-symlink file");
                    }
                    manifest_inputs(path, &manifest, kind)
                })()
                .map_err(|error: anyhow::Error| anyhow::anyhow!("{}: {error:#}", repair()));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "reading configuration {} for {}; repair the manifest before retrying",
                        manifest.display(),
                        path.display()
                    )
                })
            }
        }
    }
    match kind {
        Kind::Specification => legacy_specification(path),
        Kind::Authored => legacy_authored(path),
        Kind::Coverage => legacy_coverage(path),
    }
}

fn manifest_selection(manifest: &Path, kind: Kind) -> Result<Vec<String>> {
    let text =
        fs::read_to_string(manifest).with_context(|| format!("reading {}", manifest.display()))?;
    // Struct deserialization rejects duplicate top-level keys and multiple YAML documents.
    let configuration: Manifest = serde_yaml::from_str(&text).context("invalid input manifest")?;
    if configuration.format != FORMAT {
        bail!(
            "unsupported format {:?}; expected {FORMAT}",
            configuration.format
        );
    }
    let mut names = BTreeSet::new();
    for (role, entries) in [
        ("specification", &configuration.specification),
        ("scenarios", &configuration.scenarios),
    ] {
        for entry in entries {
            SourceIdentity::new(entry.clone()).with_context(|| format!("{role} entry {entry:?}: use a nonempty root-relative path with no empty/dot segments, backslash, colon or controls"))?;
            if !names.insert(entry) {
                bail!("{role} entry {entry:?} repeats a path within or across the two lists; list each path once in exactly one role");
            }
        }
    }
    let mut selected = match kind {
        Kind::Specification => configuration.specification,
        Kind::Authored | Kind::Coverage => configuration.scenarios,
    };
    if selected.is_empty() {
        bail!("the explicit {} list selected no {} files; list the intended files or select a file/legacy child directory",kind.role(),if matches!(kind,Kind::Specification){"specification"}else{"authored"});
    }
    selected.sort();
    Ok(selected)
}

fn manifest_inputs(path: &Path, manifest: &Path, kind: Kind) -> Result<Vec<Input>> {
    let selected = manifest_selection(manifest, kind)?;
    // Components discard only redundant trailing separators / `.` so `linked-root/` cannot
    // hide the selected root's symlink from lstat. Parent components remain unresolved.
    let requested_root: PathBuf = path.components().collect();
    if fs::symlink_metadata(&requested_root)?
        .file_type()
        .is_symlink()
    {
        bail!(
            "selected root {} is a symlink; select its real directory",
            path.display()
        );
    }
    let root = path
        .canonicalize()
        .with_context(|| format!("resolving {}", path.display()))?;
    let mut targets = BTreeSet::new();
    let mut files = Vec::new();
    for identity in selected {
        let mut file = root.clone();
        for segment in identity.split('/') {
            file.push(segment);
            let metadata = fs::symlink_metadata(&file).with_context(|| {
                format!(
                    "{} entry {identity:?}: reading {}; list an existing regular contained file",
                    kind.role(),
                    file.display()
                )
            })?;
            if metadata.file_type().is_symlink() {
                bail!(
                    "{} entry {identity:?} contains a symlink at {}; list a real contained file",
                    kind.role(),
                    file.display()
                );
            }
        }
        if !fs::symlink_metadata(&file)?.is_file() {
            bail!("{} entry {identity:?} is not a regular file; list a file rather than a directory or special node",kind.role());
        }
        let canonical = file.canonicalize().with_context(|| {
            format!(
                "{} entry {identity:?}: resolving {}",
                kind.role(),
                file.display()
            )
        })?;
        if !canonical.starts_with(&root) {
            bail!(
                "{} entry {identity:?} escapes the selected root",
                kind.role()
            );
        }
        if !targets.insert(canonical) {
            bail!(
                "{} entry {identity:?} repeats a canonical target; list each target once",
                kind.role()
            );
        }
        files.push((identity, path.join(file.strip_prefix(&root)?)));
    }
    // Validate the entire active selection before reading any selected source. Inactive paths
    // have had lexical checks only; unlisted paths were never enumerated or inspected.
    files
        .into_iter()
        .map(|(identity, origin)| {
            let text = fs::read_to_string(&origin).with_context(|| {
                format!(
                    "{} entry {identity:?}: reading {}",
                    kind.role(),
                    origin.display()
                )
            })?;
            Ok(Input {
                identity,
                origin,
                text,
            })
        })
        .collect()
}

fn read(identity: String, origin: PathBuf) -> Result<Input> {
    let text =
        fs::read_to_string(&origin).with_context(|| format!("reading {}", origin.display()))?;
    Ok(Input {
        identity,
        origin,
        text,
    })
}
fn yaml(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "yaml" || extension == "yml")
}

fn legacy_specification(path: &Path) -> Result<Vec<Input>> {
    let root = path
        .canonicalize()
        .with_context(|| format!("resolving {}", path.display()))?;
    let mut files = Vec::new();
    if root.is_file() {
        files.push(root.clone());
    } else {
        if !root.join("system.yaml").is_file() {
            bail!("{} is not an ESS specification: a directory must contain `system.yaml`; for a mixed layout use {MANIFEST} or a separate model input directory",root.display());
        }
        let mut visited = BTreeSet::new();
        let mut pending = vec![root.clone()];
        while let Some(directory) = pending.pop() {
            let canonical = directory
                .canonicalize()
                .with_context(|| format!("resolving {}", directory.display()))?;
            if !visited.insert(canonical) {
                continue;
            }
            let mut children = fs::read_dir(&directory)
                .with_context(|| format!("reading {}", directory.display()))?
                .map(|e| e.map(|e| e.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            children.sort();
            // The stack pops the lexically first child before another alias is considered.
            for child in children.into_iter().rev() {
                if child.is_dir() {
                    pending.push(child);
                } else if yaml(&child) {
                    files.push(child);
                }
            }
        }
    }
    files.sort();
    let base = if root.is_file() {
        root.parent().unwrap_or(&root)
    } else {
        &root
    };
    files
        .into_iter()
        .map(|file| {
            read(
                file.strip_prefix(base)
                    .unwrap_or(&file)
                    .display()
                    .to_string(),
                file,
            )
        })
        .collect()
}

fn legacy_authored(path: &Path) -> Result<Vec<Input>> {
    if path.is_file() {
        return Ok(vec![read(path.display().to_string(), path.to_path_buf())?]);
    }
    if !path.is_dir() {
        bail!("{} is not a directory of scenarios", path.display());
    }
    let entries = fs::read_dir(path)
        .with_context(|| format!("reading {}", path.display()))?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    let subdirectories = entries.iter().any(|p| p.is_dir());
    let mut files: Vec<_> = entries.into_iter().filter(|p| yaml(p)).collect();
    if files.is_empty() {
        let hint = if subdirectories {
            "subdirectories are present but are not searched; pass the intended child directory or scenario file with --scenarios"
        } else {
            "pass a directory containing scenario files directly, or one scenario file, with --scenarios"
        };
        bail!("refused --scenarios {}: no .yaml or .yml scenario files found directly in this directory; {hint}",path.display());
    }
    files.sort();
    files
        .into_iter()
        .map(|file| read(file.display().to_string(), file))
        .collect()
}

fn legacy_coverage(path: &Path) -> Result<Vec<Input>> {
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("reading {}", path.display()))?;
    if metadata.file_type().is_symlink() {
        bail!("coverage authored root/file is a symlink");
    }
    let (root, mut files) = if metadata.is_file() {
        (
            path.parent().unwrap_or(Path::new(".")),
            vec![path.to_path_buf()],
        )
    } else if metadata.is_dir() {
        let files = fs::read_dir(path)?
            .map(|e| e.map(|e| e.path()))
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            .filter(|p| yaml(p))
            .collect::<Vec<_>>();
        if files.is_empty() {
            bail!("refused --scenarios: no .yaml or .yml scenario files found directly in this directory; subdirectories are not searched");
        }
        (path, files)
    } else {
        bail!("coverage authored input must be a regular file or directory");
    };
    let root = if root.as_os_str().is_empty() {
        Path::new(".")
    } else {
        root
    };
    if fs::symlink_metadata(root)?.file_type().is_symlink() {
        bail!("coverage authored root is a symlink");
    }
    files.sort();
    files
        .into_iter()
        .map(|file| {
            let metadata = fs::symlink_metadata(&file)?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                bail!(
                    "coverage authored input is not a regular non-symlink file: {}",
                    file.display()
                );
            }
            let identity = file
                .strip_prefix(root)
                .or_else(|_| {
                    file.file_name()
                        .map(Path::new)
                        .ok_or_else(|| anyhow::anyhow!("missing file name"))
                })?
                .to_str()
                .context("authored identity is not UTF-8")?
                .to_owned();
            read(identity, file)
        })
        .collect()
}
