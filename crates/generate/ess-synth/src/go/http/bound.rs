//! The Go server's half of an `ess-http/1` binding (`ess generate synthesize --target go --http`).
//!
//! The Rust server's behaviour (`crate::rust::http::bound`), in the second language: template
//! dispatch, the request the binding declares, the answer it declares per outcome, and the
//! standard refusals it overrides (`docs/design/http-presentation-binding.md`, "Precedence" and
//! "What the servers write (k5, k6)"). Emitted only with a binding, and only for a surface the
//! binding routes something of; every other surface keeps its bytes.
//!
//! What is read from the binding is read through `ess_gen::http`, as the Rust server reads it.

use std::fmt::Write as _;

use ess_compiler::ir::{EssIr, ResolvedCommand, ResolvedComponent, ResolvedField, ResolvedView};
use ess_gen::ess_http::{
    self, AnswerSource, Body, Encoding, Header, Param, RedirectMode, RefusalBody, RefusalKey,
};
use ess_gen::http::{self, Served};

use super::super::layout::Layout;
use super::super::name;
use super::super::selection::go_string;
use super::{ident, Row};

/// The `SURFACE_HELPERS` text as a bound package needs it: an answer carries headers, may carry no
/// body and no media type, and a `204` is written with neither a length nor a body.
///
/// # Panics
///
/// When the fixed text no longer holds what this rewrites: the two cannot drift apart.
pub(super) fn surface_helpers(base: &str) -> String {
    let mut helpers = base.to_owned();
    for (from, to) in [
        (
            "\tbody        string\n}",
            "\tbody        string\n\t// headers is every header the binding declares for this answer, in the order it\n\t// is written.\n\theaders [][2]string\n}",
        ),
        (
            "\twriter.Header().Set(\"Content-Type\", r.contentType)\n\twriter.Header().Set(\"Content-Length\", strconv.Itoa(len(r.body)))\n\twriter.Header().Set(\"Connection\", \"close\")\n\twriter.WriteHeader(r.status)\n\t_, _ = writer.Write([]byte(r.body))\n}",
            "\tif r.contentType != \"\" {\n\t\twriter.Header().Set(\"Content-Type\", r.contentType)\n\t} else {\n\t\t// An answer without a body names no media type, and net/http must not sniff one.\n\t\twriter.Header()[\"Content-Type\"] = nil\n\t}\n\t// A 1xx, 204 or 304 has no content and no Content-Length (RFC 9110 §8.6 and §15).\n\tbodiless := r.status < 200 || r.status == http.StatusNoContent || r.status == http.StatusNotModified\n\tif !bodiless {\n\t\twriter.Header().Set(\"Content-Length\", strconv.Itoa(len(r.body)))\n\t}\n\tfor _, header := range r.headers {\n\t\t// Assigned rather than Set: the name is written as the binding spells it.\n\t\twriter.Header()[header[0]] = append(writer.Header()[header[0]], header[1])\n\t}\n\twriter.Header().Set(\"Connection\", \"close\")\n\twriter.WriteHeader(r.status)\n\tif !bodiless {\n\t\t_, _ = writer.Write([]byte(r.body))\n\t}\n}",
        ),
    ] {
        assert!(
            helpers.contains(from),
            "the surface helpers no longer hold what a bound package rewrites: {from}"
        );
        helpers = helpers.replacen(from, to, 1);
    }
    helpers
}

/// What a bound package adds after the query reader: template matching, the `form` and `query`
/// decoders, and the writers of a declared answer. The Rust server's `http` module, function for
/// function, refusal for refusal.
pub(super) const BOUND: &str = r##"
// ---- an `ess-http/1` binding, served ------------------------------------------------------------

// mediaForm is the media type of a `form` request body.
const mediaForm = "application/x-www-form-urlencoded"

// flatKind is how one flat value — a path segment, a `form` or `query` value — is read as the
// JSON value its field's decoder reads: the table `ess_gen::http::Flat` states, which the Rust
// server reads too.
type flatKind int

const (
	// flatString is `String`, or a newtype of it: any text, the empty text included.
	flatString flatKind = iota
	// flatText is any other scalar a decoder reads from JSON text. Empty text is malformed.
	flatText
	// flatInteger is `Integer`: a whole number's spelling is a JSON number, anything else stays
	// text for the decoder to refuse by name. Empty text is malformed.
	flatInteger
	// flatBoolean is `Boolean`: `true` or `false`, anything else stays text. Empty text is
	// malformed.
	flatBoolean
)

// flatShape is how many values one `form` or `query` key carries.
type flatShape int

const (
	// shapeOne is exactly one; given twice, the request is malformed (RFC 6749 §3.1).
	shapeOne flatShape = iota
	// shapeOptional is at most one; absent, the field is absent.
	shapeOptional
	// shapeList is any number: the key repeats once per element.
	shapeList
)

// flatField is one field a `form` or `query` request carries, by its wire name.
type flatField struct {
	wire  string
	kind  flatKind
	shape flatShape
}

// pathMember is one field a path variable carries, by its wire name, read.
type pathMember struct {
	wire  string
	value any
}

// maybeText is a scalar of an answer as text, or nothing for an absent optional.
type maybeText struct {
	text    string
	present bool
}

// namedText is one challenge param or redirect parameter.
type namedText struct {
	name  string
	value maybeText
}

// some is text that is there.
func some(text string) maybeText {
	return maybeText{text: text, present: true}
}

// segments is a path's segments after its leading /: none for the root, and not ok for a path
// without one.
func segments(path string) ([]string, bool) {
	rest, ok := strings.CutPrefix(path, "/")
	if !ok {
		return nil, false
	}
	if rest == "" {
		return []string{}, true
	}
	return strings.Split(rest, "/"), true
}

// matched is the raw text of each variable segment of path, in order, where path matches
// template: as many segments, each literal equal, and each variable holding a segment that is
// not empty. Not ok where it does not match.
func matched(template string, path string) ([]string, bool) {
	expected, ok := segments(template)
	if !ok {
		return nil, false
	}
	given, ok := segments(path)
	if !ok || len(expected) != len(given) {
		return nil, false
	}
	values := []string{}
	for index, segment := range expected {
		if strings.HasPrefix(segment, "{") && strings.HasSuffix(segment, "}") {
			if given[index] == "" {
				return nil, false
			}
			values = append(values, given[index])
		} else if segment != given[index] {
			return nil, false
		}
	}
	return values, true
}

// flatValue is one flat text as the JSON value its field's decoder reads, or a refusal naming at
// when the text is empty and the field is not a `String`.
func flatValue(text string, kind flatKind, at string) (any, error) {
	switch {
	case kind == flatString:
		return text, nil
	case text == "":
		return nil, fmt.Errorf("%s: expected a value, found the empty text", at)
	case kind == flatInteger && whole(text):
		return json.Number(text), nil
	case kind == flatBoolean && text == "true":
		return true, nil
	case kind == flatBoolean && text == "false":
		return false, nil
	}
	return text, nil
}

// pathValue is one path variable's raw segment, percent-decoded (a + stays a +), as the JSON
// value its field's decoder reads.
func pathValue(raw string, kind flatKind, at string) (any, error) {
	text, err := url.PathUnescape(raw)
	if err != nil || !utf8.ValidString(text) {
		return nil, fmt.Errorf("%s: expected percent-encoded UTF-8 text, found `%s`", at, raw)
	}
	return flatValue(text, kind, at)
}

