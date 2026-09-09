//! Original-byte OCI proof shared by the bundle and Helm consumers.
//! This is a finite transport profile, not publisher authorization.
use anyhow::{bail, ensure, Context, Result};
use ess_deployment::Digest;
use serde::{de, Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};

const MANIFEST_LIMIT: u64 = 1024 * 1024;
const SMALL_LIMIT: u64 = 1024 * 1024;
const BUNDLE_LIMIT: u64 = 32 * 1024 * 1024;
const CHART_LIMIT: u64 = 64 * 1024 * 1024;
const DIAGNOSTIC_LIMIT: usize = 64 * 1024;
const DEADLINE: Duration = Duration::from_secs(60);
const MAGIC: &[u8; 8] = b"ESSOCI1\n";
const OCI: &str = "application/vnd.oci.image.manifest.v1+json";
const BUNDLE: &str = "application/vnd.beyond10x.ess.release-bundle.v1";
const BUNDLE_JSON: &str = "application/vnd.beyond10x.ess.release-bundle.v1+json";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const HELM: &str = "application/vnd.cncf.helm.config.v1+json";
const CHART: &str = "application/vnd.cncf.helm.chart.content.v1.tar+gzip";
const PROVENANCE: &str = "application/vnd.cncf.helm.chart.provenance.v1.prov";

/// Which closed original-byte transport profile a cache entry belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    /// An `ess-release-bundle/1` payload.
    Bundle,
    /// A Helm chart payload.
    Helm,
}
impl Profile {
    fn namespace(self) -> &'static str {
        match self {
            Self::Bundle => "bundle",
            Self::Helm => "helm",
        }
    }
}

// Unlike Option<T>, this distinguishes absent from an explicitly invalid null.
#[derive(Default, Debug)]
enum Optional<T> {
    #[default]
    Absent,
    Present(T),
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Optional<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        T::deserialize(d).map(Self::Present)
    }
}

