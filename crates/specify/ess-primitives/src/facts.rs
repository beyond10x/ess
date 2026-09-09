//! Facts: the observable values predicates are evaluated against.
//!
//! A fact is a dotted path bound to a scalar value, such as `tests.unit.failed = 0`. Facts
//! are *projected* from evidence and from the artifact graph — the engine never invents one
//! — and a predicate can only reference paths the protocol declares observable, so a
//! completion condition cannot quietly depend on something nothing produces.
//!
//! # Ordered scales
//!
//! Some facts are ordered but not numeric (`risk`, `severity`). A protocol declares scales
//! so that `risk >= medium` has a defined meaning; without a declared scale, ordering
//! comparisons on strings evaluate to [`Unknown`](crate::predicate::Truth::Unknown) rather
//! than to an arbitrary lexicographic answer.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;

use crate::error::ParseError;

/// A number that is always finite, so it can be totally ordered, compared for equality, and
/// written back out as the value it was read as.
///
/// NaN is refused because it makes the ordering partial, and the infinities are refused for a
/// reason that only shows up on the way out: JSON has no spelling for them, so `serde_json` writes
/// either one as `null`. A guard reading `amount >= 1e400` would then be published as
/// `any_of: [null]` — not a crash, and not a refusal, but a document that says something the author
/// never wrote. Refusing at the door is the only place that difference is still visible.
///
/// # Exact, and still spelt the way it always was
///
/// This was `Number(f64)`, which made `9007199254740993` and `9007199254740992` one value, made
/// `Number::from(i64::MAX)` a number that is not `i64::MAX`, and made
/// [`is_integral`](Self::is_integral) answer `false` for most of the integers `Primitive::Integer`
/// declares admissible. Review finding F08.
///
/// A number now carries an exact decimal — `units × 10⁻ˢᶜᵃˡᵉ` — **and** the binary64 it has always
/// been written as, and [`Serialize`](serde::Serialize) writes the second one. That is what makes
/// the exactness free of byte consequences: there is no value for which the serializer can produce
/// bytes different from the ones it produced before, because it serializes the same `f64` it always
/// did. `docs/design/review-primitive-semantics.md` states which spellings that freezes, and which
/// stage changes them.
#[derive(Debug, Clone, Copy)]
pub struct Number(Repr);

/// How one number is held.
///
/// `Exact` covers every `i64`, every `usize`, and every decimal literal whose digits fit an `i128`
/// with a scale under 256 — which is every value `Primitive::Integer` and `Primitive::Decimal`
/// name. `Binary64` is the remainder: a magnitude with no short decimal spelling, where the value
/// has never been anything but its binary64 anyway.
#[derive(Debug, Clone, Copy)]
enum Repr {
    /// `units × 10^-scale`, beside the binary64 this value serialises as.
    Exact {
        /// The scaled integer. Normalised: `scale` is the smallest one that spells this value.
        units: i128,
        /// The number of decimal places.
        scale: u8,
        /// What every writer of this value has always written.
        binary: f64,
    },
    /// A value outside the exact range, held as it always was.
    Binary64(f64),
}

/// The magnitude below which an integral binary64 is carried as the integer it is.
///
/// `2^63`. Above it the shortest round-tripping decimal is used, which is what `f64`'s own
/// `Display` prints and is far outside anything `Primitive::Integer` admits.
const INTEGER_CARRIER: f64 = 9_223_372_036_854_775_808.0;

/// The largest scale an exact decimal is held at.
///
/// A literal needing more places than this is a binary64 and says so, rather than being silently
/// truncated to a value the author did not write.
const MAX_SCALE: u32 = 255;

impl<'de> serde::Deserialize<'de> for Number {
    /// Reads an integer token as the integer, and everything else through [`Number::new`].
    ///
    /// Hand-written rather than derived: `#[serde(transparent)]` with a derived implementation
    /// reads straight into the field and never calls the constructor, which is how `.nan` in a
    /// document produced a `Number` this type's own documentation says cannot exist.
    ///
    /// An integer token is read exactly because [`Serialize`](serde::Serialize) writes one exactly
    /// — the two doors are one decision, and moving only the reader is what made
    /// `9223372036854775807` a value that was admitted, written, and then refused. See the
    /// round-trip law in `docs/design/review-primitive-semantics.md`.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(NumberVisitor)
    }
}

/// Reads whichever token a self-describing format hands over.
struct NumberVisitor;

impl serde::de::Visitor<'_> for NumberVisitor {
    type Value = Number;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a finite number")
    }

    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Number, E> {
        Ok(Number::from(value))
    }

    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Number, E> {
        Ok(Number::from_integer(i128::from(value)))
    }

    fn visit_i128<E: serde::de::Error>(self, value: i128) -> Result<Number, E> {
        Ok(Number::from_integer(value))
    }

    fn visit_u128<E: serde::de::Error>(self, value: u128) -> Result<Number, E> {
        i128::try_from(value)
            .map(Number::from_integer)
            .map_err(|_| E::custom("number does not fit the domain number"))
    }

    fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Number, E> {
        Number::new(value).map_err(E::custom)
    }
}