// flatObject is a `form` body or a query string as the object the input's decoder reads: one
// member per declared field the text names, keyed by its wire name, and a list as an array, empty
// when its key is absent.
//
// Keys and values are form-decoded (+ and %XX, UTF-8). A key the path carries, no field declares,
// or that does not decode is unknown: ignored where ignore, and malformed otherwise. A scalar key
// given twice, a value that does not decode and an empty value for a field that is not a
// `String` are malformed.
func flatObject(text string, fields []flatField, carried []string, ignore bool, at string) (map[string]any, error) {
	found := make([][]string, len(fields))
	for _, pair := range strings.Split(text, "&") {
		if pair == "" {
			continue
		}
		rawKey, rawValue, _ := strings.Cut(pair, "=")
		key, ok := formDecoded(rawKey)
		if !ok {
			if ignore {
				continue
			}
			return nil, fmt.Errorf("%s: a key is not percent-encoded UTF-8: `%s`", at, rawKey)
		}
		place := nested(at, key)
		index := -1
		for position, field := range fields {
			if field.wire == key {
				index = position
				break
			}
		}
		pathCarries := false
		for _, wire := range carried {
			if wire == key {
				pathCarries = true
			}
		}
		if index < 0 || pathCarries {
			if ignore {
				continue
			}
			if pathCarries {
				return nil, fmt.Errorf("%s: the path carries this field, so it is not a parameter here", place)
			}
			return nil, fmt.Errorf("%s: no field of this request is named so", place)
		}
		value, ok := formDecoded(rawValue)
		if !ok {
			return nil, fmt.Errorf("%s: expected percent-encoded UTF-8 text, found `%s`", place, rawValue)
		}
		found[index] = append(found[index], value)
	}
	object := map[string]any{}
	for index, field := range fields {
		place := nested(at, field.wire)
		if field.shape == shapeList {
			items := make([]any, 0, len(found[index]))
			for _, text := range found[index] {
				value, err := flatValue(text, field.kind, place)
				if err != nil {
					return nil, err
				}
				items = append(items, value)
			}
			object[field.wire] = items
			continue
		}
		switch len(found[index]) {
		case 0:
		case 1:
			value, err := flatValue(found[index][0], field.kind, place)
			if err != nil {
				return nil, err
			}
			object[field.wire] = value
		default:
			return nil, fmt.Errorf("%s: expected one value, found %d", place, len(found[index]))
		}
	}
	return object, nil
}

// contentTypeIs reports whether the request's Content-Type is expected, its parameters ignored
// and its case too.
func contentTypeIs(request *http.Request, expected string) bool {
	values := request.Header.Values("Content-Type")
	if len(values) == 0 {
		return false
	}
	media, _, _ := strings.Cut(values[0], ";")
	return strings.EqualFold(strings.TrimSpace(media), expected)
}

// formObject is a `form` request's body as flatObject reads it, or a refusal where the body is
// not `application/x-www-form-urlencoded` or not UTF-8.
func formObject(request *http.Request, body []byte, fields []flatField, carried []string, ignore bool) (map[string]any, error) {
	if !contentTypeIs(request, mediaForm) {
		return nil, fmt.Errorf("the body is not `application/x-www-form-urlencoded`")
	}
	if !utf8.Valid(body) {
		return nil, fmt.Errorf("the body is not UTF-8")
	}
	return flatObject(string(body), fields, carried, ignore, "body")
}

// jsonBody is a `json` body as the value the input's decoder reads, read by parseJSON as the Rust
// server reads one. Where the body is not required, no body, or one of only whitespace, is the
// input {}.
func jsonBody(body []byte, required bool) (any, error) {
	if !utf8.Valid(body) {
		return nil, fmt.Errorf("the body is not UTF-8")
	}
	if !required && len(bytes.TrimSpace(body)) == 0 {
		return map[string]any{}, nil
	}
	value, err := parseJSON(string(body))
	if err != nil {
		return nil, fmt.Errorf("the body is not JSON: %s", err)
	}
	return decoderValue(value), nil
}

// jsonMember is one member of an object parseJSON read.
type jsonMember struct {
	name  string
	value any
}

// jsonObject is an object parseJSON read: every member in the order it arrived, a repeated name
// included.
type jsonObject []jsonMember

// jsonDepth is how deep a document may nest: the Rust server's reader refuses deeper.
const jsonDepth = 64

// jsonError is where a document stopped being JSON and what was legal there, worded as the Rust
// server's reader words it.
func jsonError(at int, expected string) error {
	return fmt.Errorf("at byte %d: expected %s", at, expected)
}

// parseJSON reads exactly one JSON document followed by nothing but whitespace, as the Rust
// server's `json::parse` reads one, refusal for refusal: text after the document, a lone surrogate
// escape and nesting past jsonDepth are refused, a control character inside a string is not, a
// number keeps its spelling and an object every member in order (encoding/json differs on each).
func parseJSON(text string) (any, error) {
	value, at, err := readJSONValue(text, skipJSONSpace(text, 0), 0)
	if err != nil {
		return nil, err
	}
	if at = skipJSONSpace(text, at); at != len(text) {
		return nil, jsonError(at, "the end of the document")
	}
	return value, nil
}

// skipJSONSpace is the offset of the first byte at or after at that is not JSON whitespace.
func skipJSONSpace(text string, at int) int {
	for at < len(text) && (text[at] == ' ' || text[at] == '\t' || text[at] == '\n' || text[at] == '\r') {
		at++
	}
	return at
}

// readJSONValue reads one value at at, which is already past any leading whitespace, and the
// offset after it.
func readJSONValue(text string, at int, depth int) (any, int, error) {
	if depth > jsonDepth {
		return nil, at, jsonError(at, "a document nested less than 64 deep")
	}
	if at >= len(text) {
		return nil, at, jsonError(at, "a value")
	}
	switch character := text[at]; {
	case character == '{':
		return readJSONObject(text, at, depth)
	case character == '[':
		return readJSONArray(text, at, depth)
	case character == '"':
		read, next, err := readJSONText(text, at)
		if err != nil {
			return nil, next, err
		}
		return read, next, nil
	case character == 't':
		return readJSONWord(text, at, "true", true)
	case character == 'f':
		return readJSONWord(text, at, "false", false)
	case character == 'n':
		return readJSONWord(text, at, "null", nil)
	case character == '-' || (character >= '0' && character <= '9'):
		return readJSONNumber(text, at)
	}
	return nil, at, jsonError(at, "a value")
}

// readJSONWord reads one of the three bare words.
func readJSONWord(text string, at int, word string, value any) (any, int, error) {
	if !strings.HasPrefix(text[at:], word) {
		return nil, at, jsonError(at, "`"+word+"`")
	}
	return value, at + len(word), nil
}

// jsonDigits is the offset after the ASCII digits starting at at.
func jsonDigits(text string, at int) int {
	for at < len(text) && text[at] >= '0' && text[at] <= '9' {
		at++
	}
	return at
}

