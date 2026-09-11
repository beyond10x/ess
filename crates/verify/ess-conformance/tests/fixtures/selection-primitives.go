package essconform

import (
	"encoding/json"
	"math"
	"testing"
)

var suiteJSON = `{}`

func TestSelectionPrimitiveValues(t *testing.T) {
	cases := []struct {
		kind string
		good Node
		bad  Node
	}{
		{"String", "text", true},
		{"Boolean", true, "true"},
		{"Integer", json.Number("42"), json.Number("1.5")},
		{"Decimal", json.Number("1.5"), "1.5"},
		{"Decimal", 1.5, math.Inf(1)},
		{"Decimal", json.Number("1.5"), math.NaN()},
		{"Timestamp", "2026-09-11T10:00:00Z", false},
		{"Duration", "PT12S", false},
		{"Uuid", "123e4567-e89b-12d3-a456-426614174000", "invalid"},
		{"Bytes", "YQ==", "invalid"},
		{"Binary64", 1.5, math.Inf(1)},
		{"Binary64", json.Number("1.5"), math.NaN()},
	}
	for _, tc := range cases {
		t.Run(tc.kind, func(t *testing.T) {
			observation := selectionObservation{}
			budget := 0
			if err := observation.validateValue(tc.kind, tc.good, true, &budget, 0); err != nil {
				t.Fatalf("valid %s: %v", tc.kind, err)
			}
			if err := observation.validateValue(tc.kind, tc.bad, true, &budget, 0); err == nil {
				t.Fatalf("invalid %s admitted: %#v", tc.kind, tc.bad)
			}
			// A valid first candidate cannot hide a later wrongly typed list element.
			if err := observation.validateValue("List<"+tc.kind+">", []Node{tc.good, tc.bad}, true, &budget, 0); err == nil {
				t.Fatalf("invalid %s tail admitted", tc.kind)
			}
		})
	}
}