impl serde::Serialize for Number {
    /// The binary64 for every value binary64 carries, and the exact integer for the ones it does
    /// not.
    ///
    /// The first class is every number in every artifact this repository has published: an
    /// artifact's numbers came from a document, and at the base commit a document's numbers were
    /// read through binary64, so their carried value *is* the canonical decimal of their binary.
    /// Those write exactly what they wrote before — an integral witness is still `1.0`
    /// (`ess-conformance/src/witness.rs`), and a report still quotes what this writes.
    ///
    /// The second class is a value binary64 never carried: `i64::MAX`, `2^53 + 1`. At the base
    /// those were rounded on the way in and written wrong on the way out, so writing the integer
    /// is not a change to any spelling a reader has ever received — it is the first correct one,
    /// and it is what makes the round-trip law hold for the constructors the design page names.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            // The invariant on `Repr::Exact` guarantees `scale == 0` here: a non-integer whose
            // binary does not carry it is never admitted to `Exact`.
            Repr::Exact {
                units,
                scale,
                binary,
            } if Some((units, scale)) != canonical_decimal(binary) => match i64::try_from(units) {
                Ok(units) => serializer.serialize_i64(units),
                Err(_) => serializer.serialize_i128(units),
            },
            _ => serializer.serialize_f64(self.get()),
        }
    }
}

impl Number {
    /// Builds a number, refusing anything that is not finite.
    pub fn new(value: f64) -> Result<Self, ParseError> {
        if !value.is_finite() {
            let found = if value.is_nan() { "NaN" } else { "an infinity" };
            return Err(ParseError::shape("number", "a finite number", found));
        }
        Ok(Self(Repr::of_binary64(value)))
    }

    /// The underlying binary64 — what this value serialises and has always serialised as.
    ///
    /// Lossy above 2^53, and deliberately unchanged: every existing caller reads a count, a list
    /// index or a sign from it. [`as_i64`](Self::as_i64) is the exact answer.
    pub const fn get(self) -> f64 {
        match self.0 {
            Repr::Exact { binary, .. } | Repr::Binary64(binary) => binary,
        }
    }

    /// `true` when this value is an exact integer an [`i64`] holds.
    ///
    /// One range, and it is the one [`as_i64`](Self::as_i64) and the generated `int64`/`i64` codecs
    /// answer on: `[i64::MIN, i64::MAX]`. `is_integral` and `as_i64` are the same question, so a
    /// value admitted as a `Primitive::Integer` always carries the integer it was admitted as.
    pub fn is_integral(self) -> bool {
        self.as_i64().is_some()
    }

    /// The exact integer, when this value is one that fits an [`i64`].
    ///
    /// `None` for `2^63`, which is not an `i64` however it was spelt, and `None` for anything with
    /// a fractional part.
    pub fn as_i64(self) -> Option<i64> {
        // A `Binary64` is a magnitude no `(i128, u8)` spells — at or beyond `10^38`, or below
        // `10^-38` — so it is never an `i64` either.
        match self.0 {
            Repr::Exact {
                units, scale: 0, ..
            } => i64::try_from(units).ok(),
            _ => None,
        }
    }

    /// The exact decimal spelling: no exponent, no trailing zeroes, one spelling per value.
    ///
    /// This is the grammar `ess-gen`'s `DECIMAL_PATTERN` publishes for `Primitive::Decimal`.
    pub fn exact_text(self) -> String {
        match self.0 {
            Repr::Exact { units, scale, .. } => exact_text(units, scale),
            Repr::Binary64(value) => format!("{value}"),
        }
    }

    /// The exact value as `(units, scale)`, when there is one.
    const fn exact(self) -> Option<(i128, u8)> {
        match self.0 {
            Repr::Exact { units, scale, .. } => Some((units, scale)),
            Repr::Binary64(_) => None,
        }
    }

    /// An exact integer of any width this type can hold.
    pub(crate) fn from_integer(units: i128) -> Self {
        Self(Repr::of_integer(units))
    }

    /// Reads the exact decimal an author wrote, keeping it only where the write can give it back.
    ///
    /// **`Repr::Exact` may only carry a value that survives its own write** — the invariant the
    /// round-trip law rests on. A value qualifies two ways, and only two:
    ///
    /// * it equals the canonical decimal of its binary64, so the write is that binary64; or
    /// * it is an **integer**, so the write is the integer token and the read door reads it back.
    ///
    /// A non-integer with more places than binary64 carries is neither.
    /// `parse_literal("1.0000000000000000001")` used to keep all twenty digits, write `1.0`, and
    /// come back integral — admitted differently on the two sides of one write. It now collapses to
    /// the canonical decimal of `1.0`, which is what the write was always going to say. Reading the
    /// authored digits back is the canonical-serialization stage, and it moves both doors at once.
    ///
    /// `None` when `text` is not a decimal literal at all.
    pub(crate) fn parse_decimal(text: &str) -> Option<Self> {
        let binary = text.parse::<f64>().ok().filter(|value| value.is_finite())?;
        let authored = exact_of_decimal_text(text);
        let survives_the_write = |(units, scale): &(i128, u8)| {
            *scale == 0 || Some((*units, *scale)) == canonical_decimal(binary)
        };
        Some(Self(match authored.filter(survives_the_write) {
            Some((units, scale)) => Repr::exact(units, scale, binary),
            None => Repr::of_binary64(binary),
        }))
    }
}

