//! Explicit binary64 conversion, shared with emitted adapters; exact JSON remains the default.

use serde_json::{value::RawValue, Value};

use super::{finding, Binary64Range, Binary64Step, Refused};

pub(super) fn literal(text: &str) -> Option<f64> {
    let raw: Box<RawValue> = serde_json::from_str(text).ok()?;
    if raw.get() != text || !matches!(text.as_bytes().first(), Some(b'-' | b'0'..=b'9')) {
        return None;
    }
    finite(text)
}

pub(super) fn finite(text: &str) -> Option<f64> {
    text.parse::<f64>().ok().filter(|value| value.is_finite())
}

pub(super) fn convert(
    value: &Value,
    steps: &[Binary64Step],
    range: Binary64Range,
    at: &str,
) -> Result<Value, Refused> {
    let mut value = value
        .as_number()
        .and_then(|number| finite(&number.to_string()))
        .ok_or_else(|| {
            error(
                &format!("{at}/value"),
                "binary64_value",
                "expected a finite numeric value",
            )
        })?;
    for (index, step) in steps.iter().enumerate() {
        let at = format!("{at}/steps/{index}");
        let (Binary64Step::Multiply { value: token }
        | Binary64Step::Minimum { value: token }
        | Binary64Step::Maximum { value: token }) = step;
        let operand = literal(token).ok_or_else(|| {
            error(
                &format!("{at}/value"),
                "binary64_literal",
                "expected a finite JSON numeric token",
            )
        })?;
        value = match step {
            Binary64Step::Multiply { .. } => value * operand,
            Binary64Step::Minimum { .. } => minimum(value, operand),
            Binary64Step::Maximum { .. } => maximum(value, operand),
        };
        if !value.is_finite() {
            return Err(error(
                &at,
                "binary64_overflow",
                "binary64 intermediate result is not finite",
            ));
        }
    }
    match range {
        Binary64Range::Reject => truncate(value, at),
    }
}

fn minimum(left: f64, right: f64) -> f64 {
    if left == 0.0 && right == 0.0 {
        f64::from_bits(left.to_bits() | right.to_bits())
    } else {
        left.min(right)
    }
}

fn maximum(left: f64, right: f64) -> f64 {
    if left == 0.0 && right == 0.0 {
        f64::from_bits(left.to_bits() & right.to_bits())
    } else {
        left.max(right)
    }
}

#[allow(clippy::cast_possible_truncation)] // The explicit range check precedes intentional truncation.
fn truncate(value: f64, at: &str) -> Result<Value, Refused> {
    if !(-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&value) {
        return Err(error(
            at,
            "binary64_range",
            "truncated binary64 value is outside signed 64-bit representation",
        ));
    }
    Ok(Value::Number((value as i64).into()))
}

fn error(at: &str, rule: &str, detail: &str) -> Refused {
    Refused(vec![finding(at, rule, detail)])
}

#[cfg(test)]
mod tests {
    use super::{literal, maximum, minimum};

    #[test]
    fn rounding_matches_independently_observed_binary64_bits() {
        for (text, bits) in [
            ("1.001", 0x3ff0_0418_9374_bc6a),
            ("9007199254740993", 0x4340_0000_0000_0000),
            ("9007199254740995", 0x4340_0000_0000_0002),
            ("1e-999", 0),
            ("-1e-999", 0x8000_0000_0000_0000),
            ("5e-324", 1),
        ] {
            assert_eq!(literal(text).unwrap().to_bits(), bits, "{text}");
        }
    }

    #[test]
    fn zero_clamps_are_sign_stable_in_either_operand_order() {
        for (left, right) in [(0.0, -0.0), (-0.0, 0.0)] {
            assert_eq!(minimum(left, right).to_bits(), (-0.0_f64).to_bits());
            assert_eq!(maximum(left, right).to_bits(), 0.0_f64.to_bits());
        }
    }
}
