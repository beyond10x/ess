package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"
	"unicode/utf16"
	"unicode/utf8"
)

// The UTF-8 byte length of a String, `{utf8_bytes: <parent>}` (docs/design/expression-family-source22.md,
// "String .utf8_bytes", decisions 11 and 19): the shared vectors
// `crates/specify/ess-primitives/tests/vectors/utf8-bytes.json`, copied beside this file, answered
// by the Go reader as `tests/utf8_bytes.rs` answers them in Rust and `predicate.test.ts` in
// TypeScript. A text is written as its UTF-16 code units so a lone surrogate can be written at all;
// here it is the WTF-8 bytes a Go string holds for one, which is no UTF-8, and its byte length is
// Unknown. Each paired fault — a scalar count, UTF-16 units, graphemes, raw bytes of invalid text —
// must disagree with a row.

type utf8Text struct {
	Name  string   `json:"name"`
	Units []uint16 `json:"units"`
	Bytes *int     `json:"bytes"`
	Count *int     `json:"count"`
}

// wtf8 is the Go string a run of UTF-16 code units spells: a surrogate pair is its scalar, and a
// lone surrogate its three WTF-8 bytes, which `utf8.ValidString` refuses — never U+FFFD, which is
// what `utf16.Decode` would put in its place.
func wtf8(units []uint16) string {
	var out []byte
	for index := 0; index < len(units); index++ {
		unit := rune(units[index])
		if utf16.IsSurrogate(unit) && unit < 0xdc00 && index+1 < len(units) {
			if pair := utf16.DecodeRune(unit, rune(units[index+1])); pair != utf8.RuneError {
				out = utf8.AppendRune(out, pair)
				index++
				continue
			}
		}
		if utf16.IsSurrogate(unit) {
			out = append(out, byte(0xe0|unit>>12), byte(0x80|(unit>>6)&0x3f), byte(0x80|unit&0x3f))
			continue
		}
		out = utf8.AppendRune(out, unit)
	}
	return string(out)
}

func utf8Row(t *testing.T, name string, raw []byte) Row {
	var row Row
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	if err := decoder.Decode(&row); err != nil {
		t.Fatalf("%s: row: %v", name, err)
	}
	return row
}

func utf8Vectors(t *testing.T) (texts []utf8Text, evaluate []struct {
	Name      string          `json:"name"`
	Predicate json.RawMessage `json:"predicate"`
	Row       json.RawMessage `json:"row"`
	Truth     string          `json:"truth"`
}, refused []struct {
	Name      string          `json:"name"`
	Predicate json.RawMessage `json:"predicate"`
}) {
	raw, err := os.ReadFile("utf8-bytes.json")
	if err != nil {
		t.Fatalf("vectors: %v", err)
	}
	var vectors struct {
		Texts    []utf8Text `json:"texts"`
		Evaluate []struct {
			Name      string          `json:"name"`
			Predicate json.RawMessage `json:"predicate"`
			Row       json.RawMessage `json:"row"`
			Truth     string          `json:"truth"`
		} `json:"evaluate"`
		Refused []struct {
			Name      string          `json:"name"`
			Predicate json.RawMessage `json:"predicate"`
		} `json:"refused"`
	}
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatalf("vectors: %v", err)
	}
	return vectors.Texts, vectors.Evaluate, vectors.Refused
}

func TestUtf8Bytes(t *testing.T) {
	names := map[truth]string{truthTrue: "true", truthFalse: "false", truthUnknown: "unknown"}
	texts, evaluate, refused := utf8Vectors(t)
	answered := 0
	for _, vector := range texts {
		source := factSource{"label": wtf8(vector.Units)}
		if vector.Bytes == nil {
			parsed, err := parsePredicate(json.RawMessage(`{"compare": {"left": {"utf8_bytes": "label"}, "op": "gte", "right": 0}}`))
			if err != nil {
				t.Fatalf("%s: %v", vector.Name, err)
			}
			if got := names[parsed.evaluate(source)]; got != "unknown" {
				t.Errorf("%s: got %s, want unknown", vector.Name, got)
			}
			answered++
			continue
		}
		for _, check := range []string{
			fmt.Sprintf(`{"compare": {"left": {"utf8_bytes": "label"}, "op": "eq", "right": %d}}`, *vector.Bytes),
			fmt.Sprintf(`{"label.count": {"eq": %d}}`, *vector.Count),
		} {
			parsed, err := parsePredicate(json.RawMessage(check))
			if err != nil {
				t.Fatalf("%s: %v", vector.Name, err)
			}
			if got := names[parsed.evaluate(source)]; got != "true" {
				t.Errorf("%s: %s is %s", vector.Name, check, got)
			}
		}
		answered++
	}
	for _, vector := range evaluate {
		parsed, err := parsePredicate(vector.Predicate)
		if err != nil {
			t.Errorf("%s: %v", vector.Name, err)
			continue
		}
		if got := names[parsed.evaluate(facts(utf8Row(t, vector.Name, vector.Row)))]; got != vector.Truth {
			t.Errorf("%s: got %s, want %s", vector.Name, got, vector.Truth)
		}
		answered++
	}
	for _, vector := range refused {
		if _, err := parsePredicate(vector.Predicate); err == nil {
			t.Errorf("%s: read, want refused", vector.Name)
		}
		var node any
		decoder := json.NewDecoder(strings.NewReader(string(vector.Predicate)))
		decoder.UseNumber()
		if err := decoder.Decode(&node); err == nil {
			if err := admitPredicateVersion(node, 40); err == nil {
				t.Errorf("%s: admitted, want refused", vector.Name)
			}
		}
		answered++
	}
	if answered < 40 {
		t.Fatalf("only %d vectors were answered", answered)
	}
}

// ---- paired faults ------------------------------------------------------------------------------

type measure func(text string) (int, bool)

func utf8Length(text string) (int, bool) {
	if !utf8.ValidString(text) {
		return 0, false
	}
	return len(text), true
}

func scalarCount(text string) (int, bool) { return utf8.RuneCountInString(text), true }

func utf16Units(text string) (int, bool) { return len(utf16.Encode([]rune(text))), true }

func rawBytes(text string) (int, bool) { return len(text), true }

func graphemeCount(text string) (int, bool) {
	count := 0
	for _, character := range text {
		if character < 0x300 || character > 0x36f {
			count++
		}
	}
	return count, true
}

func TestUtf8BytesFaults(t *testing.T) {
	texts, _, _ := utf8Vectors(t)
	answer := func(compute measure, vector utf8Text) string {
		length, ok := compute(wtf8(vector.Units))
		if !ok {
			return "unknown"
		}
		return fmt.Sprint(length)
	}
	want := func(vector utf8Text) string {
		if vector.Bytes == nil {
			return "unknown"
		}
		return fmt.Sprint(*vector.Bytes)
	}
	for _, vector := range texts {
		if got := answer(utf8Length, vector); got != want(vector) {
			t.Errorf("the reference answers %s with %s", vector.Name, got)
		}
	}
	for _, fault := range []struct {
		name    string
		compute measure
	}{
		{"scalar count", scalarCount},
		{"UTF-16 units", utf16Units},
		{"graphemes", graphemeCount},
		{"raw bytes of invalid text", rawBytes},
	} {
		killed := false
		for _, vector := range texts {
			if answer(fault.compute, vector) != want(vector) {
				killed = true
				break
			}
		}
		if !killed {
			t.Errorf("no vector tells the %s fault apart", fault.name)
		}
	}
}