impl Repr {
    /// **The only place an [`Exact`](Repr::Exact) is built**, and the two postconditions it holds.
    ///
    /// 1. The value survives its own write. `scale == 0` means the write is the integer token,
    ///    which the read door reads back; otherwise the value must *be* the canonical decimal of
    ///    its binary, so the write is that binary64. The caller establishes this; a value that
    ///    holds neither must not reach here.
    /// 2. **The variant is a function of the carried binary64**: an `Exact` exists only where
    ///    [`canonical_decimal`] has an answer, so a [`Binary64`](Repr::Binary64) — which exists
    ///    exactly where it has none — can never carry the same `f64` as an `Exact`.
    ///
    /// The second is checked here rather than argued about elsewhere, which is what makes
    /// [`Number::cmp`]'s totality a one-function claim: its `total_cmp` arm runs only when the
    /// variants differ, and two differing variants never carry one `f64`.
    fn exact(units: i128, scale: u8, binary: f64) -> Self {
        if canonical_decimal(binary).is_none() {
            return Self::Binary64(binary);
        }
        Self::Exact {
            units,
            scale,
            binary,
        }
    }

    /// The canonical decimal of a binary64, which is by construction a value the write gives back.
    fn of_binary64(value: f64) -> Self {
        match canonical_decimal(value) {
            Some((units, scale)) => Self::exact(units, scale, value),
            None => Self::Binary64(value),
        }
    }

    /// An exact integer of any width this type can hold.
    ///
    /// Always admissible under postcondition 1: an integer's write is the integer token when
    /// binary64 does not carry it, and the binary64 when it does.
    #[allow(clippy::cast_precision_loss)]
    fn of_integer(units: i128) -> Self {
        Self::exact(units, 0, units as f64)
    }
}

/// The decimal a binary64 is carried as, or `None` where `(i128, u8)` cannot spell one.
///
/// Two rules, because one is not enough to be truthful:
///
/// * **Below `2^63` an integral binary64 is carried as the integer it is.** The shortest
///   round-tripping decimal for `2^63` is `9223372036854776000`, a different number; carrying that
///   made [`Number::as_i64`] answer about a value nobody wrote.
/// * **Otherwise, the shortest decimal that round-trips** — what `f64`'s own `Display` has always
///   printed, and what a reader means by the value.
///
/// This is a pure function of the `f64`, and it is what makes [`Repr`]'s variant a function of the
/// carried binary64: `Repr::Binary64(v)` exactly when this returns `None` for `v`. Two `Number`s
/// carrying one binary64 are therefore the same variant, which is the whole of the argument that
/// [`Number::cmp`] is a total order.
fn canonical_decimal(value: f64) -> Option<(i128, u8)> {
    if value.fract() == 0.0 && (-INTEGER_CARRIER..=INTEGER_CARRIER).contains(&value) {
        // `|value| <= 2^63` and the value is integral, so the conversion is exact.
        #[allow(clippy::cast_possible_truncation)]
        return Some((value as i128, 0));
    }
    exact_of_decimal_text(&format!("{value}"))
}

/// Reads a decimal literal — `[+-]?digits[.digits][eE[+-]?digits]` — as `units × 10^-scale`.
///
/// `None` where the text is not that grammar, where the digits do not fit an `i128`, or where the
/// scale would exceed [`MAX_SCALE`]. Every rejection falls back to the binary64, which is what the
/// value was before this function existed.
fn exact_of_decimal_text(text: &str) -> Option<(i128, u8)> {
    let (negative, rest) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let (mantissa, exponent) = match rest.find(['e', 'E']) {
        Some(at) => (&rest[..at], rest[at + 1..].parse::<i32>().ok()?),
        None => (rest, 0),
    };
    let (integer, fraction) = match mantissa.split_once('.') {
        Some((integer, fraction)) => (integer, fraction),
        None => (mantissa, ""),
    };
    if integer.is_empty() && fraction.is_empty() {
        return None;
    }
    if !integer
        .bytes()
        .chain(fraction.bytes())
        .all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let digits: String = integer.chars().chain(fraction.chars()).collect();
    let mut units: i128 = digits.parse().ok()?;
    if negative {
        units = -units;
    }
    // `scale` counts places right of the point; a positive exponent removes them and, once there
    // are none left, multiplies.
    let mut scale = i64::try_from(fraction.len()).ok()? - i64::from(exponent);
    while scale < 0 {
        units = units.checked_mul(10)?;
        scale += 1;
    }
    let scale = u32::try_from(scale).ok()?;
    if scale > MAX_SCALE {
        return None;
    }
    Some(normalise(units, scale))
}

