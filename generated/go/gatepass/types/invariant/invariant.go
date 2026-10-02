// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

// Package invariant is the three-valued reading every generated BrokenInvariant shares.
//
// True holds, False is broken and Unknown is undecided: a value the invariant reads is absent, so
// it decides nothing. Numbers compare by their exact decimal value, a `Timestamp` by the RFC 3339
// instant it names, other text by its UTF-8 bytes, and `Bytes` as padded base64 — the reading the
// conformance interpreter gives the same values.
package invariant

import (
	"encoding/base64"
	"strconv"
	"strings"
)

// Truth is one invariant's reading of one value.
type Truth int

// Unknown is the reading of an invariant a value it reads is absent from.
const Unknown Truth = 0

// False is the reading of a broken invariant.
const False Truth = 1

// True is the reading of an invariant that holds.
const True Truth = 2

// Number is a number by its exact decimal value: digits × 10^exponent, with no leading or
// trailing zero in digits, and zero as no digits at all — so equal values are equal here.
type Number struct {
	negative bool
	digits   string
	exponent int64
}

// maxExponent bounds a written exponent, so the arithmetic on it below cannot overflow.
const maxExponent = 1 << 60

// ParseNumber reads a decimal spelling such as `-10.50` or `15e-1`; ok is false when it is not
// one.
func ParseNumber(text string) (Number, bool) {
	negative := false
	rest := text
	if strings.HasPrefix(rest, "-") {
		negative = true
		rest = rest[1:]
	} else if strings.HasPrefix(rest, "+") {
		rest = rest[1:]
	}
	mantissa := rest
	var exponent int64
	if at := strings.IndexAny(rest, "eE"); at >= 0 {
		mantissa = rest[:at]
		parsed, err := strconv.ParseInt(rest[at+1:], 10, 64)
		if err != nil || parsed > maxExponent || parsed < -maxExponent {
			return Number{}, false
		}
		exponent = parsed
	}
	whole, fraction, _ := strings.Cut(mantissa, ".")
	if (whole == "" && fraction == "") || !digitsOnly(whole) || !digitsOnly(fraction) {
		return Number{}, false
	}
	all := whole + fraction
	leading := strings.TrimLeft(all, "0")
	significant := strings.TrimRight(leading, "0")
	if significant == "" {
		return Number{}, true
	}
	dropped := int64(len(leading) - len(significant))
	scale := int64(len(fraction))
	return Number{negative: negative, digits: significant, exponent: exponent - scale + dropped}, true
}

// digitsOnly reports whether every byte of part is an ASCII digit.
func digitsOnly(part string) bool {
	for index := 0; index < len(part); index++ {
		if part[index] < '0' || part[index] > '9' {
			return false
		}
	}
	return true
}

// sign is -1, 0 or 1.
func (n Number) sign() int {
	if n.digits == "" {
		return 0
	}
	if n.negative {
		return -1
	}
	return 1
}

// Cmp orders two numbers by their exact value: -1, 0 or 1.
func (n Number) Cmp(other Number) int {
	if n.sign() != other.sign() {
		if n.sign() < other.sign() {
			return -1
		}
		return 1
	}
	if n.digits == "" {
		return 0
	}
	magnitude := 0
	left := int64(len(n.digits)) + n.exponent
	right := int64(len(other.digits)) + other.exponent
	if left < right {
		magnitude = -1
	} else if left > right {
		magnitude = 1
	} else {
		magnitude = strings.Compare(n.digits, other.digits)
	}
	if n.negative {
		return -magnitude
	}
	return magnitude
}

// kind is what a Fact holds.
type kind int

const (
	absent kind = iota
	boolean
	number
	text
)

// Fact is one value an invariant reads, or no value at all: the zero Fact is absent.
type Fact struct {
	kind   kind
	flag   bool
	number Number
	text   string
}

// Absent is the fact of a value that is not there.
var Absent Fact

// Bool is a Boolean.
func Bool(value bool) Fact {
	return Fact{kind: boolean, flag: value}
}

// Integer is an `Integer`.
func Integer(value int64) Fact {
	return NumberOf(strconv.FormatInt(value, 10))
}

// Count is the number of elements, or of Unicode scalar values in a text.
func Count(value int) Fact {
	return Integer(int64(value))
}

// NumberOf is a `Decimal` or a number literal by its spelling, and absent when it spells no
// number.
func NumberOf(spelling string) Fact {
	parsed, ok := ParseNumber(spelling)
	if !ok {
		return Absent
	}
	return Fact{kind: number, number: parsed}
}

// Text is text: a `String`, an enum variant's name, or the rendering of a `Timestamp`,
// `Duration` or `Uuid`.
func Text(value string) Fact {
	return Fact{kind: text, text: value}
}

