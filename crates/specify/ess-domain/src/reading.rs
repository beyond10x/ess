//! Declared clock readings and occurrence-scoped observation authority.

mod coordinate;
mod normalize;
pub use coordinate::{
    compare_clock_readings, resolve_clock_reading, ClockCoordinate, ClockReadingEvidence,
};
pub use normalize::{normalize_text, normalize_unix, ReadingError};

/// Wire representation of a clock reading, separate from its observed source.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ReadingEncoding {
    /// Text carries its effective numeric offset or UTC marker.
    OffsetDateTimeText,
    /// Millisecond local text has a literal, non-authoritative Z suffix.
    LocalDateTimeMillisLiteralZ,
    /// Exact integer seconds since the Unix coordinate origin.
    UnixSeconds,
}

/// Possible production role; a declaration does not observe which alternative occurred.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ReadingRole {
    /// The producing process supplied the reading.
    ProducerProcess,
    /// The consuming process supplied a fallback reading.
    ConsumerProcess,
    /// The original source branch is unavailable.
    Unknown,
}

/// Where the offset interpretation can come from.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum OffsetAuthority {
    /// The declared encoding actually encodes the offset.
    EncodedOffset,
    /// Integer Unix seconds define the coordinate, not the source clock identity.
    EncodingDefinedEpoch,
    /// A fixed formatter offset must be observed for this occurrence.
    RequiresObservation,
    /// Offset authority is unavailable.
    Unknown,
}

/// One declared origin alternative.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReadingOrigin {
    /// Which process role could have produced the value.
    pub role: ReadingRole,
    /// What offset evidence is required.
    pub offset: OffsetAuthority,
}

/// Closed attachment to a directly String/Integer-backed named newtype.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReadingContract {
    /// Scalar representation.
    pub encoding: ReadingEncoding,
    /// At most three distinct origin alternatives; this is not runtime evidence.
    pub origins: Vec<ReadingOrigin>,
}

impl ReadingContract {
    /// Refuse contradictory, empty or duplicate declaration alternatives.
    pub fn validate(&self) -> Result<(), String> {
        if self.origins.is_empty() || self.origins.len() > 3 {
            return Err("reading requires one to three origin alternatives".into());
        }
        for (index, origin) in self.origins.iter().enumerate() {
            if self.origins[..index].contains(origin) {
                return Err("duplicate reading origin alternative".into());
            }
            let compatible = match self.encoding {
                ReadingEncoding::OffsetDateTimeText => {
                    origin.offset == OffsetAuthority::EncodedOffset
                }
                ReadingEncoding::UnixSeconds => {
                    origin.offset == OffsetAuthority::EncodingDefinedEpoch
                }
                ReadingEncoding::LocalDateTimeMillisLiteralZ => matches!(
                    origin.offset,
                    OffsetAuthority::RequiresObservation | OffsetAuthority::Unknown
                ),
            };
            if !compatible {
                return Err("reading encoding and offset authority contradict each other".into());
            }
        }
        Ok(())
    }

    /// The only representation admitted for this encoding.
    pub const fn representation(&self) -> crate::types::Primitive {
        match self.encoding {
            ReadingEncoding::UnixSeconds => crate::types::Primitive::Integer,
            _ => crate::types::Primitive::String,
        }
    }
}

/// Observed identity of one process clock epoch; neither field is an authored guarantee.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceEpoch {
    /// Actual process instance observed by the adapter.
    pub process_instance: String,
    /// Epoch within that instance; a restart/reset requires distinct evidence.
    pub epoch: String,
}

/// Actual formatter mode observed for an occurrence.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FormatterObservation {
    /// Text was produced with an encoded effective offset.
    EncodedOffset,
    /// Exact integer Unix seconds were produced.
    UnixSeconds,
    /// A literal-Z formatter used this independently observed fixed offset.
    FixedOffset {
        /// Minutes east of UTC, within ±840.
        minutes: i16,
    },
    /// Formatter configuration or initialization order was not observed.
    Unknown,
}

/// Adapter facts tied to one scenario and already observed event occurrence/member.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingEvidence {
    /// Current scenario correlation.
    pub correlation: String,
    /// Exact occurrence key requested by the runner.
    pub occurrence: String,
    /// Unknown when the process clock/epoch was not observed.
    pub source: Option<SourceEpoch>,
    /// Unknown when producer versus fallback was erased.
    pub origin: ReadingRole,
    /// Observed mode, not an expected normalized value.
    pub formatter: FormatterObservation,
}

/// A normalized coordinate bound to the observed process and epoch that produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedReading {
    /// Milliseconds in the admitted Unix coordinate range.
    pub unix_millis: i64,
    /// Matching identity is required before comparing readings.
    pub source: SourceEpoch,
}

/// Check the trusted adapter's scope and declared mode before interpreting the scalar.
pub fn resolve_reading(
    contract: &ReadingContract,
    value: &str,
    evidence: &ReadingEvidence,
    correlation: &str,
    occurrence: &str,
) -> Result<ResolvedReading, ReadingError> {
    contract
        .validate()
        .map_err(|_| ReadingError::InvalidContract)?;
    let source = evidence
        .source
        .as_ref()
        .ok_or(ReadingError::UnknownEvidence)?;
    let role = |role: ReadingRole| match role {
        ReadingRole::ProducerProcess => "producer_process",
        ReadingRole::ConsumerProcess => "consumer_process",
        ReadingRole::Unknown => "unknown",
    };
    let origins = contract
        .origins
        .iter()
        .map(|origin| {
            (
                role(origin.role),
                match origin.offset {
                    OffsetAuthority::EncodedOffset => "encoded_offset",
                    OffsetAuthority::EncodingDefinedEpoch => "encoding_defined_epoch",
                    OffsetAuthority::RequiresObservation => "requires_observation",
                    OffsetAuthority::Unknown => "unknown",
                },
            )
        })
        .collect::<Vec<_>>();
    let (formatter, offset_minutes) = match evidence.formatter {
        FormatterObservation::EncodedOffset => ("encoded_offset", None),
        FormatterObservation::UnixSeconds => ("unix_seconds", None),
        FormatterObservation::FixedOffset { minutes } => ("fixed_offset", Some(minutes)),
        FormatterObservation::Unknown => ("unknown", None),
    };
    let result = resolve_clock_reading(
        match contract.encoding {
            ReadingEncoding::OffsetDateTimeText => "offset_date_time_text",
            ReadingEncoding::LocalDateTimeMillisLiteralZ => "local_date_time_millis_literal_z",
            ReadingEncoding::UnixSeconds => "unix_seconds",
        },
        &origins,
        value,
        &ClockReadingEvidence {
            correlation: evidence.correlation.clone(),
            occurrence: evidence.occurrence.clone(),
            process_instance: source.process_instance.clone(),
            epoch: source.epoch.clone(),
            origin: role(evidence.origin).into(),
            formatter: formatter.into(),
            offset_minutes,
        },
        correlation,
        occurrence,
    )?;
    Ok(ResolvedReading {
        unix_millis: result.unix_millis,
        source: source.clone(),
    })
}

/// Order coordinates of the same observed process epoch; cross-source calibration is unsupported.
pub fn compare_readings(
    left: &ResolvedReading,
    right: &ResolvedReading,
) -> Result<std::cmp::Ordering, ReadingError> {
    if left.source != right.source {
        return Err(ReadingError::DifferentClock);
    }
    Ok(left.unix_millis.cmp(&right.unix_millis))
}