/// Strips the trailing zeroes a scale does not need, so one value has one `(units, scale)`.
fn normalise(mut units: i128, mut scale: u32) -> (i128, u8) {
    while scale > 0 && units % 10 == 0 {
        units /= 10;
        scale -= 1;
    }
    // `scale <= MAX_SCALE` holds for every caller and shrinks here; the clamp is written so this
    // function has no panicking path at all.
    (units, u8::try_from(scale).unwrap_or(u8::MAX))
}

/// `units × 10^-scale`, spelt without an exponent and without trailing zeroes.
fn exact_text(units: i128, scale: u8) -> String {
    let digits = units.unsigned_abs().to_string();
    let sign = if units < 0 { "-" } else { "" };
    let scale = usize::from(scale);
    if scale == 0 {
        return format!("{sign}{digits}");
    }
    let padded = if digits.len() <= scale {
        format!("{}{digits}", "0".repeat(scale + 1 - digits.len()))
    } else {
        digits
    };
    let (whole, fraction) = padded.split_at(padded.len() - scale);
    format!("{sign}{whole}.{fraction}")
}

/// Compares two exact decimals without widening either one.
///
/// Aligning the scales would need an `i256` for a scale-255 value against a scale-0 one, so the
/// comparison is on the digits: sign, then magnitude, then place by place.
fn exact_cmp(left: (i128, u8), right: (i128, u8)) -> Ordering {
    let sign = |units: i128| units.signum();
    match sign(left.0).cmp(&sign(right.0)) {
        Ordering::Equal => {}
        other => return other,
    }
    let negative = left.0 < 0;
    let split = |(units, scale): (i128, u8)| {
        let digits = units.unsigned_abs().to_string();
        let scale = usize::from(scale);
        let padded = if digits.len() <= scale {
            format!("{}{digits}", "0".repeat(scale + 1 - digits.len()))
        } else {
            digits
        };
        let at = padded.len() - scale;
        (padded[..at].to_owned(), padded[at..].to_owned())
    };
    let (left_whole, left_fraction) = split(left);
    let (right_whole, right_fraction) = split(right);
    let whole = left_whole
        .len()
        .cmp(&right_whole.len())
        .then_with(|| left_whole.cmp(&right_whole));
    let places = left_fraction.len().max(right_fraction.len());
    let pad = |value: String| format!("{value}{}", "0".repeat(places - value.len()));
    let magnitude = whole.then_with(|| pad(left_fraction).cmp(&pad(right_fraction)));
    if negative {
        magnitude.reverse()
    } else {
        magnitude
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Self(Repr::of_integer(i128::from(value)))
    }
}

impl From<usize> for Number {
    fn from(value: usize) -> Self {
        Self(Repr::of_integer(value as i128))
    }
}

impl From<u32> for Number {
    fn from(value: u32) -> Self {
        Self(Repr::of_integer(i128::from(value)))
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Number {}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Number {
    /// The exact value, and the binary64 only where there is no exact value.
    ///
    /// The exact comparison is what fixes F08: `2^53` and `2^53 + 1` are one `f64`, and comparing
    /// the scaled integers says which is which.
    ///
    /// **The order is total, and the argument is one function away.** The `total_cmp` arm runs only
    /// when the two variants differ, and [`Repr::exact`]'s second postcondition is that the variant
    /// is a function of the carried binary64 — so differing variants never carry one `f64`, the
    /// arm is never a tie, and the two arms never disagree about one pair. The earlier version of
    /// this comment argued it from magnitude bands instead, and that argument was false:
    /// `parse_decimal("1e-41")` and `of_binary64(1e20)` are both `Exact` inside the band it named.
    ///
    /// **`-0.0` and `0.0` are one value.** `units × 10^-scale` has one zero, which is what the
    /// `Decimal` row of the design page's matrix says; it is what `PartialEq` answered before this
    /// type was rewritten (`eq` was `f64 ==`); and it is what a guard `amount == 0` has to mean.
    /// The signed zero `Primitive::Binary64` promises survives in the bytes, because
    /// [`Serialize`](serde::Serialize) writes the carried binary64.
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.exact(), other.exact()) {
            (Some(left), Some(right)) => exact_cmp(left, right),
            _ => self.get().total_cmp(&other.get()),
        }
    }
}

impl fmt::Display for Number {
    /// The exact spelling.
    ///
    /// Identical to the binary64 rendering for every value that has one — which is every value
    /// below 2^53, and so every value in every fixture this repository holds. Above it, this stops
    /// printing a number the value is not.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.exact_text())
    }
}

/// Whether `text` is a UUID in the canonical hyphenated form, and nothing else.
///
/// The one form `ess-gen`'s `UUID_PATTERN` publishes: eight, four, four, four and twelve
/// hexadecimal digits in either case. The `urn:uuid:` and brace-wrapped spellings are refused, as
/// is anything with surrounding whitespace — one value has one spelling, or two systems agree on
/// the schema and disagree on equality.
///
/// It lives here because `ess-domain`, `ess-conformance`, `ess-gen` and `infra-*` all depend on
/// this crate and none depends on another, so this is the only place the four can share one answer.
pub fn is_canonical_uuid(text: &str) -> bool {
    let groups = [8usize, 4, 4, 4, 12];
    let mut parts = text.split('-');
    for width in groups {
        let Some(part) = parts.next() else {
            return false;
        };
        if part.len() != width || !part.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return false;
        }
    }
    parts.next().is_none()
}

