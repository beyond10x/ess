//! How the Go target carries a finite `Binary64`.

/// `true` once this target's codec carries a finite `Binary64` in every carried position
/// (`crate::failure::binary64` reads it). Flipped by the unit that adds the codec, never before
/// the generated code compiles.
pub(crate) const CARRIED: bool = true;

/// Whether the model has a `Binary64` anywhere: the wrapper, its imports and `binary64At` are
/// emitted only then, so every other model keeps its bytes.
pub(crate) fn used(ir: &ess_compiler::EssIr) -> bool {
    !ess_compiler::binary64::locations(ir).is_empty()
}

/// The imports the wrapper needs, written after the `primitives` package clause.
pub(crate) const IMPORTS: &str =
    "\nimport (\n\t\"errors\"\n\t\"math\"\n\t\"strconv\"\n\t\"strings\"\n)\n";

/// The wrapper, appended to the `primitives` package.
pub(crate) const PRIMITIVE: &str = r#"
// Binary64 is a finite IEEE 754 binary64, preserving the sign of zero.
//
// The field is unexported, so the only way to make one is [NewBinary64], which refuses a NaN
// and an infinity — but Go's zero value needs no constructor, so `Binary64{}` is still
// spellable, and it is +0. Two values are equal under `==` as IEEE 754 compares them, so +0
// and -0 are equal; the sign stays readable through [Binary64.Float64] and [Binary64.Token].
type Binary64 struct {
	value float64
}

// NewBinary64 wraps a finite value as a Binary64, and refuses a NaN or an infinity.
func NewBinary64(value float64) (Binary64, error) {
	if math.IsNaN(value) || math.IsInf(value, 0) {
		return Binary64{}, errors.New("Binary64 requires a finite value")
	}
	return Binary64{value: value}, nil
}

// Float64 is the wrapped value.
func (v Binary64) Float64() float64 {
	return v.value
}

// Decimal is the shortest plain decimal that reads back as the value, as guards, filters,
// invariants and orders compare it: `-0` for negative zero, never an exponent.
func (v Binary64) Decimal() string {
	return strconv.FormatFloat(v.value, 'f', -1, 64)
}

// Token is the JSON number the wire writes: the shortest digits that read back as the value,
// laid out as `3.0`, `-0.0`, `0.73`, `1e16` and `1e-6` are, the bytes the Rust server writes.
func (v Binary64) Token() string {
	if v.value == 0 {
		if math.Signbit(v.value) {
			return "-0.0"
		}
		return "0.0"
	}
	sign := ""
	if v.value < 0 {
		sign = "-"
	}
	mantissa, exponent, _ := strings.Cut(strconv.FormatFloat(math.Abs(v.value), 'e', -1, 64), "e")
	digits := strings.Replace(mantissa, ".", "", 1)
	power, _ := strconv.Atoi(exponent)
	point := power + 1
	count := len(digits)
	switch {
	case count <= point && point <= 16:
		return sign + digits + strings.Repeat("0", point-count) + ".0"
	case 0 < point && point <= 16:
		return sign + digits[:point] + "." + digits[point:]
	case -5 < point && point <= 0:
		return sign + "0." + strings.Repeat("0", -point) + digits
	case count == 1:
		return sign + digits + "e" + strconv.Itoa(power)
	default:
		return sign + digits[:1] + "." + digits[1:] + "e" + strconv.Itoa(power)
	}
}
"#;

/// The wire reader of a `Binary64`, with the wrapper and constructor spelled as the importing
/// file names them.
pub(crate) fn wire_helper(wrapper: &str, constructor: &str) -> String {
    WIRE_HELPER
        .replace("{wrapper}", wrapper)
        .replace("{constructor}", constructor)
}

const WIRE_HELPER: &str = r#"
// binary64At is the finite Binary64 at this path: a JSON number, rounded once to the nearest
// binary64, and refused where it rounds to an infinity.
func binary64At(value any, at string, expected string) ({wrapper}, error) {
	number, ok := value.(json.Number)
	if !ok {
		return {wrapper}{}, DecodeError{At: at, Expected: expected, Found: describes(value)}
	}
	parsed, _ := strconv.ParseFloat(number.String(), 64)
	held, err := {constructor}(parsed)
	if err != nil {
		return {wrapper}{}, DecodeError{At: at, Expected: expected, Found: "the number " + number.String()}
	}
	return held, nil
}
"#;
