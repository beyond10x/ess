package essconform

import (
	"encoding/json"
	"os"
	"strings"
	"testing"
)

// The explicit fact operand `{fact: <path>}` (docs/design/expression-family-source22.md, A1): the
// shared vectors `crates/specify/ess-primitives/tests/vectors/root-fact-operand.json`, copied
// beside this file, answered by the Go reader as `tests/root_fact_operand.rs` answers them in
// Rust and `predicate.test.ts` in TypeScript.
func TestRootFactOperand(t *testing.T) {
	raw, err := os.ReadFile("root-fact-operand.json")
	if err != nil {
		t.Fatalf("vectors: %v", err)
	}
	var vectors struct {
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
	names := map[truth]string{truthTrue: "true", truthFalse: "false", truthUnknown: "unknown"}
	for _, vector := range vectors.Evaluate {
		parsed, err := parsePredicate(vector.Predicate)
		if err != nil {
			t.Errorf("%s: %v", vector.Name, err)
			continue
		}
		var row Row
		decoder := json.NewDecoder(strings.NewReader(string(vector.Row)))
		decoder.UseNumber()
		if err := decoder.Decode(&row); err != nil {
			t.Fatalf("%s: row: %v", vector.Name, err)
		}
		if got := names[parsed.evaluate(facts(row))]; got != vector.Truth {
			t.Errorf("%s: got %s, want %s", vector.Name, got, vector.Truth)
		}
	}
	for _, vector := range vectors.Refused {
		if _, err := parsePredicate(vector.Predicate); err == nil {
			t.Errorf("%s: read, want refused", vector.Name)
		}
	}
	if len(vectors.Evaluate) == 0 || len(vectors.Refused) == 0 {
		t.Fatalf("no vectors were read")
	}
}