/// Whether `text` is base64 with padding, the grammar `ess-gen`'s `BASE64_PATTERN` publishes for
/// `Primitive::Bytes`.
///
/// The standard alphabet, in complete four-character groups, with `=` only in the final group and
/// only as one or two characters. The URL-safe alphabet and unpadded base64 are refused for the
/// reason the UUID form is: the projection names one encoding.
pub fn is_padded_base64(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() % 4 != 0 {
        return false;
    }
    let padding = bytes.iter().rev().take_while(|byte| **byte == b'=').count();
    if padding > 2 {
        return false;
    }
    bytes[..bytes.len() - padding]
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'+' || *byte == b'/')
}

impl schemars::JsonSchema for Number {
    fn schema_name() -> String {
        "Number".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::Number.into()),
            ..Default::default()
        };
        schema.metadata().description = Some("A number; NaN is not permitted.".to_owned());
        schema.into()
    }
}

/// The value a fact is bound to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum FactValue {
    /// A boolean, such as `recovery_verified = true`.
    Bool(bool),
    /// A number, such as `tests.unit.failed = 0`.
    Number(Number),
    /// A string, such as `test.result = failed`.
    Text(String),
}

impl FactValue {
    /// A boolean fact value.
    pub const fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    /// A numeric fact value from a count.
    pub fn count(value: usize) -> Self {
        Self::Number(Number::from(value))
    }

    /// A numeric fact value.
    pub fn number(value: f64) -> Result<Self, ParseError> {
        Ok(Self::Number(Number::new(value)?))
    }

    /// A textual fact value.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// The name of this value's type, for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "boolean",
            Self::Number(_) => "number",
            Self::Text(_) => "text",
        }
    }

    /// The string contents, when this is text.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// The numeric contents, when this is a number.
    pub fn as_number(&self) -> Option<Number> {
        match self {
            Self::Number(number) => Some(*number),
            _ => None,
        }
    }

    /// The boolean contents, when this is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Interprets a bare path reference: booleans by their value, everything else as "the
    /// fact is present, therefore true".
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::Number(number) => number.get() != 0.0,
            Self::Text(text) => !text.is_empty() && text != "false",
        }
    }

    /// Parses a literal as written in a predicate expression.
    ///
    /// `true`/`false` become booleans, anything parsable as a number becomes a number, a
    /// quoted string becomes text verbatim, and any other bare word becomes text.
    pub fn parse_literal(raw: &str) -> Self {
        let trimmed = raw.trim();
        if let Some(quoted) = strip_quotes(trimmed) {
            return Self::Text(quoted.to_owned());
        }
        match trimmed {
            "true" => return Self::Bool(true),
            "false" => return Self::Bool(false),
            _ => {}
        }
        // `1e400` parses as `f64::INFINITY` rather than failing, so the check is finiteness and
        // not merely NaN: an unrepresentable literal stays the text the author typed, which is
        // both truthful and comparable, instead of becoming an infinity that serialises as `null`.
        if let Some(number) = Number::parse_decimal(trimmed) {
            return Self::Number(number);
        }
        Self::Text(trimmed.to_owned())
    }
}

/// Removes matching single or double quotes, returning the contents.
fn strip_quotes(value: &str) -> Option<&str> {
    for quote in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
            return Some(&value[1..value.len() - 1]);
        }
    }
    None
}

impl fmt::Display for FactValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::Number(value) => write!(f, "{value}"),
            Self::Text(value) => f.write_str(value),
        }
    }
}

impl schemars::JsonSchema for FactValue {
    fn schema_name() -> String {
        "FactValue".to_owned()
    }

    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject::default();
        schema.subschemas().any_of = Some(vec![
            <bool>::json_schema(generator),
            <Number>::json_schema(generator),
            <String>::json_schema(generator),
        ]);
        schema.metadata().description = Some("A fact value: boolean, number or string.".to_owned());
        schema.into()
    }
}

impl From<bool> for FactValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<usize> for FactValue {
    fn from(value: usize) -> Self {
        Self::Number(Number::from(value))
    }
}

impl From<Number> for FactValue {
    fn from(value: Number) -> Self {
        Self::Number(value)
    }
}

impl From<&str> for FactValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

/// A dotted path identifying an observable value, such as `tests.unit.failed`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(into = "String")]
pub struct FactPath {
    segments: Vec<String>,
}

impl FactPath {
    /// Parses a dotted fact path.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ParseError> {
        let value = value.as_ref();
        let reject = |reason: String| Err(ParseError::identifier("fact path", value, reason));

