//! Format authority for responses and struct types that ignore the fields they do not declare:
//! suite `/48` ordinary and `/49` coverage (beyond10x/ess#500).
//!
//! # What changes meaning
//!
//! From `ess/24` a command may write `undeclared_fields: ignored`, which opens its `response:`, and
//! a `kind: struct` type may write it, which opens that record wherever it is reached. Through `/47`
//! every response observation was closed: the Rust, Go and TypeScript observers failed a returned
//! object carrying a key its declaration does not name (`ESS-CF-PAYLOAD`).
//!
//! From `/48` an `expect_direct_response` or `expect_response_payload` observation may carry two
//! members beside its declarations, each omitted when closed:
//!
//! | member | what it opens |
//! |---|---|
//! | `undeclared_fields: "ignored"` | the response object itself, the command's root |
//! | `undeclared_fields_ignored: [<type>, …]` | each named `struct` declaration, wherever the response reaches it |
//!
//! At an opened object a reader admits keys the declaration does not name and reads nothing of
//! them. Every declared field keeps its presence and type check, so a missing or mistyped declared
//! field still fails. Every other object — a closed sibling struct, a union, the root of a closed
//! response — still refuses an undeclared key.
//!
//! # What selects it
//!
//! A suite one of whose response observations carries either member ([`used_by`]). Every other
//! suite keeps its bytes and its format. A reader that admits only through `/47` refuses a `/48`
//! suite by version, before any target callback, rather than enforcing closure the specification
//! no longer states; every reader refuses either member under a lower major.
//!
//! # Cumulative
//!
//! `/48` and `/49` are cumulative over every major below them, the constrained-response pair
//! (`/46`, `/47`) included.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use ess_compiler::ir::EssIr;
use ess_domain::types::UndeclaredFields;
use ess_domain::QualifiedName;

use crate::selection::Declaration;
use crate::{ConformanceSuite, ScenarioStep};

/// The ordinary suite major that carries opened response objects.
pub const ORDINARY: u32 = 48;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 49;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// The struct declarations among `declarations` that `ir` declares `undeclared_fields: ignored`,
/// by name. Empty, and then absent from an observation's bytes, where none is.
pub(crate) fn opened(
    ir: &EssIr,
    declarations: &BTreeMap<QualifiedName, Declaration>,
) -> BTreeSet<QualifiedName> {
    declarations
        .iter()
        .filter(|(name, declaration)| {
            matches!(declaration, Declaration::Struct { .. })
                && ir.undeclared_fields(name).is_ignored()
        })
        .map(|(name, _)| name.clone())
        .collect()
}

/// Admit the opened structs an observation carries against the declarations they travel with:
/// each names a `struct` declaration of the same observation.
pub(crate) fn validate(
    declarations: &BTreeMap<QualifiedName, Declaration>,
    opened: &BTreeSet<QualifiedName>,
) -> Result<(), String> {
    for name in opened {
        if !matches!(declarations.get(name), Some(Declaration::Struct { .. })) {
            return Err(format!(
                "undeclared_fields_ignored names `{name}`, which is no struct declaration of the \
                 response"
            ));
        }
    }
    Ok(())
}

/// A root `undeclared_fields` member that is present says `ignored`: `refused` is the closed
/// default, and writing it would be a second spelling of the bytes a closed observation has.
pub(crate) fn root_present<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<UndeclaredFields, D::Error> {
    match <UndeclaredFields as serde::Deserialize>::deserialize(d)? {
        UndeclaredFields::Ignored => Ok(UndeclaredFields::Ignored),
        UndeclaredFields::Refused => Err(serde::de::Error::custom(
            "a response's undeclared_fields, when present, is ignored",
        )),
    }
}

/// An `undeclared_fields_ignored` member that is present names at least one type, each once.
pub(crate) fn opened_present<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeSet<QualifiedName>, D::Error> {
    let names = <Vec<QualifiedName> as serde::Deserialize>::deserialize(d)?;
    if names.is_empty() {
        return Err(serde::de::Error::custom(
            "undeclared_fields_ignored, when present, names at least one type",
        ));
    }
    let count = names.len();
    let set: BTreeSet<_> = names.into_iter().collect();
    if set.len() != count {
        return Err(serde::de::Error::custom(
            "undeclared_fields_ignored names a type more than once",
        ));
    }
    Ok(set)
}

/// Whether a suite carries an opened response object (suite/48 and /49).
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::ExpectDirectResponse { response } => {
                response.undeclared_fields.is_ignored()
                    || !response.undeclared_fields_ignored.is_empty()
            }
            ScenarioStep::ExpectResponsePayload { response } => {
                response.undeclared_fields.is_ignored()
                    || !response.undeclared_fields_ignored.is_empty()
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

/// Refuse an explicitly pinned suite below `/48` that carries an opened response object.
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), crate::AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(crate::AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            "ignored undeclared response fields require suite/48 or /49",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ess_domain::{Field, TypeRef};

    fn declarations() -> BTreeMap<QualifiedName, Declaration> {
        BTreeMap::from([
            (
                "catalog.keys.PublicKey".parse().unwrap(),
                Declaration::Struct {
                    fields: vec![Field::new(
                        "kid",
                        TypeRef::Primitive(ess_domain::Primitive::String),
                    )],
                },
            ),
            (
                "catalog.keys.Use".parse().unwrap(),
                Declaration::Enum {
                    variants: vec!["sig".into()],
                },
            ),
        ])
    }

    #[test]
    fn an_opened_name_must_be_a_struct_of_the_same_observation() {
        let declarations = declarations();
        let struct_name: QualifiedName = "catalog.keys.PublicKey".parse().unwrap();
        assert_eq!(
            validate(&declarations, &BTreeSet::from([struct_name])),
            Ok(())
        );
        for name in ["catalog.keys.Use", "catalog.keys.Absent"] {
            let refused =
                validate(&declarations, &BTreeSet::from([name.parse().unwrap()])).unwrap_err();
            assert!(refused.contains(name), "{refused}");
        }
    }

    #[test]
    fn a_present_member_is_never_a_second_spelling_of_closed() {
        #[derive(serde::Deserialize, Debug)]
        struct Probe {
            #[serde(default, deserialize_with = "root_present")]
            #[allow(dead_code)]
            undeclared_fields: UndeclaredFields,
            #[serde(default, deserialize_with = "opened_present")]
            #[allow(dead_code)]
            undeclared_fields_ignored: BTreeSet<QualifiedName>,
        }
        for refused in [
            r#"{"undeclared_fields":"refused"}"#,
            r#"{"undeclared_fields_ignored":[]}"#,
            r#"{"undeclared_fields_ignored":["a.b.C","a.b.C"]}"#,
        ] {
            assert!(
                serde_json::from_str::<Probe>(refused).is_err(),
                "{refused} is admitted"
            );
        }
        for admitted in [
            "{}",
            r#"{"undeclared_fields":"ignored"}"#,
            r#"{"undeclared_fields_ignored":["a.b.C"]}"#,
        ] {
            serde_json::from_str::<Probe>(admitted).unwrap_or_else(|e| panic!("{admitted}: {e}"));
        }
    }
}