// readJSONNumber reads a number, keeping its spelling.
func readJSONNumber(text string, at int) (any, int, error) {
	start := at
	if text[at] == '-' {
		at++
	}
	digits := at
	if at = jsonDigits(text, at); at == digits {
		return nil, at, jsonError(at, "a digit")
	}
	if at < len(text) && text[at] == '.' {
		fraction := at + 1
		if at = jsonDigits(text, fraction); at == fraction {
			return nil, at, jsonError(at, "a digit after the decimal point")
		}
	}
	if at < len(text) && (text[at] == 'e' || text[at] == 'E') {
		at++
		if at < len(text) && (text[at] == '+' || text[at] == '-') {
			at++
		}
		exponent := at
		if at = jsonDigits(text, exponent); at == exponent {
			return nil, at, jsonError(at, "a digit in the exponent")
		}
	}
	return json.Number(text[start:at]), at, nil
}

// readJSONText reads a string, resolving its escapes.
func readJSONText(text string, at int) (string, int, error) {
	at++
	var out strings.Builder
	for {
		if at >= len(text) {
			return "", at, jsonError(at, "a closing quote")
		}
		switch text[at] {
		case '"':
			return out.String(), at + 1, nil
		case '\\':
			at++
			if at >= len(text) {
				return "", at, jsonError(at, "an escape")
			}
			escape := text[at]
			at++
			switch escape {
			case '"', '\\', '/':
				out.WriteByte(escape)
			case 'b':
				out.WriteByte('\b')
			case 'f':
				out.WriteByte('\f')
			case 'n':
				out.WriteByte('\n')
			case 'r':
				out.WriteByte('\r')
			case 't':
				out.WriteByte('\t')
			case 'u':
				character, next, err := readJSONUnicode(text, at)
				if err != nil {
					return "", next, err
				}
				out.WriteRune(character)
				at = next
			default:
				return "", at, jsonError(at, "one of \" \\ / b f n r t u")
			}
		default:
			start := at
			for at < len(text) && text[at] != '"' && text[at] != '\\' {
				at++
			}
			out.WriteString(text[start:at])
		}
	}
}

// readJSONUnicode reads a \u escape after its \u, joining a surrogate pair; a lone half of one is
// refused rather than read as U+FFFD, a character nobody sent.
func readJSONUnicode(text string, at int) (rune, int, error) {
	first, at, err := readJSONHex(text, at)
	if err != nil {
		return 0, at, err
	}
	if first >= 0xD800 && first < 0xDC00 {
		if at+1 >= len(text) || text[at] != '\\' || text[at+1] != 'u' {
			return 0, at, jsonError(at, "the low half of a surrogate pair")
		}
		second, next, err := readJSONHex(text, at+2)
		if err != nil {
			return 0, next, err
		}
		if second < 0xDC00 || second >= 0xE000 {
			return 0, next, jsonError(next, "the low half of a surrogate pair")
		}
		return rune(0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00)), next, nil
	}
	if first >= 0xDC00 && first < 0xE000 {
		return 0, at, jsonError(at, "a character")
	}
	return rune(first), at, nil
}

// readJSONHex reads exactly four hexadecimal digits.
func readJSONHex(text string, at int) (int, int, error) {
	value := 0
	for count := 0; count < 4; count++ {
		if at >= len(text) {
			return 0, at, jsonError(at, "four hexadecimal digits")
		}
		var digit int
		switch character := text[at]; {
		case character >= '0' && character <= '9':
			digit = int(character - '0')
		case character >= 'a' && character <= 'f':
			digit = int(character-'a') + 10
		case character >= 'A' && character <= 'F':
			digit = int(character-'A') + 10
		default:
			return 0, at, jsonError(at, "four hexadecimal digits")
		}
		value = value*16 + digit
		at++
	}
	return value, at, nil
}

// readJSONArray reads an array.
func readJSONArray(text string, at int, depth int) (any, int, error) {
	items := []any{}
	if at = skipJSONSpace(text, at+1); at < len(text) && text[at] == ']' {
		return items, at + 1, nil
	}
	for {
		item, next, err := readJSONValue(text, skipJSONSpace(text, at), depth+1)
		if err != nil {
			return nil, next, err
		}
		items = append(items, item)
		switch at = skipJSONSpace(text, next); {
		case at < len(text) && text[at] == ',':
			at++
		case at < len(text) && text[at] == ']':
			return items, at + 1, nil
		default:
			return nil, at, jsonError(at, "`,` or `]`")
		}
	}
}

// readJSONObject reads an object.
func readJSONObject(text string, at int, depth int) (any, int, error) {
	object := jsonObject{}
	if at = skipJSONSpace(text, at+1); at < len(text) && text[at] == '}' {
		return object, at + 1, nil
	}
	for {
		if at = skipJSONSpace(text, at); at >= len(text) || text[at] != '"' {
			return nil, at, jsonError(at, "a member name")
		}
		name, next, err := readJSONText(text, at)
		if err != nil {
			return nil, next, err
		}
		if at = skipJSONSpace(text, next); at >= len(text) || text[at] != ':' {
			return nil, at, jsonError(at, "`:`")
		}
		value, next, err := readJSONValue(text, skipJSONSpace(text, at+1), depth+1)
		if err != nil {
			return nil, next, err
		}
		object = append(object, jsonMember{name: name, value: value})
		switch at = skipJSONSpace(text, next); {
		case at < len(text) && text[at] == ',':
			at++
		case at < len(text) && text[at] == '}':
			return object, at + 1, nil
		default:
			return nil, at, jsonError(at, "`,` or `}`")
		}
	}
}

// decoderValue is a value parseJSON read, as the input's decoder reads one: an object as a map
// holding the first member of each name, the one the Rust server's decoder reads. The body itself
// stays a map, the shape every reader of a request hands withPath.
func decoderValue(value any) any {
	shaped, ok := value.(jsonObject)
	if !ok {
		return memberValue(value)
	}
	object := map[string]any{}
	for _, member := range shaped {
		if _, seen := object[member.name]; !seen {
			object[member.name] = memberValue(member.value)
		}
	}
	return object
}

// memberValue is a value inside the body, as the input's decoder reads one.
func memberValue(value any) any {
	switch shaped := value.(type) {
	case jsonObject:
		return decoderValue(shaped)
	case []any:
		items := make([]any, len(shaped))
		for index, item := range shaped {
			items[index] = memberValue(item)
		}
		return items
	}
	return value
}

// withPath is value with the members the path carries added. A path variable's value wins over
// the request for its field, and the request naming that field is malformed: the path carries it.
// A value that is not an object is the decoder's to refuse.
func withPath(value any, path []pathMember, at string) (any, error) {
	object, ok := value.(map[string]any)
	if !ok || len(path) == 0 {
		return value, nil
	}
	for _, member := range path {
		if _, present := object[member.wire]; present {
			return nil, fmt.Errorf("%s: the path carries this field, so the request does not", nested(at, member.wire))
		}
	}
	for _, member := range path {
		object[member.wire] = member.value
	}
	return object, nil
}

// memberOf is the member name of an object, the first of that name where parseJSON read it, or
// nil.
func memberOf(value any, name string) any {
	switch object := value.(type) {
	case map[string]any:
		return object[name]
	case jsonObject:
		for _, member := range object {
			if member.name == name {
				return member.value
			}
		}
	}
	return nil
}

// scalarText is a scalar of an answer as text: a string as itself, a number as its spelling, a
// boolean as true or false; absent for anything else, an absent optional included.
func scalarText(value any) maybeText {
	switch scalar := value.(type) {
	case string:
		return some(scalar)
	case json.Number:
		return some(scalar.String())
	case bool:
		return some(strconv.FormatBool(scalar))
	}
	return maybeText{}
}

