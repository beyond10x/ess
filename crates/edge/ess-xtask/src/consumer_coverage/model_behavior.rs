//! Reviewed native behavior for production model consumers.

use anyhow::{bail, Context, Result};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub(super) const FORMAT: &str = "ess-consumer-model-behavior/1";
pub(super) const EXECUTION_FORMAT: &str = "ess-consumer-model-behavior-execution/1";
pub(super) const CASE_FORMAT: &str = "ess-consumer-model-behavior-case/1";
const TARGET_PROFILE: &str = "x86_64-unknown-linux-gnu/default";
const TOOL_REQUIREMENT: &str = "frozen Rust 1.98.1; locked offline owner-target build";
const NESTED_RUNTIME: &str = "None claimed; direct Rust assertions only";

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct Authority {
    format: String,
    pub cases: BTreeMap<String, Case>,
    pub claims: Vec<Claim>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
#[allow(
    clippy::struct_field_names,
    reason = "case_ast_sha256 is the reviewed authority wire field"
)]
pub(super) struct Case {
    pub identity: CaseIdentity,
    pub target_source: String,
    pub target_source_file_sha256: String,
    pub source: String,
    pub source_file_sha256: String,
    pub case_ast_sha256: String,
    pub reviewed_assertion: String,
    pub attribution_limit: String,
    pub source_evidence: String,
    pub review_reference: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
pub(super) struct CaseIdentity {
    pub package: String,
    pub target_kind: String,
    pub target_name: String,
    pub full_name: String,
    pub features: Vec<String>,
    pub target_profile: String,
    pub tool_requirements: Vec<String>,
    pub nested_runtime: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct Claim {
    pub identity: super::reconciliation::Identity,
    pub cases: Vec<String>,
    pub entrypoint: String,
    pub entrypoint_sha256: String,
    pub entrypoint_source: String,
    pub entrypoint_ast_sha256: String,
    pub entrypoint_source_file_sha256: String,
    pub behavior: Behavior,
    pub source_evidence: String,
    pub review_reference: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(super) enum Behavior {
    Supported { reason: String },
    Refused { reason: String, refusal: String },
}

impl Behavior {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Supported { reason } if !reason.trim().is_empty() => Ok(()),
            Self::Refused { reason, refusal }
                if !reason.trim().is_empty() && !refusal.trim().is_empty() =>
            {
                Ok(())
            }
            _ => bail!("model behavior lacks a precise supported or refused disposition"),
        }
    }
}

pub(super) fn read_authority(bytes: &[u8]) -> Result<Authority> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = UniqueValue.deserialize(&mut deserializer)?;
    deserializer.end()?;
    let authority: Authority = serde_json::from_value(value)?;
    validate_authority(&authority)?;
    Ok(authority)
}

pub(super) fn authority_value(authority: &Authority) -> Value {
    serde_json::to_value(authority).expect("model behavior authority serializes")
}

