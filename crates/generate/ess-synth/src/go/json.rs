//! `Json` in the Go target (beyond10x/ess#224).
//!
//! # The representation: `primitives.Json`, a wrapper over the document's text
//!
//! A `Json` value is carried as the JSON text it is, inside the generated `primitives.Json` — the
//! same shape as `Decimal`, `Timestamp`, `Duration` and `Uuid`: an unexported `string`, a `NewJson`
//! constructor and a `Value` accessor. Text keeps both things a `Json` value must not lose, the
//! order of an object's members and the spelling of a number, because nothing reads them into
//! anything that could reorder or round them.
//!
//! `encoding/json`'s `json.RawMessage` keeps the same bytes and was the other candidate. It is not
//! the representation, for three reasons that all come from its being a `[]byte`:
//!
//! - a `[]byte` is not comparable, so every generated struct, union shape and event holding a
//!   `Json` member would stop being comparable with `==` — a property a `String`, `Decimal` or
//!   `Uuid` member leaves intact, and one callers of the generated types can already be using;
//! - a slice aliases: a caller that kept a `RawMessage` it was handed could change a value an
//!   entity or an event already holds, which no other generated value permits;
//! - it would be the one primitive not distinct from its representation, where every other
//!   wrapper exists precisely to be distinct (see `PRIMITIVES_DOC`).
//!
//! `json.RawMessage` is still what the served surface writes: the encoder hands
//! `json.RawMessage(value.Value())` to `encoding/json`, which embeds it member for member and
//! digit for digit. What it does not keep is a string's escaping: a string is written the way
//! `encoding/json` escapes one (`<`, `>` and `&` as `<`, `>`, `&`), which is the
//! same string, as the Rust target's writer re-escapes one too. Go's zero value needs no constructor, so `Json{}` is spellable; its `Value` is
//! `null`, so a zero `Json` is a legal document rather than a body that fails to encode.
//!
//! # Reading one off the wire with its members in order
//!
//! The served surface parses a body into `any` with `encoding/json`, which reads an object into a
//! `map[string]any` and so forgets its member order. A model that uses `Json` therefore reads
//! bodies into a tree whose objects remember their order ([`WIRE`]'s `object`), after the standard
//! decoder has accepted the body — so every refusal a malformed body gets is the one it got
//! before — and `jsonAt` writes a `Json` position back out of that tree as text.
//!
//! # Only when the model uses `Json`
//!
//! A model that does not keeps its bytes: no `Json` wrapper in `primitives`, no ordered reader, no
//! `bytes` import in `wire.go`. [`with_json`] applies the few substitutions to the fixed helpers,
//! and refuses (panics) when a piece it replaces is not there, so the fixed text and the
//! substitution cannot drift apart silently.

/// `true` when the model names `Json` anywhere the Go target types.
pub(crate) fn used(ir: &ess_compiler::EssIr) -> bool {
    !ess_compiler::binary64::locations_of(ir, ess_domain::Primitive::Json).is_empty()
}

/// The `Json` wrapper appended to the `primitives` package of a model that uses `Json`.
pub(crate) const PRIMITIVE: &str = "
// Json is any JSON document, carried as its text — object members in the order they arrived,
// numbers in the spelling they arrived in.
//
// Text rather than a parsed tree, so nothing can reorder or round it; a wrapper rather than
// `json.RawMessage`, so a type holding one stays comparable and the value cannot be changed
// through a slice someone else kept. The field is unexported, so the only way to make one is
// [NewJson] — but Go's zero value needs no constructor, so `Json{}` is still spellable, and it is
// the document `null` (see TARGET.md).
type Json struct {
	value string
}

// NewJson wraps a JSON document's text as a Json. The caller owns the spelling: a text that is
// not JSON is refused when the value is written, not here.
func NewJson(value string) Json {
	return Json{value: value}
}

// Value is the wrapped document's text; `null` for the zero value.
func (v Json) Value() string {
	if v.value == \"\" {
		return \"null\"
	}
	return v.value
}
";

/// What `wire.go` adds for a model that uses `Json`: the ordered tree and the `Json` reader.
pub(crate) const WIRE: &str = r#"
// object is a JSON object as it arrived: its members, and the order they arrived in.
//
// Only a model that uses `Json` reads objects this way, because only a `Json` value has to leave
// with its members where they were.
type object struct {
	members map[string]any
	order   []string
}

// ordered reads one JSON document into the tree the codecs walk, keeping each object's member
// order.
//
// Called only on a body the standard decoder has already accepted, so its refusals are that
// decoder's, word for word.
func ordered(body []byte) (any, error) {
	decoder := json.NewDecoder(bytes.NewReader(body))
	decoder.UseNumber()
	return orderedValue(decoder)
}