// notFieldText is the first rune of text a field value cannot carry — a control character other
// than a horizontal tab — or not ok.
func notFieldText(text string) (rune, bool) {
	for _, character := range text {
		if unicode.IsControl(character) && character != '\t' {
			return character, true
		}
	}
	return 0, false
}

// pushHeader adds the header name with value to answer; an absent value omits it. A value that
// is not a field value (RFC 9110 §5.5) is never written, truncated or re-encoded: a header value
// from input must not inject a header.
func pushHeader(answer *response, name string, value maybeText) error {
	if !value.present {
		return nil
	}
	if found, ok := notFieldText(value.text); ok {
		return fmt.Errorf("header `%s` would hold %q, which is not field-value text", name, found)
	}
	if strings.HasPrefix(value.text, " ") || strings.HasPrefix(value.text, "\t") || strings.HasSuffix(value.text, " ") || strings.HasSuffix(value.text, "\t") {
		return fmt.Errorf("header `%s` would start or end with a space or a tab, which a reader strips", name)
	}
	answer.headers = append(answer.headers, [2]string{name, value.text})
	return nil
}

// challenge is one RFC 9110 §11.6.1 challenge: scheme, then each param present as name="value",
// in the order given, separated by ", ". Each value is a quoted string whose " and \ are escaped,
// so no value can leave it; an absent param is omitted.
func challenge(scheme string, params []namedText) (string, error) {
	out := scheme
	first := true
	for _, param := range params {
		if !param.value.present {
			continue
		}
		if found, ok := notFieldText(param.value.text); ok {
			return "", fmt.Errorf("challenge param `%s` would hold %q, which a quoted string cannot carry", param.name, found)
		}
		if first {
			out += " "
		} else {
			out += ", "
		}
		first = false
		out += param.name + "=\"" + strings.NewReplacer("\"", "\\\"", "\\", "\\\\").Replace(param.value.text) + "\""
	}
	return out, nil
}

// formEncoded is text as application/x-www-form-urlencoded writes it: ASCII letters, digits and
// *-._ as themselves, a space as +, every other byte as %XX.
func formEncoded(text string) string {
	var out strings.Builder
	for index := 0; index < len(text); index++ {
		switch character := text[index]; {
		case (character >= 'A' && character <= 'Z') || (character >= 'a' && character <= 'z') || (character >= '0' && character <= '9') || character == '*' || character == '-' || character == '.' || character == '_':
			out.WriteByte(character)
		case character == ' ':
			out.WriteByte('+')
		default:
			fmt.Fprintf(&out, "%%%02X", character)
		}
	}
	return out.String()
}

// redirect is where a redirect sends the caller: location, with each parameter present appended
// to its query (keeping a query it already has) or written as its fragment, form-encoded, in the
// order given. Refused when location is absent or is not an absolute URI without a fragment: a
// scheme (an ASCII letter, then letters, digits, +, - and .), a :, and visible ASCII only.
func redirect(location maybeText, fragment bool, parameters []namedText) (string, error) {
	if !location.present {
		return "", fmt.Errorf("the redirect has no location")
	}
	text := location.text
	scheme, _, colon := strings.Cut(text, ":")
	absolute := colon && scheme != "" && ((scheme[0] >= 'A' && scheme[0] <= 'Z') || (scheme[0] >= 'a' && scheme[0] <= 'z'))
	for index := 1; absolute && index < len(scheme); index++ {
		character := scheme[index]
		absolute = (character >= 'A' && character <= 'Z') || (character >= 'a' && character <= 'z') || (character >= '0' && character <= '9') || character == '+' || character == '-' || character == '.'
	}
	graphic := true
	for index := 0; index < len(text); index++ {
		if text[index] < 0x21 || text[index] > 0x7e {
			graphic = false
		}
	}
	if !absolute || strings.Contains(text, "#") || !graphic {
		return "", fmt.Errorf("the redirect location %q is not an absolute URI without a fragment", text)
	}
	written := []string{}
	for _, parameter := range parameters {
		if parameter.value.present {
			written = append(written, formEncoded(parameter.name)+"="+formEncoded(parameter.value.text))
		}
	}
	if len(written) == 0 {
		return text, nil
	}
	joined := strings.Join(written, "&")
	if fragment {
		return text + "#" + joined, nil
	}
	_, query, hasQuery := strings.Cut(text, "?")
	switch {
	case hasQuery && (query == "" || strings.HasSuffix(query, "&")):
		return text + joined, nil
	case hasQuery:
		return text + "&" + joined, nil
	}
	return text + "?" + joined, nil
}

// orderedObject is an object this package's encoders write: its members in the order its type
// declares them, the order the Rust server writes them in, where a map would leave encoding/json
// to sort them.
type orderedObject struct {
	names  []string
	values []any
}

// set appends one member.
func (o *orderedObject) set(name string, value any) {
	o.names = append(o.names, name)
	o.values = append(o.values, value)
}

// MarshalJSON writes the members in the order they were set.
func (o *orderedObject) MarshalJSON() ([]byte, error) {
	var out bytes.Buffer
	out.WriteByte('{')
	for index, name := range o.names {
		if index > 0 {
			out.WriteByte(',')
		}
		key, err := json.Marshal(name)
		if err != nil {
			return nil, err
		}
		out.Write(key)
		out.WriteByte(':')
		value, err := json.Marshal(o.values[index])
		if err != nil {
			return nil, err
		}
		out.Write(value)
	}
	out.WriteByte('}')
	return out.Bytes(), nil
}

// pushText appends text as a JSON string, escaped as the Rust server's `json::push_text` escapes
// one: ", \, newline, carriage return and tab by name, every other control character as \u00xx,
// and everything else, U+2028 and U+2029 included, as itself.
func pushText(out *strings.Builder, text string) {
	out.WriteByte('"')
	for _, character := range text {
		switch {
		case character == '"':
			out.WriteString("\\\"")
		case character == '\\':
			out.WriteString("\\\\")
		case character == '\n':
			out.WriteString("\\n")
		case character == '\r':
			out.WriteString("\\r")
		case character == '\t':
			out.WriteString("\\t")
		case character < 0x20:
			fmt.Fprintf(out, "\\u%04x", character)
		default:
			out.WriteRune(character)
		}
	}
	out.WriteByte('"')
}

// pushJSON appends a value as the Rust server's `push_json` writes one: a value parseJSON read
// exactly as it was read, members in order; any other value as encoding/json writes it, read
// back by parseJSON.
func pushJSON(out *strings.Builder, value any) {
	switch shaped := value.(type) {
	case nil:
		out.WriteString("null")
	case bool:
		out.WriteString(strconv.FormatBool(shaped))
	case json.Number:
		out.WriteString(shaped.String())
	case string:
		pushText(out, shaped)
	case []any:
		out.WriteByte('[')
		for index, item := range shaped {
			if index > 0 {
				out.WriteByte(',')
			}
			pushJSON(out, item)
		}
		out.WriteByte(']')
	case jsonObject:
		out.WriteByte('{')
		for index, member := range shaped {
			if index > 0 {
				out.WriteByte(',')
			}
			pushText(out, member.name)
			out.WriteByte(':')
			pushJSON(out, member.value)
		}
		out.WriteByte('}')
	default:
		encoded, err := json.Marshal(shaped)
		if err != nil {
			out.WriteString("null")
			return
		}
		read, err := parseJSON(string(encoded))
		if err != nil {
			out.WriteString("null")
			return
		}
		pushJSON(out, read)
	}
}

