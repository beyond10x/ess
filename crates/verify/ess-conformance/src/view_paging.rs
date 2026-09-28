//! The suite format authority for a page of a paged view (`paging:`, ess/16, beyond10x/ess#174,
//! `docs/design/view-paging.md`).
//!
//! A paged view answers `size` rows of its declared order starting at `(page - first_page) *
//! size`, and with `total: true` the number of rows its filter admits beside them. The page and the
//! size travel as the view's declared parameters, under their declared names, in the read's
//! `params` — a target reads them there as it reads any other parameter. What the read's answer
//! adds is the total ([`SemanticViewResult::total`](crate::target::SemanticViewResult::total)).
//!
//! A scenario that arranges more rows than a page holds asserts a page with a
//! [`Page`](crate::scenario::ViewExpectation::Page) expectation: its exact length, a floor on its
//! total, and — for the page after the first — that it continues the page before it, which the run
//! snapshotted ([`SnapshotView`](crate::ScenarioStep::SnapshotView)).
//!
//! Such a suite is written in ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], the
//! round-3 pair [`crate::leaf_payloads`] registered. An older reader does not know `page` and would
//! refuse it as an unknown expectation, blaming the document for the age of the tool; the number
//! turns that into "upgrade the tool". The Go and TypeScript runtimes execute these majors
//! too (beyond10x/ess#188).

use ess_domain::view::Ranking;

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioStep, ViewExpectation};

/// The first ordinary suite major that carries a `page` expectation.
pub const ORDINARY: u32 = crate::leaf_payloads::ORDINARY;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = crate::leaf_payloads::COVERAGE;

/// The expectation's tag in a suite document.
pub(crate) const TAG: &str = "page";

pub(crate) const REQUIRES: &str = "a `page` view expectation requires suite/26 or /27";

/// How a page continues the page before it, which the run snapshotted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Follows {
    /// The view's declared order, most significant key first: the page's rows are in it, and none
    /// ranks before the snapshot's last row.
    pub order_by: Vec<Ranking>,
    /// The fields that identify a row. No row of the page carries the values of a snapshot row in
    /// all of them. Empty: the view projects no identity, and rows are not told apart.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub distinct_by: Vec<String>,
}

/// Whether any view expectation of the suite is a page.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::ExpectView { expectation, .. }
            | ScenarioStep::EventuallyView { expectation, .. } => {
                matches!(expectation, ViewExpectation::Page { .. })
            }
            _ => false,
        })
    })
}

/// Why a page expectation is no claim, if it is none: a size of zero asks for nothing, a page
/// longer than its size is one no target can answer, and a continuation with no order compares
/// nothing.
pub(crate) fn defect(
    size: u64,
    rows: usize,
    total_at_least: Option<u64>,
    unordered_continuation: bool,
) -> Option<String> {
    if size == 0 {
        return Some("a page of size 0 holds no row whatever the target does".to_owned());
    }
    if u64::try_from(rows).map_or(true, |rows| rows > size) {
        return Some(format!(
            "a page of size {size} cannot hold {rows} rows, so no target can satisfy it"
        ));
    }
    if total_at_least.is_some_and(|total| u64::try_from(rows).is_ok_and(|rows| total < rows)) {
        return Some(format!(
            "a total of at least {} is less than the {rows} rows the page itself holds",
            total_at_least.unwrap_or_default()
        ));
    }
    if unordered_continuation {
        return Some(
            "a page continues another only in a declared order, and names none".to_owned(),
        );
    }
    None
}

/// Refuse an explicitly pinned older format, and a page that is no claim, before serialization or
/// target effects.
pub(crate) fn admit_format(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if !used_by(suite) {
        return Ok(());
    }
    if suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    for (id, scenario) in &suite.scenarios {
        for (position, step) in scenario.steps.iter().enumerate() {
            let (ScenarioStep::ExpectView { expectation, .. }
            | ScenarioStep::EventuallyView { expectation, .. }) = step
            else {
                continue;
            };
            let ViewExpectation::Page {
                size,
                rows,
                total_at_least,
                follows,
                ..
            } = expectation
            else {
                continue;
            };
            if let Some(reason) = defect(
                *size,
                *rows,
                *total_at_least,
                follows
                    .as_ref()
                    .is_some_and(|follows| follows.order_by.is_empty()),
            ) {
                return Err(AdmissionError::new(
                    "InvalidSuite",
                    format!("$suite.scenarios.{id}.steps.{position}.expectation"),
                    reason,
                ));
            }
        }
    }
    Ok(())
}

/// Whether a suite document labelled `major` may carry the expectation tagged `tag`.
pub(crate) fn needs_newer(tag: &str, major: u32) -> bool {
    tag == TAG && major < ORDINARY
}

/// Admit a `page` expectation's bytes: exactly `expect`, `page`, `size` and `rows`, optionally
/// `at_least` (a boolean), `total_at_least` and `follows`, each number a whole one and `follows` exactly `order_by` and
/// optionally `distinct_by`.
pub(crate) fn admit_json(value: &crate::count_json::Json) -> Result<(), AdmissionError> {
    let object = value.closed(
        &["expect", "page", "size", "rows"],
        &["at_least", "total_at_least", "follows"],
    )?;
    object["page"].unsigned()?;
    if let Some(at_least) = object.get("at_least") {
        at_least.boolean()?;
    }
    let size = object["size"].unsigned()?;
    let rows = object["rows"].unsigned()?;
    let total = object
        .get("total_at_least")
        .map(crate::count_json::Json::unsigned)
        .transpose()?;
    let unordered = match object.get("follows") {
        None => false,
        Some(follows) => {
            let fields = follows.closed(&["order_by"], &["distinct_by"])?;
            let order_by = fields["order_by"].array()?;
            for key in order_by {
                key.text()?;
            }
            if let Some(distinct) = fields.get("distinct_by") {
                for field in distinct.array()? {
                    field.text()?;
                }
            }
            order_by.is_empty()
        }
    };
    let rows = usize::try_from(rows).unwrap_or(usize::MAX);
    if let Some(reason) = defect(size, rows, total, unordered) {
        return Err(value.error("InvalidSuite", reason));
    }
    Ok(())
}
