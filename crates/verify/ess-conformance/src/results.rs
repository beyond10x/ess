//! `ess-conformance-results/1`: per-scenario results a runner outside ESS supplies.
//!
//! A runner written in any language executes an admitted suite itself and hands ESS one terminal
//! status per scenario. ESS admits the suite, checks the results against it and writes
//! `ess-conformance-report/2` ([`CountReport::from_external`]) with coverage, suite reference and
//! policy taken from its own admission. The report's `producer_profile` is
//! `external-scenario-status/1`, so evidence built on it cannot be read as a run ESS executed.
//!
//! The document is closed:
//!
//! ```json
//! {
//!   "format": "ess-conformance-results/1",
//!   "completed_at": 1700000000000,
//!   "suite_digest": "sha256:…",
//!   "results": [
//!     {"scenario_id": "…", "status": "passed"},
//!     {"scenario_id": "…", "status": "failed", "message": "…"}
//!   ]
//! }
//! ```
//!
//! `suite_digest` and `message` are optional. `status` is one of `passed`, `failed`, `error` and
//! `unsupported`, with the Rust runner's meanings. Every scenario of the suite has exactly one
//! result, and no result names a scenario the suite does not contain.
use crate::{
    admission::AdmissionError, count_json::Json, coverage::AdmittedInput, AdmittedSuite,
    CountReport, ScenarioId, Status,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The one results format this build reads.
pub const RESULTS_FORMAT: &str = "ess-conformance-results/1";

/// One supplied terminal result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalResult {
    /// A scenario of the admitted suite.
    pub scenario: ScenarioId,
    /// Its terminal status.
    pub status: Status,
    /// The runner's own explanation; not carried into report/2.
    pub message: Option<String>,
}

/// A results document checked against one admitted suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalResults {
    admitted_digest: String,
    completed_at: u64,
    results: Vec<ExternalResult>,
}

impl ExternalResults {
    /// Read `ess-conformance-results/1` and check it against `admitted`, naming every refusal.
    pub fn from_json(text: &str, admitted: &AdmittedSuite) -> Result<Self, AdmissionError> {
        let raw = Json::parse(text, "$")?;
        let (fields, completed_at) = envelope(&raw, admitted)?;
        let known: BTreeMap<String, &ScenarioId> = admitted
            .suite()
            .scenarios
            .keys()
            .map(|id| (id.to_string(), id))
            .collect();
        let mut issues = Vec::new();
        let mut seen = BTreeSet::new();
        let mut results = Vec::new();
        for entry in fields["results"].array()? {
            match result(entry, &known, &mut seen) {
                Ok(Some(result)) => results.push(result),
                Ok(None) => {}
                Err(error) => issues.extend(error.issues),
            }
        }
        for name in known.keys().filter(|name| !seen.contains(*name)) {
            issues.extend(
                fields["results"]
                    .error(
                        "MissingResult",
                        format!("{name} is a scenario of the admitted suite with no result"),
                    )
                    .issues,
            );
        }
        if !issues.is_empty() {
            return Err(AdmissionError { issues });
        }
        results.sort_by(|a, b| a.scenario.cmp(&b.scenario));
        Ok(Self {
            admitted_digest: admitted.digest().to_owned(),
            completed_at,
            results,
        })
    }
    /// Exact epoch milliseconds the runner finished at.
    pub fn completed_at(&self) -> u64 {
        self.completed_at
    }
    /// One result per scenario of the suite, in scenario order.
    pub fn results(&self) -> &[ExternalResult] {
        &self.results
    }
    /// The `sha256-json-bytes/1` digest of the suite these results were checked against.
    pub fn admitted_digest(&self) -> &str {
        &self.admitted_digest
    }
}

/// The closed top level: its format, exact `completed_at` and, when carried, the suite digest.
fn envelope<'a>(
    raw: &'a Json,
    admitted: &AdmittedSuite,
) -> Result<(&'a BTreeMap<String, Json>, u64), AdmissionError> {
    if let Some(format) = raw.object()?.get("format") {
        if format.text()? != RESULTS_FORMAT {
            return Err(format.error(
                "UnsupportedResultsFormat",
                format!("expected {RESULTS_FORMAT}"),
            ));
        }
    }
    let fields = raw.closed(&["format", "completed_at", "results"], &["suite_digest"])?;
    let completed_at = fields["completed_at"].unsigned()?;
    if let Some(carried) = fields.get("suite_digest") {
        let digest = carried
            .text()
            .map_err(|_| carried.error("SuiteDigestMismatch", "suite_digest must be a string"))?;
        if digest != admitted.digest() {
            return Err(carried.error(
                "SuiteDigestMismatch",
                format!(
                    "results are for suite {digest}; the admitted suite is {}",
                    admitted.digest()
                ),
            ));
        }
    }
    Ok((fields, completed_at))
}