// bare is a value as JSON, as the Rust server writes the body of a bare answer.
func bare(value any) string {
	var out strings.Builder
	pushJSON(&out, value)
	return out.String()
}

// errorObject is {<member>: <wire name>, …the error's own fields}: the error object a binding
// declares, the fields in the order the envelope holds them.
func errorObject(member string, wire string, payload any) string {
	var out strings.Builder
	out.WriteByte('{')
	pushText(&out, member)
	out.WriteByte(':')
	pushText(&out, wire)
	if fields, ok := payload.(jsonObject); ok {
		for _, field := range fields {
			out.WriteByte(',')
			pushText(&out, field.name)
			out.WriteByte(':')
			pushJSON(&out, field.value)
		}
	}
	out.WriteByte('}')
	return out.String()
}

// literalObject is {<member>: <text>}: a standard refusal's literal body.
func literalObject(member string, text string) string {
	var out strings.Builder
	out.WriteByte('{')
	pushText(&out, member)
	out.WriteByte(':')
	pushText(&out, text)
	out.WriteByte('}')
	return out.String()
}

// readEnvelope is the envelope a command's answer renders, as parseJSON reads it, or nil.
func readEnvelope(envelope string) any {
	read, err := parseJSON(envelope)
	if err != nil {
		return nil
	}
	return read
}

// spelledPath is the request's path as its request line spells it: the target up to any ?,
// escapes as they were sent. Routes match it and a 404 names it, as in the Rust server.
func spelledPath(request *http.Request) string {
	if request.RequestURI == "" {
		return request.URL.EscapedPath()
	}
	path, _, _ := strings.Cut(request.RequestURI, "?")
	return path
}

// boundMethodNotAllowed is the 405 a bound surface answers for a path its routes match under
// other methods: every one of them, upper-case, sorted, named in the body and in Allow.
func boundMethodNotAllowed(allowed []string) response {
	sort.Strings(allowed)
	distinct := []string{}
	for _, method := range allowed {
		if len(distinct) == 0 || distinct[len(distinct)-1] != method {
			distinct = append(distinct, method)
		}
	}
	joined := strings.Join(distinct, ", ")
	answer := methodNotAllowed(joined)
	answer.headers = append(answer.headers, [2]string{"Allow", joined})
	return answer
}
"##;

/// The imports [`BOUND`] needs beyond the ones the helpers file always has.
pub(super) const IMPORTS: [&str; 5] = ["net/url", "sort", "strings", "unicode", "unicode/utf8"];

/// The route match of a bound surface: every row of the table, in path order, tried in turn.
///
/// Every row is matched against the path as the request line spells it (`spelledPath`), as the
/// Rust server matches: a row the binding declares is a template whose variables are
/// percent-decoded one segment at a time; a derived row and the two documents are exact paths, so
/// `/openapi%2Ejson` is not `/openapi.json`. A path some row matches under the request's method is
/// answered by that row; one some row matches under another method is `405` naming every method a
/// matching row answers; one no row matches is `404`, naming the path as it was spelled.
///
/// Every text from the model or the binding is written into Go source with [`go_string`].
pub(super) fn dispatch(
    body: &mut String,
    ir: &EssIr,
    layout: &Layout,
    routes: &[http::Route<'_>],
    rows: &[Row],
    system: &str,
    exported: &str,
) {
    let grants = http::checks_grants(ir);
    let caller = if grants { "caller *Caller, " } else { "" };
    let _ = write!(
        body,
        "\n// dispatch{exported} answers one request.\n//\n// The routes are templates an \
         `ess-http/1` binding declares, beside the routes it does not name.\n// A path no route \
         matches is a 404 naming where the whole table is published; a path some\n// route \
         matches under another method is a 405 naming every method its routes answer.\nfunc \
         dispatch{exported}(system *{system}, {caller}request *http.Request) response {{\n\tif \
         refused := tooManyHeaders(request); refused != nil {{\n\t\treturn *refused\n\t}}\n\tbody, \
         refused := readBody(request)\n\tif refused != nil {{\n\t\treturn *refused\n\t}}\n\t// \
         Held from the port call through Pump and TakePublished: the system is shared by\n\t// \
         every connection.\n\tserving.Lock()\n\tdefer serving.Unlock()\n\tspelled := \
         spelledPath(request)\n\tallowed := []string{{}}\n"
    );
    let mut reads_body = false;
    for (method, path, _, _) in rows {
        let route = routes
            .iter()
            .find(|route| &route.path == path && route.method.as_str() == *method);
        let declared = route.filter(|route| route.declared.is_some());
        let variables = declared.is_some_and(|route| !http::path_fields(ir, route).is_empty());
        let path_text = go_string(path);
        let opening = match (declared, variables) {
            (Some(_), true) => format!("\tif values, ok := matched({path_text}, spelled); ok {{\n"),
            (Some(_), false) => format!("\tif _, ok := matched({path_text}, spelled); ok {{\n"),
            (None, _) => format!("\tif spelled == {path_text} {{\n"),
        };
        body.push_str(&opening);
        let method_text = go_string(method);
        let _ = writeln!(body, "\t\tif request.Method == {method_text} {{");
        let call = match route {
            None if path == http::OPENAPI => format!(
                "\t\t\treturn response{{status: 200, contentType: mediaJSON, body: \
                 openapi{exported}}}\n"
            ),
            None => format!(
                "\t\t\treturn response{{status: 200, contentType: mediaMarkdown, body: \
                 docs{exported}}}\n"
            ),
            Some(route) => {
                if matches!(route.serves, Served::Command(_)) {
                    reads_body = true;
                }
                answer_call(ir, layout, route, declared.is_some(), variables)
            }
        };
        body.push_str(&call);
        let _ = writeln!(
            body,
            "\t\t}}\n\t\tallowed = append(allowed, {method_text})\n\t}}"
        );
    }
    if !reads_body {
        body.push_str("\t_ = body\n");
    }
    body.push_str(
        "\tif len(allowed) == 0 {\n\t\treturn refusal(404, fmt.Sprintf(\"`%s` is not a path this \
         surface declares; `GET /openapi.json` publishes every one that is\", spelled))\n\t}\n\t\
         return boundMethodNotAllowed(allowed)\n}\n",
    );
}

/// What one matched row does under its own method: check the grant, then answer.
fn answer_call(
    ir: &EssIr,
    layout: &Layout,
    route: &http::Route<'_>,
    declared: bool,
    variables: bool,
) -> String {
    let values = if variables { "values" } else { "nil" };
    let refuse = |standard: &str| {
        if declared {
            format!("refusalNotGranted({standard})")
        } else {
            standard.to_owned()
        }
    };
    let mut call = String::new();
    match route.serves {
        Served::Command(handle) => {
            let command = ir.command(handle);
            if http::checks_grants(ir) {
                let _ = writeln!(
                    call,
                    "\t\t\tif refused := admit(caller, {}); refused != nil {{\n\t\t\t\treturn \
                     {}\n\t\t\t}}",
                    go_string(&command.name.to_string()),
                    refuse("*refused")
                );
            }
            if declared {
                let _ = writeln!(
                    call,
                    "\t\t\treturn serveBound{}(system, request, body, {values})",
                    ident(layout, &command.name)
                );
            } else {
                let _ = writeln!(
                    call,
                    "\t\t\treturn serve{}(system, body)",
                    ident(layout, &command.name)
                );
            }
        }
        Served::View(handle) => {
            let view = ir.view(handle);
            if http::read_checked(ir, &view.name) {
                let _ = writeln!(
                    call,
                    "\t\t\tif refused := admitRead(caller, {}); refused != nil {{\n\t\t\t\treturn \
                     {}\n\t\t\t}}",
                    go_string(&view.name.to_string()),
                    refuse("*refused")
                );
            }
            if declared {
                let _ = writeln!(
                    call,
                    "\t\t\treturn serveBound{}(system, request, {values})",
                    ident(layout, &view.name)
                );
            } else {
                let _ = writeln!(
                    call,
                    "\t\t\treturn serve{}(system{})",
                    ident(layout, &view.name),
                    if view.params.is_empty() {
                        ""
                    } else {
                        ", request.URL.RawQuery"
                    }
                );
            }
        }
    }
    call
}

/// How one field the path or a flat request carries is read from text, as Go source.
fn flat_kind(ir: &EssIr, field: &ResolvedField) -> &'static str {
    // A compiled binding admits only fields with a flat spelling (`ESS-HTTP-009`, `ESS-HTTP-021`).
    match http::flat(ir, &field.type_ref).map_or(http::Flat::Text, |(_, flat)| flat) {
        http::Flat::String => "flatString",
        http::Flat::Text => "flatText",
        http::Flat::Integer => "flatInteger",
        http::Flat::Boolean => "flatBoolean",
    }
}

