//! The suite format authority for per-leaf struct comparison, and the round-3 suite pair
//! (beyond10x/ess#179).
//!
//! `docs/design/value-expressions.md` E5 is the binding design. A nested mapping over a struct
//! target with one undetermined leaf — `rank: {generated: true}` beside four `input.` leaves — used
//! to assert nothing about the struct, so an implementation dropping a copied leaf passed. Each
//! determined leaf is now written as its own expected value under its **dotted path**
//! (`lead.number`) in an event payload or a view row, and the undetermined one stays covered by the
//! payload shape, whose leaves are keyed by the same paths. A field name cannot contain a dot, so a
//! dotted key is never a field of its own.
//!
//! Such a suite is written in ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], and each
//! implies every major below it. The number moves because of what an older reader does with the
//! key: it looks for a top-level field named `lead.number`, finds none, and fails a conforming
//! implementation — a wrong verdict caused by the age of the tool. A reader that checks this
//! number first refuses the suite instead. The Go and TypeScript runtimes execute these majors
//! too (beyond10x/ess#188).
//!
//! These two majors are the round-3 pair: every later round-3 suite construct shares them. The
//! second is presence of an `Optional` aggregate ([`defined_aggregates`](crate::defined_aggregates)),
//! which only the model can recognise, so format selection over a model asks both.

use std::collections::BTreeMap;

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioStep, ViewExpectation};

/// The first ordinary suite major that carries per-leaf struct values.
pub const ORDINARY: u32 = 26;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = 27;

const REQUIRES: &str = "per-leaf struct values require suite/26 or /27";

fn dotted<V>(values: &BTreeMap<String, V>) -> bool {
    values.keys().any(|key| key.contains('.'))
}

fn in_expectation(expectation: &ViewExpectation) -> bool {
    match expectation {
        ViewExpectation::Contains { fields }
        | ViewExpectation::Excludes { fields }
        | ViewExpectation::At { fields, .. } => dotted(fields),
        _ => false,
    }
}

/// Whether any event payload or view row expectation of the suite names a leaf by a dotted path.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::ExpectEvent { payload, .. }
            | ScenarioStep::EventuallyEvent { payload, .. } => dotted(payload),
            ScenarioStep::ExpectEventValues { payload, .. } => dotted(payload),
            ScenarioStep::ExpectView { expectation, .. }
            | ScenarioStep::EventuallyView { expectation, .. } => in_expectation(expectation),
            _ => false,
        })
    })
}

/// Refuse an explicitly pinned older format, and a dotted payload key that names no leaf of the
/// step's own shape, before serialization or target effects.
///
/// The second refusal is what keeps a misspelt path from becoming a check no implementation can
/// pass: the shape is the declaration's own list of leaves, and a determined leaf is always one.
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
            let (keys, shape): (Vec<&String>, _) = match step {
                ScenarioStep::ExpectEvent { payload, shape, .. }
                | ScenarioStep::EventuallyEvent { payload, shape, .. } => {
                    (payload.keys().collect(), shape)
                }
                ScenarioStep::ExpectEventValues { payload, shape, .. } => {
                    (payload.keys().collect(), shape)
                }
                _ => continue,
            };
            if shape.is_empty() {
                continue;
            }
            if let Some(key) = keys
                .into_iter()
                .find(|key| key.contains('.') && !shape.leaves().contains_key(*key))
            {
                return Err(AdmissionError::new(
                    "InvalidSuite",
                    format!("$suite.scenarios.{id}.steps.{position}.payload.{key}"),
                    format!("`{key}` names no leaf of the step's own shape"),
                ));
            }
        }
    }
    Ok(())
}