        if value.is_empty() {
            return reject("must not be empty".to_owned());
        }
        let segments: Vec<String> = value.split('.').map(ToOwned::to_owned).collect();
        for segment in &segments {
            if segment.is_empty() {
                return reject(
                    "has an empty segment; dots must not lead, trail or repeat".to_owned(),
                );
            }
            for ch in segment.chars() {
                if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
                    return reject(format!("contains disallowed character {ch:?}"));
                }
            }
        }
        if !value.starts_with(|c: char| c.is_ascii_alphabetic()) {
            return reject("must start with a letter".to_owned());
        }
        Ok(Self { segments })
    }

    /// Builds a path from already-valid segments, joining with dots.
    ///
    /// # Panics
    ///
    /// Panics when the resulting path is not a valid fact path. Callers inside this workspace
    /// build paths from validated identifiers, so a panic here is a bug in fact projection,
    /// not bad input.
    pub fn from_segments<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let joined = segments
            .into_iter()
            .map(|segment| segment.as_ref().to_owned())
            .collect::<Vec<_>>()
            .join(".");
        Self::new(&joined)
            .unwrap_or_else(|error| panic!("fact projection built an invalid path: {error}"))
    }

    /// The path segments.
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// The first segment, which is the namespace the protocol declares as observable.
    pub fn namespace(&self) -> &str {
        &self.segments[0]
    }

    /// This path with `segment` appended.
    #[must_use]
    pub fn child(&self, segment: &str) -> Self {
        let mut segments = self.segments.clone();
        segments.push(segment.to_owned());
        Self { segments }
    }

    /// The pattern published in generated JSON Schema.
    pub const PATTERN: &'static str = "^[A-Za-z][A-Za-z0-9_-]*(\\.[A-Za-z0-9_-]+)*$";
}

impl fmt::Display for FactPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.segments.join("."))
    }
}

impl fmt::Debug for FactPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FactPath({self})")
    }
}

impl FromStr for FactPath {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl From<FactPath> for String {
    fn from(value: FactPath) -> Self {
        value.to_string()
    }
}

impl<'de> serde::Deserialize<'de> for FactPath {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Self::new(raw).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for FactPath {
    fn schema_name() -> String {
        "FactPath".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(Self::PATTERN.to_owned());
        schema.metadata().description =
            Some("Dotted path to an observable value, such as `tests.unit.failed`.".to_owned());
        schema.into()
    }
}

/// A pattern matching a family of fact paths, such as `tests.**` or `artifact.*.status`.
///
/// `*` matches exactly one segment; a trailing `**` matches one or more remaining segments.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(into = "String")]
pub struct FactPattern {
    segments: Vec<String>,
}

impl FactPattern {
    /// Parses a fact pattern.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ParseError> {
        let value = value.as_ref();
        let reject = |reason: String| Err(ParseError::identifier("fact pattern", value, reason));

        if value.is_empty() {
            return reject("must not be empty".to_owned());
        }
        let segments: Vec<String> = value.split('.').map(ToOwned::to_owned).collect();
        let last = segments.len() - 1;
        for (index, segment) in segments.iter().enumerate() {
            if segment == "**" {
                if index != last {
                    return reject("`**` may only appear as the final segment".to_owned());
                }
                continue;
            }
            if segment == "*" {
                continue;
            }
            if segment.is_empty() {
                return reject("has an empty segment".to_owned());
            }
            for ch in segment.chars() {
                if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
                    return reject(format!("contains disallowed character {ch:?}"));
                }
            }
        }
        Ok(Self { segments })
    }

    /// `true` when `path` matches this pattern.
    pub fn matches(&self, path: &FactPath) -> bool {
        let actual = path.segments();
        for (index, pattern) in self.segments.iter().enumerate() {
            if pattern == "**" {
                return actual.len() > index;
            }
            let Some(segment) = actual.get(index) else {
                return false;
            };
            if pattern != "*" && pattern != segment {
                return false;
            }
        }
        actual.len() == self.segments.len()
    }

    /// The pattern published in generated JSON Schema.
    pub const PATTERN: &'static str = "^([A-Za-z0-9_*-]+)(\\.[A-Za-z0-9_*-]+)*$";
}

impl fmt::Display for FactPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.segments.join("."))
    }
}

impl fmt::Debug for FactPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FactPattern({self})")
    }
}

impl FromStr for FactPattern {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl From<FactPattern> for String {
    fn from(value: FactPattern) -> Self {
        value.to_string()
    }
}

impl<'de> serde::Deserialize<'de> for FactPattern {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Self::new(raw).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for FactPattern {
    fn schema_name() -> String {
        "FactPattern".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(Self::PATTERN.to_owned());
        schema.metadata().description = Some(
            "Pattern over fact paths; `*` matches one segment, a trailing `**` matches the rest."
                .to_owned(),
        );
        schema.into()
    }
}

/// Named ordered scales, so that non-numeric facts can still be compared with `<` and `>=`.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(transparent)]
pub struct Scales {
    /// Scale name to its values, lowest rank first.
    scales: BTreeMap<String, Vec<String>>,
}

impl Scales {
    /// A shared empty scale set, used as the default for fact sources that declare none.
    pub fn empty() -> &'static Self {
        static EMPTY: OnceLock<Scales> = OnceLock::new();
        EMPTY.get_or_init(Self::default)
    }

