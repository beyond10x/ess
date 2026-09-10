//! Preserve the reviewed mapping without granting any semantic eligibility.
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};

pub(super) fn check(root: &Path) -> Result<()> {
    let directory = root.join("crates/edge/ess-xtask/src/consumer_coverage");
    let read = |name: &str| -> Result<Value> {
        serde_json::from_slice(&fs::read(directory.join(name))?)
            .with_context(|| format!("read preservation authority {name}"))
    };
    validate(
        &read("feature-preservation.json")?,
        &read("profiles.json")?,
        &read("reviewed-candidates.json")?,
    )
    .context("feature-preservation mapping refused")
}

fn validate(mapping: &Value, profiles: &Value, reviewed: &Value) -> Result<()> {
    for (field, expected) in [
        ("format", json!("ess-feature-preservation/1")),
        (
            "baseline_revision",
            json!("abf51add80d0f083c1f701330173cc10d609ad46"),
        ),
        (
            "source_authorities",
            json!([
                "profiles.json",
                "reviewed-candidates.json",
                "initial-baseline.json",
                "reviewed-schema-metadata.json",
                "macro-guards.json"
            ]),
        ),
    ] {
        if mapping.get(field) != Some(&expected) {
            bail!("changed {field}");
        }
    }
    let mut remaining = BTreeMap::new();
    for profile in profiles
        .as_array()
        .context("consumer profiles must be an array")?
    {
        let id = profile["id"].as_str().context("consumer profile id")?;
        if remaining.insert(id, profile).is_some() {
            bail!("duplicate profile {id}");
        }
    }
    let requirements = reviewed["requirements"]
        .as_array()
        .context("reviewed requirements must be an array")?;
    for row in mapping["mapping"]
        .as_array()
        .context("mapping must be an array")?
    {
        let id = row["consumer"].as_str().context("mapping consumer id")?;
        let profile = remaining
            .remove(id)
            .with_context(|| format!("unknown or duplicate consumer {id}"))?;
        let mut expected_requirements = Vec::new();
        for requirement in requirements {
            if requirement["consumers"]
                .as_array()
                .context("reviewed requirement consumers")?
                .iter()
                .any(|consumer| consumer.as_str() == Some(id))
            {
                expected_requirements.push(json!({
                    "id": requirement["id"],
                    "cases": requirement["cases"],
                    "behavior": requirement["behavior"],
                }));
            }
        }
        for (field, expected) in [
            ("semantic_entrypoints", profile["entrypoints"].clone()),
            ("destination_entrypoints", profile["entrypoints"].clone()),
            ("existing_claim_boundary", profile["claim_boundary"].clone()),
            (
                "current_source_classification",
                profile["classification"].clone(),
            ),
            (
                "acceptance_profiles",
                json!([profile["execution_profile"], "task check"]),
            ),
            ("reviewed_requirements", json!(expected_requirements)),
            (
                "qualification",
                json!("existing-accounting-authority; not-promoted-by-this-mapping"),
            ),
            ("unknown_authority", json!("initial-baseline.json")),
        ] {
            if row.get(field) != Some(&expected) {
                bail!("consumer {id}: changed {field}");
            }
        }
    }
    if let Some(id) = remaining.keys().next() {
        bail!("missing consumer {id}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Value, Value, Value) {
        (
            serde_json::from_str(include_str!("feature-preservation.json")).unwrap(),
            serde_json::from_str(include_str!("profiles.json")).unwrap(),
            serde_json::from_str(include_str!("reviewed-candidates.json")).unwrap(),
        )
    }

    #[test]
    fn actual_mapping_preserves_every_current_consumer_and_reviewed_requirement() {
        let (mapping, profiles, reviewed) = fixture();
        validate(&mapping, &profiles, &reviewed).unwrap();
    }

    #[test]
    fn omitted_duplicate_and_unknown_consumers_refuse() {
        let (mapping, profiles, reviewed) = fixture();
        let mut omitted = mapping.clone();
        let removed = omitted["mapping"].as_array_mut().unwrap().remove(0);
        let error = validate(&omitted, &profiles, &reviewed)
            .unwrap_err()
            .to_string();
        assert!(error.contains("missing consumer"));
        assert!(error.contains(removed["consumer"].as_str().unwrap()));
        let mut duplicate = mapping.clone();
        duplicate["mapping"].as_array_mut().unwrap().push(removed);
        assert!(validate(&duplicate, &profiles, &reviewed)
            .unwrap_err()
            .to_string()
            .contains("duplicate consumer"));
        let mut unknown = mapping;
        unknown["mapping"][0]["consumer"] = json!("unreviewed-consumer");
        assert!(validate(&unknown, &profiles, &reviewed)
            .unwrap_err()
            .to_string()
            .contains("unknown or duplicate consumer unreviewed-consumer"));
    }

    #[test]
    fn entrypoints_execution_and_claim_boundaries_cannot_drift() {
        let (mapping, profiles, reviewed) = fixture();
        for field in [
            "semantic_entrypoints",
            "destination_entrypoints",
            "existing_claim_boundary",
            "current_source_classification",
            "acceptance_profiles",
            "qualification",
            "unknown_authority",
        ] {
            let mut changed = mapping.clone();
            changed["mapping"][0][field] = json!("unreviewed-change");
            assert!(
                validate(&changed, &profiles, &reviewed)
                    .unwrap_err()
                    .to_string()
                    .contains(field),
                "{field}"
            );
        }
    }

    #[test]
    fn reviewed_requirements_cases_and_behavior_cannot_be_dropped_or_changed() {
        let (mapping, profiles, reviewed) = fixture();
        for field in ["id", "cases", "behavior"] {
            let mut changed = mapping.clone();
            changed["mapping"][0]["reviewed_requirements"][0][field] = json!("unreviewed-change");
            assert!(
                validate(&changed, &profiles, &reviewed)
                    .unwrap_err()
                    .to_string()
                    .contains("reviewed_requirements"),
                "{field}"
            );
        }
        let mut dropped = mapping;
        dropped["mapping"][0]["reviewed_requirements"]
            .as_array_mut()
            .unwrap()
            .remove(0);
        assert!(validate(&dropped, &profiles, &reviewed)
            .unwrap_err()
            .to_string()
            .contains("reviewed_requirements"));
    }

    #[test]
    fn changed_source_authorities_require_the_mapping_to_follow() {
        let (mapping, mut profiles, mut reviewed) = fixture();
        profiles[0]["entrypoints"] = json!(["new::entrypoint"]);
        assert!(validate(&mapping, &profiles, &reviewed)
            .unwrap_err()
            .to_string()
            .contains("semantic_entrypoints"));
        let (_, profiles, _) = fixture();
        let id = mapping["mapping"][0]["reviewed_requirements"][0]["id"]
            .as_str()
            .unwrap();
        let requirement = reviewed["requirements"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["id"].as_str() == Some(id))
            .unwrap();
        requirement["behavior"] = json!("new reviewed behavior");
        assert!(validate(&mapping, &profiles, &reviewed)
            .unwrap_err()
            .to_string()
            .contains("reviewed_requirements"));
    }

    #[test]
    fn baseline_identity_and_authority_list_cannot_be_silently_replaced() {
        let (mapping, profiles, reviewed) = fixture();
        for field in ["format", "baseline_revision", "source_authorities"] {
            let mut changed = mapping.clone();
            changed[field] = Value::Null;
            assert!(validate(&changed, &profiles, &reviewed)
                .unwrap_err()
                .to_string()
                .contains(field));
        }
    }
}