/// One entry, with every refusal it carries; `None` only when a refusal is already recorded.
fn result(
    entry: &Json,
    known: &BTreeMap<String, &ScenarioId>,
    seen: &mut BTreeSet<String>,
) -> Result<Option<ExternalResult>, AdmissionError> {
    let fields = entry.closed(&["scenario_id", "status"], &["message"])?;
    let named = fields["scenario_id"].text()?;
    let mut issues = Vec::new();
    let status = match fields["status"].text() {
        Ok("passed") => Some(Status::Passed),
        Ok("failed") => Some(Status::Failed),
        Ok("error") => Some(Status::Error),
        Ok("unsupported") => Some(Status::Unsupported),
        Ok(other) => {
            issues.extend(
                fields["status"]
                    .error(
                        "UnknownStatus",
                        format!(
                            "{other:?} for {named}; expected passed, failed, error or unsupported"
                        ),
                    )
                    .issues,
            );
            None
        }
        Err(error) => {
            issues.extend(error.issues);
            None
        }
    };
    let message = match fields.get("message").map(Json::text).transpose() {
        Ok(message) => message.map(str::to_owned),
        Err(error) => {
            issues.extend(error.issues);
            None
        }
    };
    let scenario = if let Some(&scenario) = known.get(named) {
        if !seen.insert(named.to_owned()) {
            issues.extend(
                fields["scenario_id"]
                    .error(
                        "DuplicateResult",
                        format!("{named} has more than one result"),
                    )
                    .issues,
            );
        }
        Some(scenario)
    } else {
        issues.extend(
            fields["scenario_id"]
                .error(
                    "UnknownScenario",
                    format!("{named} is not a scenario of the admitted suite"),
                )
                .issues,
        );
        None
    };
    if !issues.is_empty() {
        return Err(AdmissionError { issues });
    }
    Ok(scenario
        .zip(status)
        .map(|(scenario, status)| ExternalResult {
            scenario: scenario.clone(),
            status,
            message,
        }))
}

/// The runner that supplied results: `<name>@<version>`, split at the last `@`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Runner {
    name: String,
    version: String,
}

impl Runner {
    /// Parse `<name>@<version>`; both halves nonempty, no whitespace, control characters or `;`.
    pub fn parse(text: &str) -> Result<Self, String> {
        let refused = || {
            format!("runner {text:?} is not <name>@<version> without whitespace, controls or `;`")
        };
        if text
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == ';')
        {
            return Err(refused());
        }
        let (name, version) = text.rsplit_once('@').ok_or_else(refused)?;
        if name.is_empty() || version.is_empty() {
            return Err(refused());
        }
        Ok(Self {
            name: name.into(),
            version: version.into(),
        })
    }
    /// The runner's name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// The runner's version.
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl fmt::Display for Runner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// Refuse an implementation name that is empty, blank or carries control characters.
pub(crate) fn implementation(name: &str) -> Result<(), AdmissionError> {
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Err(AdmissionError::new(
            "InvalidImplementation",
            "$implementation",
            format!("implementation {name:?} must be nonblank and free of control characters"),
        ));
    }
    Ok(())
}

/// Admit a suite as `ess verify conform run --suite` or `--suite-input` would.
///
/// An `ess-conformance-input/1` carrier (it has a top-level `format`) yields its selected suite;
/// any other document is admitted as an original suite.
pub fn admit_suite(text: &str) -> Result<AdmittedSuite, AdmissionError> {
    let carrier = Json::parse(text, "$suite")?
        .object()
        .is_ok_and(|fields| fields.contains_key("format"));
    if carrier {
        Ok(AdmittedInput::from_json(text)?.selected().clone())
    } else {
        AdmittedSuite::from_json(text)
    }
}

/// Suite bytes and results bytes to report/2: the library behind `ess verify conform report`.
pub fn report(
    suite: &str,
    results: &str,
    implementation: &str,
    runner: Option<&str>,
) -> Result<CountReport, AdmissionError> {
    let runner = runner
        .map(Runner::parse)
        .transpose()
        .map_err(|detail| AdmissionError::new("InvalidRunner", "$runner", detail))?;
    self::implementation(implementation)?;
    let admitted = admit_suite(suite)?;
    let results = ExternalResults::from_json(results, &admitted)?;
    CountReport::from_external(&results, &admitted, implementation, runner)
}