    /// `true` when no scales are declared.
    pub fn is_empty(&self) -> bool {
        self.scales.is_empty()
    }

    /// Absorbs another scale set, keeping this set's definitions on conflict.
    pub fn extend(&mut self, other: &Self) {
        for (name, values) in &other.scales {
            self.scales
                .entry(name.clone())
                .or_insert_with(|| values.clone());
        }
    }

    /// Declares a scale, lowest value first.
    pub fn insert(&mut self, name: impl Into<String>, values: Vec<String>) {
        self.scales.insert(name.into(), values);
    }

    /// The declared scales.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.scales.iter()
    }

    /// Compares two values using the unique scale that contains both.
    ///
    /// Returns `None` when no scale contains both values, or when more than one does and they
    /// disagree — an ambiguous comparison is reported as unknown rather than guessed.
    pub fn compare(&self, left: &str, right: &str) -> Option<Ordering> {
        let mut result: Option<Ordering> = None;
        for values in self.scales.values() {
            let left_rank = values.iter().position(|value| value == left);
            let right_rank = values.iter().position(|value| value == right);
            if let (Some(left_rank), Some(right_rank)) = (left_rank, right_rank) {
                let ordering = left_rank.cmp(&right_rank);
                match result {
                    None => result = Some(ordering),
                    Some(previous) if previous == ordering => {}
                    Some(_) => return None,
                }
            }
        }
        result
    }
}

/// A source of facts a predicate can be evaluated against.
pub trait FactSource {
    /// The value bound to `path`, or `None` when nothing has observed it.
    fn fact(&self, path: &FactPath) -> Option<FactValue>;

    /// The ordered scales available for non-numeric comparison.
    fn scales(&self) -> &Scales {
        Scales::empty()
    }

    /// How many elements the collection at `path` has, or `None` when nothing has observed it.
    ///
    /// A quantifier needs to know how far to count, and a fact is a scalar bound to a dotted
    /// path — there is no collection value to ask for a length. So a collection publishes its own
    /// size as `<path>.count` and its elements as `<path>.0.…`, `<path>.1.…`, which is the shape
    /// every projection in this workspace already flattens to. The default implementation reads
    /// that convention and nothing else, so no existing source has to change to gain quantifiers,
    /// and a source that knows its own cardinality directly may override it.
    ///
    /// A negative or fractional count is not a smaller collection, it is a projection defect, and
    /// answering `None` reports it as *unobserved* rather than silently walking zero elements —
    /// which is the difference between a quantifier that says `unknown` and one that says `true`.
    fn cardinality(&self, path: &FactPath) -> Option<usize> {
        match self.fact(&path.child("count"))? {
            FactValue::Number(number) => whole_count(number.get()),
            _ => None,
        }
    }
}

/// A count as a number of elements, or `None` when it is not one.
///
/// The upper bound is 2^53, the largest integer an `f64` holds exactly. Past it the value is not a
/// larger collection somebody observed, it is the rounding a `f64` did on the way in, and treating
/// it as a length would walk a number of elements nobody wrote down.
fn whole_count(value: f64) -> Option<usize> {
    /// 2^53.
    const EXACT_INTEGER_LIMIT: f64 = 9_007_199_254_740_992.0;

    if value < 0.0 || value.fract() != 0.0 || value > EXACT_INTEGER_LIMIT {
        return None;
    }
    // Every branch that reaches here has established a non-negative whole number at most 2^53, and
    // `usize` is 64-bit on every target this workspace builds for, so the conversion is exact.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "bounded and whole by the guard above"
    )]
    Some(value as usize)
}

/// An in-memory set of facts.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct FactStore {
    facts: BTreeMap<FactPath, FactValue>,
    #[serde(skip_serializing_if = "is_empty_scales")]
    scales: Scales,
}

/// Whether a scale set is empty, for output suppression.
fn is_empty_scales(scales: &Scales) -> bool {
    scales.is_empty()
}

impl FactStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds `path` to `value`, replacing any previous binding.
    pub fn set(&mut self, path: FactPath, value: impl Into<FactValue>) {
        self.facts.insert(path, value.into());
    }

    /// Binds a path given as a string.
    ///
    /// # Panics
    ///
    /// Panics when `path` is not a valid fact path; intended for statically-known paths in
    /// fact projection and tests.
    pub fn set_path(&mut self, path: &str, value: impl Into<FactValue>) {
        let path = FactPath::new(path).unwrap_or_else(|error| panic!("invalid fact path: {error}"));
        self.set(path, value);
    }

    /// Binds `path` to `value` only when it is not already bound.
    pub fn set_if_absent(&mut self, path: FactPath, value: impl Into<FactValue>) {
        self.facts.entry(path).or_insert_with(|| value.into());
    }

    /// Absorbs every fact from `other`, overwriting on conflict.
    pub fn extend(&mut self, other: Self) {
        self.facts.extend(other.facts);
    }

    /// Absorbs facts from an iterator.
    pub fn extend_facts<I: IntoIterator<Item = (FactPath, FactValue)>>(&mut self, facts: I) {
        self.facts.extend(facts);
    }

    /// Declares the ordered scales used for non-numeric comparisons.
    pub fn set_scales(&mut self, scales: Scales) {
        self.scales = scales;
    }

    /// The number of bound facts.
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// `true` when no facts are bound.
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Every bound fact, in path order.
    pub fn iter(&self) -> impl Iterator<Item = (&FactPath, &FactValue)> {
        self.facts.iter()
    }

    /// Every bound path.
    pub fn paths(&self) -> impl Iterator<Item = &FactPath> {
        self.facts.keys()
    }
}

