//! A `Decimal` in an exact observer (`docs/design/retained-command-results.md`).
//!
//! Both exact observers, the complete subject a refusal leaves unchanged and the retained result a
//! retry returns, compare a `Decimal` at any depth by its numeric value, so `1.5` equals `1.50`.
//! Nothing writes an expected `Decimal`: each observer compares the target with itself, before
//! and after. A reader below suite/[`ORDINARY`] refused such a shape at the step, so a suite
//! carrying one is ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], cumulative over
//! every pair below, and every reader refuses it under a lower major. A suite without one keeps
//! its earlier format and bytes. A `Binary64` is compared by its nearest binary64 from suite/62
//! ([`crate::binary64_values`]).

use crate::admission::AdmissionError;
use crate::scenario::ScenarioStep;
use crate::selection::Declaration;
use crate::ConformanceSuite;
use ess_domain::{Field, Primitive, QualifiedName, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

/// The ordinary suite major whose exact observers carry a `Decimal`.
pub const ORDINARY: u32 = 58;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 59;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// What a refusal under a lower major says.
pub const REQUIRES: &str = "a Decimal in an exact observer (a complete subject or a retained \
                            result) requires suite/58 or /59";

/// Whether an exact schema reaches a `Decimal` through any field or declaration it names.
pub(crate) fn carries(
    fields: &[Field],
    declarations: &BTreeMap<QualifiedName, Declaration>,
) -> bool {
    let mut seen = BTreeSet::new();
    fields
        .iter()
        .any(|field| reaches(&field.type_ref, declarations, &mut seen))
}

fn reaches(
    ty: &TypeRef,
    declarations: &BTreeMap<QualifiedName, Declaration>,
    seen: &mut BTreeSet<QualifiedName>,
) -> bool {
    match ty {
        TypeRef::Primitive(primitive) => *primitive == Primitive::Decimal,
        TypeRef::List(of) | TypeRef::Optional(of) | TypeRef::Map(_, of) => {
            reaches(of, declarations, seen)
        }
        TypeRef::Named(name) => {
            seen.insert(name.clone())
                && declarations.get(name).is_some_and(|declared| {
                    declared
                        .references()
                        .into_iter()
                        .any(|ty| reaches(ty, declarations, seen))
                })
        }
    }
}

/// Whether a suite carries a `Decimal` in an exact observer.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::SnapshotCompleteSubject { shape, .. } => {
                carries(&shape.fields, &shape.declarations)
            }
            ScenarioStep::CaptureCommandResult { capture }
            | ScenarioStep::ExpectReplayResult { capture } => {
                carries(&capture.fields, &capture.declarations)
            }
            _ => false,
        })
    })
}

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`].
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}

/// Refuses a `Decimal` in an exact observer under a major below [`ORDINARY`].
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
