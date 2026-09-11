//! Evidence checks shared verbatim with the dependency-free native Rust emitter.

/// Facts independently observed for a single reading occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockReadingEvidence {
    /// Scenario correlation supplied by the caller.
    pub correlation: String,
    /// Requested event occurrence and member.
    pub occurrence: String,
    /// Actual process instance; empty means unknown.
    pub process_instance: String,
    /// Actual clock epoch; empty means unknown.
    pub epoch: String,
    /// Observed `producer_process` or `consumer_process` role; unknown refuses.
    pub origin: String,
    /// Observed `encoded_offset`, `unix_seconds` or `fixed_offset` mode.
    pub formatter: String,
    /// Independently observed minutes east of UTC for `fixed_offset` only.
    pub offset_minutes: Option<i16>,
}

/// Coordinate bound to the actual observed source epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockCoordinate {
    /// Exact normalized milliseconds; this is not elapsed physical time.
    pub unix_millis: i64,
    /// Actual observed process instance.
    pub process_instance: String,
    /// Actual observed epoch.
    pub epoch: String,
}

/// Resolve a declared reading using occurrence-scoped facts, without consulting a live clock.
pub fn resolve_clock_reading(
    encoding: &str,
    origins: &[(&str, &str)],
    value: &str,
    evidence: &ClockReadingEvidence,
    correlation: &str,
    occurrence: &str,
) -> Result<ClockCoordinate, super::ReadingError> {
    use super::ReadingError as E;
    if evidence.correlation != correlation || evidence.occurrence != occurrence {
        return Err(E::MismatchedOccurrence);
    }
    if [
        &evidence.correlation,
        &evidence.occurrence,
        &evidence.process_instance,
        &evidence.epoch,
    ]
    .into_iter()
    .any(|value| value.is_empty() || value.len() > 512)
        || !matches!(
            evidence.origin.as_str(),
            "producer_process" | "consumer_process"
        )
    {
        return Err(E::UnknownEvidence);
    }
    if origins.is_empty() || origins.len() > 3 {
        return Err(E::InvalidContract);
    }
    for (index, origin) in origins.iter().enumerate() {
        if origins[..index].contains(origin)
            || !matches!(
                origin.0,
                "producer_process" | "consumer_process" | "unknown"
            )
            || !matches!(
                (encoding, origin.1),
                ("offset_date_time_text", "encoded_offset")
                    | ("unix_seconds", "encoding_defined_epoch")
                    | (
                        "local_date_time_millis_literal_z",
                        "requires_observation" | "unknown"
                    )
            )
        {
            return Err(E::InvalidContract);
        }
    }
    let authority = match (encoding, evidence.formatter.as_str()) {
        ("offset_date_time_text", "encoded_offset") if evidence.offset_minutes.is_none() => {
            "encoded_offset"
        }
        ("unix_seconds", "unix_seconds") if evidence.offset_minutes.is_none() => {
            "encoding_defined_epoch"
        }
        ("local_date_time_millis_literal_z", "fixed_offset")
            if evidence.offset_minutes.is_some() =>
        {
            "requires_observation"
        }
        (_, "unknown") => return Err(E::UnknownEvidence),
        _ => return Err(E::IncompatibleFormatter),
    };
    if !origins.contains(&(evidence.origin.as_str(), authority)) {
        return Err(E::IncompatibleOrigin);
    }
    let unix_millis = match encoding {
        "unix_seconds" => {
            let seconds = value.parse::<i64>().map_err(|_| E::InvalidValue)?;
            if seconds.to_string() != value {
                return Err(E::InvalidValue);
            }
            super::normalize_unix(seconds)?
        }
        "offset_date_time_text" => super::normalize_text(value, None)?,
        "local_date_time_millis_literal_z" => {
            super::normalize_text(value, evidence.offset_minutes)?
        }
        _ => return Err(E::InvalidContract),
    };
    Ok(ClockCoordinate {
        unix_millis,
        process_instance: evidence.process_instance.clone(),
        epoch: evidence.epoch.clone(),
    })
}

/// Compare coordinates only within the same actually observed source epoch.
pub fn compare_clock_readings(
    left: &ClockCoordinate,
    right: &ClockCoordinate,
) -> Result<std::cmp::Ordering, super::ReadingError> {
    if left.process_instance.is_empty()
        || left.epoch.is_empty()
        || right.process_instance.is_empty()
        || right.epoch.is_empty()
    {
        return Err(super::ReadingError::UnknownEvidence);
    }
    if left.process_instance != right.process_instance || left.epoch != right.epoch {
        return Err(super::ReadingError::DifferentClock);
    }
    Ok(left.unix_millis.cmp(&right.unix_millis))
}
