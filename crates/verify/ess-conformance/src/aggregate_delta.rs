//! The suite format authority for the change in an ungrouped aggregate (story
//! `ungrouped-aggregate-views-are-witnessed`, beyond10x/ess#148).
//!
//! `docs/design/aggregate-views.md`, "Scoping", is the binding design. An ungrouped aggregate view
//! with no parameter holds one row over every row of its source, including rows other users of a
//! shared target made (§8), so its absolute value is a claim about them. What the scenario's own
//! rows decide is the **change**: the view is read and snapshotted before the rows are created
//! ([`SnapshotView`](crate::ScenarioStep::SnapshotView)), and read again after them with a
//! [`ChangedBy`](crate::scenario::ViewExpectation::ChangedBy) expectation that holds when every
//! named field moved by exactly the stated amount. Only `count` and `sum` change by an amount the
//! new rows alone decide, so only those fields are named.
//!
//! Such a suite is written in ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], the
//! round-3 pair [`crate::leaf_payloads`] registered. The number moves because of what an older
//! reader does with the expectation: it does not know `changed_by` and refuses it as an unknown
//! expectation, which blames the document for the age of the tool; the number turns that into
//! "upgrade the tool". The Go and TypeScript runtimes refuse these majors by version, so they need
//! no execution support for it.

use std::collections::{BTreeMap, BTreeSet};

use ess_primitives::node::Node;

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioStep, ViewExpectation};

/// The first ordinary suite major that carries a `changed_by` expectation.
pub const ORDINARY: u32 = crate::leaf_payloads::ORDINARY;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = crate::leaf_payloads::COVERAGE;

/// The expectation's tag in a suite document.
pub(crate) const TAG: &str = "changed_by";

pub(crate) const REQUIRES: &str = "a `changed_by` view expectation requires suite/26 or /27";

/// Whether any view expectation of the suite is a change.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::ExpectView { expectation, .. }
            | ScenarioStep::EventuallyView { expectation, .. } => {
                matches!(expectation, ViewExpectation::ChangedBy { .. })
            }
            _ => false,
        })
    })
}

/// Refuse an explicitly pinned older format, and a change that names no field or a value that is
/// not a number, before serialization or target effects.
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
            let ViewExpectation::ChangedBy {
                fields,
                absent_is_zero,
            } = expectation
            else {
                continue;
            };
            let at = format!("$suite.scenarios.{id}.steps.{position}.expectation");
            if let Some(reason) = defect(fields, absent_is_zero) {
                return Err(AdmissionError::new("InvalidSuite", at, reason));
            }
        }
    }
    Ok(())
}

/// Why a change is no claim, if it is none: no field at all, a value that is not a number or has
/// no exact decimal spelling, or an `absent_is_zero` entry that names no field of the change.
pub(crate) fn defect(
    fields: &BTreeMap<String, Node>,
    absent_is_zero: &BTreeSet<String>,
) -> Option<String> {
    if fields.is_empty() {
        return Some("a change that names no field requires nothing".to_owned());
    }
    fields
        .iter()
        .find(|(_, value)| !matches!(value, Node::Number(_)))
        .map(|(field, _)| format!("the change of `{field}` is not a number"))
        .or_else(|| {
            // The runner compares exactly; an amount with no exact decimal spelling (`1e40`, or
            // one past the exact range) is one it can never compare, so it is no claim either.
            fields
                .iter()
                .find(|(_, value)| crate::aggregate::change(&Node::Null, value).is_none())
                .map(|(field, _)| format!("the change of `{field}` has no exact decimal spelling"))
        })
        .or_else(|| {
            absent_is_zero
                .iter()
                .find(|field| !fields.contains_key(*field))
                .map(|field| format!("`absent_is_zero` names `{field}`, which the change does not"))
        })
}

/// Whether a suite document labelled `major` may carry the expectation tagged `tag`.
pub(crate) fn needs_newer(tag: &str, major: u32) -> bool {
    tag == TAG && major < ORDINARY
}

/// Admit a `changed_by` expectation's bytes: exactly `expect` and `fields`, and each field's amount
/// a finite JSON number.
pub(crate) fn admit_json(value: &crate::count_json::Json) -> Result<(), AdmissionError> {
    let object = value.closed(&["expect", "fields"], &["absent_is_zero"])?;
    let fields = object["fields"].object()?;
    if fields.is_empty() {
        return Err(object["fields"].error(
            "InvalidSuite",
            "a change that names no field requires nothing",
        ));
    }
    for (field, amount) in fields {
        amount.payload()?;
        if !amount.raw.parse::<f64>().is_ok_and(f64::is_finite) {
            return Err(amount.error(
                "InvalidSuite",
                format!("the change of `{field}` is not a number"),
            ));
        }
    }
    if let Some(listed) = object.get("absent_is_zero") {
        for entry in listed.array()? {
            let name = entry.text()?;
            if !fields.contains_key(name) {
                return Err(entry.error(
                    "InvalidSuite",
                    format!("`absent_is_zero` names `{name}`, which the change does not"),
                ));
            }
        }
    }
    Ok(())
}
