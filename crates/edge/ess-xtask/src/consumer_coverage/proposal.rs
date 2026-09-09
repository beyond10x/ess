//! Finite Stage 1 review material. No output from this module is accepted eligibility.
use super::account::{Cell, Disposition, Status};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    id: String,
    entrypoints: Vec<String>,
    claim_boundary: String,
    #[serde(rename = "execution_profile")]
    execution: String,
    classification: ProfileClass,
    evidence_status: CandidateStatus,
    follow_up: Option<String>,
    owner: Option<String>,
}
#[derive(Deserialize, Serialize, PartialEq)]
enum ProfileClass {
    #[serde(rename = "model-consumer")]
    ModelConsumer,
    #[serde(rename = "foreign-context")]
    ForeignContext,
}
#[derive(Deserialize, Serialize)]
enum CandidateStatus {
    #[serde(rename = "UNACCEPTED_SOURCE_CANDIDATE")]
    Unaccepted,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CaseIdentity {
    pub package: String,
    pub target_kind: TestTarget,
    pub target_name: String,
    pub full_name: String,
    pub features: Vec<String>,
    pub target_profile: String,
    pub tool_requirements: Vec<String>,
    pub nested_runtime: String,
}
#[derive(Deserialize, Serialize)]
pub(super) enum TestTarget {
    #[serde(rename = "test")]
    Test,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReviewedCase {
    identity: CaseIdentity,
    source: String,
    case_ast_sha256: String,
    reviewed_assertion: String,
    attribution_limit: String,
    source_file_sha256: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Requirement {
    id: String,
    models: Vec<String>,
    consumers: Vec<String>,
    cases: Vec<String>,
    reason: String,
    #[serde(default)]
    behavior: ClaimedBehavior,
}
#[derive(Deserialize, Serialize, Default)]
#[serde(tag = "kind", deny_unknown_fields)]
enum ClaimedBehavior {
    #[default]
    Supported,
    Refused {
        refusal: String,
    },
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Reviewed {
    cases: BTreeMap<String, ReviewedCase>,
    requirements: Vec<Requirement>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Classification {
    class: EntryClass,
    reason: String,
    macro_ast_sha256: Option<String>,
}
#[derive(Deserialize, Serialize)]
enum EntryClass {
    BoundEntry,
    OwnedHelper,
    ForeignContext,
    DiagnosticSurface,
    FixtureRealization,
}

pub(super) fn build(
    models: &Value,
    inventory: &mut Value,
    source: &BTreeMap<String, String>,
    build_profile: &Value,
) -> Result<BTreeMap<String, Value>> {
    let profiles: Vec<Profile> = serde_json::from_str(include_str!("profiles.json"))?;
    let reviewed: Reviewed = serde_json::from_str(include_str!("reviewed-candidates.json"))?;
    let classifications: BTreeMap<String, Classification> =
        serde_json::from_str(include_str!("entry-classifications.json"))?;
    let mut entries = BTreeMap::new();
    let mut cases = BTreeMap::new();
    for package in inventory["packages"]
        .as_object()
        .context("packages")?
        .values()
    {
        for (id, row) in package["entries"].as_object().context("entries")? {
            if entries.insert(id.clone(), row.clone()).is_some() {
                bail!("duplicate workspace consumer identity {id}");
            }
        }
        for (id, row) in package["cases"].as_object().context("cases")? {
            if cases.insert(id.clone(), row.clone()).is_some() {
                bail!("duplicate workspace source case {id}");
            }
        }
    }
    let BoundProfiles {
        ids: profile_ids,
        output: profile_output,
    } = bind_profiles(&profiles, &entries, &classifications, build_profile)?;
    let case_output = review_cases(&reviewed, &cases, source, build_profile)?;
    let metadata = super::metadata::candidates(models, &json!(profile_ids))?;
    let ProposalCells {
        rows,
        groups,
        matrix,
    } = materialize(models, &profile_ids, &profiles, &reviewed, &metadata)?;
    for package in inventory["packages"]
        .as_object_mut()
        .context("packages")?
        .values_mut()
    {
        for (id, row) in package["entries"].as_object_mut().context("entries")? {
            row["review_classification"] = serde_json::to_value(&classifications[id])?;
        }
    }
    Ok(BTreeMap::from([
        ("consumer-profiles.json".into(), json!(profile_output)),
        ("reviewed-case-candidates.json".into(), json!(case_output)),
        (
            "mandatory-requirements.json".into(),
            json!(reviewed.requirements),
        ),
        ("pending-owner-groups.json".into(), json!(groups)),
        (
            "unaccepted-cells.json".into(),
            json!(super::account::Candidates {
                format: super::account::Format::V1,
                stage: super::account::CandidateStage::Candidate,
                cells: rows,
            }),
        ),
        ("checkpoint-summary.json".into(), matrix),
    ]))
}
struct BoundProfiles {
    ids: BTreeMap<String, Value>,
    output: BTreeMap<String, Value>,
}
fn bind_profiles(
    profiles: &[Profile],
    entries: &BTreeMap<String, Value>,
    classifications: &BTreeMap<String, Classification>,
    build_profile: &Value,
) -> Result<BoundProfiles> {
    classify(entries, classifications)?;
    let mut profile_ids = BTreeMap::new();
    let mut profile_output = BTreeMap::new();
    let mut bound = BTreeSet::new();
    let generator_names = ess_gen::generators()
        .iter()
        .map(|g| g.name().to_owned())
        .collect::<BTreeSet<_>>();
    let mut expected_generators = BTreeSet::new();
    for p in profiles {
        if p.entrypoints.is_empty() || p.claim_boundary.is_empty() {
            bail!("empty consumer boundary {}", p.id);
        }
        execution_profile(&p.execution)?;
        for entry in &p.entrypoints {
            if !entries.contains_key(entry) {
                bail!("missing bound entry {} {entry}", p.id);
            }
            bound.insert(entry);
        }
        if let Some(name) = p.id.strip_prefix("generator-") {
            expected_generators.insert(name.to_owned());
        }
        let fp = fingerprint(
            &serde_json::to_value(p)?,
            &p.entrypoints,
            entries,
            build_profile,
        )?;
        if profile_output
            .insert(p.id.clone(), json!({"definition":p,"profile_sha256":fp}))
            .is_some()
        {
            bail!("duplicate profile {}", p.id);
        }
        if p.classification == ProfileClass::ModelConsumer {
            profile_ids.insert(p.id.clone(), Value::String(fp));
        }
    }
    if expected_generators != generator_names {
        bail!("runtime published generator alternatives changed: actual {generator_names:?}; bound {expected_generators:?}");
    }
    for (id, c) in classifications {
        if matches!(c.class, EntryClass::BoundEntry) != bound.contains(id) {
            bail!("concrete bound-entry classification disagrees with profile entrypoints: {id}");
        }
    }
    let target_variants = entries
        .keys()
        .filter_map(|id| id.strip_prefix("ess_synth::lib(ess_synth)::enum::Target/variant/"))
        .collect::<BTreeSet<_>>();
    if target_variants != ["Rust", "Go", "Web", "Clap"].into_iter().collect() {
        bail!("synthesis Target alternatives require reviewed profiles: {target_variants:?}");
    }

    Ok(BoundProfiles {
        ids: profile_ids,
        output: profile_output,
    })
}
fn review_cases(
    reviewed: &Reviewed,
    cases: &BTreeMap<String, Value>,
    source: &BTreeMap<String, String>,
    build_profile: &Value,
) -> Result<BTreeMap<String, Value>> {
    let mut case_output = BTreeMap::new();
    for (id, case) in &reviewed.cases {
        let actual = cases
            .get(id)
            .with_context(|| format!("missing reviewed source case {id}"))?;
        let identity = &case.identity;
        if actual["case_ast_sha256"] != case.case_ast_sha256
            || actual["source"] != case.source
            || source.get(&case.source) != Some(&case.source_file_sha256)
            || actual["full_name"] != identity.full_name
            || actual["package"] != identity.package
            || actual["target_name"] != identity.target_name
            || actual["ignored"] != false
        {
            bail!("stale, renamed or ignored reviewed case {id}");
        }
        if !identity.features.is_empty()
            || identity.target_profile != "x86_64-unknown-linux-gnu/default"
            || identity.nested_runtime != "None claimed; direct Rust assertions only"
        {
            bail!("unreviewed case execution profile {id}");
        }
        case_output.insert(id.clone(),json!({"reviewed":case,"actual_source_candidate":actual,"complete_source_sha256":super::hash_json(&json!(source)),"compiled_provider_profile":super::hash_json(build_profile),"execution_status":"UNEXECUTED","attribution_status":"SOURCE_REVIEW_CANDIDATE; causal production mutation and exact execution remain owed"}));
    }

    Ok(case_output)
}
struct ProposalCells {
    rows: Vec<Cell>,
    groups: BTreeMap<String, Value>,
    matrix: Value,
}
// This produces accounting inputs, never approval of an unclassified inventory.
// The successful extraction path must separately satisfy every finite classification.
pub(super) fn accounting_inputs(inventory: &Value, build: &Value) -> Result<(Value, Value)> {
    let profiles: Vec<Profile> = serde_json::from_str(include_str!("profiles.json"))?;
    let reviewed: Reviewed = serde_json::from_str(include_str!("reviewed-candidates.json"))?;
    let mut entries = BTreeMap::new();
    for package in inventory["packages"]
        .as_object()
        .context("packages")?
        .values()
    {
        for (id, row) in package["entries"].as_object().context("entries")? {
            if entries.insert(id.clone(), row.clone()).is_some() {
                bail!("duplicate workspace consumer identity {id}");
            }
        }
    }
    let mut identities = BTreeMap::new();
    for profile in profiles {
        if profile.classification == ProfileClass::ModelConsumer {
            let shape = fingerprint(
                &serde_json::to_value(&profile)?,
                &profile.entrypoints,
                &entries,
                build,
            )?;
            if identities.insert(profile.id.clone(), shape).is_some() {
                bail!("duplicate declared consumer profile {}", profile.id);
            }
        }
    }
    let mut claims = Vec::new();
    for requirement in reviewed.requirements {
        for model in &requirement.models {
            for consumer in &requirement.consumers {
                let behavior = serde_json::to_value(&requirement.behavior)?;
                let mut claim = json!({"model":model,"consumer":consumer,"cases":requirement.cases,"kind":behavior["kind"],"reason":requirement.reason});
                if let Some(refusal) = behavior.get("refusal") {
                    claim["refusal"] = refusal.clone();
                }
                claims.push(claim);
            }
        }
    }
    Ok((json!(identities), json!(claims)))
}
fn materialize(
    models: &Value,
    profile_ids: &BTreeMap<String, Value>,
    profiles: &[Profile],
    reviewed: &Reviewed,
    metadata: &super::metadata::Candidates,
) -> Result<ProposalCells> {
    let models = models.as_object().context("model IDs")?;
    let mut required = BTreeMap::new();
    let mut requirement_ids = BTreeSet::new();
    for r in &reviewed.requirements {
        if !requirement_ids.insert(&r.id) || r.models.is_empty() || r.consumers.is_empty() {
            bail!("duplicate or empty mandatory requirement {}", r.id);
        }
        for id in &r.cases {
            if !reviewed.cases.contains_key(id) {
                bail!("unknown mandatory case {} {id}", r.id);
            }
        }
        for model in &r.models {
            if !models.contains_key(model) {
                bail!("stale mandatory model {} {model}", r.id);
            }
            for consumer in &r.consumers {
                if !profile_ids.contains_key(consumer) {
                    bail!("stale mandatory consumer {} {consumer}", r.id);
                }
                if required.insert((model, consumer), r).is_some() {
                    bail!("contradictory mandatory pair {model} {consumer}");
                }
            }
        }
    }
    let mut rows = Vec::new();
    let mut groups = BTreeMap::new();
    for profile in profiles
        .iter()
        .filter(|p| p.classification == ProfileClass::ModelConsumer)
    {
        let reason=format!("No exact behavior assertion is yet qualified for this model obligation under {}. Boundary: {} Root must select independent follow-up ownership after inspecting the finite cells; source candidates remain unexecuted.",profile.id,profile.claim_boundary);
        let mut pending = Vec::new();
        for (model, shape) in models {
            let metadata_row = metadata
                .rows()
                .iter()
                .find(|row| row.model == *model && row.consumer == profile.id);
            if metadata_row.is_some() && required.contains_key(&(model, &profile.id)) {
                bail!(
                    "contradictory behavioral/schema metadata pair {model} {}",
                    profile.id
                );
            }
            let disposition = if let Some(r) = required.get(&(model, &profile.id)) {
                if r.cases.is_empty() {
                    Disposition::MandatoryUnqualified {
                        requirement: r.id.clone(),
                        reason: r.reason.clone(),
                    }
                } else {
                    Disposition::CaseCandidate {
                        requirement: r.id.clone(),
                        cases: r.cases.clone(),
                        reason: r.reason.clone(),
                    }
                }
            } else if let Some(evidence) = metadata_row {
                Disposition::SchemaDocumentMetadataCandidate {
                    evidence: evidence.clone(),
                }
            } else {
                pending.push(model);
                Disposition::PendingOwner {
                    reason: reason.clone(),
                }
            };
            rows.push(Cell {
                model: model.clone(),
                shape: shape.as_str().context("model shape")?.into(),
                consumer: profile.id.clone(),
                profile: profile_ids[&profile.id]
                    .as_str()
                    .context("profile hash")?
                    .into(),
                status: Status::Unaccepted,
                disposition,
            });
        }
        groups.insert(profile.id.clone(),json!({"consumer":profile.id,"profile":profile_ids[&profile.id],"exact_pending_model_ids":pending,"reason":reason,"owner":null,"follow_up":null,"eligibility":"INVALID_PENDING_OWNER"}));
    }
    let matrix = super::account::check(&json!(models), &json!(profile_ids), &rows)?;

    Ok(ProposalCells {
        rows,
        groups,
        matrix,
    })
}
fn execution_profile(name: &str) -> Result<()> {
    if !matches!(
        name,
        "default-rust"
            | "frozen-rust"
            | "generated-go"
            | "rust-wasm-node-firefox"
            | "generated-rust-clap"
            | "required-firefox"
            | "generated-rust-client"
            | "typescript-6.0.3-node-22.23.1-v8-12.4.254.21-node.56-optional"
            | "external-bundle"
            | "external-schema"
            | "oci-bytes"
            | "maintenance-observation"
            | "infra-observation/2;infra-ir/2;infra-graph/2;infra-drift/2;infra-simulation/2"
            | "infra-v1"
    ) {
        bail!("unknown consumer execution profile {name}");
    }
    Ok(())
}

pub(super) fn fingerprint(
    boundary: &Value,
    entrypoints: &[String],
    entries: &BTreeMap<String, Value>,
    build: &Value,
) -> Result<String> {
    let mut declarations = BTreeMap::new();
    for id in entrypoints {
        let entry = entries
            .get(id)
            .with_context(|| format!("missing consumer entrypoint {id}"))?;
        let signature = entry["declaration_sha256"]
            .as_str()
            .with_context(|| format!("missing entrypoint declaration shape {id}"))?;
        declarations.insert(id,json!({"signature":signature,"conditions":entry["profile_conditions"],"member_conditions":entry["member_profile_conditions"]}));
    }
    let compiled = &build["compiled_build"];
    let environment = &compiled["environment"];
    let mut requirements = BTreeMap::new();
    for key in [
        "TARGET",
        "CARGO_CFG_TARGET_ARCH",
        "CARGO_CFG_TARGET_OS",
        "CARGO_CFG_TARGET_FEATURE",
        "CARGO_CFG_FEATURE",
        "CARGO_CFG_PANIC",
        "CARGO_ENCODED_RUSTFLAGS",
        "OPT_LEVEL",
        "DEBUG",
        "CARGO_CFG_DEBUG_ASSERTIONS",
    ] {
        requirements.insert(key, environment[key].clone());
    }
    Ok(super::hash_json(
        &json!({"boundary":boundary,"entrypoint_declarations":declarations,"semantic_build":requirements,"rustc_sha256":compiled["tools"]["RUSTC"]["sha256"],"cargo_sha256":compiled["tools"]["CARGO"]["sha256"]}),
    ))
}

fn classify(
    entries: &BTreeMap<String, Value>,
    classifications: &BTreeMap<String, Classification>,
) -> Result<()> {
    for id in entries.keys() {
        if !classifications.contains_key(id) {
            bail!("unclassified concrete consumer entry {id}; finite review required");
        }
    }
    for (id, c) in classifications {
        if !entries.contains_key(id) || c.reason.trim().is_empty() {
            bail!("stale or reasonless consumer classification {id}");
        }
        let nonmodel_macro = id.contains("::macro::")
            && !["ess_domain::", "ess_compiler::", "ess_primitives::"]
                .iter()
                .any(|prefix| id.starts_with(prefix));
        if nonmodel_macro {
            if c.macro_ast_sha256.as_deref() != entries[id]["source_item_sha256"].as_str() {
                bail!("consumer macro AST guard changed or absent: {id}; explicit finite reclassification required");
            }
        } else if c.macro_ast_sha256.is_some() {
            bail!("unexpected non-model macro guard for {id}");
        }
    }
    Ok(())
}
#[cfg(test)]
pub(super) fn classification_fixture(entries: &Value, classifications: &Value) -> Result<()> {
    classify(
        &serde_json::from_value(entries.clone())?,
        &serde_json::from_value(classifications.clone())?,
    )
}
