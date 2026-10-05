package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"
)

// Distinct list members (docs/design/expression-family-source22.md, `distinct`): the shared vectors
// `crates/specify/ess-primitives/tests/vectors/distinct.json`, copied beside this file, answered by
// the Go reader as `tests/distinct.rs` answers them in Rust and `predicate.test.ts` in TypeScript.
// Each paired fault — neighbours only, `by` ignored, spellings compared, instants for text, Unknown
// read as distinct, one shared null, binary64 numbers, lexical decimals, any two accepted — must
// disagree with a row.

type distinctVector struct {
	Name      string          `json:"name"`
	Predicate json.RawMessage `json:"predicate"`
	Row       json.RawMessage `json:"row"`
	Truth     string          `json:"truth"`
}

func distinctVectors(t *testing.T) (evaluate, refused, unkinded []distinctVector) {
	raw, err := os.ReadFile("distinct.json")
	if err != nil {
		t.Fatalf("vectors: %v", err)
	}
	var vectors struct {
		Evaluate []distinctVector `json:"evaluate"`
		Refused  []distinctVector `json:"refused"`
		Unkinded []distinctVector `json:"unkinded"`
	}
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatalf("vectors: %v", err)
	}
	return vectors.Evaluate, vectors.Refused, vectors.Unkinded
}

// distinctNumbers decodes one document keeping every number's digits.
func distinctNumbers(t *testing.T, name string, raw []byte) any {
	var value any
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	if err := decoder.Decode(&value); err != nil {
		t.Fatalf("%s: %v", name, err)
	}
	return value
}

func TestDistinct(t *testing.T) {
	names := map[truth]string{truthTrue: "true", truthFalse: "false", truthUnknown: "unknown"}
	evaluate, refused, unkinded := distinctVectors(t)
	answered := 0
	for _, vector := range evaluate {
		parsed, err := parsePredicate(vector.Predicate)
		if err != nil {
			t.Errorf("%s: %v", vector.Name, err)
			continue
		}
		row, _ := distinctNumbers(t, vector.Name, vector.Row).(map[string]any)
		if got := names[parsed.evaluate(facts(Row(row)))]; got != vector.Truth {
			t.Errorf("%s: got %s, want %s", vector.Name, got, vector.Truth)
		}
		answered++
	}
	for _, vector := range append(refused, unkinded...) {
		if _, err := parsePredicate(vector.Predicate); err == nil {
			t.Errorf("%s: read, want refused", vector.Name)
		}
		answered++
	}
	if answered < 50 {
		t.Fatalf("only %d vectors were answered", answered)
	}
}

func TestDistinctAdmission(t *testing.T) {
	canonical := distinctNumbers(t, "canonical", []byte(`{"distinct": {"in": "files", "as": "file", "by": "file.path", "kind": "string"}}`))
	if err := admitPredicateVersion(canonical, 40); err != nil {
		t.Errorf("suite/40 admits distinct: %v", err)
	}
	err := admitPredicateVersion(canonical, 39)
	if err == nil || !strings.Contains(err.Error(), "suite/40 or /41") {
		t.Errorf("suite/39 refuses distinct naming /40: %v", err)
	}
	nested := distinctNumbers(t, "nested", []byte(`{"not": {"forall": {"in": "groups", "as": "g", "that": {"distinct": {"in": "g.tags", "as": "t", "kind": "string"}}}}}`))
	if err := admitPredicateVersion(nested, 39); err == nil {
		t.Errorf("suite/39 refuses distinct inside a quantifier body")
	}
	unkinded := distinctNumbers(t, "unkinded", []byte(`{"distinct": {"in": "tags", "as": "tag"}}`))
	if err := admitPredicateVersion(unkinded, 40); err == nil {
		t.Errorf("suite/40 refuses a key without its kind")
	}
	field := distinctNumbers(t, "field", []byte(`{"distinct": {"in": ["a", "b"]}}`))
	if err := admitPredicateVersion(field, 39); err != nil {
		t.Errorf("a fact named distinct keeps its meaning: %v", err)
	}
}

// ---- paired faults ------------------------------------------------------------------------------

// distinctRow is one top-level `distinct` vector as its row holds it.
type distinctRow struct {
	name     string
	keys     []any
	elements []any
	kind     string
	truth    string
}