#[derive(Debug, Default)]
struct Annotations;
impl<'de> Deserialize<'de> for Annotations {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Annotations;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("bounded unique string annotations")
            }
            fn visit_map<M: de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<Annotations, M::Error> {
                let mut keys = std::collections::BTreeSet::new();
                while let Some((key, value)) = map.next_entry::<String, String>()? {
                    if key.len() > 4096 || value.len() > 8192 || keys.len() >= 64 {
                        return Err(de::Error::custom("annotation limit exceeded"));
                    }
                    if !keys.insert(key) {
                        return Err(de::Error::custom("duplicate annotation key"));
                    }
                }
                Ok(Annotations)
            }
        }
        d.deserialize_map(Visitor)
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Descriptor {
    media_type: String,
    digest: Digest,
    size: u64,
    #[serde(default, rename = "annotations")]
    _annotations: Optional<Annotations>,
    #[serde(default)]
    data: Optional<String>,
}
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Manifest {
    schema_version: u64,
    media_type: String,
    #[serde(default)]
    artifact_type: Optional<String>,
    config: Descriptor,
    layers: Vec<Descriptor>,
    #[serde(default, rename = "annotations")]
    _annotations: Optional<Annotations>,
}
struct CheckedManifest {
    manifest: Manifest,
    limits: Vec<u64>,
    payload_index: usize,
}
fn manifest(bytes: &[u8], requested: &Digest, profile: Profile) -> Result<CheckedManifest> {
    ensure!(
        bytes.len() as u64 <= MANIFEST_LIMIT,
        "OCI manifest limit exceeded"
    );
    ensure!(
        Digest::of_bytes(bytes) == *requested,
        "requested OCI manifest digest mismatch"
    );
    let m: Manifest =
        serde_json::from_slice(bytes).context("decoding closed OCI manifest profile")?;
    ensure!(
        m.schema_version == 2 && m.media_type == OCI,
        "unsupported OCI manifest profile"
    );
    let mut limits = vec![SMALL_LIMIT];
    let payload_index;
    match profile {
        Profile::Bundle => {
            ensure!(
                matches!(&m.artifact_type, Optional::Present(t) if t == BUNDLE),
                "unsupported bundle artifactType"
            );
            ensure!(
                m.config.media_type == EMPTY
                    && m.config.size == 2
                    && m.config.digest == Digest::of_bytes(b"{}"),
                "bundle config must identify exact empty JSON"
            );
            ensure!(
                matches!(&m.config.data, Optional::Absent)
                    || matches!(&m.config.data, Optional::Present(v) if v == "e30="),
                "unsupported bundle config data"
            );
            ensure!(
                m.layers.len() == 1 && m.layers[0].media_type == BUNDLE_JSON,
                "bundle requires one JSON layer"
            );
            ensure!(
                matches!(m.layers[0].data, Optional::Absent),
                "unsupported bundle layer data"
            );
            limits.push(BUNDLE_LIMIT);
            payload_index = 1;
        }
        Profile::Helm => {
            ensure!(
                matches!(&m.artifact_type, Optional::Absent)
                    || matches!(&m.artifact_type, Optional::Present(t) if t == HELM),
                "unsupported Helm artifactType"
            );
            ensure!(
                m.config.media_type == HELM,
                "unsupported Helm config profile"
            );
            ensure!(
                (1..=2).contains(&m.layers.len()),
                "Helm requires one chart and optional provenance"
            );
            let mut chart = None;
            let mut provenance = false;
            for (index, layer) in m.layers.iter().enumerate() {
                match layer.media_type.as_str() {
                    CHART if chart.is_none() => {
                        chart = Some(index + 1);
                        limits.push(CHART_LIMIT);
                    }
                    PROVENANCE if !provenance => {
                        provenance = true;
                        limits.push(SMALL_LIMIT);
                    }
                    _ => bail!("unsupported or duplicate Helm layer profile"),
                }
            }
            payload_index = chart.context("Helm chart layer is missing")?;
            ensure!(
                std::iter::once(&m.config)
                    .chain(&m.layers)
                    .all(|d| matches!(d.data, Optional::Absent)),
                "unsupported Helm descriptor data"
            );
        }
    }
    for (d, limit) in std::iter::once(&m.config).chain(&m.layers).zip(&limits) {
        ensure!(
            d.size <= *limit,
            "OCI descriptor size exceeds local limit {limit}"
        );
    }
    Ok(CheckedManifest {
        manifest: m,
        limits,
        payload_index,
    })
}
fn check_blob(bytes: &[u8], d: &Descriptor) -> Result<()> {
    ensure!(bytes.len() as u64 == d.size, "OCI descriptor size mismatch");
    ensure!(
        Digest::of_bytes(bytes) == d.digest,
        "OCI descriptor digest mismatch"
    );
    Ok(())
}
fn check_payload(bytes: &[u8], profile: Profile) -> Result<()> {
    if profile == Profile::Bundle {
        bundle(bytes)?;
    }
    Ok(())
}
/// Reads verified original bytes as a release bundle.
pub fn bundle(bytes: &[u8]) -> Result<ess_deployment::ReleaseBundle> {
    let text = std::str::from_utf8(bytes).context("bundle payload is not UTF-8")?;
    let parsed = ess_deployment::ReleaseBundle::from_json(text)
        .context("parsing verified OCI bundle JSON")?;
    let checked =
        ess_deployment::verify_release_bundle(parsed).context("validating verified OCI bundle")?;
    ensure!(
        text == checked.to_canonical_json(),
        "OCI bundle is not canonical release-bundle JSON"
    );
    Ok(checked)
}