fn validate_authority(authority: &Authority) -> Result<()> {
    if authority.format != FORMAT || authority.cases.is_empty() {
        bail!("unknown or empty model-behavior authority")
    }
    for (id, case) in &authority.cases {
        let expected = format!(
            "{}:{}:{}:{}",
            case.identity.package,
            case.identity.target_kind,
            case.identity.target_name,
            case.identity.full_name
        );
        if id != &expected
            || case.identity.target_kind != "bin"
            || !case.identity.features.is_empty()
            || case.identity.target_profile != TARGET_PROFILE
            || case.identity.tool_requirements != [TOOL_REQUIREMENT]
            || case.identity.nested_runtime != NESTED_RUNTIME
            || [
                &case.identity.package,
                &case.identity.target_name,
                &case.identity.full_name,
                &case.target_source,
                &case.source,
                &case.reviewed_assertion,
                &case.attribution_limit,
                &case.source_evidence,
                &case.review_reference,
            ]
            .iter()
            .any(|value| value.trim().is_empty())
        {
            bail!("unreviewed executable case contract {id}")
        }
        for digest in [
            &case.target_source_file_sha256,
            &case.source_file_sha256,
            &case.case_ast_sha256,
        ] {
            validate_digest(digest)?;
        }
    }
    let mut claimed = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for claim in &authority.claims {
        if !identities.insert(claim.identity.clone())
            || claim.cases.is_empty()
            || claim.cases.iter().collect::<BTreeSet<_>>().len() != claim.cases.len()
            || [
                &claim.entrypoint,
                &claim.entrypoint_source,
                &claim.source_evidence,
                &claim.review_reference,
                &claim.identity.model,
                &claim.identity.shape,
                &claim.identity.consumer,
                &claim.identity.profile,
            ]
            .iter()
            .any(|value| value.trim().is_empty())
        {
            bail!("duplicate or incomplete model-behavior claim")
        }
        for digest in [
            &claim.identity.shape,
            &claim.identity.profile,
            &claim.entrypoint_sha256,
            &claim.entrypoint_ast_sha256,
            &claim.entrypoint_source_file_sha256,
        ] {
            validate_digest(digest)?;
        }
        claim.behavior.validate()?;
        for case in &claim.cases {
            if !authority.cases.contains_key(case) {
                bail!("model-behavior claim names unknown case {case}")
            }
            claimed.insert(case.clone());
        }
    }
    if claimed != authority.cases.keys().cloned().collect() {
        bail!("model-behavior case/claim partition is incomplete")
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
enum ExecutionFormat {
    #[serde(rename = "ess-consumer-model-behavior-execution/1")]
    V1,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
enum CandidateStage {
    #[serde(rename = "candidate")]
    Candidate,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
enum PlanStage {
    #[serde(rename = "execution-plan")]
    Plan,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
enum ExecutedStage {
    #[serde(rename = "executed")]
    Executed,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct Candidate {
    format: ExecutionFormat,
    stage: CandidateStage,
    authority_sha256: String,
    source_sha256: String,
    provider_profile_sha256: String,
    selected_cases: Vec<String>,
    cases: BTreeMap<String, Case>,
    claims: Vec<Claim>,
    qualified_cells: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct Plan {
    format: ExecutionFormat,
    stage: PlanStage,
    authority_sha256: String,
    source_sha256: String,
    provider_profile_sha256: String,
    selected_cases: Vec<String>,
    cases: BTreeMap<String, Case>,
    claims: Vec<Claim>,
    qualified_cells: usize,
    candidate_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct Executed {
    format: ExecutionFormat,
    stage: ExecutedStage,
    authority_sha256: String,
    source_sha256: String,
    provider_profile_sha256: String,
    selected_cases: Vec<String>,
    cases: BTreeMap<String, Case>,
    claims: Vec<Claim>,
    qualified_cells: usize,
    candidate_sha256: String,
    plan_sha256: String,
    receipts: BTreeMap<String, Receipt>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct Receipt {
    pub format: String,
    pub authority_sha256: String,
    pub plan_sha256: String,
    pub identity: CaseIdentity,
    pub target_source: String,
    pub target_source_file_sha256: String,
    pub source: String,
    pub source_file_sha256: String,
    pub case_ast_sha256: String,
    pub native: Value,
    pub source_sha256: String,
    pub provider_profile_sha256: String,
    pub executed: usize,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub nested_runtime: String,
}

pub(super) fn candidate(
    authority: &Authority,
    models: &Value,
    source_profile: &Value,
    profiles: &Value,
    inventory: &Value,
    metadata: &Value,
) -> Result<Value> {
    validate_authority(authority)?;
    let models = models
        .as_object()
        .context("current model-behavior obligations")?;
    let sources = source_profile["source"]
        .as_object()
        .context("model-behavior source profile")?;
    let packages = inventory["packages"]
        .as_object()
        .context("model-behavior consumer packages")?;
    for (id, reviewed) in &authority.cases {
        if sources.get(&reviewed.target_source)
            != Some(&Value::String(reviewed.target_source_file_sha256.clone()))
            || sources.get(&reviewed.source)
                != Some(&Value::String(reviewed.source_file_sha256.clone()))
        {
            bail!("model-behavior case source bytes changed: {id}")
        }
        let actual = packages
            .get(&reviewed.identity.package)
            .and_then(|package| package["cases"].get(id))
            .with_context(|| format!("missing model-behavior source case {id}"))?;
        let features_match = actual.get("required_features").is_some_and(|required| {
            (required.is_null() && reviewed.identity.features.is_empty())
                || required == &serde_json::json!(reviewed.identity.features)
        });
        if actual["package"] != reviewed.identity.package
            || actual["target_kind"] != reviewed.identity.target_kind
            || actual["target_name"] != reviewed.identity.target_name
            || actual["full_name"] != reviewed.identity.full_name
            || actual["source"] != reviewed.source
            || actual["case_ast_sha256"] != reviewed.case_ast_sha256
            || !features_match
            || actual["ignored"] != false
            || actual["contains_early_return"] != false
            || actual["contains_catch_unwind"] != false
            || actual["nested_command_candidates"] != serde_json::json!([])
            || actual["assertion_candidates"]
                .as_array()
                .is_none_or(Vec::is_empty)
        {
            bail!("model-behavior source case differs from reviewed authority: {id}")
        }
        validate_target(metadata, reviewed)?;
    }
    for claim in &authority.claims {
        if models.get(&claim.identity.model) != Some(&Value::String(claim.identity.shape.clone())) {
            bail!(
                "model-behavior model is absent or stale: {}",
                claim.identity.model
            )
        }
        let profile = profiles.get(&claim.identity.consumer).with_context(|| {
            format!("missing model-behavior profile {}", claim.identity.consumer)
        })?;
        if profile["profile_sha256"] != claim.identity.profile
            || profile["definition"]["entrypoints"]
                .as_array()
                .is_none_or(|entries| entries != &[Value::String(claim.entrypoint.clone())])
        {
            bail!(
                "model-behavior consumer profile changed: {}",
                claim.identity.consumer
            )
        }
        let entry = packages
            .values()
            .filter_map(|package| package["entries"].get(&claim.entrypoint))
            .collect::<Vec<_>>();
        let [entry] = entry.as_slice() else {
            bail!(
                "model-behavior entrypoint is absent or ambiguous: {}",
                claim.entrypoint
            )
        };
        if entry["source"] != claim.entrypoint_source
            || entry["declaration_sha256"] != claim.entrypoint_sha256
            || entry["source_item_sha256"] != claim.entrypoint_ast_sha256
            || sources.get(&claim.entrypoint_source)
                != Some(&Value::String(claim.entrypoint_source_file_sha256.clone()))
        {
            bail!("model-behavior entrypoint changed: {}", claim.entrypoint)
        }
    }
    let selected_cases = authority.cases.keys().cloned().collect::<Vec<_>>();
    Ok(serde_json::to_value(Candidate {
        format: ExecutionFormat::V1,
        stage: CandidateStage::Candidate,
        authority_sha256: super::hash_json(&authority_value(authority)),
        source_sha256: super::hash_json(&source_profile["source"]),
        provider_profile_sha256: super::hash_json(source_profile),
        selected_cases,
        cases: authority.cases.clone(),
        claims: authority.claims.clone(),
        qualified_cells: 0,
    })?)
}

fn validate_target(metadata: &Value, reviewed: &Case) -> Result<()> {
    let matches = metadata["packages"]
        .as_array()
        .context("Cargo metadata packages")?
        .iter()
        .filter(|package| package["name"] == reviewed.identity.package)
        .flat_map(|package| package["targets"].as_array().into_iter().flatten())
        .filter(|target| {
            let features_match = target.get("required-features").map_or_else(
                || reviewed.identity.features.is_empty(),
                |required| required == &serde_json::json!(reviewed.identity.features),
            );
            target["name"] == reviewed.identity.target_name
                && target["kind"] == serde_json::json!([reviewed.identity.target_kind])
                && features_match
                && target["src_path"]
                    .as_str()
                    .is_some_and(|path| path.ends_with(&reviewed.target_source))
        })
        .count();
    if matches != 1 {
        bail!(
            "model-behavior target is absent or ambiguous: {}:{}:{}",
            reviewed.identity.package,
            reviewed.identity.target_kind,
            reviewed.identity.target_name
        )
    }
    Ok(())
}

pub(super) fn plan(
    proposed: &Value,
    authority: &Authority,
    models: &Value,
    source_profile: &Value,
    profiles: &Value,
    inventory: &Value,
    metadata: &Value,
) -> Result<Value> {
    let proposed = read_candidate(proposed)?;
    let fresh_value = candidate(
        authority,
        models,
        source_profile,
        profiles,
        inventory,
        metadata,
    )?;
    let fresh: Candidate = serde_json::from_value(fresh_value.clone())?;
    if proposed != fresh {
        bail!("model-behavior candidate is stale or changed")
    }
    Ok(serde_json::to_value(Plan {
        format: fresh.format,
        stage: PlanStage::Plan,
        authority_sha256: fresh.authority_sha256,
        source_sha256: fresh.source_sha256,
        provider_profile_sha256: fresh.provider_profile_sha256,
        selected_cases: fresh.selected_cases,
        cases: fresh.cases,
        claims: fresh.claims,
        qualified_cells: 0,
        candidate_sha256: super::hash_json(&fresh_value),
    })?)
}

pub(super) fn read_candidate(value: &Value) -> Result<Candidate> {
    let candidate: Candidate = serde_json::from_value(value.clone())?;
    if candidate.qualified_cells != 0
        || candidate.selected_cases != candidate.cases.keys().cloned().collect::<Vec<_>>()
    {
        bail!("invalid model-behavior candidate")
    }
    for digest in [
        &candidate.authority_sha256,
        &candidate.source_sha256,
        &candidate.provider_profile_sha256,
    ] {
        validate_digest(digest)?;
    }
    if candidate.cases.is_empty() || candidate.claims.is_empty() {
        bail!("empty model-behavior candidate")
    }
    Ok(candidate)
}

pub(super) fn required_cases(plan: &Value) -> Result<BTreeMap<String, Case>> {
    let plan: Plan = serde_json::from_value(plan.clone())?;
    validate_plan_shape(&plan)?;
    Ok(plan.cases)
}

pub(super) fn claims(plan: &Value) -> Result<Vec<Claim>> {
    let plan: Plan = serde_json::from_value(plan.clone())?;
    validate_plan_shape(&plan)?;
    Ok(plan.claims)
}

pub(super) fn case_ids(plan: &Value) -> Result<BTreeSet<String>> {
    Ok(required_cases(plan)?.into_keys().collect())
}

fn validate_plan_shape(plan: &Plan) -> Result<()> {
    if plan.qualified_cells != 0
        || plan.selected_cases != plan.cases.keys().cloned().collect::<Vec<_>>()
    {
        bail!("invalid model-behavior execution plan")
    }
    let mut identities = BTreeSet::new();
    let mut claimed_cases = BTreeSet::new();
    for claim in &plan.claims {
        if !identities.insert(&claim.identity)
            || claim.cases.is_empty()
            || claim.cases.iter().collect::<BTreeSet<_>>().len() != claim.cases.len()
        {
            bail!("invalid model-behavior execution-plan claim")
        }
        claim.behavior.validate()?;
        for case in &claim.cases {
            if !plan.cases.contains_key(case) {
                bail!("model-behavior execution-plan claim names unknown case {case}")
            }
            claimed_cases.insert(case.clone());
        }
    }
    if claimed_cases != plan.cases.keys().cloned().collect() {
        bail!("model-behavior execution-plan case/claim partition is incomplete")
    }
    for digest in [
        &plan.authority_sha256,
        &plan.source_sha256,
        &plan.provider_profile_sha256,
        &plan.candidate_sha256,
    ] {
        validate_digest(digest)?;
    }
    Ok(())
}

pub(super) fn executed(plan_value: &Value, receipts: BTreeMap<String, Receipt>) -> Result<Value> {
    let plan: Plan = serde_json::from_value(plan_value.clone())?;
    validate_plan_shape(&plan)?;
    if receipts.keys().ne(plan.cases.keys()) {
        bail!("model-behavior receipts do not cover the exact selected cases")
    }
    Ok(serde_json::to_value(Executed {
        format: plan.format,
        stage: ExecutedStage::Executed,
        authority_sha256: plan.authority_sha256,
        source_sha256: plan.source_sha256,
        provider_profile_sha256: plan.provider_profile_sha256,
        selected_cases: plan.selected_cases,
        cases: plan.cases,
        claims: plan.claims,
        qualified_cells: 0,
        candidate_sha256: plan.candidate_sha256,
        plan_sha256: super::hash_json(plan_value),
        receipts,
    })?)
}

pub(super) fn verify_execution(
    execution_value: &Value,
    plan_value: &Value,
    candidate_value: &Value,
) -> Result<Vec<Claim>> {
    let execution: Executed = serde_json::from_value(execution_value.clone())?;
    let plan: Plan = serde_json::from_value(plan_value.clone())?;
    let candidate: Candidate = serde_json::from_value(candidate_value.clone())?;
    validate_plan_shape(&plan)?;
    if execution.qualified_cells != 0
        || execution.authority_sha256 != plan.authority_sha256
        || execution.source_sha256 != plan.source_sha256
        || execution.provider_profile_sha256 != plan.provider_profile_sha256
        || execution.selected_cases != plan.selected_cases
        || execution.cases != plan.cases
        || execution.claims != plan.claims
        || execution.candidate_sha256 != plan.candidate_sha256
        || execution.plan_sha256 != super::hash_json(plan_value)
        || execution.candidate_sha256 != super::hash_json(candidate_value)
        || candidate.authority_sha256 != plan.authority_sha256
        || candidate.source_sha256 != plan.source_sha256
        || candidate.provider_profile_sha256 != plan.provider_profile_sha256
        || candidate.cases != plan.cases
        || candidate.claims != plan.claims
        || candidate.selected_cases != plan.selected_cases
        || candidate.qualified_cells != 0
    {
        bail!("model-behavior execution does not bind the exact candidate and plan")
    }
    for (id, case) in &plan.cases {
        let receipt = execution
            .receipts
            .get(id)
            .with_context(|| format!("missing model-behavior receipt {id}"))?;
        if receipt.format != CASE_FORMAT
            || receipt.authority_sha256 != plan.authority_sha256
            || receipt.plan_sha256 != execution.plan_sha256
            || receipt.identity != case.identity
            || receipt.target_source != case.target_source
            || receipt.target_source_file_sha256 != case.target_source_file_sha256
            || receipt.source != case.source
            || receipt.source_file_sha256 != case.source_file_sha256
            || receipt.case_ast_sha256 != case.case_ast_sha256
            || receipt.source_sha256 != plan.source_sha256
            || receipt.provider_profile_sha256 != plan.provider_profile_sha256
            || (
                receipt.executed,
                receipt.passed,
                receipt.failed,
                receipt.ignored,
            ) != (1, 1, 0, 0)
            || receipt.nested_runtime != "none claimed"
            || !valid_native(&receipt.native)
        {
            bail!("invalid model-behavior receipt {id}")
        }
    }
    if execution.receipts.keys().ne(plan.cases.keys()) {
        bail!("model-behavior execution has missing or extra receipts")
    }
    Ok(plan.claims)
}

fn valid_native(native: &Value) -> bool {
    let Some(object) = native.as_object() else {
        return false;
    };
    let expected = [
        "copy",
        "original_device",
        "original_inode",
        "original_mode",
        "sha256",
        "size",
        "source",
    ];
    object.keys().map(String::as_str).eq(expected)
        && object["source"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
        && object["copy"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
        && object["sha256"]
            .as_str()
            .is_some_and(|value| validate_digest(value).is_ok())
        && object["size"].as_u64().is_some_and(|value| value > 0)
        && object["original_mode"]
            .as_u64()
            .is_some_and(|value| value & 0o111 != 0)
        && object["original_device"].as_u64().is_some()
        && object["original_inode"].as_u64().is_some()
}

fn validate_digest(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("model-behavior digest is not 64 lowercase hexadecimal characters")
    }
    Ok(())
}

struct UniqueValue;

impl<'de> DeserializeSeed<'de> for UniqueValue {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

struct UniqueValueVisitor;

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> std::result::Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> std::result::Result<Value, E> {
        Ok(value.into())
    }

    fn visit_u64<E>(self, value: u64) -> std::result::Result<Value, E> {
        Ok(value.into())
    }

    fn visit_f64<E>(self, value: f64) -> std::result::Result<Value, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> std::result::Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> std::result::Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        UniqueValue.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> std::result::Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(UniqueValue)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object key `{key}`"
                )));
            }
            values.insert(key, map.next_value_seed(UniqueValue)?);
        }
        Ok(Value::Object(values))
    }
}
