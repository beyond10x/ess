package essconform

import (
	"encoding/json"
	"os"
	"strings"
	"testing"
)

// The RFC 3339 instant grammar and the comparison tagged to compare instants
// (docs/design/expression-family-source22.md, decisions 2 and 14): the shared vectors
// `crates/specify/ess-primitives/tests/vectors/rfc3339-instants.json`, copied beside this file,
// answered by the Go reader as `tests/instant_comparison.rs` answers them in Rust and
// `predicate.test.ts` in TypeScript.
func TestInstantComparison(t *testing.T) {
	raw, err := os.ReadFile("rfc3339-instants.json")
	if err != nil {
		t.Fatalf("vectors: %v", err)
	}
	var vectors struct {
		Parse []struct {
			Text  string `json:"text"`
			Valid bool   `json:"valid"`
		} `json:"parse"`
		Order []struct {
			Left     string `json:"left"`
			Right    string `json:"right"`
			Ordering string `json:"ordering"`
		} `json:"order"`
		Tagged []struct {
			Name      string          `json:"name"`
			Predicate json.RawMessage `json:"predicate"`
			Row       json.RawMessage `json:"row"`
			Truth     string          `json:"truth"`
		} `json:"tagged"`
		Refused []struct {
			Name      string          `json:"name"`
			Predicate json.RawMessage `json:"predicate"`
		} `json:"refused"`
	}
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatalf("vectors: %v", err)
	}
	for _, vector := range vectors.Parse {
		if _, ok := parseInstant(vector.Text); ok != vector.Valid {
			t.Errorf("parse %q: got %v, want %v", vector.Text, ok, vector.Valid)
		}
	}
	orderings := map[string]int{"less": -1, "equal": 0, "greater": 1}
	for _, vector := range vectors.Order {
		left, okLeft := parseInstant(vector.Left)
		right, okRight := parseInstant(vector.Right)
		if !okLeft || !okRight || left.compare(right) != orderings[vector.Ordering] {
			t.Errorf("order %s %s: want %s", vector.Left, vector.Right, vector.Ordering)
		}
	}
	names := map[truth]string{truthTrue: "true", truthFalse: "false", truthUnknown: "unknown"}
	for _, vector := range vectors.Tagged {
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
	if len(vectors.Parse) == 0 || len(vectors.Tagged) == 0 {
		t.Fatalf("no vectors were read")
	}
}