// Bytes is `Bytes`, as the padded base64 the wire carries them in.
func Bytes(value []byte) Fact {
	return Text(base64.StdEncoding.EncodeToString(value))
}

// Op is a comparison operator.
type Op int

// Eq is `==`.
const Eq Op = 0

// Ne is `!=`.
const Ne Op = 1

// Lt is `<`.
const Lt Op = 2

// Le is `<=`.
const Le Op = 3

// Gt is `>`.
const Gt Op = 4

// Ge is `>=`.
const Ge Op = 5

// orders reports whether the operator orders its operands rather than equating them.
func (o Op) orders() bool {
	return o != Eq && o != Ne
}

// accepts reports whether an ordering of -1, 0 or 1 satisfies the operator.
func (o Op) accepts(ordering int) Truth {
	holds := false
	switch o {
	case Eq:
		holds = ordering == 0
	case Ne:
		holds = ordering != 0
	case Lt:
		holds = ordering < 0
	case Le:
		holds = ordering <= 0
	case Gt:
		holds = ordering > 0
	case Ge:
		holds = ordering >= 0
	}
	return Known(holds)
}

// TextOp is a string operator.
type TextOp int

// StartsWith is `starts_with`.
const StartsWith TextOp = 0

// EndsWith is `ends_with`.
const EndsWith TextOp = 1

// Contains is `contains`.
const Contains TextOp = 2

// Known is the reading of a decided truth.
func Known(holds bool) Truth {
	if holds {
		return True
	}
	return False
}

// Broken reports whether a value breaks the invariant: false, and not merely unknown.
func Broken(truth Truth) bool {
	return truth == False
}

// All is Kleene conjunction: false dominates, then unknown.
func All(truths ...Truth) Truth {
	result := True
	for _, truth := range truths {
		if truth == False {
			return False
		}
		if truth == Unknown {
			result = Unknown
		}
	}
	return result
}

// Any is Kleene disjunction: true dominates, then unknown.
func Any(truths ...Truth) Truth {
	result := False
	for _, truth := range truths {
		if truth == True {
			return True
		}
		if truth == Unknown {
			result = Unknown
		}
	}
	return result
}

// Not is Kleene negation: unknown stays unknown.
func Not(truth Truth) Truth {
	switch truth {
	case True:
		return False
	case False:
		return True
	}
	return Unknown
}

// Truthy reports whether the fact is observed and truthy: true, a number other than zero, or
// text that is neither empty nor `false`.
func Truthy(fact Fact) Truth {
	switch fact.kind {
	case boolean:
		return Known(fact.flag)
	case number:
		return Known(fact.number.digits != "")
	case text:
		return Known(fact.text != "" && fact.text != "false")
	}
	return Unknown
}

// AnyOf reports whether the observed fact equals one of values.
func AnyOf(fact Fact, values ...Fact) Truth {
	if fact.kind == absent {
		return Unknown
	}
	for _, value := range values {
		if value == fact {
			return True
		}
	}
	return False
}

// NoneOf reports whether the observed fact equals none of values.
func NoneOf(fact Fact, values ...Fact) Truth {
	return Not(AnyOf(fact, values...))
}

// TextMatch reports whether the observed text begins with, ends with or contains literal, byte
// for byte; any other observed value, or a literal that is not text, does not match.
func TextMatch(fact Fact, op TextOp, literal Fact) Truth {
	if fact.kind == absent {
		return Unknown
	}
	if fact.kind != text || literal.kind != text {
		return False
	}
	switch op {
	case StartsWith:
		return Known(strings.HasPrefix(fact.text, literal.text))
	case EndsWith:
		return Known(strings.HasSuffix(fact.text, literal.text))
	}
	return Known(strings.Contains(fact.text, literal.text))
}

// FoldMatch reports whether the observed text equals one of literals under ASCII case folding.
func FoldMatch(fact Fact, literals ...string) Truth {
	if fact.kind == absent {
		return Unknown
	}
	if fact.kind != text {
		return False
	}
	for _, literal := range literals {
		if foldEqual(fact.text, literal) {
			return True
		}
	}
	return False
}

// foldEqual is equality under ASCII case folding, every other byte compared as it is.
func foldEqual(left string, right string) bool {
	if len(left) != len(right) {
		return false
	}
	for index := 0; index < len(left); index++ {
		if lower(left[index]) != lower(right[index]) {
			return false
		}
	}
	return true
}

// lower folds one ASCII upper-case letter.
func lower(character byte) byte {
	if character >= 'A' && character <= 'Z' {
		return character + ('a' - 'A')
	}
	return character
}

