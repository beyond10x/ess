//! Unsupported finite-number contracts are refused before publication or target interaction.

use crate::{ConformanceSuite, Holds, ScenarioStep};
use ess_domain::Primitive;
use std::fmt;

/// An unsupported conformance type contract, carrying all located uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionError {
    /// Model or suite locations that require a newer qualified consumer contract.
    pub locations: Vec<String>,
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: finite Binary64 is not admitted by the current conformance suite and codecs",
            self.locations.join(", ")
        )
    }
}
impl std::error::Error for AdmissionError {}

/// Check the model even when synthesis would omit unsupported fields or whole scenarios.
pub fn model(ir: &ess_compiler::EssIr) -> Result<(), AdmissionError> {
    checked(ess_compiler::binary64::locations(ir).into_iter().collect())
}

/// Check directly constructed suites before artifact creation or target effects.
pub fn suite(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    let mut found = Vec::new();
    for (id, scenario) in &suite.scenarios {
        for (index, step) in scenario.steps.iter().enumerate() {
            if let ScenarioStep::ExpectEvent { shape, .. }
            | ScenarioStep::EventuallyEvent { shape, .. } = step
            {
                for (name, leaf) in shape.leaves() {
                    if matches!(
                        leaf.holds,
                        Holds::Primitive {
                            kind: Primitive::Binary64
                        }
                    ) {
                        found.push(format!(
                            "/scenarios/{}/steps/{index}/shape/{}",
                            escape(&id.to_string()),
                            escape(name)
                        ));
                    }
                }
            }
        }
    }
    checked(found)
}

fn escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
fn checked(locations: Vec<String>) -> Result<(), AdmissionError> {
    if locations.is_empty() {
        Ok(())
    } else {
        Err(AdmissionError { locations })
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)] // Serde serialize_with requires a borrowed field.
pub(crate) fn serialize_primitive<S: serde::Serializer>(
    primitive: &Primitive,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if *primitive == Primitive::Binary64 {
        return Err(serde::ser::Error::custom(
            "Binary64 is not admitted by this conformance suite format",
        ));
    }
    serde::Serialize::serialize(primitive, serializer)
}
pub(crate) fn deserialize_primitive<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Primitive, D::Error> {
    let primitive = <Primitive as serde::Deserialize>::deserialize(deserializer)?;
    if primitive == Primitive::Binary64 {
        return Err(serde::de::Error::custom(
            "Binary64 is not admitted by this conformance suite format",
        ));
    }
    Ok(primitive)
}
