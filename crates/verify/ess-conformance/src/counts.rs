//! Explicit count-stage report/2 and detailed run/2, paired with original suite bytes.
use crate::{
    admission::{AdmissionError, AdmittedSuite},
    count_json::Json,
    coverage::{Counts as CoverageCounts, Knowledge, Refusal, Selection, SuiteReference},
    ConformanceReport, ExecutedRun, ScenarioId, ScenarioResult, Status,
};
use ess_primitives::evidence::SpecDigest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The opt-in standalone count format.
pub const COUNT_REPORT_FORMAT: &str = "ess-conformance-report/2";
/// The separate opt-in detailed format.
pub const COUNT_RUN_FORMAT: &str = "ess-conformance-run/2";

/// Execution and conformance are separate instances of this vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CountStatus {
    /// Every executed scenario passed (qualification additionally requires known coverage).
    Passed,
    /// Execution contradicted the producer's required checks.
    Failed,
    /// Execution or coverage is inconclusive.
    Inconclusive,
}
/// Producer-specific terminal outcome semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProducerProfile {
    /// Rust error/unsupported semantics; skipped is unavailable.
    #[serde(rename = "rust-scenario-status/1")]
    Rust,
    /// Go skipped semantics; errors and unsupported are unavailable categories.
    #[serde(rename = "go-scenario-status/1")]
    Go,
}
/// Five terminal categories and their checked sum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ScenarioCounts {
    /// Every selected terminal result.
    pub total: u64,
    /// Final passes.
    pub passed: u64,
    /// Final failures only.
    pub failed: u64,
    /// Rust runner/target errors.
    pub error: u64,
    /// Rust unsupported observations.
    pub unsupported: u64,
    /// Go skipped scenarios.
    pub skipped: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Outcomes {
    passed: Vec<ScenarioId>,
    failed: Vec<ScenarioId>,
    error: Vec<ScenarioId>,
    unsupported: Vec<ScenarioId>,
    skipped: Vec<ScenarioId>,
}
impl Outcomes {
    fn arrays(&self) -> [&[ScenarioId]; 5] {
        [
            &self.passed,
            &self.failed,
            &self.error,
            &self.unsupported,
            &self.skipped,
        ]
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnknownCoverage {
    knowledge: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InventorySummary {
    knowledge: Knowledge,
    selection: Selection,
    counts: CoverageCounts,
    refused: Vec<Refusal>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum ReportCoverage {
    Inventory(InventorySummary),
    Legacy(UnknownCoverage),
}
impl ReportCoverage {
    fn of(admitted: &AdmittedSuite) -> Self {
        admitted.coverage().map_or_else(
            || {
                Self::Legacy(UnknownCoverage {
                    knowledge: "unknown".into(),
                })
            },
            |c| {
                Self::Inventory(InventorySummary {
                    knowledge: c.knowledge,
                    selection: c.selection.clone(),
                    counts: c.counts.clone(),
                    refused: c.refused.clone(),
                })
            },
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireReport {
    format: String,
    specification: String,
    spec_digest: SpecDigest,
    implementation: String,
    producer_profile: ProducerProfile,
    suite: SuiteReference,
    execution_status: CountStatus,
    counts: ScenarioCounts,
    outcomes: Outcomes,
    coverage: ReportCoverage,
    conformance_status: CountStatus,
    policy: String,
    completed_at: u64,
}
/// A coherent count report paired with the exact original suite. Unknown coverage never qualifies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct CountReport(WireReport);

fn error(detail: impl Into<String>) -> AdmissionError {
    AdmissionError::new("InvalidCountReport", "$", detail)
}
fn length<T>(values: &[T]) -> Result<u64, AdmissionError> {
    u64::try_from(values.len()).map_err(|e| error(e.to_string()))
}
fn canonical(value: &impl Serialize) -> Result<String, AdmissionError> {
    // serde_json's ordered map sorts all typed and payload object keys while retaining numeric types.
    let mut value = serde_json::to_value(value).map_err(|e| error(e.to_string()))?;
    value.sort_all_objects();
    let mut text = serde_json::to_string_pretty(&value).map_err(|e| error(e.to_string()))?;
    text.push('\n');
    Ok(text)
}
impl CountReport {
    /// Publish complete Rust results only for the exact admitted suite bytes actually executed.
    pub fn from_run(run: &ExecutedRun, admitted: &AdmittedSuite) -> Result<Self, AdmissionError> {
        if run.suite_digest() != admitted.digest() {
            return Err(error("run executed different admitted suite bytes"));
        }
        if run.suite != admitted.suite().provenance {
            return Err(error("run provenance differs from admitted suite"));
        }
        if run.status != ConformanceReport::verdict(&run.scenarios) {
            return Err(error("run verdict contradicts terminal results"));
        }
        let mut outcomes = Outcomes::default();
        for result in &run.scenarios {
            let ids = match result.status {
                Status::Passed => &mut outcomes.passed,
                Status::Failed => &mut outcomes.failed,
                Status::Error => &mut outcomes.error,
                Status::Unsupported => &mut outcomes.unsupported,
            };
            ids.push(result.scenario.clone());
        }
        for ids in [
            &mut outcomes.passed,
            &mut outcomes.failed,
            &mut outcomes.error,
            &mut outcomes.unsupported,
        ] {
            ids.sort();
        }
        let counts = ScenarioCounts {
            total: length(&run.scenarios)?,
            passed: length(&outcomes.passed)?,
            failed: length(&outcomes.failed)?,
            error: length(&outcomes.error)?,
            unsupported: length(&outcomes.unsupported)?,
            skipped: 0,
        };
        let execution_status = execution(ProducerProfile::Rust, &counts)?;
        let result = Self(WireReport {
            format: COUNT_REPORT_FORMAT.into(),
            specification: format!("{}/{}", run.suite.system, run.suite.specification_version),
            spec_digest: run.suite.spec_digest.clone(),
            implementation: run.implementation.to_string(),
            producer_profile: ProducerProfile::Rust,
            suite: SuiteReference {
                version: run.suite.suite_version.to_string(),
                digest_profile: "sha256-json-bytes/1".into(),
                digest: admitted.digest().into(),
            },
            execution_status,
            counts,
            outcomes,
            coverage: ReportCoverage::of(admitted),
            conformance_status: qualification(execution_status, admitted),
            policy: "complete-selection/1".into(),
            completed_at: run.completed_at.epoch_millis(),
        });
        result.validate(admitted)?;
        Ok(result)
    }
    /// Parse original report bytes and check them against an admitted exact suite.
    pub fn from_json(text: &str, admitted: &AdmittedSuite) -> Result<Self, AdmissionError> {
        let raw = Json::parse(text, "$")?;
        Self::from_raw(&raw, admitted)
    }
    fn from_raw(raw: &Json, admitted: &AdmittedSuite) -> Result<Self, AdmissionError> {
        let fields = raw.object()?;
        fields
            .get("completed_at")
            .ok_or_else(|| error("missing completed_at"))?
            .unsigned()?;
        let counts = fields
            .get("counts")
            .ok_or_else(|| error("missing counts"))?
            .object()?;
        for count in counts.values() {
            count.unsigned()?;
        }
        let coverage = fields
            .get("coverage")
            .ok_or_else(|| error("missing coverage"))?;
        crate::coverage::validate_summary_json(coverage, admitted.coverage().is_some())?;
        let wire = serde_json::from_str(&raw.raw).map_err(|e| error(e.to_string()))?;
        let result = Self(wire);
        result.validate(admitted)?;
        Ok(result)
    }
    fn validate(&self, admitted: &AdmittedSuite) -> Result<(), AdmissionError> {
        let r = &self.0;
        if r.format != COUNT_REPORT_FORMAT
            || r.policy != "complete-selection/1"
            || r.coverage != ReportCoverage::of(admitted)
        {
            return Err(error("unsupported format, policy or count-stage coverage"));
        }
        if r.suite.version != admitted.suite().provenance.suite_version.to_string()
            || r.suite.digest_profile != "sha256-json-bytes/1"
            || r.suite.digest != admitted.digest()
        {
            return Err(error("exact suite reference mismatch"));
        }
        let provenance = &admitted.suite().provenance;
        if r.spec_digest != provenance.spec_digest
            || r.specification
                != format!("{}/{}", provenance.system, provenance.specification_version)
        {
            return Err(error("specification identity mismatch"));
        }
        let arrays = r.outcomes.arrays();
        let counts = [
            r.counts.passed,
            r.counts.failed,
            r.counts.error,
            r.counts.unsupported,
            r.counts.skipped,
        ];
        let mut sum = 0_u64;
        let mut union = BTreeSet::new();
        for (ids, count) in arrays.into_iter().zip(counts) {
            sum = sum
                .checked_add(count)
                .ok_or_else(|| error("category sum overflow"))?;
            if length(ids)? != count || ids.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(error(
                    "category count or sorted distinct outcome IDs disagree",
                ));
            }
            for id in ids {
                if !union.insert(id) {
                    return Err(error("outcome categories overlap"));
                }
            }
        }
        if r.counts.total != sum || union.into_iter().ne(admitted.suite().scenarios.keys()) {
            return Err(error(
                "total or exact selected outcome membership disagrees",
            ));
        }
        let expected = execution(r.producer_profile, &r.counts)?;
        if r.execution_status != expected
            || r.conformance_status != qualification(expected, admitted)
        {
            return Err(error(
                "execution/conformance status contradicts producer outcomes and unknown coverage",
            ));
        }
        Ok(())
    }
    /// Canonical UTF-8 JSON, with sorted object keys and exactly one final LF.
    pub fn to_canonical_json(&self) -> Result<String, AdmissionError> {
        canonical(self)
    }
    /// The actual terminal counts.
    pub fn counts(&self) -> &ScenarioCounts {
        &self.0.counts
    }
    /// The producer's execution verdict.
    pub fn execution_status(&self) -> CountStatus {
        self.0.execution_status
    }
    /// Qualification under complete-selection/1; never passed for count-stage unknown coverage.
    pub fn conformance_status(&self) -> CountStatus {
        self.0.conformance_status
    }
    /// Exact unsigned epoch milliseconds.
    pub fn completed_at(&self) -> u64 {
        self.0.completed_at
    }
}
fn execution(profile: ProducerProfile, c: &ScenarioCounts) -> Result<CountStatus, AdmissionError> {
    if (profile == ProducerProfile::Rust && c.skipped != 0)
        || (profile == ProducerProfile::Go && (c.error != 0 || c.unsupported != 0))
    {
        return Err(error("category unavailable for producer profile"));
    }
    Ok(if c.failed > 0 || c.unsupported > 0 {
        CountStatus::Failed
    } else if c.error > 0 || c.skipped > 0 {
        CountStatus::Inconclusive
    } else {
        CountStatus::Passed
    })
}
fn qualification(execution: CountStatus, admitted: &AdmittedSuite) -> CountStatus {
    if execution == CountStatus::Failed {
        CountStatus::Failed
    } else if execution == CountStatus::Passed
        && !admitted.suite().is_empty()
        && admitted
            .coverage()
            .is_some_and(crate::coverage::Inventory::is_complete)
    {
        CountStatus::Passed
    } else {
        CountStatus::Inconclusive
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRun {
    format: String,
    summary: WireReport,
    started_at: u64,
    scenarios: Vec<ScenarioResult>,
}
/// Closed detailed Rust run/2; its summary is a standalone report/2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct CountRun(WireRun);
impl CountRun {
    /// Construct the separate detailed surface from one complete run.
    pub fn from_run(run: &ExecutedRun, admitted: &AdmittedSuite) -> Result<Self, AdmissionError> {
        let result = Self(WireRun {
            format: COUNT_RUN_FORMAT.into(),
            summary: CountReport::from_run(run, admitted)?.0,
            started_at: run.started_at.epoch_millis(),
            scenarios: run.scenarios.clone(),
        });
        result.validate()?;
        Ok(result)
    }
    /// Read original JSON with exact integer tokens and paired summary/result agreement.
    pub fn from_json(text: &str, admitted: &AdmittedSuite) -> Result<Self, AdmissionError> {
        let raw = Json::parse(text, "$")?;
        let fields = raw.closed(&["format", "summary", "started_at", "scenarios"], &[])?;
        fields["started_at"].unsigned()?;
        CountReport::from_raw(&fields["summary"], admitted)?;
        for result in fields["scenarios"].array()? {
            let r = result.object()?;
            r.get("duration_ms")
                .ok_or_else(|| error("missing duration_ms"))?
                .unsigned()?;
        }
        let result = Self(serde_json::from_str(text).map_err(|e| error(e.to_string()))?);
        result.validate()?;
        Ok(result)
    }
    fn validate(&self) -> Result<(), AdmissionError> {
        if self.0.format != COUNT_RUN_FORMAT
            || self.0.summary.producer_profile != ProducerProfile::Rust
        {
            return Err(error("unsupported detailed format/profile"));
        }
        let elapsed = self
            .0
            .summary
            .completed_at
            .checked_sub(self.0.started_at)
            .ok_or_else(|| error("completed_at precedes started_at"))?;
        let duration = self.0.scenarios.iter().try_fold(0_u64, |sum, scenario| {
            sum.checked_add(scenario.duration_ms)
                .ok_or_else(|| error("scenario duration sum overflow"))
        })?;
        if duration > elapsed {
            return Err(error("scenario durations exceed the run interval"));
        }
        if self
            .0
            .scenarios
            .windows(2)
            .any(|p| p[0].scenario >= p[1].scenario)
        {
            return Err(error("detailed IDs must be ordered and distinct"));
        }
        let mut outcomes = Outcomes::default();
        for result in &self.0.scenarios {
            if result.status
                != result
                    .checks
                    .iter()
                    .fold(Status::Passed, |status, check| status.worst(check.status))
            {
                return Err(error("scenario status contradicts its checks"));
            }
            for check in &result.checks {
                if let Some(d) = &check.diagnostic {
                    if d.scenario != result.scenario || d.code != check.code {
                        return Err(error("diagnostic identity/code mismatch"));
                    }
                }
            }
            let ids = match result.status {
                Status::Passed => &mut outcomes.passed,
                Status::Failed => &mut outcomes.failed,
                Status::Error => &mut outcomes.error,
                Status::Unsupported => &mut outcomes.unsupported,
            };
            ids.push(result.scenario.clone());
        }
        if outcomes != self.0.summary.outcomes {
            return Err(error("detailed results disagree with summary outcomes"));
        }
        Ok(())
    }
    /// Canonical sorted JSON with a final LF.
    pub fn to_canonical_json(&self) -> Result<String, AdmissionError> {
        canonical(self)
    }
}

#[cfg(test)]
mod tests {
    use super::canonical;
    use crate::count_json::Json;

    #[test]
    fn exact_unsigned_scalar_vectors_do_not_use_binary64() {
        for value in [
            0,
            9_007_199_254_740_993,
            9_223_372_036_854_775_807,
            9_223_372_036_854_775_808,
            u64::MAX,
        ] {
            assert_eq!(
                Json::parse(&value.to_string(), "$.counts.total")
                    .unwrap()
                    .unsigned()
                    .unwrap(),
                value
            );
        }
    }

    #[test]
    fn payload_number_and_utf8_canonical_profile_is_frozen_separately_from_scalars() {
        let value = serde_json::json!({"z":[1.0,-0.0,0.000_001,1e21,1e-7],"é":"e\u{301}<>&/\u{2028}\n\u{1}","a":{}});
        assert_eq!(canonical(&value).unwrap(),"{\n  \"a\": {},\n  \"z\": [\n    1.0,\n    -0.0,\n    1e-6,\n    1e+21,\n    1e-7\n  ],\n  \"é\": \"e\u{301}<>&/\u{2028}\\n\\u0001\"\n}\n");
    }
}
