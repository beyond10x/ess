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

use ess_compiler::ir::{
    EssIr, ResolvedBody, ResolvedCommand, ResolvedComponent, ResolvedField, ResolvedTypeRef,
    ResolvedView,
};
use ess_gen::ess_http::{
    self, AnswerSource, Body, Encoding, Header, Param, RedirectMode, RefusalBody, RefusalKey,
    ResponseEncoding,
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

/// Whether the binding bounds the request of some declared route of `routes`: the package then
/// reads every body through [`capped_read_body`] and its surface's `bodyLimit`. Otherwise no byte
/// of the package changes. Decided once per site, never re-derived per call.
pub(super) fn caps_a_request(binding: &http::Binding, routes: &[http::Route<'_>]) -> bool {
    routes.iter().any(|route| {
        route
            .declared
            .is_some_and(|declared| binding.ir().bounds_request(declared))
    })
}

/// How the requests of one served component are read: whether this component caps a request, and
/// whether any component of its package does (the package's `readBody` then takes the bound).
#[derive(Clone, Copy)]
pub(super) struct Reading {
    pub(super) capped: bool,
    pub(super) package_capped: bool,
}

/// `readBody` of `base` as a capped package has it: the bound comes from the route the request
/// matches (`bodyLimit`), and a body announced longer than that bound is refused before one byte
/// of it is read.
///
/// # Panics
///
/// When `base` no longer holds the `readBody` this rewrites: the two cannot drift apart.
pub(super) fn capped_read_body(base: &str) -> String {
    let from = "// readBody reads at most maxBody bytes of a request, or the refusal that says why it could not.\nfunc readBody(request *http.Request) ([]byte, *response) {\n\tif request.Body == nil {\n\t\treturn nil, nil\n\t}\n\tdefer func() { _ = request.Body.Close() }()\n\tbody, err := io.ReadAll(io.LimitReader(request.Body, maxBody+1))\n\tif err != nil {\n\t\tanswer := refusal(400, fmt.Sprintf(\"the body could not be read: %s\", err))\n\t\treturn nil, &answer\n\t}\n\tif len(body) > maxBody {\n\t\tanswer := refusal(413, fmt.Sprintf(\"the body is longer than %d bytes, which is all this surface reads\", maxBody))\n\t\treturn nil, &answer\n\t}\n\treturn body, nil\n}\n";
    let to = "// readBody reads at most reading.limit bytes of a request, or the refusal that says why it could\n// not. A body announced longer is refused before any of it is read: nothing waits for bytes the\n// caller has not sent, and the body is never closed on that path. A body of unannounced length\n// (ContentLength -1) is read up to the limit plus one byte and refused if it goes past.\nfunc readBody(request *http.Request, reading bodyCap) ([]byte, *response) {\n\tif request.Body == nil {\n\t\treturn nil, nil\n\t}\n\tif request.ContentLength > int64(reading.limit) {\n\t\tanswer := reading.tooLarge(reading.limit)\n\t\treturn nil, &answer\n\t}\n\tdefer func() { _ = request.Body.Close() }()\n\tbody, err := io.ReadAll(io.LimitReader(request.Body, int64(reading.limit)+1))\n\tif err != nil {\n\t\tanswer := refusal(400, fmt.Sprintf(\"the body could not be read: %s\", err))\n\t\treturn nil, &answer\n\t}\n\tif len(body) > reading.limit {\n\t\tanswer := reading.tooLarge(reading.limit)\n\t\treturn nil, &answer\n\t}\n\treturn body, nil\n}\n";
    assert!(
        base.contains(from),
        "the surface helpers no longer hold the `readBody` a capped package rewrites: {from}"
    );
    base.replacen(from, to, 1)
}

/// What a capped package adds after [`BOUND`]: the most body bytes a request reads and the answer
/// to a larger one, per matched route (`bodyLimit`, in the surface).
pub(super) const CAPPED: &str = r#"
// ---- a request bound, declared by the binding ---------------------------------------------------

// bodyCap is how many body bytes the route a request matches reads, and the answer to a larger one.
type bodyCap struct {
	limit    int
	tooLarge func(limit int) response
}

// bodyTooLarge is the answer to a body past the fixed bound, where the binding bounds nothing for
// the route: a 413 naming the bound.
func bodyTooLarge(limit int) response {
	return refusal(413, fmt.Sprintf("the body is longer than %d bytes, which is all this surface reads", limit))
}
"#;

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
		} else if segment != literalSpelling(given[index]) {
			return nil, false
		}
	}
	return values, true
}

// literalSpelling is text with each %XX escape of an unreserved character decoded (RFC 3986
// §6.2.2.2), the spelling a segment is compared with a literal in: %7Earchived is ~archived.
// Every other escape stays as written, and a variable's text is decoded where it is read.
func literalSpelling(text string) string {
	if !strings.Contains(text, "%") {
		return text
	}
	var out strings.Builder
	for index := 0; index < len(text); index++ {
		if text[index] == '%' && index+2 < len(text) {
			high, highOk := hexValue(text[index+1])
			low, lowOk := hexValue(text[index+2])
			if decoded := high*16 + low; highOk && lowOk && unreservedByte(decoded) {
				out.WriteByte(decoded)
				index += 2
				continue
			}
		}
		out.WriteByte(text[index])
	}
	return out.String()
}

// hexValue is the value of one hexadecimal digit.
func hexValue(digit byte) (byte, bool) {
	switch {
	case '0' <= digit && digit <= '9':
		return digit - '0', true
	case 'a' <= digit && digit <= 'f':
		return digit - 'a' + 10, true
	case 'A' <= digit && digit <= 'F':
		return digit - 'A' + 10, true
	}
	return 0, false
}

