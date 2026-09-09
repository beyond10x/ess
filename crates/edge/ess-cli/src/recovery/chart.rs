//! OCI proof consumption, bounded archive decoding and the exact five-file projection.
//!
//! The first recovery profile admits *verified ESS-generated charts*, not arbitrary Helm charts.
//! What that means concretely is this module: the chart payload comes out of the existing
//! original-byte OCI proof, is decoded through a bounded reader that admits only regular members
//! under one root, and every member must equal the corresponding file that
//! [`ess_deployment::project_helm`] produces from the caller-pinned runtime. A chart that is
//! merely a valid Helm chart is refused.
//!
//! A chart in cache proves neither that Helm ran nor which cluster it affected. Nothing here is
//! target progress.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};

use ess_deployment::{Identifier, RuntimeIr};
use serde::Deserialize as _;

use super::journal::{read_protected, under};
use super::model::{
    Admitted, ChartSource, Digest, ObjectAddress, ObjectKind, Refusal, RefusalCode,
    ReleaseProjection, Text,
};
use super::{Barrier, Host, Trust};

/// The largest admitted decompressed chart archive.
pub const ARCHIVE_LIMIT: u64 = 16 * 1024 * 1024;
/// The largest admitted rendered document stream.
pub const RENDER_LIMIT: usize = 8 * 1024 * 1024;
/// The exact five files `project_helm` emits.
pub const PROJECTION_FILES: &[&str] = &[
    "Chart.yaml",
    "templates/services.yaml",
    "templates/workloads.yaml",
    "values.schema.json",
    "values.yaml",
];
/// The directory the caller-pinned runtime documents are published under.
pub const RUNTIME_DIRECTORY: &str = "runtime";

fn preparation(detail: impl Into<String>) -> Refusal {
    Refusal::new(RefusalCode::PreparationFailed, detail)
}

/// Reads the caller-pinned `ess-runtime-ir/1` document named by a chart source.
///
/// The digest is the file's name and is checked against its bytes; the reader is the existing
/// strict `RuntimeIr` reader with its intrinsic validation, not a second parser.
pub fn read_runtime(host: &dyn Host, root: &Path, chart: &ChartSource) -> Admitted<RuntimeIr> {
    let path = root.join(RUNTIME_DIRECTORY).join(format!(
        "{}.json",
        chart.runtime_digest.as_str().replace(':', "-")
    ));
    let text = under(
        RefusalCode::PreparationFailed,
        read_protected(host, &path, Trust::Administrative),
    )?;
    let digest = Digest::of_bytes(text.as_bytes());
    if digest != chart.runtime_digest {
        return Err(preparation(format!(
            "{} does not hash to the pinned runtime digest",
            path.display()
        )));
    }
    let runtime = RuntimeIr::from_json(&text)
        .map_err(|error| preparation(format!("the pinned runtime is not admissible: {error}")))?;
    runtime.validate().map_err(|diagnostics| {
        preparation(format!("the pinned runtime is invalid: {diagnostics:?}"))
    })?;
    if runtime.to_canonical_json() != text {
        return Err(preparation(
            "the pinned runtime document is not its own canonical spelling",
        ));
    }
    Ok(runtime)
}

/// The five projection files a chart source must reproduce exactly.
pub fn projection_files(
    runtime: &RuntimeIr,
    chart: &ChartSource,
) -> Admitted<BTreeMap<String, String>> {
    let name = Identifier::new(chart.chart_name.as_str())
        .map_err(|error| preparation(format!("the chart name is not an identifier: {error}")))?;
    let version = chart
        .chart_version
        .as_str()
        .parse::<semver::Version>()
        .map_err(|error| preparation(format!("the chart version is not semantic: {error}")))?;
    let projection = ess_deployment::project_helm(runtime, &name, &version);
    let files = projection.files().clone();
    let names: Vec<&str> = files.keys().map(String::as_str).collect();
    if names != PROJECTION_FILES {
        return Err(preparation(format!(
            "the projector emitted {names:?} and this profile admits exactly {PROJECTION_FILES:?}"
        )));
    }
    Ok(files)
}

