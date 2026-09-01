//! Standalone ESS conformance reports.
//!
//! A conformance run publishes one closed ESS document. Workflow systems may adapt that document
//! at their own boundary; this crate does not know their evidence, producer, or provenance models.

use ess_primitives::verification::VerificationStatus;

use crate::report::{ConformanceReport, ConformanceStatus, Status};

/// Persisted format for a standalone ESS conformance report.
pub const STANDALONE_REPORT_FORMAT: &str = "ess-conformance-report/1";

/// The stable, transport-neutral summary one conformance execution publishes.
///
/// This document contains only ESS vocabulary. AEP or another workflow system may adapt it into
/// its own evidence model without making ESS depend on that model.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandaloneConformanceReport {
    /// Format claim, always [`STANDALONE_REPORT_FORMAT`].
    pub format: String,
    /// Human-readable specification identity.
    pub specification: String,
    /// Digest of the compiled specification.
    pub spec_digest: ess_primitives::evidence::SpecDigest,
    /// Implementation identity returned by the target.
    pub implementation: String,
    /// Overall verification result.
    pub status: VerificationStatus,
    /// Number of scenarios executed.
    pub scenarios_total: usize,
    /// Number of scenarios that did not pass.
    pub scenarios_failed: usize,
    /// Persisted conformance-suite format.
    pub suite_version: String,
    /// Scenario ids and statuses for every non-pass.
    pub failed_scenarios: Vec<String>,
    /// Deterministic completion timestamp supplied by the runner clock.
    pub completed_at: ess_primitives::time::Timestamp,
}

impl StandaloneConformanceReport {
    /// Canonical pretty JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        let mut rendered = serde_json::to_string_pretty(self)
            .unwrap_or_else(|error| panic!("a standalone conformance report serializes: {error}"));
        rendered.push('\n');
        rendered
    }

    /// Reads and validates the closed standalone report shape.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

impl ConformanceReport {
    /// Publishes this run as a standalone ESS report with no workflow-system coupling.
    pub fn standalone(&self) -> StandaloneConformanceReport {
        StandaloneConformanceReport {
            format: STANDALONE_REPORT_FORMAT.to_owned(),
            specification: format!("{}/{}", self.suite.system, self.suite.specification_version),
            spec_digest: self.suite.spec_digest.clone(),
            implementation: self.implementation.to_string(),
            status: report_status(self.status),
            scenarios_total: self.scenarios.len(),
            scenarios_failed: self
                .scenarios
                .iter()
                .filter(|result| result.status != Status::Passed)
                .count(),
            suite_version: self.suite.suite_version.to_string(),
            failed_scenarios: self
                .failures()
                .map(|result| format!("{} {}", result.status, result.scenario))
                .collect(),
            completed_at: self.completed_at,
        }
    }
}

fn report_status(status: ConformanceStatus) -> VerificationStatus {
    match status {
        ConformanceStatus::Passed => VerificationStatus::Passed,
        ConformanceStatus::Failed => VerificationStatus::Failed,
        ConformanceStatus::Error => VerificationStatus::Inconclusive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{CheckResult, Diagnostic, ScenarioResult};
    use crate::scenario::{CommandRef, ScenarioId, SuiteFormat, SuiteProvenance};
    use crate::target::ImplementationIdentity;
    use ess_domain::command::OutcomeName;
    use ess_domain::name::QualifiedName;
    use ess_primitives::evidence::SpecDigest;

    fn scenario(status: Status) -> ScenarioResult {
        let outcome = crate::scenario::OutcomeRef::new(
            CommandRef::new(
                QualifiedName::new("billing.invoice.CreateInvoice").expect("valid name"),
            ),
            OutcomeName::new("accepted").expect("valid outcome"),
        );
        ScenarioResult {
            scenario: ScenarioId::Outcome {
                outcome: outcome.clone(),
            },
            purpose: "a positive amount is accepted".to_owned(),
            status,
            checks: vec![match status {
                Status::Passed => CheckResult::passed(crate::report::CheckCode::Outcome, "outcome"),
                Status::Failed => CheckResult::failed(
                    "outcome",
                    Diagnostic::new(
                        crate::report::CheckCode::Outcome,
                        ScenarioId::Outcome { outcome },
                    ),
                ),
                Status::Error => CheckResult::errored(
                    "outcome",
                    Diagnostic::new(
                        crate::report::CheckCode::Target,
                        ScenarioId::Outcome { outcome },
                    ),
                ),
                Status::Unsupported => CheckResult::unsupported(
                    "outcome",
                    Diagnostic::new(
                        crate::report::CheckCode::Target,
                        ScenarioId::Outcome { outcome },
                    ),
                ),
            }],
            duration_ms: 0,
        }
    }

    fn report(scenarios: Vec<ScenarioResult>) -> ConformanceReport {
        ConformanceReport {
            suite: SuiteProvenance {
                suite_version: SuiteFormat::CURRENT,
                system: "billing".to_owned(),
                specification_version: "v3".to_owned(),
                spec_digest: SpecDigest::new(
                    "13577b3ce695932e980d418d5863bcde07f4c362516d53147870d31eaf2ed861",
                )
                .expect("a digest"),
                contract_digest: SpecDigest::new(
                    "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
                )
                .expect("a digest"),
            },
            implementation: ImplementationIdentity::new("billing-reference", "0.1.0"),
            started_at: ess_primitives::time::Timestamp::from_epoch_millis(1_700_000_000_000),
            completed_at: ess_primitives::time::Timestamp::from_epoch_millis(1_700_000_001_000),
            status: ConformanceReport::verdict(&scenarios),
            scenarios,
        }
    }

    #[test]
    fn a_standalone_report_carries_every_field_an_adapter_needs() {
        let standalone =
            report(vec![scenario(Status::Passed), scenario(Status::Failed)]).standalone();

        assert_eq!(standalone.format, STANDALONE_REPORT_FORMAT);
        assert_eq!(standalone.status, VerificationStatus::Failed);
        assert_eq!(standalone.scenarios_total, 2);
        assert_eq!(standalone.scenarios_failed, 1);
        assert_eq!(
            standalone.spec_digest.as_str(),
            "13577b3ce695932e980d418d5863bcde07f4c362516d53147870d31eaf2ed861"
        );
    }

    #[test]
    fn the_closed_report_round_trips_with_identical_canonical_bytes() {
        let report = report(vec![scenario(Status::Passed)]).standalone();
        let written = report.to_canonical_json();
        let read = StandaloneConformanceReport::from_json(&written).expect("report reads");

        assert_eq!(read, report);
        assert_eq!(read.to_canonical_json().as_bytes(), written.as_bytes());
        assert!(written.ends_with('\n'));
    }

    #[test]
    fn unknown_report_fields_are_refused() {
        let report = report(vec![scenario(Status::Passed)]).standalone();
        let mut value = serde_json::to_value(report).expect("report serializes");
        value.as_object_mut().expect("report is an object").insert(
            "workflow_evidence".to_owned(),
            serde_json::Value::Bool(true),
        );

        let error = StandaloneConformanceReport::from_json(&value.to_string())
            .expect_err("unknown workflow coupling is refused");
        assert!(error.to_string().contains("unknown field"), "{error}");
    }
}
