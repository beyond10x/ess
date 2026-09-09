//! Closed finite admission consumes the root-owned initial eligibility unchanged.
use super::{account, metadata};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    format: String,
    status: String,
    eligible_pairs: usize,
    mandatory_pairs_excluded: usize,
    stage1_source_commit: String,
    stage1_binding_sha256: String,
    source_checkpoint_sha256: String,
    extraction_profile: String,
    models: BTreeMap<String, String>,
    groups: Vec<Group>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    consumer: String,
    profile: String,
    exact_model_ids: Vec<String>,
    owner: String,
    follow_up: String,
    reason: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    model: String,
    consumer: String,
    cases: Vec<String>,
    kind: Behavior,
    reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    refusal: Option<String>,
}
#[derive(Deserialize, Serialize)]
enum Behavior {
    Supported,
    Refused,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlannedCell {
    model: String,
    shape: String,
    consumer: String,
    profile: String,
    disposition: Disposition,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Disposition {
    Supported {
        cases: Vec<String>,
        reason: String,
    },
    Refused {
        cases: Vec<String>,
        reason: String,
        refusal: String,
    },
    BaselineUnknown {
        owner: String,
        follow_up: String,
        reason: String,
    },
    SchemaDocumentMetadata {
        evidence: metadata::Row,
    },
}

#[derive(Serialize, Deserialize)]
enum Pending {
    #[serde(rename = "PENDING_CASE_AND_GUARD_EXECUTION")]
    Execution,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionPlan {
    format: account::Format,
    stage: account::ExecutionStage,
    execution_status: Pending,
    discovered_models: usize,
    bound_profiles: usize,
    cells: Vec<PlannedCell>,
    required_cases: BTreeSet<String>,
    required_metadata_guards: BTreeSet<metadata::Row>,
    metadata_manifest_sha256: Option<String>,
    pending_metadata_candidates: usize,
    #[serde(rename = "Supported")]
    supported: usize,
    #[serde(rename = "Refused")]
    refused: usize,
    #[serde(rename = "BaselineUnknown")]
    baseline_unknown: usize,
    #[serde(rename = "SchemaDocumentMetadata")]
    schema_document_metadata: usize,
}

#[cfg(test)]
pub(super) fn plan(
    models: &Value,
    profiles: &Value,
    baseline: &Value,
    claims: &Value,
) -> Result<Value> {
    plan_with_metadata(models, profiles, baseline, claims, None)
}
pub(super) fn plan_with_metadata(
    models: &Value,
    profiles: &Value,
    baseline: &Value,
    claims: &Value,
    metadata: Option<&metadata::Candidates>,
) -> Result<Value> {
    let models: BTreeMap<String, String> = serde_json::from_value(models.clone())?;
    let profiles: BTreeMap<String, String> = serde_json::from_value(profiles.clone())?;
    let baseline: Baseline = serde_json::from_value(baseline.clone())?;
    let claims: Vec<Claim> = serde_json::from_value(claims.clone())?;
    validate_header(&baseline)?;
    let mut cells = BTreeMap::new();
    let mut problems = Vec::new();
    let unknowns = baseline_cells(&baseline, &mut cells, &mut problems)?;
    let mut required_cases = BTreeSet::new();
    for claim in claims {
        let disposition = behavior(&claim)?;
        required_cases.extend(claim.cases);
        let shape = models
            .get(&claim.model)
            .with_context(|| format!("unknown claimed model {}", claim.model))?;
        let profile = profiles
            .get(&claim.consumer)
            .with_context(|| format!("unknown claimed consumer {}", claim.consumer))?;
        insert(
            &mut cells,
            PlannedCell {
                model: claim.model,
                shape: shape.clone(),
                consumer: claim.consumer,
                profile: profile.clone(),
                disposition,
            },
            &mut problems,
        );
    }
    let mut required_metadata_guards = BTreeSet::new();
    if let Some(metadata) = metadata {
        for row in metadata.rows() {
            required_metadata_guards.insert(row.clone());
            insert(
                &mut cells,
                PlannedCell {
                    model: row.model.clone(),
                    shape: row.shape.clone(),
                    consumer: row.consumer.clone(),
                    profile: row.profile.clone(),
                    disposition: Disposition::SchemaDocumentMetadata {
                        evidence: row.clone(),
                    },
                },
                &mut problems,
            );
        }
    }
    for cell in cells.values() {
        if models.get(&cell.model) != Some(&cell.shape)
            || profiles.get(&cell.consumer) != Some(&cell.profile)
        {
            problems.push(format!(
                "stale shape/profile or removed cell {} {}",
                cell.model, cell.consumer
            ));
        }
    }
    for model in models.keys() {
        for consumer in profiles.keys() {
            if !cells.contains_key(&(model.clone(), consumer.clone())) {
                problems.push(format!("unaccounted {model} {consumer}"));
            }
        }
    }
    if !problems.is_empty() {
        let current_unknowns = cells
            .values()
            .filter(|cell| {
                matches!(cell.disposition, Disposition::BaselineUnknown { .. })
                    && models.get(&cell.model) == Some(&cell.shape)
                    && profiles.get(&cell.consumer) == Some(&cell.profile)
            })
            .count();
        bail!(
            "{}",
            json!({"format":account::FORMAT,"stage":"execution-plan","accounting_complete":false,"discovered_models":models.len(),"bound_profiles":profiles.len(),"Supported":0,"Refused":0,"BaselineUnknown":current_unknowns,"SchemaDocumentMetadata":0,"pending_metadata_candidates":required_metadata_guards.len(),"executed_cases":0,"executed_metadata_guards":0,"unaccounted_or_stale":problems})
        );
    }
    Ok(serde_json::to_value(ExecutionPlan {
        format: account::Format::V1,
        stage: account::ExecutionStage::ExecutionPlan,
        execution_status: Pending::Execution,
        discovered_models: models.len(),
        bound_profiles: profiles.len(),
        cells: cells.into_values().collect(),
        required_cases,
        pending_metadata_candidates: required_metadata_guards.len(),
        required_metadata_guards,
        metadata_manifest_sha256: metadata.map(|m| m.digest().to_owned()),
        supported: 0,
        refused: 0,
        baseline_unknown: unknowns,
        schema_document_metadata: 0,
    })?)
}
fn baseline_cells(
    baseline: &Baseline,
    cells: &mut BTreeMap<(String, String), PlannedCell>,
    problems: &mut Vec<String>,
) -> Result<usize> {
    let mut unknowns = 0;
    for group in &baseline.groups {
        if [&group.owner, &group.follow_up, &group.reason]
            .iter()
            .any(|s| s.trim().is_empty())
            || group.follow_up == "story:review-consumer-coverage"
            || group.exact_model_ids.is_empty()
        {
            bail!(
                "baseline unknown lacks finite independent ownership: {}",
                group.consumer
            );
        }
        for model in &group.exact_model_ids {
            let shape = baseline
                .models
                .get(model)
                .with_context(|| format!("baseline dictionary lacks {model}"))?;
            insert(
                cells,
                PlannedCell {
                    model: model.clone(),
                    shape: shape.clone(),
                    consumer: group.consumer.clone(),
                    profile: group.profile.clone(),
                    disposition: Disposition::BaselineUnknown {
                        owner: group.owner.clone(),
                        follow_up: group.follow_up.clone(),
                        reason: group.reason.clone(),
                    },
                },
                problems,
            );
            unknowns += 1;
        }
    }
    if unknowns != baseline.eligible_pairs {
        problems.push("accepted finite pair count disagrees with manifest".into());
    }
    Ok(unknowns)
}
fn behavior(claim: &Claim) -> Result<Disposition> {
    if claim.reason.trim().is_empty()
        || claim.cases.is_empty()
        || claim.cases.iter().any(|c| c.trim().is_empty())
        || claim.cases.iter().collect::<BTreeSet<_>>().len() != claim.cases.len()
    {
        bail!(
            "behavior claim lacks exact distinct cases: {} {}",
            claim.model,
            claim.consumer
        );
    }
    Ok(match claim.kind {
        Behavior::Supported => {
            if claim.refusal.is_some() {
                bail!("contradictory supported/refused claim");
            }
            Disposition::Supported {
                cases: claim.cases.clone(),
                reason: claim.reason.clone(),
            }
        }
        Behavior::Refused => Disposition::Refused {
            cases: claim.cases.clone(),
            reason: claim.reason.clone(),
            refusal: claim
                .refusal
                .clone()
                .filter(|s| !s.trim().is_empty())
                .context("refused claim lacks named boundary refusal")?,
        },
    })
}
fn insert(
    cells: &mut BTreeMap<(String, String), PlannedCell>,
    cell: PlannedCell,
    problems: &mut Vec<String>,
) {
    let key = (cell.model.clone(), cell.consumer.clone());
    if cells.insert(key.clone(), cell).is_some() {
        problems.push(format!(
            "duplicate or contradictory cell {} {}",
            key.0, key.1
        ));
    }
}
fn validate_header(b: &Baseline) -> Result<()> {
    if b.format != "ess-consumer-initial-eligibility/1"
        || b.status != "ACCEPTED_INITIAL_ELIGIBILITY"
        || [
            &b.stage1_source_commit,
            &b.stage1_binding_sha256,
            &b.source_checkpoint_sha256,
            &b.extraction_profile,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
    {
        bail!("unknown or unaccepted baseline format/checkpoint");
    }
    let consumers = b
        .groups
        .iter()
        .map(|g| &g.consumer)
        .collect::<BTreeSet<_>>();
    if consumers.len() != b.groups.len()
        || b.models.len().checked_mul(consumers.len())
            != b.eligible_pairs.checked_add(b.mandatory_pairs_excluded)
    {
        bail!("invalid frozen baseline partition or duplicate consumer group");
    }
    Ok(())
}

fn read_plan(value: &Value) -> Result<ExecutionPlan> {
    for key in ["required_cases", "required_metadata_guards"] {
        let rows = value[key].as_array().context("planned requirement array")?;
        if rows
            .iter()
            .map(super::hash_json)
            .collect::<BTreeSet<_>>()
            .len()
            != rows.len()
        {
            bail!("duplicate execution-plan requirement {key}");
        }
    }
    let plan: ExecutionPlan = serde_json::from_value(value.clone())?;
    let mut pairs = BTreeSet::new();
    let mut models = BTreeMap::new();
    let mut profiles = BTreeMap::new();
    let mut cases = BTreeSet::new();
    let mut guards = BTreeSet::new();
    let mut unknowns = 0;
    for cell in &plan.cells {
        if !pairs.insert((&cell.model, &cell.consumer)) {
            bail!("duplicate execution-plan cell");
        }
        if models
            .insert(&cell.model, &cell.shape)
            .is_some_and(|s| s != &cell.shape)
            || profiles
                .insert(&cell.consumer, &cell.profile)
                .is_some_and(|s| s != &cell.profile)
        {
            bail!("inconsistent execution-plan model/profile identity");
        }
        match &cell.disposition {
            Disposition::Supported {
                cases: linked,
                reason,
            }
            | Disposition::Refused {
                cases: linked,
                reason,
                ..
            } => {
                if linked.is_empty()
                    || reason.trim().is_empty()
                    || linked.iter().collect::<BTreeSet<_>>().len() != linked.len()
                {
                    bail!("execution-plan behavior lacks exact cases/reason");
                }
                cases.extend(linked.iter().cloned());
            }
            Disposition::BaselineUnknown {
                owner,
                follow_up,
                reason,
            } => {
                if [owner, follow_up, reason]
                    .iter()
                    .any(|s| s.trim().is_empty())
                {
                    bail!("execution-plan unknown lacks ownership");
                }
                unknowns += 1;
            }
            Disposition::SchemaDocumentMetadata { evidence } => {
                evidence.matches_cell(&cell.model, &cell.shape, &cell.consumer, &cell.profile)?;
                guards.insert(evidence.clone());
            }
        }
        if let Disposition::Refused { refusal, .. } = &cell.disposition {
            if refusal.trim().is_empty() {
                bail!("execution-plan refusal lacks named boundary");
            }
        }
    }
    if cases != plan.required_cases
        || guards != plan.required_metadata_guards
        || models.len() != plan.discovered_models
        || profiles.len() != plan.bound_profiles
        || models.len().checked_mul(profiles.len()) != Some(plan.cells.len())
        || plan.baseline_unknown != unknowns
        || plan.pending_metadata_candidates != guards.len()
        || plan.metadata_manifest_sha256.is_some() == guards.is_empty()
        || plan.supported != 0
        || plan.refused != 0
        || plan.schema_document_metadata != 0
    {
        bail!("inconsistent or prematurely qualified execution plan");
    }
    Ok(plan)
}
pub(super) fn qualify(
    value: &Value,
    verified: &super::native::VerifiedCases,
    authority: &metadata::Authority,
    metadata: &metadata::Verified<'_>,
) -> Result<Value> {
    let plan = read_plan(value)?;
    authority.verify()?;
    metadata.check(
        authority,
        &plan.required_metadata_guards,
        plan.metadata_manifest_sha256
            .as_deref()
            .context("missing planned metadata manifest")?,
    )?;
    let result = qualify_cells(&plan, verified.ids(), &verified.receipt, metadata.receipt())?;
    authority.verify()?;
    Ok(result)
}
fn qualify_cells(
    plan: &ExecutionPlan,
    executed: &BTreeSet<String>,
    case_receipt: &Value,
    guard_receipt: &Value,
) -> Result<Value> {
    if &plan.required_cases != executed {
        bail!("qualifying cases differ from this run's exact executed set");
    }
    let mut counts = BTreeMap::from([
        ("Supported", 0usize),
        ("Refused", 0),
        ("BaselineUnknown", 0),
        ("SchemaDocumentMetadata", 0),
    ]);
    for cell in &plan.cells {
        let kind = match &cell.disposition {
            Disposition::Supported { .. } => "Supported",
            Disposition::Refused { .. } => "Refused",
            Disposition::BaselineUnknown { .. } => "BaselineUnknown",
            Disposition::SchemaDocumentMetadata { .. } => "SchemaDocumentMetadata",
        };
        *counts.get_mut(kind).expect("closed disposition") += 1;
    }
    if counts.values().sum::<usize>() != plan.cells.len()
        || counts["SchemaDocumentMetadata"] != plan.required_metadata_guards.len()
    {
        bail!("qualified accounting conservation failed");
    }
    Ok(json!({
        "format":account::FORMAT,"stage":account::QualifiedStage::Qualified,
        "status":"ACCOUNTED_WITH_VISIBLE_INITIAL_UNKNOWNS",
        "cells":plan.cells,"counts":counts,"executed_cases":case_receipt,
        "executed_case_count":executed.len(),"executed_metadata_guards":1,
        "metadata_guard":guard_receipt,
        "claim_limit":"Exact reviewed behavior assertions only; baseline unknown remains unproven. SchemaDocumentMetadata is executed representation bookkeeping, not behavior or runtime conformance."
    }))
}

#[cfg(test)]
#[path = "enforce_metadata_tests.rs"]
mod metadata_tests;
