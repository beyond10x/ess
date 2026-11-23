//! How the Rust target carries a finite `Binary64`.

/// `true` once this target's codec carries a finite `Binary64` in every carried position
/// (`crate::failure::binary64` reads it). Flipped by the unit that adds the codec, never before
/// the generated code compiles.
pub(crate) const CARRIED: bool = true;

/// Whether the model has a `Binary64` anywhere.
///
/// The wrapper and the wire helper are emitted only then, so every other model keeps its bytes.
pub(crate) fn used(ir: &ess_compiler::EssIr) -> bool {
    !ess_compiler::binary64::locations(ir).is_empty()
}

/// The text appended to the generated `primitives.rs`.
pub(crate) const PRIMITIVE: &str = r#"
/// A finite IEEE 754 binary64, preserving the sign of zero.
///
/// The field is private, so a NaN or an infinity cannot be held: [`Binary64::new`] refuses
/// both. Equality is IEEE 754's, so `+0` and `-0` are equal, and `Eq` is lawful because no NaN
/// can be held. There is no `Hash` and no `Ord`: guards and orders compare the decimal
/// `Display` writes. The sign of zero stays readable through [`Binary64::get`],
/// [`Binary64::token`] and `Display`.
#[derive(Debug, Clone, Copy)]
pub struct Binary64(f64);

/// Why a value is not a [`Binary64`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binary64Error {
    /// A NaN or an infinity.
    NonFinite,
}

impl std::fmt::Display for Binary64Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Binary64 requires a finite value")
    }
}

impl std::error::Error for Binary64Error {}

impl Binary64 {
    /// The value, when it is finite.
    ///
    /// # Errors
    ///
    /// [`Binary64Error::NonFinite`] for a NaN or an infinity.
    pub fn new(value: f64) -> Result<Self, Binary64Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(Binary64Error::NonFinite)
        }
    }

    /// The wrapped value.
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }

    /// A JSON number token, rounded once to the nearest binary64; `None` where it rounds to
    /// an infinity or spells no number.
    #[must_use]
    pub fn from_token(token: &str) -> Option<Self> {
        token.parse::<f64>().ok().and_then(|value| Self::new(value).ok())
    }

    /// The JSON number the wire writes: the shortest digits that read back as the value,
    /// laid out as `3.0`, `-0.0`, `0.73`, `1e16` and `1e-6` are.
    #[must_use]
    pub fn token(&self) -> String {
        let value = self.0;
        if value == 0.0 {
            return if value.is_sign_negative() { "-0.0" } else { "0.0" }.to_owned();
        }
        let sign = if value < 0.0 { "-" } else { "" };
        let scientific = format!("{:e}", value.abs());
        let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
        let digits = mantissa.replacen('.', "", 1);
        let power: i32 = exponent.parse().unwrap_or(0);
        let point = power + 1;
        let count = i32::try_from(digits.len()).unwrap_or(i32::MAX);
        let zeros = |n: i32| "0".repeat(usize::try_from(n).unwrap_or(0));
        if count <= point && point <= 16 {
            format!("{sign}{digits}{}.0", zeros(point - count))
        } else if 0 < point && point <= 16 {
            let (whole, fraction) = digits.split_at(usize::try_from(point).unwrap_or(0));
            format!("{sign}{whole}.{fraction}")
        } else if -5 < point && point <= 0 {
            format!("{sign}0.{}{digits}", zeros(-point))
        } else if count == 1 {
            format!("{sign}{digits}e{power}")
        } else {
            format!("{sign}{}.{}e{power}", &digits[..1], &digits[1..])
        }
    }
}

impl PartialEq for Binary64 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Binary64 {}

impl std::fmt::Display for Binary64 {
    /// The shortest plain decimal that reads back as the value: `-0`, `0.0000001`, never an
    /// exponent. Guards, filters, invariants and orders compare this text by value.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
"#;

/// The helper the wire module reads a `Binary64` through, with `types` the name of the types
/// crate.
pub(crate) fn wire_helper(types: &str) -> String {
    WIRE_HELPER.replace("TYPES", types)
}

const WIRE_HELPER: &str = r#"
/// The finite `Binary64` at this path: a JSON number, rounded once to the nearest binary64,
/// and refused where it rounds to an infinity.
#[allow(dead_code)]
fn binary64_at(value: &json::Value, at: &str) -> Result<TYPES::primitives::Binary64, json::DecodeError> {
    match value {
        json::Value::Number(token) => TYPES::primitives::Binary64::from_token(token).ok_or_else(|| json::DecodeError {
            at: at.to_owned(),
            expected: "a finite Binary64 number".to_owned(),
            found: format!("the number {token}"),
        }),
        other => Err(json::DecodeError::of(at, "a finite Binary64 number", other)),
    }
}
"#;