func distinctRows(t *testing.T) []distinctRow {
	evaluate, _, _ := distinctVectors(t)
	var rows []distinctRow
	for _, vector := range evaluate {
		predicate, _ := distinctNumbers(t, vector.Name, vector.Predicate).(map[string]any)
		fields, ok := predicate["distinct"].(map[string]any)
		if !ok {
			continue
		}
		row, _ := distinctNumbers(t, vector.Name, vector.Row).(map[string]any)
		elements, ok := row[fields["in"].(string)].([]any)
		if !ok {
			continue
		}
		by, _ := fields["by"].(string)
		keys := make([]any, len(elements))
		for index, element := range elements {
			at := element
			if by != "" {
				for _, segment := range strings.Split(by, ".")[1:] {
					mapping, isMap := at.(map[string]any)
					if !isMap {
						at = nil
						break
					}
					at = mapping[segment]
				}
			}
			keys[index] = at
		}
		rows = append(rows, distinctRow{vector.Name, keys, elements, fields["kind"].(string), vector.Truth})
	}
	return rows
}

// spelled is each key under a kind, empty where it is no key.
func spelled(keys []any, kind string) []string {
	out := make([]string, len(keys))
	for index, key := range keys {
		if key == nil {
			continue
		}
		if text, ok := distinctKey(kind, key); ok {
			out[index] = "k:" + text
		}
	}
	return out
}

func pairwise(keys []string, adjacentOnly bool) string {
	if len(keys) < 2 {
		return "true"
	}
	unknown := false
	for left := range keys {
		for right := left + 1; right < len(keys); right++ {
			if adjacentOnly && right > left+1 {
				break
			}
			switch {
			case keys[left] == "" || keys[right] == "":
				unknown = true
			case keys[left] == keys[right]:
				return "false"
			}
		}
	}
	if unknown {
		return "unknown"
	}
	return "true"
}

type distinctJudge func(row distinctRow) string

func TestDistinctFaults(t *testing.T) {
	rows := distinctRows(t)
	if len(rows) < 30 {
		t.Fatalf("only %d rows", len(rows))
	}
	reference := func(row distinctRow) string { return pairwise(spelled(row.keys, row.kind), false) }
	for _, row := range rows {
		if got := reference(row); got != row.truth {
			t.Errorf("the reference answers %s with %s", row.name, got)
		}
	}
	whole := func(values []any) []string {
		out := make([]string, len(values))
		for index, value := range values {
			if value != nil {
				encoded, _ := json.Marshal(value)
				out[index] = "k:" + string(encoded)
			}
		}
		return out
	}
	faults := []struct {
		name  string
		judge distinctJudge
	}{
		{"neighbours only", func(row distinctRow) string { return pairwise(spelled(row.keys, row.kind), true) }},
		{"`by` ignored", func(row distinctRow) string { return pairwise(whole(row.elements), false) }},
		{"spellings compared", func(row distinctRow) string {
			if row.kind == "timestamp" {
				return pairwise(spelled(row.keys, "string"), false)
			}
			return reference(row)
		}},
		{"instants for text", func(row distinctRow) string {
			if row.kind == "string" {
				keys := spelled(row.keys, "timestamp")
				plain := spelled(row.keys, "string")
				for index := range keys {
					if keys[index] == "" {
						keys[index] = plain[index]
					}
				}
				return pairwise(keys, false)
			}
			return reference(row)
		}},
		{"Unknown read as distinct", func(row distinctRow) string {
			var known []string
			for _, key := range spelled(row.keys, row.kind) {
				if key != "" {
					known = append(known, key)
				}
			}
			return pairwise(known, false)
		}},
		{"one shared null", func(row distinctRow) string {
			keys := spelled(row.keys, row.kind)
			for index, key := range row.keys {
				if key == nil {
					keys[index] = "null"
				}
			}
			return pairwise(keys, false)
		}},
		{"binary64 numbers", func(row distinctRow) string {
			keys := spelled(row.keys, row.kind)
			if row.kind == "integer" || row.kind == "decimal" {
				for index, key := range row.keys {
					if number, ok := key.(json.Number); ok {
						value, _ := number.Float64()
						keys[index] = fmt.Sprint(value)
					}
				}
			}
			return pairwise(keys, false)
		}},
		{"lexical decimals", func(row distinctRow) string {
			keys := spelled(row.keys, row.kind)
			if row.kind == "decimal" {
				for index, key := range row.keys {
					if number, ok := key.(json.Number); ok {
						keys[index] = number.String()
					}
				}
			}
			return pairwise(keys, false)
		}},
		{"any two accepted", func(row distinctRow) string {
			if len(row.keys) == 2 {
				return "true"
			}
			return reference(row)
		}},
	}
	for _, fault := range faults {
		killed := false
		for _, row := range rows {
			if fault.judge(row) != row.truth {
				killed = true
				break
			}
		}
		if !killed {
			t.Errorf("no vector tells the %s fault apart", fault.name)
		}
	}
}