// Compare compares two operands. instant says a fact operand is a declared `Timestamp`, ordered
// by the instant it names; bytes says every fact operand orders its text by UTF-8 bytes.
func Compare(left Fact, op Op, right Fact, instant bool, bytes bool) Truth {
	if left.kind == absent || right.kind == absent {
		return Unknown
	}
	if instant && left.kind == text && right.kind == text {
		leftSeconds, leftNanos, leftOk := rfc3339(left.text)
		rightSeconds, rightNanos, rightOk := rfc3339(right.text)
		if leftOk && rightOk {
			return op.accepts(instantOrder(leftSeconds, leftNanos, rightSeconds, rightNanos))
		}
	}
	if left.kind == number && right.kind == number {
		return op.accepts(left.number.Cmp(right.number))
	}
	if left.kind == text && right.kind == text && op.orders() {
		if !instant && bytes {
			return op.accepts(strings.Compare(left.text, right.text))
		}
		return Unknown
	}
	if op.orders() {
		return Unknown
	}
	return Known((left == right) == (op == Eq))
}

// instantOrder orders two instants given as seconds and nanoseconds: -1, 0 or 1.
func instantOrder(leftSeconds int64, leftNanos int64, rightSeconds int64, rightNanos int64) int {
	if leftSeconds != rightSeconds {
		if leftSeconds < rightSeconds {
			return -1
		}
		return 1
	}
	if leftNanos != rightNanos {
		if leftNanos < rightNanos {
			return -1
		}
		return 1
	}
	return 0
}

// rfc3339 is the instant an RFC 3339 `date-time` names, as seconds from the epoch and
// nanoseconds; ok is false when it names none.
func rfc3339(value string) (int64, int64, bool) {
	digits := func(from int, to int) (int64, bool) {
		if from >= to || to > len(value) {
			return 0, false
		}
		var total int64
		for index := from; index < to; index++ {
			if value[index] < '0' || value[index] > '9' {
				return 0, false
			}
			total = total*10 + int64(value[index]-'0')
		}
		return total, true
	}
	at := func(index int, expected string) bool {
		return index < len(value) && strings.IndexByte(expected, value[index]) >= 0
	}
	if !at(4, "-") || !at(7, "-") || !at(10, "Tt") || !at(13, ":") || !at(16, ":") {
		return 0, 0, false
	}
	year, yearOk := digits(0, 4)
	month, monthOk := digits(5, 7)
	day, dayOk := digits(8, 10)
	if !yearOk || !monthOk || !dayOk {
		return 0, 0, false
	}
	leap := (year%4 == 0 && year%100 != 0) || year%400 == 0
	var length int64
	switch month {
	case 1, 3, 5, 7, 8, 10, 12:
		length = 31
	case 4, 6, 9, 11:
		length = 30
	case 2:
		length = 28
		if leap {
			length = 29
		}
	default:
		return 0, 0, false
	}
	if day < 1 || day > length {
		return 0, 0, false
	}
	hour, hourOk := digits(11, 13)
	minute, minuteOk := digits(14, 16)
	second, secondOk := digits(17, 19)
	if !hourOk || !minuteOk || !secondOk || hour > 23 || minute > 59 || second > 59 {
		return 0, 0, false
	}
	position := 19
	var nanos int64
	if at(position, ".") {
		start := position + 1
		end := start
		for end < len(value) && value[end] >= '0' && value[end] <= '9' {
			end++
		}
		width := end - start
		if width == 0 || width > 9 {
			return 0, 0, false
		}
		fraction, ok := digits(start, end)
		if !ok {
			return 0, 0, false
		}
		for padding := width; padding < 9; padding++ {
			fraction *= 10
		}
		nanos = fraction
		position = end
	}
	var offset int64
	rest := value[position:]
	switch {
	case rest == "Z" || rest == "z":
		offset = 0
	case len(rest) == 6 && (rest[0] == '+' || rest[0] == '-') && rest[3] == ':':
		hours, hoursOk := digits(position+1, position+3)
		minutes, minutesOk := digits(position+4, position+6)
		if !hoursOk || !minutesOk || hours > 23 || minutes > 59 {
			return 0, 0, false
		}
		offset = hours*3600 + minutes*60
		if rest[0] == '-' {
			offset = -offset
		}
	default:
		return 0, 0, false
	}
	shifted := year
	if month <= 2 {
		shifted--
	}
	era := shifted
	if era < 0 {
		era -= 399
	}
	era /= 400
	yearOfEra := shifted - era*400
	shiftedMonth := month - 3
	if month <= 2 {
		shiftedMonth = month + 9
	}
	dayOfYear := (153*shiftedMonth+2)/5 + day - 1
	dayOfEra := yearOfEra*365 + yearOfEra/4 - yearOfEra/100 + dayOfYear
	days := era*146097 + dayOfEra - 719468
	return days*86400 + hour*3600 + minute*60 + second - offset, nanos, true
}
