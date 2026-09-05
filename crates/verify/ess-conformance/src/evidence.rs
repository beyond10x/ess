//! Standalone ESS conformance reports.
//!
//! A conformance run publishes one closed ESS document. Workflow systems may adapt that document
//! at their own boundary; this crate does not know their evidence, producer, or provenance models.

use ess_primitives::verification::VerificationStatus;

use crate::report::{ConformanceReport, ConformanceStatus, Status};
use crate::scenario::SuiteFormat;

/// Persisted format for a standalone ESS conformance report.
pub const STANDALONE_REPORT_FORMAT: &str = "ess-conformance-report/1";

/// The stable, transport-neutral summary one conformance execution publishes.
///
/// This document contains only ESS vocabulary. AEP or another workflow system may adapt it into
/// its own evidence model without making ESS depend on that model.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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

impl<'de> serde::Deserialize<'de> for StandaloneConformanceReport {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Keep the unchecked wire shape private: every Serde entry point must validate the same
        // claims before returning the public report, not only callers of `from_json`.
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireReport {
            format: String,
            specification: String,
            spec_digest: ess_primitives::evidence::SpecDigest,
            implementation: String,
            status: VerificationStatus,
            scenarios_total: usize,
            scenarios_failed: usize,
            suite_version: String,
            failed_scenarios: Vec<String>,
            completed_at: ess_primitives::time::Timestamp,
        }

        let wire: WireReport = serde::Deserialize::deserialize(deserializer)?;
        let report = Self {
            format: wire.format,
            specification: wire.specification,
            spec_digest: wire.spec_digest,
            implementation: wire.implementation,
            status: wire.status,
            scenarios_total: wire.scenarios_total,
            scenarios_failed: wire.scenarios_failed,
            suite_version: wire.suite_version,
            failed_scenarios: wire.failed_scenarios,
            completed_at: wire.completed_at,
        };
        report.validate().map_err(serde::de::Error::custom)?;
        Ok(report)
    }
}

impl StandaloneConformanceReport {
    fn validate(&self) -> Result<(), &'static str> {
        if self.format != STANDALONE_REPORT_FORMAT {
            return Err("unsupported standalone conformance report format");
        }
        if !SuiteFormat::parse(&self.suite_version).is_ok_and(SuiteFormat::is_supported) {
            return Err("suite_version is not a supported conformance-suite format");
        }
        if self.scenarios_failed > self.scenarios_total {
            return Err("scenarios_failed exceeds scenarios_total");
        }
        if self.scenarios_failed != self.failed_scenarios.len() {
            return Err("scenarios_failed disagrees with failed_scenarios length");
        }

