//! Finite, reviewed reconciliation of the frozen eligibility manifest with current identities.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const FORMAT: &str = "ess-consumer-reconciliation/1";

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    pub model: String,
    pub shape: String,
    pub consumer: String,
    pub profile: String,
}

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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    baseline_sha256: String,
    source_evidence: String,
    review_reference: String,
    decisions: Vec<Decision>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    old: Identity,
    source_evidence: String,
    review_reference: String,
    action: Action,
}

#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Action {
    RetireConsumerProfile,
    RetireModel,
    Replace {
        current: Identity,
        claim: ReplacementClaim,
    },
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(super) enum ReplacementClaim {
    Supported {
        cases: Vec<String>,
        reason: String,
    },
    Refused {
        cases: Vec<String>,
        reason: String,
        refusal: String,
    },
    AggregateClosure {
        closure: String,
    },
}

impl ReplacementClaim {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Supported { cases, reason } => validate_behavior(cases, reason, None),
            Self::Refused {
                cases,
                reason,
                refusal,
            } => validate_behavior(cases, reason, Some(refusal)),
            Self::AggregateClosure { closure } => {
                if closure.trim().is_empty() {
                    bail!("replacement aggregate closure is unnamed");
                }
                Ok(())
            }
        }
    }
}

fn validate_behavior(cases: &[String], reason: &str, refusal: Option<&String>) -> Result<()> {
    if cases.is_empty()
        || cases.iter().any(|case| case.trim().is_empty())
        || cases.iter().collect::<BTreeSet<_>>().len() != cases.len()
        || reason.trim().is_empty()
        || refusal.is_some_and(|value| value.trim().is_empty())
    {
        bail!("replacement behavior claim lacks exact cases or evidence");
    }
    Ok(())
}

pub(super) struct Replacement {
    pub current: Identity,
    pub claim: ReplacementClaim,
    pub unknown: Unknown,
}

#[derive(Clone)]
pub(super) struct Unknown {
    pub identity: Identity,
    pub owner: String,
    pub follow_up: String,
    pub reason: String,
}

pub(super) struct Resolution {
    pub unchanged: Vec<Unknown>,
    pub replacements: BTreeMap<Identity, Replacement>,
    pub retired: BTreeSet<Identity>,
    pub digest: String,
    pub receipt: Value,
}

