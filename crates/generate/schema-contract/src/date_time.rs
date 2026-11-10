//! The RFC 3339 `date-time` format checker every validator built from this crate uses.
//!
//! The dependency's own checker tests all digit bytes at once with a borrowing subtraction, so
//! `+`, `-` and `.` pass as digits at some positions (`202+-10-29T12:00:00Z`). This checker
//! accepts exactly the strings that one accepts when every digit position holds an ASCII digit:
//! the separator is `T` or `t`, fractional seconds hold one or more digits, the offset is `Z`,
//! `z` or `±HH:MM`, and a leap second (`:60`) is valid only at 23:59 UTC.

/// Whether `text` is an RFC 3339 `date-time`.
#[must_use]
pub fn is_date_time(text: &str) -> bool {
    let bytes = text.as_bytes();
    let Some(split) = bytes.iter().position(|byte| matches!(byte, b'T' | b't')) else {
        return false;
    };
    is_date(&bytes[..split]) && is_time(&bytes[split + 1..])
}

fn digits(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0_u32, |value, byte| {
        byte.is_ascii_digit()
            .then(|| value * 10 + u32::from(byte - b'0'))
    })
}

fn is_date(bytes: &[u8]) -> bool {
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    let (Some(year), Some(month), Some(day)) = (
        digits(&bytes[..4]),
        digits(&bytes[5..7]),
        digits(&bytes[8..]),
    ) else {
        return false;
    };
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let last = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=last).contains(&day)
}

fn is_time(bytes: &[u8]) -> bool {
    if bytes.len() < 9 || bytes[2] != b':' || bytes[5] != b':' {
        return false;
    }
    let (Some(hour), Some(minute), Some(second)) = (
        digits(&bytes[..2]),
        digits(&bytes[3..5]),
        digits(&bytes[6..8]),
    ) else {
        return false;
    };
    if hour > 23 || minute > 59 || second > 60 {
        return false;
    }
    let mut at = 8;
    if bytes[at] == b'.' {
        at += 1;
        let start = at;
        while at < bytes.len() && bytes[at].is_ascii_digit() {
            at += 1;
        }
        if at == start {
            return false;
        }
    }
    let Some(&sign) = bytes.get(at) else {
        return false;
    };
    let offset: i64 = match sign {
        b'Z' | b'z' => {
            if at + 1 != bytes.len() {
                return false;
            }
            0
        }
        b'+' | b'-' => {
            let rest = &bytes[at + 1..];
            if rest.len() != 5 || rest[2] != b':' {
                return false;
            }
            let (Some(offset_hour), Some(offset_minute)) = (digits(&rest[..2]), digits(&rest[3..]))
            else {
                return false;
            };
            if offset_hour > 23 || offset_minute > 59 {
                return false;
            }
            let minutes = i64::from(offset_hour * 60 + offset_minute);
            if sign == b'+' {
                -minutes
            } else {
                minutes
            }
        }
        _ => return false,
    };
    if second == 60 {
        let utc = i64::from(hour * 60 + minute) + offset;
        return utc.rem_euclid(24 * 60) == 23 * 60 + 59;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::is_date_time;

    #[test]
    fn accepts_rfc_3339_and_refuses_what_it_does_not_spell() {
        for good in [
            "2026-10-29T12:00:00Z",
            "2026-10-29t12:00:00z",
            "2026-10-29T12:00:00.5+01:00",
            "2024-02-29T23:59:60Z",
            "2026-10-29T00:59:60+01:00",
        ] {
            assert!(is_date_time(good), "{good}");
        }
        for bad in [
            "",
            "2026-10-29",
            "2026-02-29T12:00:00Z",
            "2026-10-29T12:00:00",
            "2026-10-29T12:00:00.Z",
            "2026-10-29T12:00:60Z",
            "2026-10-29T24:00:00Z",
            "2026-10-29T12:00:00+0100",
            "2026-10-29T12:00:00Zx",
            "２０２６-10-29T12:00:00Z",
        ] {
            assert!(!is_date_time(bad), "{bad}");
        }
    }
}
