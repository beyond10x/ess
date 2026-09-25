//! Closed finite admission consumes the root-owned initial eligibility unchanged.
use super::{account, metadata};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlannedCell {
    model: String,
    shape: String,
    consumer: String,
    profile: String,
    disposition: Disposition,
}
#[derive(Clone, Serialize, Deserialize)]
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
#[serde(deny_unknown_fields)]
struct PlannedCellV2 {
    model: String,
    shape: String,
    consumer: String,
    profile: String,
    disposition: DispositionV2,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum DispositionV2 {
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
    AggregateClosure {
        closure: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionPlanV2 {
    format: account::FormatV2,
    stage: account::ExecutionStage,
    execution_status: Pending,
    discovered_models: usize,
    bound_profiles: usize,
    cells: Vec<PlannedCellV2>,
    required_cases: BTreeSet<String>,
    required_metadata_guards: BTreeSet<metadata::Row>,
    metadata_manifest_sha256: Option<String>,
    reconciliation: Value,
    reconciliation_sha256: String,
    baseline_sha256: String,
    acquisition_plan: Value,
    acquisition_authority_sha256: String,
    aggregate_manifest_sha256: String,
    aggregate_authority: Value,
    pending_metadata_candidates: usize,
    pending_acquisition_rows: usize,
    pending_aggregate_closures: usize,
    #[serde(rename = "Supported")]
    supported: usize,
    #[serde(rename = "Refused")]
    refused: usize,
    #[serde(rename = "BaselineUnknown")]
    baseline_unknown: usize,
    #[serde(rename = "SchemaDocumentMetadata")]
    schema_document_metadata: usize,
    #[serde(rename = "AggregateClosure")]
    aggregate_closure: usize,
}

#[derive(Clone, Serialize, Deserialize)]
enum Pending {
    #[serde(rename = "PENDING_CASE_AND_GUARD_EXECUTION")]
    Execution,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionPlanV3 {
    format: account::FormatV3,
    stage: account::ExecutionStage,
    execution_status: Pending,
    discovered_models: usize,
    bound_profiles: usize,
    cells: Vec<PlannedCellV2>,
    required_cases: BTreeSet<String>,
    required_legacy_cases: BTreeSet<String>,
    required_metadata_guards: BTreeSet<metadata::Row>,
    metadata_manifest_sha256: Option<String>,
    reconciliation: Value,
    reconciliation_sha256: String,
    baseline_sha256: String,
    acquisition_plan: Value,
    acquisition_authority_sha256: String,
    aggregate_manifest_sha256: String,
    aggregate_authority: Value,
    model_behavior_plan: Value,
    pending_metadata_candidates: usize,
    pending_acquisition_rows: usize,
    pending_aggregate_closures: usize,
    #[serde(rename = "Supported")]
    supported: usize,
    #[serde(rename = "Refused")]
    refused: usize,
    #[serde(rename = "BaselineUnknown")]
    baseline_unknown: usize,
    #[serde(rename = "SchemaDocumentMetadata")]
    schema_document_metadata: usize,
    #[serde(rename = "AggregateClosure")]
    aggregate_closure: usize,
}
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
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

fn behavior_v2(claim: &Claim) -> Result<DispositionV2> {
    Ok(match behavior(claim)? {
        Disposition::Supported { cases, reason } => DispositionV2::Supported { cases, reason },
        Disposition::Refused {
            cases,
            reason,
            refusal,
        } => DispositionV2::Refused {
            cases,
            reason,
            refusal,
        },
        _ => unreachable!("behavior produces only executable dispositions"),
    })
}

fn insert_v2(
    cells: &mut BTreeMap<(String, String), PlannedCellV2>,
    cell: PlannedCellV2,
    problems: &mut Vec<String>,
) {
    let key = (cell.model.clone(), cell.consumer.clone());
    if cells.insert(key.clone(), cell).is_some() {
        problems.push(format!(
            "duplicate or contradictory v2 cell {} {}",
            key.0, key.1
        ));
    }
}

#[derive(Clone, Copy)]
pub(super) struct PlanV2Inputs<'a> {
    pub models: &'a Value,
    pub profiles: &'a Value,
    pub claims: &'a Value,
    pub metadata: &'a metadata::Candidates,
    pub reconciliation: &'a super::reconciliation::Resolution,
    pub baseline_sha256: &'a str,
    pub acquisition_plan: &'a Value,
    pub aggregate_authority: &'a Value,
}

#[expect(
    clippy::too_many_lines,
    reason = "the closed accounting plan is easier to audit as one ordered conservation proof"
)]
pub(super) fn plan_v2(inputs: PlanV2Inputs<'_>) -> Result<Value> {
    let PlanV2Inputs {
        models: models_value,
        profiles: profiles_value,
        claims: claims_value,
        metadata,
        reconciliation,
        baseline_sha256,
        acquisition_plan,
        aggregate_authority,
    } = inputs;
    let models: BTreeMap<String, String> = serde_json::from_value(models_value.clone())?;
    let profiles: BTreeMap<String, String> = serde_json::from_value(profiles_value.clone())?;
    let claims: Vec<Claim> = serde_json::from_value(claims_value.clone())?;
    if acquisition_plan["required_rows"] != 8
        || acquisition_plan["qualified_rows"] != 0
        || acquisition_plan["status"] != "PENDING_CASE_AND_GUARD_EXECUTION"
    {
        bail!("accounting v2 requires the exact pending eight-row acquisition plan");
    }
    let mut cells = BTreeMap::new();
    let mut problems = Vec::new();
    for unknown in &reconciliation.unchanged {
        insert_v2(
            &mut cells,
            PlannedCellV2 {
                model: unknown.identity.model.clone(),
                shape: unknown.identity.shape.clone(),
                consumer: unknown.identity.consumer.clone(),
                profile: unknown.identity.profile.clone(),
                disposition: DispositionV2::BaselineUnknown {
                    owner: unknown.owner.clone(),
                    follow_up: unknown.follow_up.clone(),
                    reason: unknown.reason.clone(),
                },
            },
            &mut problems,
        );
    }
    let mut required_cases = BTreeSet::new();
    for claim in claims {
        let disposition = behavior_v2(&claim)?;
        required_cases.extend(claim.cases.iter().cloned());
        let shape = models
            .get(&claim.model)
            .with_context(|| format!("unknown claimed model {}", claim.model))?;
        let profile = profiles
            .get(&claim.consumer)
            .with_context(|| format!("unknown claimed consumer {}", claim.consumer))?;
        insert_v2(
            &mut cells,
            PlannedCellV2 {
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
    for row in metadata.rows() {
        required_metadata_guards.insert(row.clone());
        insert_v2(
            &mut cells,
            PlannedCellV2 {
                model: row.model.clone(),
                shape: row.shape.clone(),
                consumer: row.consumer.clone(),
                profile: row.profile.clone(),
                disposition: DispositionV2::SchemaDocumentMetadata {
                    evidence: row.clone(),
                },
            },
            &mut problems,
        );
    }
    let aggregate_claims = super::aggregate::claims(aggregate_authority)?;
    for (identity, closure) in &aggregate_claims {
        if models.get(&identity.model) != Some(&identity.shape)
            || profiles.get(&identity.consumer) != Some(&identity.profile)
        {
            problems.push(format!(
                "stale aggregate cell {} {}",
                identity.model, identity.consumer
            ));
            continue;
        }
        insert_v2(
            &mut cells,
            PlannedCellV2 {
                model: identity.model.clone(),
                shape: identity.shape.clone(),
                consumer: identity.consumer.clone(),
                profile: identity.profile.clone(),
                disposition: DispositionV2::AggregateClosure {
                    closure: closure.clone(),
                },
            },
            &mut problems,
        );
    }
    required_cases.extend(super::aggregate::required_cases(aggregate_authority)?);
    for replacement in reconciliation.replacements.values() {
        let current = &replacement.current;
        let matches = cells
            .get(&(current.model.clone(), current.consumer.clone()))
            .is_some_and(|cell| replacement_matches(&replacement.claim, &cell.disposition));
        if !matches {
            problems.push(format!(
                "reconciliation replacement lacks its exact current behavior {} {}",
                current.model, current.consumer
            ));
        }
    }
    for cell in cells.values() {
        if models.get(&cell.model) != Some(&cell.shape)
            || profiles.get(&cell.consumer) != Some(&cell.profile)
        {
            problems.push(format!(
                "stale v2 shape/profile or removed cell {} {}",
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
    let current_unknowns = cells
        .values()
        .filter(|cell| matches!(cell.disposition, DispositionV2::BaselineUnknown { .. }))
        .count();
    if !problems.is_empty() {
        bail!(
            "{}",
            json!({
                "format":account::FORMAT_V2,
                "stage":"execution-plan",
                "accounting_complete":false,
                "discovered_models":models.len(),
                "bound_profiles":profiles.len(),
                "Supported":0,
                "Refused":0,
                "BaselineUnknown":current_unknowns,
                "SchemaDocumentMetadata":0,
                "AggregateClosure":0,
                "historical_retirements":reconciliation.retired.len(),
                "reconciliation":reconciliation.receipt,
                "pending_acquisition_rows":8,
                "executed_acquisition_rows":0,
                "unaccounted_or_stale":problems,
            })
        );
    }
    Ok(serde_json::to_value(ExecutionPlanV2 {
        format: account::FormatV2::V2,
        stage: account::ExecutionStage::ExecutionPlan,
        execution_status: Pending::Execution,
        discovered_models: models.len(),
        bound_profiles: profiles.len(),
        cells: cells.into_values().collect(),
        required_cases,
        pending_metadata_candidates: required_metadata_guards.len(),
        required_metadata_guards,
        metadata_manifest_sha256: Some(metadata.digest().to_owned()),
        reconciliation: reconciliation.receipt.clone(),
        reconciliation_sha256: reconciliation.digest.clone(),
        baseline_sha256: baseline_sha256.to_owned(),
        acquisition_plan: acquisition_plan.clone(),
        acquisition_authority_sha256: acquisition_plan["authority_sha256"]
            .as_str()
            .context("acquisition authority digest")?
            .to_owned(),
        aggregate_manifest_sha256: super::hash_json(aggregate_authority),
        aggregate_authority: aggregate_authority.clone(),
        pending_acquisition_rows: 8,
        pending_aggregate_closures: aggregate_claims.len(),
        supported: 0,
        refused: 0,
        baseline_unknown: current_unknowns,
        schema_document_metadata: 0,
        aggregate_closure: 0,
    })?)
}

#[derive(Clone, Copy)]
pub(super) struct PlanV3Inputs<'a> {
    pub accounting: PlanV2Inputs<'a>,
    pub model_behavior_plan: &'a Value,
}

pub(super) fn plan_v3(inputs: PlanV3Inputs<'_>) -> Result<Value> {
    let model_claims = super::model_behavior::claims(inputs.model_behavior_plan)?;
    let mut combined = inputs
        .accounting
        .claims
        .as_array()
        .context("legacy accounting claims")?
        .clone();
    for claim in &model_claims {
        let behavior = match &claim.behavior {
            super::model_behavior::Behavior::Supported { reason } => {
                json!({"kind":"Supported","reason":reason})
            }
            super::model_behavior::Behavior::Refused { reason, refusal } => {
                json!({"kind":"Refused","reason":reason,"refusal":refusal})
            }
        };
        let mut row = json!({
            "model":claim.identity.model,
            "consumer":claim.identity.consumer,
            "cases":claim.cases,
            "kind":behavior["kind"],
            "reason":behavior["reason"],
        });
        if let Some(refusal) = behavior.get("refusal") {
            row["refusal"] = refusal.clone();
        }
        combined.push(row);
    }
    let combined = json!(combined);
    let mut legacy_cases = BTreeSet::new();
    for claim in serde_json::from_value::<Vec<Claim>>(inputs.accounting.claims.clone())? {
        legacy_cases.extend(claim.cases);
    }
    legacy_cases.extend(super::aggregate::required_cases(
        inputs.accounting.aggregate_authority,
    )?);
    let model_cases = super::model_behavior::case_ids(inputs.model_behavior_plan)?;
    validate_case_partition(
        &legacy_cases.union(&model_cases).cloned().collect(),
        &legacy_cases,
        &model_cases,
    )?;
    let accounting = inputs.accounting;
    let mut value = plan_v2(PlanV2Inputs {
        models: accounting.models,
        profiles: accounting.profiles,
        claims: &combined,
        metadata: accounting.metadata,
        reconciliation: accounting.reconciliation,
        baseline_sha256: accounting.baseline_sha256,
        acquisition_plan: accounting.acquisition_plan,
        aggregate_authority: accounting.aggregate_authority,
    })?;
    value["format"] = json!(account::FORMAT_V3);
    value["required_legacy_cases"] = json!(legacy_cases);
    value["model_behavior_plan"] = inputs.model_behavior_plan.clone();
    read_plan_v3(&value)?;
    Ok(value)
}

fn replacement_matches(
    claim: &super::reconciliation::ReplacementClaim,
    disposition: &DispositionV2,
) -> bool {
    match (claim, disposition) {
        (
            super::reconciliation::ReplacementClaim::Supported {
                cases: expected_cases,
                reason: expected_reason,
            },
            DispositionV2::Supported { cases, reason },
        ) => expected_cases == cases && expected_reason == reason,
        (
            super::reconciliation::ReplacementClaim::Refused {
                cases: expected_cases,
                reason: expected_reason,
                refusal: expected_refusal,
            },
            DispositionV2::Refused {
                cases,
                reason,
                refusal,
            },
        ) => expected_cases == cases && expected_reason == reason && expected_refusal == refusal,
        (
            super::reconciliation::ReplacementClaim::AggregateClosure { closure: expected },
            DispositionV2::AggregateClosure { closure },
        ) => expected == closure,
        _ => false,
    }
}

#[cfg(test)]
pub(super) fn test_replacement_matches(claim: &Value, disposition: &Value) -> Result<bool> {
    Ok(replacement_matches(
        &serde_json::from_value(claim.clone())?,
        &serde_json::from_value(disposition.clone())?,
    ))
}
#[cfg(test)]
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
#[cfg(test)]
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

#[cfg(test)]
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
#[cfg(test)]
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

fn read_plan_v2(value: &Value) -> Result<ExecutionPlanV2> {
    let plan: ExecutionPlanV2 = serde_json::from_value(value.clone())?;
    let mut pairs = BTreeSet::new();
    let mut models = BTreeMap::new();
    let mut profiles = BTreeMap::new();
    let mut cases = BTreeSet::new();
    let mut guards = BTreeSet::new();
    let mut unknowns = 0;
    let mut aggregates = 0;
    for cell in &plan.cells {
        if !pairs.insert((&cell.model, &cell.consumer)) {
            bail!("duplicate v2 execution-plan cell");
        }
        if models
            .insert(&cell.model, &cell.shape)
            .is_some_and(|shape| shape != &cell.shape)
            || profiles
                .insert(&cell.consumer, &cell.profile)
                .is_some_and(|profile| profile != &cell.profile)
        {
            bail!("inconsistent v2 execution-plan model/profile identity");
        }
        match &cell.disposition {
            DispositionV2::Supported {
                cases: linked,
                reason,
            }
            | DispositionV2::Refused {
                cases: linked,
                reason,
                ..
            } => {
                if linked.is_empty()
                    || reason.trim().is_empty()
                    || linked.iter().collect::<BTreeSet<_>>().len() != linked.len()
                {
                    bail!("v2 behavior lacks exact distinct cases and reason");
                }
                cases.extend(linked.iter().cloned());
            }
            DispositionV2::BaselineUnknown {
                owner,
                follow_up,
                reason,
            } => {
                if [owner, follow_up, reason]
                    .iter()
                    .any(|value| value.trim().is_empty())
                {
                    bail!("v2 unknown lacks visible independent ownership");
                }
                unknowns += 1;
            }
            DispositionV2::SchemaDocumentMetadata { evidence } => {
                evidence.matches_cell(&cell.model, &cell.shape, &cell.consumer, &cell.profile)?;
                guards.insert(evidence.clone());
            }
            DispositionV2::AggregateClosure { closure } => {
                if closure.trim().is_empty() {
                    bail!("v2 aggregate cell lacks closure identity");
                }
                aggregates += 1;
            }
        }
        if let DispositionV2::Refused { refusal, .. } = &cell.disposition {
            if refusal.trim().is_empty() {
                bail!("v2 refusal lacks named boundary");
            }
        }
    }
    cases.extend(super::aggregate::required_cases(&plan.aggregate_authority)?);
    if cases != plan.required_cases
        || guards != plan.required_metadata_guards
        || models.len() != plan.discovered_models
        || profiles.len() != plan.bound_profiles
        || models.len().checked_mul(profiles.len()) != Some(plan.cells.len())
        || plan.baseline_unknown != unknowns
        || plan.pending_aggregate_closures != aggregates
        || plan.pending_acquisition_rows != 8
        || plan.acquisition_plan["required_rows"] != 8
        || plan.acquisition_plan["qualified_rows"] != 0
        || plan.acquisition_authority_sha256 != plan.acquisition_plan["authority_sha256"]
        || plan.reconciliation_sha256 != plan.reconciliation["reconciliation_sha256"]
        || super::hash_json(&plan.aggregate_authority) != plan.aggregate_manifest_sha256
        || plan.pending_metadata_candidates != guards.len()
        || plan.metadata_manifest_sha256.is_none()
        || plan.supported != 0
        || plan.refused != 0
        || plan.schema_document_metadata != 0
        || plan.aggregate_closure != 0
    {
        bail!("inconsistent or prematurely qualified v2 execution plan");
    }
    Ok(plan)
}

fn read_plan_v3(value: &Value) -> Result<ExecutionPlanV3> {
    let plan: ExecutionPlanV3 = serde_json::from_value(value.clone())?;
    let mut v2 = value.clone();
    v2["format"] = json!(account::FORMAT_V2);
    v2.as_object_mut()
        .context("accounting v3 plan object")?
        .remove("required_legacy_cases");
    v2.as_object_mut()
        .context("accounting v3 plan object")?
        .remove("model_behavior_plan");
    read_plan_v2(&v2)?;
    let model_cases = super::model_behavior::case_ids(&plan.model_behavior_plan)?;
    let expected_legacy = plan
        .required_cases
        .difference(&model_cases)
        .cloned()
        .collect::<BTreeSet<_>>();
    validate_case_partition(
        &plan.required_cases,
        &plan.required_legacy_cases,
        &model_cases,
    )?;
    if plan.required_legacy_cases != expected_legacy {
        bail!("accounting v3 legacy case set is not the exact non-model set")
    }
    let claims = super::model_behavior::claims(&plan.model_behavior_plan)?;
    for claim in &claims {
        let matches = plan
            .cells
            .iter()
            .filter(|cell| {
                cell.model == claim.identity.model
                    && cell.shape == claim.identity.shape
                    && cell.consumer == claim.identity.consumer
                    && cell.profile == claim.identity.profile
                    && model_claim_matches(&cell.disposition, claim)
            })
            .count();
        if matches != 1 {
            bail!(
                "accounting v3 model-behavior claim is not consumed exactly once: {} {}",
                claim.identity.model,
                claim.identity.consumer
            )
        }
    }
    for cell in &plan.cells {
        let linked = match &cell.disposition {
            DispositionV2::Supported { cases, .. } | DispositionV2::Refused { cases, .. } => cases,
            DispositionV2::BaselineUnknown { .. }
            | DispositionV2::SchemaDocumentMetadata { .. }
            | DispositionV2::AggregateClosure { .. } => continue,
        };
        if linked.iter().any(|case| model_cases.contains(case)) {
            if linked.iter().any(|case| !model_cases.contains(case)) {
                bail!("accounting v3 behavior cell mixes legacy and model cases")
            }
            let matches = claims
                .iter()
                .filter(|claim| {
                    cell.model == claim.identity.model
                        && cell.shape == claim.identity.shape
                        && cell.consumer == claim.identity.consumer
                        && cell.profile == claim.identity.profile
                        && model_claim_matches(&cell.disposition, claim)
                })
                .count();
            if matches != 1 {
                bail!(
                    "accounting v3 model-backed cell is not claimed exactly once: {} {}",
                    cell.model,
                    cell.consumer
                )
            }
        }
    }
    Ok(plan)
}

#[cfg(test)]
pub(super) fn test_read_plan_v3(value: &Value) -> Result<()> {
    read_plan_v3(value).map(|_| ())
}

fn validate_case_partition(
    required: &BTreeSet<String>,
    legacy: &BTreeSet<String>,
    model: &BTreeSet<String>,
) -> Result<()> {
    if !legacy.is_disjoint(model)
        || legacy.union(model).cloned().collect::<BTreeSet<_>>() != *required
    {
        bail!("accounting v3 required-case partition is inconsistent")
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn test_case_partition(
    required: &[&str],
    legacy: &[&str],
    model: &[&str],
) -> Result<()> {
    let set = |values: &[&str]| values.iter().map(|value| (*value).to_owned()).collect();
    validate_case_partition(&set(required), &set(legacy), &set(model))
}

fn model_claim_matches(disposition: &DispositionV2, claim: &super::model_behavior::Claim) -> bool {
    match (disposition, &claim.behavior) {
        (
            DispositionV2::Supported { cases, reason },
            super::model_behavior::Behavior::Supported {
                reason: claim_reason,
            },
        ) => cases == &claim.cases && reason == claim_reason,
        (
            DispositionV2::Refused {
                cases,
                reason,
                refusal,
            },
            super::model_behavior::Behavior::Refused {
                reason: claim_reason,
                refusal: claim_refusal,
            },
        ) => cases == &claim.cases && reason == claim_reason && refusal == claim_refusal,
        _ => false,
    }
}

pub(super) fn aggregate_cells(value: &Value) -> Result<Value> {
    let cells = if value["format"] == account::FORMAT_V3 {
        read_plan_v3(value)?.cells
    } else {
        read_plan_v2(value)?.cells
    };
    let rows = cells
        .iter()
        .filter_map(|cell| match &cell.disposition {
            DispositionV2::Supported { cases, .. } => Some(json!({
                "model":cell.model,
                "consumer":cell.consumer,
                "profile":cell.profile,
                "kind":"Supported",
                "cases":cases,
            })),
            DispositionV2::Refused { cases, refusal, .. } => Some(json!({
                "model":cell.model,
                "consumer":cell.consumer,
                "profile":cell.profile,
                "kind":"Refused",
                "cases":cases,
                "refusal":refusal,
            })),
            DispositionV2::AggregateClosure { .. } => Some(json!({
                "model":cell.model,
                "consumer":cell.consumer,
                "profile":cell.profile,
                "kind":"AggregateClosure",
                "cases":[],
            })),
            DispositionV2::BaselineUnknown { .. }
            | DispositionV2::SchemaDocumentMetadata { .. } => None,
        })
        .collect::<Vec<_>>();
    Ok(json!(rows))
}

#[allow(
    dead_code,
    reason = "the accounting/2 qualifier remains available for compatibility verification"
)]
pub(super) fn qualify_v2(
    value: &Value,
    verified: &super::native::VerifiedCases,
    authority: &metadata::Authority,
    metadata: &metadata::Verified<'_>,
    acquisition: &Value,
    aggregate: &Value,
) -> Result<Value> {
    let plan = read_plan_v2(value)?;
    authority.verify()?;
    metadata.check(
        authority,
        &plan.required_metadata_guards,
        plan.metadata_manifest_sha256
            .as_deref()
            .context("missing planned metadata manifest")?,
    )?;
    if &plan.required_cases != verified.ids()
        || acquisition["format"] != super::scenario_acquisition::FORMAT
        || acquisition["qualified_rows"] != 8
        || acquisition["guard_executed"] != 1
        || aggregate["format"] != super::aggregate::FORMAT
        || aggregate["aggregate_manifest_sha256"] != plan.aggregate_manifest_sha256
        || aggregate["qualified_closures"] != plan.pending_aggregate_closures
    {
        bail!("v2 qualification evidence differs from the exact execution plan");
    }
    let mut counts = BTreeMap::from([
        ("Supported", 0usize),
        ("Refused", 0),
        ("BaselineUnknown", 0),
        ("SchemaDocumentMetadata", 0),
        ("AggregateClosure", 0),
    ]);
    for cell in &plan.cells {
        let kind = match cell.disposition {
            DispositionV2::Supported { .. } => "Supported",
            DispositionV2::Refused { .. } => "Refused",
            DispositionV2::BaselineUnknown { .. } => "BaselineUnknown",
            DispositionV2::SchemaDocumentMetadata { .. } => "SchemaDocumentMetadata",
            DispositionV2::AggregateClosure { .. } => "AggregateClosure",
        };
        *counts.get_mut(kind).expect("closed v2 disposition") += 1;
    }
    if counts.values().sum::<usize>() != plan.cells.len() {
        bail!("v2 qualified accounting conservation failed");
    }
    authority.verify()?;
    Ok(json!({
        "format":account::FORMAT_V2,
        "stage":account::QualifiedStage::Qualified,
        "status":"ACCOUNTED_WITH_VISIBLE_INITIAL_UNKNOWNS",
        "cells":plan.cells,
        "counts":counts,
        "historical_retirements":plan.reconciliation["retirements"],
        "reconciliation":plan.reconciliation,
        "scenario_acquisition":acquisition,
        "aggregate_proof":aggregate,
        "executed_cases":verified.receipt,
        "executed_case_count":verified.ids().len(),
        "executed_metadata_guards":1,
        "metadata_guard":metadata.receipt(),
        "claim_limit":"Exact reviewed behavior only; visible initial unknowns and aggregate residuals remain unproven. Acquisition and historical retirements are separate from current cell conservation."
    }))
}

#[allow(
    clippy::too_many_arguments,
    reason = "accounting/3 verifies each independent closed evidence authority explicitly"
)]
pub(super) fn qualify_v3(
    value: &Value,
    verified: &super::native::VerifiedCases,
    authority: &metadata::Authority,
    metadata: &metadata::Verified<'_>,
    acquisition: &Value,
    aggregate: &Value,
    model_candidate: &Value,
    model_execution: &Value,
) -> Result<Value> {
    let plan = read_plan_v3(value)?;
    authority.verify()?;
    metadata.check(
        authority,
        &plan.required_metadata_guards,
        plan.metadata_manifest_sha256
            .as_deref()
            .context("missing planned metadata manifest")?,
    )?;
    let executed_claims = super::model_behavior::verify_execution(
        model_execution,
        &plan.model_behavior_plan,
        model_candidate,
    )?;
    if &plan.required_legacy_cases != verified.ids()
        || executed_claims != super::model_behavior::claims(&plan.model_behavior_plan)?
        || acquisition["format"] != super::scenario_acquisition::FORMAT
        || acquisition["qualified_rows"] != 8
        || acquisition["guard_executed"] != 1
        || aggregate["format"] != super::aggregate::FORMAT
        || aggregate["aggregate_manifest_sha256"] != plan.aggregate_manifest_sha256
        || aggregate["qualified_closures"] != plan.pending_aggregate_closures
    {
        bail!("v3 qualification evidence differs from the exact execution plan")
    }
    let model_case_count = model_execution["receipts"]
        .as_object()
        .context("model-behavior execution receipts")?
        .len();
    let mut counts = BTreeMap::from([
        ("Supported", 0usize),
        ("Refused", 0),
        ("BaselineUnknown", 0),
        ("SchemaDocumentMetadata", 0),
        ("AggregateClosure", 0),
    ]);
    for cell in &plan.cells {
        let kind = match cell.disposition {
            DispositionV2::Supported { .. } => "Supported",
            DispositionV2::Refused { .. } => "Refused",
            DispositionV2::BaselineUnknown { .. } => "BaselineUnknown",
            DispositionV2::SchemaDocumentMetadata { .. } => "SchemaDocumentMetadata",
            DispositionV2::AggregateClosure { .. } => "AggregateClosure",
        };
        *counts.get_mut(kind).expect("closed v3 disposition") += 1;
    }
    if counts.values().sum::<usize>() != plan.cells.len() {
        bail!("v3 qualified accounting conservation failed")
    }
    authority.verify()?;
    Ok(json!({
        "format":account::FORMAT_V3,
        "stage":account::QualifiedStage::Qualified,
        "status":"ACCOUNTED_WITH_VISIBLE_INITIAL_UNKNOWNS",
        "cells":plan.cells,
        "counts":counts,
        "historical_retirements":plan.reconciliation["retirements"],
        "reconciliation":plan.reconciliation,
        "scenario_acquisition":acquisition,
        "aggregate_proof":aggregate,
        "executed_cases":verified.receipt,
        "model_behavior_execution":model_execution,
        "executed_case_count":verified.ids().len() + model_case_count,
        "executed_metadata_guards":1,
        "metadata_guard":metadata.receipt(),
        "claim_limit":"Exact reviewed behavior only; visible initial unknowns and aggregate residuals remain unproven. Acquisition and historical retirements are separate from current cell conservation."
    }))
}

#[cfg(test)]
#[path = "enforce_metadata_tests.rs"]
mod metadata_tests;
