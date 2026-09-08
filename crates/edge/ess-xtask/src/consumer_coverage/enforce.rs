//! Closed finite admission consumes the root-owned initial eligibility unchanged.
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
}

pub(super) fn plan(
    models: &Value,
    profiles: &Value,
    baseline: &Value,
    claims: &Value,
) -> Result<Value> {
    let models: BTreeMap<String, String> = serde_json::from_value(models.clone())?;
    let profiles: BTreeMap<String, String> = serde_json::from_value(profiles.clone())?;
    let baseline: Baseline = serde_json::from_value(baseline.clone())?;
    let claims: Vec<Claim> = serde_json::from_value(claims.clone())?;
    validate_header(&baseline)?;
    let mut cells = BTreeMap::new();
    let mut problems = Vec::new();
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
            let cell = PlannedCell {
                model: model.clone(),
                shape: shape.clone(),
                consumer: group.consumer.clone(),
                profile: group.profile.clone(),
                disposition: Disposition::BaselineUnknown {
                    owner: group.owner.clone(),
                    follow_up: group.follow_up.clone(),
                    reason: group.reason.clone(),
                },
            };
            insert(&mut cells, cell, &mut problems);
            unknowns += 1;
        }
    }
    if unknowns != baseline.eligible_pairs {
        problems.push("accepted finite pair count disagrees with manifest".into());
    }
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
            json!({"accounting_complete":false,"discovered_models":models.len(),"bound_profiles":profiles.len(),"Supported":0,"Refused":0,"BaselineUnknown":current_unknowns,"unaccounted_or_stale":problems})
        );
    }
    Ok(
        json!({"execution_status":"PENDING_CASE_EXECUTION","discovered_models":models.len(),"bound_profiles":profiles.len(),"cells":cells.into_values().collect::<Vec<_>>(),"required_cases":required_cases,"Supported":0,"Refused":0,"BaselineUnknown":unknowns}),
    )
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

pub(super) fn qualify(plan: &Value, verified: &super::native::VerifiedCases) -> Result<Value> {
    let required: BTreeSet<String> = serde_json::from_value(plan["required_cases"].clone())?;
    if &required != verified.ids() {
        bail!("qualifying cases differ from this run's exact executed set");
    }
    let cells: Vec<PlannedCell> = serde_json::from_value(plan["cells"].clone())?;
    let mut counts = BTreeMap::from([
        ("Supported", 0usize),
        ("Refused", 0),
        ("BaselineUnknown", 0),
    ]);
    for cell in &cells {
        let (kind, cases) = match &cell.disposition {
            Disposition::Supported { cases, .. } => ("Supported", Some(cases)),
            Disposition::Refused { cases, .. } => ("Refused", Some(cases)),
            Disposition::BaselineUnknown { .. } => ("BaselineUnknown", None),
        };
        if let Some(cases) = cases {
            if cases.is_empty() || cases.iter().any(|id| !verified.ids().contains(id)) {
                bail!(
                    "cell lacks actual case execution {} {}",
                    cell.model,
                    cell.consumer
                );
            }
        }
        *counts.get_mut(kind).expect("closed disposition") += 1;
    }
    Ok(
        json!({"status":"ACCOUNTED_WITH_VISIBLE_INITIAL_UNKNOWNS","cells":cells,"counts":counts,"executed_cases":verified.receipt,"claim_limit":"Exact reviewed assertions only; baseline unknown is neither supported nor refused."}),
    )
}