/// The statements reading each path variable of `route`, and the `[]pathMember` literal of them,
/// in path order. Each statement returns `(nil, err)` from the closure it is emitted into.
fn path_members(ir: &EssIr, route: &http::Route<'_>) -> (String, String) {
    let mut statements = String::new();
    let mut members = Vec::new();
    for (index, (variable, field)) in http::path_fields(ir, route).iter().enumerate() {
        let _ = writeln!(
            statements,
            "\t\tmember{index}, err := pathValue(path[{index}], {}, {})\n\t\tif err != nil \
             {{\n\t\t\treturn nil, err\n\t\t}}",
            flat_kind(ir, field),
            go_string(&format!("path.{variable}"))
        );
        members.push(format!(
            "{{{}, member{index}}}",
            go_string(ess_gen::schema::wire_field_name(field))
        ));
    }
    (
        statements,
        format!("[]pathMember{{{}}}", members.join(", ")),
    )
}

/// A flat request's `[]flatField` literal: every field the path does not carry, in declaration
/// order.
fn flat_table(ir: &EssIr, fields: &[ResolvedField], route: &http::Route<'_>) -> String {
    let carried = http::carried(ir, route);
    let rows: Vec<String> = fields
        .iter()
        .filter(|field| !carried.contains(field.name.as_str()))
        .map(|field| {
            let shape = match http::flat(ir, &field.type_ref).map(|(shape, _)| shape) {
                Some(http::Shape::Optional) => "shapeOptional",
                Some(http::Shape::List) => "shapeList",
                Some(http::Shape::One) | None => "shapeOne",
            };
            format!(
                "{{{}, {}, {shape}}}",
                go_string(ess_gen::schema::wire_field_name(field)),
                flat_kind(ir, field)
            )
        })
        .collect();
    format!("[]flatField{{{}}}", rows.join(", "))
}

/// The wire names of the fields the path carries, as a `[]string` literal.
fn carried_table(ir: &EssIr, route: &http::Route<'_>) -> String {
    let wires: Vec<String> = http::path_fields(ir, route)
        .iter()
        .map(|(_, field)| go_string(ess_gen::schema::wire_field_name(field)))
        .collect();
    format!("[]string{{{}}}", wires.join(", "))
}

/// One bound route's handler: read the request the binding declares, run the port, answer what
/// the binding declares.
pub(super) fn handler(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    component: &ResolvedComponent,
    route: &http::Route<'_>,
    system: &str,
    records_invocations: bool,
) {
    match route.serves {
        Served::Command(handle) => command(
            out,
            ir,
            layout,
            component,
            route,
            ir.command(handle),
            system,
            records_invocations,
        ),
        Served::View(handle) => view(out, ir, layout, component, route, ir.view(handle), system),
    }
}

/// A bound command route.
#[allow(clippy::too_many_arguments)]
fn command(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    component: &ResolvedComponent,
    route: &http::Route<'_>,
    command: &ResolvedCommand,
    system: &str,
    records_invocations: bool,
) {
    let function = ident(layout, &command.name);
    let field = name::exported(&component.name.to_string());
    let method = layout.declared(&command.name);
    let request = http::request(route);
    let ignore = request.unknown == ess_http::Unknown::Ignore;
    let read = match request.encoding {
        Encoding::Json => format!(
            "jsonBody(body, {})",
            http::body_required_without(command, &http::carried(ir, route))
        ),
        Encoding::Form => format!(
            "formObject(request, body, {}, {}, {ignore})",
            flat_table(ir, &command.input, route),
            carried_table(ir, route)
        ),
        Encoding::Query => format!(
            "flatObject(request.URL.RawQuery, {}, {}, {ignore}, \"query\")",
            flat_table(ir, &command.input, route),
            carried_table(ir, route)
        ),
    };
    let at = go_string(if request.encoding == Encoding::Query {
        "query"
    } else {
        "body"
    });
    let (statements, members) = path_members(ir, route);
    let declares = http::declares_answers(route);
    let answer = if declares {
        format!("answerBound{function}(answer{function}(outcome), value)")
    } else {
        format!("answer{function}(outcome)")
    };
    let _ = write!(
        out,
        "\n// serveBound{function} answers `{}` `{}`: `{}`, read and answered as its `ess-http/1`\n\
         // binding declares.\nfunc serveBound{function}(system *{system}, request *http.Request, \
         body []byte, path []string) response {{\n\tvalue, err := func() (any, error) \
         {{\n\t\tvalue, err := {read}\n\t\tif err != nil {{\n\t\t\treturn nil, \
         err\n\t\t}}\n{statements}\t\treturn withPath(value, {members}, {at})\n\t}}()\n\tif err \
         != nil {{\n\t\treturn refusalMalformed(err.Error())\n\t}}\n\tinput, err := \
         decodeCommand{function}(value, \"body\")\n\tif err != nil {{\n\t\treturn \
         refusalMalformed(err.Error())\n\t}}\n\toutcome, unmet := \
         system.{field}.{method}(input)\n\tif unmet != nil {{\n\t\treturn \
         refusalUnfinished(unmet.Error(), false)\n\t}}\n\tfailure := \
         system.Pump()\n\tsystem.TakePublished()\n{invocations}\tif failure != nil {{\n\t\treturn \
         refusalUnfinished(\"delivering what the command published: \"+failure.Error(), \
         true)\n\t}}\n\treturn {answer}\n}}\n",
        route.method.as_str(),
        route.path,
        command.name,
        invocations = if records_invocations {
            "\tsystem.TakeInvocations()\n"
        } else {
            ""
        },
    );
    if declares {
        outcome_answer(out, ir, route, command, function);
    }
}