struct Proof {
    original_manifest: Vec<u8>,
    blobs: Vec<Vec<u8>>,
    payload_index: usize,
}
impl Proof {
    fn payload(&self) -> &[u8] {
        &self.blobs[self.payload_index]
    }
    fn write(&self, out: &mut impl Write) -> Result<()> {
        out.write_all(MAGIC)?;
        out.write_all(&(self.original_manifest.len() as u64).to_be_bytes())?;
        out.write_all(&self.original_manifest)?;
        out.write_all(&u32::try_from(self.blobs.len())?.to_be_bytes())?;
        for blob in &self.blobs {
            out.write_all(&(blob.len() as u64).to_be_bytes())?;
            out.write_all(blob)?;
        }
        Ok(())
    }
}
fn bytes_with_limit(reader: impl Read, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "OCI returned bytes exceed local limit {limit}"
    );
    Ok(bytes)
}
fn framed_bytes(reader: &mut impl Read, limit: u64) -> Result<Vec<u8>> {
    let mut length = [0; 8];
    reader
        .read_exact(&mut length)
        .context("truncated OCI proof length")?;
    let length = u64::from_be_bytes(length);
    ensure!(
        length <= limit,
        "OCI proof frame length exceeds local limit {limit}"
    );
    let bytes = bytes_with_limit(reader.take(length), limit)?;
    ensure!(bytes.len() as u64 == length, "truncated OCI proof bytes");
    Ok(bytes)
}
fn read_proof(reader: &mut impl Read, requested: &Digest, profile: Profile) -> Result<Proof> {
    let mut magic = [0; 8];
    reader
        .read_exact(&mut magic)
        .context("truncated OCI proof magic")?;
    ensure!(&magic == MAGIC, "unsupported OCI proof magic");
    let original_manifest = framed_bytes(reader, MANIFEST_LIMIT)?;
    let checked = manifest(&original_manifest, requested, profile)?;
    let mut count = [0; 4];
    reader
        .read_exact(&mut count)
        .context("truncated OCI proof count")?;
    ensure!(
        u32::from_be_bytes(count) as usize == checked.limits.len(),
        "OCI proof blob count mismatch"
    );
    let mut blobs = Vec::new();
    for (descriptor, limit) in std::iter::once(&checked.manifest.config)
        .chain(&checked.manifest.layers)
        .zip(&checked.limits)
    {
        let bytes = framed_bytes(reader, *limit)?;
        check_blob(&bytes, descriptor)?;
        blobs.push(bytes);
    }
    let mut trailing = [0];
    ensure!(
        reader.read(&mut trailing)? == 0,
        "trailing bytes in OCI proof"
    );
    let proof = Proof {
        original_manifest,
        blobs,
        payload_index: checked.payload_index,
    };
    check_payload(proof.payload(), profile)?;
    Ok(proof)
}
fn regular(path: &Path) -> Result<File> {
    ensure!(
        fs::symlink_metadata(path)?.file_type().is_file(),
        "OCI proof/output must be a regular file: {}",
        path.display()
    );
    let file = File::open(path)?;
    ensure!(
        file.metadata()?.is_file(),
        "opened OCI proof/output is nonregular"
    );
    Ok(file)
}
fn read_entry(path: &Path, requested: &Digest, profile: Profile) -> Result<Proof> {
    read_proof(&mut regular(path)?, requested, profile)
        .with_context(|| format!("invalid OCI proof entry {}", path.display()))
}
fn exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