// unreservedByte is whether a byte is an RFC 3986 unreserved character.
func unreservedByte(value byte) bool {
	return 'a' <= value && value <= 'z' || 'A' <= value && value <= 'Z' ||
		'0' <= value && value <= '9' || value == '-' || value == '.' || value == '_' || value == '~'
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
// input {}. A JSON body stays closed: a member none of fields names is malformed, the first in
// the order the members arrived, with the status the Rust server answers. The detail is not
// always the Rust decoder's: a member sent under a field's model name rather than its wire name
// is refused there for that, and here as a member no field names. A member the path carries is a
// field, and withPath refuses it.
func jsonBody(body []byte, required bool, fields []string) (any, error) {
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
	if object, ok := value.(jsonObject); ok {
		for _, member := range object {
			named := false
			for _, field := range fields {
				named = named || field == member.name
			}
			if !named {
				return nil, DecodeError{At: nested("body", member.name), Expected: "no member: no field is named so", Found: describes(memberValue(member.value))}
			}
		}
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

/// The readers of a route with `unknown: ignore` (beyond10x/ess#752), which follow [`BOUND`] only
/// in a surface whose binding declares one, so every other bound surface keeps its bytes. They
/// answer as the Rust server's `http::ignoring` and `http::echoed_keys` do.
pub(super) const IGNORING: &str = r#"
// openJSONBody is a `json` body as jsonBody reads it, no member refused: ignoring decides which
// members are read.
func openJSONBody(body []byte, required bool) (any, error) {
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

// ignoring is a `json` request read under `unknown: ignore`: value less every member no field of
// fields names, at its top level only (a struct inside keeps its own closedness), and less every
// member the path carries that holds the path's value. A member the path carries that holds
// another value is malformed. A value that is not an object is the decoder's to refuse.
func ignoring(value any, fields []string, path []pathMember, at string) (any, error) {
	object, ok := value.(map[string]any)
	if !ok {
		return value, nil
	}
	for _, carried := range path {
		if member, present := object[carried.wire]; present && !sameValue(member, carried.value) {
			return nil, echoed(at, carried.wire)
		}
	}
	kept := map[string]any{}
	for _, field := range fields {
		if member, present := object[field]; present {
			kept[field] = member
		}
	}
	return kept, nil
}

// echoedKeys holds each key of a `form` body or a query string read under `unknown: ignore` that
// the path carries to the path's value, read as its field reads it: a key whose value does not
// decode or is not the path's is malformed.
func echoedKeys(text string, carried []flatField, path []pathMember, at string) error {
	for _, pair := range strings.Split(text, "&") {
		if pair == "" {
			continue
		}
		rawKey, rawValue, _ := strings.Cut(pair, "=")
		key, ok := formDecoded(rawKey)
		if !ok {
			continue
		}
		for index, field := range carried {
			if field.wire != key {
				continue
			}
			place := nested(at, key)
			decoded, ok := formDecoded(rawValue)
			if !ok {
				return fmt.Errorf("%s: expected percent-encoded UTF-8 text, found `%s`", place, rawValue)
			}
			read, err := flatValue(decoded, field.kind, place)
			if err != nil {
				return err
			}
			if !sameValue(read, path[index].value) {
				return echoed(at, key)
			}
		}
	}
	return nil
}

// echoed is the refusal of a member or key the path carries with another value.
func echoed(at string, wire string) error {
	return fmt.Errorf("%s: the path carries this field with another value", nested(at, wire))
}

// sameValue is whether a request's member holds what the path's value does: the same text or
// boolean, or two numbers that denote one integer.
func sameValue(member any, carried any) bool {
	switch held := carried.(type) {
	case string:
		text, ok := member.(string)
		return ok && text == held
	case bool:
		value, ok := member.(bool)
		return ok && value == held
	case json.Number:
		number, ok := member.(json.Number)
		if !ok {
			return false
		}
		if number == held {
			return true
		}
		one, err := integerAt(number, "", "")
		other, otherErr := integerAt(held, "", "")
		return err == nil && otherErr == nil && one == other
	}
	return false
}
"#;

/// Whether a route of `binding` declares `unknown: ignore` beside a `json` body or a path variable,
/// which the surface reads with [`IGNORING`].
pub(super) fn ignores_unknown(binding: &http::Binding) -> bool {
    binding.ir().routes().iter().any(|route| {
        route.request.as_ref().is_some_and(|request| {
            request.unknown == ess_http::Unknown::Ignore
                && (request.encoding == ess_http::Encoding::Json || !route.parameters.is_empty())
        })
    })
}

/// The reader of a `multipart` request (beyond10x/ess#754), which follows [`BOUND`] only in a
/// surface whose binding has a `multipart` route, so every other bound surface keeps its bytes. It
/// reads as the Rust server's `http::multipart_object` does, with the same refusals.
pub(super) const MULTIPART: &str = r#"
// ---- a `multipart/form-data` request (RFC 7578), served -----------------------------------------

// mediaMultipart is the media type of a `multipart` request body.
const mediaMultipart = "multipart/form-data"

// partAttribute is the input field a file part's filename or media type is read into.
type partAttribute struct {
	wire     string
	required bool
}

// partField is one field a `multipart` request carries, by its wire name: a text part, read as a
// `form` value of kind is, or a file part, whose bytes are read as base64 text, its filename and
// media type into the input fields filename and mediaType name, of one of mediaTypes or of any
// where that is empty.
type partField struct {
	wire       string
	kind       flatKind
	shape      flatShape
	file       bool
	filename   *partAttribute
	mediaType  *partAttribute
	mediaTypes []string
}

// sentPart is one part of a `multipart/form-data` body, as sent: its `Content-Disposition` name
// and filename, if it has one, its media type without parameters (`text/plain` where it states
// none, RFC 7578 §4.4) and its content.
type sentPart struct {
	name      string
	filename  *string
	mediaType string
	content   []byte
}

// sameASCII is whether two texts are equal but for ASCII case.
func sameASCII(one string, other string) bool {
	if len(one) != len(other) {
		return false
	}
	for index := 0; index < len(one); index++ {
		left, right := one[index], other[index]
		if 'A' <= left && left <= 'Z' {
			left += 'a' - 'A'
		}
		if 'A' <= right && right <= 'Z' {
			right += 'a' - 'A'
		}
		if left != right {
			return false
		}
	}
	return true
}

// unquoted is a header parameter's value: a quoted string unquoted, its \ escapes undone, or the
// token.
func unquoted(text string) string {
	if len(text) < 2 || !strings.HasPrefix(text, "\"") || !strings.HasSuffix(text, "\"") {
		return text
	}
	var out strings.Builder
	escaped := false
	for _, character := range text[1 : len(text)-1] {
		if escaped || character != '\\' {
			out.WriteRune(character)
			escaped = false
		} else {
			escaped = true
		}
	}
	return out.String()
}

// parameters is a header value's first item and its name=value parameters, each name in lower
// case, split at each ; outside a quoted string.
func parameters(value string) (string, [][2]string) {
	items := []string{}
	var item strings.Builder
	quoted, escaped := false, false
	for _, character := range value {
		switch {
		case escaped:
			escaped = false
		case character == '\\' && quoted:
			escaped = true
		case character == '"':
			quoted = !quoted
		case character == ';' && !quoted:
			items = append(items, item.String())
			item.Reset()
			continue
		}
		item.WriteRune(character)
	}
	items = append(items, item.String())
	pairs := [][2]string{}
	for _, one := range items[1:] {
		name, text, ok := strings.Cut(one, "=")
		if ok {
			pairs = append(pairs, [2]string{strings.ToLower(strings.TrimSpace(name)), unquoted(strings.TrimSpace(text))})
		}
	}
	return strings.TrimSpace(items[0]), pairs
}

// multipartBoundary is the boundary of a `multipart/form-data` request, or a refusal where its
// Content-Type is not `multipart/form-data` or states no boundary of 1 to 70 characters.
func multipartBoundary(request *http.Request) (string, error) {
	essence, pairs := parameters(request.Header.Get("Content-Type"))
	if !sameASCII(essence, mediaMultipart) {
		return "", fmt.Errorf("the body is not `multipart/form-data`")
	}
	for _, pair := range pairs {
		if pair[0] == "boundary" && len(pair[1]) >= 1 && len(pair[1]) <= 70 {
			return pair[1], nil
		}
	}
	return "", fmt.Errorf("the body is `multipart/form-data` without a `boundary`")
}

// sentPartOf is one part read from its header section and content.
func sentPartOf(head []byte, content []byte) (sentPart, error) {
	if !utf8.Valid(head) {
		return sentPart{}, fmt.Errorf("a part's headers are not UTF-8")
	}
	var name, filename *string
	mediaType := "text/plain"
	for _, line := range strings.Split(string(head), "\r\n") {
		if line == "" {
			continue
		}
		key, value, ok := strings.Cut(line, ":")
		if !ok {
			return sentPart{}, fmt.Errorf("a part's header is not `name: value`: `%s`", line)
		}
		key = strings.TrimSpace(key)
		if sameASCII(key, "content-disposition") {
			kind, pairs := parameters(value)
			if !sameASCII(kind, "form-data") {
				return sentPart{}, fmt.Errorf("a part's disposition is `%s`, not `form-data`", kind)
			}
			for _, pair := range pairs {
				text := pair[1]
				switch pair[0] {
				case "name":
					name = &text
				case "filename":
					filename = &text
				}
			}
		} else if sameASCII(key, "content-type") {
			mediaType, _ = parameters(value)
		}
	}
	if name == nil {
		return sentPart{}, fmt.Errorf("a part has no `Content-Disposition: form-data` name")
	}
	return sentPart{name: *name, filename: filename, mediaType: mediaType, content: content}, nil
}

// multipartParts is the parts of a `multipart/form-data` body, in the order sent (RFC 2046
// §5.1.1): what precedes the first boundary and follows the closing one is ignored. A request
// that is not `multipart/form-data` with a boundary, holds no boundary, is not closed by its final
// boundary, or holds a part without a `form-data` name is malformed.
func multipartParts(request *http.Request, body []byte) ([]sentPart, error) {
	boundary, err := multipartBoundary(request)
	if err != nil {
		return nil, err
	}
	delimiter := []byte("--" + boundary)
	closing := append([]byte("\r\n"), delimiter...)
	unclosed := fmt.Errorf("the body is not closed by its final boundary")
	at := 0
	if bytes.HasPrefix(body, delimiter) {
		at = len(delimiter)
	} else {
		start := bytes.Index(body, closing)
		if start < 0 {
			return nil, fmt.Errorf("the body holds no `multipart` boundary")
		}
		at = start + len(closing)
	}
	parts := []sentPart{}
	for {
		if bytes.HasPrefix(body[at:], []byte("--")) {
			return parts, nil
		}
		for at < len(body) && (body[at] == ' ' || body[at] == '\t') {
			at++
		}
		if !bytes.HasPrefix(body[at:], []byte("\r\n")) {
			return nil, unclosed
		}
		at += 2
		head, start := body[at:at], at+2
		if !bytes.HasPrefix(body[at:], []byte("\r\n")) {
			end := bytes.Index(body[at:], []byte("\r\n\r\n"))
			if end < 0 {
				return nil, unclosed
			}
			head, start = body[at:at+end], at+end+4
		}
		end := bytes.Index(body[start:], closing)
		if end < 0 {
			return nil, unclosed
		}
		part, err := sentPartOf(head, body[start:start+end])
		if err != nil {
			return nil, err
		}
		parts = append(parts, part)
		at = start + end + len(closing)
	}
}

// multipartObject is a `multipart` body as the object the input's decoder reads: one member per
// declared field a part names, keyed by its wire name, a list as an array, empty when no part
// names it, and a file part's filename and media type under the wire names it reads them into. A
// part the path carries, or no field declares, is unknown: ignored where ignore, and malformed
// otherwise. A part that is not a list's given twice, a text part that is not UTF-8 or is empty
// for a field that is not a `String`, a file part of a media type it may not carry and one
// without the filename a required input field reads are malformed.
func multipartObject(request *http.Request, body []byte, fields []partField, carried []string, ignore bool, at string) (map[string]any, error) {
	parts, err := multipartParts(request, body)
	if err != nil {
		return nil, err
	}
	found := make([][]sentPart, len(fields))
	for _, part := range parts {
		place := nested(at, part.name)
		index := -1
		for position, field := range fields {
			if field.wire == part.name {
				index = position
				break
			}
		}
		pathCarries := false
		for _, wire := range carried {
			if wire == part.name {
				pathCarries = true
			}
		}
		if index < 0 || pathCarries {
			if ignore {
				continue
			}
			if pathCarries {
				return nil, fmt.Errorf("%s: the path carries this field, so it is not a part here", place)
			}
			return nil, fmt.Errorf("%s: no field of this request is named so", place)
		}
		found[index] = append(found[index], part)
	}
	object := map[string]any{}
	for index, field := range fields {
		place := nested(at, field.wire)
		if field.shape != shapeList && len(found[index]) > 1 {
			return nil, fmt.Errorf("%s: expected one part, found %d", place, len(found[index]))
		}
		values := []any{}
		for _, part := range found[index] {
			value, err := partValue(part, field, place, object)
			if err != nil {
				return nil, err
			}
			values = append(values, value)
		}
		if field.shape == shapeList {
			object[field.wire] = values
		} else if len(values) == 1 {
			object[field.wire] = values[0]
		}
	}
	return object, nil
}

// partValue is one part's value, a file part's filename and media type set on object.
func partValue(part sentPart, field partField, place string, object map[string]any) (any, error) {
	if !field.file {
		if !utf8.Valid(part.content) {
			return nil, fmt.Errorf("%s: expected UTF-8 text", place)
		}
		return flatValue(string(part.content), field.kind, place)
	}
	if len(field.mediaTypes) > 0 {
		admitted := false
		spelled := []string{}
		for _, one := range field.mediaTypes {
			admitted = admitted || sameASCII(one, part.mediaType)
			spelled = append(spelled, "`"+one+"`")
		}
		if !admitted {
			return nil, fmt.Errorf("%s: the part's media type is `%s`, not one of %s", place, part.mediaType, strings.Join(spelled, ", "))
		}
	}
	if field.filename != nil {
		if part.filename != nil {
			object[field.filename.wire] = *part.filename
		} else if field.filename.required {
			return nil, fmt.Errorf("%s: the part has no filename, which input field `%s` requires", place, field.filename.wire)
		}
	}
	if field.mediaType != nil {
		object[field.mediaType.wire] = part.mediaType
	}
	return base64.StdEncoding.EncodeToString(part.content), nil
}
"#;

/// What a surface adds where a `multipart` route has a path variable and `unknown: ignore`: a part
/// the path carries is held to the path's value, as a `form` key is ([`IGNORING`]'s
/// `echoedKeys`).
pub(super) const ECHOED_PARTS: &str = r#"
// echoedParts holds each part of a `multipart` body read under `unknown: ignore` that the path
// carries to the path's value, read as its field reads it: a part whose text does not decode or is
// not the path's is malformed.
func echoedParts(request *http.Request, body []byte, carried []flatField, path []pathMember, at string) error {
	parts, err := multipartParts(request, body)
	if err != nil {
		return err
	}
	for _, part := range parts {
		for index, field := range carried {
			if field.wire != part.name {
				continue
			}
			place := nested(at, part.name)
			if !utf8.Valid(part.content) {
				return fmt.Errorf("%s: expected UTF-8 text", place)
			}
			read, err := flatValue(string(part.content), field.kind, place)
			if err != nil {
				return err
			}
			if !sameValue(read, path[index].value) {
				return echoed(at, part.name)
			}
		}
	}
	return nil
}
"#;

/// What a surface adds where a route lists `request.query` (beyond10x/ess#754): the join of the
/// fields read from the query string with the body's, as the Rust server's `http::with_query`.
pub(super) const QUERIED: &str = r#"
// withQuery is value with the members the query string carries added (beyond10x/ess#754), each
// of listed that query holds, in the order listed: a field the route's `request.query` lists is
// read from the query string alone, and the body naming it is malformed.
func withQuery(value any, query map[string]any, listed []flatField, at string) (any, error) {
	object, ok := value.(map[string]any)
	if !ok {
		return value, nil
	}
	for _, field := range listed {
		if _, read := query[field.wire]; !read {
			continue
		}
		if _, present := object[field.wire]; present {
			return nil, fmt.Errorf("%s: the query carries this field, so the body does not", nested(at, field.wire))
		}
	}
	for wire, member := range query {
		object[wire] = member
	}
	return object, nil
}
"#;

/// Whether a route of `binding` lists `request.query`, which the surface joins with [`QUERIED`].
pub(super) fn reads_query_beside_body(binding: &http::Binding) -> bool {
    binding.ir().routes().iter().any(|route| {
        route
            .request
            .as_ref()
            .is_some_and(|request| !request.query.is_empty())
    })
}

/// Whether a route of `binding` takes a `multipart` request, which the surface reads with
/// [`MULTIPART`].
pub(super) fn reads_multipart(binding: &http::Binding) -> bool {
    binding.ir().routes().iter().any(|route| {
        route
            .request
            .as_ref()
            .is_some_and(|request| request.encoding == ess_http::Encoding::Multipart)
    })
}

/// Whether a `multipart` route of `binding` has a path variable and `unknown: ignore`, which the
/// surface reads with [`ECHOED_PARTS`].
pub(super) fn echoes_parts(binding: &http::Binding) -> bool {
    binding.ir().routes().iter().any(|route| {
        !route.parameters.is_empty()
            && route.request.as_ref().is_some_and(|request| {
                request.encoding == ess_http::Encoding::Multipart
                    && request.unknown == ess_http::Unknown::Ignore
            })
    })
}

/// The reader of a bare request body (`request: {body: input.<field>}`), which follows [`BOUND`]
/// only in a surface whose binding declares one, so every other bound surface keeps its bytes.
pub(super) const BARE_BODY: &str = r#"
// bareBody reads a `json` body whose whole value is the one input field `wire`, as the input
// object holding that field alone: read as jsonBody reads a body, the value kept as a member of
// the body is. Where the body is not required, no body, or one of only whitespace, is the input {}.
// members, where not nil, are the members the value's struct declares, the value itself or each
// element of the list it is, and any other member is refused, as the Rust server's decoder does.
func bareBody(body []byte, required bool, wire string, members []string) (any, error) {
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
	if members != nil {
		if err := closedMembers(value, members); err != nil {
			return nil, err
		}
	}
	return map[string]any{wire: memberValue(value)}, nil
}

// closedMembers refuses an object holding a member none of members names: the value, or each
// element of the list it is. A value of another shape is the decoder's to refuse.
func closedMembers(value any, members []string) error {
	objects, at := []jsonObject{}, []string{}
	switch shaped := value.(type) {
	case jsonObject:
		objects, at = append(objects, shaped), append(at, "body")
	case []any:
		for index, item := range shaped {
			if object, ok := item.(jsonObject); ok {
				objects, at = append(objects, object), append(at, fmt.Sprintf("body[%d]", index))
			}
		}
	}
	for position, object := range objects {
		for _, member := range object {
			named := false
			for _, name := range members {
				named = named || name == member.name
			}
			if !named {
				return DecodeError{At: nested(at[position], member.name), Expected: "no member: no field is named so", Found: describes(memberValue(member.value))}
			}
		}
	}
	return nil
}
"#;

/// The members a bare body's value may hold, each written as Go source: the wire names of the
/// fields of the struct the field is, or the list of which it is, where that struct refuses the
/// fields it does not declare. `None` for any other field, whose value the decoder holds to its
/// type alone. A struct nested deeper is held by the decoder, as in a body that is not bare.
fn bare_members(ir: &EssIr, field: &ResolvedField) -> Option<Vec<String>> {
    let handle = match &field.type_ref {
        ResolvedTypeRef::Declared { name } => name,
        ResolvedTypeRef::List { of } => match of.as_ref() {
            ResolvedTypeRef::Declared { name } => name,
            _ => return None,
        },
        _ => return None,
    };
    let declared = ir.named_type(handle);
    let ResolvedBody::Struct { fields, .. } = &declared.body else {
        return None;
    };
    ir.undeclared_fields(&declared.name).is_refused().then(|| {
        fields
            .iter()
            .map(|field| go_string(ess_gen::schema::wire_field_name(field)))
            .collect()
    })
}

/// Whether a route of `binding` takes one input field as its bare request body, which the surface
/// reads with [`BARE_BODY`].
pub(super) fn reads_a_bare_body(binding: &http::Binding) -> bool {
    binding.ir().routes().iter().any(|route| {
        route
            .request
            .as_ref()
            .is_some_and(|request| request.body.is_some())
    })
}

/// The route match of a bound surface: every row of the table, tried in turn in
/// [`http::dispatch_order`], path order but for a literal segment taking precedence over a
/// variable at the first position two templates of one method differ.
///
/// Every row is matched against the path as the request line spells it (`spelledPath`), as the
/// Rust server matches: a row the binding declares is a template whose variables are
/// percent-decoded one segment at a time; a derived row and the two documents are exact paths, so
/// `/openapi%2Ejson` is not `/openapi.json`. A path some row matches under the request's method is
/// answered by that row; one some row matches under another method is `405` naming every method a
/// matching row answers; one no row matches is `404`, naming the path as it was spelled.
///
/// Every text from the model or the binding is written into Go source with [`go_string`].
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    body: &mut String,
    ir: &EssIr,
    layout: &Layout,
    routes: &[http::Route<'_>],
    rows: &[Row],
    system: &str,
    exported: &str,
    reading: Reading,
) {
    let grants = http::checks_grants(ir);
    let caller = if grants { "caller *Caller, " } else { "" };
    let read = if reading.capped {
        format!("readBody(request, bodyLimit{exported}(request.Method, spelledPath(request)))")
    } else if reading.package_capped {
        // A component the binding bounds nothing of, in a package whose `readBody` takes a bound:
        // it reads the fixed bound, as it does without a binding.
        "readBody(request, bodyCap{limit: maxBody, tooLarge: bodyTooLarge})".to_owned()
    } else {
        "readBody(request)".to_owned()
    };
    let _ = write!(
        body,
        "\n// dispatch{exported} answers one request.\n//\n// The routes are templates an \
         `ess-http/1` binding declares, beside the routes it does not name.\n// A path no route \
         matches is a 404 naming where the whole table is published; a path some\n// route \
         matches under another method is a 405 naming every method its routes answer.\nfunc \
         dispatch{exported}(system *{system}, {caller}request *http.Request) response {{\n\tif \
         refused := tooManyHeaders(request); refused != nil {{\n\t\treturn *refused\n\t}}\n\tbody, \
         refused := {read}\n\tif refused != nil {{\n\t\treturn *refused\n\t}}\n\t// \
         Held from the port call through Pump and TakePublished: the system is shared by\n\t// \
         every connection.\n\tserving.Lock()\n\tdefer serving.Unlock()\n\tspelled := \
         spelledPath(request)\n\tallowed := []string{{}}\n"
    );
    let mut reads_body = false;
    let keys: Vec<(&str, &str)> = rows
        .iter()
        .map(|(method, path, _, _)| (*method, path.as_str()))
        .collect();
    for (method, path, _, _) in http::dispatch_order(&keys)
        .into_iter()
        .map(|index| &rows[index])
    {
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

/// `bodyLimit{exported}`: for a request's method and spelled path, the cap of the row that answers
/// it and the function that refuses a larger body. One block per row in [`http::dispatch_order`],
/// as [`dispatch`] walks them, so a literal row answers before a variable row, and a row that
/// bounds nothing (a document, a derived route, a declared route that reads no body) answers the
/// fixed bound before a later, more general row can capture its request. Asked before the body is
/// read, as `dispatch` is.
pub(super) fn body_limit(
    out: &mut String,
    binding: &http::Binding,
    ir: &EssIr,
    layout: &Layout,
    routes: &[http::Route<'_>],
    rows: &[Row],
    exported: &str,
) {
    let _ = write!(
        out,
        "\n// bodyLimit{exported} says how many body bytes the route a request matches reads, and the \
         answer to a larger\n// one (`limits`, `refusals.too_large`); the fixed bound and its 413 \
         where the binding bounds nothing.\nfunc bodyLimit{exported}(method string, spelled string) \
         bodyCap {{\n"
    );
    let fixed = "bodyCap{limit: maxBody, tooLarge: bodyTooLarge}";
    let keys: Vec<(&str, &str)> = rows
        .iter()
        .map(|(method, path, _, _)| (*method, path.as_str()))
        .collect();
    for (method, path, _, _) in http::dispatch_order(&keys)
        .into_iter()
        .map(|index| &rows[index])
    {
        let route = routes
            .iter()
            .find(|route| &route.path == path && route.method.as_str() == *method);
        let declared = route.and_then(|route| route.declared);
        let path_text = go_string(path);
        let opening = if declared.is_some() {
            format!("\tif _, ok := matched({path_text}, spelled); ok {{\n")
        } else {
            format!("\tif spelled == {path_text} {{\n")
        };
        out.push_str(&opening);
        let result = match (route, declared) {
            (Some(route), Some(declared)) if binding.ir().bounds_request(declared) => {
                let cap = binding
                    .ir()
                    .request_bytes(declared)
                    .unwrap_or(ess_http::FIXED_REQUEST_BYTES);
                format!(
                    "bodyCap{{limit: {cap}, tooLarge: refusalTooLarge{}}}",
                    refusal_suffix(ir, layout, route)
                )
            }
            _ => fixed.to_owned(),
        };
        let _ = writeln!(
            out,
            "\t\tif method == {} {{\n\t\t\treturn {result}\n\t\t}}\n\t}}",
            go_string(method)
        );
    }
    let _ = writeln!(out, "\treturn {fixed}\n}}");
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
    let suffix = refusal_suffix(ir, layout, route);
    let refuse = |standard: &str| {
        if declared {
            format!("refusalNotGranted{suffix}({standard})")
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

/// The path members a bound command route adds to its input, as Go source, and the statements
/// before that read them. Under `unknown: ignore` (beyond10x/ess#752), on a `json` route or one
/// with a path variable, what no field names is dropped and what the path carries is held to the
/// path's value first, as the Rust server reads it; every other route keeps its bytes.
fn ignoring(
    ir: &EssIr,
    route: &http::Route<'_>,
    command: &ResolvedCommand,
    at: &str,
    statements: &mut String,
    members: String,
) -> String {
    let request = http::request(route);
    if request.unknown != ess_http::Unknown::Ignore
        || (request.encoding != Encoding::Json && http::path_fields(ir, route).is_empty())
    {
        return members;
    }
    let _ = writeln!(statements, "\t\tcarried := {members}");
    let _ = match request.encoding {
        Encoding::Json => write!(
            statements,
            "\t\tvalue, err = ignoring(value, []string{{{}}}, carried, {at})\n\t\tif err != nil \
             {{\n\t\t\treturn nil, err\n\t\t}}\n",
            command
                .input
                .iter()
                .filter(|field| !http::carried(ir, route).contains(field.name.as_str()))
                .map(|field| go_string(ess_gen::schema::wire_field_name(field)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Encoding::Multipart => write!(
            statements,
            "\t\tif err := echoedParts(request, body, {}, carried, {at}); err != nil \
             {{\n\t\t\treturn nil, err\n\t\t}}\n",
            carried_flat_table(ir, route)
        ),
        encoding => write!(
            statements,
            "\t\tif err := echoedKeys({}, {}, carried, {at}); err != nil {{\n\t\t\treturn nil, \
             err\n\t\t}}\n",
            if encoding == Encoding::Query {
                "request.URL.RawQuery"
            } else {
                "string(body)"
            },
            carried_flat_table(ir, route)
        ),
    };
    "carried".to_owned()
}

/// The `[]flatField` literal of the fields the path carries, in path order, each one value.
fn carried_flat_table(ir: &EssIr, route: &http::Route<'_>) -> String {
    let rows: Vec<String> = http::path_fields(ir, route)
        .iter()
        .map(|(_, field)| {
            format!(
                "{{{}, {}, shapeOne}}",
                go_string(ess_gen::schema::wire_field_name(field)),
                flat_kind(ir, field)
            )
        })
        .collect();
    format!("[]flatField{{{}}}", rows.join(", "))
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

/// The closing statements of a bound command route's reader, as Go source: the body joined with
/// the path's `members`, joined in turn with the fields the route's `request.query` lists, read
/// from the query string as `encoding: query` reads them (beyond10x/ess#754), as the Rust server
/// joins them. A route that lists none keeps its bytes.
fn queried(
    ir: &EssIr,
    route: &http::Route<'_>,
    command: &ResolvedCommand,
    at: &str,
    members: &str,
) -> String {
    let request = http::request(route);
    if request.query.is_empty() {
        return format!("\t\treturn withPath(value, {members}, {at})\n");
    }
    let rows: Vec<String> = request
        .query_fields()
        .filter_map(|name| command.input.iter().find(|field| field.name == name))
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
    format!(
        "\t\tjoined, err := withPath(value, {members}, {at})\n\t\tif err != nil {{\n\t\t\treturn \
         nil, err\n\t\t}}\n\t\tlisted := []flatField{{{}}}\n\t\tquery, err := \
         flatObject(request.URL.RawQuery, listed, nil, {}, \"query\")\n\t\tif err != nil \
         {{\n\t\t\treturn nil, err\n\t\t}}\n\t\treturn withQuery(joined, query, listed, {at})\n",
        rows.join(", "),
        request.unknown == ess_http::Unknown::Ignore
    )
}

/// A `multipart` request's `[]partField` literal (beyond10x/ess#754): every input field neither the
/// path nor a part attribute carries, in declaration order, a `Bytes` field ([`http::file_part`]) a
/// file part with the wire names its attributes read into, as the Rust server's table states it.
fn part_table(ir: &EssIr, command: &ResolvedCommand, route: &http::Route<'_>) -> String {
    let carried = http::carried(ir, route);
    let read = http::attribute_fields(route.declared);
    let request = http::request(route);
    // The input field an attribute reads, as a `*partAttribute` literal.
    let attribute = |source: Option<&ess_http::Source>| {
        source
            .and_then(|source| {
                command
                    .input
                    .iter()
                    .find(|field| field.name == source.field())
            })
            .map_or_else(
                || "nil".to_owned(),
                |field| {
                    format!(
                        "&partAttribute{{{}, {}}}",
                        go_string(ess_gen::schema::wire_field_name(field)),
                        !field.type_ref.is_optional()
                    )
                },
            )
    };
    let rows: Vec<String> = command
        .input
        .iter()
        .filter(|field| {
            !carried.contains(field.name.as_str()) && !read.contains(field.name.as_str())
        })
        .map(|field| {
            let shape = match http::flat(ir, &field.type_ref).map(|(shape, _)| shape) {
                Some(http::Shape::Optional) => "shapeOptional",
                Some(http::Shape::List) => "shapeList",
                Some(http::Shape::One) | None => "shapeOne",
            };
            let wire = go_string(ess_gen::schema::wire_field_name(field));
            if http::file_part(ir, &field.type_ref).is_none() {
                return format!(
                    "{{{wire}, {}, {shape}, false, nil, nil, nil}}",
                    flat_kind(ir, field)
                );
            }
            let declared = request.parts.get(&field.name);
            let types: Vec<String> = declared
                .map(|part| part.media_types.iter().map(|it| go_string(it)).collect())
                .unwrap_or_default();
            format!(
                "{{{wire}, flatText, {shape}, true, {}, {}, {}}}",
                attribute(declared.and_then(|part| part.filename.as_ref())),
                attribute(declared.and_then(|part| part.media_type.as_ref())),
                if types.is_empty() {
                    "nil".to_owned()
                } else {
                    format!("[]string{{{}}}", types.join(", "))
                }
            )
        })
        .collect();
    format!("[]partField{{{}}}", rows.join(", "))
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
    let bare = http::bare_body(command, route.declared);
    // What the path or the query string (beyond10x/ess#754) carries is not the body's to hold.
    let mut outside = http::carried(ir, route);
    outside.extend(http::queried(route.declared));
    let read = match request.encoding {
        Encoding::Json if bare.is_some() => format!(
            "bareBody(body, {}, {}, {})",
            http::body_required_without(command, &outside),
            go_string(
                bare.map(ess_gen::schema::wire_field_name)
                    .unwrap_or_default()
            ),
            bare.and_then(|field| bare_members(ir, field)).map_or_else(
                || "nil".to_owned(),
                |members| format!("[]string{{{}}}", members.join(", "))
            )
        ),
        Encoding::Json if ignore => format!(
            "openJSONBody(body, {})",
            http::body_required_without(command, &outside),
        ),
        Encoding::Json => format!(
            "jsonBody(body, {}, []string{{{}}})",
            http::body_required_without(command, &outside),
            command
                .input
                .iter()
                .map(|field| go_string(ess_gen::schema::wire_field_name(field)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Encoding::Multipart => format!(
            "multipartObject(request, body, {}, {}, {ignore}, \"body\")",
            part_table(ir, command, route),
            carried_table(ir, route)
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
    let (mut statements, members) = path_members(ir, route);
    let members = ignoring(ir, route, command, &at, &mut statements, members);
    let joined = queried(ir, route, command, &at, &members);
    let suffix = refusal_suffix(ir, layout, route);
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
         err\n\t\t}}\n{statements}{joined}\t}}()\n\tif err \
         != nil {{\n\t\treturn refusalMalformed{suffix}(err.Error())\n\t}}\n\tinput, err := \
         decodeCommand{function}(value, \"body\")\n\tif err != nil {{\n\t\treturn \
         refusalMalformed{suffix}(err.Error())\n\t}}\n\toutcome, unmet := \
         system.{field}.{method}(input)\n\tif unmet != nil {{\n\t\treturn \
         refusalUnfinished{suffix}(unmet.Error(), false)\n\t}}\n\tfailure := \
         system.Pump()\n\tsystem.TakePublished()\n{invocations}\tif failure != nil {{\n\t\treturn \
         refusalUnfinished{suffix}(\"delivering what the command published: \"+failure.Error(), \
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
        outcome_answer(out, ir, route, command, [function, suffix]);
    }
}

/// `answerBound*`: the declared answer of each outcome the route lists, read off the envelope the
/// shared rendering wrote, and that envelope at its derived status for every other outcome.
fn outcome_answer(
    out: &mut String,
    ir: &EssIr,
    route: &http::Route<'_>,
    command: &ResolvedCommand,
    [function, suffix]: [&str; 2],
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
            .any(|(_, declared)| declared.body.reads_response());
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
         refusalUnfinished{suffix}(err.Error(), true)\n\t}}\n\tif declared != nil {{\n\t\treturn \
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

/// The answer of an outcome whose body is one response `field`, bare: as JSON, or with
/// `encoding: text` its string value alone, read into `bodyText` by a line pushed onto `arm`.
fn field_answer(
    command: &ResolvedCommand,
    field: &str,
    declared: &ess_http::Outcome,
    arm: &mut String,
) -> String {
    let status = declared.status;
    let wire = command
        .response
        .iter()
        .find(|declared| declared.name == field)
        .map_or(field, ess_gen::schema::wire_field_name);
    if declared.encoding == ResponseEncoding::Text {
        // The media type is written inline rather than as a constant beside `mediaJSON`, so a
        // server whose binding answers no text keeps its bytes.
        let _ = writeln!(
            arm,
            "\t\tbodyText, _ := memberOf(reply, {}).(string)",
            go_string(wire)
        );
        format!(
            "response{{status: {status}, contentType: {}, body: bodyText}}",
            go_string(http::TEXT_CONTENT_TYPE)
        )
    } else {
        format!(
            "response{{status: {status}, contentType: mediaJSON, body: bare(memberOf(reply, \
             {}))}}",
            go_string(wire)
        )
    }
}

/// A wrapped outcome body's answer (beyond10x/ess#755): the response, or its one `field`, under
/// the one `member`, written as the bare form writes it.
fn wrapped_answer(
    command: &ResolvedCommand,
    status: u16,
    member: &str,
    field: Option<&str>,
) -> String {
    let value = match field {
        None => "reply".to_owned(),
        Some(field) => format!(
            "memberOf(reply, {})",
            go_string(
                command
                    .response
                    .iter()
                    .find(|declared| declared.name == *field)
                    .map_or(field, ess_gen::schema::wire_field_name)
            )
        ),
    };
    format!(
        "response{{status: {status}, contentType: mediaJSON, body: \
         bare(jsonObject{{{{name: {}, value: {value}}}}})}}",
        go_string(member)
    )
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
    let mut arm = String::new();
    let body = match &declared.body {
        Body::Envelope => {
            format!("response{{status: {status}, contentType: mediaJSON, body: envelope}}")
        }
        Body::Response => {
            format!("response{{status: {status}, contentType: mediaJSON, body: bare(reply)}}")
        }
        Body::Wrapped { member, field } => {
            wrapped_answer(command, status, member, field.as_deref())
        }
        Body::ResponseField(field) => field_answer(command, field, declared, &mut arm),
        Body::None => format!("response{{status: {status}}}"),
        Body::Error { member } => format!(
            "response{{status: {status}, contentType: mediaJSON, body: errorObject({}, {}, \
             payload)}}",
            go_string(member),
            go_string(wire_name.as_deref().unwrap_or_default())
        ),
    };
    let _ = writeln!(arm, "\t\tanswer := {body}");
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
    let suffix = refusal_suffix(ir, layout, route);
    let _ = write!(
        out,
        "\n// serveBound{function} answers `{}` `{}`: `{}`, read and answered as its `ess-http/1`\n\
         // binding declares.\nfunc serveBound{function}(system *{system}, request *http.Request, \
         path []string) response {{\n",
        route.method.as_str(),
        route.path,
        view.name,
    );
    let representations = route
        .declared
        .map_or(&[][..], |declared| &declared.representations[..]);
    out.push_str(&selection(representations, suffix));
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
             refusalMalformed{suffix}(err.Error())\n\t}}\n\tparams, err := \
             decodeParams{function}(value, \"query\")\n\tif err != nil {{\n\t\treturn \
             refusalMalformed{suffix}(err.Error())\n\t}}\n",
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
         refusalUnfinished{suffix}(unmet.Error(), false)\n\t}}\n\tencoded := make([]any, 0, \
         len(rows))\n\tfor _, row := range rows {{\n\t\tencoded = append(encoded, \
         encodeView{function}(row))\n\t}}\n"
    );
    if !representations.is_empty() {
        out.push_str(&csv_answer(view));
    }
    json_answer(out, route);
}

/// Go code answering a bound view route's encoded rows as JSON: `rows`, `array` or `single`, bare
/// or under the member a wrapped body names, and a `single` view's empty answer.
fn json_answer(out: &mut String, route: &http::Route<'_>) {
    let body = route
        .declared
        .map_or(ess_http::ViewBody::Rows, |declared| declared.body.form);
    let empty = route
        .declared
        .filter(|_| body == ess_http::ViewBody::Single)
        .and_then(|declared| declared.empty.as_ref());
    // The member a wrapped body names (beyond10x/ess#755): the bare answer goes under it.
    let member = route
        .declared
        .and_then(|declared| declared.body.member.as_deref());
    match (body, empty) {
        (_, Some(empty)) => {
            let one = member.map_or_else(
                || "encoded[0]".to_owned(),
                |member| {
                    format!(
                        "jsonObject{{{{name: {}, value: encoded[0]}}}}",
                        go_string(member)
                    )
                },
            );
            let none = match &empty.body {
                RefusalBody::None => format!("response{{status: {}}}", empty.status),
                RefusalBody::Literal { member, text } => format!(
                    "response{{status: {}, contentType: mediaJSON, body: literalObject({}, {})}}",
                    empty.status,
                    go_string(member),
                    go_string(text)
                ),
            };
            let _ = write!(
                out,
                "\tif len(encoded) == 0 {{\n\t\treturn {none}\n\t}}\n\treturn \
                 response{{status: 200, contentType: mediaJSON, body: bare({one})}}\n}}\n"
            );
        }
        // The rows, bare: the array `rows` would hold.
        (ess_http::ViewBody::Array, None) => match member {
            None => out.push_str("\treturn rendered(200, encoded)\n}\n"),
            Some(member) => {
                let _ = write!(
                    out,
                    "\treturn rendered(200, map[string]any{{{}: encoded}})\n}}\n",
                    go_string(member)
                );
            }
        },
        _ => out.push_str("\treturn rendered(200, map[string]any{\"rows\": encoded})\n}\n"),
    }
}

/// Go code choosing the representation the query selects (beyond10x/ess#753), as the Rust server
/// does: `chosen`, its `separator`, whether a `header` line comes first, and its `headers`; a value
/// no representation names, and the parameter given twice, are `malformed`. Written only on a
/// route that declares representations, so every other package keeps its bytes.
fn selection(representations: &[ess_http::Representation], suffix: &str) -> String {
    let Some(first) = representations.first() else {
        return String::new();
    };
    let parameter = &first.query.parameter;
    let values: Vec<String> = representations
        .iter()
        .map(|representation| format!("`{}`", representation.query.value))
        .collect();
    let expected = match &values[..] {
        [only] => only.clone(),
        several => format!("one of {}", several.join(", ")),
    };
    let mut cases = String::new();
    for representation in representations {
        let headers: Vec<String> = representation
            .headers
            .iter()
            .filter_map(|(name, header)| match header {
                Header::Literal(text) => {
                    Some(format!("{{{}, {}}}", go_string(name), go_string(text)))
                }
                Header::Source(_) | Header::Challenge(_) => None,
            })
            .collect();
        let _ = writeln!(
            cases,
            "\tcase value == {}:\n\t\tchosen, separator, header, headers = true, {}, {}, \
             [][2]string{{{}}}",
            go_string(&representation.query.value),
            go_string(&representation.encoding.csv.separator.as_char().to_string()),
            representation.encoding.csv.header,
            headers.join(", ")
        );
    }
    format!(
        "\t// The representation the query selects: its separator, whether a header line comes \
         first, and\n\t// its headers; none chosen answers the JSON body.\n\tselected, err := \
         queryObject(request.URL.RawQuery, []queryParam{{{{wire: {parameter}, scalar: \
         queryText}}}}, \"query\")\n\tif err != nil {{\n\t\treturn \
         refusalMalformed{suffix}(err.Error())\n\t}}\n\tchosen, separator, header := false, \"\", \
         false\n\tvar headers [][2]string\n\tswitch value, named := selected[{parameter}]; {{\n\tcase \
         !named:\n{cases}\tdefault:\n\t\treturn refusalMalformed{suffix}({})\n\t}}\n",
        go_string(&format!("query.{parameter}: expected {expected}")),
        parameter = go_string(parameter),
    )
}

/// Go code answering the encoded rows as CSV when [`selection`] chose a representation, as the
/// Rust server writes it: the rows the JSON answer renders, read back, the view's fields in
/// declaration order under their wire names, CRLF line ends, an absent value an empty field, each
/// value in its flat text spelling, and a value holding the separator, `"`, CR or LF quoted with
/// `"` doubled.
fn csv_answer(view: &ResolvedView) -> String {
    let columns: Vec<String> = view
        .fields
        .iter()
        .map(|field| go_string(ess_gen::schema::wire_field_name(field)))
        .collect();
    format!(
        "\tif chosen {{\n\t\tread, _ := parseJSON(rendered(200, encoded).body)\n\t\titems, _ := \
         read.([]any)\n\t\tcolumns := []string{{{}}}\n\t\tvar out strings.Builder\n\t\tfield := \
         func(text string) {{\n\t\t\tif strings.ContainsAny(text, separator+\"\\\"\\r\\n\") \
         {{\n\t\t\t\tout.WriteByte('\"')\n\t\t\t\tout.WriteString(strings.ReplaceAll(text, \
         \"\\\"\", \"\\\"\\\"\"))\n\t\t\t\tout.WriteByte('\"')\n\t\t\t}} else \
         {{\n\t\t\t\tout.WriteString(text)\n\t\t\t}}\n\t\t}}\n\t\tif header {{\n\t\t\tfor index, \
         column := range columns {{\n\t\t\t\tif index > 0 {{\n\t\t\t\t\tout.WriteString(separator)\
         \n\t\t\t\t}}\n\t\t\t\tfield(column)\n\t\t\t}}\n\t\t\tout.WriteString(\"\\r\\n\")\n\t\t}}\n\t\t\
         for _, row := range items {{\n\t\t\tfor index, column := range columns {{\n\t\t\t\tif index \
         > 0 {{\n\t\t\t\t\tout.WriteString(separator)\n\t\t\t\t}}\n\t\t\t\tswitch value := \
         memberOf(row, column).(type) {{\n\t\t\t\tcase nil:\n\t\t\t\t\tfield(\"\")\n\t\t\t\tcase \
         string:\n\t\t\t\t\tfield(value)\n\t\t\t\tdefault:\n\t\t\t\t\tfield(bare(value))\n\t\t\t\t}}\
         \n\t\t\t}}\n\t\t\tout.WriteString(\"\\r\\n\")\n\t\t}}\n\t\treturn response{{status: 200, \
         contentType: \"text/csv; charset=utf-8\", body: out.String(), headers: headers}}\n\t}}\n",
        columns.join(", ")
    )
}

/// The standard refusals of a bound package's declared routes, as its binding answers them
/// (`refusals:`), and today's answer for each the binding does not override. `refusalNotGranted`
/// only where some declared route checks a grant, and `refusalTooLarge` only where one it answers
/// bounds its request, so nothing emitted goes unused.
pub(super) fn refusals(binding: &http::Binding, checks_grant: bool, too_large: bool) -> String {
    refusal_set(
        &RefusalSet {
            suffix: "",
            on: "a bound route".to_owned(),
            from: "",
            checks_grant,
            too_large,
        },
        |key| binding.refusal(key),
    )
}

/// The standard refusals of each declared route that overrides one itself (`routes[i].refusals`),
/// in route order: its own set of functions, named with [`refusal_suffix`], answering its own
/// overrides over the binding's. Every other route answers through [`refusals`], so it keeps its
/// bytes.
pub(super) fn route_refusals(
    ir: &EssIr,
    layout: &Layout,
    binding: &http::Binding,
    routes: &[http::Route<'_>],
) -> String {
    let mut out = String::new();
    for route in routes {
        let Some(own) = route.declared.filter(|own| !own.refusals.is_empty()) else {
            continue;
        };
        let checks_grant = match route.serves {
            Served::Command(_) => http::checks_grants(ir),
            Served::View(handle) => http::read_checked(ir, handle.name()),
        };
        out.push_str(&refusal_set(
            &RefusalSet {
                suffix: refusal_suffix(ir, layout, route),
                on: format!("`{} {}`", route.method.as_str(), route.path),
                from: "routes[].",
                checks_grant,
                too_large: binding.ir().bounds_request(own),
            },
            |key| binding.refusal_on(own, key),
        ));
    }
    out
}

/// The suffix of the standard-refusal functions a declared route answers through: none for the
/// binding's (`refusalMalformed`, …), the construct's identifier for a route that overrides one
/// itself (`routes[i].refusals`).
fn refusal_suffix<'a>(ir: &EssIr, layout: &'a Layout, route: &http::Route<'_>) -> &'a str {
    if route
        .declared
        .is_none_or(|declared| declared.refusals.is_empty())
    {
        return "";
    }
    match route.serves {
        Served::Command(handle) => ident(layout, &ir.command(handle).name),
        Served::View(handle) => ident(layout, &ir.view(handle).name),
    }
}

/// One set of standard-refusal functions: their name suffix, the route they answer on, as the
/// comments name it, where the overrides are declared, whether a grant is checked and whether a
/// route answering through the set bounds its request.
struct RefusalSet<'a> {
    suffix: &'a str,
    on: String,
    from: &'static str,
    checks_grant: bool,
    too_large: bool,
}

/// The functions answering each standard refusal for `set`: `refusalMalformed`,
/// `refusalUnfinished` and, where a grant is checked, `refusalNotGranted`, each with
/// `set.suffix`. `lookup` gives the override standing for a key.
fn refusal_set<'a>(
    set: &RefusalSet<'_>,
    lookup: impl Fn(RefusalKey) -> Option<&'a ess_http::Refusal>,
) -> String {
    let RefusalSet {
        suffix, on, from, ..
    } = set;
    let mut out = format!(
        "\n// refusalMalformed{suffix} is a request {on} cannot read as its binding declares: \
         `{from}refusals.malformed`,\n// else 400.\n"
    );
    match lookup(RefusalKey::Malformed) {
        Some(refusal) => {
            let _ = write!(
                out,
                "func refusalMalformed{suffix}(_ string) response {{\n{}}}\n",
                overridden(refusal)
            );
        }
        None => {
            let _ = write!(
                out,
                "func refusalMalformed{suffix}(detail string) response {{\n\treturn refusal(400, \
                 detail)\n}}\n"
            );
        }
    }
    let _ = write!(
        out,
        "\n// refusalUnfinished{suffix} is an unfinished realization on {on}: \
         `{from}refusals.unfinished`, else 501\n// with committed.\n"
    );
    match lookup(RefusalKey::Unfinished) {
        Some(refusal) => {
            let _ = write!(
                out,
                "func refusalUnfinished{suffix}(_ string, _ bool) response {{\n{}}}\n",
                overridden(refusal)
            );
        }
        None => {
            let _ = write!(
                out,
                "func refusalUnfinished{suffix}(detail string, committed bool) response \
                 {{\n\treturn unfinished(detail, committed)\n}}\n"
            );
        }
    }
    if set.checks_grant {
        let _ = write!(
            out,
            "\n// refusalNotGranted{suffix} is a caller no grant admits, on {on}: \
             `{from}refusals.not_granted`, else the\n// standard 403.\n"
        );
        match lookup(RefusalKey::NotGranted) {
            Some(refusal) => {
                let _ = write!(
                    out,
                    "func refusalNotGranted{suffix}(_ response) response {{\n{}}}\n",
                    overridden(refusal)
                );
            }
            None => {
                let _ = write!(
                    out,
                    "func refusalNotGranted{suffix}(standard response) response {{\n\treturn \
                     standard\n}}\n"
                );
            }
        }
    }
    if set.too_large {
        let _ = write!(
            out,
            "\n// refusalTooLarge{suffix} is a body past the most {on} reads: \
             `{from}refusals.too_large`, else the\n// 413 naming the limit.\n"
        );
        match lookup(RefusalKey::TooLarge) {
            Some(refusal) => {
                let _ = write!(
                    out,
                    "func refusalTooLarge{suffix}(_ int) response {{\n{}}}\n",
                    overridden(refusal)
                );
            }
            None => {
                let _ = write!(
                    out,
                    "func refusalTooLarge{suffix}(limit int) response {{\n\treturn \
                     bodyTooLarge(limit)\n}}\n"
                );
            }
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