        // In v1 this list and count contain every non-pass. Rust writes error/unsupported; Go
        // writes skipped. The document has no producer identity, so retain both vocabularies.
        let mut expected_status = VerificationStatus::Passed;
        for entry in &self.failed_scenarios {
            let (status, scenario) = entry
                .split_once(' ')
                .ok_or("failed_scenarios entry requires a status and scenario identity")?;
            if scenario.trim().is_empty() {
                return Err("failed_scenarios entry requires a scenario identity");
            }
            match status {
                "failed" | "unsupported" => expected_status = VerificationStatus::Failed,
                "error" | "skipped" => {
                    if expected_status != VerificationStatus::Failed {
                        expected_status = VerificationStatus::Inconclusive;
                    }
                }
                _ => return Err("failed_scenarios entry requires a known non-pass status"),
            }
        }
        if self.status != expected_status {
            return Err("status contradicts the listed scenario outcomes");
        }
        Ok(())
    }

    /// Canonical pretty JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        let mut rendered = serde_json::to_string_pretty(self)
            .unwrap_or_else(|error| panic!("a standalone conformance report serializes: {error}"));
        rendered.push('\n');
        rendered
    }

    /// Reads and validates the closed standalone report shape, versions, counts, and status.
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
                component: None,
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

    fn assert_reader_refuses(value: serde_json::Value) {
        let text = value.to_string();
        let accepted = [
            (
                "report reader",
                StandaloneConformanceReport::from_json(&text).is_ok(),
            ),
            (
                "JSON value",
                serde_json::from_value::<StandaloneConformanceReport>(value).is_ok(),
            ),
            (
                "JSON reader",
                serde_json::from_reader::<_, StandaloneConformanceReport>(text.as_bytes()).is_ok(),
            ),
            (
                "YAML",
                serde_yaml::from_str::<StandaloneConformanceReport>(&text).is_ok(),
            ),
        ];
        assert!(
            accepted.iter().all(|(_, accepted)| !accepted),
            "invalid report accepted: {accepted:?}"
        );
    }

    fn assert_reader_preserves(report: &StandaloneConformanceReport) {
        let written = report.to_canonical_json();
        let from_report = StandaloneConformanceReport::from_json(&written).expect("report reads");
        let from_value = serde_json::from_value::<StandaloneConformanceReport>(
            serde_json::to_value(report).expect("report serializes"),
        )
        .expect("value reads");
        let from_reader =
            serde_json::from_reader::<_, StandaloneConformanceReport>(written.as_bytes())
                .expect("reader reads");
        let from_yaml = serde_yaml::from_str::<StandaloneConformanceReport>(&written)
            .expect("other Serde formats accept valid reports");
        for read in [from_report, from_value, from_reader, from_yaml] {
            assert_eq!(&read, report);
            assert_eq!(read.to_canonical_json().as_bytes(), written.as_bytes());
        }
    }

    #[test]
    fn report_readers_refuse_unknown_report_formats() {
        for format in [
            "other/99",
            "ess-conformance-report/2",
            "ess-conformance-report/01",
            "",
        ] {
            let mut value = serde_json::to_value(report(vec![]).standalone()).expect("report");
            value["format"] = format.into();
            assert_reader_refuses(value);
        }
    }

    #[test]
    fn report_readers_refuse_malformed_and_unsupported_suite_versions() {
        for version in [
            "other/99",
            "ess-conformance/5",
            "ess-conformance/99",
            "ess-conformance/0",
            "ess-conformance/01",
            "ess-conformance/+1",
            "ess-conformance/4294967296",
            "",
        ] {
            let mut value = serde_json::to_value(report(vec![]).standalone()).expect("report");
            value["suite_version"] = version.into();
            assert_reader_refuses(value);
        }
    }

    #[test]
    fn report_readers_refuse_more_nonpasses_than_executed_scenarios() {
        let mut value = serde_json::to_value(report(vec![scenario(Status::Failed)]).standalone())
            .expect("report");
        value["scenarios_total"] = 0.into();
        assert_reader_refuses(value);
    }

    #[test]
    fn report_readers_refuse_nonpass_count_and_list_disagreement() {
        for count in [0, 2] {
            let mut value = serde_json::to_value(
                report(vec![scenario(Status::Passed), scenario(Status::Failed)]).standalone(),
            )
            .expect("report");
            value["scenarios_failed"] = count.into();
            assert_reader_refuses(value);
        }
    }

    #[test]
    fn report_readers_refuse_nonpass_entries_without_a_known_nonpass_status() {
        for entry in [
            "passed scenario",
            "unknown scenario",
            "failed",
            "failed ",
            "failed  \t",
            " failed scenario",
            "Failed scenario",
            "",
        ] {
            let mut value =
                serde_json::to_value(report(vec![scenario(Status::Failed)]).standalone())
                    .expect("report");
            value["failed_scenarios"] = serde_json::json!([entry]);
            assert_reader_refuses(value);
        }
    }

    #[test]
    fn report_readers_refuse_status_claims_that_contradict_the_list() {
        for (entries, expected) in [
            (vec![], "passed"),
            (vec!["failed scenario"], "failed"),
            (vec!["unsupported scenario"], "failed"),
            (vec!["error scenario"], "inconclusive"),
            (vec!["skipped scenario"], "inconclusive"),
            (vec!["failed scenario", "skipped other"], "failed"),
            (vec!["unsupported scenario", "error other"], "failed"),
            (vec!["error scenario", "skipped other"], "inconclusive"),
        ] {
            for status in ["passed", "failed", "inconclusive", "skipped"] {
                if status == expected {
                    continue;
                }
                let mut value = serde_json::to_value(report(vec![]).standalone()).expect("report");
                value["status"] = status.into();
                value["scenarios_total"] = entries.len().into();
                value["scenarios_failed"] = entries.len().into();
                value["failed_scenarios"] = serde_json::json!(entries);
                assert_reader_refuses(value);
            }
        }
    }

    #[test]
    fn report_readers_preserve_rust_producer_bytes_for_every_supported_suite() {
        for version in [
            "ess-conformance/1",
            "ess-conformance/2",
            "ess-conformance/3",
            "ess-conformance/4",
        ] {
            for statuses in [
                vec![],
                vec![Status::Passed],
                vec![Status::Failed],
                vec![Status::Error],
                vec![Status::Unsupported],
                vec![
                    Status::Passed,
                    Status::Failed,
                    Status::Error,
                    Status::Unsupported,
                ],
            ] {
                let mut report = report(statuses.into_iter().map(scenario).collect());
                report.suite.suite_version =
                    SuiteFormat::parse(version).expect("supported version");
                assert_reader_preserves(&report.standalone());
            }
        }
    }

    #[test]
    fn report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts() {
        // The generated Go runtime writes both failures and skips into this list and count.
        // Freeze its field order and complete bytes independently of the Rust report writer.
        let written = concat!(
            "{\n",
            "  \"format\": \"ess-conformance-report/1\",\n",
            "  \"specification\": \"billing/v3\",\n",
            "  \"spec_digest\": \"13577b3ce695932e980d418d5863bcde07f4c362516d53147870d31eaf2ed861\",\n",
            "  \"implementation\": \"billing-go 0.1.0\",\n",
            "  \"status\": \"inconclusive\",\n",
            "  \"scenarios_total\": 2,\n",
            "  \"scenarios_failed\": 1,\n",
            "  \"suite_version\": \"ess-conformance/4\",\n",
            "  \"failed_scenarios\": [\n",
            "    \"skipped billing.invoice.CreateInvoice/outcome/accepted\"\n",
            "  ],\n",
            "  \"completed_at\": 1700000001000\n",
            "}\n",
        );
        let skipped = StandaloneConformanceReport::from_json(written).expect("Go report reads");
        assert_eq!(skipped.to_canonical_json(), written);
        assert_reader_preserves(&skipped);

        let mut failed = skipped;
        failed.status = VerificationStatus::Failed;
        failed.scenarios_failed = 2;
        failed
            .failed_scenarios
            .push("failed billing.invoice.CreateInvoice/outcome/rejected".into());
        assert_reader_preserves(&failed);
    }

    #[test]
    fn report_readers_do_not_guess_a_producer_from_status_vocabulary() {
        let mut report =
            report(vec![scenario(Status::Error), scenario(Status::Error)]).standalone();
        report.failed_scenarios[1] = "skipped an opaque scenario identity".into();
        assert_reader_preserves(&report);
    }
}
