package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

func TestQuotedRoundtrip(t *testing.T) {
	suite, err := admitRunInput(suiteJSON)
	if err != nil {
		t.Fatal(err)
	}
	var document struct {
		Scenarios map[string]struct {
			Steps []struct {
				Expectation struct{ Predicate json.RawMessage }
			}
		}
	}
	if err := json.Unmarshal([]byte(suite.original), &document); err != nil {
		t.Fatal(err)
	}
	raw, err := os.ReadFile("expected.json")
	if err != nil {
		t.Fatal(err)
	}
	var expected []string
	if err := json.Unmarshal(raw, &expected); err != nil {
		t.Fatal(err)
	}
	steps := document.Scenarios["quoted.core/authored/operand"].Steps
	if len(steps) != len(expected) {
		t.Fatal("missing cases")
	}
	for index, step := range steps {
		parsed, err := parsePredicate(step.Expectation.Predicate)
		if err != nil {
			t.Fatal(err)
		}
		if parsed.right.literal != expected[index] {
			t.Fatalf("case%d: %#v != %#v", index, parsed.right.literal, expected[index])
		}
		if parsed.evaluate(factSource{"to": expected[index]}) != truthTrue {
			t.Fatalf("case%d does not match", index)
		}
	}
	t.Logf("%d actual Rust-emitted literals preserved", len(expected))
}

func TestQuotedVersionAuthority(t *testing.T) {
	suite, err := admitRunInput(suiteJSON)
	if err != nil {
		t.Fatal(err)
	}
	var original map[string]any
	if err := json.Unmarshal([]byte(suite.original), &original); err != nil {
		t.Fatal(err)
	}
	if referenceFor(suite)["version"] != "ess-conformance/9" {
		t.Fatal("wrong parent version")
	}
	for _, major := range []string{"ess-conformance/5", "ess-conformance/7"} {
		original["provenance"].(map[string]any)["suite_version"] = major
		raw, _ := json.Marshal(original)
		if _, err := admitSuite(string(raw)); err == nil {
			t.Fatalf("accepted incompatible %s", major)
		}
	}
	for _, value := range []any{
		map[string]any{"to": map[string]any{"eq": "\"true\""}},
		map[string]any{"not": map[string]any{"to": map[string]any{"eq": "\"a.b\""}}},
		map[string]any{"forall": map[string]any{"in": "rows", "as": "row", "that": map[string]any{"to": map[string]any{"eq": "\"true\""}}}},
	} {
		if err := admitPredicateVersion(value, 7); err == nil {
			t.Fatalf("old admitted %#v", value)
		}
		if err := admitPredicateVersion(value, 8); err != nil {
			t.Fatal(err)
		}
	}
	for _, value := range []any{"to == \"a.b\"", map[string]any{"to": map[string]any{"eq": "\"busy\" status"}}} {
		if err := admitPredicateVersion(value, 4); err != nil {
			t.Fatal(err)
		}
	}
	t.Log("old/new version guard and actual coverage parent version checked")
}