impl FactSource for FactStore {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.facts.get(path).cloned()
    }

    fn scales(&self) -> &Scales {
        &self.scales
    }
}

impl FromIterator<(FactPath, FactValue)> for FactStore {
    fn from_iter<I: IntoIterator<Item = (FactPath, FactValue)>>(iter: I) -> Self {
        Self {
            facts: iter.into_iter().collect(),
            scales: Scales::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_rejects_fact_paths() {
        assert_eq!(
            FactPath::new("tests.unit.failed")
                .expect("parses")
                .segments()
                .len(),
            3
        );
        assert!(FactPath::new("artifact.architecture-design.status").is_ok());
        assert!(FactPath::new("tests..failed").is_err());
        assert!(FactPath::new(".tests").is_err());
        assert!(FactPath::new("1tests").is_err());
        assert!(FactPath::new("tests unit").is_err());
    }

    #[test]
    fn patterns_match_segment_wise() {
        let exact = FactPattern::new("tests.unit.failed").expect("parses");
        let one = FactPattern::new("artifact.*.status").expect("parses");
        let rest = FactPattern::new("tests.**").expect("parses");

        let path = |value: &str| FactPath::new(value).expect("parses");

        assert!(exact.matches(&path("tests.unit.failed")));
        assert!(!exact.matches(&path("tests.unit")));

        assert!(one.matches(&path("artifact.design.status")));
        assert!(!one.matches(&path("artifact.design.review.status")));

        assert!(rest.matches(&path("tests.unit")));
        assert!(rest.matches(&path("tests.unit.failed")));
        assert!(
            !rest.matches(&path("tests")),
            "`**` requires at least one segment"
        );
    }

    #[test]
    fn a_number_too_large_to_represent_stays_the_text_it_was_written_as() {
        // `1e400` overflows to `f64::INFINITY`, which JSON cannot spell: `serde_json` writes it as
        // `null`. A guard published as `any_of: [null]` says something its author never wrote, and
        // nothing downstream can tell that from a deliberate null.
        let value = FactValue::parse_literal("1e400");
        assert_eq!(
            value,
            FactValue::text("1e400"),
            "an unrepresentable literal must not become an infinity"
        );
        let json = serde_json::to_string(&value).expect("a fact value serialises");
        assert!(
            !json.contains("null"),
            "a literal must never round-trip into a null: {json}"
        );
    }

    #[test]
    fn an_infinity_is_refused_because_it_cannot_be_written_back_out() {
        let error = Number::new(f64::INFINITY).expect_err("an infinity is not a number here");
        assert!(error.to_string().contains("infinity"), "{error}");
        Number::new(f64::NEG_INFINITY).expect_err("nor is a negative one");
    }

    #[test]
    fn a_document_cannot_deserialise_a_number_the_constructor_would_refuse() {
        // `#[serde(transparent)]` with a derived `Deserialize` reads straight into the field and
        // never calls `new`, so `.nan` in a document used to produce a `Number` this type says
        // cannot exist — and a NaN makes `Ord` a lie for every value it is compared against.
        for spelling in [".nan", ".inf", "-.inf"] {
            serde_yaml::from_str::<Number>(spelling)
                .expect_err(&format!("`{spelling}` is not a finite number"));
        }
        let ordinary: Number = serde_yaml::from_str("1.5").expect("a finite number still reads");
        assert!((ordinary.get() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn parses_literals_by_shape() {
        assert_eq!(FactValue::parse_literal("true"), FactValue::Bool(true));
        assert_eq!(FactValue::parse_literal("0"), FactValue::count(0));
        assert_eq!(
            FactValue::parse_literal("0.01"),
            FactValue::number(0.01).expect("finite")
        );
        assert_eq!(
            FactValue::parse_literal("failed"),
            FactValue::text("failed")
        );
        assert_eq!(
            FactValue::parse_literal("\"1.2.3\""),
            FactValue::text("1.2.3")
        );
    }

    #[test]
    fn scales_order_non_numeric_values() {
        let mut scales = Scales::default();
        scales.insert(
            "risk",
            ["low", "medium", "high", "critical"]
                .map(ToOwned::to_owned)
                .to_vec(),
        );

        assert_eq!(scales.compare("high", "medium"), Some(Ordering::Greater));
        assert_eq!(scales.compare("low", "low"), Some(Ordering::Equal));
        assert_eq!(scales.compare("high", "unknown-value"), None);
    }

    #[test]
    fn rejects_nan_numbers() {
        assert!(Number::new(f64::NAN).is_err());
        assert!(Number::new(1.5).is_ok());
    }
}
