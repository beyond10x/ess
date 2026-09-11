//! Coordinate comparison from existing event values and adapter-observed clock evidence.
use crate::{
    scenario::EventRef,
    target::{ConformanceTarget, ObservedEvent, TargetError},
};
use ess_compiler::{ir::ResolvedTypeRef, EssIr};
use ess_domain::reading::{
    compare_readings, resolve_reading, ReadingContract, ReadingEncoding, ReadingError,
};
use ess_primitives::{ids::CorrelationId, node::Node};

/// One already observed event member and the declaration the adapter must independently check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingReference {
    /// Event that supplied the scalar.
    pub event: EventRef,
    /// Zero-based occurrence among observed events of this declared event type.
    pub occurrence: u32,
    /// Declared member name, not a wire alias.
    pub field: String,
    /// The nominal reading type declaring the encoding.
    pub declared_type: ess_domain::QualifiedName,
    /// Declared requirements, never a certificate of runtime evidence.
    pub contract: ReadingContract,
}
impl ReadingReference {
    /// Obtain the retained reading attachment through bounded transparent newtype wrappers.
    pub fn from_event(
        ir: &EssIr,
        event: EventRef,
        occurrence: u32,
        field: &str,
    ) -> Result<Self, String> {
        let fields = &ir
            .events()
            .get(event.name())
            .ok_or("undeclared reading event")?
            .fields;
        let mut kind = &fields
            .iter()
            .find(|value| value.name == field)
            .ok_or("undeclared reading field")?
            .type_ref;
        for _ in 0..=ess_domain::types::MAX_TYPE_DEPTH {
            let ResolvedTypeRef::Declared { name } = kind else {
                return Err("member has no nominal reading contract".into());
            };
            let declared = ir.named_type(name);
            if let Some(contract) = &declared.reading {
                let reference = Self {
                    event,
                    occurrence,
                    field: field.into(),
                    declared_type: declared.name.clone(),
                    contract: contract.clone(),
                };
                reference.validate()?;
                return Ok(reference);
            }
            let ess_compiler::ir::ResolvedBody::Newtype { of, .. } = &declared.body else {
                return Err("member has no reading contract".into());
            };
            kind = of;
        }
        Err("reading wrapper depth exceeded".into())
    }
    /// Stable member occurrence within the runner's scenario correlation.
    pub fn occurrence_key(&self) -> String {
        format!("{}#{}.{}", self.event, self.occurrence, self.field)
    }
    /// Structural persisted admission; the adapter retains authority for its actual declared model.
    pub fn validate(&self) -> Result<(), String> {
        if self.occurrence > 65_535
            || self.field.is_empty()
            || self.field.len() > 128
            || !self.field.starts_with(|c: char| c.is_ascii_alphabetic())
            || !self
                .field
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            || self.occurrence_key().len() > 512
        {
            return Err("invalid bounded reading occurrence/member".into());
        }
        self.contract.validate()
    }
}

/// Expected coordinate ordering; it is not elapsed physical time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingOrder {
    /// The first coordinate is smaller.
    Before,
    /// The coordinates are equal.
    Equal,
    /// The first coordinate is greater.
    After,
}

/// Scope of facts requested from the actual target adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadingObservationRequest {
    /// The actual event occurrence/member and its claimed declaration to validate.
    pub reading: ReadingReference,
    /// Current scenario correlation; evidence from another scenario is invalid.
    pub correlation: CorrelationId,
}

pub(crate) fn compare<T: ConformanceTarget>(
    left: &ReadingReference,
    right: &ReadingReference,
    order: ReadingOrder,
    seen: &[ObservedEvent],
    correlation: &CorrelationId,
    target: &T,
) -> Result<bool, TargetError> {
    let resolve = |reference: &ReadingReference| {
        reference
            .validate()
            .map_err(|detail| TargetError::unavailable("clock reading", detail))?;
        let event = seen
            .iter()
            .filter(|event| event.event == reference.event)
            .nth(reference.occurrence as usize)
            .ok_or_else(|| {
                TargetError::unavailable(
                    "clock reading",
                    "referenced event occurrence has not been observed",
                )
            })?;
        let value = event.payload.get(&reference.field).ok_or_else(|| {
            TargetError::unavailable("clock reading", "declared event member is absent")
        })?;
        let scalar = match (&reference.contract.encoding, value) {
            (ReadingEncoding::UnixSeconds, Node::Number(value)) => {
                value.as_i64().map(|value| value.to_string())
            }
            (
                ReadingEncoding::OffsetDateTimeText | ReadingEncoding::LocalDateTimeMillisLiteralZ,
                Node::Text(value),
            ) => Some(value.clone()),
            _ => None,
        }
        .ok_or_else(|| {
            TargetError::unavailable(
                "clock reading",
                "reading scalar does not match its encoding",
            )
        })?;
        let evidence = target.observe_clock_reading(ReadingObservationRequest {
            reading: reference.clone(),
            correlation: correlation.clone(),
        })?;
        resolve_reading(
            &reference.contract,
            &scalar,
            &evidence,
            &correlation.to_string(),
            &reference.occurrence_key(),
        )
        .map_err(reading_error)
    };
    let ordering = compare_readings(&resolve(left)?, &resolve(right)?).map_err(reading_error)?;
    Ok(matches!(
        (order, ordering),
        (ReadingOrder::Before, std::cmp::Ordering::Less)
            | (ReadingOrder::Equal, std::cmp::Ordering::Equal)
            | (ReadingOrder::After, std::cmp::Ordering::Greater)
    ))
}
fn reading_error(error: ReadingError) -> TargetError {
    match error {
        ReadingError::UnknownEvidence | ReadingError::DifferentClock => {
            TargetError::unsupported("clock reading", error.to_string())
        }
        _ => TargetError::unavailable("clock reading", error.to_string()),
    }
}
/// Whether the suite requires the coordinated new reading observation vocabulary.
pub fn used_by(suite: &crate::ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario
            .steps
            .iter()
            .any(|step| matches!(step, crate::ScenarioStep::ExpectReadingOrder { .. }))
    })
}