// orderedValue reads the next value from the decoder's tokens.
func orderedValue(decoder *json.Decoder) (any, error) {
	token, err := decoder.Token()
	if err != nil {
		return nil, err
	}
	delimiter, ok := token.(json.Delim)
	if !ok {
		return token, nil
	}
	if delimiter == '[' {
		items := []any{}
		for decoder.More() {
			item, err := orderedValue(decoder)
			if err != nil {
				return nil, err
			}
			items = append(items, item)
		}
		_, err = decoder.Token()
		return items, err
	}
	held := &object{members: map[string]any{}}
	for decoder.More() {
		key, err := decoder.Token()
		if err != nil {
			return nil, err
		}
		name, _ := key.(string)
		member, err := orderedValue(decoder)
		if err != nil {
			return nil, err
		}
		// The last of two equal names wins, as it does for the standard decoder; the member keeps
		// the place its name first took.
		if _, seen := held.members[name]; !seen {
			held.order = append(held.order, name)
		}
		held.members[name] = member
	}
	_, err = decoder.Token()
	return held, err
}

// jsonAt is the `Json` value at this path, as text: object members in the order they arrived,
// numbers in the spelling they arrived in.
//
// Every JSON value is a `Json` value, so a tree the reader built is never refused; it takes the
// path and the expectation so a `Json` leaf is read the way every other leaf is.
func jsonAt(value any, at string, expected string) (string, error) {
	var out bytes.Buffer
	if err := writeJSON(&out, value); err != nil {
		return "", DecodeError{At: at, Expected: expected, Found: describes(value)}
	}
	return out.String(), nil
}

// writeJSON appends one value of the tree as JSON text.
func writeJSON(out *bytes.Buffer, value any) error {
	switch shaped := value.(type) {
	case nil:
		out.WriteString("null")
	case bool:
		out.WriteString(strconv.FormatBool(shaped))
	case json.Number:
		out.WriteString(shaped.String())
	case string:
		return writeText(out, shaped)
	case []any:
		out.WriteByte('[')
		for index, item := range shaped {
			if index > 0 {
				out.WriteByte(',')
			}
			if err := writeJSON(out, item); err != nil {
				return err
			}
		}
		out.WriteByte(']')
	case *object:
		out.WriteByte('{')
		for index, name := range shaped.order {
			if index > 0 {
				out.WriteByte(',')
			}
			if err := writeText(out, name); err != nil {
				return err
			}
			out.WriteByte(':')
			if err := writeJSON(out, shaped.members[name]); err != nil {
				return err
			}
		}
		out.WriteByte('}')
	default:
		return fmt.Errorf("a value of an unknown shape")
	}
	return nil
}

// writeText appends one string as a JSON string, escaped as `encoding/json` escapes it.
func writeText(out *bytes.Buffer, text string) error {
	encoded, err := json.Marshal(text)
	if err != nil {
		return err
	}
	out.Write(encoded)
	return nil
}
"#;

/// The fixed helpers' pieces a model that uses `Json` reads differently: objects arrive as the
/// ordered `*object` rather than a `map[string]any`.
pub(crate) const WIRE_SUBSTITUTIONS: &[(&str, &str)] = &[
    (
        "\tcase map[string]any:\n\t\treturn \"an object\"\n",
        "\tcase *object:\n\t\treturn \"an object\"\n",
    ),
    (
        "\tobject, ok := value.(map[string]any)\n\tif !ok {\n\t\treturn nil, DecodeError{At: at, \
         Expected: expected, Found: describes(value)}\n\t}\n\treturn object, nil\n",
        "\tobject, ok := value.(*object)\n\tif !ok {\n\t\treturn nil, DecodeError{At: at, \
         Expected: expected, Found: describes(value)}\n\t}\n\treturn object.members, nil\n",
    ),
];

/// The same for the surface's `readJSON`: the standard decoder still decides what is refused, and
/// the accepted body is then read into the ordered tree.
pub(crate) const SURFACE_SUBSTITUTIONS: &[(&str, &str)] = &[(
    "\t\tanswer := refusal(400, fmt.Sprintf(\"the body is not JSON: %s\", err))\n\t\treturn nil, \
     &answer\n\t}\n\treturn value, nil\n",
    "\t\tanswer := refusal(400, fmt.Sprintf(\"the body is not JSON: %s\", err))\n\t\treturn nil, \
     &answer\n\t}\n\t// A model that uses `Json` keeps each object's member order; see `ordered`.\n\
     \tkept, err := ordered(body)\n\tif err != nil {\n\t\tanswer := refusal(400, \
     fmt.Sprintf(\"the body is not JSON: %s\", err))\n\t\treturn nil, &answer\n\t}\n\treturn \
     kept, nil\n",
)];

/// The fixed helper text with each substitution applied.
///
/// # Panics
///
/// When a piece to replace is not in `fixed` exactly once — the fixed helpers moved and this
/// module did not, a defect in ess-synth rather than a fact about a specification.
pub(crate) fn with_json(fixed: &str, substitutions: &[(&str, &str)]) -> String {
    let mut out = fixed.to_owned();
    for (from, to) in substitutions {
        assert_eq!(
            out.matches(from).count(),
            1,
            "a Json substitution no longer matches the fixed Go helpers: {from:?}"
        );
        out = out.replace(from, to);
    }
    out
}