/// One admitted member of a chart archive.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Member {
    path: String,
    contents: Vec<u8>,
}

/// Decodes a gzip/TAR chart archive under one root, bounded, admitting only regular members.
///
/// Every refusal here is a named dimension of the matrix: links, traversal, duplicate members,
/// extra members, missing members, more than one chart root and a nonregular entry. The bound is
/// applied to the *decompressed* stream, before allocation, so a small archive cannot expand into
/// an unbounded read.
fn decode_archive(bytes: &[u8]) -> Admitted<(String, Vec<Member>)> {
    let decoder = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder.take(ARCHIVE_LIMIT + 1));
    let mut members: Vec<Member> = Vec::new();
    let mut total: u64 = 0;
    let mut roots: BTreeSet<String> = BTreeSet::new();
    let entries = archive
        .entries()
        .map_err(|error| preparation(format!("the chart archive is unreadable: {error}")))?;
    for entry in entries {
        let mut entry = entry
            .map_err(|error| preparation(format!("the chart archive is unreadable: {error}")))?;
        let kind = entry.header().entry_type();
        let path = entry
            .path()
            .map_err(|error| {
                preparation(format!("a chart member has no admissible path: {error}"))
            })?
            .to_string_lossy()
            .into_owned();
        if !kind.is_file() && !kind.is_dir() {
            return Err(preparation(format!(
                "chart member {path:?} is a {kind:?}; this profile admits regular members only"
            )));
        }
        if path.starts_with('/')
            || path
                .split('/')
                .any(|component| component == ".." || component == ".")
        {
            return Err(preparation(format!(
                "chart member {path:?} traverses or is absolute"
            )));
        }
        let Some((root, relative)) = path.split_once('/') else {
            if kind.is_dir() {
                roots.insert(path.trim_end_matches('/').to_owned());
                continue;
            }
            return Err(preparation(format!(
                "chart member {path:?} is not under a chart root"
            )));
        };
        roots.insert(root.to_owned());
        if kind.is_dir() {
            continue;
        }
        let relative = relative.to_owned();
        if relative.is_empty() {
            return Err(preparation(format!("chart member {path:?} names no file")));
        }
        if members.iter().any(|member| member.path == relative) {
            return Err(preparation(format!(
                "chart member {relative:?} appears more than once"
            )));
        }
        let size = entry
            .header()
            .size()
            .map_err(|error| preparation(format!("a chart member declares no size: {error}")))?;
        total = total.saturating_add(size);
        if total > ARCHIVE_LIMIT {
            return Err(preparation(
                "the chart archive is past the admitted decompressed bound",
            ));
        }
        let mut contents = Vec::new();
        entry.read_to_end(&mut contents).map_err(|error| {
            preparation(format!("chart member {relative:?} is unreadable: {error}"))
        })?;
        members.push(Member {
            path: relative,
            contents,
        });
    }
    let mut roots = roots.into_iter();
    let root = roots
        .next()
        .ok_or_else(|| preparation("the chart archive has no root directory"))?;
    if roots.next().is_some() {
        return Err(preparation(
            "the chart archive has more than one root directory",
        ));
    }
    members.sort_by(|left, right| left.path.cmp(&right.path));
    Ok((root, members))
}

/// The chart, its exact private bytes and the values prepared for one operation.
#[derive(Debug, Clone)]
pub struct PreparedChart {
    /// The verified private chart snapshot.
    pub chart_path: PathBuf,
    /// The private values document.
    pub values_path: PathBuf,
    /// The exact values bytes, for the recheck immediately before launch.
    pub values: String,
    /// The digest of the exact chart payload bytes.
    pub payload_digest: Digest,
    /// The admitted rendered objects, kept for the authored-field comparison after apply.
    pub rendered: Vec<RenderedObject>,
}

