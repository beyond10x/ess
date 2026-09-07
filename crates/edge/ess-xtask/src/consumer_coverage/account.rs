//! Finite candidate accounting does not grant baseline admission.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Cell {
    pub model: String,
    pub shape: String,
    pub consumer: String,
    pub profile: String,
    pub status: Status,
    pub disposition: Disposition,
}
#[derive(Debug, Serialize, Deserialize)]
pub(super) enum Status {
    #[serde(rename = "UNACCEPTED")]
    Unaccepted,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(super) enum Disposition {
    PendingOwner {
        reason: String,
    },
    UnknownProposal {
        owner: String,
        follow_up: String,
        reason: String,
    },
    MandatoryUnqualified {
        requirement: String,
        reason: String,
    },
    CaseCandidate {
        requirement: String,
        cases: Vec<String>,
        reason: String,
    },
}
#[cfg(test)]
pub(super) fn checkpoint(models: &Value, consumers: &Value, rows: &Value) -> Result<Value> {
    let rows: Vec<Cell> = serde_json::from_value(rows.clone())?;
    check(models, consumers, &rows)
}
pub(super) fn check(models: &Value, consumers: &Value, rows: &[Cell]) -> Result<Value> {
    let models = models.as_object().context("model inventory object")?;
    let consumers = consumers.as_object().context("consumer profile object")?;
    let mut seen = BTreeSet::new();
    let mut counts = std::collections::BTreeMap::<&str, usize>::new();
    for row in rows {
        if !seen.insert((&row.model, &row.consumer)) {
            bail!("duplicate checkpoint cell {} {}", row.model, row.consumer);
        }
        if models.get(&row.model) != Some(&Value::String(row.shape.clone()))
            || consumers.get(&row.consumer) != Some(&Value::String(row.profile.clone()))
        {
            bail!(
                "stale or unknown checkpoint cell {} {}",
                row.model,
                row.consumer
            );
        }
        let (kind, reason) = match &row.disposition {
            Disposition::PendingOwner { reason } => ("PendingOwner", reason),
            Disposition::UnknownProposal {
                owner,
                follow_up,
                reason,
            } => {
                if owner.trim().is_empty()
                    || follow_up.trim().is_empty()
                    || follow_up == "story:review-consumer-coverage"
                {
                    bail!("unknown proposal lacks independent follow-up ownership");
                }
                ("UnknownProposal", reason)
            }
            Disposition::MandatoryUnqualified {
                requirement,
                reason,
            } => {
                if requirement.is_empty() {
                    bail!("unnamed mandatory requirement");
                }
                ("MandatoryUnqualified", reason)
            }
            Disposition::CaseCandidate {
                requirement,
                cases,
                reason,
            } => {
                if requirement.is_empty()
                    || cases.is_empty()
                    || cases.iter().collect::<BTreeSet<_>>().len() != cases.len()
                {
                    bail!("missing or duplicate exact case candidate");
                }
                ("CaseCandidate", reason)
            }
        };
        if reason.trim().is_empty() {
            bail!("checkpoint cell lacks a precise unproven behavior statement");
        }
        *counts.entry(kind).or_default() += 1;
    }
    for model in models.keys() {
        for consumer in consumers.keys() {
            if !seen.contains(&(model, consumer)) {
                bail!("missing checkpoint cell {model} {consumer}");
            }
        }
    }
    Ok(
        serde_json::json!({"accounting_complete":true,"eligibility_valid":false,"status":"UNACCEPTED","discovered_models":models.len(),"bound_consumer_profiles":consumers.len(),"cells":rows.len(),"dispositions":counts,"Supported":0,"Refused":0,"BaselineUnknown":0,"reason":"Stage 1 source candidates are unexecuted; ownerless and mandatory records cannot be admitted as baseline unknowns."}),
    )
}
