//! Executed bookkeeping for exactly six reviewed schema-document relationships.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

pub(super) const GUARD: &str = "cli-schema-document-metadata";
pub(super) const SOURCE: &str = "crates/edge/ess-xtask/src/consumer_coverage/metadata.rs";
const DECISION: &str = "docs/design/cli-schema-metadata-accounting.md";
const DIALECT: &str = "http://json-schema.org/draft-07/schema#";
const LIMIT: &str = "Schema representation bookkeeping only; no CLI behavior, schema evaluation, runtime conformance or proof about a future private call graph.";
const CONSUMERS: [&str; 3] = [
    "cli-binding-resolution",
    "cli-binding-rust-emission",
    "cli-binding-process-execution",
];
const COMMON: [&str; 4] = [
    "ess_domain::lib(ess_domain)::spec::impl<RawSpecFile;>::parse",
    "ess_domain::lib(ess_domain)::spec::impl<Specification;>::assemble",
    "ess_compiler::lib(ess_compiler)::resolve::fn::compile",
    "ess_cli_contract::lib(ess_cli_contract)::resolve::fn::compile",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Relationship {
    RootDialectIdentifier,
    RootDefinitionsContainer,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub(super) struct Row {
    pub model: String,
    pub shape: String,
    pub consumer: String,
    pub profile: String,
    pub relationship: Relationship,
    pub guard: String,
    pub guard_source_sha256: String,
    pub reason: String,
    pub decision: String,
}
impl Row {
    pub(super) fn matches_cell(
        &self,
        model: &str,
        shape: &str,
        consumer: &str,
        profile: &str,
    ) -> Result<()> {
        if (
            self.model.as_str(),
            self.shape.as_str(),
            self.consumer.as_str(),
            self.profile.as_str(),
        ) != (model, shape, consumer, profile)
        {
            bail!("metadata evidence differs from its cell identity");
        }
        Ok(())
    }
}
#[derive(Deserialize)]
enum ManifestFormat {
    #[serde(rename = "ess-consumer-schema-metadata/1")]
    V1,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: ManifestFormat,
    rows: Vec<Row>,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Candidates {
    rows: BTreeSet<Row>,
    digest: String,
}
impl Candidates {
    pub(super) fn rows(&self) -> &BTreeSet<Row> {
        &self.rows
    }
    pub(super) fn digest(&self) -> &str {
        &self.digest
    }
    fn verify_shapes(&self, fresh_wire: &Value) -> Result<()> {
        for row in self.rows() {
            if fresh_wire["obligations"][&row.model] != row.shape {
                bail!(
                    "metadata row shape differs from the fresh provider: {}",
                    row.model
                );
            }
        }
        Ok(())
    }
}
fn guard_digest() -> String {
    super::hash_bytes(include_bytes!("metadata.rs"))
}
pub(super) fn candidates(models: &Value, profiles: &Value) -> Result<Candidates> {
    validate(
        include_bytes!("reviewed-schema-metadata.json"),
        models,
        profiles,
        &serde_json::from_str(include_str!("profiles.json"))?,
        &guard_digest(),
    )
}
fn expected() -> BTreeSet<(String, String, Relationship)> {
    [
        (
            "wire:RawSpecFile#/$schema",
            Relationship::RootDialectIdentifier,
        ),
        (
            "wire:RawSpecFile#/definitions",
            Relationship::RootDefinitionsContainer,
        ),
    ]
    .into_iter()
    .flat_map(|(model, role)| {
        CONSUMERS
            .into_iter()
            .map(move |consumer| (model.to_owned(), consumer.to_owned(), role.clone()))
    })
    .collect()
}
fn validate(
    bytes: &[u8],
    models: &Value,
    profiles: &Value,
    definitions: &Value,
    digest: &str,
) -> Result<Candidates> {
    let manifest: Manifest = serde_json::from_slice(bytes)?;
    let ManifestFormat::V1 = manifest.format;
    let mut identities = BTreeSet::new();
    let mut rows = BTreeSet::new();
    for row in manifest.rows {
        let identity = (
            row.model.clone(),
            row.consumer.clone(),
            row.relationship.clone(),
        );
        if !identities.insert(identity) {
            bail!("duplicate schema metadata relationship");
        }
        if models.get(&row.model) != Some(&json!(row.shape))
            || profiles.get(&row.consumer) != Some(&json!(row.profile))
            || [&row.shape, &row.profile, &row.guard_source_sha256]
                .iter()
                .any(|s| s.len() != 64 || !s.bytes().all(|c| c.is_ascii_hexdigit()))
            || row.guard != GUARD
            || row.guard_source_sha256 != digest
            || row.decision != DECISION
            || row.reason.trim().is_empty()
        {
            bail!(
                "stale or unreviewed schema metadata row {} {}",
                row.model,
                row.consumer
            );
        }
        rows.insert(row);
    }
    if identities != expected() {
        bail!("schema metadata must contain exactly the six reviewed literal relationships");
    }
    validate_entries(definitions)?;
    Ok(Candidates {
        rows,
        digest: super::hash_bytes(bytes),
    })
}
fn validate_entries(definitions: &Value) -> Result<()> {
    let definitions = definitions.as_array().context("profile definitions")?;
    for consumer in CONSUMERS {
        let matched = definitions
            .iter()
            .filter(|p| p["id"] == consumer)
            .collect::<Vec<_>>();
        if matched.len() != 1 {
            bail!("missing or duplicate authored CLI metadata boundary {consumer}");
        }
        let definition = matched[0];
        let mut entries = COMMON.to_vec();
        match consumer {
            "cli-binding-rust-emission" => {
                entries.push("ess_cli_project::lib(ess_cli_project)::fn::project");
            }
            "cli-binding-process-execution" => {
                entries.push("ess_cli_project::lib(ess_cli_project)::runtime::fn::run");
            }
            _ => {}
        }
        if definition["entrypoints"] != json!(entries)
            || definition["classification"] != "model-consumer"
            || definition["execution_profile"] != "default-rust"
        {
            bail!("changed authored CLI metadata entry boundary {consumer}");
        }
    }
    Ok(())
}

/// A current check invocation, never reconstructed from a receipt.
pub(super) struct Authority {
    root: PathBuf,
    profile: Value,
}
impl Authority {
    pub(super) fn capture(root: &Path, profile: &Value) -> Result<Self> {
        let authority = Self {
            root: root.to_owned(),
            profile: profile.clone(),
        };
        authority.verify()?;
        Ok(authority)
    }
    pub(super) fn verify(&self) -> Result<()> {
        let files = super::source_files(&self.root)?;
        let compiled_source: Value = serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/consumer-source.json"
        )))?;
        let compiled_build: Value = serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/consumer-build.json"
        )))?;
        if json!(files) != self.profile["source"]
            || self.profile["source"] != self.profile["compiled_provider_source"]
            || self.profile["compiled_provider_source"] != compiled_source
            || self.profile["compiled_build"] != compiled_build
            || self.profile["source"][SOURCE] != guard_digest()
            || self.profile["provider_executable_sha256"]
                != super::hash_bytes(&fs::read(std::env::current_exe()?)?)
        {
            bail!("schema metadata source/provider authority changed");
        }
        let invocation = super::invocation(&self.root, &self.profile["compiled_build"])?;
        if invocation != self.profile["current_invocation"] {
            bail!("schema metadata invocation authority changed");
        }
        super::validate_build(
            &self.profile["compiled_build"],
            |key| std::env::var(key).ok(),
            |path| Ok(fs::read(path)?),
        )
    }
}
/// Private fields and no Deserialize: a saved success cannot mint this proof.
pub(super) struct Verified<'a> {
    authority: &'a Authority,
    rows: BTreeSet<Row>,
    manifest: String,
    receipt: Value,
}
impl Verified<'_> {
    pub(super) fn receipt(&self) -> &Value {
        &self.receipt
    }
    pub(super) fn check(
        &self,
        authority: &Authority,
        required: &BTreeSet<Row>,
        manifest: &str,
    ) -> Result<()> {
        if !std::ptr::eq(authority, self.authority)
            || required != &self.rows
            || manifest != self.manifest
        {
            bail!("metadata planned/proved pairs, guard identity or current-run authority differ");
        }
        Ok(())
    }
}
fn observe_schema(retained: &Value, inventoried: &Value) -> Result<(Value, Value)> {
    let fresh = serde_json::to_value(schemars::schema_for!(ess_domain::spec::RawSpecFile))?;
    let actual = verify_schema(&fresh, retained, inventoried)?;
    Ok((fresh, actual))
}
fn verify_schema(fresh: &Value, retained: &Value, inventoried: &Value) -> Result<Value> {
    if fresh != retained {
        bail!("retained schema differs from the fresh compiled RawSpecFile provider");
    }
    let properties = fresh["properties"]
        .as_object()
        .context("authored root properties")?;
    if fresh["$schema"] != DIALECT
        || !fresh["definitions"].is_object()
        || properties.contains_key("$schema")
        || properties.contains_key("definitions")
    {
        bail!("unsupported schema metadata dialect, definitions or authored-property relationship");
    }
    let actual = super::wire::extract(fresh)?;
    if &actual != inventoried {
        bail!("fresh schema wire obligations/reference inventory differs");
    }
    Ok(actual)
}
pub(super) fn execute<'a>(
    authority: &'a Authority,
    retained: &Value,
    inventoried: &Value,
    models: &Value,
    profiles: &Value,
) -> Result<Verified<'a>> {
    authority.verify()?;
    let (fresh, fresh_wire) = observe_schema(retained, inventoried)?;
    let candidates = candidates(models, profiles)?;
    candidates.verify_shapes(&fresh_wire)?;
    let receipt = json!({
        "guard":GUARD,"executed_guards":1,"metadata_cells":candidates.rows.len(),
        "required_relationships":candidates.rows,"manifest_sha256":candidates.digest,
        "guard_source_sha256":guard_digest(),"source_sha256":super::hash_json(&authority.profile["source"]),
        "provider_sha256":authority.profile["provider_executable_sha256"],
        "compiled_build":authority.profile["compiled_build"],
        "invocation":authority.profile["current_invocation"],
        "authority_sha256":super::hash_json(&authority.profile),
        "fresh_schema_sha256":super::hash_json(&fresh),
        "wire_inventory_sha256":super::hash_json(inventoried),
        "claim_limit":LIMIT,
    });
    authority.verify()?;
    Ok(Verified {
        authority,
        rows: candidates.rows,
        manifest: candidates.digest,
        receipt,
    })
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod tests;