/// Admits a verified chart payload against the pinned runtime's exact projection.
///
/// This is step 3 and step 4 of C07 together: every archive member must equal the corresponding
/// projection file, and a template action that arrived through runtime *string data* rather than
/// through the fixed projector is refused. The second check is what stops an injected template
/// from riding in on an otherwise byte-equal file — the projector's own output is the only
/// admitted source of a delimiter.
pub fn admit_payload(
    payload: &[u8],
    runtime: &RuntimeIr,
    chart: &ChartSource,
) -> Admitted<BTreeMap<String, String>> {
    let expected = projection_files(runtime, chart)?;
    let (root, members) = decode_archive(payload)?;
    if root != chart.chart_name.as_str() {
        return Err(preparation(format!(
            "the chart archive's root is {root:?} and the pinned chart is {:?}",
            chart.chart_name
        )));
    }
    let present: Vec<&str> = members.iter().map(|member| member.path.as_str()).collect();
    let wanted: Vec<&str> = expected.keys().map(String::as_str).collect();
    if present != wanted {
        return Err(preparation(format!(
            "the chart archive holds {present:?} and the pinned projection is {wanted:?}"
        )));
    }
    for member in &members {
        let projected = &expected[&member.path];
        if member.contents != projected.as_bytes() {
            return Err(preparation(format!(
                "chart member {:?} does not equal its projected bytes",
                member.path
            )));
        }
    }
    check_injected_actions(runtime, &expected)?;
    Ok(expected)
}

/// Refuses a template delimiter that reached the projection through runtime string data.
fn check_injected_actions(runtime: &RuntimeIr, files: &BTreeMap<String, String>) -> Admitted<()> {
    let authored = runtime.to_canonical_json();
    for delimiter in ["{{", "}}"] {
        if authored.contains(delimiter) {
            return Err(preparation(format!(
                "the pinned runtime carries the template delimiter {delimiter:?} in its own data; \
                 admitted template actions come from the fixed projector only"
            )));
        }
    }
    for (name, contents) in files {
        if name.starts_with("templates/") {
            continue;
        }
        if contents.contains("{{") {
            return Err(preparation(format!(
                "the projected file {name:?} carries a template action outside templates/"
            )));
        }
    }
    Ok(())
}

/// Materializes the private chart snapshot and values for one operation.
pub fn prepare(
    host: &dyn Host,
    directory: &Path,
    payload: &[u8],
    values: &str,
    label: &str,
) -> Admitted<PreparedChart> {
    let chart_path = directory.join("chart.tgz");
    let values_path = directory.join("values.yaml");
    host.barrier(Barrier::ChartWrite, label)?;
    std::fs::write(&chart_path, payload).map_err(|error| {
        preparation(format!(
            "the private chart snapshot could not be written: {error}"
        ))
    })?;
    host.barrier(Barrier::ValuesWrite, label)?;
    std::fs::write(&values_path, values).map_err(|error| {
        preparation(format!("the private values could not be written: {error}"))
    })?;
    Ok(PreparedChart {
        chart_path,
        values_path,
        values: values.to_owned(),
        payload_digest: Digest::of_bytes(payload),
        rendered: Vec::new(),
    })
}

/// One rendered object: its address and the digest of its complete authored content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedObject {
    /// The address.
    pub address: ObjectAddress,
    /// The complete authored rendered content, canonically encoded.
    pub content: serde_json::Value,
}