/// `answerBound*`: the declared answer of each outcome the route lists, read off the envelope the
/// shared rendering wrote, and that envelope at its derived status for every other outcome.
fn outcome_answer(
    out: &mut String,
    ir: &EssIr,
    route: &http::Route<'_>,
    command: &ResolvedCommand,
    function: &str,
) {
    let listed: Vec<(&ess_compiler::ir::ResolvedOutcome, &ess_http::Outcome)> = command
        .outcomes
        .iter()
        .filter_map(|outcome| http::listed(route, outcome).map(|declared| (outcome, declared)))
        .collect();
    let sources: Vec<&AnswerSource> = listed
        .iter()
        .flat_map(|(_, declared)| answer_sources(declared))
        .collect();
    let reads_input = sources
        .iter()
        .any(|source| matches!(source, AnswerSource::Input(_)));
    let reads_response = sources
        .iter()
        .any(|source| matches!(source, AnswerSource::Response(_)))
        || listed
            .iter()
            .any(|(_, declared)| matches!(declared.body, Body::Response | Body::ResponseField(_)));
    let reads_payload = sources
        .iter()
        .any(|source| matches!(source, AnswerSource::Error(_)))
        || listed
            .iter()
            .any(|(_, declared)| matches!(declared.body, Body::Error { .. }));
    let _ = write!(
        out,
        "\n// answerBound{function} is the answer `{}` gives as its binding declares it: each \
         listed\n// outcome's status, body, headers and redirect, and the envelope at its derived \
         status for\n// every other outcome.\nfunc answerBound{function}(answer response, input \
         any) response {{\n\tdeclared, err := declared{function}(answer.body, input)\n\tif err != \
         nil {{\n\t\t// The effect stands; only its answer could not be written.\n\t\treturn \
         refusalUnfinished(err.Error(), true)\n\t}}\n\tif declared != nil {{\n\t\treturn \
         *declared\n\t}}\n\treturn answer\n}}\n\n// declared{function} is each listed outcome of \
         `{}`, answered as declared; nil for an outcome\n// the binding does not list.\nfunc \
         declared{function}(envelope string, input any) (*response, error) {{\n\tread := \
         readEnvelope(envelope)\n",
        command.name, command.name,
    );
    if !reads_input {
        out.push_str("\t_ = input\n");
    }
    if reads_response {
        out.push_str("\treply := memberOf(read, \"response\")\n");
    }
    if reads_payload {
        out.push_str("\tpayload := memberOf(read, \"payload\")\n");
    }
    out.push_str("\toutcome, _ := memberOf(read, \"outcome\").(string)\n\tswitch outcome {\n");
    for (outcome, declared) in &listed {
        let _ = write!(
            out,
            "\tcase {}:\n{}\t\treturn &answer, nil\n",
            go_string(outcome.name.as_str()),
            outcome_arm(ir, command, outcome, declared)
        );
    }
    out.push_str("\t}\n\treturn nil, nil\n}\n");
}

/// One listed outcome's arm of `declared*`: its body, its headers and its redirect, as Go
/// statements building `answer`.
fn outcome_arm(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ess_compiler::ir::ResolvedOutcome,
    declared: &ess_http::Outcome,
) -> String {
    let error = outcome.error.as_ref().map(|handle| ir.error(handle));
    let wire_name = error.map(|error| error.naming.wire_or(&error.name).to_owned());
    let status = declared.status;
    let body = match &declared.body {
        Body::Envelope => {
            format!("response{{status: {status}, contentType: mediaJSON, body: envelope}}")
        }
        Body::Response => {
            format!("response{{status: {status}, contentType: mediaJSON, body: bare(reply)}}")
        }
        Body::ResponseField(field) => {
            let wire = command
                .response
                .iter()
                .find(|declared| declared.name == *field)
                .map_or(field.as_str(), ess_gen::schema::wire_field_name);
            format!(
                "response{{status: {status}, contentType: mediaJSON, body: \
                 bare(memberOf(reply, {}))}}",
                go_string(wire)
            )
        }
        Body::None => format!("response{{status: {status}}}"),
        Body::Error { member } => format!(
            "response{{status: {status}, contentType: mediaJSON, body: errorObject({}, {}, \
             payload)}}",
            go_string(member),
            go_string(wire_name.as_deref().unwrap_or_default())
        ),
    };
    let mut arm = format!("\t\tanswer := {body}\n");
    let text = |source: &AnswerSource| source_text(command, error, wire_name.as_deref(), source);
    for (name, header) in &declared.headers {
        let name = go_string(name);
        match header {
            Header::Literal(literal) => {
                let _ = writeln!(
                    arm,
                    "\t\tanswer.headers = append(answer.headers, [2]string{{{name}, {}}})",
                    go_string(literal)
                );
            }
            Header::Source(source) => {
                let _ = writeln!(
                    arm,
                    "\t\tif err := pushHeader(&answer, {name}, {}); err != nil {{\n\t\t\treturn \
                     nil, err\n\t\t}}",
                    text(source)
                );
            }
            Header::Challenge(challenge) => {
                let params: Vec<String> = challenge
                    .params
                    .iter()
                    .map(|(param, value)| match value {
                        Param::Literal(literal) => {
                            format!("{{{}, some({})}}", go_string(param), go_string(literal))
                        }
                        Param::Source(source) => {
                            format!("{{{}, {}}}", go_string(param), text(source))
                        }
                    })
                    .collect();
                let _ = writeln!(
                    arm,
                    "\t\t{{\n\t\t\tvalue, err := challenge({}, []namedText{{{}}})\n\t\t\tif err \
                     != nil {{\n\t\t\t\treturn nil, err\n\t\t\t}}\n\t\t\tanswer.headers = \
                     append(answer.headers, [2]string{{{name}, value}})\n\t\t}}",
                    go_string(&challenge.scheme),
                    params.join(", ")
                );
            }
        }
    }
    if let Some(redirect) = &declared.redirect {
        let parameters: Vec<String> = redirect
            .parameters
            .iter()
            .map(|(name, source)| format!("{{{}, {}}}", go_string(name), text(source)))
            .collect();
        let _ = writeln!(
            arm,
            "\t\tlocation, err := redirect({}, {}, []namedText{{{}}})\n\t\tif err != nil \
             {{\n\t\t\treturn nil, err\n\t\t}}\n\t\tanswer.headers = append(answer.headers, \
             [2]string{{\"Location\", location}})",
            text(&redirect.location),
            redirect.mode == RedirectMode::Fragment,
            parameters.join(", ")
        );
    }
    arm
}

/// Every source one listed outcome reads: its headers', its challenge params' and its redirect's.
fn answer_sources(declared: &ess_http::Outcome) -> Vec<&AnswerSource> {
    let mut sources = Vec::new();
    for header in declared.headers.values() {
        match header {
            Header::Literal(_) => {}
            Header::Source(source) => sources.push(source),
            Header::Challenge(challenge) => {
                for param in challenge.params.values() {
                    if let Param::Source(source) = param {
                        sources.push(source);
                    }
                }
            }
        }
    }
    if let Some(redirect) = &declared.redirect {
        sources.push(&redirect.location);
        sources.extend(redirect.parameters.values());
    }
    sources
}

