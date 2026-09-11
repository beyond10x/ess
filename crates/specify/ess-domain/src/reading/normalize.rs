//! Fixed grammar and range shared with generated native Rust helpers.

/// Closed reasons a clock reading cannot establish the requested coordinate comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingError {
    /// The attachment contradicts its encoding/authority contract.
    InvalidContract,
    /// The scalar does not follow the admitted grammar.
    InvalidValue,
    /// A valid-looking value exceeds the declared bounded coordinate/offset range.
    OutOfRange,
    /// Required source, epoch, origin or formatter facts were not observed.
    UnknownEvidence,
    /// Evidence belongs to a different scenario or occurrence.
    MismatchedOccurrence,
    /// The observed production branch is not a declared alternative.
    IncompatibleOrigin,
    /// The observed formatter mode does not implement the declared encoding.
    IncompatibleFormatter,
    /// Comparing different observed clocks requires unsupported calibration.
    DifferentClock,
}

impl std::fmt::Display for ReadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ReadingError {}

/// Exact epoch seconds in the admitted 1970–9999 range, with no floating-point conversion.
pub fn normalize_unix(seconds: i64) -> Result<i64, ReadingError> {
    if !(0..=253_402_300_799).contains(&seconds) {
        return Err(ReadingError::OutOfRange);
    }
    Ok(seconds * 1000)
}

/// Parse offset text, or literal-Z millisecond text with an observed offset east of UTC.
pub fn normalize_text(text: &str, local_offset_minutes: Option<i16>) -> Result<i64, ReadingError> {
    let b = text.as_bytes();
    if !(20..=29).contains(&b.len()) || !b.is_ascii() {
        return Err(ReadingError::InvalidValue);
    }
    if b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
        return Err(ReadingError::InvalidValue);
    }
    let year = digits(&b[0..4])?;
    let month = digits(&b[5..7])?;
    let day = digits(&b[8..10])?;
    let hour = digits(&b[11..13])?;
    let minute = digits(&b[14..16])?;
    let second = digits(&b[17..19])?;
    if !(1970..=9999).contains(&year) {
        return Err(ReadingError::OutOfRange);
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let month_index =
        usize::try_from(month.saturating_sub(1)).map_err(|_| ReadingError::InvalidValue)?;
    if !(1..=12).contains(&month)
        || day < 1
        || day > month_days[month_index]
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(ReadingError::InvalidValue);
    }
    let (millis, suffix) = if b[19] == b'.' {
        if b.len() < 24 {
            return Err(ReadingError::InvalidValue);
        }
        (digits(&b[20..23])?, &b[23..])
    } else {
        (0, &b[19..])
    };
    let offset = if let Some(minutes) = local_offset_minutes {
        if b[19] != b'.' || suffix != b"Z" {
            return Err(ReadingError::InvalidValue);
        }
        i64::from(minutes)
    } else if suffix == b"Z" {
        0
    } else {
        if suffix.len() != 6 || !matches!(suffix[0], b'+' | b'-') || suffix[3] != b':' {
            return Err(ReadingError::InvalidValue);
        }
        let hours = digits(&suffix[1..3])?;
        let minutes = digits(&suffix[4..6])?;
        if hours > 14 || minutes > 59 || (hours == 14 && minutes != 0) {
            return Err(ReadingError::OutOfRange);
        }
        (hours * 60 + minutes) * if suffix[0] == b'-' { -1 } else { 1 }
    };
    if !(-840..=840).contains(&offset) {
        return Err(ReadingError::OutOfRange);
    }
    let years = year - 1970;
    let through = year - 1;
    let leaps = through / 4 - through / 100 + through / 400 - (1969 / 4 - 1969 / 100 + 1969 / 400);
    let before_month: i64 = month_days[..month_index].iter().sum();
    let days = years * 365 + leaps + before_month + day - 1;
    let coordinate = (((days * 24 + hour) * 60 + minute - offset) * 60 + second) * 1000 + millis;
    if !(0..=253_402_300_799_999).contains(&coordinate) {
        return Err(ReadingError::OutOfRange);
    }
    Ok(coordinate)
}

fn digits(bytes: &[u8]) -> Result<i64, ReadingError> {
    bytes.iter().try_fold(0, |n, byte| {
        if byte.is_ascii_digit() {
            Ok(n * 10 + i64::from(byte - b'0'))
        } else {
            Err(ReadingError::InvalidValue)
        }
    })
}