/// Admits one rendered document and returns the address it claims.
///
/// Split out of [`admit_rendered`] because it is the whole of C07 step 5 for a single document —
/// kind, apiVersion, hooks, keep policy, `generateName`, namespace and name — and the loop that
/// applies it to a stream, and refuses a duplicate address across the stream, is a different
/// question.
fn admit_document(json: &serde_json::Value, namespace: &Text) -> Admitted<ObjectAddress> {
    let object = json
        .as_object()
        .ok_or_else(|| preparation("a rendered document is not an object"))?;
    let kind = object
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| preparation("a rendered document names no kind"))?;
    let declared = ObjectKind::parse(kind).ok_or_else(|| {
        Refusal::new(
            RefusalCode::UnsupportedProfile,
            format!("this profile admits no rendered {kind}"),
        )
    })?;
    let api_version = object
        .get("apiVersion")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| preparation("a rendered document names no apiVersion"))?;
    if api_version != declared.api_version() {
        return Err(preparation(format!(
            "a rendered {kind} names apiVersion {api_version:?}"
        )));
    }
    let metadata = object
        .get("metadata")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| preparation("a rendered document carries no metadata"))?;
    if metadata.contains_key("generateName") {
        return Err(preparation(
            "a rendered object uses generateName, whose address is not knowable before apply",
        ));
    }
    if let Some(annotations) = metadata
        .get("annotations")
        .and_then(serde_json::Value::as_object)
    {
        for key in annotations.keys() {
            if key.starts_with("helm.sh/hook") {
                return Err(preparation(format!(
                    "a rendered object declares the hook annotation {key:?}"
                )));
            }
            if key == "helm.sh/resource-policy" {
                return Err(preparation(
                    "a rendered object declares a Helm keep policy, which prevents removal",
                ));
            }
        }
    }
    if let Some(rendered_namespace) = metadata
        .get("namespace")
        .and_then(serde_json::Value::as_str)
    {
        if rendered_namespace != namespace.as_str() {
            return Err(Refusal::new(
                RefusalCode::UnsupportedProfile,
                format!(
                    "a rendered object names namespace {rendered_namespace:?} and this permit \
                     is bound to {namespace}"
                ),
            ));
        }
    }
    let name = metadata
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| preparation("a rendered object has no name"))?;
    Ok(ObjectAddress {
        kind: declared,
        name: Text::new(name)?,
    })
}

/// Parses a complete rendered document stream and refuses everything outside the profile.
///
/// The per-document rejections are [`admit_document`]'s; what this function adds is the stream:
/// a duplicate address across documents, and an empty stream. An empty one is refused too — a
/// render that produced nothing is not a render that produced the inventory.
pub fn admit_rendered(text: &str, namespace: &Text) -> Admitted<Vec<RenderedObject>> {
    if text.len() > RENDER_LIMIT {
        return Err(preparation(
            "the rendered stream is past the admitted bound",
        ));
    }
    let mut rendered: Vec<RenderedObject> = Vec::new();
    for document in serde_yaml::Deserializer::from_str(text) {
        let value = serde_yaml::Value::deserialize(document)
            .map_err(|error| preparation(format!("a rendered document is not YAML: {error}")))?;
        if value.is_null() {
            continue;
        }
        let json: serde_json::Value =
            serde_yaml::from_value(serde_yaml::to_value(&value).map_err(|error| {
                preparation(format!("a rendered document is not encodable: {error}"))
            })?)
            .map_err(|error| {
                preparation(format!("a rendered document is not encodable: {error}"))
            })?;
        let address = admit_document(&json, namespace)?;
        if rendered.iter().any(|object| object.address == address) {
            return Err(preparation(format!(
                "the rendered stream carries {} {} more than once",
                address.kind, address.name
            )));
        }
        rendered.push(RenderedObject {
            address,
            content: json,
        });
    }
    if rendered.is_empty() {
        return Err(preparation("the rendered stream carries no object"));
    }
    rendered.sort_by(|left, right| left.address.cmp(&right.address));
    Ok(rendered)
}

/// Requires a rendered stream to equal exactly the projection being admitted.
///
/// Baseline is compared against baseline and desired against desired. A baseline-only retirement
/// has no desired projection to render, and that absence is permitted rather than a gap.
pub fn admit_inventory(
    rendered: &[RenderedObject],
    projection: &ReleaseProjection,
) -> Admitted<Vec<ObjectAddress>> {
    let present: Vec<ObjectAddress> = rendered
        .iter()
        .map(|object| object.address.clone())
        .collect();
    let expected = projection.addresses();
    if present != expected {
        return Err(Refusal::new(
            RefusalCode::BaselineMismatch,
            format!(
                "the rendered inventory has {} objects and the admitted projection has {}",
                present.len(),
                expected.len()
            ),
        ));
    }
    Ok(present)
}