#[expect(
    clippy::too_many_lines,
    reason = "the finite reconciliation is a single consumed-once partition audit"
)]
pub(super) fn resolve(
    baseline_value: &Value,
    baseline_digest: &str,
    current_models_value: &Value,
    current_profiles_value: &Value,
    acquisition_profiles_value: &Value,
    authority_value: &Value,
) -> Result<Resolution> {
    let baseline: Baseline = serde_json::from_value(baseline_value.clone())?;
    let current_models: BTreeMap<String, String> =
        serde_json::from_value(current_models_value.clone())?;
    let current_profiles: BTreeMap<String, String> =
        serde_json::from_value(current_profiles_value.clone())?;
    let acquisition_profiles: BTreeMap<String, String> =
        serde_json::from_value(acquisition_profiles_value.clone())?;
    let manifest: Manifest = serde_json::from_value(authority_value.clone())?;
    if baseline.format != "ess-consumer-initial-eligibility/1"
        || baseline.status != "ACCEPTED_INITIAL_ELIGIBILITY"
        || [
            &baseline.stage1_source_commit,
            &baseline.stage1_binding_sha256,
            &baseline.source_checkpoint_sha256,
            &baseline.extraction_profile,
        ]
        .iter()
        .any(|value| value.trim().is_empty())
    {
        bail!("unknown or unaccepted initial eligibility");
    }
    if manifest.format != FORMAT
        || manifest.baseline_sha256 != baseline_digest
        || manifest.source_evidence.trim().is_empty()
        || manifest.review_reference.trim().is_empty()
    {
        bail!("unknown, stale or unevidenced reconciliation authority");
    }
    if current_profiles
        .keys()
        .any(|consumer| acquisition_profiles.contains_key(consumer))
    {
        bail!("scenario acquisition profile remains in the model matrix");
    }

    let mut old_cells = BTreeMap::<Identity, Unknown>::new();
    for group in &baseline.groups {
        if group.exact_model_ids.is_empty()
            || [&group.owner, &group.follow_up, &group.reason]
                .iter()
                .any(|value| value.trim().is_empty())
        {
            bail!("baseline unknown lacks finite visible ownership");
        }
        for model in &group.exact_model_ids {
            let identity = Identity {
                model: model.clone(),
                shape: baseline
                    .models
                    .get(model)
                    .with_context(|| format!("baseline dictionary lacks {model}"))?
                    .clone(),
                consumer: group.consumer.clone(),
                profile: group.profile.clone(),
            };
            let unknown = Unknown {
                identity: identity.clone(),
                owner: group.owner.clone(),
                follow_up: group.follow_up.clone(),
                reason: group.reason.clone(),
            };
            if old_cells.insert(identity, unknown).is_some() {
                bail!("duplicate frozen eligible tuple");
            }
        }
    }
    if old_cells.len() != baseline.eligible_pairs
        || baseline
            .eligible_pairs
            .checked_add(baseline.mandatory_pairs_excluded)
            != baseline.models.len().checked_mul(baseline.groups.len())
    {
        bail!("invalid frozen baseline partition");
    }

    let mut supplied = BTreeMap::new();
    for decision in manifest.decisions {
        if decision.source_evidence.trim().is_empty() || decision.review_reference.trim().is_empty()
        {
            bail!("reconciliation decision lacks source or review evidence");
        }
        if supplied.insert(decision.old.clone(), decision).is_some() {
            bail!("duplicate reconciliation decision");
        }
    }

    let mut unchanged = Vec::new();
    let mut replacements = BTreeMap::new();
    let mut retired = BTreeSet::new();
    for (old, unknown) in old_cells {
        let exact_current = current_models.get(&old.model) == Some(&old.shape)
            && current_profiles.get(&old.consumer) == Some(&old.profile);
        if exact_current {
            if supplied.remove(&old).is_some() {
                bail!("unchanged frozen tuple must not be reconciled");
            }
            unchanged.push(unknown);
            continue;
        }
        let decision = supplied
            .remove(&old)
            .with_context(|| format!("missing reconciliation decision {old:?}"))?;
        match decision.action {
            Action::RetireConsumerProfile => {
                if !acquisition_profiles.contains_key(&old.consumer)
                    || current_profiles.contains_key(&old.consumer)
                {
                    bail!("consumer-profile retirement is absent, current, or unguarded");
                }
                retired.insert(old);
            }
            Action::RetireModel => {
                if current_models.contains_key(&old.model)
                    || !current_profiles.contains_key(&old.consumer)
                {
                    bail!("model retirement names a current model or noncurrent consumer");
                }
                retired.insert(old);
            }
            Action::Replace { current, claim } => {
                if current.model != old.model
                    || current.consumer != old.consumer
                    || current_models.get(&current.model) != Some(&current.shape)
                    || current_profiles.get(&current.consumer) != Some(&current.profile)
                    || current == old
                    || acquisition_profiles.contains_key(&current.consumer)
                {
                    bail!("replacement does not name the exact changed current tuple");
                }
                claim.validate()?;
                replacements.insert(
                    old,
                    Replacement {
                        current,
                        claim,
                        unknown,
                    },
                );
            }
        }
    }
    if let Some(extra) = supplied.keys().next() {
        bail!("extra reconciliation decision {extra:?}");
    }
    let digest = super::hash_json(authority_value);
    let receipt = json!({
        "format": FORMAT,
        "baseline_sha256": baseline_digest,
        "reconciliation_sha256": digest,
        "consumed_decisions": replacements.len() + retired.len(),
        "unchanged_unknowns": unchanged.len(),
        "replacements": replacements.len(),
        "retirements": retired.len(),
        "source_evidence": manifest.source_evidence,
        "review_reference": manifest.review_reference,
    });
    Ok(Resolution {
        unchanged,
        replacements,
        retired,
        digest,
        receipt,
    })
}

#[cfg(test)]
pub(super) fn validate(
    baseline: &Value,
    baseline_digest: &str,
    current_models: &Value,
    current_profiles: &Value,
    acquisition_profiles: &Value,
    authority: &Value,
) -> Result<Value> {
    Ok(resolve(
        baseline,
        baseline_digest,
        current_models,
        current_profiles,
        acquisition_profiles,
        authority,
    )?
    .receipt)
}