static NEXT: AtomicU64 = AtomicU64::new(0);
fn unique(parent: &Path, prefix: &str) -> PathBuf {
    parent.join(format!(
        ".{prefix}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}
struct StagedFile(PathBuf);
impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
struct AcquisitionDirectory(PathBuf);
impl AcquisitionDirectory {
    fn new(parent: &Path) -> Result<Self> {
        loop {
            let path = unique(parent, "acquire");
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
    }
}
impl Drop for AcquisitionDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn publish(
    proof: &Proof,
    path: &Path,
    requested: &Digest,
    profile: Profile,
    before_link: impl FnOnce(&Path) -> Result<()>,
) -> Result<Proof> {
    let parent = path.parent().context("proof entry requires a parent")?;
    let (stage, mut file) = loop {
        let path = unique(parent, "proof");
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => break (StagedFile(path), file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e).context("creating private OCI proof stage"),
        }
    };
    proof.write(&mut file)?;
    file.flush()?;
    drop(file);
    let verified = read_entry(&stage.0, requested, profile)?;
    before_link(&stage.0)?;
    match fs::hard_link(&stage.0, path) {
        Ok(()) => Ok(verified),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            read_entry(path, requested, profile)
        }
        Err(e) => Err(e).context("atomic no-replacement OCI proof publication failed"),
    }
}

fn drain(mut stream: impl Read + Send + 'static) -> mpsc::Receiver<std::io::Result<Vec<u8>>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let mut retained = Vec::new();
            let mut chunk = [0; 8192];
            loop {
                let n = stream.read(&mut chunk)?;
                if n == 0 {
                    break;
                }
                let keep = n.min(DIAGNOSTIC_LIMIT - retained.len());
                retained.extend_from_slice(&chunk[..keep]);
            }
            Ok(retained)
        })();
        let _ = tx.send(result);
    });
    rx
}
fn fetch(command: &mut Command, deadline: Instant) -> Result<()> {
    ensure!(
        Instant::now() < deadline,
        "OCI acquisition deadline exceeded before client launch"
    );
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("starting ORAS fetch")?;
    let stdout = drain(child.stdout.take().context("ORAS stdout pipe missing")?);
    let stderr = drain(child.stderr.take().context("ORAS stderr pipe missing")?);
    let mut outputs = BTreeMap::new();
    let mut status = None;
    loop {
        if Instant::now() >= deadline {
            // Reap the invocation's owned process before any refusal can escape.
            if status.is_none() {
                let _ = child.kill();
            }
            child.wait().context("reaping timed-out ORAS child")?;
            bail!("OCI acquisition deadline exceeded; owned ORAS child killed and reaped");
        }
        if status.is_none() {
            match child.try_wait() {
                Ok(value) => status = value,
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error).context("waiting for owned ORAS child");
                }
            }
        }
        for (name, rx) in [("stdout", &stdout), ("stderr", &stderr)] {
            if !outputs.contains_key(name) {
                match rx.try_recv() {
                    Ok(value) => {
                        outputs.insert(name, value);
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        bail!("ORAS diagnostic drain disconnected");
                    }
                }
            }
        }
        if let Some(status) = status.filter(|_| outputs.len() == 2) {
            let out = outputs.remove("stdout").expect("collected stdout")?;
            let err = outputs.remove("stderr").expect("collected stderr")?;
            ensure!(
                status.success(),
                "ORAS fetch failed with {status}; stdout: {}; stderr: {}",
                String::from_utf8_lossy(&out),
                String::from_utf8_lossy(&err)
            );
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn acquire(
    repository: &str,
    requested: &Digest,
    profile: Profile,
    parent: &Path,
    deadline: Instant,
) -> Result<Proof> {
    let directory = AcquisitionDirectory::new(parent)?;
    let path = directory.0.join("manifest");
    fetch(
        Command::new("oras")
            .args(["manifest", "fetch", "--output"])
            .arg(&path)
            .arg(format!("{repository}@{requested}")),
        deadline,
    )?;
    let original_manifest = bytes_with_limit(regular(&path)?, MANIFEST_LIMIT)?;
    let checked = manifest(&original_manifest, requested, profile)?;
    let mut blobs = Vec::new();
    for (index, (descriptor, limit)) in std::iter::once(&checked.manifest.config)
        .chain(&checked.manifest.layers)
        .zip(&checked.limits)
        .enumerate()
    {
        let bytes = if profile == Profile::Bundle && index == 0 {
            b"{}".to_vec()
        } else {
            let path = directory.0.join(format!("blob-{index}"));
            fetch(
                Command::new("oras")
                    .args(["blob", "fetch", "--output"])
                    .arg(&path)
                    .arg(format!("{repository}@{}", descriptor.digest)),
                deadline,
            )?;
            bytes_with_limit(regular(&path)?, *limit)?
        };
        check_blob(&bytes, descriptor)?;
        blobs.push(bytes);
    }
    let proof = Proof {
        original_manifest,
        blobs,
        payload_index: checked.payload_index,
    };
    check_payload(proof.payload(), profile)?;
    ensure!(
        Instant::now() < deadline,
        "OCI acquisition deadline exceeded during verification"
    );
    Ok(proof)
}

/// The verified original payload bytes for one pinned OCI reference.
///
/// This is the only cross-module entry point of the cache, and everything else in the module stays
/// private. What it proves is the *transport*: the requested manifest's original bytes and each
/// referenced descriptor's original bytes. It proves nothing about publisher authorization, and a
/// chart being in the cache establishes neither that Helm ran nor which cluster it affected.
pub fn payload(reference: &str, cache: &Path, profile: Profile) -> Result<Vec<u8>> {
    let (repository, requested) = reference
        .rsplit_once('@')
        .context("OCI source must be pinned as repository@sha256:digest")?;
    ensure!(
        !repository.is_empty() && !repository.starts_with('-') && !repository.contains('@'),
        "invalid pinned OCI repository"
    );
    let requested = Digest::new(requested).context("invalid requested OCI manifest digest")?;
    let parent = cache
        .join("oci-proof-v1")
        .join(profile.namespace())
        .join("sha256");
    let path = parent.join(format!(
        "{}.entry",
        requested.as_str().trim_start_matches("sha256:")
    ));
    let proof = if exists(&path)? {
        read_entry(&path, &requested, profile)?
    } else {
        let deadline = Instant::now() + DEADLINE;
        fs::create_dir_all(&parent).context("creating OCI proof namespace")?;
        let proof = acquire(repository, &requested, profile, &parent, deadline)?;
        publish(&proof, &path, &requested, profile, |_| {
            ensure!(
                Instant::now() < deadline,
                "OCI acquisition deadline exceeded before publication"
            );
            Ok(())
        })?
    };
    Ok(proof
        .blobs
        .into_iter()
        .nth(proof.payload_index)
        .expect("checked payload index"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn proof() -> (Proof, Digest) {
        let config = b"opaque config".to_vec();
        let chart = b"opaque chart".to_vec();
        let bytes=format!(r#"{{"schemaVersion":2,"mediaType":"{OCI}","config":{{"mediaType":"{HELM}","digest":"{}","size":{}}},"layers":[{{"mediaType":"{CHART}","digest":"{}","size":{}}}]}}"#,Digest::of_bytes(&config),config.len(),Digest::of_bytes(&chart),chart.len()).into_bytes();
        let requested = Digest::of_bytes(&bytes);
        (
            Proof {
                original_manifest: bytes,
                blobs: vec![config, chart],
                payload_index: 1,
            },
            requested,
        )
    }
    #[test]
    fn every_binary_frame_bound_is_checked_before_read_allocation() {
        for limit in [MANIFEST_LIMIT, SMALL_LIMIT, BUNDLE_LIMIT, CHART_LIMIT] {
            // Reader counts actual requested bytes; over-limit lengths must not read the body.
            let raw = (limit + 1).to_be_bytes().to_vec();
            let mut reader = std::io::Cursor::new(raw);
            assert!(framed_bytes(&mut reader, limit)
                .unwrap_err()
                .to_string()
                .contains("frame length exceeds"));
            assert_eq!(reader.position(), 8);
            let mut reader = std::io::Cursor::new(limit.to_be_bytes().to_vec());
            assert!(framed_bytes(&mut reader, limit)
                .unwrap_err()
                .to_string()
                .contains("truncated OCI proof bytes"));
            let mut raw = 8u64.to_be_bytes().to_vec();
            raw.extend(b"12345678");
            let mut reader = raw.as_slice();
            assert_eq!(framed_bytes(&mut reader, limit).unwrap(), b"12345678");
        }
    }
    #[test]
    fn descriptor_limit_equal_is_admitted_and_plus_one_is_refused() {
        for (kind, limit) in [
            (HELM, SMALL_LIMIT),
            (CHART, CHART_LIMIT),
            (PROVENANCE, SMALL_LIMIT),
            (BUNDLE_JSON, BUNDLE_LIMIT),
        ] {
            for excess in [false, true] {
                let (p, _) = proof();
                let mut value: serde_json::Value =
                    serde_json::from_slice(&p.original_manifest).unwrap();
                let profile;
                if kind == BUNDLE_JSON {
                    profile = Profile::Bundle;
                    value["artifactType"] = BUNDLE.into();
                    value["config"] = serde_json::json!({"mediaType":EMPTY,"digest":Digest::of_bytes(b"{}").as_str(),"size":2});
                    value["layers"][0]["mediaType"] = BUNDLE_JSON.into();
                    value["layers"][0]["size"] = (limit + u64::from(excess)).into();
                } else {
                    profile = Profile::Helm;
                    if kind == HELM {
                        value["config"]["size"] = (limit + u64::from(excess)).into();
                    } else if kind == PROVENANCE {
                        let mut d = value["layers"][0].clone();
                        d["mediaType"] = PROVENANCE.into();
                        d["size"] = (limit + u64::from(excess)).into();
                        value["layers"].as_array_mut().unwrap().push(d);
                    } else {
                        value["layers"][0]["size"] = (limit + u64::from(excess)).into();
                    }
                }
                let bytes = serde_json::to_vec(&value).unwrap();
                let result = manifest(&bytes, &Digest::of_bytes(&bytes), profile);
                assert_eq!(result.is_ok(), !excess, "{kind}, excess={excess}");
            }
        }
    }
    #[test]
    fn staged_write_failure_and_failed_hard_link_leave_no_final_entry() {
        struct BrokenWriter;
        impl Write for BrokenWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("injected incomplete write"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let directory = crate::TemporaryDirectory::create("ess-proof-io-test").unwrap();
        let entry = directory.path().join("entry");
        let (proof, requested) = proof();
        let error = publish(&proof, &entry, &requested, Profile::Helm, |_| {
            bail!("injected interruption after complete stage")
        })
        .err()
        .unwrap();
        assert!(error.to_string().contains("injected interruption"));
        assert!(!entry.exists());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
        let error = publish(&proof, &entry, &requested, Profile::Helm, |stage| {
            fs::remove_file(stage)?;
            Ok(())
        })
        .err()
        .unwrap();
        assert!(error.to_string().contains("atomic no-replacement"));
        assert!(!entry.exists());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
        assert!(proof.write(&mut BrokenWriter).is_err());
        assert!(!entry.exists());
    }
    #[test]
    fn complete_concurrent_winner_is_reused_and_corrupt_winner_is_preserved() {
        for corrupt in [false, true] {
            let directory = crate::TemporaryDirectory::create("ess-proof-winner-test").unwrap();
            let entry = directory.path().join("entry");
            let (proof, requested) = proof();
            let mut winner = Vec::new();
            proof.write(&mut winner).unwrap();
            if corrupt {
                winner[0] ^= 1;
            }
            let result = publish(&proof, &entry, &requested, Profile::Helm, |_| {
                fs::write(&entry, &winner)?;
                Ok(())
            });
            assert_eq!(result.is_ok(), !corrupt);
            assert_eq!(fs::read(&entry).unwrap(), winner);
            assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
        }
    }
    #[cfg(unix)]
    #[test]
    fn owned_open_handle_keeps_verified_bytes_after_shared_path_replacement() {
        let directory = crate::TemporaryDirectory::create("ess-proof-handle-test").unwrap();
        let entry = directory.path().join("entry");
        let (proof, requested) = proof();
        let mut bytes = Vec::new();
        proof.write(&mut bytes).unwrap();
        fs::write(&entry, &bytes).unwrap();
        let mut owned = regular(&entry).unwrap();
        let replacement = directory.path().join("replacement");
        fs::write(&replacement, b"changed").unwrap();
        fs::rename(&replacement, &entry).unwrap();
        assert_eq!(
            read_proof(&mut owned, &requested, Profile::Helm)
                .unwrap()
                .payload(),
            proof.payload()
        );
        assert_eq!(fs::read(&entry).unwrap(), b"changed");
    }
}
