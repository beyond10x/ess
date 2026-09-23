//! Eight exact scenario-acquisition obligations, separate from model-cell accounting.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const FORMAT: &str = "ess-consumer-scenario-acquisition/2";
const CANDIDATE_FORMAT: &str = "ess-consumer-scenario-acquisition-candidate/2";
const PLAN_FORMAT: &str = "ess-consumer-scenario-acquisition-plan/2";
const PROOF_FORMAT: &str = "ess-consumer-scenario-acquisition-proof/2";
const ENTRYPOINT: &str = "ess_cli::bin(ess)::input_discovery::fn::authored";
const ENTRYPOINT_SOURCE: &str = "crates/edge/ess-cli/src/input_discovery.rs";
const TARGET_SOURCE: &str = "crates/edge/ess-cli/src/main.rs";
const CASE_SOURCE: &str = "crates/edge/ess-cli/src/input_discovery_accounting_tests.rs";

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
enum Role {
    Authored,
    Coverage,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
enum Mode {
    Manifest,
    LegacyDirectory,
    DirectFile,
    Omitted,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
pub(super) struct Case {
    pub id: String,
    pub package: String,
    pub target_kind: TargetKind,
    pub target_name: String,
    pub target_source: String,
    #[serde(rename = "case_source")]
    pub source: String,
    pub full_name: String,
    #[serde(rename = "case_ast_sha256")]
    pub ast_sha256: String,
    pub source_file_sha256: String,
    pub features: Vec<String>,
    pub target_profile: String,
    pub tool_requirements: Vec<String>,
    pub nested_runtime: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum TargetKind {
    #[serde(rename = "bin")]
    BinaryUnit,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
struct Row {
    profile: String,
    profile_sha256: String,
    entrypoint: String,
    entrypoint_sha256: String,
    entrypoint_source: String,
    entrypoint_ast_sha256: String,
    entrypoint_source_file_sha256: String,
    role: Role,
    mode: Mode,
    case: Case,
    source_reference: String,
    review_reference: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidates {
    format: String,
    rows: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Authority {
    format: String,
    rows: Vec<Row>,
}

fn expected() -> BTreeMap<String, (Role, Mode, String)> {
    let mut rows = BTreeMap::new();
    for (role_id, role) in [("authored", Role::Authored), ("coverage", Role::Coverage)] {
        for (mode_id, mode) in [
            ("manifest", Mode::Manifest),
            ("legacy-directory", Mode::LegacyDirectory),
            ("direct-file", Mode::DirectFile),
            ("omitted-scenarios", Mode::Omitted),
        ] {
            let profile = format!("acquisition-{role_id}-{mode_id}");
            let full_name = format!(
                "input_discovery::accounting_tests::acquisition_{role_id}_{}",
                mode_id.replace('-', "_")
            );
            rows.insert(profile, (role.clone(), mode, full_name));
        }
    }
    rows
}

fn validate_rows(rows: &[Row]) -> Result<BTreeSet<String>> {
    let mut expected = expected();
    let mut cases = BTreeSet::new();
    for row in rows {
        let (role, mode, full_name) = expected
            .remove(&row.profile)
            .with_context(|| format!("unknown or duplicate acquisition profile {}", row.profile))?;
        if row.role != role
            || row.mode != mode
            || row.entrypoint != ENTRYPOINT
            || row.profile_sha256.trim().is_empty()
            || row.entrypoint_sha256.trim().is_empty()
            || row.entrypoint_source != ENTRYPOINT_SOURCE
            || row.entrypoint_ast_sha256.trim().is_empty()
            || row.entrypoint_source_file_sha256.trim().is_empty()
            || row.source_reference.trim().is_empty()
            || row.review_reference.trim().is_empty()
        {
            bail!("stale role, mode, identity or evidence for {}", row.profile);
        }
        let case = &row.case;
        if case.package != "ess-cli"
            || case.target_kind != TargetKind::BinaryUnit
            || case.target_name != "ess"
            || case.target_source != TARGET_SOURCE
            || case.source != CASE_SOURCE
            || case.full_name != full_name
            || case.ast_sha256.trim().is_empty()
            || case.source_file_sha256.trim().is_empty()
            || !case.features.is_empty()
            || case.target_profile != "x86_64-unknown-linux-gnu/default"
            || case.tool_requirements != ["frozen Rust 1.98.1; locked offline owner-target build"]
            || case.nested_runtime != "None claimed; direct Rust assertions only"
        {
            bail!("unreviewed acquisition owner-case contract {}", row.profile);
        }
        if !cases.insert(case.id.clone()) {
            bail!("duplicate acquisition case identity {}", case.id);
        }
    }
    if let Some(missing) = expected.keys().next() {
        bail!("missing acquisition profile {missing}");
    }
    Ok(cases)
}

pub(super) fn candidates(
    profiles: &BTreeMap<String, Value>,
    entries: &BTreeMap<String, Value>,
    cases: &BTreeMap<String, Value>,
    source: &BTreeMap<String, String>,
) -> Result<Value> {
    let expected = expected();
    let actual_profiles = profiles
        .iter()
        .filter(|(_, row)| row["definition"]["classification"] == "scenario-acquisition")
        .map(|(profile, _)| profile.clone())
        .collect::<BTreeSet<_>>();
    let expected_profiles = expected.keys().cloned().collect::<BTreeSet<_>>();
    if actual_profiles != expected_profiles {
        bail!(
            "scenario-acquisition candidate inventory differs from the exact reviewed set: expected {expected_profiles:?}, found {actual_profiles:?}"
        );
    }
    let entry = entries
        .get(ENTRYPOINT)
        .context("missing exact scenario-acquisition entrypoint")?;
    let entrypoint_sha256 = entry["declaration_sha256"]
        .as_str()
        .context("scenario-acquisition entrypoint declaration hash")?;
    if entry["source"] != ENTRYPOINT_SOURCE {
        bail!("scenario-acquisition entrypoint source route changed");
    }
    let entrypoint_ast_sha256 = entry["source_item_sha256"]
        .as_str()
        .context("scenario-acquisition entrypoint full AST hash")?;
    let entrypoint_source_file_sha256 = source
        .get(ENTRYPOINT_SOURCE)
        .context("scenario-acquisition entrypoint is outside complete source authority")?;
    let mut rows = Vec::new();
    for (profile, (role, mode, full_name)) in expected {
        let definition = profiles
            .get(&profile)
            .with_context(|| format!("missing scenario-acquisition profile {profile}"))?;
        if definition["definition"]["classification"] != "scenario-acquisition"
            || definition["definition"]["entrypoints"] != json!([ENTRYPOINT])
        {
            bail!("scenario-acquisition profile remains a model consumer: {profile}");
        }
        let case_id = format!("ess-cli:bin:ess:{full_name}");
        let actual = cases
            .get(&case_id)
            .with_context(|| format!("missing direct acquisition owner case {case_id}"))?;
        if actual["source"] != CASE_SOURCE
            || actual["package"] != "ess-cli"
            || actual["target_kind"] != "bin"
            || actual["target_name"] != "ess"
            || actual["full_name"] != full_name
            || actual["ignored"] != false
            || actual["contains_early_return"] != false
            || actual["contains_catch_unwind"] != false
            || actual["nested_command_candidates"] != json!([])
            || actual["assertion_candidates"]
                .as_array()
                .is_none_or(Vec::is_empty)
        {
            bail!("direct acquisition owner-case source contract changed: {case_id}");
        }
        let case_ast_sha256 = actual["case_ast_sha256"]
            .as_str()
            .context("acquisition case AST hash")?;
        let source_file_sha256 = source
            .get(CASE_SOURCE)
            .context("acquisition case source is outside complete source authority")?;
        rows.push(Row {
            profile,
            profile_sha256: definition["profile_sha256"]
                .as_str()
                .context("acquisition profile hash")?
                .to_owned(),
            entrypoint: ENTRYPOINT.to_owned(),
            entrypoint_sha256: entrypoint_sha256.to_owned(),
            entrypoint_source: ENTRYPOINT_SOURCE.to_owned(),
            entrypoint_ast_sha256: entrypoint_ast_sha256.to_owned(),
            entrypoint_source_file_sha256: entrypoint_source_file_sha256.to_owned(),
            role,
            mode,
            case: Case {
                id: case_id,
                package: "ess-cli".into(),
                target_kind: TargetKind::BinaryUnit,
                target_name: "ess".into(),
                target_source: TARGET_SOURCE.into(),
                source: CASE_SOURCE.into(),
                full_name,
                ast_sha256: case_ast_sha256.to_owned(),
                source_file_sha256: source_file_sha256.to_owned(),
                features: Vec::new(),
                target_profile: "x86_64-unknown-linux-gnu/default".into(),
                tool_requirements: vec![
                    "frozen Rust 1.98.1; locked offline owner-target build".into()
                ],
                nested_runtime: "None claimed; direct Rust assertions only".into(),
            },
            source_reference: CASE_SOURCE.into(),
            review_reference: "story:consumer-accounting-baseline-never-extended".into(),
        });
    }
    validate_rows(&rows)?;
    Ok(json!({"format":CANDIDATE_FORMAT,"rows":rows}))
}

pub(super) fn plan(candidate_value: &Value, authority_value: &Value) -> Result<Value> {
    let candidates: Candidates = serde_json::from_value(candidate_value.clone())?;
    let authority: Authority = serde_json::from_value(authority_value.clone())?;
    if candidates.format != CANDIDATE_FORMAT || authority.format != FORMAT {
        bail!("unknown scenario-acquisition format");
    }
    let candidate_cases = validate_rows(&candidates.rows)?;
    let authority_cases = validate_rows(&authority.rows)?;
    if candidates.rows != authority.rows || candidate_cases != authority_cases {
        bail!("reviewed acquisition rows differ from current exact candidates");
    }
    Ok(json!({
        "format": PLAN_FORMAT,
        "status": "PENDING_CASE_AND_GUARD_EXECUTION",
        "candidate_sha256": super::hash_json(candidate_value),
        "authority_sha256": super::hash_json(authority_value),
        "required_rows": candidates.rows.len(),
        "qualified_rows": 0,
        "required_cases": candidate_cases,
        "rows": candidates.rows,
    }))
}

pub(super) fn profile_identities(profile_output: &Value) -> Result<Value> {
    let profiles = profile_output
        .as_object()
        .context("consumer profile output object")?;
    let actual_profiles = profiles
        .iter()
        .filter(|(_, row)| row["definition"]["classification"] == "scenario-acquisition")
        .map(|(profile, _)| profile.clone())
        .collect::<BTreeSet<_>>();
    let expected_profiles = expected().into_keys().collect::<BTreeSet<_>>();
    if actual_profiles != expected_profiles {
        bail!(
            "scenario-acquisition profile inventory differs from the exact reviewed set: expected {expected_profiles:?}, found {actual_profiles:?}"
        );
    }
    let mut result = BTreeMap::new();
    for profile in expected().keys() {
        let row = profiles
            .get(profile)
            .with_context(|| format!("missing acquisition profile output {profile}"))?;
        if row["definition"]["classification"] != "scenario-acquisition" {
            bail!("acquisition profile remains in model-cell accounting: {profile}");
        }
        result.insert(
            profile.clone(),
            row["profile_sha256"]
                .as_str()
                .context("acquisition profile hash")?
                .to_owned(),
        );
    }
    Ok(json!(result))
}

pub(super) fn required_cases(plan: &Value) -> Result<BTreeMap<String, Case>> {
    if plan["format"] != PLAN_FORMAT {
        bail!("unknown scenario-acquisition plan format");
    }
    let rows: Vec<Row> = serde_json::from_value(plan["rows"].clone())?;
    let mut cases = BTreeMap::new();
    for row in rows {
        if cases.insert(row.case.id.clone(), row.case).is_some() {
            bail!("duplicate planned acquisition case");
        }
    }
    Ok(cases)
}

pub(super) fn execution_proof(plan: &Value, executed: &BTreeSet<String>) -> Result<Value> {
    if plan["format"] != PLAN_FORMAT {
        bail!("unknown scenario-acquisition plan format");
    }
    let required: BTreeSet<String> = serde_json::from_value(plan["required_cases"].clone())?;
    if &required != executed {
        bail!("executed acquisition cases differ from the planned exact set");
    }
    Ok(json!({
        "format":PROOF_FORMAT,
        "candidate_sha256":plan["candidate_sha256"],
        "authority_sha256":plan["authority_sha256"],
        "guard_executed":1,
        "planned_rows_sha256":super::hash_json(&plan["rows"]),
        "proved_rows_sha256":super::hash_json(&plan["rows"]),
        "executed_cases":executed,
    }))
}

#[cfg(test)]
pub(super) fn test_proof(plan: &Value) -> Result<Value> {
    execution_proof(
        plan,
        &serde_json::from_value(plan["required_cases"].clone())?,
    )
}

pub(super) fn qualify(plan: &Value, proof: &Value) -> Result<Value> {
    let fields = proof.as_object().context("acquisition proof object")?;
    if plan["format"] != PLAN_FORMAT
        || fields.len() != 7
        || proof["format"] != PROOF_FORMAT
        || proof["candidate_sha256"] != plan["candidate_sha256"]
        || proof["authority_sha256"] != plan["authority_sha256"]
        || proof["guard_executed"] != 1
        || proof["planned_rows_sha256"] != super::hash_json(&plan["rows"])
        || proof["proved_rows_sha256"] != super::hash_json(&plan["rows"])
    {
        bail!("stale, incomplete or caller-forged acquisition guard proof");
    }
    let planned: BTreeSet<String> = serde_json::from_value(plan["required_cases"].clone())?;
    let executed: BTreeSet<String> = serde_json::from_value(proof["executed_cases"].clone())?;
    if planned != executed {
        bail!("acquisition execution differs from the exact planned case set");
    }
    Ok(json!({
        "format":FORMAT,
        "status":"QUALIFIED_EXACT_ACQUISITION",
        "qualified_rows":plan["required_rows"],
        "executed_cases":executed.len(),
        "guard_executed":1,
        "rows":plan["rows"],
    }))
}