/// One answer source as a `maybeText` Go expression over `input`, `reply` and `payload`, each
/// field by its wire name.
fn source_text(
    command: &ResolvedCommand,
    error: Option<&ess_compiler::ir::ResolvedError>,
    wire_name: Option<&str>,
    source: &AnswerSource,
) -> String {
    let wire_of = |fields: &[ResolvedField], name: &str| {
        fields.iter().find(|field| field.name == name).map_or_else(
            || name.to_owned(),
            |field| ess_gen::schema::wire_field_name(field).to_owned(),
        )
    };
    match source {
        AnswerSource::Input(field) => format!(
            "scalarText(memberOf(input, {}))",
            go_string(&wire_of(&command.input, field))
        ),
        AnswerSource::Response(field) => format!(
            "scalarText(memberOf(reply, {}))",
            go_string(&wire_of(&command.response, field))
        ),
        AnswerSource::Error(field) => format!(
            "scalarText(memberOf(payload, {}))",
            go_string(&wire_of(
                error.map_or(&[][..], |error| &error.fields[..]),
                field
            ))
        ),
        AnswerSource::ErrorWire => format!("some({})", go_string(wire_name.unwrap_or_default())),
    }
}

/// A bound view route.
fn view(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    component: &ResolvedComponent,
    route: &http::Route<'_>,
    view: &ResolvedView,
    system: &str,
) {
    let function = ident(layout, &view.name);
    let field = name::exported(&component.name.to_string());
    let method = layout.declared(&view.name);
    let _ = write!(
        out,
        "\n// serveBound{function} answers `{}` `{}`: `{}`, read and answered as its `ess-http/1`\n\
         // binding declares.\nfunc serveBound{function}(system *{system}, request *http.Request, \
         path []string) response {{\n",
        route.method.as_str(),
        route.path,
        view.name,
    );
    let arguments = if view.params.is_empty() {
        out.push_str("\t_, _ = request, path\n");
        String::new()
    } else {
        let carried = http::carried(ir, route);
        let unmapped: Vec<&ResolvedField> = view
            .params
            .iter()
            .filter(|param| !carried.contains(param.name.as_str()))
            .collect();
        let (statements, members) = path_members(ir, route);
        let _ = write!(
            out,
            "\tvalue, err := func() (any, error) {{\n\t\tvalue, err := \
             queryObject(request.URL.RawQuery, {}, \"query\")\n\t\tif err != nil {{\n\t\t\treturn \
             nil, err\n\t\t}}\n{statements}\t\treturn withPath(value, {members}, \
             \"query\")\n\t}}()\n\tif err != nil {{\n\t\treturn \
             refusalMalformed(err.Error())\n\t}}\n\tparams, err := \
             decodeParams{function}(value, \"query\")\n\tif err != nil {{\n\t\treturn \
             refusalMalformed(err.Error())\n\t}}\n",
            super::query_params(ir, &unmapped),
        );
        view.params
            .iter()
            .map(|param| {
                format!(
                    "params.{}",
                    super::super::items::member_ident(&view.params, &param.name)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let _ = write!(
        out,
        "\trows, unmet := system.{field}.{method}({arguments})\n\tif unmet != nil {{\n\t\treturn \
         refusalUnfinished(unmet.Error(), false)\n\t}}\n\tencoded := make([]any, 0, \
         len(rows))\n\tfor _, row := range rows {{\n\t\tencoded = append(encoded, \
         encodeView{function}(row))\n\t}}\n"
    );
    let single = route
        .declared
        .filter(|declared| declared.body == ess_http::ViewBody::Single)
        .and_then(|declared| declared.empty);
    match single {
        Some(empty) => {
            let _ = write!(
                out,
                "\tif len(encoded) == 0 {{\n\t\treturn response{{status: {}}}\n\t}}\n\treturn \
                 response{{status: 200, contentType: mediaJSON, body: bare(encoded[0])}}\n}}\n",
                empty.status
            );
        }
        None => out.push_str("\treturn rendered(200, map[string]any{\"rows\": encoded})\n}\n"),
    }
}

/// The standard refusals of a bound package's declared routes, as its binding answers them
/// (`refusals:`), and today's answer for each the binding does not override. `refusalNotGranted`
/// only where some declared route checks a grant, so nothing emitted goes unused.
pub(super) fn refusals(binding: &http::Binding, checks_grant: bool) -> String {
    let mut out = String::from(
        "\n// refusalMalformed is a request a bound route cannot read as its binding declares: \
         `refusals.malformed`,\n// else 400.\n",
    );
    match binding.refusal(RefusalKey::Malformed) {
        Some(refusal) => {
            let _ = write!(
                out,
                "func refusalMalformed(_ string) response {{\n{}}}\n",
                overridden(refusal)
            );
        }
        None => out.push_str(
            "func refusalMalformed(detail string) response {\n\treturn refusal(400, detail)\n}\n",
        ),
    }
    out.push_str(
        "\n// refusalUnfinished is an unfinished realization on a bound route: \
         `refusals.unfinished`, else 501\n// with committed.\n",
    );
    match binding.refusal(RefusalKey::Unfinished) {
        Some(refusal) => {
            let _ = write!(
                out,
                "func refusalUnfinished(_ string, _ bool) response {{\n{}}}\n",
                overridden(refusal)
            );
        }
        None => out.push_str(
            "func refusalUnfinished(detail string, committed bool) response {\n\treturn \
             unfinished(detail, committed)\n}\n",
        ),
    }
    if checks_grant {
        out.push_str(
            "\n// refusalNotGranted is a caller no grant admits, on a bound route: \
             `refusals.not_granted`, else the\n// standard 403.\n",
        );
        match binding.refusal(RefusalKey::NotGranted) {
            Some(refusal) => {
                let _ = write!(
                    out,
                    "func refusalNotGranted(_ response) response {{\n{}}}\n",
                    overridden(refusal)
                );
            }
            None => out.push_str(
                "func refusalNotGranted(standard response) response {\n\treturn standard\n}\n",
            ),
        }
    }
    out
}

/// The body of a function answering one overridden standard refusal: its status, its literal
/// body or none, and its literal headers, every one checked when the binding compiled.
fn overridden(refusal: &ess_http::Refusal) -> String {
    let body = match &refusal.body {
        RefusalBody::None => format!("response{{status: {}}}", refusal.status),
        RefusalBody::Literal { member, text } => format!(
            "response{{status: {}, contentType: mediaJSON, body: literalObject({}, {})}}",
            refusal.status,
            go_string(member),
            go_string(text)
        ),
    };
    if refusal.headers.is_empty() {
        return format!("\treturn {body}\n");
    }
    let mut out = format!("\tanswer := {body}\n");
    for (name, header) in &refusal.headers {
        let value = match header {
            Header::Literal(text) => text.clone(),
            Header::Challenge(challenge) => challenge_text(challenge),
            // A refusal's header never reads a source (`ESS-HTTP-026`).
            Header::Source(_) => continue,
        };
        let _ = writeln!(
            out,
            "\tanswer.headers = append(answer.headers, [2]string{{{}, {}}})",
            go_string(name),
            go_string(&value)
        );
    }
    out.push_str("\treturn answer\n");
    out
}

/// A challenge whose params are all literals, written as the emitted `challenge` writes one.
fn challenge_text(challenge: &ess_http::Challenge) -> String {
    let mut out = challenge.scheme.clone();
    let mut first = true;
    for (name, param) in &challenge.params {
        let Param::Literal(text) = param else {
            continue;
        };
        out.push_str(if first { " " } else { ", " });
        first = false;
        let _ = write!(out, "{name}=\"");
        for c in text.chars() {
            if c == '"' || c == '\\' {
                out.push('\\');
            }
            out.push(c);
        }
        out.push('"');
    }
    out
}
